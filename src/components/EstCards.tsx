import { fmt, horaMinuto } from "../lib/format";
import { iniciaisEstabelecimento } from "../lib/dominio";
import type { Estabelecimento, ResumoCaixa } from "../lib/types";

export function estadoCaixa(r: ResumoCaixa | undefined) {
  const s = r?.sessao_aberta;
  if (s) {
    return {
      aberta: true,
      texto: "Aberta desde " + horaMinuto(s.aberta_em) + (r?.aberta_por_nome ? " · " + r.aberta_por_nome : ""),
      cor: "#15803D",
    };
  }
  const f = r?.ultima_fechada;
  return {
    aberta: false,
    texto: "Fechada" + (f?.fechada_em ? " às " + horaMinuto(f.fechada_em) : ""),
    cor: "#B91C1C",
  };
}

export function recebidoNaSessao(r: ResumoCaixa | undefined): number {
  return (r?.totais ?? []).reduce((a, t) => a + t.total_centimos, 0);
}

/** Cartões grandes do passo "Onde vai trabalhar?". */
export function EstCardsGrandes({
  estabelecimentos,
  resumos,
  onPick,
}: {
  estabelecimentos: Estabelecimento[];
  resumos: Record<string, ResumoCaixa>;
  onPick: (id: string) => void;
}) {
  return (
    <div style={{ display: "grid", gridTemplateColumns: "repeat(2,minmax(0,380px))", gap: 28 }}>
      {estabelecimentos.map((e) => {
        const r = resumos[e.id];
        const st = estadoCaixa(r);
        return (
          <button
            key={e.id}
            onClick={() => onPick(e.id)}
            className="blueprint card-btn"
            style={{ display: "flex", flexDirection: "column", gap: 14, minHeight: 230, padding: 24 }}
          >
            <span style={{ display: "flex", alignItems: "center", gap: 14 }}>
              <span className="avatar heading-num" style={{ width: 52, height: 52, fontSize: 20 }}>
                {iniciaisEstabelecimento(e.nome)}
              </span>
              <span style={{ display: "flex", flexDirection: "column" }}>
                <span className="heading-num" style={{ fontSize: 26, lineHeight: 1.1 }}>
                  {e.nome}
                </span>
                <span className="muted" style={{ fontSize: 13 }}>
                  {e.endereco ?? e.tipo_negocio ?? ""}
                </span>
              </span>
            </span>
            <span
              style={{
                marginTop: "auto",
                display: "grid",
                gridTemplateColumns: "1fr 1fr",
                gap: 12,
                paddingTop: 16,
                borderTop: "1px solid var(--color-divider)",
                width: "100%",
              }}
            >
              <span style={{ display: "flex", flexDirection: "column", gap: 2 }}>
                <span className="kicker">Caixa</span>
                <span style={{ fontWeight: 500, color: st.cor }}>{st.texto}</span>
              </span>
              <span style={{ display: "flex", flexDirection: "column", gap: 2 }}>
                <span className="kicker">Recebido na sessão</span>
                <span className="heading-num" style={{ fontSize: 20 }}>
                  {fmt(recebidoNaSessao(r))}
                </span>
              </span>
            </span>
          </button>
        );
      })}
    </div>
  );
}
