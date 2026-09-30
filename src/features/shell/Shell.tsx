import { useCallback, useEffect, useRef, useState } from "react";
import { useRelogio } from "../../components/BrandPanel";
import { estadoCaixa, recebidoNaSessao } from "../../components/EstCards";
import { ReauthModal } from "../../components/ReauthModal";
import { Toast, type ToastMsg } from "../../components/Toast";
import * as api from "../../lib/api";
import { erroTexto, fmt, fmtS, horaMinuto, iniciais } from "../../lib/format";
import {
  IVA_PADRAO,
  METODOS,
  NOME_TIPO,
  iniciaisEstabelecimento,
  type LinhaCarrinho,
  type MetodoId,
} from "../../lib/dominio";
import type { Cliente, Empresa, Estabelecimento, Operador, ProdutoServico, ResumoCaixa, TipoDocumento } from "../../lib/types";
import { Caixa } from "../caixa/Caixa";
import { Produtos } from "../produtos/Produtos";
import { CheckModal } from "../venda/CheckModal";
import { Venda } from "../venda/Venda";

interface Props {
  empresa: Empresa | null;
  estabelecimentos: Estabelecimento[];
  operador: Operador;
  turnoDesde: Date;
  estId: string;
  setEstId: (id: string) => void;
  resumos: Record<string, ResumoCaixa>;
  refrescarResumo: (estId: string) => Promise<void>;
  /** Falso enquanto o ecrã de bloqueio está por cima — ignora atalhos. */
  ativo: boolean;
  onBloquear: () => void;
  onTrocar: () => void;
}

type Pagina = "venda" | "caixa" | "produtos" | "vazio";

interface Ticket {
  linhas: LinhaCarrinho[];
  tipo: TipoDocumento;
  clienteId: string;
}

const TICKET_VAZIO: Ticket = { linhas: [], tipo: "fatura_recibo", clienteId: "" };

interface Reauth {
  label: string;
  acao: (password: string) => Promise<void>;
}

