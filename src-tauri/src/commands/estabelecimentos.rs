use crate::auth::AuthState;
use crate::domain::estabelecimento::Estabelecimento;
use crate::permissoes;
use chrono::Utc;
use sqlx::SqlitePool;
use tauri::State;
use uuid::Uuid;

#[tauri::command]
pub async fn listar_estabelecimentos(
    pool: State<'_, SqlitePool>,
) -> Result<Vec<Estabelecimento>, String> {
    sqlx::query_as::<_, Estabelecimento>(
        "SELECT * FROM estabelecimentos WHERE ativo = 1 ORDER BY nome",
    )
    .fetch_all(pool.inner())
    .await
    .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn criar_estabelecimento(
    pool: State<'_, SqlitePool>,
    auth_state: State<'_, AuthState>,
    empresa_id: String,
    nome: String,
    tipo_negocio: Option<String>,
    endereco: Option<String>,
    password_confirmacao: String,
) -> Result<Estabelecimento, String> {
    let utilizador_id = auth_state.exigir_sessao_ativa()?;
    permissoes::exigir_permissao(
        pool.inner(),
        &utilizador_id,
        "configuracao.alterar",
        Some(&password_confirmacao),
    )
    .await?;

    let id = Uuid::new_v4().to_string();
    let agora = Utc::now().to_rfc3339();

    sqlx::query(
        r#"
        INSERT INTO estabelecimentos (id, empresa_id, nome, tipo_negocio, endereco, ativo, criado_em)
        VALUES (?, ?, ?, ?, ?, 1, ?)
        "#,
    )
    .bind(&id)
    .bind(&empresa_id)
    .bind(&nome)
    .bind(&tipo_negocio)
    .bind(&endereco)
    .bind(&agora)
    .execute(pool.inner())
    .await
    .map_err(|e| e.to_string())?;

    sqlx::query_as::<_, Estabelecimento>("SELECT * FROM estabelecimentos WHERE id = ?")
        .bind(&id)
        .fetch_one(pool.inner())
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn atualizar_estabelecimento(
    pool: State<'_, SqlitePool>,
    auth_state: State<'_, AuthState>,
    estabelecimento_id: String,
    nome: String,
    tipo_negocio: Option<String>,
    endereco: Option<String>,
    password_confirmacao: String,
) -> Result<Estabelecimento, String> {
    let utilizador_id = auth_state.exigir_sessao_ativa()?;
    permissoes::exigir_permissao(
        pool.inner(),
        &utilizador_id,
        "configuracao.alterar",
        Some(&password_confirmacao),
    )
    .await?;

    sqlx::query(
        "UPDATE estabelecimentos SET nome = ?, tipo_negocio = ?, endereco = ? WHERE id = ?",
    )
    .bind(&nome)
    .bind(&tipo_negocio)
    .bind(&endereco)
    .bind(&estabelecimento_id)
    .execute(pool.inner())
    .await
    .map_err(|e| e.to_string())?;

    sqlx::query_as::<_, Estabelecimento>("SELECT * FROM estabelecimentos WHERE id = ?")
        .bind(&estabelecimento_id)
        .fetch_one(pool.inner())
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn desativar_estabelecimento(
    pool: State<'_, SqlitePool>,
    auth_state: State<'_, AuthState>,
    estabelecimento_id: String,
    password_confirmacao: String,
) -> Result<(), String> {
    let utilizador_id = auth_state.exigir_sessao_ativa()?;
    permissoes::exigir_permissao(
        pool.inner(),
        &utilizador_id,
        "configuracao.alterar",
        Some(&password_confirmacao),
    )
    .await?;

    sqlx::query("UPDATE estabelecimentos SET ativo = 0 WHERE id = ?")
        .bind(&estabelecimento_id)
        .execute(pool.inner())
        .await
        .map_err(|e| e.to_string())?;
    Ok(())
}
