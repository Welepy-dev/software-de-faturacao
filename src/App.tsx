import { useEffect, useState } from "react";
import { listarEstabelecimentos } from "./features/estabelecimentos/api";
import { SessaoCaixaPanel } from "./features/caixa/SessaoCaixaPanel";
import type { Estabelecimento } from "./lib/types";
import "./App.css";

function App() {
  const [estabelecimentos, setEstabelecimentos] = useState<Estabelecimento[]>([]);
  const [erro, setErro] = useState<string | null>(null);

  useEffect(() => {
    listarEstabelecimentos()
      .then(setEstabelecimentos)
      .catch((e) => setErro(String(e)));
  }, []);

  return (
    <main className="container">
      <h1>Faturação</h1>

      {erro && <p style={{ color: "red" }}>{erro}</p>}

      {estabelecimentos.length === 0 ? (
        <p>
          Nenhum estabelecimento cadastrado ainda. (O CRUD de Empresa/Estabelecimento ainda não
          está implementado neste scaffold — ver docs/ARQUITETURA.md §8.)
        </p>
      ) : (
        estabelecimentos.map((estabelecimento) => (
          <SessaoCaixaPanel key={estabelecimento.id} estabelecimento={estabelecimento} />
        ))
      )}
    </main>
  );
}

export default App;