export function Shell(p: Props) {
  const agora = useRelogio();
  const [pagina, setPagina] = useState<Pagina>("venda");
  const [vazioLabel, setVazioLabel] = useState("");
  // Um ticket por estabelecimento: trocar de estabelecimento não perde o ticket do outro.
  const [tickets, setTickets] = useState<Record<string, Ticket>>({});
  const [metodo, setMetodo] = useState<MetodoId>("dinheiro");
  const [produtos, setProdutos] = useState<ProdutoServico[]>([]);
  const [clientes, setClientes] = useState<Cliente[]>([]);
  const [modal, setModal] = useState<null | "check" | "est">(null);
  const [reauth, setReauth] = useState<Reauth | null>(null);
  const [toast, setToast] = useState<ToastMsg | null>(null);
  const [aEmitir, setAEmitir] = useState(false);
  const searchRef = useRef<HTMLInputElement>(null);

  const est = p.estabelecimentos.find((e) => e.id === p.estId)!;
  const resumo = p.resumos[p.estId];
  const caixa = estadoCaixa(resumo);
  const ticket = tickets[p.estId] ?? TICKET_VAZIO;
  const cliente = clientes.find((c) => c.id === ticket.clienteId) ?? null;

  const setTicket = useCallback(
    (fn: (t: Ticket) => Ticket) =>
      setTickets((ts) => ({ ...ts, [p.estId]: fn(ts[p.estId] ?? TICKET_VAZIO) })),
    [p.estId],
  );
  const erro = (e: unknown) => setToast({ msg: erroTexto(e), erro: true });
  const fecharToast = useCallback(() => setToast(null), []);

  const carregarProdutos = useCallback(() => api.listarProdutos().then(setProdutos).catch(erro), []);
  useEffect(() => {
    carregarProdutos();
    api.listarClientes().then(setClientes).catch(erro);
  }, [carregarProdutos]);

  function abrirCheck() {
    if (!ticket.linhas.length) {
      setToast({ msg: "O ticket está vazio" });
      return;
    }
    setModal("check");
  }

  /**
   * Emissão: rascunho → linhas → emitir (transação atómica no backend:
   * número da série, totais, hash) → pagamento (outra transação, que
   * incrementa os totais da sessão de caixa).
   */
  async function emitir() {
    const sessao = resumo?.sessao_aberta;
    setAEmitir(true);
    let emitido: string | null = null;
    try {
      const doc = await api.criarDocumentoRascunho(p.estId, ticket.tipo, ticket.clienteId || null);
      for (const l of ticket.linhas) {
        await api.adicionarLinhaDocumento(doc.id, {
          produto_servico_id: l.produtoId,
          descricao: l.descricao,
          quantidade: l.quantidade,
          preco_unitario_centimos: l.precoCentimos,
          desconto_centimos: 0,
          taxa_imposto_percentagem: IVA_PADRAO,
          justificacao_isencao: null,
        });
      }
      const r = await api.emitirDocumento(doc.id);
      emitido = `${NOME_TIPO[ticket.tipo]} ${r.prefixo} ${r.ano_economico}/${r.numero}`;
      if (ticket.tipo === "fatura_recibo" && sessao) {
        await api.registarPagamento({
          documentoId: doc.id,
          sessaoCaixaId: sessao.id,
          metodoPagamento: metodo,
          valorCentimos: r.total_centimos,
        });
      }
      setTicket(() => TICKET_VAZIO);
      setModal(null);
      setToast({ msg: `${emitido} emitida · ${fmt(r.total_centimos)}` });
      await p.refrescarResumo(p.estId);
    } catch (e) {
      setModal(null);
      setToast({
        msg: emitido
          ? `${emitido} foi emitida, mas o pagamento não ficou registado: ${erroTexto(e)}`
          : erroTexto(e),
        erro: true,
      });
      if (emitido) setTicket(() => TICKET_VAZIO);
    } finally {
      setAEmitir(false);
    }
  }

  function pedirFecho(contagem: number, diferenca: number) {
    const sessao = resumo?.sessao_aberta;
    if (!sessao) return;
    setReauth({
      label: diferenca === 0 ? "Fechar caixa" : "Fechar caixa com diferença de " + fmtS(diferenca),
      acao: async (pwd) => {
        await api.fecharSessaoCaixa(sessao.id, contagem, pwd);
        setReauth(null);
        setToast({
          msg: `Caixa fechada · ${est.nome}.` + (p.estabelecimentos.length > 1 ? " As outras caixas continuam como estavam." : ""),
        });
        await p.refrescarResumo(p.estId);
      },
    });
  }

  async function abrirCaixa(fundo: number) {
    try {
      await api.abrirSessaoCaixa(p.estId, fundo);
      setToast({ msg: `Caixa aberta com ${fmt(fundo)} · ${est.nome}` });
    } catch (e) {
      erro(e);
    }
    await p.refrescarResumo(p.estId);
  }

  // Atalhos de teclado do design v3.
  useEffect(() => {
    if (!p.ativo) return;
    function onKey(e: KeyboardEvent) {
      if (reauth) return;
      if (e.key === "Escape" && modal) {
        setModal(null);
        return;
      }
      if (e.key === "F9") {
        e.preventDefault();
        if (pagina === "venda" && !modal) abrirCheck();
      }
      if (e.key === "F2") {
        e.preventDefault();
        setPagina("venda");
        setTimeout(() => searchRef.current?.focus());
      }
      if (e.key === "F12") {
        e.preventDefault();
        p.onBloquear();
      }
      if (e.altKey && ["1", "2", "3"].includes(e.key) && pagina === "venda") {
        e.preventDefault();
        setMetodo(METODOS[+e.key - 1].id);
      }
    }
    window.addEventListener("keydown", onKey);
    return () => window.removeEventListener("keydown", onKey);
  });

  const nav: { label: string; items: { label: string; pg: Pagina; count?: string }[] }[] = [
    { label: "Operação", items: [{ label: "Venda rápida", pg: "venda" }, { label: "Caixa", pg: "caixa" }] },
    {
      label: "Gestão",
      items: [
        { label: "Produtos", pg: "produtos", count: String(produtos.length) },
        { label: "Documentos", pg: "vazio" },
        { label: "Clientes", pg: "vazio" },
        { label: "Relatórios", pg: "vazio" },
        { label: "Configurações", pg: "vazio" },
      ],
    },
  ];

  const titulos: Record<Pagina, [string, string]> = {
    venda: ["Venda rápida", "Caixa " + (caixa.aberta ? "aberta" : "fechada")],
    caixa: ["Caixa", est.nome],
    produtos: ["Produtos", "Partilhados entre estabelecimentos"],
    vazio: [vazioLabel, ""],
  };

  return (
    <div style={{ flex: 1, display: "flex", minHeight: 0 }}>
      <aside
        className="side"
        style={{ width: 252, flex: "none", background: "#0B1730", color: "#E4E8F0", display: "flex", flexDirection: "column", minHeight: 0 }}
      >
        <div style={{ height: 60, flex: "none", display: "flex", alignItems: "center", gap: 10, padding: "0 18px" }}>
          <span className="heading-num" style={{ fontSize: 22 }}>
            Série
          </span>
          <span className="tag tag-accent">{p.empresa?.perfil_complexidade === "simples" ? "Simples" : "Completo"}</span>
        </div>
        <div style={{ padding: "4px 16px 16px" }}>
          <button
            onClick={() => setModal("est")}
            style={{
              width: "100%",
              display: "flex",
              alignItems: "center",
              gap: 10,
              padding: 10,
              minHeight: 60,
              background: "rgba(255,255,255,.05)",
              border: "1px solid rgba(255,255,255,.14)",
              borderRadius: 3,
              cursor: "pointer",
              textAlign: "left",
              color: "#E4E8F0",
            }}
          >
            <span className="avatar" style={{ width: 38, height: 38, fontSize: 13 }}>
              {iniciaisEstabelecimento(est.nome)}
            </span>
            <span style={{ flex: 1, minWidth: 0, display: "flex", flexDirection: "column" }}>
              <span style={{ fontSize: 14, fontWeight: 500 }}>{est.nome}</span>
              <span style={{ fontSize: 12, color: caixa.aberta ? "#6EE7A0" : "#FCA5A5" }}>
                Caixa ·{" "}
                {caixa.aberta && resumo?.sessao_aberta ? "aberta desde " + horaMinuto(resumo.sessao_aberta.aberta_em) : "fechada"}
              </span>
            </span>
            <span style={{ color: "#8C98B0" }}>▾</span>
          </button>
        </div>
        <nav style={{ flex: 1, overflow: "auto", padding: "0 8px 16px", display: "flex", flexDirection: "column", gap: 16 }}>
          {nav.map((g) => (
            <div key={g.label} style={{ display: "flex", flexDirection: "column", gap: 1 }}>
              <div className="kicker" style={{ color: "#8C98B0", padding: "0 10px 4px" }}>
                {g.label}
              </div>
              {g.items.map((it) => {
                const ativo = pagina === it.pg && (it.pg !== "vazio" || vazioLabel === it.label);
                return (
                  <button
                    key={it.label}
                    className="nav-item"
                    onClick={() => {
                      setPagina(it.pg);
                      setVazioLabel(it.label);
                    }}
                    style={{
                      boxShadow: ativo ? "inset 3px 0 0 #3B82F6" : "none",
                      background: ativo ? "rgba(255,255,255,.08)" : "transparent",
                      color: ativo ? "#fff" : "#B4BED1",
                    }}
                  >
                    <span>{it.label}</span>
                    <span style={{ fontSize: 12, color: "#8C98B0" }}>{it.count ?? ""}</span>
                  </button>
                );
              })}
            </div>
          ))}
        </nav>
        <div style={{ borderTop: "1px solid rgba(255,255,255,.1)", padding: "12px 16px", display: "flex", flexDirection: "column", gap: 8 }}>
          <div style={{ display: "flex", alignItems: "center", gap: 10 }}>
            <span className="avatar" style={{ width: 38, height: 38, borderRadius: "50%", fontWeight: 600, fontSize: 13 }}>
              {iniciais(p.operador.nome)}
            </span>
            <span style={{ display: "flex", flexDirection: "column" }}>
              <span style={{ fontSize: 14, fontWeight: 500 }}>{p.operador.nome}</span>
              <span style={{ fontSize: 12, color: "#8C98B0" }}>
                {p.operador.papel ?? "Operador"} · turno desde {horaMinuto(p.turnoDesde)}
              </span>
            </span>
          </div>
          <div style={{ display: "grid", gridTemplateColumns: "1fr 1fr", gap: 8 }}>
            <button onClick={p.onTrocar} className="btn btn-secondary" style={{ minHeight: 44, whiteSpace: "nowrap" }}>
              Trocar
            </button>
            <button onClick={p.onBloquear} className="btn btn-secondary" style={{ minHeight: 44, whiteSpace: "nowrap" }}>
              Bloquear · F12
            </button>
          </div>
          <div className="kicker" style={{ color: "#8C98B0" }}>
            Bloqueio automático após 45 min sem atividade
          </div>
        </div>
      </aside>

      <main style={{ flex: 1, minWidth: 0, display: "flex", flexDirection: "column", minHeight: 0 }}>
        <header
          style={{
            height: 60,
            flex: "none",
            background: "#fff",
            display: "flex",
            alignItems: "center",
            gap: 16,
            padding: "0 24px",
            borderBottom: "1px solid var(--color-divider)",
          }}
        >
          <div style={{ display: "flex", alignItems: "baseline", gap: 12, minWidth: 0 }}>
            <h2 style={{ margin: 0, fontSize: 18, fontWeight: 600 }}>{titulos[pagina][0]}</h2>
            <span className="muted" style={{ fontSize: 13 }}>
              {titulos[pagina][1]}
            </span>
          </div>
          <div style={{ marginLeft: "auto", display: "flex", alignItems: "center", gap: 16 }}>
            <span
              title="A comunicação com a AGT ainda não está implementada"
              style={{ display: "flex", alignItems: "center", gap: 8, minHeight: 36, padding: "0 12px", border: "1px solid var(--color-divider)", fontSize: 13 }}
            >
              <span style={{ width: 8, height: 8, borderRadius: "50%", background: "#98A2B3" }} />
              Local
              <span className="muted">sem comunicação AGT</span>
            </span>
            <span style={{ fontSize: 15, fontWeight: 500 }}>{horaMinuto(agora)}</span>
          </div>
        </header>
        {pagina === "venda" && !caixa.aberta && ticket.tipo === "fatura_recibo" && (
          <div style={{ flex: "none", display: "flex", alignItems: "center", gap: 10, padding: "10px 24px", background: "#FDF1E0", color: "#B45309" }}>
            <strong>Caixa fechada.</strong>
            <span>Abra a caixa deste estabelecimento para emitir faturas-recibo.</span>
            <button className="btn btn-ghost" style={{ color: "#B45309" }} onClick={() => setPagina("caixa")}>
              Ir para Caixa
            </button>
          </div>
        )}
        <div style={{ flex: 1, minHeight: 0, overflow: "auto", display: "flex", flexDirection: "column" }}>
          {pagina === "venda" && (
            <Venda
              produtos={produtos}
              clientes={clientes}
              linhas={ticket.linhas}
              setLinhas={(fn) => setTicket((t) => ({ ...t, linhas: fn(t.linhas) }))}
              tipo={ticket.tipo}
              setTipo={(tipo) => setTicket((t) => ({ ...t, tipo }))}
              clienteId={ticket.clienteId}
              setClienteId={(clienteId) => setTicket((t) => ({ ...t, clienteId }))}
              metodo={metodo}
              setMetodo={setMetodo}
              onAbrirCheck={abrirCheck}
              searchRef={searchRef}
            />
          )}
          {pagina === "caixa" && <Caixa key={p.estId} resumo={resumo} onFechar={pedirFecho} onAbrir={abrirCaixa} />}
          {pagina === "produtos" && p.empresa && (
            <Produtos empresaId={p.empresa.id} produtos={produtos} onCriado={carregarProdutos} />
          )}
          {pagina === "vazio" && (
            <div className="muted" style={{ flex: 1, display: "grid", placeItems: "center", textAlign: "center", padding: 48 }}>
              {vazioLabel} ainda não está ligado ao backend nesta versão.
            </div>
          )}
        </div>
      </main>

      {modal === "check" && (
        <CheckModal
          empresa={p.empresa}
          estName={est.nome}
          tipo={ticket.tipo}
          cliente={cliente}
          linhas={ticket.linhas}
          metodo={metodo}
          caixaAberta={caixa.aberta}
          aEmitir={aEmitir}
          onEmitir={emitir}
          onClose={() => setModal(null)}
        />
      )}

      {modal === "est" && (
        <div className="overlay">
          <div className="dialog" style={{ width: "min(560px,100%)" }}>
            <span className="dialog-title" style={{ fontSize: 26 }}>
              Mudar de estabelecimento
            </span>
            <span className="muted" style={{ fontSize: 13 }}>
              O ticket e a caixa de cada estabelecimento ficam como estão.
            </span>
            {p.estabelecimentos.map((e) => {
              const st = estadoCaixa(p.resumos[e.id]);
              const n = tickets[e.id]?.linhas.length ?? 0;
              return (
                <button
                  key={e.id}
                  onClick={() => {
                    p.setEstId(e.id);
                    setModal(null);
                    setToast({ msg: e.nome + " · sessão e caixa próprias" });
                  }}
                  className="card-btn"
                  style={{
                    display: "flex",
                    alignItems: "center",
                    gap: 12,
                    padding: "12px 14px",
                    minHeight: 68,
                    borderRadius: 3,
                    border: "1px solid " + (e.id === p.estId ? "#1D4ED8" : "var(--color-divider)"),
                  }}
                >
                  <span className="avatar" style={{ width: 40, height: 40 }}>
                    {iniciaisEstabelecimento(e.nome)}
                  </span>
                  <span style={{ flex: 1, display: "flex", flexDirection: "column" }}>
                    <span style={{ fontSize: 15, fontWeight: 500 }}>{e.nome}</span>
                    <span style={{ fontSize: 12, color: st.cor }}>
                      Caixa · {st.texto} · {fmt(recebidoNaSessao(p.resumos[e.id]))}
                    </span>
                  </span>
                  <span className="muted" style={{ fontSize: 12 }}>
                    {n ? n + " linha(s) no ticket" : ""}
                  </span>
                </button>
              );
            })}
            <div className="dialog-actions">
              <button onClick={() => setModal(null)} className="btn btn-secondary" style={{ minHeight: 44 }}>
                Fechar
              </button>
            </div>
          </div>
        </div>
      )}

      {reauth && (
        <ReauthModal
          label={reauth.label}
          userName={p.operador.nome}
          estName={est.nome}
          onConfirm={reauth.acao}
          onCancel={() => setReauth(null)}
        />
      )}

      <Toast toast={toast} onClose={fecharToast} />
    </div>
  );
}
