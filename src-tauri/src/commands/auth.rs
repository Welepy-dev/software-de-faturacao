use crate::auth::{self, AuthState, SessaoUtilizador};
use crate::domain::utilizador::{EstadoSessao, UtilizadorPublico};
use sqlx::SqlitePool;
use std::time::Instant;
use tauri::State;

#[derive(sqlx::FromRow)]
struct UtilizadorComHash {
    id: String,
    nome: String,
    email: String,
    password_hash: String,
    ativo: bool,
}

/// Login: autentica o operador para o turno inteiro (docs/ARQUITETURA.md
/// §5) — não é preciso voltar a autenticar por cada ação normal.
#[tauri::command]
pub async fn login(
    pool: State<'_, SqlitePool>,
    auth_state: State<'_, AuthState>,
    email: String,
    password: String,
) -> Result<UtilizadorPublico, String> {
    let utilizador = sqlx::query_as::<_, UtilizadorComHash>(
        "SELECT id, nome, email, password_hash, ativo FROM utilizadores WHERE email = ?",
    )
    .bind(&email)
    .fetch_optional(pool.inner())
    .await
    .map_err(|e| e.to_string())?
    .ok_or_else(|| "Credenciais inválidas.".to_string())?;

    if !utilizador.ativo {
        return Err("Utilizador inativo.".to_string());
    }

    auth::verificar_password(&password, &utilizador.password_hash)?;

    let mut guard = auth_state.0.lock().unwrap();
    *guard = Some(SessaoUtilizador {
        utilizador_id: utilizador.id.clone(),
        nome: utilizador.nome.clone(),
        email: utilizador.email.clone(),
        ultima_atividade: Instant::now(),
        bloqueada: false,
    });
    drop(guard);

    Ok(UtilizadorPublico {
        id: utilizador.id,
        nome: utilizador.nome,
        email: utilizador.email,
        ativo: utilizador.ativo,
    })
}

#[tauri::command]
pub fn logout(auth_state: State<'_, AuthState>) -> Result<(), String> {
    let mut guard = auth_state.0.lock().unwrap();
    *guard = None;
    Ok(())
}

/// Chamado periodicamente pelo frontend a cada interação do operador —
/// mantém a sessão "viva" e devolve se entretanto bloqueou por
/// inatividade (45 min, docs/ARQUITETURA.md §5).
#[tauri::command]
pub fn tocar_atividade(auth_state: State<'_, AuthState>) -> Result<Option<EstadoSessao>, String> {
    auth_state.verificar_bloqueio();
    auth_state.tocar_atividade();
    estado_sessao(auth_state)
}

#[tauri::command]
pub fn estado_sessao(auth_state: State<'_, AuthState>) -> Result<Option<EstadoSessao>, String> {
    auth_state.verificar_bloqueio();
    let guard = auth_state.0.lock().unwrap();
    Ok(guard.as_ref().map(|s| EstadoSessao {
        utilizador_id: s.utilizador_id.clone(),
        nome: s.nome.clone(),
        bloqueada: s.bloqueada,
    }))
}

/// Bloqueio manual (botão "Bloquear" / F12) — mesmo efeito do bloqueio por
/// inatividade: o turno fica, só o mesmo operador desbloqueia.
#[tauri::command]
pub fn bloquear_sessao(auth_state: State<'_, AuthState>) -> Result<(), String> {
    let mut guard = auth_state.0.lock().unwrap();
    if let Some(sessao) = guard.as_mut() {
        sessao.bloqueada = true;
    }
    Ok(())
}

/// Desbloqueia a sessão do mesmo operador com a password — não é um
/// login novo, o turno continua onde estava.
#[tauri::command]
pub async fn desbloquear_sessao(
    pool: State<'_, SqlitePool>,
    auth_state: State<'_, AuthState>,
    password: String,
) -> Result<(), String> {
    let utilizador_id = {
        let guard = auth_state.0.lock().unwrap();
        guard
            .as_ref()
            .map(|s| s.utilizador_id.clone())
            .ok_or_else(|| "Sem sessão activa.".to_string())?
    };

    let hash: String = sqlx::query_scalar("SELECT password_hash FROM utilizadores WHERE id = ?")
        .bind(&utilizador_id)
        .fetch_one(pool.inner())
        .await
        .map_err(|e| e.to_string())?;

    auth::verificar_password(&password, &hash)?;

    let mut guard = auth_state.0.lock().unwrap();
    if let Some(sessao) = guard.as_mut() {
        sessao.bloqueada = false;
        sessao.ultima_atividade = Instant::now();
    }
    Ok(())
}

#[derive(serde::Serialize)]
pub struct Operador {
    pub id: String,
    pub nome: String,
    pub email: String,
    pub papel: Option<String>,
    /// Estabelecimentos atribuídos em `utilizador_estabelecimentos`. Vazio
    /// quer dizer "sem restrição" (ainda não há UI para atribuir).
    pub estabelecimento_ids: Vec<String>,
}

/// Lista os operadores ativos para o ecrã de login ("Quem está ao
/// balcão?") — chamado antes de haver sessão, por isso nunca devolve
/// nada além de identificação pública (sem hash, sem permissões).
#[tauri::command]
pub async fn listar_operadores(pool: State<'_, SqlitePool>) -> Result<Vec<Operador>, String> {
    let linhas: Vec<(String, String, String, Option<String>)> = sqlx::query_as(
        r#"
        SELECT u.id, u.nome, u.email,
               (SELECT p.nome FROM utilizador_papeis up JOIN papeis p ON p.id = up.papel_id
                WHERE up.utilizador_id = u.id LIMIT 1)
        FROM utilizadores u
        WHERE u.ativo = 1
        ORDER BY u.nome
        "#,
    )
    .fetch_all(pool.inner())
    .await
    .map_err(|e| e.to_string())?;

    let atribuicoes: Vec<(String, String)> = sqlx::query_as(
        "SELECT utilizador_id, estabelecimento_id FROM utilizador_estabelecimentos",
    )
    .fetch_all(pool.inner())
    .await
    .map_err(|e| e.to_string())?;

    Ok(linhas
        .into_iter()
        .map(|(id, nome, email, papel)| Operador {
            estabelecimento_ids: atribuicoes
                .iter()
                .filter(|(u, _)| *u == id)
                .map(|(_, e)| e.clone())
                .collect(),
            id,
            nome,
            email,
            papel,
        })
        .collect())
}
