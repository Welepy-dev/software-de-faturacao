# Investigação AGT / SAF-T (AO) — padrões técnicos de certificação

Resultado da investigação pendente referida em `docs/ARQUITETURA.md` §8 (ponto 4)
e na secção 6 da spec original. Cobre: formato SAF-T(AO), requisitos de
hash/assinatura, processo de certificação, contingência/offline, e o que se
sabe concretamente sobre o Decreto Presidencial n.º 71/25.

**Atualização (2026-09-29): os dois PDFs foram lidos directamente** (não só
via resumos de terceiros ou conversão automática para markdown), o que
confirmou a maior parte do que estava abaixo e revelou requisitos novos —
ver secção 0.

## 0. Fontes primárias lidas directamente

- **Decreto Presidencial n.º 71/25**, texto integral, publicado no Diário
  da República I Série n.º 52 de 20 de março de 2025 — PDF oficial em
  `ucm.minfin.gov.ao` (link nas Fontes). 42 artigos, 7 capítulos. Esta é a
  lei em si; lida na íntegra.
- **"Regras e Requisitos para Validação de Sistemas de Processamento
  Electrónico de Facturação dos Contribuintes"** (Anexos I e II) — o mesmo
  documento técnico já referido abaixo, agora lido directamente (27
  páginas) em vez de via resumo. **Confirma-se agora, lendo o próprio
  documento, que é literalmente um molde de "Proposta de Decreto
  Executivo" de 2018**: o número do decreto, a data e a referência ao
  "Decreto Presidencial n.º__/__" que lhe daria base legal estão todos por
  preencher (campos em branco no próprio PDF) — nunca foi assinado nem
  publicado nesta forma. Ainda assim continua a ser a fonte mais detalhada
  disponível, e o Decreto 71/25 (artigo 36.º) remete exactamente para este
  tipo de diploma — um Decreto Executivo do Ministério das Finanças a
  definir "a estrutura de dados do software e modelo da facturação
  electrónica, assim como as demais especificações técnicas e
  procedimentais" — que **continua por publicar/encontrar**. Tratar este
  documento como o desenho técnico mais provável, não como texto em vigor.

### Correcções face à investigação anterior (baseada em resumos de terceiros)

- **Data de entrada em vigor**: o artigo 42.º do Decreto 71/25 diz
  literalmente *"O presente Decreto Presidencial entra em vigor 6 (seis)
  meses após a sua publicação"* — publicado a 20/03/2025, portanto entra em
  vigor **≈20 de setembro de 2025**, não "1 de outubro de 2025" como
  fontes secundárias (Cegid/Saphety/EDICOM) resumiam.
- **Faseamento (grandes contribuintes vs. restantes)**: o artigo 37.º
  ("Disposição transitória") conta os 12 meses de faseamento a partir da
  **entrada em vigor do Decreto Executivo do artigo 36.º** (ainda não
  publicado/encontrado), não a partir de uma data fixa do calendário. As
  datas "1 jan 2026 / 1 jan 2027" das fontes secundárias são aproximações,
  não texto legal — tratar com mais cautela até o Decreto Executivo
  aparecer.
- **RSA-1024 + SHA-1**: o anexo técnico **nunca refere explicitamente**
  nem o tamanho da chave nem o algoritmo de hash (só diz "deve ser
  utilizado o algoritmo RSA"). A inferência de "1024 bits" vem do exemplo
  de assinatura no documento ter exactamente 172 caracteres em base-64
  (consistente com uma assinatura RSA de 128 bytes) — é uma inferência
  razoável, mas **não é texto explícito da fonte**; baixar a confiança
  deste ponto face à investigação anterior, que o tratava como confirmado.
- **QR code**: confirmado directamente no texto do Decreto 71/25 (artigo
  19.º) que a autenticidade é garantida "através da aposição de um código
  digital definido pela Administração Geral Tributária" — linguagem
  genérica, sem menção a QR code em lado nenhum de nenhuma das duas fontes
  primárias. Reforça (não resolve) a suspeita de que não há QR code.

### Requisitos novos, não capturados na investigação anterior

- **Total por extenso**: artigo 10.º n.º1 d) do Decreto exige "preço
  unitário e total em moeda nacional, bem como a sua estipulação por
  extenso" — o total tem de aparecer escrito por extenso no documento
  (como um cheque), não só em algarismos. Não modelado ainda — é um
  cálculo de apresentação (formatação), não precisa de coluna nova, mas
  precisa de uma função "número por extenso em português" na camada de
  impressão.
