import { useMemo, useState, type RefObject } from "react";
import { fmt, norm, qtyStr } from "../../lib/format";
import { IVA_PADRAO, METODOS, TIPOS_VENDA, totaisCarrinho, type LinhaCarrinho, type MetodoId } from "../../lib/dominio";
import type { Cliente, ProdutoServico, TipoDocumento } from "../../lib/types";

interface Props {
  produtos: ProdutoServico[];
  clientes: Cliente[];
  linhas: LinhaCarrinho[];
  setLinhas: (fn: (l: LinhaCarrinho[]) => LinhaCarrinho[]) => void;
  tipo: TipoDocumento;
  setTipo: (t: TipoDocumento) => void;
  clienteId: string;
  setClienteId: (id: string) => void;
  metodo: MetodoId;
  setMetodo: (m: MetodoId) => void;
  onAbrirCheck: () => void;
  searchRef: RefObject<HTMLInputElement | null>;
}

const seg = (ativo: boolean, i: number) => ({
  background: ativo ? "#1D4ED8" : "transparent",
  color: ativo ? "#fff" : "#101828",
  borderLeft: i ? "1px solid var(--color-divider)" : "0",
});

/**
 * "Venda rápida" (layout A · Grelha tátil do design v3). O ticket vive só
 * no frontend até à emissão — adicionar, alterar ou remover linhas não
 * toca no backend e nunca pede password.
 */
