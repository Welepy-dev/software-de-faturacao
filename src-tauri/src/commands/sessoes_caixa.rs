use crate::auth::AuthState;
use crate::domain::sessao_caixa::{SessaoCaixa, SessaoCaixaTotalPorMetodo};
use crate::permissoes;
use chrono::Utc;
use serde::Serialize;
use sqlx::SqlitePool;
use tauri::State;
use uuid::Uuid;

/// Abre uma sessão de caixa no estabelecimento indicado. Falha com uma
/// mensagem clara se já existir uma sessão aberta nesse estabelecimento —
/// a regra é imposta pelo índice único parcial na base de dados
/// (`idx_sessao_caixa_unica_aberta`), não só verificada aqui, para não
/// haver corrida entre dois pedidos simultâneos.
#[tauri::command]
pub async fn abrir_sessao_caixa(
    pool: State<'_, SqlitePool>,
    auth_state: State<'_, AuthState>,
    estabelecimento_id: String,
    fundo_maneio_inicial_centimos: i64,
) -> Result<SessaoCaixa, String> {
    let utilizador_id = auth_state.exigir_sessao_ativa()?;
    permissoes::exigir_permissao(pool.inner(), &utilizador_id, "caixa.abrir", None).await?;

    let id = Uuid::new_v4().to_string();
    let agora = Utc::now().to_rfc3339();

    let resultado = sqlx::query(
        r#"
        INSERT INTO sessoes_caixa
            (id, estabelecimento_id, aberta_por_utilizador_id, fundo_maneio_inicial_centimos, estado, aberta_em)
        VALUES (?, ?, ?, ?, 'aberta', ?)
        "#,
    )
    .bind(&id)
    .bind(&estabelecimento_id)
    .bind(&utilizador_id)
    .bind(fundo_maneio_inicial_centimos)
    .bind(&agora)
    .execute(pool.inner())
    .await;

    if let Err(sqlx::Error::Database(db_err)) = &resultado {
        if db_err.is_unique_violation() {
            return Err(
                "Já existe uma sessão de caixa aberta neste estabelecimento. \
                 Entre na sessão existente ou aguarde que seja fechada."
                    .to_string(),
            );
        }
    }
    resultado.map_err(|e| e.to_string())?;

    buscar_sessao(pool, id).await
}

/// Regista um pagamento e, na mesma transação, incrementa o total corrente
/// da sessão para o método de pagamento usado — é esta acumulação
/// incremental que torna o fecho de caixa automático (docs/ARQUITETURA.md §4.2).
#[tauri::command]
pub async fn registar_pagamento(
    pool: State<'_, SqlitePool>,
    auth_state: State<'_, AuthState>,
    documento_id: String,
    sessao_caixa_id: String,
    metodo_pagamento: String,
    valor_centimos: i64,
) -> Result<(), String> {
    let utilizador_id = auth_state.exigir_sessao_ativa()?;
    permissoes::exigir_permissao(pool.inner(), &utilizador_id, "pagamentos.registar", None)
        .await?;

    let mut tx = pool.inner().begin().await.map_err(|e| e.to_string())?;
    let id = Uuid::new_v4().to_string();
    let agora = Utc::now().to_rfc3339();

    sqlx::query(
        r#"
        INSERT INTO pagamentos
            (id, documento_id, sessao_caixa_id, utilizador_id, metodo_pagamento, valor_centimos, criado_em)
        VALUES (?, ?, ?, ?, ?, ?, ?)
        "#,
    )
    .bind(&id)
    .bind(&documento_id)
    .bind(&sessao_caixa_id)
    .bind(&utilizador_id)
    .bind(&metodo_pagamento)
    .bind(valor_centimos)
    .bind(&agora)
    .execute(&mut *tx)
    .await
    .map_err(|e| e.to_string())?;

    sqlx::query(
        r#"
        INSERT INTO sessao_caixa_totais_por_metodo
            (id, sessao_caixa_id, metodo_pagamento, total_acumulado_centimos)
        VALUES (?, ?, ?, ?)
        ON CONFLICT (sessao_caixa_id, metodo_pagamento)
        DO UPDATE SET total_acumulado_centimos = total_acumulado_centimos + excluded.total_acumulado_centimos
        "#,
    )
    .bind(Uuid::new_v4().to_string())
    .bind(&sessao_caixa_id)
    .bind(&metodo_pagamento)
    .bind(valor_centimos)
    .execute(&mut *tx)
    .await
    .map_err(|e| e.to_string())?;

    tx.commit().await.map_err(|e| e.to_string())?;
    Ok(())
}