- **"Este documento não serve de factura"**: anexo técnico, ponto 4 c) —
  qualquer documento que não seja factura/rectificativo mas seja
  apresentável ao cliente (ex. orçamento, proforma noutros contextos) tem
  de conter esta frase, obrigatoriamente.
- **"Consumidor final"**: anexo técnico, ponto 6 e) — quando o cliente não
  fornece NIF, a linha do NIF deve ficar inutilizada/em branco ou mostrar
  literalmente "Consumidor final". Relevante directamente para o balcão da
  pastelaria/restaurante, onde a maioria dos clientes não vai dar NIF.
- **Snapshot dos dados do cliente no documento**: anexo técnico, ponto 6
  n) — a reimpressão de um documento (2.ª via) tem de preservar o
  **conteúdo original**, mesmo que os dados do cliente (nome, morada)
  tenham sido entretanto alterados na ficha do cliente. **Isto é uma
  lacuna real no nosso schema**: hoje `documentos` só guarda
  `cliente_id` (FK), não um snapshot do nome/NIF/morada usados na emissão
  — se o cliente for editado depois, a 2.ª via mostraria os dados novos,
  violando esta regra. Ver nota em §7 abaixo; não corrigido ainda, fica
  para decisão.
- **Excepção ao "só nota de crédito"**: Decreto, artigo 8.º n.º8 — quando
  a rectificação é só dos dados do **próprio emitente** (alínea a) do
  artigo 10.º n.º1 — nome/NIF/morada do fornecedor, não do cliente), a
  factura pode ser anulada usando funcionalidade própria do software
  validado, **sem** nota de crédito. É exactamente a "excepção a definir"
  que o nosso `docs/ARQUITETURA.md` §6 já assinalava como pendente.
- **Bloqueio automático por falha de comunicação**: anexo técnico, ponto
  17.º (Decreto) — se o sistema não comunicar com a AGT há mais de 60
  dias, tem de bloquear a emissão de novas facturas. Regra de negócio para
  o motor de emissão, não afecta o schema.
- **Triplicado físico**: Decreto, artigo 7.º n.º3 — quando a factura é
  impressa via software validado, deve ser em triplicado (original ao
  cliente, cópia no arquivo do fornecedor, outra a acompanhar os bens em
  circulação) — relevante só quando há bens a circular fisicamente (ex.
  entregas), não altera o nosso `via` (`original`/`segunda_via`), que
  cobre a distinção legal que já temos.
- **Sorteio "Factura Premiada"**: Decreto, capítulo V — sorteio de prémios
  ligado a facturas com NIF de pessoa singular. Não é requisito técnico
  nosso (quem gere o sorteio é a AGT), mas é um incentivo forte para o
  cliente pedir para colocar o NIF na factura — vale a pena tornar o campo
  NIF do cliente rápido de preencher/pesquisar no ecrã de balcão.

**Fonte técnica principal** (secção original, mantida): um documento
oficial alojado em `ucm.minfin.gov.ao` — *"Regras e Requisitos para
Validação de Sistemas de Processamento Electrónico de Facturação dos
Contribuintes"* (Anexos I e II), formalmente rotulado **"Proposta de
Decreto Executivo"**, com metadados de criação em 2018-11-24 — ver secção
0 acima para a confirmação directa deste estatuto.

---

