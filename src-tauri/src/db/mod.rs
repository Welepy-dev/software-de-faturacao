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

#[cfg(test)]
mod tests {
    use sqlx::sqlite::SqlitePoolOptions;
    use sqlx::SqlitePool;
    use uuid::Uuid;

    /// Base de dados em memória com o schema aplicado — sem passar pelo
    /// `AppHandle` do Tauri, que não existe fora de uma app a correr.
    async fn pool_teste() -> SqlitePool {
        // max_connections(1): uma BD ":memory:" é por ligação — com mais
        // do que uma ligação no pool, cada uma veria uma BD vazia diferente.
        let pool = SqlitePoolOptions::new()
            .max_connections(1)
            .connect("sqlite::memory:")
            .await
            .expect("ligar à BD em memória");
        sqlx::migrate!("./migrations")
            .run(&pool)
            .await
            .expect("aplicar migrations");
        pool
    }

    /// Cria empresa + estabelecimento + utilizador mínimos para os testes
    /// abaixo poderem inserir sessões/pagamentos. Devolve
    /// (estabelecimento_id, utilizador_id).
    async fn seed_minimo(pool: &SqlitePool) -> (String, String) {
        let empresa_id = Uuid::new_v4().to_string();
        let estabelecimento_id = Uuid::new_v4().to_string();
        let utilizador_id = Uuid::new_v4().to_string();
        let agora = "2026-01-01T00:00:00Z";

        sqlx::query(
            "INSERT INTO empresas (id, nome, nif, moeda, perfil_complexidade, criado_em, atualizado_em) VALUES (?, 'Empresa Teste', '123', 'AOA', 'simples', ?, ?)",
        )
        .bind(&empresa_id)
        .bind(agora)
        .bind(agora)
        .execute(pool)
        .await
        .unwrap();

        sqlx::query(
            "INSERT INTO estabelecimentos (id, empresa_id, nome, ativo, criado_em) VALUES (?, ?, 'Estabelecimento Teste', 1, ?)",
        )
        .bind(&estabelecimento_id)
        .bind(&empresa_id)
        .bind(agora)
        .execute(pool)
        .await
        .unwrap();

        sqlx::query(
            "INSERT INTO utilizadores (id, nome, email, password_hash, password_alterada_em, ativo, criado_em) VALUES (?, 'Operador Teste', 'op@teste.local', 'hash', ?, 1, ?)",
        )
        .bind(&utilizador_id)
        .bind(agora)
        .bind(agora)
        .execute(pool)
        .await
        .unwrap();

        (estabelecimento_id, utilizador_id)
    }

    /// Problema 1 (docs/ARQUITETURA.md §4.1): nunca pode haver duas
    /// sessões de caixa abertas para o mesmo estabelecimento — imposto
    /// pelo índice único parcial `idx_sessao_caixa_unica_aberta`.
    #[tokio::test]
    async fn nao_permite_duas_sessoes_abertas_no_mesmo_estabelecimento() {
        let pool = pool_teste().await;
        let (estabelecimento_id, utilizador_id) = seed_minimo(&pool).await;

        let inserir_sessao = |id: String| {
            let pool = pool.clone();
            let estabelecimento_id = estabelecimento_id.clone();
            let utilizador_id = utilizador_id.clone();
            async move {
                sqlx::query(
                    "INSERT INTO sessoes_caixa (id, estabelecimento_id, aberta_por_utilizador_id, estado, aberta_em) VALUES (?, ?, ?, 'aberta', '2026-01-01T08:00:00Z')",
                )
                .bind(id)
                .bind(estabelecimento_id)
                .bind(utilizador_id)
                .execute(&pool)
                .await
            }
        };

        let primeira = inserir_sessao(Uuid::new_v4().to_string()).await;
        assert!(primeira.is_ok(), "a primeira sessão deve abrir sem erro");

        let segunda = inserir_sessao(Uuid::new_v4().to_string()).await;
        assert!(
            segunda.is_err(),
            "uma segunda sessão aberta no mesmo estabelecimento deve ser rejeitada"
        );
        match segunda.unwrap_err() {
            sqlx::Error::Database(db_err) => assert!(db_err.is_unique_violation()),
            outro => panic!("esperava violação de unicidade, obteve: {outro:?}"),
        }

        // Depois de fechar a primeira, uma nova sessão já deve poder abrir.
        sqlx::query("UPDATE sessoes_caixa SET estado = 'fechada' WHERE estabelecimento_id = ?")
            .bind(&estabelecimento_id)
            .execute(&pool)
            .await
            .unwrap();

        let terceira = inserir_sessao(Uuid::new_v4().to_string()).await;
        assert!(
            terceira.is_ok(),
            "depois de fechar a sessão anterior, deve ser possível abrir uma nova"
        );
    }

    /// Problema 2 (docs/ARQUITETURA.md §4.2): o total por método de
    /// pagamento acumula-se com cada pagamento, na mesma transação —
    /// nunca é recalculado do zero.
    #[tokio::test]
    async fn acumula_totais_por_metodo_incrementalmente() {
        let pool = pool_teste().await;
        let (estabelecimento_id, utilizador_id) = seed_minimo(&pool).await;

        let sessao_id = Uuid::new_v4().to_string();
        sqlx::query(
            "INSERT INTO sessoes_caixa (id, estabelecimento_id, aberta_por_utilizador_id, estado, aberta_em) VALUES (?, ?, ?, 'aberta', '2026-01-01T08:00:00Z')",
        )
        .bind(&sessao_id)
        .bind(&estabelecimento_id)
        .bind(&utilizador_id)
        .execute(&pool)
        .await
        .unwrap();

        async fn acumular(pool: &SqlitePool, sessao_id: &str, metodo: &str, valor_centimos: i64) {
            sqlx::query(
                r#"
                INSERT INTO sessao_caixa_totais_por_metodo (id, sessao_caixa_id, metodo_pagamento, total_acumulado_centimos)
                VALUES (?, ?, ?, ?)
                ON CONFLICT (sessao_caixa_id, metodo_pagamento)
                DO UPDATE SET total_acumulado_centimos = total_acumulado_centimos + excluded.total_acumulado_centimos
                "#,
            )
            .bind(Uuid::new_v4().to_string())
            .bind(sessao_id)
            .bind(metodo)
            .bind(valor_centimos)
            .execute(pool)
            .await
            .unwrap();
        }

        acumular(&pool, &sessao_id, "dinheiro", 5_000).await;
        acumular(&pool, &sessao_id, "dinheiro", 2_500).await;
        acumular(&pool, &sessao_id, "multicaixa", 10_000).await;

        let total_dinheiro: i64 = sqlx::query_scalar(
            "SELECT total_acumulado_centimos FROM sessao_caixa_totais_por_metodo WHERE sessao_caixa_id = ? AND metodo_pagamento = 'dinheiro'",
        )
        .bind(&sessao_id)
        .fetch_one(&pool)
        .await
        .unwrap();
        assert_eq!(total_dinheiro, 7_500);

        let total_multicaixa: i64 = sqlx::query_scalar(
            "SELECT total_acumulado_centimos FROM sessao_caixa_totais_por_metodo WHERE sessao_caixa_id = ? AND metodo_pagamento = 'multicaixa'",
        )
        .bind(&sessao_id)
        .fetch_one(&pool)
        .await
        .unwrap();
        assert_eq!(total_multicaixa, 10_000);
    }
}
