use crate::auth::AuthState;
use crate::domain::documento::{Documento, LinhaDocumento, NovaLinhaDocumento};
use crate::emissao::{self, ResultadoEmissao};
use crate::permissoes;
use chrono::Utc;
use serde::Serialize;
use sqlx::SqlitePool;
use tauri::State;
use uuid::Uuid;

#[derive(Debug, Serialize)]
pub struct DocumentoCompleto {
    pub documento: Documento,
    pub linhas: Vec<LinhaDocumento>,
}

async fn obter_documento_completo(
    pool: &SqlitePool,
    documento_id: &str,
) -> Result<DocumentoCompleto, String> {
    let documento: Documento = sqlx::query_as("SELECT * FROM documentos WHERE id = ?")
        .bind(documento_id)
        .fetch_one(pool)
        .await
        .map_err(|e| e.to_string())?;

    let linhas: Vec<LinhaDocumento> =
        sqlx::query_as("SELECT * FROM linhas_documento WHERE documento_id = ? ORDER BY ordem")
            .bind(documento_id)
            .fetch_all(pool)
            .await
            .map_err(|e| e.to_string())?;

    Ok(DocumentoCompleto { documento, linhas })
}

#[tauri::command]
pub async fn obter_documento(
    pool: State<'_, SqlitePool>,
    documento_id: String,
) -> Result<DocumentoCompleto, String> {
    obter_documento_completo(pool.inner(), &documento_id).await
}

#[tauri::command]
pub async fn listar_documentos(
    pool: State<'_, SqlitePool>,
    estabelecimento_id: String,
) -> Result<Vec<Documento>, String> {
    sqlx::query_as::<_, Documento>(
        "SELECT * FROM documentos WHERE estabelecimento_id = ? ORDER BY criado_em DESC",
    )
    .bind(&estabelecimento_id)
    .fetch_all(pool.inner())
    .await
    .map_err(|e| e.to_string())
}

/// Cria um documento em rascunho — livremente editável/eliminável até ser
/// emitido (docs/ARQUITETURA.md §2.9 e §6).
#[tauri::command]
pub async fn criar_documento_rascunho(
    pool: State<'_, SqlitePool>,
    auth_state: State<'_, AuthState>,
    estabelecimento_id: String,
    cliente_id: Option<String>,
    tipo: String,
    documento_origem_id: Option<String>,
    observacoes: Option<String>,
    condicoes_pagamento: Option<String>,
    data_vencimento: Option<String>,
) -> Result<Documento, String> {
    let utilizador_id = auth_state.exigir_sessao_ativa()?;
    permissoes::exigir_permissao(pool.inner(), &utilizador_id, "documentos.criar", None).await?;

    let id = Uuid::new_v4().to_string();
    let agora = Utc::now().to_rfc3339();

    sqlx::query(
        r#"
        INSERT INTO documentos
            (id, estabelecimento_id, cliente_id, utilizador_emissor_id, tipo, estado,
             documento_origem_id, observacoes, condicoes_pagamento, data_vencimento, criado_em)
        VALUES (?, ?, ?, ?, ?, 'rascunho', ?, ?, ?, ?, ?)
        "#,
    )
    .bind(&id)
    .bind(&estabelecimento_id)
    .bind(&cliente_id)
    .bind(&utilizador_id)
    .bind(&tipo)
    .bind(&documento_origem_id)
    .bind(&observacoes)
    .bind(&condicoes_pagamento)
    .bind(&data_vencimento)
    .bind(&agora)
    .execute(pool.inner())
    .await
    .map_err(|e| e.to_string())?;

    sqlx::query_as::<_, Documento>("SELECT * FROM documentos WHERE id = ?")
        .bind(&id)
        .fetch_one(pool.inner())
        .await
        .map_err(|e| e.to_string())
}

async fn exigir_rascunho(pool: &SqlitePool, documento_id: &str) -> Result<(), String> {
    let estado: String = sqlx::query_scalar("SELECT estado FROM documentos WHERE id = ?")
        .bind(documento_id)
        .fetch_one(pool)
        .await
        .map_err(|e| e.to_string())?;
    if estado != "rascunho" {
        return Err(
            "Só é possível editar linhas de um documento em rascunho — este já foi emitido."
                .to_string(),
        );
    }
    Ok(())
}

