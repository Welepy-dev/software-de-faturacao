use serde::Serialize;
use sqlx::FromRow;

#[derive(Debug, Serialize, FromRow)]
pub struct Estabelecimento {
    pub id: String,
    pub empresa_id: String,
    pub nome: String,
    pub tipo_negocio: Option<String>,
    pub endereco: Option<String>,
    pub ativo: bool,
    pub criado_em: String,
}
