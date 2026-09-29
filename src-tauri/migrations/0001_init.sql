-- Schema inicial — ver docs/ARQUITETURA.md secção 2 para o desenho completo.
--
-- Convenções:
--   * ids são TEXT (UUID v4) gerados na aplicação, não AUTOINCREMENT — evita
--     expor contagem de registos e facilita merge entre dispositivos no
--     cenário de contingência/sincronização.
--   * timestamps são TEXT em ISO 8601 (UTC).
--   * valores monetários são INTEGER em cêntimos (nunca REAL), para evitar
--     erros de arredondamento em cálculos fiscais.

PRAGMA foreign_keys = ON;

-- 2.1 Empresa ---------------------------------------------------------------
CREATE TABLE empresas (
    id                          TEXT PRIMARY KEY,
    nome                        TEXT NOT NULL,
    nif                         TEXT NOT NULL,
    morada                      TEXT,
    logotipo_path               TEXT,
    regime_fiscal               TEXT,
    moeda                       TEXT NOT NULL DEFAULT 'AOA',
    dados_bancarios             TEXT,
    condicoes_pagamento_padrao  TEXT,
    perfil_complexidade         TEXT NOT NULL DEFAULT 'simples'
                                CHECK (perfil_complexidade IN ('simples', 'completo')),
    criado_em                   TEXT NOT NULL,
    atualizado_em               TEXT NOT NULL
);

-- 2.2 Estabelecimento ---------------------------------------------------------
CREATE TABLE estabelecimentos (
    id            TEXT PRIMARY KEY,
    empresa_id    TEXT NOT NULL REFERENCES empresas(id),
    nome          TEXT NOT NULL,
    tipo_negocio  TEXT,
    endereco      TEXT,
    ativo         INTEGER NOT NULL DEFAULT 1,
    criado_em     TEXT NOT NULL
);

CREATE INDEX idx_estabelecimentos_empresa ON estabelecimentos(empresa_id);

-- 2.3 Utilizador --------------------------------------------------------------
CREATE TABLE utilizadores (
    id             TEXT PRIMARY KEY,
    nome           TEXT NOT NULL,
    email          TEXT NOT NULL UNIQUE,
    password_hash  TEXT NOT NULL,
    ativo          INTEGER NOT NULL DEFAULT 1,
    criado_em      TEXT NOT NULL
);

-- utilizador <-> estabelecimentos a que tem acesso
CREATE TABLE utilizador_estabelecimentos (
    utilizador_id      TEXT NOT NULL REFERENCES utilizadores(id),
    estabelecimento_id TEXT NOT NULL REFERENCES estabelecimentos(id),
    PRIMARY KEY (utilizador_id, estabelecimento_id)
);

-- 2.4 Papel / Permissão --------------------------------------------------------
CREATE TABLE papeis (
    id    TEXT PRIMARY KEY,
    nome  TEXT NOT NULL UNIQUE
);

CREATE TABLE permissoes (
    id                      TEXT PRIMARY KEY,
    codigo                  TEXT NOT NULL UNIQUE, -- ex: 'faturas.criar', 'caixa.fechar'
    modulo                  TEXT NOT NULL,
    requer_reautenticacao   INTEGER NOT NULL DEFAULT 0
);

CREATE TABLE papel_permissoes (
    papel_id      TEXT NOT NULL REFERENCES papeis(id),
    permissao_id  TEXT NOT NULL REFERENCES permissoes(id),
    PRIMARY KEY (papel_id, permissao_id)
);

CREATE TABLE utilizador_papeis (
    utilizador_id  TEXT NOT NULL REFERENCES utilizadores(id),
    papel_id       TEXT NOT NULL REFERENCES papeis(id),
    PRIMARY KEY (utilizador_id, papel_id)
);

-- 2.5 Cliente -------------------------------------------------------------------
CREATE TABLE clientes (
    id          TEXT PRIMARY KEY,
    empresa_id  TEXT NOT NULL REFERENCES empresas(id),
    nome        TEXT NOT NULL,
    email       TEXT,
    telefone    TEXT,
    morada      TEXT,
    nif         TEXT,
    criado_em   TEXT NOT NULL
);

CREATE INDEX idx_clientes_empresa ON clientes(empresa_id);

