// Invocações dos comandos Tauri — a lista autorizada está em
// src-tauri/src/lib.rs (invoke_handler!). O Tauri converte os argumentos
// camelCase daqui para os parâmetros snake_case do Rust.
import { invoke } from "@tauri-apps/api/core";
import type {
  Cliente,
  Documento,
  Empresa,
  EstadoSessao,
  Estabelecimento,
  NovaLinhaDocumento,
  Operador,
  ProdutoServico,
  ResultadoEmissao,
  ResumoCaixa,
  SessaoCaixa,
  TipoDocumento,
  UtilizadorPublico,
} from "./types";

// Autenticação --------------------------------------------------------------

export const listarOperadores = () => invoke<Operador[]>("listar_operadores");

export const login = (email: string, password: string) =>
  invoke<UtilizadorPublico>("login", { email, password });

export const logout = () => invoke<void>("logout");

export const estadoSessao = () => invoke<EstadoSessao | null>("estado_sessao");

export const tocarAtividade = () => invoke<EstadoSessao | null>("tocar_atividade");

export const bloquearSessao = () => invoke<void>("bloquear_sessao");

export const desbloquearSessao = (password: string) =>
  invoke<void>("desbloquear_sessao", { password });

export const criarPrimeiroUtilizador = (nome: string, email: string, password: string) =>
  invoke<UtilizadorPublico>("criar_primeiro_utilizador", { nome, email, password });

// Empresa e estabelecimentos ------------------------------------------------

export const listarEmpresas = () => invoke<Empresa[]>("listar_empresas");

export const criarEmpresa = (a: {
  nome: string;
  nif: string;
  morada: string | null;
  perfilComplexidade: "simples" | "completo";
}) => invoke<Empresa>("criar_empresa", { ...a, moeda: "AOA" });

export const listarEstabelecimentos = () => invoke<Estabelecimento[]>("listar_estabelecimentos");

export const criarEstabelecimento = (a: {
  empresaId: string;
  nome: string;
  tipoNegocio: string | null;
  endereco: string | null;
  passwordConfirmacao: string;
}) => invoke<Estabelecimento>("criar_estabelecimento", a);

// Produtos e clientes -------------------------------------------------------

export const listarProdutos = () => invoke<ProdutoServico[]>("listar_produtos_servicos");

export const criarProduto = (a: {
  empresaId: string;
  nome: string;
  tipo: "produto" | "servico";
  unidadeMedida: string;
  precoBaseCentimos: number;
}) =>
  invoke<ProdutoServico>("criar_produto_servico", {
    ...a,
    descricao: null,
    taxaImpostoId: null,
    controlaStock: false,
  });

export const listarClientes = () => invoke<Cliente[]>("listar_clientes");

// Sessões de caixa ----------------------------------------------------------

export const obterResumoCaixa = (estabelecimentoId: string) =>
  invoke<ResumoCaixa>("obter_resumo_caixa", { estabelecimentoId });

export const abrirSessaoCaixa = (estabelecimentoId: string, fundoManeioInicialCentimos: number) =>
  invoke<SessaoCaixa>("abrir_sessao_caixa", { estabelecimentoId, fundoManeioInicialCentimos });

export const fecharSessaoCaixa = (
  sessaoCaixaId: string,
  contagemFisicaDinheiroCentimos: number,
  passwordConfirmacao: string,
) =>
  invoke<SessaoCaixa>("fechar_sessao_caixa", {
    sessaoCaixaId,
    contagemFisicaDinheiroCentimos,
    passwordConfirmacao,
  });

export const registarPagamento = (a: {
  documentoId: string;
  sessaoCaixaId: string;
  metodoPagamento: string;
  valorCentimos: number;
}) => invoke<void>("registar_pagamento", a);

// Documentos ----------------------------------------------------------------

export const criarDocumentoRascunho = (
  estabelecimentoId: string,
  tipo: TipoDocumento,
  clienteId: string | null,
) =>
  invoke<Documento>("criar_documento_rascunho", {
    estabelecimentoId,
    clienteId,
    tipo,
    documentoOrigemId: null,
    observacoes: null,
    condicoesPagamento: null,
    dataVencimento: null,
  });

export const adicionarLinhaDocumento = (documentoId: string, linha: NovaLinhaDocumento) =>
  invoke<unknown>("adicionar_linha_documento", { documentoId, linha });

export const emitirDocumento = (documentoId: string) =>
  invoke<ResultadoEmissao>("emitir_documento", { documentoId });
