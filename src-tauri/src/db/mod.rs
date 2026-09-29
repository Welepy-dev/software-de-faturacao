use sqlx::sqlite::{SqliteConnectOptions, SqlitePool, SqlitePoolOptions};
use std::str::FromStr;
use tauri::{AppHandle, Manager};

/// Abre (criando se necessário) a base de dados SQLite local no diretório de
/// dados da aplicação e aplica as migrations pendentes.
pub async fn init_pool(app: &AppHandle) -> Result<SqlitePool, Box<dyn std::error::Error>> {
    let app_dir = app.path().app_data_dir()?;
    std::fs::create_dir_all(&app_dir)?;
    let db_path = app_dir.join("faturacao.db");

    let options = SqliteConnectOptions::from_str(&format!("sqlite://{}", db_path.display()))?
        .create_if_missing(true)
        .foreign_keys(true);

    let pool = SqlitePoolOptions::new()
        .max_connections(5)
        .connect_with(options)
        .await?;

    // Caminho relativo a CARGO_MANIFEST_DIR (src-tauri/), não a este ficheiro.
    sqlx::migrate!("./migrations").run(&pool).await?;

    Ok(pool)
}
