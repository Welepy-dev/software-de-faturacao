import { useState } from "react";
import * as api from "../../lib/api";
import { erroTexto, fmt, parseKz } from "../../lib/format";
import type { ProdutoServico } from "../../lib/types";

interface Props {
  empresaId: string;
  produtos: ProdutoServico[];
  onCriado: () => void;
}

/**
 * Lista e criação mínima de produtos — no design v3 "Produtos e stock"
 * ainda não tinha mockup; existe para a Venda rápida ter o que vender.
 */
export function Produtos({ empresaId, produtos, onCriado }: Props) {
  const [nome, setNome] = useState("");
  const [preco, setPreco] = useState("");
  const [unidade, setUnidade] = useState("un");
  const [tipo, setTipo] = useState<"produto" | "servico">("produto");
  const [erro, setErro] = useState<string | null>(null);
  const [aEnviar, setAEnviar] = useState(false);

  async function criar() {
    setAEnviar(true);
    setErro(null);
    try {
      await api.criarProduto({
        empresaId,
        nome: nome.trim(),
        tipo,
        unidadeMedida: unidade.trim() || "un",
        precoBaseCentimos: parseKz(preco),
      });
      setNome("");
      setPreco("");
      onCriado();
    } catch (e) {
      setErro(erroTexto(e));
    } finally {
      setAEnviar(false);
    }
  }

  return (
    <div style={{ padding: "20px 24px 32px", display: "grid", gridTemplateColumns: "minmax(0,1fr) 360px", gap: 40, alignItems: "start" }}>
      <table className="table">
        <thead>
          <tr>
            <th>Nome</th>
            <th>Tipo</th>
            <th>Unidade</th>
            <th style={{ textAlign: "right" }}>Preço (sem IVA)</th>
          </tr>
        </thead>
        <tbody>
          {produtos.map((p) => (
            <tr key={p.id} style={{ height: 46 }}>
              <td style={{ fontWeight: 500 }}>{p.nome}</td>
              <td>{p.tipo === "produto" ? "Produto" : "Serviço"}</td>
              <td className="muted">{p.unidade_medida}</td>
              <td style={{ textAlign: "right", fontWeight: 500 }}>{fmt(p.preco_base_centimos)}</td>
            </tr>
          ))}
          {produtos.length === 0 && (
            <tr>
              <td colSpan={4} className="muted" style={{ padding: 32, textAlign: "center" }}>
                Ainda não há produtos.
              </td>
            </tr>
          )}
        </tbody>
      </table>
      <div className="blueprint" style={{ padding: 20, display: "flex", flexDirection: "column", gap: 14 }}>
        <h3 style={{ margin: 0 }}>Novo produto</h3>
        <div className="field">
          <label>Nome</label>
          <input className="input" value={nome} onChange={(e) => setNome(e.target.value)} />
        </div>
        <div style={{ display: "grid", gridTemplateColumns: "1fr 1fr", gap: 12 }}>
          <div className="field">
            <label>Preço sem IVA (Kz)</label>
            <input className="input" value={preco} onChange={(e) => setPreco(e.target.value)} style={{ textAlign: "right" }} />
          </div>
          <div className="field">
            <label>Unidade</label>
            <input className="input" value={unidade} onChange={(e) => setUnidade(e.target.value)} />
          </div>
        </div>
        <div className="field">
          <label>Tipo</label>
          <select className="input" value={tipo} onChange={(e) => setTipo(e.target.value as "produto" | "servico")}>
            <option value="produto">Produto</option>
            <option value="servico">Serviço</option>
          </select>
        </div>
        {erro && <div className="error-text">{erro}</div>}
        <button
          className="btn btn-primary"
          style={{ minHeight: 48 }}
          disabled={!nome.trim() || parseKz(preco) <= 0 || aEnviar}
          onClick={criar}
        >
          Criar produto
        </button>
      </div>
    </div>
  );
}
