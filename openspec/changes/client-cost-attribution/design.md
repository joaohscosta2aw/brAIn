## Context

Repositório sem código. Esta change cria o primeiro binário do projeto.
Motivação em `proposal.md`; requisitos nos specs desta change.

Decisões já travadas que este design **não** rediscute, apenas obedece:
Rust com CLI primeiro (**D-2**), SQLite único atrás de traits (**D-1**), SQL
confinado a `storage/` (**D-9**), hierarquia de integração headless JSON →
session files → PTY (**D-4**), custo do provider prevalece sobre catálogo (**D-6**).

Restrição que molda tudo: **não há daemon, orquestração nem UI na v0.0**. O modo é
observe — o Brian lê o que os providers já produziram, sem dirigir nenhum agente.

Escopo confirmado: os cinco providers do blueprint (Claude, Codex, Gemini, Grok,
ZCode). Isso é o que torna a change não-trivial — cada um expõe consumo de um jeito
diferente, e vários não expõem custo em dólar.

## Goals / Non-Goals

**Goals:**

- Núcleo do ledger e da atribuição totalmente agnóstico de provider.
- Cada provider entra como incremento independente; a falha de um não derruba os outros.
- Procedência de cada número rastreável até sua origem.
- Esquema de dados que a change `capacity-windows-and-plans` estenda sem migração destrutiva.

**Non-Goals (nível de design):**

- Coleta contínua ou em tempo real. A importação é sob demanda.
- Qualquer pré-cálculo de janela, saldo ou percentual — pertence à change seguinte.
- Abstração de "provider" que antecipe roteamento ou execução (v0.2+). Aqui um
  provider é apenas uma fonte de consumo.

## Decisions

### Coleta por importação sob demanda, não por observação contínua

O operador dispara a importação; o sistema lê as fontes de uso já existentes de cada
provider e concilia com o ledger.

*Alternativa considerada:* watcher de arquivos ou hook pós-sessão, capturando consumo
no momento em que ocorre. Rejeitada para a v0.0 — exige processo residente, que a
ordem de construção coloca na v0.2. Importação idempotente entrega a mesma verdade
sem processo residente, ao custo de latência até o dado aparecer.

### Cada provider declara seu tier de integração; degradação é explícita

Cada adapter de coleta declara em qual tier de D-4 opera e quais campos consegue
fornecer. O núcleo não presume paridade entre providers: um adapter que não fornece
custo produz `cost_source = unknown`, e isso aparece no resultado em vez de ser
mascarado.

*Alternativa considerada:* trait única exigindo todos os campos de todo adapter.
Rejeitada — é o erro que o próprio blueprint corrigiu na revisão v1.0 (trait de oito
métodos forçando implementações incompletas). Capacidade declarada é honesta;
capacidade presumida vira número inventado.

### Idempotência por identificador estável do provider, com fallback impresso

A deduplicação usa o identificador de chamada fornecido pelo provider quando existe.
Quando não existe, o registro é identificado por uma impressão digital derivada de
provider, modelo, instante, contagens de token e referência de sessão.

*Alternativa considerada:* chave puramente posicional (arquivo + offset). Rejeitada
porque as fontes são reescritas e compactadas pelos próprios providers.

*Limitação aceita:* duas chamadas idênticas ao mesmo modelo no mesmo instante, sem ID
de provider e sem referência de sessão, colidem e contam como uma. A referência de
sessão reduz isso a um caso remoto; o adapter que não tiver nenhum desses sinais
declara tier degradado.

### Ausente, zero e desconhecido são três estados distintos

Token não reportado é ausente. Token reportado como nenhum consumo é zero. Custo que
não pôde ser determinado é desconhecido, nunca zero.

### Custo pago e custo equivalente são colunas distintas, não alternativas

O registro carrega dois valores monetários independentes. O custo pago existe quando
o provider o informa. O custo equivalente em API é derivado de tokens e catálogo, e
é calculável para qualquer `billing_mode` — inclusive assinatura.

*Razão:* os dois respondem perguntas diferentes de negócio. O pago é base de custo;
o equivalente é base de faturamento, porque o cliente pode ser cobrado a preço de
token mesmo quando o consumo ocorreu sob assinatura. A diferença entre eles é margem,
e é o que sustenta a decisão de permanecer na assinatura ou migrar para API.

