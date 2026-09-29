use sqlx::SqlitePool;

/// Verifica se o utilizador tem a permissão indicada (por código, ex.
/// "caixa.fechar"), através dos papéis que lhe estão atribuídos
/// (docs/ARQUITETURA.md §2.4 — permissões por ação/módulo, não papéis
/// fixos).
pub async fn utilizador_tem_permissao(
    pool: &SqlitePool,
    utilizador_id: &str,
    codigo_permissao: &str,
) -> Result<bool, sqlx::Error> {
    let existe: Option<i64> = sqlx::query_scalar(
        r#"
        SELECT 1
        FROM utilizador_papeis up
        JOIN papel_permissoes pp ON pp.papel_id = up.papel_id
        JOIN permissoes p ON p.id = pp.permissao_id
        WHERE up.utilizador_id = ? AND p.codigo = ?
        LIMIT 1
        "#,
    )
    .bind(utilizador_id)
    .bind(codigo_permissao)
    .fetch_optional(pool)
    .await?;

    Ok(existe.is_some())
}

async fn permissao_requer_reautenticacao(
    pool: &SqlitePool,
    codigo_permissao: &str,
) -> Result<bool, sqlx::Error> {
    let requer: Option<bool> =
        sqlx::query_scalar("SELECT requer_reautenticacao FROM permissoes WHERE codigo = ?")
            .bind(codigo_permissao)
            .fetch_optional(pool)
            .await?;
    Ok(requer.unwrap_or(false))
}

/// Porta única para comandos sensíveis: confirma que o utilizador tem a
/// permissão e, se essa permissão exigir reautenticação
/// (`Permissao.requer_reautenticacao`), valida a password recebida do
/// pedido — nunca confia numa reautenticação "recente" guardada em
/// memória, a password é pedida em cada acção sensível
/// (docs/ARQUITETURA.md §5).
pub async fn exigir_permissao(
    pool: &SqlitePool,
    utilizador_id: &str,
    codigo_permissao: &str,
    password_confirmacao: Option<&str>,
) -> Result<(), String> {
    let tem = utilizador_tem_permissao(pool, utilizador_id, codigo_permissao)
        .await
        .map_err(|e| e.to_string())?;
    if !tem {
        return Err(format!(
            "Sem permissão para esta ação ({codigo_permissao})."
        ));
    }

    let requer_reauth = permissao_requer_reautenticacao(pool, codigo_permissao)
        .await
        .map_err(|e| e.to_string())?;

    if requer_reauth {
        let senha = password_confirmacao
            .ok_or_else(|| "Esta ação exige confirmação da password.".to_string())?;
        let hash: String =
            sqlx::query_scalar("SELECT password_hash FROM utilizadores WHERE id = ?")
                .bind(utilizador_id)
                .fetch_one(pool)
                .await
                .map_err(|e| e.to_string())?;
        crate::auth::verificar_password(senha, &hash)?;
    }

    Ok(())
}
