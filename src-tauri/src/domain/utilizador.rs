use serde::Serialize;
use sqlx::FromRow;

/// Vista pública de um utilizador — nunca inclui `password_hash`.
#[derive(Debug, Serialize, FromRow)]
pub struct UtilizadorPublico {
    pub id: String,
    pub nome: String,
    pub email: String,
    pub ativo: bool,
}

#[derive(Debug, Serialize)]
pub struct EstadoSessao {
    pub utilizador_id: String,
    pub nome: String,
    pub bloqueada: bool,
}