## 1. Enquadramento legal e calendário (confirmado — texto integral lido, ver §0)

- **Decreto Presidencial n.º 71/25** (20 de março de 2025) aprova o
  **Regime Jurídico das Facturas**, tornando obrigatória a factura
  eletrónica emitida por software validado pela AGT para os regimes Geral
  e Simplificado de IVA (artigo 16.º).
- **Entrada em vigor**: artigo 42.º — 6 meses após a publicação, ou seja
  **≈20 de setembro de 2025**. (Corrige a estimativa anterior de "1 de
  outubro de 2025", que vinha de resumos de terceiros.)
- **Faseamento** (artigo 37.º): nos primeiros 12 meses após a entrada em
  vigor do Decreto Executivo do artigo 36.º (estrutura de dados/
  especificações técnicas — **ainda não publicado/encontrado**), a
  obrigatoriedade aplica-se só a grandes contribuintes e fornecedores do
  Estado; passado esse prazo, aplica-se a todos os contribuintes sujeitos
  a facturação eletrónica. As datas "1 jan 2026 / 1 jan 2027" de fontes
  secundárias são aproximações a esta regra, não texto legal.
- Revoga o Decreto Presidencial n.º 292/18 (Regime Jurídico das Facturas e
  Documentos Equivalentes) e o n.º 144/23 (Auto-Facturação).
- Os detalhes técnicos das secções seguintes vêm do documento "Regras e
  Requisitos" (anexo técnico ainda em forma de proposta não assinada de
  2018 — ver §0), que o próprio Decreto 71/25 (artigo 36.º) indica ser o
  tipo de diploma que fixa estes detalhes — mas que **ainda não foi
  encontrado publicado/actualizado para 2025**.

## 2. SAF-T (AO) — formato e submissão (confirmado)

