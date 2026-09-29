import { invoke } from "@tauri-apps/api/core";
import type { SessaoCaixa } from "../../lib/types";

export function obterSessaoAberta(estabelecimentoId: string): Promise<SessaoCaixa | null> {
  return invoke("obter_sessao_aberta", { estabelecimentoId });
}

export function abrirSessaoCaixa(
  estabelecimentoId: string,
  utilizadorId: string,
  fundoManeioInicialCentimos: number,
): Promise<SessaoCaixa> {
  return invoke("abrir_sessao_caixa", {
    estabelecimentoId,
    utilizadorId,
    fundoManeioInicialCentimos,
  });
}

export function fecharSessaoCaixa(
  sessaoCaixaId: string,
  utilizadorId: string,
  contagemFisicaDinheiroCentimos: number,
): Promise<SessaoCaixa> {
  return invoke("fechar_sessao_caixa", {
    sessaoCaixaId,
    utilizadorId,
    contagemFisicaDinheiroCentimos,
  });
}
