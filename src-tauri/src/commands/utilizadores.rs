use crate::auth::{self, AuthState};
use crate::domain::utilizador::UtilizadorPublico;
use crate::permissoes;
use chrono::Utc;
use sqlx::SqlitePool;
use tauri::State;
use uuid::Uuid;

/// Bootstrap: só funciona enquanto não existir nenhum utilizador — cria o
/// primeiro operador já como Administrador, sem exigir permissão (não há
/// ninguém com permissão nenhuma ainda). Depois disto, usar
/// `criar_utilizador`.
#[tauri::command]
pub async fn criar_primeiro_utilizador(
    pool: State<'_, SqlitePool>,
    nome: String,
    email: String,
    password: String,
) -> Result<UtilizadorPublico, String> {
    let existentes: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM utilizadores")
        .fetch_one(pool.inner())
        .await
        .map_err(|e| e.to_string())?;

    if existentes > 0 {
        return Err(
            "Já existe pelo menos um utilizador — use a criação normal de utilizadores."
                .to_string(),
        );
    }

    criar_utilizador_interno(pool.inner(), &nome, &email, &password, "papel-administrador").await
}

/// Cria um novo utilizador (ação sensível: exige permissão
/// `utilizadores.gerir` + password de confirmação).
#[tauri::command]
pub async fn criar_utilizador(
    pool: State<'_, SqlitePool>,
    auth_state: State<'_, AuthState>,
    nome: String,
    email: String,
    password: String,
    papel_id: String,
    password_confirmacao: String,
) -> Result<UtilizadorPublico, String> {
    let utilizador_id = auth_state.exigir_sessao_ativa()?;
    permissoes::exigir_permissao(
        pool.inner(),
        &utilizador_id,
        "utilizadores.gerir",
        Some(&password_confirmacao),
    )
    .await?;

    criar_utilizador_interno(pool.inner(), &nome, &email, &password, &papel_id).await
}

async fn criar_utilizador_interno(
    pool: &SqlitePool,
    nome: &str,
    email: &str,
    password: &str,
    papel_id: &str,
) -> Result<UtilizadorPublico, String> {
    let id = Uuid::new_v4().to_string();
    let agora = Utc::now().to_rfc3339();
    let hash = auth::gerar_hash_password(password)?;

    let mut tx = pool.begin().await.map_err(|e| e.to_string())?;

    sqlx::query(
        r#"
        INSERT INTO utilizadores (id, nome, email, password_hash, password_alterada_em, ativo, criado_em)
        VALUES (?, ?, ?, ?, ?, 1, ?)
        "#,
    )
    .bind(&id)
    .bind(nome)
    .bind(email)
    .bind(&hash)
    .bind(&agora)
    .bind(&agora)
    .execute(&mut *tx)
    .await
    .map_err(|e| e.to_string())?;

    sqlx::query("INSERT INTO utilizador_papeis (utilizador_id, papel_id) VALUES (?, ?)")
        .bind(&id)
        .bind(papel_id)
        .execute(&mut *tx)
        .await
        .map_err(|e| e.to_string())?;

    tx.commit().await.map_err(|e| e.to_string())?;

    Ok(UtilizadorPublico {
        id,
        nome: nome.to_string(),
        email: email.to_string(),
        ativo: true,
    })
}

#[tauri::command]
pub async fn listar_utilizadores(
    pool: State<'_, SqlitePool>,
    auth_state: State<'_, AuthState>,
    password_confirmacao: String,
) -> Result<Vec<UtilizadorPublico>, String> {
    let utilizador_id = auth_state.exigir_sessao_ativa()?;
    permissoes::exigir_permissao(
        pool.inner(),
        &utilizador_id,
        "utilizadores.gerir",
        Some(&password_confirmacao),
    )
    .await?;

    sqlx::query_as::<_, UtilizadorPublico>(
        "SELECT id, nome, email, ativo FROM utilizadores ORDER BY nome",
    )
    .fetch_all(pool.inner())
    .await
    .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn desativar_utilizador(
    pool: State<'_, SqlitePool>,
    auth_state: State<'_, AuthState>,
    utilizador_id_alvo: String,
    password_confirmacao: String,
) -> Result<(), String> {
    let utilizador_id = auth_state.exigir_sessao_ativa()?;
    permissoes::exigir_permissao(
        pool.inner(),
        &utilizador_id,
        "utilizadores.gerir",
        Some(&password_confirmacao),
    )
    .await?;

    sqlx::query("UPDATE utilizadores SET ativo = 0 WHERE id = ?")
        .bind(&utilizador_id_alvo)
        .execute(pool.inner())
        .await
        .map_err(|e| e.to_string())?;
    Ok(())
}
