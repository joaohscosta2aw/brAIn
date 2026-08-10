## Context

`router::decidir` (`provider-router-rules-minimal`, já arquivada) resolve
regra→provider de forma pura, sem histórico. O `run` table já persiste
`status`/`started_at`/`finished_at` para todo run real (D-12, desde
`isolated-tracked-run`). Este design usa esse histórico já existente — sem
tabela nova — para produzir um score, só quando `--scored` pede.

Ver proposal.md para motivação (reversão consciente de D-8) e não-objetivos
(sem fórmula de "seis termos" — não documentada no blueprint atual).

## Goals / Non-Goals

**Goals:**
- Score simples, defensável, 100% honesto sobre o `n` que o sustenta.
- Zero regressão: sem `--scored`, comportamento idêntico a
  `routing/provider-rules`.

**Non-Goals:**
- Sem custo/retries no score (Brian não calcula nenhum dos dois hoje —
  non-goal do proposal.md).
- Sem pesos configuráveis/aprendidos (Fase 3 "pesos aprendidos" do blueprint
  não tem onde aprender pesos sem uma base de avaliação humana, que não
  existe) — score usa uma fórmula fixa, não ajustável.

## Decisions

**Fórmula: taxa de sucesso, desempate por duração, desempate final por
nome.** Para cada provider candidato (disponível e aprovado pela regra Fase
1, se houver): `taxa_sucesso = concluidos / n` sobre os runs finalizados
desse `client_id`; ranking por `(taxa_sucesso desc, duracao_media asc,
provider_id asc)`. Alternativas consideradas e rejeitadas:
- **Pesos aprendidos** (blueprint Fase 3 literal): não há onde aprender —
  sem avaliação humana rotulada (blueprint §112.5, Evaluator, que não
  existe), pesos "aprendidos" seriam só números inventados vestidos de
  aprendizado. Rejeitado por ser menos honesto que uma fórmula fixa e
  declarada.
- **Fórmula de seis termos do v0.1-draft**: não documentada no blueprint
  atual — não há o que reproduzir.

**Provider sem histórico (`n=0`) entra no ranking com `taxa_sucesso = 0.0`,
tratado explicitamente como conservador, não como neutro.** Alternativa
considerada: tratar `n=0` como neutro (ex.: 0.5) — rejeitada por inventar um
número sem base nenhuma, o que é exatamente o tipo de "confiança
injustificada" que D-8 existia para evitar. A escolha conservadora
("provider nunca testado fica em último até ter dado real") é simples,
determinística e declarada — nunca escondida (spec: "Provider sem histórico
nenhum não é penalizado silenciosamente" garante que `n=0` aparece explícito
na explicação, mesmo que rankeado por último).

**`runs_finalizados_do_cliente(client_id)` nova no `Store`** — reaproveita
exatamente a mesma tabela `run`, mesmo padrão de `runs_em_execucao`/
`runs_abandonados` já existentes (filtro por `status IN
('concluido','falhou')`).

**`--scored` é opt-in explícito, não o padrão.** Reduz o risco de o operador
achar que está recebendo "roteamento inteligente" por padrão quando na
prática o `n` disponível ainda é pequeno — mesma disciplina de
transparência que levou a registrar a reversão de D-8 em
`docs/DECISIONS.md`.

**`brian router score --provider <id>`** roda a mesma função de cálculo de
score, sem decidir nada — só mostra `(taxa_sucesso, duracao_media, n)` por
provider candidato, para auditoria manual sem precisar de um run real
(mesmo papel que `--explain-only` cumpre para `routing/provider-rules`).

## Risks / Trade-offs

- **`n` pequeno produz decisões instáveis** (um único run ruim derruba a
  taxa de sucesso inteira) → aceito e declarado — é exatamente o "risco
  assumido" registrado na nota de reversão de D-8/D-13.
- **Duração como critério de desempate pode favorecer runs triviais** (uma
  tarefa pequena termina rápido, não significa "melhor provider") → aceito;
  é só desempate secundário, a taxa de sucesso já é o critério primário.
