-- Ajustes descobertos na investigação AGT/SAF-T (docs/AGT-SAFT.md) + seed de
-- dados de referência (papéis/permissões, códigos AGT).
--
-- Nota sobre IDs: as linhas de referência abaixo (papéis, permissões,
-- códigos AGT) usam IDs fixos legíveis em vez de UUID — são dados de
-- configuração da aplicação, não registos de negócio criados em execução
-- (a convenção de UUID em 0001_init.sql aplica-se a esses últimos).

-- 1. Snapshot dos dados do cliente no documento (docs/AGT-SAFT.md §0):
-- a reimpressão de um documento emitido tem de preservar o nome/NIF/morada
-- do cliente tal como estavam no momento da emissão, mesmo que a ficha do
-- cliente seja editada depois.
ALTER TABLE documentos ADD COLUMN cliente_nome_snapshot TEXT;
ALTER TABLE documentos ADD COLUMN cliente_nif_snapshot TEXT;
ALTER TABLE documentos ADD COLUMN cliente_morada_snapshot TEXT;

-- 2. Cliente precisa de estado ativo/inativo (desativação em vez de
-- eliminação, mesmo padrão já usado em estabelecimentos/produtos/utilizadores).
ALTER TABLE clientes ADD COLUMN ativo INTEGER NOT NULL DEFAULT 1;

-- 3. Seed de papéis e permissões (docs/ARQUITETURA.md §2.4 e §5).
-- `requer_reautenticacao = 1` marca as ações sensíveis definidas com o
-- cliente: anular documento, gerir utilizadores, alterar configuração
-- fiscal, fechar sessão de caixa, desconto excecional, reembolso.
-- `documentos.corrigir_identificacao` cobre a exceção do artigo 8º nº8 do
-- Decreto 71/25 (docs/AGT-SAFT.md §0): corrigir só nome/NIF/morada do
-- emitente OU do adquirente num documento já emitido, sem nota de
-- crédito — usa a mesma password de confirmação por ser uma alteração a
-- um documento normalmente imutável.
INSERT INTO permissoes (id, codigo, modulo, requer_reautenticacao) VALUES
    ('perm-documentos-criar',            'documentos.criar',            'documentos',    0),
    ('perm-documentos-editar-rascunho',  'documentos.editar_rascunho',  'documentos',    0),
    ('perm-documentos-ver',              'documentos.ver',              'documentos',    0),
    ('perm-documentos-anular',           'documentos.anular',           'documentos',    1),
    ('perm-documentos-corrigir-identificacao', 'documentos.corrigir_identificacao', 'documentos', 1),
    ('perm-documentos-desconto-excecional', 'documentos.desconto_excecional', 'documentos', 1),
    ('perm-pagamentos-registar',         'pagamentos.registar',         'pagamentos',    0),
    ('perm-pagamentos-reembolsar',       'pagamentos.reembolsar',       'pagamentos',    1),
    ('perm-caixa-abrir',                 'caixa.abrir',                 'caixa',         0),
    ('perm-caixa-fechar',                'caixa.fechar',                'caixa',         1),
    ('perm-clientes-gerir',              'clientes.gerir',              'clientes',      0),
    ('perm-produtos-gerir',              'produtos.gerir',              'produtos',      0),
    ('perm-utilizadores-gerir',          'utilizadores.gerir',          'utilizadores',  1),
    ('perm-configuracao-alterar',        'configuracao.alterar',        'configuracao',  1),
    ('perm-relatorios-ver',              'relatorios.ver',              'relatorios',    0);

INSERT INTO papeis (id, nome) VALUES
    ('papel-administrador', 'Administrador'),
    ('papel-funcionario',   'Funcionário');

-- Administrador: todas as permissões.
INSERT INTO papel_permissoes (papel_id, permissao_id)
SELECT 'papel-administrador', id FROM permissoes;

-- Funcionário: operações de balcão (criar/anular documentos, pagamentos,
-- caixa, clientes, produtos) — sem gestão de utilizadores, configuração
-- fiscal, nem relatórios (regra explícita do cliente).
INSERT INTO papel_permissoes (papel_id, permissao_id) VALUES
    ('papel-funcionario', 'perm-documentos-criar'),
    ('papel-funcionario', 'perm-documentos-editar-rascunho'),
    ('papel-funcionario', 'perm-documentos-ver'),
    ('papel-funcionario', 'perm-documentos-anular'),
    ('papel-funcionario', 'perm-documentos-corrigir-identificacao'),
    ('papel-funcionario', 'perm-documentos-desconto-excecional'),
    ('papel-funcionario', 'perm-pagamentos-registar'),
    ('papel-funcionario', 'perm-pagamentos-reembolsar'),
    ('papel-funcionario', 'perm-caixa-abrir'),
    ('papel-funcionario', 'perm-caixa-fechar'),
    ('papel-funcionario', 'perm-clientes-gerir'),
    ('papel-funcionario', 'perm-produtos-gerir');

-- 4. Seed de códigos AGT (docs/AGT-SAFT.md §0/§7) — só os tipos confirmados
-- (FT, NC); os restantes (fatura_recibo, guia_remessa, recibo, proforma,
-- encomenda, orçamento) precisam de confirmação antes de preencher.
INSERT INTO codigos_documento_agt (tipo_documento, codigo_agt, tabela_saft) VALUES
    ('fatura',       'FT', 'SalesInvoices'),
    ('nota_credito', 'NC', 'SalesInvoices');
