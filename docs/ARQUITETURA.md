# Arquitetura — Software de Faturação

Documento de referência para o desenho do sistema. Cobre: stack, modelo de
dados, fluxo do ciclo comercial, e desenho das sessões de caixa por
estabelecimento. Serve de base para as próximas sessões de implementação.

Contexto de negócio, requisitos legais (Decreto Presidencial n.º 71/25) e os
3 problemas prioritários do cliente atual estão descritos na especificação
original fornecida pelo cliente (sessões partilhadas, fecho de caixa manual,
password por item).

---

## 1. Stack

| Camada | Escolha | Porquê |
|---|---|---|
| Runtime desktop | **Tauri 2** | Binários pequenos, baixo consumo de RAM (vs Electron), sandboxing por permissões nativo do Tauri — bom fit para um produto que vai lidar com dados fiscais sensíveis. |
| Backend/core | **Rust** | Lógica de negócio (numeração de documentos, cálculo de sessão de caixa, hash de documentos, regras fiscais) corre no lado Rust, não no frontend — evita que lógica crítica fique exposta/alterável no webview. |
| Frontend | **React + TypeScript + Vite** | Ecossistema mais popular, fácil de encontrar componentes de tabela/formulário para um produto data-heavy como este. |
| UI kit | **Mantine** ou **shadcn/ui** (decidir na primeira sessão de scaffold) | Componentes de tabela, formulário e data entry prontos — reduz tempo em CRUDs repetitivos (clientes, produtos, documentos). |
| Base de dados local | **SQLite** (via `sqlx` ou `rusqlite`, com migrations versionadas) | Requisito legal de funcionamento offline; ficheiro único facilita backup/conservação de dados (obrigação legal de arquivo). |
| Geração de PDF | Render HTML do documento no lado Rust (ex. `printpdf` ou motor de template + wkhtmltopdf-like) ou impressão via webview do Tauri | Decidir na fase de implementação de impressão; requisito legal de marca "original"/"2ª via" tem de estar no template. |
| Sincronização (contingência) | Fila de eventos local (tabela `sync_queue`) + cliente HTTP para o backend/AGT quando há rede | Cumpre o requisito de emissão offline com sincronização posterior. |

**Investigado, mas não implementado ainda:** o formato de comunicação com a
AGT / exportação SAF-T e os requisitos de assinatura já foram pesquisados —
ver [`docs/AGT-SAFT.md`](AGT-SAFT.md) e a secção 9 abaixo. A codificação da
assinatura RSA-SHA1 e do export SAF-T em si continuam por implementar.

**Não escolhido agora, mas a decidir mais tarde:** se existirá um backend
central multi-dispositivo (hoje o desenho é single-desktop com SQLite
local; se no futuro for preciso sincronizar entre vários postos do mesmo
estabelecimento, isso implica rever esta camada).

---

## 2. Modelo de entidades

Nomes em português para bater certo com o domínio do cliente e a
terminologia legal angolana. Campos-chave apenas (não é o schema SQL final).

### 2.1 Empresa
Tenant raiz da instalação (o cliente pode ter só uma, mas o modelo já
suporta multi-empresa se um cliente futuro precisar).
- nome, NIF, morada, logótipo, regime fiscal, moeda (fixa em moeda nacional
  nos documentos fiscais), dados bancários, condições de pagamento padrão.
- `perfil_complexidade`: `simples` | `completo` — ativa/desativa etapas do
  ciclo comercial (secção 4).

### 2.2 Estabelecimento
- pertence a `Empresa`. nome, tipo de negócio (ex. pastelaria, restaurante),
  endereço.
- **cada Estabelecimento tem a sua própria Sessão de Caixa ativa** — resolve
  o problema 1.
- tem as suas próprias `SérieDocumental` (numeração independente por
  estabelecimento, tipo de documento e ano económico, conforme o Decreto).

