mod commands;
mod db;
mod domain;

use tauri::Manager;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .setup(|app| {
            let app_handle = app.handle().clone();
            let pool = tauri::async_runtime::block_on(db::init_pool(&app_handle))?;
            app.manage(pool);
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::estabelecimentos::listar_estabelecimentos,
            commands::sessoes_caixa::abrir_sessao_caixa,
            commands::sessoes_caixa::registar_pagamento,
            commands::sessoes_caixa::fechar_sessao_caixa,
            commands::sessoes_caixa::obter_sessao_aberta,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
