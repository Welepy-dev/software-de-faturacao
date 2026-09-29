//! Motor de emissão de documentos — a "máquina de faturas" de
//! docs/ARQUITETURA.md §3.1: transição atómica rascunho → emitido, que
//! consome o número da série, calcula totais, assina em cadeia e tira o
//! snapshot dos dados do cliente. Tudo dentro de uma única transação SQL
//! (tudo ou nada), para nunca "queimar" um número por uma falha a meio.

use crate::domain::documento::LinhaDocumento;
use crate::util::numero_por_extenso::valor_por_extenso;
use chrono::Utc;
use serde::Serialize;
use sha2::{Digest, Sha256};
use sqlx::SqlitePool;
use uuid::Uuid;

pub struct TotaisDocumento {
    pub subtotal_centimos: i64,
    pub total_descontos_centimos: i64,
    pub total_impostos_centimos: i64,
    pub total_centimos: i64,
}

/// Calcula os totais a partir das linhas. Soma em `f64` (mais de 2 casas
/// decimais de precisão intermédia, conforme docs/AGT-SAFT.md §5) e só
/// arredonda a cêntimos no fim, nunca linha a linha.
pub fn calcular_totais(linhas: &[LinhaDocumento]) -> TotaisDocumento {
    let mut subtotal = 0f64;
    let mut descontos = 0f64;
    let mut impostos = 0f64;

    for linha in linhas {
        let bruto = linha.preco_unitario_centimos as f64 * linha.quantidade;
        let desconto = linha.desconto_centimos as f64;
        let base_tributavel = bruto - desconto;
        let imposto = base_tributavel * linha.taxa_imposto_percentagem / 100.0;

        subtotal += bruto;
        descontos += desconto;
        impostos += imposto;
    }

    let subtotal_centimos = subtotal.round() as i64;
    let total_descontos_centimos = descontos.round() as i64;
    let total_impostos_centimos = impostos.round() as i64;
    let total_centimos = subtotal_centimos - total_descontos_centimos + total_impostos_centimos;

    TotaisDocumento {
        subtotal_centimos,
        total_descontos_centimos,
        total_impostos_centimos,
        total_centimos,
    }
}

/// Prefixo por omissão para uma série nova, quando o utilizador ainda não
/// configurou um prefixo próprio — só uma sigla curta e legível.
fn prefixo_padrao_para_tipo(tipo: &str) -> &'static str {
    match tipo {
        "orcamento" => "ORC",
        "proforma" => "PRO",
        "encomenda" => "ENC",
        "guia_remessa" => "GR",
        "fatura" => "FT",
        "fatura_recibo" => "FR",
        "recibo" => "RC",
        "nota_credito" => "NC",
        _ => "DOC",
    }
}

/// Documentos que, segundo docs/AGT-SAFT.md §3, não precisam de assinatura
/// em cadeia — só do carimbo textual (tratado na camada de impressão).
fn tipo_isento_de_assinatura(tipo: &str) -> bool {
    tipo == "recibo"
}

fn compor_invoice_no(codigo: &str, prefixo: &str, numero: i64) -> String {
    format!("{codigo} {prefixo}/{numero}")
}

/// Assinatura placeholder (SHA-256 da mensagem, em base-64) — **não é a
/// assinatura RSA exigida pela AGT**, cujo algoritmo exacto ainda não
/// está confirmado (docs/AGT-SAFT.md §3). Mantém a mensagem, a cadeia por
/// série/tipo e o formato de armazenamento correctos, para trocar só esta
/// função por RSA real assim que o algoritmo for confirmado.
fn assinar_placeholder(mensagem: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(mensagem.as_bytes());
    base64_simples(&hasher.finalize())
}

fn base64_simples(bytes: &[u8]) -> String {
    const TABELA: &[u8; 64] =
        b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let mut saida = String::new();
    for grupo in bytes.chunks(3) {
        let b0 = grupo[0] as u32;
        let b1 = *grupo.get(1).unwrap_or(&0) as u32;
        let b2 = *grupo.get(2).unwrap_or(&0) as u32;
        let n = (b0 << 16) | (b1 << 8) | b2;
        saida.push(TABELA[((n >> 18) & 0x3F) as usize] as char);
        saida.push(TABELA[((n >> 12) & 0x3F) as usize] as char);
        saida.push(if grupo.len() > 1 {
            TABELA[((n >> 6) & 0x3F) as usize] as char
        } else {
            '='
        });
        saida.push(if grupo.len() > 2 {
            TABELA[(n & 0x3F) as usize] as char
        } else {
            '='
        });
    }
    saida
}

