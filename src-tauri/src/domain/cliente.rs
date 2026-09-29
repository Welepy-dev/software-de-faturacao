use serde::Serialize;
use sqlx::FromRow;

#[derive(Debug, Serialize, FromRow)]
pub struct Cliente {
    pub id: String,
    pub empresa_id: String,
    pub nome: String,
    pub email: Option<String>,
    pub telefone: Option<String>,
    pub morada: Option<String>,
    pub nif: Option<String>,
    pub ativo: bool,
    pub criado_em: String,
}
