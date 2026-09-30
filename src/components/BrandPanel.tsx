import { useEffect, useState } from "react";
import { dataCurta, horaMinuto } from "../lib/format";
import type { Empresa } from "../lib/types";

/** Painel escuro à esquerda dos ecrãs de login e de configuração inicial. */
export function BrandPanel({ empresa }: { empresa: Empresa | null }) {
  const agora = useRelogio();
  return (
    <div
      style={{
        background: "#0B1730",
        color: "#fff",
        padding: 40,
        borderRight: "4px solid #1D4ED8",
        display: "flex",
        flexDirection: "column",
        gap: 10,
      }}
    >
      <div className="mono-total" style={{ fontSize: 26 }}>
        Série
      </div>
      <div style={{ fontSize: 15, color: "#BFD3FB" }}>Faturação para o balcão</div>
      <div style={{ marginTop: "auto", display: "flex", flexDirection: "column", gap: 6, fontSize: 14, color: "#E8EEFD" }}>
        {empresa && (
          <>
            <div style={{ fontSize: 16, color: "#fff", fontWeight: 500 }}>{empresa.nome}</div>
            <div>
              NIF {empresa.nif}
              {empresa.morada ? " · " + empresa.morada : ""}
            </div>
          </>
        )}
        <div>
          {horaMinuto(agora)} · {dataCurta(agora)}
        </div>
        <div style={{ marginTop: 16, fontSize: 12, color: "#93B4F5" }}>
          Software certificado n.º [a definir] · versão 0.1
        </div>
      </div>
    </div>
  );
}

export function useRelogio(): Date {
  const [agora, setAgora] = useState(() => new Date());
  useEffect(() => {
    const t = setInterval(() => setAgora(new Date()), 15000);
    return () => clearInterval(t);
  }, []);
  return agora;
}