-- 2.7 TaxaImposto -----------------------------------------------------------
CREATE TABLE taxas_imposto (
    id           TEXT PRIMARY KEY,
    empresa_id   TEXT NOT NULL REFERENCES empresas(id),
    nome         TEXT NOT NULL,
    percentagem  REAL NOT NULL,
    ativa        INTEGER NOT NULL DEFAULT 1
);

-- 2.6 Produto/Serviço ---------------------------------------------------------
CREATE TABLE produtos_servicos (
    id                  TEXT PRIMARY KEY,
    empresa_id          TEXT NOT NULL REFERENCES empresas(id),
    nome                TEXT NOT NULL,
    descricao           TEXT,
    tipo                TEXT NOT NULL CHECK (tipo IN ('produto', 'servico')),
    unidade_medida      TEXT NOT NULL DEFAULT 'unidade',
    preco_base_centimos INTEGER NOT NULL,
    taxa_imposto_id     TEXT REFERENCES taxas_imposto(id),
    controla_stock      INTEGER NOT NULL DEFAULT 0,
    ativo               INTEGER NOT NULL DEFAULT 1,
    criado_em           TEXT NOT NULL
);

CREATE INDEX idx_produtos_empresa ON produtos_servicos(empresa_id);

-- 2.8 SérieDocumental -----------------------------------------------------------
CREATE TABLE series_documentais (
    id                  TEXT PRIMARY KEY,
    estabelecimento_id  TEXT NOT NULL REFERENCES estabelecimentos(id),
    tipo_documento      TEXT NOT NULL,
    prefixo             TEXT NOT NULL,
    ano_economico       INTEGER NOT NULL,
    proximo_numero      INTEGER NOT NULL DEFAULT 1,
    UNIQUE (estabelecimento_id, tipo_documento, ano_economico)
);

-- 2.12 SessãoCaixa (núcleo dos problemas 1 e 2) --------------------------------
CREATE TABLE sessoes_caixa (
    id                              TEXT PRIMARY KEY,
    estabelecimento_id              TEXT NOT NULL REFERENCES estabelecimentos(id),
    aberta_por_utilizador_id        TEXT NOT NULL REFERENCES utilizadores(id),
    fechada_por_utilizador_id       TEXT REFERENCES utilizadores(id),
    fundo_maneio_inicial_centimos   INTEGER NOT NULL DEFAULT 0,
    estado                          TEXT NOT NULL DEFAULT 'aberta'
                                    CHECK (estado IN ('aberta', 'fechada')),
    aberta_em                       TEXT NOT NULL,
    fechada_em                      TEXT,
    contagem_fisica_dinheiro_centimos INTEGER,
    diferenca_calculada_centimos      INTEGER
);

-- Regra dura: no máximo uma sessão aberta por estabelecimento (problema 1).
CREATE UNIQUE INDEX idx_sessao_caixa_unica_aberta
    ON sessoes_caixa(estabelecimento_id)
    WHERE estado = 'aberta';

-- Totais acumulados incrementalmente por método de pagamento (problema 2).
CREATE TABLE sessao_caixa_totais_por_metodo (
    id                    TEXT PRIMARY KEY,
    sessao_caixa_id       TEXT NOT NULL REFERENCES sessoes_caixa(id),
    metodo_pagamento      TEXT NOT NULL,
    total_acumulado_centimos INTEGER NOT NULL DEFAULT 0,
    UNIQUE (sessao_caixa_id, metodo_pagamento)
);