### 2.3 Utilizador
- nome, email, password (hash), estado (ativo/inativo).
- `password_alterada_em` — requisito AGT de mudança periódica de password
  obrigatória (docs/AGT-SAFT.md §5); a aplicação força reset ao expirar.
  Nota: a mesma fonte confirma que o administrador nunca pode ver/conhecer
  a password de outro utilizador — reforça que só o hash é guardado.
- `Papel` (0..N) — ver secção 2.4, permissões.
- sessão de login autentica uma vez por turno; reautenticação só é pedida
  para ações marcadas como sensíveis (ver `Permissao.requer_reautenticacao`)
  — resolve o problema 3.

### 2.4 Papel / Permissão
Modelado como permissões por ação/módulo, não como dois papéis fixos:
- `Permissao`: código (ex. `faturas.criar`, `faturas.anular`,
  `relatorios.ver`, `caixa.fechar`, `utilizadores.gerir`), módulo,
  `requer_reautenticacao: bool`.
- `Papel`: nome (ex. Administrador, Funcionário — mas extensível),
  lista de `Permissao`.
- `Utilizador` ↔ `Papel` (N:N), com possibilidade de overrides individuais
  se necessário no futuro.

### 2.5 Cliente
- nome, email, telefone, morada, NIF.
- histórico: faturas, pagamentos, saldo — derivado de `Documento` e
  `Pagamento`, não duplicado.

### 2.6 Produto/Serviço
- nome, descrição, tipo (produto/serviço), unidade de medida, preço base,
  `TaxaImposto` associada, controla stock (bool, relevante só no perfil
  "completo").

### 2.7 TaxaImposto
- nome (ex. IVA 14%), percentagem, ativa.

### 2.8 SérieDocumental
Garante a numeração sequencial e cronológica exigida por lei.
- estabelecimento, tipo de documento, prefixo configurável, ano económico,
  próximo número (contador atómico).
- **Numeração nunca reutilizada nem recuada** — mesmo que o documento seja
  cancelado, o número fica "queimado".

### 2.9 Documento
Entidade central; cobre proforma, encomenda, guia de remessa, fatura,
fatura-recibo, nota de crédito, recibo — diferenciados por `tipo`.
- tipo, série/número (via `SérieDocumental`), ano económico, estabelecimento,
  cliente, utilizador emissor.
- estado: `Rascunho → Emitido → Pago (total/parcial) → Vencido` (documentos
  como notas de crédito têm o seu próprio subconjunto de estados).
- `documento_original_id` — obrigatório em notas de crédito, aponta ao
  documento retificado.
- `motivo_anulacao_retificacao` — obrigatório quando aplicável.
- `hash` — assinatura RSA-SHA1 em cadeia por estabelecimento+tipo+série
  (docs/AGT-SAFT.md §3); cada assinatura incorpora o hash do documento
  anterior da mesma série. **Recibos ficam isentos** desta cadeia.
- `codigo_certificacao`, `versao_chave_assinatura` — snapshot, tirado no
  momento da emissão, da `ConfiguracaoCertificacaoAGT` ativa nesse momento
  (secção 2.15) — nunca muda depois, mesmo que a configuração da instalação
  seja atualizada mais tarde.
- `origem_registo`: `normal` | `manual` | `recuperado_backup` — proveniência
  do registo nos dois cenários de contingência documentados pela AGT
  (software local em baixo / restauro de backup com documentos em falta,
  docs/AGT-SAFT.md §6). A série de recuperação anual própria para estes
  casos ainda não está modelada — fica para a implementação do motor de
  emissão.
- `via`: `original` | `segunda_via` — controla o que é impresso.
- observações / condições de pagamento / data de vencimento.
- **Regra dura no modelo, não só na UI:** depois de `estado != Rascunho`,
  o documento é imutável — sem `UPDATE`/`DELETE` a nível de aplicação; só
  se cria um novo `Documento` do tipo Nota de Crédito que referencia o
  original. Rascunhos continuam livremente editáveis/elimináveis.