#[tauri::command]
pub async fn adicionar_linha_documento(
    pool: State<'_, SqlitePool>,
    auth_state: State<'_, AuthState>,
    documento_id: String,
    linha: NovaLinhaDocumento,
) -> Result<DocumentoCompleto, String> {
    let utilizador_id = auth_state.exigir_sessao_ativa()?;
    permissoes::exigir_permissao(
        pool.inner(),
        &utilizador_id,
        "documentos.editar_rascunho",
        None,
    )
    .await?;
    exigir_rascunho(pool.inner(), &documento_id).await?;

    let proxima_ordem: i64 =
        sqlx::query_scalar("SELECT COALESCE(MAX(ordem), -1) + 1 FROM linhas_documento WHERE documento_id = ?")
            .bind(&documento_id)
            .fetch_one(pool.inner())
            .await
            .map_err(|e| e.to_string())?;

    let id = Uuid::new_v4().to_string();

    sqlx::query(
        r#"
        INSERT INTO linhas_documento
            (id, documento_id, produto_servico_id, descricao, quantidade, preco_unitario_centimos,
             desconto_centimos, taxa_imposto_percentagem, justificacao_isencao, ordem)
        VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
        "#,
    )
    .bind(&id)
    .bind(&documento_id)
    .bind(&linha.produto_servico_id)
    .bind(&linha.descricao)
    .bind(linha.quantidade)
    .bind(linha.preco_unitario_centimos)
    .bind(linha.desconto_centimos)
    .bind(linha.taxa_imposto_percentagem)
    .bind(&linha.justificacao_isencao)
    .bind(proxima_ordem)
    .execute(pool.inner())
    .await
    .map_err(|e| e.to_string())?;

    obter_documento_completo(pool.inner(), &documento_id).await
}

#[tauri::command]
pub async fn remover_linha_documento(
    pool: State<'_, SqlitePool>,
    auth_state: State<'_, AuthState>,
    documento_id: String,
    linha_id: String,
) -> Result<DocumentoCompleto, String> {
    let utilizador_id = auth_state.exigir_sessao_ativa()?;
    permissoes::exigir_permissao(
        pool.inner(),
        &utilizador_id,
        "documentos.editar_rascunho",
        None,
    )
    .await?;
    exigir_rascunho(pool.inner(), &documento_id).await?;

    sqlx::query("DELETE FROM linhas_documento WHERE id = ? AND documento_id = ?")
        .bind(&linha_id)
        .bind(&documento_id)
        .execute(pool.inner())
        .await
        .map_err(|e| e.to_string())?;

    obter_documento_completo(pool.inner(), &documento_id).await
}

/// Emite o documento — motor de numeração/assinatura em
/// `crate::emissao` (docs/ARQUITETURA.md §3.1).
#[tauri::command]
pub async fn emitir_documento(
    pool: State<'_, SqlitePool>,
    auth_state: State<'_, AuthState>,
    documento_id: String,
) -> Result<ResultadoEmissao, String> {
    let utilizador_id = auth_state.exigir_sessao_ativa()?;
    permissoes::exigir_permissao(pool.inner(), &utilizador_id, "documentos.criar", None).await?;

    emissao::emitir_documento(pool.inner(), &documento_id).await
}