-- 2.9 Documento -------------------------------------------------------------------
CREATE TABLE documentos (
    id                            TEXT PRIMARY KEY,
    estabelecimento_id            TEXT NOT NULL REFERENCES estabelecimentos(id),
    cliente_id                    TEXT REFERENCES clientes(id),
    utilizador_emissor_id         TEXT NOT NULL REFERENCES utilizadores(id),
    tipo                          TEXT NOT NULL CHECK (tipo IN (
                                       'orcamento', 'proforma', 'encomenda',
                                       'guia_remessa', 'fatura', 'fatura_recibo',
                                       'recibo', 'nota_credito'
                                   )),
    serie_documental_id           TEXT REFERENCES series_documentais(id),
    numero                        INTEGER,
    ano_economico                 INTEGER,
    prefixo                       TEXT,
    estado                        TEXT NOT NULL DEFAULT 'rascunho' CHECK (estado IN (
                                       'rascunho', 'emitido', 'pago_parcial',
                                       'pago', 'vencido', 'anulado'
                                   )),
    documento_origem_id           TEXT REFERENCES documentos(id),
    documento_original_id         TEXT REFERENCES documentos(id),
    motivo_anulacao_retificacao   TEXT,
    hash                          TEXT,
    codigo_certificacao           TEXT,
    via                           TEXT NOT NULL DEFAULT 'original'
                                  CHECK (via IN ('original', 'segunda_via')),
    observacoes                   TEXT,
    condicoes_pagamento           TEXT,
    data_vencimento               TEXT,
    subtotal_centimos             INTEGER NOT NULL DEFAULT 0,
    total_impostos_centimos       INTEGER NOT NULL DEFAULT 0,
    total_descontos_centimos      INTEGER NOT NULL DEFAULT 0,
    total_centimos                INTEGER NOT NULL DEFAULT 0,
    emitido_em                    TEXT,
    criado_em                     TEXT NOT NULL,
    UNIQUE (estabelecimento_id, serie_documental_id, ano_economico, numero)
);

CREATE INDEX idx_documentos_estabelecimento ON documentos(estabelecimento_id);
CREATE INDEX idx_documentos_cliente ON documentos(cliente_id);
CREATE INDEX idx_documentos_estado ON documentos(estado);

-- 2.10 LinhaDocumento ---------------------------------------------------------------
CREATE TABLE linhas_documento (
    id                          TEXT PRIMARY KEY,
    documento_id                TEXT NOT NULL REFERENCES documentos(id),
    produto_servico_id          TEXT REFERENCES produtos_servicos(id),
    descricao                   TEXT NOT NULL,
    quantidade                  REAL NOT NULL,
    preco_unitario_centimos     INTEGER NOT NULL,
    desconto_centimos           INTEGER NOT NULL DEFAULT 0,
    taxa_imposto_percentagem    REAL NOT NULL DEFAULT 0,
    justificacao_isencao        TEXT,
    ordem                       INTEGER NOT NULL DEFAULT 0
);

CREATE INDEX idx_linhas_documento_documento ON linhas_documento(documento_id);

-- 2.11 Pagamento ------------------------------------------------------------------
CREATE TABLE pagamentos (
    id                 TEXT PRIMARY KEY,
    documento_id       TEXT NOT NULL REFERENCES documentos(id),
    sessao_caixa_id    TEXT NOT NULL REFERENCES sessoes_caixa(id),
    utilizador_id      TEXT NOT NULL REFERENCES utilizadores(id),
    metodo_pagamento   TEXT NOT NULL,
    valor_centimos     INTEGER NOT NULL,
    criado_em          TEXT NOT NULL
);

CREATE INDEX idx_pagamentos_documento ON pagamentos(documento_id);
CREATE INDEX idx_pagamentos_sessao ON pagamentos(sessao_caixa_id);

-- 2.13 LogAuditoria -----------------------------------------------------------------
CREATE TABLE logs_auditoria (
    id             TEXT PRIMARY KEY,
    utilizador_id  TEXT REFERENCES utilizadores(id),
    acao           TEXT NOT NULL,
    entidade       TEXT NOT NULL,
    entidade_id    TEXT NOT NULL,
    dados_antes    TEXT, -- JSON
    dados_depois   TEXT, -- JSON
    criado_em      TEXT NOT NULL
);

CREATE INDEX idx_logs_auditoria_entidade ON logs_auditoria(entidade, entidade_id);

-- 2.14 SyncQueue (contingência offline) ----------------------------------------------
CREATE TABLE sync_queue (
    id                TEXT PRIMARY KEY,
    tipo_evento       TEXT NOT NULL,
    payload           TEXT NOT NULL, -- JSON
    estado            TEXT NOT NULL DEFAULT 'pendente'
                       CHECK (estado IN ('pendente', 'sincronizado', 'erro')),
    tentativas        INTEGER NOT NULL DEFAULT 0,
    criado_em         TEXT NOT NULL,
    sincronizado_em   TEXT
);

CREATE INDEX idx_sync_queue_estado ON sync_queue(estado);
