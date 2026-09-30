import { useCallback, useEffect, useRef, useState } from "react";
import { Login, estabelecimentosDoOperador } from "./features/login/Login";
import { Setup } from "./features/setup/Setup";
import { Shell } from "./features/shell/Shell";
import * as api from "./lib/api";
import { erroTexto } from "./lib/format";
import type { Empresa, Estabelecimento, Operador, ResumoCaixa } from "./lib/types";
import "./styles/app-corp.css";
import "./App.css";

type Fase = "carregar" | "setup" | "login" | "app";

/** Intervalo mínimo entre avisos de atividade ao backend (tocar_atividade). */
const TOQUE_MS = 30_000;

function App() {
  const [fase, setFase] = useState<Fase>("carregar");
  const [erro, setErro] = useState<string | null>(null);
  const [empresa, setEmpresa] = useState<Empresa | null>(null);
  const [operadores, setOperadores] = useState<Operador[]>([]);
  const [estabelecimentos, setEstabelecimentos] = useState<Estabelecimento[]>([]);
  const [resumos, setResumos] = useState<Record<string, ResumoCaixa>>({});
  const [operador, setOperador] = useState<Operador | null>(null);
  const [estId, setEstId] = useState<string>("");
  const [turnoDesde, setTurnoDesde] = useState(() => new Date());
  const [bloqueado, setBloqueado] = useState(false);
  const ultimoToque = useRef(0);

  const refrescarResumo = useCallback(async (id: string) => {
    try {
      const r = await api.obterResumoCaixa(id);
      setResumos((rs) => ({ ...rs, [id]: r }));
    } catch (e) {
      setErro(erroTexto(e));
    }
  }, []);

  const arrancar = useCallback(async () => {
    try {
      const [emps, ops, ests, sessao] = await Promise.all([
        api.listarEmpresas(),
        api.listarOperadores(),
        api.listarEstabelecimentos(),
        api.estadoSessao(),
      ]);
      setEmpresa(emps[0] ?? null);
      setOperadores(ops);
      setEstabelecimentos(ests);
      ests.forEach((e) => refrescarResumo(e.id));

      if (!emps.length || !ops.length || !ests.length) {
        setFase("setup");
        return;
      }
      // Recarregar a webview não termina o turno: a sessão vive no backend.
      const op = sessao ? ops.find((o) => o.id === sessao.utilizador_id) : undefined;
      const permitidos = op ? estabelecimentosDoOperador(op, ests) : [];
      if (sessao && op && permitidos.length) {
        setOperador(op);
        setEstId(permitidos[0].id);
        setBloqueado(sessao.bloqueada);
        setFase("app");
      } else {
        setFase("login");
      }
    } catch (e) {
      setErro(erroTexto(e));
    }
  }, [refrescarResumo]);

  useEffect(() => {
    arrancar();
  }, [arrancar]);

  // Inatividade: o backend decide quando a sessão bloqueia (45 min). Aqui
  // só se avisa de atividade (com limite de frequência) e se consulta o
  // estado periodicamente para mostrar o ecrã de bloqueio.
  useEffect(() => {
    if (fase !== "app" || bloqueado) return;
    function aoInteragir() {
      const agora = Date.now();
      if (agora - ultimoToque.current < TOQUE_MS) return;
      ultimoToque.current = agora;
      api.tocarAtividade().then((s) => s?.bloqueada && setBloqueado(true));
    }
    const verificar = setInterval(() => {
      api.estadoSessao().then((s) => {
        if (!s) {
          setOperador(null);
          setFase("login");
        } else if (s.bloqueada) setBloqueado(true);
      });
    }, TOQUE_MS);
    window.addEventListener("pointerdown", aoInteragir);
    window.addEventListener("keydown", aoInteragir);
    return () => {
      clearInterval(verificar);
      window.removeEventListener("pointerdown", aoInteragir);
      window.removeEventListener("keydown", aoInteragir);
    };
  }, [fase, bloqueado]);

  async function bloquear() {
    try {
      await api.bloquearSessao();
      setBloqueado(true);
    } catch (e) {
      setErro(erroTexto(e));
    }
  }

  async function trocarUtilizador() {
    await api.logout().catch(() => {});
    setOperador(null);
    setBloqueado(false);
    setFase("login");
  }

  if (erro && fase === "carregar") {
    return (
      <div className="app-root" style={{ placeItems: "center", display: "grid" }}>
        <div className="error-text">Não foi possível iniciar: {erro}</div>
      </div>
    );
  }

  const permitidos = operador ? estabelecimentosDoOperador(operador, estabelecimentos) : [];

  return (
    <div className="app-root">
      {fase === "setup" && (
        <Setup
          empresa={empresa}
          operadores={operadores}
          estabelecimentos={estabelecimentos}
          onConcluido={() => {
            setFase("carregar");
            arrancar();
          }}
        />
      )}

      {fase === "login" && (
        <Login
          empresa={empresa}
          operadores={operadores}
          estabelecimentos={estabelecimentos}
          resumos={resumos}
          bloqueado={null}
          onEntrou={(op, id) => {
            setOperador(op);
            setEstId(id);
            setTurnoDesde(new Date());
            setBloqueado(false);
            ultimoToque.current = Date.now();
            setFase("app");
          }}
          onDesbloqueado={() => {}}
          onTrocarUtilizador={() => {}}
        />
      )}

      {fase === "app" && operador && (
        <>
          {/* O Shell fica montado durante o bloqueio: o ticket em curso não se perde. */}
          <div style={{ flex: 1, display: bloqueado ? "none" : "flex", minHeight: 0 }}>
            <Shell
              empresa={empresa}
              estabelecimentos={permitidos}
              operador={operador}
              turnoDesde={turnoDesde}
              estId={estId}
              setEstId={setEstId}
              resumos={resumos}
              refrescarResumo={refrescarResumo}
              ativo={!bloqueado}
              onBloquear={bloquear}
              onTrocar={trocarUtilizador}
            />
          </div>
          {bloqueado && (
            <Login
              empresa={empresa}
              operadores={operadores}
              estabelecimentos={estabelecimentos}
              resumos={resumos}
              bloqueado={operador}
              onEntrou={() => {}}
              onDesbloqueado={() => {
                ultimoToque.current = Date.now();
                setBloqueado(false);
              }}
              onTrocarUtilizador={trocarUtilizador}
            />
          )}
        </>
      )}
    </div>
  );
}

export default App;