### 2.10 LinhaDocumento
- documento, produto/serviço, descrição (snapshot no momento da emissão —
  não segue alterações futuras ao catálogo), quantidade, preço unitário,
  desconto, taxa de imposto aplicada, `justificacao_isencao` (obrigatório
  quando não há liquidação de imposto).

### 2.11 Pagamento
- documento associado, `sessao_caixa_id`, método de pagamento (dinheiro,
  transferência, multicaixa, etc.), valor, data/hora, utilizador que
  registou.
- pagamentos parciais permitidos (documento pode ter N pagamentos).

### 2.12 SessãoCaixa
Ver secção 4 para o desenho completo — é o núcleo dos problemas 1 e 2.

### 2.13 LogAuditoria
- utilizador, ação, entidade+id afetada, timestamp, snapshot antes/depois
  (JSON). Cobre o requisito legal de deteção de alterações não autorizadas
  e rastreabilidade.

### 2.14 SyncQueue (contingência)
- tipo de evento, payload, estado (pendente/sincronizado/erro), timestamp —
  fila de eventos criados offline para comunicação eletrónica posterior à
  AGT.
- **Nota**: esta fila cobre a comunicação com a AGT (envio quando a rede
  volta), mas **não** cobre o mecanismo de contingência do próprio motor de
  numeração/série (ver `Documento.origem_registo` e docs/AGT-SAFT.md §6) —
  são dois conceitos distintos.

### 2.15 ConfiguracaoCertificacaoAGT
Configuração de instalação/software (tipicamente uma única linha), não por
`Empresa` — a fonte técnica pesquisada (docs/AGT-SAFT.md §3) indica que o
par de chaves é gerado pelo **fabricante do software**, não por cliente.
- `numero_certificado_agt`, `chave_publica`, `versao_chave`.
- `chave_privada_ref` — referência ao segredo (ex. keystore do SO); a chave
  privada em si nunca é guardada em texto simples na base de dados de
  faturas.
- É a fonte de verdade copiada (snapshot) para `Documento.codigo_certificacao`
  e `Documento.versao_chave_assinatura` no momento da emissão.

### 2.16 CodigoDocumentoAGT
Tabela de referência: `tipo_documento` (do nosso enum) → código exigido
pela AGT (ex. `FT`, `NC`, `ND`) e tabela SAF-T correspondente
(`SalesInvoices`, `WorkingDocuments`, `Payments`, ...) — usada para compor o
`InvoiceNo` (`"{código} {série}/{número}"`) no SAF-T, no webservice em
tempo real e no carimbo impresso.
- **Só `fatura`→`FT`, `nota_credito`→`NC` e um equivalente a `ND` estão
  confirmados** (docs/AGT-SAFT.md §8); os restantes tipos precisam de
  confirmação directa com a AGT antes de preencher esta tabela em produção
  — por isso a migration cria a tabela vazia, sem seed.

---

## 3. Fluxo do ciclo comercial

Documento é o nó central; o ciclo é uma cadeia de documentos, cada um
opcionalmente originando o seguinte, **configurável por perfil de
complexidade**:

```
Orçamento/Proposta (opcional)
  → Proforma (opcional)
    → Encomenda (opcional)
      → Guia de Remessa (opcional, só perfil "completo" c/ stock)
        → Fatura ou Fatura-Recibo (obrigatório — ponto de emissão fiscal)
          → Pagamento(s) (parcial ou total)
            → Recibo (por pagamento)
              → Nota de Crédito (se for preciso corrigir/anular)
```

- **Perfil "Simples"**: só expõe Fatura-Recibo direta (sem proforma,
  encomenda, guia). É uma configuração da `Empresa` (`perfil_complexidade`),
  não uma versão diferente do produto — a UI esconde os passos não
  ativados, o modelo de dados e o motor de documentos continuam iguais.
