use serde::{Deserialize, Serialize};
use sqlx::FromRow;

#[derive(Debug, Serialize, FromRow)]
pub struct Documento {
    pub id: String,
    pub estabelecimento_id: String,
    pub cliente_id: Option<String>,
    pub utilizador_emissor_id: String,
    pub tipo: String,
    pub serie_documental_id: Option<String>,
    pub numero: Option<i64>,
    pub ano_economico: Option<i64>,
    pub prefixo: Option<String>,
    pub estado: String,
    pub documento_origem_id: Option<String>,
    pub documento_original_id: Option<String>,
    pub motivo_anulacao_retificacao: Option<String>,
    pub hash: Option<String>,
    pub codigo_certificacao: Option<String>,
    pub versao_chave_assinatura: Option<i64>,
    pub origem_registo: String,
    pub via: String,
    pub observacoes: Option<String>,
    pub condicoes_pagamento: Option<String>,
    pub data_vencimento: Option<String>,
    pub cliente_nome_snapshot: Option<String>,
    pub cliente_nif_snapshot: Option<String>,
    pub cliente_morada_snapshot: Option<String>,
    pub subtotal_centimos: i64,
    pub total_impostos_centimos: i64,
    pub total_descontos_centimos: i64,
    pub total_centimos: i64,
    pub emitido_em: Option<String>,
    pub criado_em: String,
}

#[derive(Debug, Serialize, FromRow)]
pub struct LinhaDocumento {
    pub id: String,
    pub documento_id: String,
    pub produto_servico_id: Option<String>,
    pub descricao: String,
    pub quantidade: f64,
    pub preco_unitario_centimos: i64,
    pub desconto_centimos: i64,
    pub taxa_imposto_percentagem: f64,
    pub justificacao_isencao: Option<String>,
    pub ordem: i64,
}

/// Dados recebidos do frontend para criar uma linha ao construir um
/// rascunho — sem `id`/`documento_id`/`ordem`, atribuídos pelo backend.
#[derive(Debug, Deserialize)]
pub struct NovaLinhaDocumento {
    pub produto_servico_id: Option<String>,
    pub descricao: String,
    pub quantidade: f64,
    pub preco_unitario_centimos: i64,
    pub desconto_centimos: i64,
    pub taxa_imposto_percentagem: f64,
    pub justificacao_isencao: Option<String>,
}
