import type { TipoDocumento, TotalMetodo } from "./types";

/**
 * Métodos de pagamento. O `id` é o valor gravado em
 * `pagamentos.metodo_pagamento` — o fecho de caixa no backend procura
 * literalmente "dinheiro" para calcular a diferença do numerário.
 */
export const METODOS = [
  { id: "dinheiro", n: "Numerário" },
  { id: "multicaixa", n: "TPA / Multicaixa" },
  { id: "transferencia", n: "Transferência" },
] as const;

export type MetodoId = (typeof METODOS)[number]["id"];

/**
 * Taxa de IVA aplicada às linhas enquanto os produtos não têm taxa própria
 * configurada (não há ainda comando para gerir `taxas_imposto`). O preço do
 * produto é sem IVA, como no motor de emissão (emissao::calcular_totais).
 */
export const IVA_PADRAO = 14;

export const TIPOS_VENDA: { v: TipoDocumento; l: string }[] = [
  { v: "fatura_recibo", l: "Fatura-recibo" },
  { v: "fatura", l: "Fatura" },
];

export const NOME_TIPO: Record<TipoDocumento, string> = {
  orcamento: "Orçamento",
  proforma: "Proforma",
  encomenda: "Encomenda",
  guia_remessa: "Guia de remessa",
  fatura: "Fatura",
  fatura_recibo: "Fatura-recibo",
  recibo: "Recibo",
  nota_credito: "Nota de crédito",
};

export interface LinhaCarrinho {
  produtoId: string;
  descricao: string;
  quantidade: number;
  precoCentimos: number;
  unidade: string;
}

/** Mesmas contas que emissao::calcular_totais — soma sem arredondar e só no fim. */
export function totaisCarrinho(linhas: LinhaCarrinho[]) {
  let base = 0;
  for (const l of linhas) base += l.precoCentimos * l.quantidade;
  const iva = (base * IVA_PADRAO) / 100;
  const baseC = Math.round(base);
  const ivaC = Math.round(iva);
  return { base: baseC, iva: ivaC, total: baseC + ivaC };
}

export function totalDoMetodo(totais: TotalMetodo[], metodo: string) {
  const t = totais.find((x) => x.metodo_pagamento === metodo);
  return { total: t?.total_centimos ?? 0, pagamentos: t?.pagamentos ?? 0 };
}

export function iniciaisEstabelecimento(nome: string): string {
  const palavras = nome
    .split(/\s+/)
    .filter((p) => p.length > 2 && !/^(de|da|do|das|dos)$/i.test(p));
  const alvo = palavras.length > 1 ? palavras.slice(1) : palavras;
  return alvo
    .slice(0, 2)
    .map((p) => p[0])
    .join("")
    .toUpperCase();
}
