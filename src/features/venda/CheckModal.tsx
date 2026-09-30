import { useState } from "react";
import { fmt, parseKz, qtyStr } from "../../lib/format";
import { IVA_PADRAO, METODOS, NOME_TIPO, totaisCarrinho, type LinhaCarrinho, type MetodoId } from "../../lib/dominio";
import type { Cliente, Empresa, TipoDocumento } from "../../lib/types";

interface Props {
  empresa: Empresa | null;
  estName: string;
  tipo: TipoDocumento;
  cliente: Cliente | null;
  linhas: LinhaCarrinho[];
  metodo: MetodoId;
  caixaAberta: boolean;
  aEmitir: boolean;
  onEmitir: (recebidoCentimos: number) => void;
  onClose: () => void;
}

type Check = { label: string; detail: string; st: "ok" | "fail" | "info" };

const TONE = { ok: ["#E7F4EC", "#15803D"], fail: ["#FDECEC", "#B91C1C"], info: ["#E8EEFD", "#1E3A8A"] };

/** "Pronto para emitir?" — pré-visualização + verificações, antes de consumir numeração. */
export function CheckModal(p: Props) {
  const t = totaisCarrinho(p.linhas);
  const [recebido, setRecebido] = useState(String(t.total / 100).replace(".", ","));
  const rec = parseKz(recebido);
  const fr = p.tipo === "fatura_recibo";
  const numerario = fr && p.metodo === "dinheiro";

  const checks: Check[] = [];
  checks.push(
    p.linhas.length && p.linhas.every((l) => l.quantidade > 0)
      ? { label: "Linhas válidas", detail: p.linhas.length + " linha(s), quantidades e preços preenchidos", st: "ok" }
      : { label: "Linhas inválidas", detail: "Há linhas sem quantidade", st: "fail" },
  );
  if (p.tipo === "fatura" && !p.cliente?.nif)
    checks.push({ label: "Adquirente sem NIF", detail: "A fatura exige cliente identificado com NIF", st: "fail" });
  else
    checks.push({
      label: "Adquirente: " + (p.cliente?.nome ?? "Consumidor final"),
      detail: p.cliente?.nif ? "NIF " + p.cliente.nif : "Venda a consumidor final",
      st: "ok",
    });
  if (fr) {
    if (numerario)
      checks.push(
        rec >= t.total
          ? { label: "Pagamento", detail: "Numerário recebido cobre o total", st: "ok" }
          : { label: "Valor recebido insuficiente", detail: "Faltam " + fmt(t.total - rec), st: "fail" },
      );
    else
      checks.push({ label: "Pagamento", detail: METODOS.find((m) => m.id === p.metodo)!.n + " · " + fmt(t.total), st: "ok" });
    checks.push(
      p.caixaAberta
        ? { label: "Caixa aberta", detail: "O pagamento atualiza os totais da sessão na mesma transação", st: "ok" }
        : { label: "Caixa fechada", detail: "Abra a caixa antes de emitir faturas-recibo", st: "fail" },
    );
  } else {
    checks.push({ label: "Pagamento não exigido", detail: NOME_TIPO[p.tipo] + " não regista pagamento na emissão", st: "info" });
  }
  checks.push({ label: "Numeração", detail: "Número sequencial da série atribuído só ao confirmar", st: "info" });
  checks.push({ label: "Cadeia de hash", detail: "Assinatura provisória · TODO: algoritmo AGT por confirmar", st: "info" });

  const bloqueado = checks.some((c) => c.st === "fail") || p.aEmitir;
  const emitir = () => !bloqueado && p.onEmitir(rec);

  return (
    <div className="overlay" onKeyDown={(e) => e.key === "Enter" && (e.preventDefault(), emitir())}>
      <div
        style={{
          width: "min(980px,100%)",
          maxHeight: "100%",
          overflow: "auto",
          background: "#fff",
          boxShadow: "var(--shadow-lg)",
          borderRadius: 3,
          display: "grid",
          gridTemplateColumns: "minmax(0,1fr) 400px",
        }}
      >
        <div style={{ padding: 24, background: "#E4E9F0", display: "flex", flexDirection: "column", gap: 12 }}>
          <span style={{ fontSize: 11, color: "#475467" }}>Pré-visualização · ainda não consome numeração</span>
          <div style={{ background: "#fff", padding: 24, fontSize: 12, display: "flex", flexDirection: "column", gap: 14 }}>
            <div style={{ display: "flex", justifyContent: "space-between", gap: 16 }}>
              <div>
                <div style={{ fontWeight: 600, fontSize: 13 }}>{p.empresa?.nome}</div>
                <div>
                  NIF {p.empresa?.nif} · {p.estName}
                </div>
              </div>
              <div style={{ textAlign: "right" }}>
                <div className="heading-num" style={{ fontSize: 20 }}>
                  {NOME_TIPO[p.tipo]}
                </div>
                <div className="muted">N.º atribuído no momento da emissão</div>
              </div>
            </div>
            <div>
              Adquirente: <strong>{p.cliente?.nome ?? "Consumidor final"}</strong>
              {p.cliente?.nif ? " · NIF " + p.cliente.nif : ""}
            </div>
            <table style={{ width: "100%", borderCollapse: "collapse" }}>
              <tbody>
                {p.linhas.map((l) => (
                  <tr key={l.produtoId} style={{ borderBottom: "1px solid #EEF2F7" }}>
                    <td style={{ padding: "6px 0" }}>{l.descricao}</td>
                    <td className="muted" style={{ padding: "6px 0", textAlign: "right" }}>
                      {qtyStr(l.quantidade)} {l.unidade} × {fmt(l.precoCentimos)}
                    </td>
                    <td style={{ padding: "6px 0", textAlign: "right", width: 110 }}>
                      {fmt(Math.round(l.quantidade * l.precoCentimos))}
                    </td>
                  </tr>
                ))}
              </tbody>
            </table>
            <div style={{ alignSelf: "flex-end", width: 240, display: "flex", flexDirection: "column", gap: 3 }}>
              <div className="row-between">
                <span>Incidência</span>
                <span>{fmt(t.base)}</span>
              </div>
              <div className="row-between">
                <span>IVA {IVA_PADRAO} %</span>
                <span>{fmt(t.iva)}</span>
              </div>
              <div className="row-between" style={{ fontWeight: 700, fontSize: 14 }}>
                <span>Total</span>
                <span>{fmt(t.total)}</span>
              </div>
            </div>
          </div>
        </div>
        <div style={{ padding: 24, display: "flex", flexDirection: "column", gap: 16 }}>
          <h3 style={{ margin: 0, fontSize: 22 }}>Pronto para emitir?</h3>
          <div style={{ display: "flex", flexDirection: "column", gap: 12 }}>
            {checks.map((c) => (
              <div key={c.label} style={{ display: "grid", gridTemplateColumns: "24px 1fr", gap: 10 }}>
                <span
                  style={{
                    width: 22,
                    height: 22,
                    borderRadius: "50%",
                    display: "grid",
                    placeItems: "center",
                    background: TONE[c.st][0],
                    color: TONE[c.st][1],
                    fontSize: 12,
                    fontWeight: 700,
                  }}
                >
                  {c.st === "ok" ? "✓" : c.st === "fail" ? "✕" : "i"}
                </span>
                <div style={{ display: "flex", flexDirection: "column" }}>
                  <span style={{ fontSize: 14, fontWeight: 500 }}>{c.label}</span>
                  <span className="muted" style={{ fontSize: 12 }}>
                    {c.detail}
                  </span>
                </div>
              </div>
            ))}
          </div>
          {numerario && (
            <div style={{ display: "grid", gridTemplateColumns: "1fr 1fr", gap: 12, alignItems: "end" }}>
              <div className="field">
                <label>Valor recebido (Kz)</label>
                <input
                  className="input"
                  autoFocus
                  value={recebido}
                  onChange={(e) => setRecebido(e.target.value)}
                  style={{ height: 48, fontSize: 18, textAlign: "right" }}
                />
              </div>
              <div style={{ display: "flex", flexDirection: "column", paddingBottom: 4 }}>
                <span className="muted" style={{ fontSize: 12 }}>
                  Troco
                </span>
                <span className="heading-num" style={{ fontSize: 24 }}>
                  {fmt(Math.max(0, rec - t.total))}
                </span>
              </div>
            </div>
          )}
          <div style={{ display: "flex", gap: 12, marginTop: "auto" }}>
            <button onClick={p.onClose} className="btn btn-secondary" style={{ minHeight: 56, padding: "0 20px" }}>
              Voltar · Esc
            </button>
            <button onClick={emitir} disabled={bloqueado} className="btn btn-primary" style={{ flex: 1, minHeight: 56, fontSize: 18 }}>
              {p.aEmitir ? "A emitir…" : "Emitir " + NOME_TIPO[p.tipo].toLowerCase() + " · Enter"}
            </button>
          </div>
        </div>
      </div>
    </div>
  );
}
