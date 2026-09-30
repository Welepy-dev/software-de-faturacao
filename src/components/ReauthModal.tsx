import { useState } from "react";
import { erroTexto } from "../lib/format";

interface Props {
  label: string;
  userName: string;
  estName: string;
  /** Executa a ação protegida com a password — o backend valida-a. */
  onConfirm: (password: string) => Promise<void>;
  onCancel: () => void;
}

/**
 * "Ação protegida" do design v3. Só o modo "A minha password": o backend
 * valida a password do operador autenticado em cada ação sensível
 * (permissoes::exigir_permissao) e ainda não suporta autorização por
 * supervisor.
 */
export function ReauthModal({ label, userName, estName, onConfirm, onCancel }: Props) {
  const [pwd, setPwd] = useState("");
  const [erro, setErro] = useState<string | null>(null);
  const [aEnviar, setAEnviar] = useState(false);

  async function confirmar() {
    if (!pwd || aEnviar) return;
    setAEnviar(true);
    setErro(null);
    try {
      await onConfirm(pwd);
    } catch (e) {
      setErro(erroTexto(e));
      setPwd("");
    } finally {
      setAEnviar(false);
    }
  }

  return (
    <div className="overlay" style={{ zIndex: 30, background: "rgba(23,32,51,.5)" }}>
      <div className="dialog" style={{ width: "min(460px,100%)", gap: 16 }}>
        <div style={{ display: "flex", flexDirection: "column", gap: 4 }}>
          <span className="kicker">Ação protegida</span>
          <span className="dialog-title" style={{ fontSize: 22 }}>
            {label}
          </span>
          <span className="muted" style={{ fontSize: 13 }}>
            Pedido por {userName} · {estName}. As restantes ações do turno continuam sem password.
          </span>
        </div>
        <div className="field">
          <label>Password de {userName}</label>
          <input
            type="password"
            className="input"
            autoFocus
            value={pwd}
            onChange={(e) => setPwd(e.target.value)}
            onKeyDown={(e) => {
              if (e.key === "Enter") confirmar();
              if (e.key === "Escape") onCancel();
            }}
            style={{ height: 48 }}
          />
          {erro && <div className="error-text" style={{ marginTop: 6 }}>{erro}</div>}
        </div>
        <button
          onClick={confirmar}
          disabled={!pwd || aEnviar}
          className="btn btn-primary"
          style={{ minHeight: 52, fontSize: 16 }}
        >
          {aEnviar ? "A confirmar…" : "Confirmar"}
        </button>
        <button onClick={onCancel} className="btn btn-ghost" style={{ minHeight: 44 }}>
          Cancelar
        </button>
        <span className="kicker">Fica na auditoria: ação, utilizador e hora.</span>
      </div>
    </div>
  );
}