- Ficheiro XML com schema XSD oficial. Uma cópia pública do XSD existe no
  GitHub: [`assoft-portugal/SAF-T-AO`](https://github.com/assoft-portugal/SAF-T-AO)
  (`SAFTAO1.01_01.xsd`) — claramente derivado do SAF-T português, adaptado
  para Angola.
- Estrutura: `AuditFile` → `Header`, `MasterFiles` (contas, clientes,
  fornecedores, produtos, tabela de impostos), `GeneralLedgerEntries`,
  `SourceDocuments` (`SalesInvoices` = tabela 4.1, `WorkingDocuments` = 4.3,
  `Payments` = 4.4, entre outras).
- **Submissão de faturação**: ficheiro mensal (faturação + aquisição de
  bens/serviços do mês anterior) até dia 20 do mês seguinte, via portal/
  webservice da AGT.
- Também existe SAF-T de contabilidade anual (até 10 de abril) e ficheiro de
  inventário reportado a 31 de dezembro (até 15 de fevereiro).
- Regra geral desde 2019/2020: contribuintes com faturação anual > 50M AKZ
  já eram obrigados a submeter SAF-T mensal — o Decreto 71/25 estende a
  obrigatoriedade de fatura eletrónica (não só o SAF-T) a mais
  contribuintes, faseadamente.

## 3. Assinatura/hash de documentos (Anexo I, ponto 34 — lido directamente)

- **Algoritmo confirmado no texto**: apenas "deve ser utilizado o
  algoritmo RSA (algoritmo de criptografia de dados que usa o sistema de
  chaves assimétricas, chave pública e chave privada)" — o anexo **não
  especifica explicitamente** tamanho de chave, função de hash, nem
  padding em lado nenhum do texto lido (ponto 34, a-g). A chave pública é
  entregue em **PEM, base-64, ficheiro `.txt`** junto da Declaração
  Modelo 8.
  **Correcção face à investigação anterior**: "RSA-1024" e "SHA-1" **não
  são texto explícito da fonte** — são uma inferência (razoável, mas não
  confirmada) a partir do exemplo de assinatura ter exactamente 172
  caracteres base-64 (consistente com uma assinatura RSA sobre chave de
  1024 bits). Tratar como hipótese de trabalho, a confirmar antes de
  implementar.
- O fabricante do software (não cada instalação) gera o par de chaves,
  mantém a chave privada protegida e exclusiva ("total protecção da chave
  privada", ponto 14), e envia a chave pública à AGT via **"Declaração
  Modelo 8"**. Mudar de par de chaves só pode ser feito pelo fabricante,
  com nova comunicação à AGT — e cada versão de chave é numerada
  sequencialmente (inteiros) e gravada junto de cada assinatura (ponto 5c
  — confirma directamente o nosso campo `versao_chave_assinatura`).
- A assinatura fica gravada **em claro** (não encriptada) na base de dados
  do sistema de facturação, associada directamente ao registo integral do
  documento original (ponto 5b) — confirma que `documentos.hash` pode ser
  uma coluna TEXT normal, sem necessidade de encriptação adicional em
  repouso (a proteção que importa é a da chave privada, não da assinatura
  já gerada).
- **O que é assinado** (mensagem = campos concatenados com `;`, sem quebras
  de linha):
  `InvoiceDate;SystemEntryDate;InvoiceNo;GrossTotal;Hash-do-documento-anterior`
  — ex.: `2018-05-18;2018-05-18T11:22:19;FAC 001/18;53.00;`
  (vazio no último campo = primeiro documento da série/tipo).
- **É uma cadeia de hash por série + tipo de documento**: cada assinatura
  incorpora o hash do documento anterior da mesma série/tipo (não da
  empresa inteira). Em séries plurianuais, o 1º documento do novo exercício
  pode encadear a partir do último documento do exercício anterior na mesma
  série/tipo.
- Assinatura resultante: RSA sobre a mensagem (função de hash exacta não
  confirmada — ver acima), saída em base64, **sempre 172 caracteres no
  exemplo do anexo**, sem separadores.
- **Quem precisa de assinatura "com eficácia externa"**: faturas e
  documentos retificativos (notas de débito/crédito), guias de
  transporte/remessa, e qualquer documento apresentável ao cliente
  (incluindo explicitamente **"consultas de mesa"** — relevante para o
  negócio de restaurante do cliente). **Recibos não precisam de
  assinatura/cadeia de hash**, só do carimbo textual (ver abaixo).
- **O que tem de aparecer impresso/no documento eletrónico**: 4 caracteres
  da assinatura (posições 1, 11, 21 e 31), separados por `-`, seguidos de
  `Processado por programa validado n.º XXXX/AGT` — ex.
  `"PbRc-Processado por programa validado n.º 0000/AGT"`. Para documentos
  sem assinatura (ex. recibos): `"Emitido por programa validado n.º
  XXXX/AGT"`.
  **Nenhuma das duas fontes primárias (Decreto 71/25 lido na íntegra,
  incluído) menciona QR code** — o Decreto (artigo 19.º) fala só em
  "aposição de um código digital definido pela Administração Geral
  Tributária", linguagem genérica consistente com o carimbo de 4
  caracteres do anexo técnico. Reforça a hipótese de que não há QR code,
  mas ver "Baixa confiança" — o Decreto Executivo que fixaria isto em
  definitivo ainda não foi encontrado.
- **Reimpressão de documentos integrados/recuperados**: quando o
  documento vem de outro sistema integrado, ou foi recuperado de um
  cenário de contingência (§6), o carimbo impresso usa a expressão
  **"Cópia do documento original"** em vez de "2.ª via" — uma variante de
  rotulagem adicional que o nosso `via` (`original`/`segunda_via`) ainda
  não distingue; coberto conceptualmente por `origem_registo`, mas a
  lógica de impressão terá de escolher o texto certo consoante
  `origem_registo`, não só `via`.

## 4. Transmissão em tempo real (confirmado, parcialmente)

- Webservice SOAP. `SOAP:Header` autentica com utilizador no formato
  `<NIF do emitente>/<UserId>`, password cifrada com uma chave simétrica
  (`Nonce`, AES-128) que por sua vez é cifrada com a chave pública do
  sistema de autenticação da AGT (RSA) e codificada em base64; inclui
  timestamp (`Created`) para proteção contra replay.
- `SOAP:Body` carrega: NIF emitente, `InvoiceNo` (mesmo formato do SAF-T:
  `"{código tipo} {série}/{número sequencial}"`, ex. `"FAC 001/18"`),
  `InvoiceType` (`FT`=Fatura, `NC`=Nota de Crédito, `ND`=Nota de Débito),
  `InvoiceStatus` (`N`=Normal, `A`=Anulada), NIF do adquirente (nacional ou
  estrangeiro — campos mutuamente exclusivos; `999999990` como
  NIF-genérico quando não recolhido), linhas resumidas por taxa de imposto,
  e totais (`TaxPayable`, `NetTotal`, `GrossTotal`).
- **Não encontrei** um contrato documentado para o caso de falha de rede
  (retry, prazo de tolerância, fila) — só o formato da transmissão em si.
  Isto é relevante diretamente para o nosso `sync_queue` — ver secção 6.

## 5. Regras de negócio relevantes (confirmado — Anexo I, pontos 19-33)

- Documentos nunca podem ter valores negativos, exceto via nota de
  débito/crédito; o valor negativo de ajuste nunca pode exceder o valor
  positivo da mesma linha/serviço na fatura original.
- Não é permitido emitir nota de crédito sobre um documento já anulado ou
  já totalmente retificado.
- Não é permitido anular um documento que já tenha documento retificativo
  (mesmo parcial) sem antes anular esse retificativo.
- Notas de crédito parciais devem ser controladas contra as
  quantidades/valores já retificados do documento original (o sistema tem
  de saber "quanto já foi creditado").
- Descontos sempre entre 0% e 100%.
- Cálculos internos devem usar mais de 2 casas decimais (só o valor
  impresso/exportado é arredondado a 2 casas).
- Cliente com NIF já preenchido e com documentos emitidos: NIF não pode ser
  alterado (só preenchido se estava vazio, ou definido como o genérico
  `999999999`). Nome também não pode ser alterado enquanto o NIF não for
  preenchido.
- Produto com documentos já emitidos: a descrição do catálogo não pode ser
  editada retroativamente (o nosso modelo já guarda a descrição como
  snapshot em `linhas_documento.descricao`, portanto isto já está coberto
  independentemente do catálogo poder mudar).
- Senhas: mudança periódica obrigatória, não pode ficar vazia, e o
  administrador **não pode ver/conhecer** as senhas dos utilizadores
  (confirma que hash é obrigatório, mas acrescenta um requisito que ainda
  não temos modelado: rotação periódica forçada).
- Política de cópias de segurança obrigatória, com registo do número de
  restauros efetuados.

## 6. Contingência / operação manual (confirmado — Anexo I, pontos 8-9)

Cobre dois cenários distintos, ambos por **série de recuperação anual,
numerada a partir de 1, nunca apagável**:

1. **Software local em baixo** → emissão manual em papel pré-impresso
   (tipografia autorizada); ao repor o sistema, cada documento manual é
   reintroduzido como um novo documento do mesmo tipo, campo
   `SourceBilling = "M"`, com campos próprios para guardar a série/número
   manual originais, e um `HashControl` específico
   (`versão-chave-privada - TipoM espaço série/número-manual`).
2. **Restauro de cópia de segurança com documentos perdidos** → os
   documentos que não constam do backup restaurado são reintroduzidos via
   os seus duplicados, também em série de recuperação anual própria,
   `SourceBilling = "M"`, `HashControl` referenciando o `InvoiceNo`
   original.

Isto é bastante mais específico do que a nossa tabela genérica
`sync_queue` — ver mapeamento abaixo.

## 7. Mapeamento ao nosso schema (`src-tauri/migrations/0001_init.sql`)

| Requisito AGT | Estado no nosso modelo |
|---|---|
| `documentos.hash` guarda a assinatura RSA-SHA1 (172 chars, base64) | Coluna já existe (`hash TEXT`), mas **falta implementar** a cadeia por estabelecimento+tipo+série e a lógica de assinatura em si — nada disto está codificado ainda. |
| `documentos.codigo_certificacao` = número de certificado da AGT para o carimbo impresso | Coluna já existe. Falta decidir a fonte de verdade — hoje é só uma coluna por documento; devia vir de uma configuração única a nível de instalação/empresa, não escrita solta por linha. |
| Fragmento de 4 caracteres da assinatura (posições 1,11,21,31) no impresso | Não precisa de coluna nova — pode ser calculado a partir de `documentos.hash` no momento de imprimir/exportar. |
| `InvoiceNo` = `"{código tipo} {série}/{número}"` | **Falta**: não temos uma tabela de mapeamento `tipo` (ex. `fatura`) → código AGT (`FT`), nem um formatador que componha esta string exata a partir de `prefixo`/`serie_documental_id`/`numero`. Necessário para SAF-T, webservice e o texto impresso. |
| Versão da chave privada usada para assinar cada documento | **Falta coluna** — sugiro `documentos.versao_chave_assinatura INTEGER`. |
| Par de chaves RSA do fabricante + n.º de certificado AGT | **Falta uma tabela/config** a nível de instalação (não por documento) — a chave privada nunca deve ficar em texto simples na BD de faturas. |
| `InvoiceStatus` (`N`/`A`) para exportação | O nosso `documentos.estado` tem 6 valores (mais granular); mapear só `emitido/pago_parcial/pago/vencido → N` e `anulado → A` na exportação — sem alterações ao schema. |
| Cadeia de contingência (série de recuperação, `SourceBilling` M/I) | O `sync_queue` genérico atual **não cobre isto** — é sobre comunicação com a AGT, não sobre o próprio motor de numeração/série em modo de recuperação. Recomendo modelar mais tarde um campo tipo `documentos.origem_registo` (`normal`/`manual`/`integrado`/`recuperado_backup`) e o conceito de série de recuperação, distintos da fila de sincronização. |
| Regra "sem nota de crédito sobre documento já anulado/retificado" etc. | Nada disto está nas *constraints* da BD (nem seria fácil pôr em `CHECK` do SQLite) — fica para a camada de aplicação (Rust), a implementar junto do motor de emissão/anulação. |
| Rotação periódica de password | Não modelado ainda — falta um campo tipo `utilizadores.password_alterada_em` + regra de expiração. |

## 8. Ainda por confirmar / baixa confiança

- **Estatuto legal exato da fonte principal**: o documento "Regras e
  Requisitos" está rotulado como *proposta* de 2018, sem data de aprovação
  final visível no próprio ficheiro. Não encontrei o decreto executivo
  final publicado no Diário da República a confirmar se este texto (ou uma
  versão revista) está mesmo em vigor tal como lido, nem se foi atualizado
  para acompanhar o Decreto 71/25 de 2025.
- **QR code**: nenhuma fonte consultada (incluindo o Anexo I) menciona
  QR code — só o carimbo textual de 4 caracteres + certificado. Isto é
  atípico face a regimes semelhantes mais recentes (ex. Portugal adicionou
  ATCUD/QR code ao SAF-T bastante depois da regra de hash original). Vale a
  pena confirmar diretamente com a AGT ou um fornecedor já certificado
  antes de fechar o design de impressão.
- **RSA-1024 + SHA-1**: parâmetros criptográficos fracos para os padrões
  atuais (2026). Não confirmei se a especificação de 2025 os manteve ou
  atualizou — a implementação devia permitir trocar o algoritmo sem grande
  refactor, e isto deve ser confirmado antes de implementar a assinatura a
  sério.
- **Processo de certificação em si** (documentos a submeter, taxas,
  processo de teste/auditoria, prazos, lista pública de software já
  certificado): não encontrado em detalhe — só o mecanismo de registo/
  rotação de chave via "Declaração Modelo 8".
- **Contrato de retry/fila para falha de transmissão em tempo real à AGT**
  (não confundir com o cenário "software local em baixo", esse sim
  documentado): não encontrado.
- Uma página (`portaldocontribuinte.minfin.gov.ao/noticia?id=985537`,
  sobre regularização de softwares de facturação) não foi possível
  carregar (erro de certificado TLS) — pode ter informação adicional sobre
  o processo de certificação em curso em 2025/2026.
- **O Decreto Executivo do artigo 36.º** (que fixaria em definitivo a
  estrutura de dados, algoritmo de assinatura exacto, e demais
  especificações técnicas) **não foi encontrado publicado** — é a peça em
  falta mais importante. Até aparecer, RSA-1024/SHA-1 e a ausência de QR
  code continuam como hipótese de trabalho (secção 3), não como facto
  confirmado.

## Fontes

**Lidas directamente na íntegra (2026-09-29):**

- [Decreto Presidencial n.º 71/25, 20 de março de 2025 — Diário da
  República I Série n.º 52](https://www.ucm.minfin.gov.ao/cs/groups/public/documents/document/aw40/otu2/~edisp/minfin4956210.pdf)
  — texto integral, 21 páginas, publicação oficial.
- **Regras e Requisitos para Validação de Sistemas de Processamento
  Electrónico de Facturação dos Contribuintes** (Anexos I e II) —
  `http://www.ucm.minfin.gov.ao/cs/groups/public/documents/document/zmlu/mdu1/~edisp/minfin055809.pdf`
  — 27 páginas, lidas na íntegra; confirmado ser um molde de "Proposta de
  Decreto Executivo" de 2018 nunca preenchido/assinado (ver §0).

**Consultadas via pesquisa/resumo (não lidas na íntegra):**

- [Faturação eletrónica em Angola para empresas — Primavera BSS](https://ao.primaverabss.com/pt/blog/faturacao-eletronica-angola-empresas/)
- [Facturação Electrónica obrigatória em Angola — Cegid](https://www.cegid.com/ao/o-que-e-a-facturacao-electronica/)
- [Angola: adotada nova legislação referente à faturação — Saphety](https://saphety.com/blog/angola-adotada-nova-legislacao-referente-a-faturacao)
- [Como escolher software certificado pela AGT — Cegid](https://www.cegid.com/ao/blog/software-certificado-agt-factura-electronica/)
- [Angola avança para a obrigatoriedade da fatura eletrónica — EDICOM](https://edicomgroup.com/pt/blog/angola-obrigatoriedade-fatura-eletronica)
- [Suspensão do Software de Facturação SAC5 — Portal do Contribuinte](https://portaldocontribuinte.minfin.gov.ao/noticia?id=985517)
- [SAF-T-AO XSD schema — GitHub assoft-portugal](https://github.com/assoft-portugal/SAF-T-AO)
- Encontrados via pesquisa mas não lidos em detalhe (podem valer
  investigação futura): [Portal do Contribuinte — regularização de
  softwares](https://portaldocontribuinte.minfin.gov.ao/noticia?id=985537)
  (falhou por erro de certificado TLS),
  [Cegid Vendus — O que é o SAF-T?](https://www.vendus.co.ao/blog/o-que-e-o-saf-t/),
  [Portal do Contribuinte — Submissão SAF-T
  Faturação](https://portaldocontribuinte.minfin.gov.ao/noticia?id=809127),
  [Validador SAFT Angola](https://validador.cacimboweb.com/),
  [SIGESC blog](https://sisgesc.net/blog/posts/faturacao-eletronica-obrigatoria-prazos-requisitos-pme-angola).
