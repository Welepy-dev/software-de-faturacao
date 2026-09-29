use crate::domain::estabelecimento::Estabelecimento;
use sqlx::SqlitePool;
use tauri::State;

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
