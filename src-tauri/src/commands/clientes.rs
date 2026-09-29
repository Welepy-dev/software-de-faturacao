use crate::auth::AuthState;
use crate::domain::cliente::Cliente;
use crate::permissoes;
use chrono::Utc;
use sqlx::SqlitePool;
use tauri::State;
use uuid::Uuid;

const NIF_GENERICO: &str = "999999999";

#[tauri::command]
pub async fn listar_clientes(pool: State<'_, SqlitePool>) -> Result<Vec<Cliente>, String> {
    sqlx::query_as::<_, Cliente>("SELECT * FROM clientes WHERE ativo = 1 ORDER BY nome")
        .fetch_all(pool.inner())
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn criar_cliente(
    pool: State<'_, SqlitePool>,
    auth_state: State<'_, AuthState>,
    empresa_id: String,
    nome: String,
    email: Option<String>,
    telefone: Option<String>,
    morada: Option<String>,
    nif: Option<String>,
) -> Result<Cliente, String> {
    let utilizador_id = auth_state.exigir_sessao_ativa()?;
    permissoes::exigir_permissao(pool.inner(), &utilizador_id, "clientes.gerir", None).await?;

    let id = Uuid::new_v4().to_string();
    let agora = Utc::now().to_rfc3339();

    sqlx::query(
        r#"
        INSERT INTO clientes (id, empresa_id, nome, email, telefone, morada, nif, ativo, criado_em)
        VALUES (?, ?, ?, ?, ?, ?, ?, 1, ?)
        "#,
    )
    .bind(&id)
    .bind(&empresa_id)
    .bind(&nome)
    .bind(&email)
    .bind(&telefone)
    .bind(&morada)
    .bind(&nif)
    .bind(&agora)
    .execute(pool.inner())
    .await
    .map_err(|e| e.to_string())?;

    sqlx::query_as::<_, Cliente>("SELECT * FROM clientes WHERE id = ?")
        .bind(&id)
        .fetch_one(pool.inner())
        .await
        .map_err(|e| e.to_string())
}

/// Atualiza um cliente, aplicando as regras do anexo técnico AGT
/// (docs/AGT-SAFT.md, Anexo I pontos 26-27) para clientes que já têm
/// documentos emitidos:
/// - o NIF só pode ser preenchido se estava vazio (nunca alterado depois
///   de definido, exceto para o genérico 999999999);
/// - o nome só pode ser alterado enquanto o NIF não estiver preenchido.
/// Sem documentos emitidos, o cliente edita-se livremente.
#[tauri::command]
pub async fn atualizar_cliente(
    pool: State<'_, SqlitePool>,
    auth_state: State<'_, AuthState>,
    cliente_id: String,
    nome: String,
    email: Option<String>,
    telefone: Option<String>,
    morada: Option<String>,
    nif: Option<String>,
) -> Result<Cliente, String> {
    let utilizador_id = auth_state.exigir_sessao_ativa()?;
    permissoes::exigir_permissao(pool.inner(), &utilizador_id, "clientes.gerir", None).await?;

    let atual: Cliente = sqlx::query_as("SELECT * FROM clientes WHERE id = ?")
        .bind(&cliente_id)
        .fetch_one(pool.inner())
        .await
        .map_err(|e| e.to_string())?;

    let tem_documentos: bool = sqlx::query_scalar(
        "SELECT EXISTS(SELECT 1 FROM documentos WHERE cliente_id = ? AND estado != 'rascunho')",
    )
    .bind(&cliente_id)
    .fetch_one(pool.inner())
    .await
    .map_err(|e| e.to_string())?;

    let (nome_final, nif_final) = if tem_documentos {
        let nif_final = match (&atual.nif, &nif) {
            (Some(anterior), _) if anterior != NIF_GENERICO => atual.nif.clone(),
            _ => nif.clone(),
        };
        let nome_final = if atual.nif.is_none() || atual.nif.as_deref() == Some(NIF_GENERICO) {
            nome
        } else {
            atual.nome.clone()
        };
        (nome_final, nif_final)
    } else {
        (nome, nif)
    };

    sqlx::query(
        "UPDATE clientes SET nome = ?, email = ?, telefone = ?, morada = ?, nif = ? WHERE id = ?",
    )
    .bind(&nome_final)
    .bind(&email)
    .bind(&telefone)
    .bind(&morada)
    .bind(&nif_final)
    .bind(&cliente_id)
    .execute(pool.inner())
    .await
    .map_err(|e| e.to_string())?;

    sqlx::query_as::<_, Cliente>("SELECT * FROM clientes WHERE id = ?")
        .bind(&cliente_id)
        .fetch_one(pool.inner())
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn desativar_cliente(
    pool: State<'_, SqlitePool>,
    auth_state: State<'_, AuthState>,
    cliente_id: String,
) -> Result<(), String> {
    let utilizador_id = auth_state.exigir_sessao_ativa()?;
    permissoes::exigir_permissao(pool.inner(), &utilizador_id, "clientes.gerir", None).await?;

    sqlx::query("UPDATE clientes SET ativo = 0 WHERE id = ?")
        .bind(&cliente_id)
        .execute(pool.inner())
        .await
        .map_err(|e| e.to_string())?;
    Ok(())
}
