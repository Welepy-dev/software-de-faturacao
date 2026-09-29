use crate::auth::AuthState;
use crate::domain::empresa::Empresa;
use crate::permissoes;
use chrono::Utc;
use sqlx::SqlitePool;
use tauri::State;
use uuid::Uuid;

#[tauri::command]
pub async fn listar_empresas(pool: State<'_, SqlitePool>) -> Result<Vec<Empresa>, String> {
    sqlx::query_as::<_, Empresa>("SELECT * FROM empresas ORDER BY nome")
        .fetch_all(pool.inner())
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn criar_empresa(
    pool: State<'_, SqlitePool>,
    nome: String,
    nif: String,
    morada: Option<String>,
    moeda: String,
    perfil_complexidade: String,
) -> Result<Empresa, String> {
    let id = Uuid::new_v4().to_string();
    let agora = Utc::now().to_rfc3339();

    sqlx::query(
        r#"
        INSERT INTO empresas (id, nome, nif, morada, moeda, perfil_complexidade, criado_em, atualizado_em)
        VALUES (?, ?, ?, ?, ?, ?, ?, ?)
        "#,
    )
    .bind(&id)
    .bind(&nome)
    .bind(&nif)
    .bind(&morada)
    .bind(&moeda)
    .bind(&perfil_complexidade)
    .bind(&agora)
    .bind(&agora)
    .execute(pool.inner())
    .await
    .map_err(|e| e.to_string())?;

    sqlx::query_as::<_, Empresa>("SELECT * FROM empresas WHERE id = ?")
        .bind(&id)
        .fetch_one(pool.inner())
        .await
        .map_err(|e| e.to_string())
}

/// Dados da empresa (incluindo nome/NIF/morada, cobertos pela excepção do
/// artigo 8º nº8 — docs/AGT-SAFT.md §0) ficam sempre editáveis, mesmo com
/// documentos já emitidos: como não são copiados para os documentos (são
/// lidos ao vivo a partir do estabelecimento), corrigir aqui corrige
/// automaticamente a identificação do emitente em qualquer reimpressão
/// futura, sem precisar de nota de crédito.
#[tauri::command]
pub async fn atualizar_empresa(
    pool: State<'_, SqlitePool>,
    auth_state: State<'_, AuthState>,
    empresa_id: String,
    nome: String,
    nif: String,
    morada: Option<String>,
    moeda: String,
    perfil_complexidade: String,
    password_confirmacao: String,
) -> Result<Empresa, String> {
    let utilizador_id = auth_state.exigir_sessao_ativa()?;
    permissoes::exigir_permissao(
        pool.inner(),
        &utilizador_id,
        "configuracao.alterar",
        Some(&password_confirmacao),
    )
    .await?;

    let agora = Utc::now().to_rfc3339();

    sqlx::query(
        r#"
        UPDATE empresas SET
            nome = ?, nif = ?, morada = ?, moeda = ?, perfil_complexidade = ?, atualizado_em = ?
        WHERE id = ?
        "#,
    )
    .bind(&nome)
    .bind(&nif)
    .bind(&morada)
    .bind(&moeda)
    .bind(&perfil_complexidade)
    .bind(&agora)
    .bind(&empresa_id)
    .execute(pool.inner())
    .await
    .map_err(|e| e.to_string())?;

    sqlx::query_as::<_, Empresa>("SELECT * FROM empresas WHERE id = ?")
        .bind(&empresa_id)
        .fetch_one(pool.inner())
        .await
        .map_err(|e| e.to_string())
}
