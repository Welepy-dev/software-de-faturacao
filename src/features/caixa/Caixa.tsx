import { useState } from "react";
import { diffColor, fmt, fmtS, horaMinuto, parseKz } from "../../lib/format";
import { METODOS, totalDoMetodo } from "../../lib/dominio";
import type { ResumoCaixa } from "../../lib/types";

interface Props {
  resumo: ResumoCaixa | undefined;
  /** Pede a password (modal de ação protegida) e fecha a sessão no backend. */
  onFechar: (contagemDinheiroCentimos: number, diferenca: number) => void;
  onAbrir: (fundoCentimos: number) => Promise<void>;
}

const SEP = "1px solid var(--color-divider)";

/**
 * Página "Caixa" do design v3 (fecho "A · Tabela por método"). Os totais
 * vêm de `sessao_caixa_totais_por_metodo`, acumulados na mesma transação
 * de cada pagamento — nada é somado aqui no fecho.
 *
 * Diferenças face ao design, por limites do backend actual:
 * - só o numerário é contado (fechar_sessao_caixa recebe só essa contagem);
 *   TPA e transferência mostram o esperado, sem campo de contagem;
 * - não há movimentos manuais (sangria/reforço) nem justificação de
 *   diferença — ainda não existem no schema;
 * - fechar pede sempre password (caixa.fechar requer reautenticação),
 *   não só quando há diferença.
 */
