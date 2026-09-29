use argon2::password_hash::rand_core::OsRng;
use argon2::password_hash::{PasswordHash, PasswordHasher, PasswordVerifier, SaltString};
use argon2::Argon2;
use std::sync::Mutex;
use std::time::{Duration, Instant};

/// Tempo de inatividade até a sessão bloquear (docs/ARQUITETURA.md §5).
/// Bloquear ≠ logout: o mesmo operador desbloqueia com a password, sem
/// perder o turno.
pub const TIMEOUT_INATIVIDADE: Duration = Duration::from_secs(45 * 60);

pub struct SessaoUtilizador {
    pub utilizador_id: String,
    pub nome: String,
    pub email: String,
    pub ultima_atividade: Instant,
    pub bloqueada: bool,
}

/// Estado de sessão em memória (gerido pelo Tauri como State) — um único
/// operador autenticado de cada vez neste processo desktop.
#[derive(Default)]
pub struct AuthState(pub Mutex<Option<SessaoUtilizador>>);

impl AuthState {
    /// Actualiza a marca de última atividade — chamado a cada acção do
    /// utilizador vinda do frontend, desde que a sessão não esteja já
    /// bloqueada (tocar não desbloqueia; só `desbloquear_sessao` com
    /// password o faz).
    pub fn tocar_atividade(&self) {
        let mut guard = self.0.lock().unwrap();
        if let Some(sessao) = guard.as_mut() {
            if !sessao.bloqueada {
                sessao.ultima_atividade = Instant::now();
            }
        }
    }

    /// Verifica se passou o tempo de inatividade e, nesse caso, marca a
    /// sessão como bloqueada. Deve ser chamado antes de qualquer leitura
    /// do estado de bloqueio.
    pub fn verificar_bloqueio(&self) {
        let mut guard = self.0.lock().unwrap();
        if let Some(sessao) = guard.as_mut() {
            if !sessao.bloqueada && sessao.ultima_atividade.elapsed() > TIMEOUT_INATIVIDADE {
                sessao.bloqueada = true;
            }
        }
    }

    /// Devolve o utilizador autenticado, falhando se não houver sessão ou
    /// se estiver bloqueada — usar no início de qualquer comando protegido.
    pub fn exigir_sessao_ativa(&self) -> Result<String, String> {
        self.verificar_bloqueio();
        let guard = self.0.lock().unwrap();
        match guard.as_ref() {
            None => Err("Sem sessão activa — é necessário fazer login.".to_string()),
            Some(s) if s.bloqueada => {
                Err("Sessão bloqueada por inatividade — introduza a password para continuar."
                    .to_string())
            }
            Some(s) => Ok(s.utilizador_id.clone()),
        }
    }
}

/// Gera o hash Argon2id de uma password — nunca se guarda a password em
/// claro (requisito AGT confirmado em docs/AGT-SAFT.md: o administrador
/// nunca pode ver/conhecer as passwords dos utilizadores).
pub fn gerar_hash_password(password: &str) -> Result<String, String> {
    let salt = SaltString::generate(&mut OsRng);
    Argon2::default()
        .hash_password(password.as_bytes(), &salt)
        .map(|h| h.to_string())
        .map_err(|e| e.to_string())
}

/// Verifica uma password contra o hash guardado. Erros de formato do hash
/// e password errada devolvem a mesma mensagem genérica, para não revelar
/// qual das duas falhou.
pub fn verificar_password(password: &str, hash: &str) -> Result<(), String> {
    let parsed_hash =
        PasswordHash::new(hash).map_err(|_| "Credenciais inválidas.".to_string())?;
    Argon2::default()
        .verify_password(password.as_bytes(), &parsed_hash)
        .map_err(|_| "Credenciais inválidas.".to_string())
}
