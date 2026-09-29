// Espelham as structs Rust em src-tauri/src/domain/*.rs — manter em sincronia.

export interface Estabelecimento {
  id: string;
  empresa_id: string;
  nome: string;
  tipo_negocio: string | null;
  endereco: string | null;
  ativo: boolean;
  criado_em: string;
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
