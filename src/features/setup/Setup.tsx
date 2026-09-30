import { useState } from "react";
import { BrandPanel } from "../../components/BrandPanel";
import * as api from "../../lib/api";
import { erroTexto } from "../../lib/format";
import type { Empresa, Estabelecimento, Operador } from "../../lib/types";

interface Props {
  empresa: Empresa | null;
  operadores: Operador[];
  estabelecimentos: Estabelecimento[];
  onConcluido: () => void;
}

type Passo = "empresa" | "admin" | "est";

/**
 * Configuração inicial numa instalação nova: empresa → primeiro
 * administrador (criar_primeiro_utilizador só funciona com a tabela de
 * utilizadores vazia) → estabelecimentos. Se já houver utilizadores mas
 * nenhum estabelecimento, o passo "admin" pede o login de um administrador.
 */
export function Setup(p: Props) {
  const [empresa, setEmpresa] = useState<Empresa | null>(p.empresa);
  const [passo, setPasso] = useState<Passo>(!p.empresa ? "empresa" : "admin");
  const [criados, setCriados] = useState<Estabelecimento[]>(p.estabelecimentos);
  const [password, setPassword] = useState("");
  const [erro, setErro] = useState<string | null>(null);
  const [aEnviar, setAEnviar] = useState(false);
  const jaHaUtilizadores = p.operadores.length > 0;

  // Campos dos formulários
  const [f, setF] = useState({
    empNome: "",
    empNif: "",
    empMorada: "",
    perfil: "completo" as "simples" | "completo",
    admNome: "",
    admEmail: "",
    admPwd: "",
    admPwd2: "",
    estNome: "",
    estTipo: "",
    estEndereco: "",
  });
  const set = (k: keyof typeof f) => (e: { target: { value: string } }) =>
    setF((x) => ({ ...x, [k]: e.target.value }));

  async function correr(fn: () => Promise<void>) {
    setAEnviar(true);
    setErro(null);
    try {
      await fn();
    } catch (e) {
      setErro(erroTexto(e));
    } finally {
      setAEnviar(false);
    }
  }

  const criarEmpresa = () =>
    correr(async () => {
      const e = await api.criarEmpresa({
        nome: f.empNome.trim(),
        nif: f.empNif.trim(),
        morada: f.empMorada.trim() || null,
        perfilComplexidade: f.perfil,
      });
      setEmpresa(e);
      setPasso("admin");
    });

  const criarAdmin = () =>
    correr(async () => {
      if (!jaHaUtilizadores) {
        if (f.admPwd.length < 8) throw "A password deve ter pelo menos 8 caracteres.";
        if (f.admPwd !== f.admPwd2) throw "As passwords não coincidem.";
        await api.criarPrimeiroUtilizador(f.admNome.trim(), f.admEmail.trim(), f.admPwd);
      }
      await api.login(f.admEmail.trim(), f.admPwd);
      setPassword(f.admPwd);
      setPasso("est");
    });

  const criarEst = () =>
    correr(async () => {
      if (!empresa) throw "Empresa em falta.";
      const e = await api.criarEstabelecimento({
        empresaId: empresa.id,
        nome: f.estNome.trim(),
        tipoNegocio: f.estTipo.trim() || null,
        endereco: f.estEndereco.trim() || null,
        passwordConfirmacao: password,
      });
      setCriados((c) => [...c, e]);
      setF((x) => ({ ...x, estNome: "", estTipo: "", estEndereco: "" }));
    });

  const concluir = () =>
    correr(async () => {
      // Termina a sessão do administrador — o turno começa no ecrã de login.
      await api.logout();
      p.onConcluido();
    });

  const passos: Passo[] = ["empresa", "admin", "est"];

  return (
    <div style={{ flex: 1, display: "grid", gridTemplateColumns: "400px minmax(0,1fr)", minHeight: 0 }}>
      <BrandPanel empresa={empresa} />
      <div style={{ padding: "48px 56px", display: "flex", flexDirection: "column", gap: 24, overflow: "auto" }}>
        <div className="kicker">
          Configuração inicial · passo {passos.indexOf(passo) + 1} de 3
        </div>

        {passo === "empresa" && (
          <Form titulo="Dados da empresa" sub="Aparecem em todos os documentos emitidos.">
            <Campo label="Nome da empresa">
              <input className="input" value={f.empNome} onChange={set("empNome")} autoFocus />
            </Campo>
            <Campo label="NIF">
              <input className="input" value={f.empNif} onChange={set("empNif")} />
            </Campo>
            <Campo label="Morada (opcional)">
              <input className="input" value={f.empMorada} onChange={set("empMorada")} />
            </Campo>
            <Campo label="Perfil">
              <select className="input" value={f.perfil} onChange={set("perfil")}>
                <option value="completo">Completo — ciclo comercial completo</option>
                <option value="simples">Simples — só venda a dinheiro</option>
              </select>
            </Campo>
            <Acao disabled={!f.empNome.trim() || !f.empNif.trim() || aEnviar} onClick={criarEmpresa}>
              Continuar
            </Acao>
          </Form>
        )}

        {passo === "admin" && (
          <Form
            titulo={jaHaUtilizadores ? "Entrar como administrador" : "Primeiro administrador"}
            sub={
              jaHaUtilizadores
                ? "É preciso um administrador para criar estabelecimentos."
                : "Tem acesso a tudo, incluindo gerir utilizadores e configuração fiscal."
            }
          >
            {!jaHaUtilizadores && (
              <Campo label="Nome">
                <input className="input" value={f.admNome} onChange={set("admNome")} autoFocus />
              </Campo>
            )}
            <Campo label="E-mail">
              <input className="input" value={f.admEmail} onChange={set("admEmail")} />
            </Campo>
            <Campo label="Password">
              <input type="password" className="input" value={f.admPwd} onChange={set("admPwd")} />
            </Campo>
            {!jaHaUtilizadores && (
              <Campo label="Repetir password">
                <input type="password" className="input" value={f.admPwd2} onChange={set("admPwd2")} />
              </Campo>
            )}
            <Acao
              disabled={(!jaHaUtilizadores && !f.admNome.trim()) || !f.admEmail.trim() || !f.admPwd || aEnviar}
              onClick={criarAdmin}
            >
              Continuar
            </Acao>
          </Form>
        )}

        {passo === "est" && (
          <Form
            titulo="Estabelecimentos"
            sub="Cada estabelecimento tem a sua própria sessão de caixa e o seu próprio fecho."
          >
            {criados.length > 0 && (
              <div style={{ display: "flex", flexDirection: "column", gap: 6 }}>
                {criados.map((e) => (
                  <div key={e.id} className="blueprint" style={{ padding: "10px 14px" }}>
                    <strong>{e.nome}</strong>
                    <span className="muted"> {e.endereco ? "· " + e.endereco : ""}</span>
                  </div>
                ))}
              </div>
            )}
            <Campo label="Nome">
              <input className="input" value={f.estNome} onChange={set("estNome")} autoFocus placeholder="Ex.: Pastelaria Doce Lar" />
            </Campo>
            <Campo label="Tipo de negócio (opcional)">
              <input className="input" value={f.estTipo} onChange={set("estTipo")} placeholder="Ex.: pastelaria" />
            </Campo>
            <Campo label="Endereço (opcional)">
              <input className="input" value={f.estEndereco} onChange={set("estEndereco")} />
            </Campo>
            <div style={{ display: "flex", gap: 12 }}>
              <button
                className="btn btn-secondary"
                style={{ minHeight: 48 }}
                disabled={!f.estNome.trim() || aEnviar}
                onClick={criarEst}
              >
                Adicionar estabelecimento
              </button>
              <Acao disabled={criados.length === 0 || aEnviar} onClick={concluir}>
                Concluir
              </Acao>
            </div>
          </Form>
        )}

        {erro && <div className="error-text">{erro}</div>}
      </div>
    </div>
  );
}

function Form({ titulo, sub, children }: { titulo: string; sub: string; children: React.ReactNode }) {
  return (
    <div style={{ display: "flex", flexDirection: "column", gap: 16, width: 440 }}>
      <div style={{ marginTop: -12 }}>
        <h1 style={{ margin: 0, fontSize: 32 }}>{titulo}</h1>
        <div className="muted" style={{ fontSize: 15 }}>
          {sub}
        </div>
      </div>
      {children}
    </div>
  );
}

function Campo({ label, children }: { label: string; children: React.ReactNode }) {
  return (
    <div className="field">
      <label>{label}</label>
      {children}
    </div>
  );
}

function Acao({ children, ...rest }: { children: React.ReactNode; disabled: boolean; onClick: () => void }) {
  return (
    <button className="btn btn-primary" style={{ minHeight: 48, fontSize: 15, alignSelf: "flex-start", padding: "0 24px" }} {...rest}>
      {children}
    </button>
  );
}