- **Perfil "Completo"**: expõe o ciclo inteiro, incluindo controlo de stock
  (dá baixa no `Produto` quando a Guia de Remessa ou a Fatura é emitida,
  dependendo da configuração).
- Cada transição de documento é rastreável (`documento_origem_id`), para
  poder navegar do Orçamento até ao Recibo final.

### 3.1 "Máquina de faturas" — o que falta investigar (pendente, secção 6
    da spec original)
Antes de implementar o motor de emissão, decidir explicitamente o que conta
como "operação completa" antes de:
1. atribuir o número sequencial definitivo (via `SérieDocumental`),
2. gerar o hash/certificação,
3. bloquear a edição do documento.

Proposta de working model a validar com o cliente/contexto legal: o
documento só sai de `Rascunho` para `Emitido` numa única transação atómica
que (a) valida todas as linhas e totais, (b) consome o próximo número da
série, (c) calcula o hash, e (d) grava — tudo ou nada. Isto evita números
"queimados" por falhas a meio do processo em vez de por cancelamento
legítimo.

---

## 4. Sessões de caixa (resolve os problemas 1 e 2)

### 4.1 Independência por estabelecimento
- Uma `SessãoCaixa` pertence sempre a **um único Estabelecimento**.
- No máximo uma sessão `aberta` por estabelecimento em cada momento —
  **regra dura, imposta no backend** (constraint/validação na criação, não
  só escondida na UI).
- Abrir sessão = escolher estabelecimento (o utilizador pode ter acesso a
  vários) + montante inicial de caixa (fundo de maneio).
- Se já existe uma sessão aberta nesse estabelecimento, um novo operador
  **não abre uma segunda sessão** — tem duas opções apresentadas pela UI:
  (a) entrar/operar dentro da sessão já aberta (ver 4.3), ou (b) esperar
  que o operador atual a feche. A ação "Abrir sessão" fica desativada/
  bloqueada enquanto houver uma sessão ativa nesse estabelecimento, e a UI
  mostra quem a tem aberta e desde quando.
- Todas as operações de venda/pagamento feitas "dentro" de um
  estabelecimento só podem acontecer com uma sessão aberta nesse
  estabelecimento — e ficam automaticamente associadas a ela.
- Fechar a sessão da pastelaria não tem qualquer relação com a sessão do
  restaurante: cálculos, totais e fecho são 100% independentes.

### 4.2 Acumulação incremental (fecho automático)
Em vez de calcular o total só no momento do fecho (o que seria uma consulta
agregada pesada e ainda dependente de os dados estarem corretos), a sessão
mantém **totais correntes, atualizados na mesma transação que regista cada
pagamento**:

- `SessãoCaixa` tem uma tabela associada `SessaoCaixaTotalPorMetodo`
  (sessão, método de pagamento, total_acumulado), atualizada com
  `UPDATE ... SET total = total + :valor` dentro da mesma transação SQL que
  insere o `Pagamento`. Isto dá:
  - total em tempo real, visível a qualquer momento durante o dia (não só
    no fecho);
  - fecho instantâneo — não há cálculo pesado nem espera;
  - consistência garantida (não há uma fase separada de "recalcular").
- No momento do fecho: o sistema apresenta o total esperado por método de
  pagamento e no total; o funcionário só introduz a **contagem física de
  dinheiro** (para o método "dinheiro"); o sistema calcula a diferença
  (sobra/falta) automaticamente — elimina o cálculo manual que causava
  erros.
- `SessãoCaixa` guarda: estabelecimento, utilizador que abriu/fechou,
  timestamp abertura/fecho, fundo de maneio inicial, totais por método
  (snapshot congelado no fecho), contagem física declarada, diferença
  calculada, estado (`aberta`/`fechada`).

### 4.3 Multi-utilizador dentro da mesma sessão
Vários funcionários podem operar na mesma sessão de caixa do mesmo
estabelecimento (ex. dois empregados de balcão no mesmo turno) — a sessão é
por estabelecimento/turno, não por utilizador individual. Cada
`Pagamento`/`Documento` regista qual utilizador o realizou, para efeitos de
auditoria, mas contribui para o mesmo total de sessão.

