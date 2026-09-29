use serde::Serialize;
use sqlx::FromRow;

#[derive(Debug, Serialize, FromRow)]
pub struct ProdutoServico {
    pub id: String,
    pub empresa_id: String,
    pub nome: String,
    pub descricao: Option<String>,
    pub tipo: String, // 'produto' | 'servico'
    pub unidade_medida: String,
    pub preco_base_centimos: i64,
    pub taxa_imposto_id: Option<String>,
    pub controla_stock: bool,
    pub ativo: bool,
    pub criado_em: String,
}
