import { useEffect, useState } from "react";
import type { Estabelecimento, SessaoCaixa } from "../../lib/types";
import { abrirSessaoCaixa, fecharSessaoCaixa, obterSessaoAberta } from "./api";

// Demonstra o fluxo que resolve os problemas 1 e 2 (docs/ARQUITETURA.md §4):
// uma sessão de caixa por estabelecimento, com fecho automático a partir dos
// totais acumulados incrementalmente. Substituir o utilizador fixo abaixo
// pelo utilizador autenticado assim que o login estiver implementado.
const UTILIZADOR_DEMO_ID = "utilizador-demo";

interface Props {
  estabelecimento: Estabelecimento;
}

export function SessaoCaixaPanel({ estabelecimento }: Props) {
  const [sessao, setSessao] = useState<SessaoCaixa | null>(null);
  const [carregando, setCarregando] = useState(true);
  const [erro, setErro] = useState<string | null>(null);

  useEffect(() => {
    obterSessaoAberta(estabelecimento.id)
      .then(setSessao)
      .catch((e) => setErro(String(e)))
      .finally(() => setCarregando(false));
  }, [estabelecimento.id]);

  async function abrir() {
    setErro(null);
    try {
      const nova = await abrirSessaoCaixa(estabelecimento.id, UTILIZADOR_DEMO_ID, 0);
      setSessao(nova);
    } catch (e) {
      setErro(String(e));
    }
  }

  async function fechar() {
    if (!sessao) return;
    setErro(null);
    try {
      const fechada = await fecharSessaoCaixa(sessao.id, UTILIZADOR_DEMO_ID, 0);
      setSessao(fechada.estado === "aberta" ? fechada : null);
    } catch (e) {
      setErro(String(e));
    }
  }

  if (carregando) return <p>A carregar sessão de {estabelecimento.nome}...</p>;

  return (
    <div style={{ border: "1px solid #ccc", borderRadius: 8, padding: 16, marginBottom: 12 }}>
      <h3>{estabelecimento.nome}</h3>
      {erro && <p style={{ color: "red" }}>{erro}</p>}
      {sessao ? (
        <>
          <p>Sessão aberta desde {new Date(sessao.aberta_em).toLocaleString("pt-AO")}</p>
          <button onClick={fechar}>Fechar sessão de caixa</button>
        </>
      ) : (
        <>
          <p>Sem sessão aberta.</p>
          <button onClick={abrir}>Abrir sessão de caixa</button>
        </>
      )}
    </div>
  );
}