---

## 5. Autenticação e fricção mínima (resolve o problema 3)

- Login autentica a sessão do utilizador para todo o turno (token/sessão em
  memória local, não por pedido).
- Adicionar/remover linha de documento, aplicar desconto normal, mudar
  quantidade, registar um pagamento normal, imprimir original/2ª via,
  abrir/fechar sessão de caixa dentro do fluxo normal → **sem
  reautenticação**.
- Reautenticação (password completa, não PIN) só pedida quando a ação
  tocar uma `Permissao` com `requer_reautenticacao = true`. Lista definida
  (`Permissao.requer_reautenticacao`), critério: ações difíceis/impossíveis
  de reverter, ou que alteram quem pode fazer o quê:
  - **Anular/retificar documento emitido** (emissão de nota de crédito) —
    é a exceção legal ao princípio da imutabilidade, tem de ficar bem
    guardada.
  - **Gerir utilizadores e permissões** (criar/desativar utilizador,
    mudar papel/permissões).
  - **Alterar configuração fiscal/da empresa** (taxas de imposto, dados da
    empresa, séries documentais/numeração, perfil simples/completo).
  - **Fechar sessão de caixa** — é o checkpoint que congela os totais do
    dia; confirmação por password reduz risco de fecho indevido/tamper.
  - **Aplicar desconto acima de um limite configurável** (ex. > X%,
    valor por definir com o cliente) — padrão comum em POS para evitar
    abuso de desconto por um operador.
  - **Registar reembolso/estorno de pagamento** — movimento de dinheiro
    para fora, maior risco que um pagamento normal.
  - Fora desta lista por agora: reimpressão (é só rastreada em log, não
    bloqueada), ver relatórios (isso é controlado por `Permissao` normal
    de acesso, não por reautenticação), operações de rascunho do próprio
    utilizador.
- **Timeout de inatividade — 45 minutos**: decorridos 45 min sem ação do
  operador, a sessão local **bloqueia** (não faz logout, não troca de
  utilizador) — a próxima ação exige a password completa do mesmo
  operador para desbloquear e continuar exatamente onde estava. É um
  "lock de ecrã", não um fim de sessão.

---

## 6. Regras de UI ditadas pela lei (resumo aplicável ao modelo)

- Nunca existe ação "Eliminar" para `Documento` com `estado != Rascunho`.
  Ações válidas pós-emissão: Visualizar, Imprimir original, Emitir 2ª via,
  Registar pagamento, Emitir recibo, Consultar histórico.
- Rascunhos: CRUD livre.
- Toda a reimpressão de um documento já emitido tem de marcar
  explicitamente "2ª via".
- Nota de crédito é o único caminho de retificação/anulação (salvo exceções
  a definir) e exige motivo + referência ao documento original.

---

## 7. Certificação AGT (resumo — detalhe completo em `docs/AGT-SAFT.md`)

Investigação feita sobre os requisitos técnicos de certificação de software
de faturação pela AGT (Angola). Fonte principal é um anexo técnico oficial
do Ministério das Finanças datado de 2018 e rotulado "proposta" — não
confirmado como atualizado para o Decreto 71/25 (2025). Tratar os pontos
abaixo como o melhor desenho disponível hoje, a confirmar antes de
implementar a sério.

- **Assinatura**: RSA-1024 + SHA-1, certificado auto-assinado, chave gerada
  pelo fabricante do software (não por instalação/cliente) — ver
  `ConfiguracaoCertificacaoAGT` (§2.15). Cadeia de hash por
  estabelecimento+tipo+série de documento (§2.9). Recibos ficam isentos.
- **Impresso no documento**: 4 caracteres da assinatura + número de
  certificado — **sem QR code** em nenhuma fonte encontrada, o que é
  atípico face a regimes semelhantes mais recentes; confirmar antes de
  fechar o design de impressão.