*Alternativa considerada:* um único campo de custo com rótulo de origem — foi o
primeiro desenho desta change. Rejeitada: força escolher entre os dois valores no
momento da gravação, destrói a informação de margem, e faria consumo de assinatura
aparecer como custo desconhecido quando na verdade seu valor faturável é perfeitamente
calculável.

*Restrição derivada:* o equivalente nunca pode ser apresentado como valor pago
(BRIAN-BLUEPRINT-V1.md §42.2). Confundi-los é erro de dinheiro, não de relatório.

A alocação proporcional do custo do plano entre clientes fica para
`capacity-windows-and-plans`, que introduz a declaração de planos.

### Correção de custo é supersessão auditável, não sobrescrita

Quando o custo real do provider chega depois de um registro estimado por catálogo, o
valor anterior e sua fonte permanecem recuperáveis. Isso segue o espírito de **D-14**
(correção cria registro que supersede) sem transformar o ledger inteiro em
append-only puro, que encareceria toda consulta agregada.

O mesmo vale para reatribuição manual de cliente.

### Isolamento entre clientes na forma da consulta

A camada de armazenamento expõe leitura de consumo já escopada por cliente. Não
existe caminho que retorne registros de múltiplos clientes e dependa do chamador
filtrar depois.

*Razão:* o spec exige isolamento por construção. Um filtro que o chamador pode
esquecer de aplicar é exatamente a falha que essa exigência existe para impedir.

### Instante em UTC, janelas derivadas na leitura

`occurred_at` é gravado como instante absoluto em UTC. Nenhuma chave de janela é
pré-calculada.

*Razão:* janelas são a próxima change e ainda podem mudar de definição. Derivar na
leitura mantém o esquema estável; se o volume tornar isso lento, índice ou coluna
derivada resolvem sem reescrever o histórico.

## Risks / Trade-offs

- **Cinco fontes de uso não documentadas e instáveis** → maior risco de cronograma
  da change. Mitigação: núcleo agnóstico e adapters independentes; a v0.0 fecha com
  os providers que funcionarem, e cada um que faltar é visível como cobertura
  declarada, não como silêncio.

- **Risco comercial R-4 (BRIAN-BLUEPRINT-V1.md §116):** os termos de uso de planos por
  assento podem proibir leitura automatizada de artefatos de sessão. Isso não é risco
  técnico e não se resolve em código. Mitigação: verificação humana antes de depender
  de qualquer fonte de assinatura; o desenho permite operar só com providers de API
  caso a verificação seja negativa.

- **Colisão de impressão digital sem ID de provider** → subcontagem silenciosa.
  Mitigação: a colisão só ocorre com instante idêntico e nenhuma referência de sessão;
  o adapter declara quando opera nesse regime.

- **Cobertura do catálogo de preço vira dependência de faturamento.** Como o custo
  equivalente é a base de cobrança, um modelo fora do catálogo deixa aquele consumo
  sem valor faturável — não apenas sem relatório. Mitigação: a parcela sem catálogo
  aparece explicitamente nas consultas e na exportação, para não virar receita
  perdida em silêncio.

- **Preço de tabela muda com o tempo.** O equivalente calculado hoje e o calculado
  daqui a seis meses para o mesmo consumo podem divergir se o catálogo for atualizado
  no lugar. Mitigação: o catálogo é versionado por vigência, de modo que o
  equivalente de um consumo passado permaneça reproduzível.

- **Crescimento do ledger** → D-1 estabelece que a decisão de SQLite se revê se uma
  consulta real passar de 200ms com doze meses de dados. Mitigação: medir com volume
  sintético antes de declarar a change pronta.

## Migration Plan

Não há dados anteriores a migrar. O esquema nasce versionado, com registro de
migrações aplicadas, para que `capacity-windows-and-plans` estenda tabelas existentes
sem recriar o banco.

Reversão: apagar o arquivo de banco. Nenhum efeito colateral externo é produzido por
esta change — nada é enviado, publicado ou modificado fora da máquina local.

## Open Questions

- Retenção do ledger a longo prazo (arquivamento de registros antigos). Não afeta
  specs nem tarefas desta change; decidir quando houver volume real.
- Se a impressão digital de deduplicação deve ser exposta como identificador estável
  para consumo externo. Só importa quando existir integração de faturamento.
