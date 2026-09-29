use serde::Serialize;
use sqlx::FromRow;

#[derive(Debug, Serialize, FromRow)]
pub struct Empresa {
    pub id: String,
    pub nome: String,
    pub nif: String,
    pub morada: Option<String>,
    pub logotipo_path: Option<String>,
    pub regime_fiscal: Option<String>,
    pub moeda: String,
    pub dados_bancarios: Option<String>,
    pub condicoes_pagamento_padrao: Option<String>,
    pub perfil_complexidade: String, // 'simples' | 'completo'
    pub criado_em: String,
    pub atualizado_em: String,
}