- **SAF-T(AO)**: XML derivado do SAF-T português, submissão mensal até dia
  20. `InvoiceNo` = `"{código AGT} {série}/{número}"` — ver
  `CodigoDocumentoAGT` (§2.16).
- **Regras de negócio ainda não impostas no schema** (ficam para a camada
  de aplicação/Rust, junto do motor de emissão): nunca anular documento com
  retificativo pendente; nota de crédito parcial não pode exceder o valor
  já creditado; NIF de cliente não pode ser alterado depois de ter
  documentos emitidos; descontos sempre 0–100%.
- **Pontos de baixa confiança a confirmar com a AGT/fornecedor certificado
  antes de implementar**: se RSA-1024/SHA-1 continuam válidos em 2025/2026
  (são fracos para os padrões atuais), ausência de QR code, e o processo de
  certificação em si (submissão, taxas, prazos).

---

## 8. Estrutura de projeto proposta (para a fase de scaffold)

```
software-de-faturacao/
├── src-tauri/           # Rust: comandos Tauri, regras de negócio, DB
│   ├── src/
│   │   ├── domain/      # entidades + regras (documento, sessão caixa, ...)
│   │   ├── db/          # migrations, queries
│   │   ├── commands/    # comandos expostos ao frontend
│   │   └── main.rs
│   └── migrations/
├── src/                 # React + TS
│   ├── features/        # clientes, produtos, documentos, caixa, relatorios
│   ├── components/
│   └── lib/
└── docs/
    ├── ARQUITETURA.md   # este documento
    └── AGT-SAFT.md      # investigação detalhada de certificação AGT/SAF-T
```

---

## 9. Próximos passos sugeridos

1. ~~Validar este documento com o cliente/stakeholder~~ — feito para as
   secções 1, 4 e 5 (sessões de caixa e autenticação já confirmadas com o
   cliente). Secção 3.1 (definição de "operação completa" antes da emissão)
   continua por validar.
2. ~~Scaffold do projeto Tauri + React + SQLite~~ — feito, ver §8.
3. Implementar o fluxo mínimo: Estabelecimento → Sessão de Caixa →
   Fatura-Recibo simples → Pagamento → total incremental → fecho. Isto já
   resolve os 3 problemas prioritários (secção 1 da spec) antes de atacar o
   ciclo comercial completo.
4. ~~Investigação pendente: padrões técnicos AGT~~ — feito, ver
   `docs/AGT-SAFT.md` e secção 7. Falta ainda: confirmar diretamente com a
   AGT/um fornecedor certificado os pontos de baixa confiança (RSA-1024/
   SHA-1, ausência de QR code) antes de implementar a assinatura a sério.
5. ~~Implementar autenticação~~ e ~~o motor de numeração/emissão de
   documentos~~ — feito (ver `src-tauri/src/auth/`, `src-tauri/src/
   permissoes/`, `src-tauri/src/emissao/`, e os comandos em `src-tauri/
   src/commands/`). **Por verificar**: nada disto foi compilado ainda —
   Rust continua por instalar neste ambiente. Primeiro passo assim que
   `cargo` estiver disponível: `cargo check` e `cargo test` (há testes em
   `src-tauri/src/db/mod.rs` para a sessão única e a acumulação
   incremental).
6. A assinatura de documentos usa uma função placeholder (SHA-256, não
   RSA) em `emissao::assinar_placeholder` — está claramente assinalada no
   código para ser substituída assim que o algoritmo AGT for confirmado
   (ver ponto 4 acima).
7. Por implementar: ecrã de configuração inicial (bootstrap: criar
   primeira empresa + primeiro utilizador via `criar_primeiro_
   utilizador`), o resto do ciclo comercial (orçamento → proforma →
   encomenda → guia de remessa), e ligação do frontend aos ~30 comandos
   Tauri já existentes.