export function Venda(p: Props) {
  const [pesquisa, setPesquisa] = useState("");
  const [cat, setCat] = useState<"todos" | "produto" | "servico">("todos");

  const temServicos = p.produtos.some((x) => x.tipo === "servico");
  const temProdutos = p.produtos.some((x) => x.tipo === "produto");
  const q = norm(pesquisa.trim());
  const visiveis = p.produtos.filter(
    (x) => (cat === "todos" || x.tipo === cat) && (!q || norm(x.nome).includes(q)),
  );
  const t = useMemo(() => totaisCarrinho(p.linhas), [p.linhas]);

  const qtdNoTicket = (id: string) =>
    p.linhas.filter((l) => l.produtoId === id).reduce((a, l) => a + l.quantidade, 0);

  function adicionar(prod: ProdutoServico) {
    p.setLinhas((ls) => {
      const i = ls.findIndex((l) => l.produtoId === prod.id);
      if (i >= 0) {
        const c = ls.slice();
        c[i] = { ...c[i], quantidade: +(c[i].quantidade + 1).toFixed(3) };
        return c;
      }
      return [
        ...ls,
        {
          produtoId: prod.id,
          descricao: prod.nome,
          quantidade: 1,
          precoCentimos: prod.preco_base_centimos,
          unidade: prod.unidade_medida,
        },
      ];
    });
  }

  function alterarQtd(i: number, delta: number) {
    p.setLinhas((ls) => {
      const c = ls.slice();
      const qn = +(c[i].quantidade + delta).toFixed(3);
      if (qn <= 0) c.splice(i, 1);
      else c[i] = { ...c[i], quantidade: qn };
      return c;
    });
  }

  function definirQtd(i: number, texto: string) {
    const n = parseFloat(texto.replace(",", "."));
    p.setLinhas((ls) => {
      const c = ls.slice();
      c[i] = { ...c[i], quantidade: isNaN(n) ? 0 : n };
      return c;
    });
  }

  const remover = (i: number) => p.setLinhas((ls) => ls.filter((_, j) => j !== i));

  const cats: { id: typeof cat; l: string }[] = [
    { id: "todos", l: "Todos" },
    ...(temProdutos && temServicos
      ? [
          { id: "produto" as const, l: "Produtos" },
          { id: "servico" as const, l: "Serviços" },
        ]
      : []),
  ];

  return (
    <div style={{ flex: 1, minHeight: 0, display: "grid", gridTemplateColumns: "minmax(0,1fr) 440px" }}>
      <div style={{ padding: "20px 24px", display: "flex", flexDirection: "column", gap: 16, minHeight: 0, overflow: "auto" }}>
        <input
          ref={p.searchRef}
          className="input"
          value={pesquisa}
          onChange={(e) => setPesquisa(e.target.value)}
          onKeyDown={(e) => {
            if (e.key === "Enter" && visiveis.length > 0) {
              adicionar(visiveis[0]);
              setPesquisa("");
            }
          }}
          placeholder="Pesquisar produto (F2) · Enter adiciona o primeiro"
          style={{ height: 48, fontSize: 16, flex: "none" }}
        />
        {cats.length > 1 && (
          <div style={{ display: "flex", gap: 8, flexWrap: "wrap" }}>
            {cats.map((c) => (
              <button
                key={c.id}
                onClick={() => setCat(c.id)}
                style={{
                  minHeight: 44,
                  padding: "0 18px",
                  borderRadius: 3,
                  border: "1px solid " + (cat === c.id ? "#1D4ED8" : "var(--color-divider)"),
                  background: cat === c.id ? "#1D4ED8" : "#fff",
                  color: cat === c.id ? "#fff" : "#101828",
                  fontWeight: 500,
                  fontSize: 14,
                  cursor: "pointer",
                }}
              >
                {c.l}
              </button>
            ))}
          </div>
        )}
        {p.produtos.length === 0 ? (
          <div className="muted" style={{ padding: "48px 0", textAlign: "center" }}>
            Ainda não há produtos. Crie-os em Gestão › Produtos.
          </div>
        ) : (
          <div style={{ display: "grid", gridTemplateColumns: "repeat(auto-fill,minmax(160px,1fr))", gap: 12 }}>
            {visiveis.map((prod) => {
              const n = qtdNoTicket(prod.id);
              return (
                <button
                  key={prod.id}
                  onClick={() => adicionar(prod)}
                  className="card-btn"
                  style={{
                    position: "relative",
                    display: "flex",
                    flexDirection: "column",
                    alignItems: "flex-start",
                    justifyContent: "space-between",
                    gap: 10,
                    minHeight: 108,
                    padding: 14,
                    border: "1px solid " + (n > 0 ? "#1D4ED8" : "var(--color-divider)"),
                    borderRadius: 3,
                  }}
                >
                  <span style={{ fontSize: 15, fontWeight: 500, lineHeight: 1.25, paddingRight: 28 }}>{prod.nome}</span>
                  <span style={{ display: "flex", justifyContent: "space-between", alignItems: "baseline", width: "100%" }}>
                    <span className="heading-num" style={{ fontSize: 19 }}>
                      {fmt(prod.preco_base_centimos)}
                    </span>
                    <span className="muted" style={{ fontSize: 12 }}>
                      /{prod.unidade_medida}
                    </span>
                  </span>
                  {n > 0 && (
                    <span
                      className="avatar"
                      style={{ position: "absolute", top: 10, right: 10, minWidth: 26, height: 26, padding: "0 6px", fontSize: 13 }}
                    >
                      {qtyStr(n)}
                    </span>
                  )}
                </button>
              );
            })}
          </div>
        )}
        <div className="muted" style={{ fontSize: 12 }}>
          Preços sem IVA · IVA {IVA_PADRAO} % acrescido na emissão.
        </div>
      </div>

      <div style={{ borderLeft: "1px solid var(--color-divider)", background: "#fff", display: "flex", flexDirection: "column", minHeight: 0 }}>
        <div style={{ padding: "16px 20px", display: "grid", gridTemplateColumns: "1fr 1fr", gap: 12, borderBottom: "1px solid var(--color-divider)" }}>
          <div className="field">
            <label>Documento</label>
            <select className="input" value={p.tipo} onChange={(e) => p.setTipo(e.target.value as TipoDocumento)} style={{ height: 44 }}>
              {TIPOS_VENDA.map((o) => (
                <option key={o.v} value={o.v}>
                  {o.l}
                </option>
              ))}
            </select>
          </div>
          <div className="field">
            <label>Cliente</label>
            <select className="input" value={p.clienteId} onChange={(e) => p.setClienteId(e.target.value)} style={{ height: 44 }}>
              <option value="">Consumidor final</option>
              {p.clientes.map((c) => (
                <option key={c.id} value={c.id}>
                  {c.nome + (c.nif ? " · " + c.nif : "")}
                </option>
              ))}
            </select>
          </div>
        </div>

        <div style={{ flex: 1, minHeight: 0, overflow: "auto", padding: "4px 20px" }}>
          {p.linhas.length === 0 && (
            <div className="muted" style={{ padding: "64px 0", textAlign: "center", fontSize: 14, lineHeight: 1.6 }}>
              Toque num produto para começar.
              <br />
              Adicionar, alterar ou remover linhas nunca pede password.
            </div>
          )}
          {p.linhas.map((l, i) => (
            <div
              key={l.produtoId}
              style={{ display: "grid", gridTemplateColumns: "minmax(0,1fr) auto", gap: "6px 12px", padding: "12px 0", borderBottom: "1px solid #EEF1F5" }}
            >
              <div style={{ fontSize: 15, fontWeight: 500 }}>{l.descricao}</div>
              <div className="heading-num" style={{ fontSize: 18, textAlign: "right" }}>
                {fmt(Math.round(l.quantidade * l.precoCentimos))}
              </div>
              <div style={{ display: "flex", alignItems: "center", gap: 6, gridColumn: "1 / -1" }}>
                <button onClick={() => alterarQtd(i, -1)} className="btn btn-secondary" style={{ width: 44, height: 44, padding: 0, fontSize: 20 }}>
                  −
                </button>
                <input
                  className="input"
                  defaultValue={qtyStr(l.quantidade)}
                  key={l.quantidade}
                  onBlur={(e) => definirQtd(i, e.target.value)}
                  onKeyDown={(e) => e.key === "Enter" && (e.target as HTMLInputElement).blur()}
                  style={{ width: 60, height: 44, textAlign: "center", fontSize: 16 }}
                />
                <button onClick={() => alterarQtd(i, 1)} className="btn btn-secondary" style={{ width: 44, height: 44, padding: 0, fontSize: 20 }}>
                  +
                </button>
                <span className="muted" style={{ fontSize: 13, marginLeft: 4 }}>
                  × {fmt(l.precoCentimos)}
                </span>
                <button
                  onClick={() => remover(i)}
                  className="btn btn-ghost"
                  title="Remover linha"
                  style={{ marginLeft: "auto", width: 44, height: 44, padding: 0, fontSize: 20 }}
                >
                  ×
                </button>
              </div>
            </div>
          ))}
        </div>

        <div style={{ padding: "14px 20px", borderTop: "1px solid var(--color-divider)", display: "flex", flexDirection: "column", gap: 5, fontSize: 14 }}>
          <div className="row-between">
            <span className="muted">Incidência (sem IVA)</span>
            <span>{fmt(t.base)}</span>
          </div>
          <div className="row-between">
            <span className="muted">IVA {IVA_PADRAO} %</span>
            <span>{fmt(t.iva)}</span>
          </div>
          <div className="row-between" style={{ alignItems: "baseline", marginTop: 4 }}>
            <span className="heading-num" style={{ fontSize: 20 }}>
              Total
            </span>
            <span className="mono-total" style={{ fontSize: 28 }}>
              {fmt(t.total)}
            </span>
          </div>
        </div>

        {p.tipo === "fatura_recibo" && (
          <div style={{ padding: "0 20px 12px" }}>
            <div className="seg" style={{ gridTemplateColumns: "repeat(3,1fr)" }}>
              {METODOS.map((m, i) => (
                <button key={m.id} onClick={() => p.setMetodo(m.id)} style={{ minHeight: 48, ...seg(p.metodo === m.id, i) }}>
                  {m.n}
                </button>
              ))}
            </div>
          </div>
        )}

        <div style={{ padding: "0 20px 20px" }}>
          <button
            onClick={p.onAbrirCheck}
            className="btn btn-primary blueprint"
            style={{ width: "100%", minHeight: 60, fontSize: 19, justifyContent: "space-between", padding: "0 20px" }}
          >
            <span>Emitir {TIPOS_VENDA.find((x) => x.v === p.tipo)?.l.toLowerCase()}</span>
            <span style={{ fontFamily: "var(--font-body)", fontSize: 12, opacity: 0.85 }}>F9</span>
          </button>
        </div>
      </div>
    </div>
  );
}