export function Caixa({ resumo, onFechar, onAbrir }: Props) {
  const [aFechar, setAFechar] = useState(false);
  const [contado, setContado] = useState("");
  const [fundo, setFundo] = useState("20 000");
  const [aAbrir, setAAbrir] = useState(false);

  const s = resumo?.sessao_aberta ?? null;
  const totais = resumo?.totais ?? [];
  const totalGeral = totais.reduce((a, t) => a + t.total_centimos, 0);
  const pagamentos = totais.reduce((a, t) => a + t.pagamentos, 0);
  const dinheiro = totalDoMetodo(totais, "dinheiro").total;
  const fundoInicial = s?.fundo_maneio_inicial_centimos ?? 0;
  const esperadoGaveta = fundoInicial + dinheiro;

  if (!s) {
    const f = resumo?.ultima_fechada ?? null;
    const tf = resumo?.totais_ultima_fechada ?? [];
    const esperadoF = (f?.fundo_maneio_inicial_centimos ?? 0) + totalDoMetodo(tf, "dinheiro").total;
    return (
      <div style={{ padding: "20px 24px 32px", display: "grid", gridTemplateColumns: "minmax(0,1fr) 380px", gap: 40, alignItems: "start" }}>
        <div style={{ display: "flex", flexDirection: "column", gap: 12 }}>
          <h3 style={{ margin: 0 }}>Último fecho</h3>
          {f ? (
            <>
              <div className="muted">
                Fechada às {f.fechada_em ? horaMinuto(f.fechada_em) : "—"}
                {resumo?.fechada_por_nome ? " por " + resumo.fechada_por_nome : ""}
              </div>
              <table className="table" style={{ fontSize: 15 }}>
                <thead>
                  <tr>
                    <th>Método</th>
                    <th style={{ textAlign: "right" }}>Esperado</th>
                    <th style={{ textAlign: "right" }}>Contado</th>
                    <th style={{ textAlign: "right" }}>Diferença</th>
                  </tr>
                </thead>
                <tbody>
                  {METODOS.map((m) => {
                    const dinheiroRow = m.id === "dinheiro";
                    const exp = dinheiroRow ? esperadoF : totalDoMetodo(tf, m.id).total;
                    const dif = f.diferenca_calculada_centimos ?? 0;
                    return (
                      <tr key={m.id} style={{ height: 48 }}>
                        <td>{m.n}</td>
                        <td style={{ textAlign: "right" }}>{fmt(exp)}</td>
                        <td style={{ textAlign: "right" }}>
                          {dinheiroRow ? fmt(f.contagem_fisica_dinheiro_centimos ?? 0) : "—"}
                        </td>
                        <td style={{ textAlign: "right", fontWeight: 500, color: dinheiroRow ? diffColor(dif) : "#667085" }}>
                          {dinheiroRow ? fmtS(dif) : "—"}
                        </td>
                      </tr>
                    );
                  })}
                </tbody>
              </table>
            </>
          ) : (
            <div className="muted">Sem fecho registado neste estabelecimento.</div>
          )}
        </div>
        <div className="blueprint" style={{ padding: 20, display: "flex", flexDirection: "column", gap: 14 }}>
          <h3 style={{ margin: 0 }}>Abrir caixa</h3>
          <div className="field">
            <label>Fundo de maneio (Kz)</label>
            <input
              className="input"
              value={fundo}
              onChange={(e) => setFundo(e.target.value)}
              style={{ height: 52, fontSize: 20, textAlign: "right" }}
            />
          </div>
          <button
            onClick={async () => {
              setAAbrir(true);
              try {
                await onAbrir(parseKz(fundo));
              } finally {
                setAAbrir(false);
              }
            }}
            disabled={aAbrir}
            className="btn btn-primary"
            style={{ minHeight: 56, fontSize: 17 }}
          >
            Abrir caixa
          </button>
          <div className="muted" style={{ fontSize: 12 }}>
            A caixa dos outros estabelecimentos não é afetada.
          </div>
        </div>
      </div>
    );
  }

  const stats = [
    { k: "Estado", v: "Aberta", sub: "desde " + horaMinuto(s.aberta_em) + (resumo?.aberta_por_nome ? " · " + resumo.aberta_por_nome : ""), color: "#15803D" },
    { k: "Fundo de maneio", v: fmt(fundoInicial), sub: "na abertura", color: "#101828" },
    { k: "Pagamentos", v: String(pagamentos), sub: "nesta sessão", color: "#101828" },
    { k: "Total recebido", v: fmt(totalGeral), sub: "todos os métodos", color: "#101828" },
    { k: "Numerário esperado", v: fmt(esperadoGaveta), sub: "na gaveta", color: "#101828" },
  ];

  const preenchido = contado.trim() !== "";
  const cnt = parseKz(contado);
  const dif = cnt - esperadoGaveta;

  return (
    <div style={{ padding: "20px 24px 32px", display: "flex", flexDirection: "column", gap: 24 }}>
      <div style={{ display: "grid", gridTemplateColumns: "repeat(5,minmax(0,1fr))", border: SEP, borderRadius: 3, overflow: "hidden", background: "#fff" }}>
        {stats.map((st, i) => (
          <div key={st.k} style={{ padding: "14px 16px", borderLeft: i ? SEP : 0, display: "flex", flexDirection: "column", gap: 2 }}>
            <span className="kicker">{st.k}</span>
            <span className="heading-num" style={{ fontSize: 24, color: st.color }}>
              {st.v}
            </span>
            <span className="muted" style={{ fontSize: 12 }}>
              {st.sub}
            </span>
          </div>
        ))}
      </div>

      {!aFechar ? (
        <div style={{ display: "grid", gridTemplateColumns: "minmax(0,1fr) minmax(0,1fr)", gap: 40, alignItems: "start" }}>
          <div style={{ display: "flex", flexDirection: "column", gap: 12 }}>
            <div className="row-between" style={{ alignItems: "baseline" }}>
              <h3 style={{ margin: 0 }}>Totais da sessão</h3>
              <span className="muted" style={{ fontSize: 12 }}>
                Atualizados na mesma transação de cada pagamento
              </span>
            </div>
            <table className="table" style={{ fontSize: 15 }}>
              <thead>
                <tr>
                  <th>Método</th>
                  <th style={{ textAlign: "right" }}>Pagamentos</th>
                  <th style={{ textAlign: "right" }}>Total</th>
                </tr>
              </thead>
              <tbody>
                {METODOS.map((m) => {
                  const t = totalDoMetodo(totais, m.id);
                  return (
                    <tr key={m.id} style={{ height: 48 }}>
                      <td>{m.n}</td>
                      <td style={{ textAlign: "right" }}>{t.pagamentos}</td>
                      <td style={{ textAlign: "right", fontWeight: 500 }}>{fmt(t.total)}</td>
                    </tr>
                  );
                })}
                <tr style={{ height: 52 }}>
                  <td className="heading-num" style={{ fontSize: 17 }}>
                    Total geral
                  </td>
                  <td style={{ textAlign: "right" }}>{pagamentos}</td>
                  <td className="heading-num" style={{ textAlign: "right", fontSize: 20 }}>
                    {fmt(totalGeral)}
                  </td>
                </tr>
              </tbody>
            </table>
            <div style={{ display: "flex", gap: 12, marginTop: 8 }}>
              <button
                onClick={() => {
                  setContado("");
                  setAFechar(true);
                }}
                className="btn btn-primary blueprint"
                style={{ minHeight: 56, fontSize: 17, padding: "0 28px" }}
              >
                Fechar caixa
              </button>
            </div>
          </div>
          <div style={{ display: "flex", flexDirection: "column", gap: 12 }}>
            <h3 style={{ margin: 0 }}>Numerário na gaveta</h3>
            <div className="blueprint" style={{ padding: "16px 18px", display: "flex", justifyContent: "space-between", alignItems: "baseline" }}>
              <span>Numerário esperado na gaveta</span>
              <span className="heading-num" style={{ fontSize: 22 }}>
                {fmt(esperadoGaveta)}
              </span>
            </div>
            <div className="muted" style={{ fontSize: 12 }}>
              Fundo de maneio {fmt(fundoInicial)} + numerário recebido {fmt(dinheiro)}
            </div>
          </div>
        </div>
      ) : (
        <div style={{ display: "grid", gridTemplateColumns: "minmax(0,1fr) 360px", gap: 40, alignItems: "start" }}>
          <div style={{ display: "flex", flexDirection: "column", gap: 12 }}>
            <div>
              <h3 style={{ margin: 0 }}>Fecho de caixa</h3>
              <div className="muted" style={{ fontSize: 14 }}>
                O esperado já está calculado. Introduza só o numerário contado.
              </div>
            </div>
            <table className="table" style={{ fontSize: 15 }}>
              <thead>
                <tr>
                  <th>Método</th>
                  <th style={{ textAlign: "right" }}>Esperado</th>
                  <th style={{ textAlign: "right", width: 210 }}>Contado</th>
                  <th style={{ textAlign: "right" }}>Diferença</th>
                </tr>
              </thead>
              <tbody>
                <tr style={{ height: 64 }}>
                  <td>Numerário</td>
                  <td style={{ textAlign: "right" }}>{fmt(esperadoGaveta)}</td>
                  <td style={{ textAlign: "right" }}>
                    <input
                      className="input"
                      autoFocus
                      value={contado}
                      onChange={(e) => setContado(e.target.value)}
                      placeholder="0,00"
                      style={{ width: 190, height: 48, textAlign: "right", fontSize: 17 }}
                    />
                  </td>
                  <td style={{ textAlign: "right", fontWeight: 500, color: preenchido ? diffColor(dif) : "#667085" }}>
                    {preenchido ? fmtS(dif) : "—"}
                  </td>
                </tr>
                {METODOS.filter((m) => m.id !== "dinheiro").map((m) => (
                  <tr key={m.id} style={{ height: 52 }}>
                    <td>{m.n}</td>
                    <td style={{ textAlign: "right" }}>{fmt(totalDoMetodo(totais, m.id).total)}</td>
                    <td className="muted" style={{ textAlign: "right", fontSize: 13 }}>
                      sem contagem
                    </td>
                    <td className="muted" style={{ textAlign: "right" }}>
                      —
                    </td>
                  </tr>
                ))}
              </tbody>
            </table>
          </div>
          <div className="blueprint" style={{ padding: 20, display: "flex", flexDirection: "column", gap: 14 }}>
            <span className="kicker">Diferença no numerário</span>
            <span className="mono-total" style={{ fontSize: 28, color: preenchido ? diffColor(dif) : "#667085" }}>
              {preenchido ? fmtS(dif) : "—"}
            </span>
            <span className="muted" style={{ fontSize: 13 }}>
              {!preenchido
                ? "Introduza o numerário contado na gaveta."
                : dif === 0
                  ? "Sem diferença."
                  : dif < 0
                    ? "Falta numerário na gaveta."
                    : "Há numerário a mais na gaveta."}
            </span>
            <button
              onClick={() => onFechar(cnt, dif)}
              disabled={!preenchido}
              className="btn btn-primary"
              style={{ minHeight: 56, fontSize: 17 }}
            >
              {!preenchido ? "Preencha o valor contado" : dif === 0 ? "Confirmar fecho" : "Fechar com diferença"}
            </button>
            <button onClick={() => setAFechar(false)} className="btn btn-secondary" style={{ minHeight: 44 }}>
              Cancelar
            </button>
            <span className="muted" style={{ fontSize: 12 }}>
              Fechar a caixa é uma ação protegida: pede a sua password.
            </span>
          </div>
        </div>
      )}
    </div>
  );
}