#[derive(Debug, Serialize)]
pub struct ResultadoEmissao {
    pub numero: i64,
    pub ano_economico: i64,
    pub prefixo: String,
    pub hash: Option<String>,
    pub codigo_certificacao: Option<String>,
    pub versao_chave_assinatura: Option<i64>,
    pub subtotal_centimos: i64,
    pub total_descontos_centimos: i64,
    pub total_impostos_centimos: i64,
    pub total_centimos: i64,
    pub valor_por_extenso: String,
}

/// Emite um documento em rascunho: consome o número da série, calcula
/// totais e assinatura, tira o snapshot do cliente, e grava tudo numa
/// única transação. Falha (sem efeitos) se o documento não estiver em
/// rascunho ou não tiver linhas.
pub async fn emitir_documento(pool: &SqlitePool, documento_id: &str) -> Result<ResultadoEmissao, String> {
    let mut tx = pool.begin().await.map_err(|e| e.to_string())?;

    let (estabelecimento_id, cliente_id, tipo, estado, moeda): (
        String,
        Option<String>,
        String,
        String,
        String,
    ) = sqlx::query_as(
        r#"
        SELECT d.estabelecimento_id, d.cliente_id, d.tipo, d.estado, e.moeda
        FROM documentos d
        JOIN estabelecimentos est ON est.id = d.estabelecimento_id
        JOIN empresas e ON e.id = est.empresa_id
        WHERE d.id = ?
        "#,
    )
    .bind(documento_id)
    .fetch_one(&mut *tx)
    .await
    .map_err(|e| e.to_string())?;

    if estado != "rascunho" {
        return Err("Só é possível emitir documentos em estado de rascunho.".to_string());
    }

    let linhas: Vec<LinhaDocumento> =
        sqlx::query_as("SELECT * FROM linhas_documento WHERE documento_id = ? ORDER BY ordem")
            .bind(documento_id)
            .fetch_all(&mut *tx)
            .await
            .map_err(|e| e.to_string())?;

    if linhas.is_empty() {
        return Err(
            "Documento sem linhas — adicione pelo menos um artigo antes de emitir.".to_string(),
        );
    }

    let totais = calcular_totais(&linhas);

    let ano_economico: i64 = Utc::now().format("%Y").to_string().parse().unwrap();

    let serie_existente: Option<(String, String, i64)> = sqlx::query_as(
        r#"
        SELECT id, prefixo, proximo_numero FROM series_documentais
        WHERE estabelecimento_id = ? AND tipo_documento = ? AND ano_economico = ?
        "#,
    )
    .bind(&estabelecimento_id)
    .bind(&tipo)
    .bind(ano_economico)
    .fetch_optional(&mut *tx)
    .await
    .map_err(|e| e.to_string())?;

    let (serie_id, prefixo, numero) = match serie_existente {
        Some((id, prefixo, proximo_numero)) => {
            sqlx::query(
                "UPDATE series_documentais SET proximo_numero = proximo_numero + 1 WHERE id = ?",
            )
            .bind(&id)
            .execute(&mut *tx)
            .await
            .map_err(|e| e.to_string())?;
            (id, prefixo, proximo_numero)
        }
        None => {
            let id = Uuid::new_v4().to_string();
            let prefixo_padrao = prefixo_padrao_para_tipo(&tipo).to_string();
            sqlx::query(
                r#"
                INSERT INTO series_documentais
                    (id, estabelecimento_id, tipo_documento, prefixo, ano_economico, proximo_numero)
                VALUES (?, ?, ?, ?, ?, 2)
                "#,
            )
            .bind(&id)
            .bind(&estabelecimento_id)
            .bind(&tipo)
            .bind(&prefixo_padrao)
            .bind(ano_economico)
            .execute(&mut *tx)
            .await
            .map_err(|e| e.to_string())?;
            (id, prefixo_padrao, 1i64)
        }
    };

    let (hash, codigo_certificacao, versao_chave_assinatura) = if tipo_isento_de_assinatura(&tipo)
    {
        (None, None, None)
    } else {
        let hash_anterior: Option<String> = sqlx::query_scalar(
            r#"
            SELECT hash FROM documentos
            WHERE estabelecimento_id = ? AND tipo = ? AND serie_documental_id = ? AND estado = 'emitido'
            ORDER BY numero DESC LIMIT 1
            "#,
        )
        .bind(&estabelecimento_id)
        .bind(&tipo)
        .bind(&serie_id)
        .fetch_optional(&mut *tx)
        .await
        .map_err(|e| e.to_string())?;

        let config: Option<(String, i64)> = sqlx::query_as(
            "SELECT numero_certificado_agt, versao_chave FROM configuracao_certificacao_agt ORDER BY criado_em DESC LIMIT 1",
        )
        .fetch_optional(&mut *tx)
        .await
        .map_err(|e| e.to_string())?;

        let codigo: Option<String> =
            sqlx::query_scalar("SELECT codigo_agt FROM codigos_documento_agt WHERE tipo_documento = ?")
                .bind(&tipo)
                .fetch_optional(&mut *tx)
                .await
                .map_err(|e| e.to_string())?;
        let codigo = codigo.unwrap_or_else(|| tipo.to_uppercase());

        let invoice_no = compor_invoice_no(&codigo, &prefixo, numero);
        let agora = Utc::now();
        let data = agora.format("%Y-%m-%d").to_string();
        let system_entry = agora.to_rfc3339();
        let total_str = format!("{:.2}", totais.total_centimos as f64 / 100.0);

        let mensagem = format!(
            "{};{};{};{};{}",
            data,
            system_entry,
            invoice_no,
            total_str,
            hash_anterior.unwrap_or_default(),
        );

        let assinatura = assinar_placeholder(&mensagem);
        match config {
            Some((numero_cert, versao)) => (Some(assinatura), Some(numero_cert), Some(versao)),
            None => (Some(assinatura), None, None),
        }
    };

    let (cliente_nome, cliente_nif, cliente_morada) = match &cliente_id {
        Some(id) => {
            let dados: Option<(String, Option<String>, Option<String>)> = sqlx::query_as(
                "SELECT nome, nif, morada FROM clientes WHERE id = ?",
            )
            .bind(id)
            .fetch_optional(&mut *tx)
            .await
            .map_err(|e| e.to_string())?;
            match dados {
                Some((nome, nif, morada)) => (Some(nome), nif, morada),
                None => (None, None, None),
            }
        }
        None => (None, None, None),
    };

    let agora_iso = Utc::now().to_rfc3339();

    sqlx::query(
        r#"
        UPDATE documentos SET
            estado = 'emitido',
            serie_documental_id = ?,
            numero = ?,
            ano_economico = ?,
            prefixo = ?,
            hash = ?,
            codigo_certificacao = ?,
            versao_chave_assinatura = ?,
            cliente_nome_snapshot = ?,
            cliente_nif_snapshot = ?,
            cliente_morada_snapshot = ?,
            subtotal_centimos = ?,
            total_descontos_centimos = ?,
            total_impostos_centimos = ?,
            total_centimos = ?,
            emitido_em = ?
        WHERE id = ?
        "#,
    )
    .bind(&serie_id)
    .bind(numero)
    .bind(ano_economico)
    .bind(&prefixo)
    .bind(&hash)
    .bind(&codigo_certificacao)
    .bind(versao_chave_assinatura)
    .bind(&cliente_nome)
    .bind(&cliente_nif)
    .bind(&cliente_morada)
    .bind(totais.subtotal_centimos)
    .bind(totais.total_descontos_centimos)
    .bind(totais.total_impostos_centimos)
    .bind(totais.total_centimos)
    .bind(&agora_iso)
    .bind(documento_id)
    .execute(&mut *tx)
    .await
    .map_err(|e| e.to_string())?;

    tx.commit().await.map_err(|e| e.to_string())?;

    let (moeda_singular, moeda_plural) = nomes_moeda(&moeda);
    let extenso = valor_por_extenso(
        totais.total_centimos,
        moeda_singular,
        moeda_plural,
        "cêntimo",
        "cêntimos",
    );

    Ok(ResultadoEmissao {
        numero,
        ano_economico,
        prefixo,
        hash,
        codigo_certificacao,
        versao_chave_assinatura,
        subtotal_centimos: totais.subtotal_centimos,
        total_descontos_centimos: totais.total_descontos_centimos,
        total_impostos_centimos: totais.total_impostos_centimos,
        total_centimos: totais.total_centimos,
        valor_por_extenso: extenso,
    })
}

fn nomes_moeda(codigo: &str) -> (&'static str, &'static str) {
    match codigo {
        "AOA" => ("kwanza", "kwanzas"),
        _ => ("unidade", "unidades"),
    }
}
