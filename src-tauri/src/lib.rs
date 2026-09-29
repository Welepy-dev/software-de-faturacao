mod auth;
mod commands;
mod db;
mod domain;
mod emissao;
mod permissoes;
mod util;

use auth::AuthState;
use tauri::Manager;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .manage(AuthState::default())
        .setup(|app| {
            let app_handle = app.handle().clone();
            let pool = tauri::async_runtime::block_on(db::init_pool(&app_handle))?;
            app.manage(pool);
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            // Autenticação
            commands::auth::login,
            commands::auth::logout,
            commands::auth::tocar_atividade,
            commands::auth::estado_sessao,
            commands::auth::desbloquear_sessao,
            // Utilizadores
            commands::utilizadores::criar_primeiro_utilizador,
            commands::utilizadores::criar_utilizador,
            commands::utilizadores::listar_utilizadores,
            commands::utilizadores::desativar_utilizador,
            // Empresas
            commands::empresas::listar_empresas,
            commands::empresas::criar_empresa,
            commands::empresas::atualizar_empresa,
            // Estabelecimentos
            commands::estabelecimentos::listar_estabelecimentos,
            commands::estabelecimentos::criar_estabelecimento,
            commands::estabelecimentos::atualizar_estabelecimento,
            commands::estabelecimentos::desativar_estabelecimento,
            // Clientes
            commands::clientes::listar_clientes,
            commands::clientes::criar_cliente,
            commands::clientes::atualizar_cliente,
            commands::clientes::desativar_cliente,
            // Produtos/Serviços
            commands::produtos_servicos::listar_produtos_servicos,
            commands::produtos_servicos::criar_produto_servico,
            commands::produtos_servicos::atualizar_produto_servico,
            commands::produtos_servicos::desativar_produto_servico,
            // Documentos
            commands::documentos::obter_documento,
            commands::documentos::listar_documentos,
            commands::documentos::criar_documento_rascunho,
            commands::documentos::adicionar_linha_documento,
            commands::documentos::remover_linha_documento,
            commands::documentos::emitir_documento,
            commands::documentos::anular_documento,
            commands::documentos::corrigir_identificacao_adquirente,
            // Sessões de caixa
            commands::sessoes_caixa::abrir_sessao_caixa,
            commands::sessoes_caixa::registar_pagamento,
            commands::sessoes_caixa::fechar_sessao_caixa,
            commands::sessoes_caixa::obter_sessao_aberta,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
