use crate::auth::AuthState;
use crate::domain::produto_servico::ProdutoServico;
use crate::permissoes;
use chrono::Utc;
use sqlx::SqlitePool;
use tauri::State;
use uuid::Uuid;

#[tauri::command]
pub async fn listar_produtos_servicos(
    pool: State<'_, SqlitePool>,
) -> Result<Vec<ProdutoServico>, String> {
    sqlx::query_as::<_, ProdutoServico>(
        "SELECT * FROM produtos_servicos WHERE ativo = 1 ORDER BY nome",
    )
    .fetch_all(pool.inner())
    .await
    .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn criar_produto_servico(
    pool: State<'_, SqlitePool>,
    auth_state: State<'_, AuthState>,
    empresa_id: String,
    nome: String,
    descricao: Option<String>,
    tipo: String,
    unidade_medida: String,
    preco_base_centimos: i64,
    taxa_imposto_id: Option<String>,
    controla_stock: bool,
) -> Result<ProdutoServico, String> {
    let utilizador_id = auth_state.exigir_sessao_ativa()?;
    permissoes::exigir_permissao(pool.inner(), &utilizador_id, "produtos.gerir", None).await?;

    let id = Uuid::new_v4().to_string();
    let agora = Utc::now().to_rfc3339();

    sqlx::query(
        r#"
        INSERT INTO produtos_servicos
            (id, empresa_id, nome, descricao, tipo, unidade_medida, preco_base_centimos, taxa_imposto_id, controla_stock, ativo, criado_em)
        VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, 1, ?)
        "#,
    )
    .bind(&id)
    .bind(&empresa_id)
    .bind(&nome)
    .bind(&descricao)
    .bind(&tipo)
    .bind(&unidade_medida)
    .bind(preco_base_centimos)
    .bind(&taxa_imposto_id)
    .bind(controla_stock)
    .bind(&agora)
    .execute(pool.inner())
    .await
    .map_err(|e| e.to_string())?;

    sqlx::query_as::<_, ProdutoServico>("SELECT * FROM produtos_servicos WHERE id = ?")
        .bind(&id)
        .fetch_one(pool.inner())
        .await
        .map_err(|e| e.to_string())
}

/// Atualiza um produto/serviço. A descrição (`descricao`) não é usada
/// diretamente nos documentos — cada linha de documento guarda a sua
/// própria `descricao` como snapshot (docs/ARQUITETURA.md §2.10), por
/// isso editar aqui nunca afecta documentos já emitidos.
#[tauri::command]
pub async fn atualizar_produto_servico(
    pool: State<'_, SqlitePool>,
    auth_state: State<'_, AuthState>,
    produto_id: String,
    nome: String,
    descricao: Option<String>,
    unidade_medida: String,
    preco_base_centimos: i64,
    taxa_imposto_id: Option<String>,
    controla_stock: bool,
) -> Result<ProdutoServico, String> {
    let utilizador_id = auth_state.exigir_sessao_ativa()?;
    permissoes::exigir_permissao(pool.inner(), &utilizador_id, "produtos.gerir", None).await?;

    sqlx::query(
        r#"
        UPDATE produtos_servicos SET
            nome = ?, descricao = ?, unidade_medida = ?, preco_base_centimos = ?,
            taxa_imposto_id = ?, controla_stock = ?
        WHERE id = ?
        "#,
    )
    .bind(&nome)
    .bind(&descricao)
    .bind(&unidade_medida)
    .bind(preco_base_centimos)
    .bind(&taxa_imposto_id)
    .bind(controla_stock)
    .bind(&produto_id)
    .execute(pool.inner())
    .await
    .map_err(|e| e.to_string())?;

    sqlx::query_as::<_, ProdutoServico>("SELECT * FROM produtos_servicos WHERE id = ?")
        .bind(&produto_id)
        .fetch_one(pool.inner())
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn desativar_produto_servico(
    pool: State<'_, SqlitePool>,
    auth_state: State<'_, AuthState>,
    produto_id: String,
) -> Result<(), String> {
    let utilizador_id = auth_state.exigir_sessao_ativa()?;
    permissoes::exigir_permissao(pool.inner(), &utilizador_id, "produtos.gerir", None).await?;

    sqlx::query("UPDATE produtos_servicos SET ativo = 0 WHERE id = ?")
        .bind(&produto_id)
        .execute(pool.inner())
        .await
        .map_err(|e| e.to_string())?;
    Ok(())
}