/// Fecha a sessão: o total esperado já está pronto (acumulado
/// incrementalmente), só falta a contagem física de dinheiro para calcular
/// a diferença — nenhum cálculo manual do funcionário. Ação sensível:
/// exige `caixa.fechar` com reautenticação.
#[tauri::command]
pub async fn fechar_sessao_caixa(
    pool: State<'_, SqlitePool>,
    auth_state: State<'_, AuthState>,
    sessao_caixa_id: String,
    contagem_fisica_dinheiro_centimos: i64,
    password_confirmacao: String,
) -> Result<SessaoCaixa, String> {
    let utilizador_id = auth_state.exigir_sessao_ativa()?;
    permissoes::exigir_permissao(
        pool.inner(),
        &utilizador_id,
        "caixa.fechar",
        Some(&password_confirmacao),
    )
    .await?;

    let totais = sqlx::query_as::<_, SessaoCaixaTotalPorMetodo>(
        "SELECT * FROM sessao_caixa_totais_por_metodo WHERE sessao_caixa_id = ?",
    )
    .bind(&sessao_caixa_id)
    .fetch_all(pool.inner())
    .await
    .map_err(|e| e.to_string())?;

    let total_dinheiro_centimos: i64 = totais
        .iter()
        .find(|t| t.metodo_pagamento == "dinheiro")
        .map(|t| t.total_acumulado_centimos)
        .unwrap_or(0);

    // O operador conta a gaveta inteira, por isso o esperado inclui o
    // fundo de maneio com que a sessão abriu, não só o recebido.
    let fundo_maneio_centimos: i64 = sqlx::query_scalar(
        "SELECT fundo_maneio_inicial_centimos FROM sessoes_caixa WHERE id = ?",
    )
    .bind(&sessao_caixa_id)
    .fetch_one(pool.inner())
    .await
    .map_err(|e| e.to_string())?;

    let diferenca =
        contagem_fisica_dinheiro_centimos - (fundo_maneio_centimos + total_dinheiro_centimos);
    let agora = Utc::now().to_rfc3339();

    sqlx::query(
        r#"
        UPDATE sessoes_caixa
        SET estado = 'fechada',
            fechada_por_utilizador_id = ?,
            fechada_em = ?,
            contagem_fisica_dinheiro_centimos = ?,
            diferenca_calculada_centimos = ?
        WHERE id = ? AND estado = 'aberta'
        "#,
    )
    .bind(&utilizador_id)
    .bind(&agora)
    .bind(contagem_fisica_dinheiro_centimos)
    .bind(diferenca)
    .bind(&sessao_caixa_id)
    .execute(pool.inner())
    .await
    .map_err(|e| e.to_string())?;

    buscar_sessao(pool, sessao_caixa_id).await
}

/// Devolve a sessão aberta do estabelecimento, se existir — a UI usa isto
/// para decidir se mostra "abrir sessão" ou "continuar/fechar sessão".
#[tauri::command]
pub async fn obter_sessao_aberta(
    pool: State<'_, SqlitePool>,
    estabelecimento_id: String,
) -> Result<Option<SessaoCaixa>, String> {
    sqlx::query_as::<_, SessaoCaixa>(
        "SELECT * FROM sessoes_caixa WHERE estabelecimento_id = ? AND estado = 'aberta'",
    )
    .bind(&estabelecimento_id)
    .fetch_optional(pool.inner())
    .await
    .map_err(|e| e.to_string())
}

#[derive(Debug, Serialize)]
pub struct TotalMetodo {
    pub metodo_pagamento: String,
    pub total_centimos: i64,
    pub pagamentos: i64,
}

