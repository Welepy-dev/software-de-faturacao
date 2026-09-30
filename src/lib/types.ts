// Espelham as structs Rust em src-tauri/src/domain/*.rs e
// src-tauri/src/commands/*.rs — manter em sincronia.

export interface Empresa {
  id: string;
  nome: string;
  nif: string;
  morada: string | null;
  moeda: string;
  perfil_complexidade: "simples" | "completo";
}

export interface Estabelecimento {
  id: string;
  empresa_id: string;
  nome: string;
  tipo_negocio: string | null;
  endereco: string | null;
  ativo: boolean;
  criado_em: string;
}

export interface Operador {
  id: string;
  nome: string;
  email: string;
  papel: string | null;
  /** Vazio = sem restrição de estabelecimento. */
  estabelecimento_ids: string[];
}

export interface UtilizadorPublico {
  id: string;
  nome: string;
  email: string;
  ativo: boolean;
}

export interface EstadoSessao {
  utilizador_id: string;
  nome: string;
  bloqueada: boolean;
}

export type EstadoSessaoCaixa = "aberta" | "fechada";

export interface SessaoCaixa {
  id: string;
  estabelecimento_id: string;
  aberta_por_utilizador_id: string;
  fechada_por_utilizador_id: string | null;
  fundo_maneio_inicial_centimos: number;
  estado: EstadoSessaoCaixa;
  aberta_em: string;
  fechada_em: string | null;
  contagem_fisica_dinheiro_centimos: number | null;
  diferenca_calculada_centimos: number | null;
}

export interface TotalMetodo {
  metodo_pagamento: string;
  total_centimos: number;
  pagamentos: number;
}

export interface ResumoCaixa {
  sessao_aberta: SessaoCaixa | null;
  aberta_por_nome: string | null;
  totais: TotalMetodo[];
  ultima_fechada: SessaoCaixa | null;
  fechada_por_nome: string | null;
  totais_ultima_fechada: TotalMetodo[];
}

export interface ProdutoServico {
  id: string;
  empresa_id: string;
  nome: string;
  descricao: string | null;
  tipo: "produto" | "servico";
  unidade_medida: string;
  preco_base_centimos: number;
  ativo: boolean;
}

export interface Cliente {
  id: string;
  nome: string;
  nif: string | null;
  morada: string | null;
}

export type TipoDocumento =
  | "orcamento"
  | "proforma"
  | "encomenda"
  | "guia_remessa"
  | "fatura"
  | "fatura_recibo"
  | "recibo"
  | "nota_credito";

export interface Documento {
  id: string;
  tipo: TipoDocumento;
  estado: string;
  numero: number | null;
  prefixo: string | null;
  total_centimos: number;
}

export interface NovaLinhaDocumento {
  produto_servico_id: string | null;
  descricao: string;
  quantidade: number;
  preco_unitario_centimos: number;
  desconto_centimos: number;
  taxa_imposto_percentagem: number;
  justificacao_isencao: string | null;
}

export interface ResultadoEmissao {
  numero: number;
  ano_economico: number;
  prefixo: string;
  hash: string | null;
  subtotal_centimos: number;
  total_descontos_centimos: number;
  total_impostos_centimos: number;
  total_centimos: number;
  valor_por_extenso: string;
}
