## Context

`execucao::executar_provider` hoje monta os argumentos do `codex`
diretamente no corpo da função, sem nenhum ponto de despacho — funcionou
enquanto só havia um provider verificado. Ver proposal.md para a
verificação manual real que confirmou `grok -p` e descartou `agy`/Gemini.

## Decisions

**`executar_provider` passa a despachar por `provider_id`** em vez de
assumir `codex` implicitamente:

```rust
match provider_id {
    "codex" => executar_codex(worktree, tarefa, model),
    "grok" => executar_grok(worktree, tarefa, model),
    _ => unreachable!(), // já filtrado por PROVIDERS_EXECUCAO_VERIFICADA antes de chegar aqui
}
```

**`executar_grok`**: `grok --cwd <worktree> -p <tarefa> --permission-mode
bypassPermissions [-m <model>]`, `stdin` nulo (mesmo padrão de
`executar_codex`). `bypassPermissions` é o modo confirmado ao vivo que
não trava esperando aprovação interativa (proposal.md) — mesmo papel que
`-s workspace-write` cumpre para o `codex`, embora não seja tecnicamente
um sandbox, é o que garante execução não-interativa de verdade.

**Sem novo campo em `ResultadoProvider`**: `sucesso` continua sendo
`saida.status.success()`, `resumo_stderr` continua as 3 primeiras linhas
de stderr — mesmo contrato para os dois providers, `executar_provider`
não muda de assinatura, só o corpo interno passa a ramificar.

**Trailers de proveniência continuam opcionais** (código já existente,
`aplicar_trailers_se_houver_commit_novo`): como `grok -p` não cria commit
próprio, um run via Grok nunca ganha trailer automático — comportamento
já tratado sem erro, só documentado aqui como consequência esperada, não
nova (proposal.md, não-objetivos).

## Risks / Trade-offs

- **Grok não commita**: run via Grok sempre chega ao gate com mudanças
  não commitadas no worktree — aceito, gate roda sobre a árvore de
  trabalho, não sobre o commit, então isso não quebra `deterministic-gate`.
  Ausência de trailer de proveniência é a única perda real, já declarada.