#[derive(Debug, Serialize)]
pub struct ResumoCaixa {
    pub sessao_aberta: Option<SessaoCaixa>,
    pub aberta_por_nome: Option<String>,
    /// Lidos de `sessao_caixa_totais_por_metodo` (acumulado incremental) —
    /// a contagem de pagamentos é só informativa.
    pub totais: Vec<TotalMetodo>,
    pub ultima_fechada: Option<SessaoCaixa>,
    pub fechada_por_nome: Option<String>,
    /// Totais da última sessão fechada, para mostrar esperado vs. contado.
    pub totais_ultima_fechada: Vec<TotalMetodo>,
}

async fn totais_da_sessao(pool: &SqlitePool, sessao_id: &str) -> Result<Vec<TotalMetodo>, String> {
    let linhas: Vec<(String, i64, i64)> = sqlx::query_as(
        r#"
        SELECT t.metodo_pagamento, t.total_acumulado_centimos,
               (SELECT COUNT(*) FROM pagamentos p
                WHERE p.sessao_caixa_id = t.sessao_caixa_id AND p.metodo_pagamento = t.metodo_pagamento)
        FROM sessao_caixa_totais_por_metodo t
        WHERE t.sessao_caixa_id = ?
        "#,
    )
    .bind(sessao_id)
    .fetch_all(pool)
    .await
    .map_err(|e| e.to_string())?;
    Ok(linhas
        .into_iter()
        .map(|(metodo_pagamento, total_centimos, pagamentos)| TotalMetodo {
            metodo_pagamento,
            total_centimos,
            pagamentos,
        })
        .collect())
}

async fn nome_utilizador(pool: &SqlitePool, id: &str) -> Result<Option<String>, String> {
    sqlx::query_scalar("SELECT nome FROM utilizadores WHERE id = ?")
        .bind(id)
        .fetch_optional(pool)
        .await
        .map_err(|e| e.to_string())
}

/// Estado da caixa de um estabelecimento para o ecrã "Caixa" e para os
/// cartões de escolha de estabelecimento: sessão aberta com os totais
/// correntes por método, e o último fecho.
#[tauri::command]
pub async fn obter_resumo_caixa(
    pool: State<'_, SqlitePool>,
    estabelecimento_id: String,
) -> Result<ResumoCaixa, String> {
    let pool = pool.inner();
    let sessao_aberta: Option<SessaoCaixa> = sqlx::query_as(
        "SELECT * FROM sessoes_caixa WHERE estabelecimento_id = ? AND estado = 'aberta'",
    )
    .bind(&estabelecimento_id)
    .fetch_optional(pool)
    .await
    .map_err(|e| e.to_string())?;

    let ultima_fechada: Option<SessaoCaixa> = sqlx::query_as(
        r#"
        SELECT * FROM sessoes_caixa
        WHERE estabelecimento_id = ? AND estado = 'fechada'
        ORDER BY fechada_em DESC LIMIT 1
        "#,
    )
    .bind(&estabelecimento_id)
    .fetch_optional(pool)
    .await
    .map_err(|e| e.to_string())?;

    let (aberta_por_nome, totais) = match &sessao_aberta {
        Some(s) => (
            nome_utilizador(pool, &s.aberta_por_utilizador_id).await?,
            totais_da_sessao(pool, &s.id).await?,
        ),
        None => (None, Vec::new()),
    };
    let (fechada_por_nome, totais_ultima_fechada) = match &ultima_fechada {
        Some(s) => (
            match &s.fechada_por_utilizador_id {
                Some(id) => nome_utilizador(pool, id).await?,
                None => None,
            },
            totais_da_sessao(pool, &s.id).await?,
        ),
        None => (None, Vec::new()),
    };

    Ok(ResumoCaixa {
        sessao_aberta,
        aberta_por_nome,
        totais,
        ultima_fechada,
        fechada_por_nome,
        totais_ultima_fechada,
    })
}

async fn buscar_sessao(pool: State<'_, SqlitePool>, id: String) -> Result<SessaoCaixa, String> {
    sqlx::query_as::<_, SessaoCaixa>("SELECT * FROM sessoes_caixa WHERE id = ?")
        .bind(&id)
        .fetch_one(pool.inner())
        .await
        .map_err(|e| e.to_string())
}
