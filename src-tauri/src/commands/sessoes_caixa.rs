use crate::auth::AuthState;
use crate::domain::sessao_caixa::{SessaoCaixa, SessaoCaixaTotalPorMetodo};
use crate::permissoes;
use chrono::Utc;
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

    let diferenca = contagem_fisica_dinheiro_centimos - total_dinheiro_centimos;
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

async fn buscar_sessao(pool: State<'_, SqlitePool>, id: String) -> Result<SessaoCaixa, String> {
    sqlx::query_as::<_, SessaoCaixa>("SELECT * FROM sessoes_caixa WHERE id = ?")
        .bind(&id)
        .fetch_one(pool.inner())
        .await
        .map_err(|e| e.to_string())
}