/// Anula um documento emitido através de nota de crédito (caminho normal,
/// docs/ARQUITETURA.md §6): cria uma nota de crédito referenciando o
/// original, com o motivo obrigatório, copia as linhas do original
/// (crédito total — crédito parcial fica para trabalho futuro), emite-a
/// de imediato, e marca o original como anulado. Tudo isto exige a
/// permissão `documentos.anular` com reautenticação.
#[tauri::command]
pub async fn anular_documento(
    pool: State<'_, SqlitePool>,
    auth_state: State<'_, AuthState>,
    documento_id: String,
    motivo: String,
    password_confirmacao: String,
) -> Result<ResultadoEmissao, String> {
    let utilizador_id = auth_state.exigir_sessao_ativa()?;
    permissoes::exigir_permissao(
        pool.inner(),
        &utilizador_id,
        "documentos.anular",
        Some(&password_confirmacao),
    )
    .await?;

    if motivo.trim().is_empty() {
        return Err("O motivo da anulação/rectificação é obrigatório.".to_string());
    }

    let original: Documento = sqlx::query_as("SELECT * FROM documentos WHERE id = ?")
        .bind(&documento_id)
        .fetch_one(pool.inner())
        .await
        .map_err(|e| e.to_string())?;

    if original.estado == "rascunho" {
        return Err(
            "Documentos em rascunho eliminam-se diretamente — não precisam de nota de crédito."
                .to_string(),
        );
    }
    if original.estado == "anulado" {
        return Err("Este documento já foi anulado.".to_string());
    }
    if original.tipo == "nota_credito" {
        return Err("Não é possível anular uma nota de crédito.".to_string());
    }

    // já existe nota de crédito (total ou parcial) sobre este documento?
    let ja_retificado: bool = sqlx::query_scalar(
        "SELECT EXISTS(SELECT 1 FROM documentos WHERE documento_original_id = ? AND estado != 'anulado')",
    )
    .bind(&documento_id)
    .fetch_one(pool.inner())
    .await
    .map_err(|e| e.to_string())?;
    if ja_retificado {
        return Err(
            "Este documento já tem um documento retificativo — anule-o primeiro.".to_string(),
        );
    }

    let linhas_originais: Vec<LinhaDocumento> =
        sqlx::query_as("SELECT * FROM linhas_documento WHERE documento_id = ? ORDER BY ordem")
            .bind(&documento_id)
            .fetch_all(pool.inner())
            .await
            .map_err(|e| e.to_string())?;

    let nota_id = Uuid::new_v4().to_string();
    let agora = Utc::now().to_rfc3339();

    let mut tx = pool.inner().begin().await.map_err(|e| e.to_string())?;

    sqlx::query(
        r#"
        INSERT INTO documentos
            (id, estabelecimento_id, cliente_id, utilizador_emissor_id, tipo, estado,
             documento_original_id, motivo_anulacao_retificacao, criado_em)
        VALUES (?, ?, ?, ?, 'nota_credito', 'rascunho', ?, ?, ?)
        "#,
    )
    .bind(&nota_id)
    .bind(&original.estabelecimento_id)
    .bind(&original.cliente_id)
    .bind(&utilizador_id)
    .bind(&documento_id)
    .bind(&motivo)
    .bind(&agora)
    .execute(&mut *tx)
    .await
    .map_err(|e| e.to_string())?;

    for linha in &linhas_originais {
        let linha_id = Uuid::new_v4().to_string();
        sqlx::query(
            r#"
            INSERT INTO linhas_documento
                (id, documento_id, produto_servico_id, descricao, quantidade, preco_unitario_centimos,
                 desconto_centimos, taxa_imposto_percentagem, justificacao_isencao, ordem)
            VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
            "#,
        )
        .bind(&linha_id)
        .bind(&nota_id)
        .bind(&linha.produto_servico_id)
        .bind(&linha.descricao)
        .bind(linha.quantidade)
        .bind(linha.preco_unitario_centimos)
        .bind(linha.desconto_centimos)
        .bind(linha.taxa_imposto_percentagem)
        .bind(&linha.justificacao_isencao)
        .bind(linha.ordem)
        .execute(&mut *tx)
        .await
        .map_err(|e| e.to_string())?;
    }

    tx.commit().await.map_err(|e| e.to_string())?;

    let resultado = emissao::emitir_documento(pool.inner(), &nota_id).await?;

    sqlx::query("UPDATE documentos SET estado = 'anulado' WHERE id = ?")
        .bind(&documento_id)
        .execute(pool.inner())
        .await
        .map_err(|e| e.to_string())?;

    Ok(resultado)
}

/// Excepção do artigo 8º nº8 do Decreto 71/25 (docs/AGT-SAFT.md §0):
/// corrige só a identificação do adquirente (nome/NIF/morada) num
/// documento já emitido, sem nota de crédito. A identificação do
/// emitente corrige-se em `atualizar_empresa` — não é copiada para o
/// documento, por isso não precisa de acção equivalente aqui.
#[tauri::command]
pub async fn corrigir_identificacao_adquirente(
    pool: State<'_, SqlitePool>,
    auth_state: State<'_, AuthState>,
    documento_id: String,
    nome: Option<String>,
    nif: Option<String>,
    morada: Option<String>,
    password_confirmacao: String,
) -> Result<Documento, String> {
    let utilizador_id = auth_state.exigir_sessao_ativa()?;
    permissoes::exigir_permissao(
        pool.inner(),
        &utilizador_id,
        "documentos.corrigir_identificacao",
        Some(&password_confirmacao),
    )
    .await?;

    let estado: String = sqlx::query_scalar("SELECT estado FROM documentos WHERE id = ?")
        .bind(&documento_id)
        .fetch_one(pool.inner())
        .await
        .map_err(|e| e.to_string())?;
    if estado == "rascunho" {
        return Err(
            "Documentos em rascunho editam-se normalmente — esta ação é só para documentos já emitidos."
                .to_string(),
        );
    }

    sqlx::query(
        r#"
        UPDATE documentos SET
            cliente_nome_snapshot = COALESCE(?, cliente_nome_snapshot),
            cliente_nif_snapshot = COALESCE(?, cliente_nif_snapshot),
            cliente_morada_snapshot = COALESCE(?, cliente_morada_snapshot)
        WHERE id = ?
        "#,
    )
    .bind(&nome)
    .bind(&nif)
    .bind(&morada)
    .bind(&documento_id)
    .execute(pool.inner())
    .await
    .map_err(|e| e.to_string())?;

    sqlx::query_as::<_, Documento>("SELECT * FROM documentos WHERE id = ?")
        .bind(&documento_id)
        .fetch_one(pool.inner())
        .await
        .map_err(|e| e.to_string())
}
