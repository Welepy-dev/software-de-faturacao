use serde::Serialize;
use sqlx::FromRow;

/// Uma sessão de caixa pertence sempre a um único Estabelecimento, e a
/// base de dados impõe (via índice único parcial) que no máximo uma esteja
/// `aberta` por estabelecimento em cada momento — ver docs/ARQUITETURA.md §4.1.
#[derive(Debug, Serialize, FromRow)]
pub struct SessaoCaixa {
    pub id: String,
    pub estabelecimento_id: String,
    pub aberta_por_utilizador_id: String,
    pub fechada_por_utilizador_id: Option<String>,
    pub fundo_maneio_inicial_centimos: i64,
    pub estado: String,
    pub aberta_em: String,
    pub fechada_em: Option<String>,
    pub contagem_fisica_dinheiro_centimos: Option<i64>,
    pub diferenca_calculada_centimos: Option<i64>,
}

/// Total corrente por método de pagamento, atualizado incrementalmente a
/// cada `Pagamento` registado (mesma transação) — ver docs/ARQUITETURA.md §4.2.
#[derive(Debug, Serialize, FromRow)]
pub struct SessaoCaixaTotalPorMetodo {
    pub id: String,
    pub sessao_caixa_id: String,
    pub metodo_pagamento: String,
    pub total_acumulado_centimos: i64,
}
