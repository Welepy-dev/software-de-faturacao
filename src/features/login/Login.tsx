import { useState } from "react";
import { BrandPanel } from "../../components/BrandPanel";
import { EstCardsGrandes } from "../../components/EstCards";
import * as api from "../../lib/api";
import { erroTexto, iniciais } from "../../lib/format";
import type { Empresa, Estabelecimento, Operador, ResumoCaixa } from "../../lib/types";

export function estabelecimentosDoOperador(op: Operador, todos: Estabelecimento[]) {
  return op.estabelecimento_ids.length === 0
    ? todos
    : todos.filter((e) => op.estabelecimento_ids.includes(e.id));
}

interface Props {
  empresa: Empresa | null;
  operadores: Operador[];
  estabelecimentos: Estabelecimento[];
  resumos: Record<string, ResumoCaixa>;
  /** Preenchido quando a sessão está bloqueada: só este operador desbloqueia. */
  bloqueado: Operador | null;
  onEntrou: (op: Operador, estId: string) => void;
  onDesbloqueado: () => void;
  onTrocarUtilizador: () => void;
}

type Passo = "user" | "pwd" | "est";

/**
 * Login "A · Utilizador primeiro" do design v3: quem está ao balcão →
 * password → onde vai trabalhar. O design usava PIN de 4 dígitos; aqui é a
 * password completa, como ficou decidido (docs/ARQUITETURA.md §5) e como o
 * backend exige.
 */
export function Login(p: Props) {
  const [passo, setPasso] = useState<Passo>(p.bloqueado ? "pwd" : "user");
  const [selId, setSelId] = useState<string | null>(p.bloqueado?.id ?? null);
  const [pwd, setPwd] = useState("");
  const [erro, setErro] = useState<string | null>(null);
  const [aEnviar, setAEnviar] = useState(false);

  const sel = p.operadores.find((o) => o.id === selId) ?? null;
  const ests = sel ? estabelecimentosDoOperador(sel, p.estabelecimentos) : [];

  async function confirmarPassword() {
    if (!sel || !pwd || aEnviar) return;
    setAEnviar(true);
    setErro(null);
    try {
      if (p.bloqueado) {
        await api.desbloquearSessao(pwd);
        p.onDesbloqueado();
        return;
      }
      await api.login(sel.email, pwd);
      if (ests.length === 1) p.onEntrou(sel, ests[0].id);
      else setPasso("est");
    } catch (e) {
      setErro(erroTexto(e));
      setPwd("");
    } finally {
      setAEnviar(false);
    }
  }

  const kicker = p.bloqueado
    ? "Sessão bloqueada · o trabalho em curso fica guardado"
    : { user: "Passo 1 de 3", pwd: "Passo 2 de 3", est: "Passo 3 de 3" }[passo];

  return (
    <div style={{ flex: 1, display: "grid", gridTemplateColumns: "400px minmax(0,1fr)", minHeight: 0 }}>
      <BrandPanel empresa={p.empresa} />
      <div style={{ padding: "48px 56px", display: "flex", flexDirection: "column", gap: 28, overflow: "auto" }}>
        <div className="kicker">{kicker}</div>

        {passo === "user" && (
          <>
            <h1 style={{ margin: "-20px 0 0", fontSize: 32 }}>Quem está ao balcão?</h1>
            <div style={{ display: "grid", gridTemplateColumns: "repeat(3,minmax(0,230px))", gap: 24 }}>
              {p.operadores.map((o) => (
                <button
                  key={o.id}
                  onClick={() => {
                    setSelId(o.id);
                    setPasso("pwd");
                    setPwd("");
                    setErro(null);
                  }}
                  className="blueprint card-btn"
                  style={{ display: "flex", flexDirection: "column", alignItems: "flex-start", gap: 14, padding: 20, minHeight: 180 }}
                >
                  <span className="avatar heading-num" style={{ width: 52, height: 52, fontSize: 22 }}>
                    {iniciais(o.nome)}
                  </span>
                  <span className="heading-num" style={{ fontSize: 24, lineHeight: 1.1 }}>
                    {o.nome}
                  </span>
                  <span className="muted" style={{ fontSize: 13, marginTop: "auto" }}>
                    {o.papel ?? "Sem papel"}
                    <br />
                    {estabelecimentosDoOperador(o, p.estabelecimentos)
                      .map((e) => e.nome)
                      .join(" · ")}
                  </span>
                </button>
              ))}
            </div>
          </>
        )}

        {passo === "pwd" && sel && (
          <div style={{ display: "flex", flexDirection: "column", gap: 22, width: 360 }}>
            <div style={{ display: "flex", flexDirection: "column", gap: 4 }}>
              <h1 style={{ margin: 0, fontSize: 32 }}>{sel.nome}</h1>
              <div className="muted">
                {sel.papel ?? "Operador"} · introduza a sua password
              </div>
            </div>
            <div className="field">
              <label>Password</label>
              <input
                type="password"
                className="input"
                autoFocus
                value={pwd}
                onChange={(e) => setPwd(e.target.value)}
                onKeyDown={(e) => e.key === "Enter" && confirmarPassword()}
                style={{ height: 52, fontSize: 18 }}
              />
              {erro && <div className="error-text" style={{ marginTop: 6 }}>{erro}</div>}
            </div>
            <button
              onClick={confirmarPassword}
              disabled={!pwd || aEnviar}
              className="btn btn-primary"
              style={{ minHeight: 56, fontSize: 17 }}
            >
              {aEnviar ? "A verificar…" : p.bloqueado ? "Desbloquear" : "Entrar"}
            </button>
            <button
              onClick={() => {
                if (p.bloqueado) p.onTrocarUtilizador();
                else {
                  setPasso("user");
                  setPwd("");
                  setErro(null);
                }
              }}
              className="btn btn-ghost"
              style={{ alignSelf: "flex-start", minHeight: 44 }}
            >
              Outro utilizador
            </button>
            <div className="muted" style={{ fontSize: 12 }}>
              Autentica uma vez por turno. Adicionar ou remover linhas nunca pede password.
            </div>
          </div>
        )}

        {passo === "est" && sel && (
          <>
            <div style={{ marginTop: -20 }}>
              <h1 style={{ margin: 0, fontSize: 32 }}>Onde vai trabalhar?</h1>
              <div className="muted" style={{ fontSize: 15 }}>
                Cada estabelecimento tem a sua sessão, a sua caixa e o seu fecho.
              </div>
            </div>
            {ests.length === 0 ? (
              <div className="muted">Não tem nenhum estabelecimento atribuído.</div>
            ) : (
              <EstCardsGrandes
                estabelecimentos={ests}
                resumos={p.resumos}
                onPick={(id) => p.onEntrou(sel, id)}
              />
            )}
          </>
        )}
      </div>
    </div>
  );
}
