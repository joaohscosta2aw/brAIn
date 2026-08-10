## Context

`comandos::resolver_provider` (`provider-router-rules-minimal`, já arquivada)
já resolve o provider de um run antes de chamar `execucao::iniciar_run`,
respeitando override explícito. Este design encaixa uma resolução equivalente
para o modelo, na mesma camada (`comandos.rs`), sem tocar `execucao.rs`.

Ver proposal.md para motivação (blueprint §12) e não-objetivos (sem
auto-detecção de modelos por falta de fonte real).

## Goals / Non-Goals

**Goals:**
- `--model-pointer` resolve provider+tier+modelo concreto, com fallback e
  degradação avisada.
- `--model` (nome concreto) sempre vence, sem exceção.

**Non-Goals:**
- Sem `brian providers models <provider>` — non-goal do proposal.md.
- Sem alteração de contrato de `execucao::iniciar_run`.

## Decisions

**Dois arquivos de dado, JSON — mesmo padrão de `routing/rules.json`.**
`models/pointers.json` (pointer → primary/fallback de `{provider, tier}`) e
`providers/<provider>/models.json` (tier → nome concreto, `resolved_at`,
`resolution_source`). Caminhos relativos a `cwd`, mesma convenção de
`routing/rules.json`.

**"Disponível" = `execucao::PROVIDERS_EXECUCAO_VERIFICADA`.** Mesma fonte de
verdade que já decide se um provider pode executar de verdade — não uma
segunda checagem de disponibilidade inventada. Hoje só `codex` está nessa
lista, então qualquer pointer cujo primary/fallback não sejam `codex` vai
recusar — isso é o comportamento honesto esperado, não um bug: o Model Router
não finge que providers não verificados estão disponíveis.

**Resolução em `src/model_router.rs`, chamada de `comandos::executar_run`
antes de `execucao::iniciar_run` — mesma estrutura de `src/router.rs`
(Provider Router).** Reaproveita o padrão já estabelecido: função pura de
decisão, testável sem processo nenhum, camada fina de integração em
`comandos.rs`.

**Degradação de tier**: "tier mais próximo" segue a ordem fixa
`strong → balanced → cheap` (e o inverso, dependendo de qual falta) — só
strong/balanced/cheap existem no vocabulário do blueprint §12.3, sem inventar
tiers novos. Se o provider não tiver NENHUM tier configurado, é o mesmo caso
de "fallback também indisponível": erro explícito, run não inicia.

**Aviso de degradação**: como Brian não tem UI (D-2, CLI first), o aviso vai
para a mesma saída que já reporta a decisão de provider — reaproveita
`registrar_evento` de `execucao.rs`? Não: a resolução de modelo acontece
antes de `iniciar_run` existir (não há `run_id` ainda nesse ponto). O aviso
vai para stderr da CLI, mesmo padrão de qualquer erro de `brian run` hoje —
sem subsistema de trace novo (non-goal do proposal.md).

## Risks / Trade-offs

- **`models.json` desatualizado** (nome de modelo não existe mais no
  provider) → falha só quando o provider real rejeitar o nome (mesmo
  comportamento de `--model` manual hoje); Brian não valida nomes de modelo
  contra o provider nesta change.
- **Só um provider verificado hoje (`codex`)** → a maioria dos pointers do
  blueprint (`reasoning`→claude, `long-context`→gemini, `research`→grok) vai
  recusar por "fallback também indisponível" até mais providers ganharem
  execução verificada — aceito, é o `PROVIDERS_EXECUCAO_VERIFICADA` fazendo
  seu trabalho, não um defeito desta change.
