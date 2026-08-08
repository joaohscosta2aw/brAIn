# Protocolo de Revisão

Para quem revisa código do BrIAn. Você é revisor independente: o agente que
implementou não é revisor final do próprio trabalho.

**Não presuma que uma escolha está correta porque ela aparece na implementação.**

## Ordem da revisão

Nesta ordem, porque cada camada barata elimina trabalho da seguinte.

1. **Verificações determinísticas** — `cargo build`, `cargo fmt --check`,
   `cargo clippy -- -D warnings`, `cargo test` e
   `./scripts/verificar-invariantes.sh`. Comandos canônicos em
   `docs/harness/ambiente.md`.
2. **Aderência ao OpenSpec** — cada cenário do spec tem comportamento
   correspondente? Existe comportamento implementado que ninguém especificou?
3. **Correção** — a lógica faz o que diz fazer, incluindo nos limites.
4. **Segurança** — secrets, entrada não confiável, dado que sai da máquina.
5. **Edge cases e failure modes** — os unhappy paths do spec estão realmente
   exercitados, não apenas mencionados.
6. **Regressões** — o que mais toca este código continua funcionando.
7. **Arquitetura** — fronteiras respeitadas, decisões travadas obedecidas.
8. **Testes** — cobrem comportamento ou apenas repetem a implementação?
9. **Simplicidade** — o que dá para apagar sem perder comportamento?
10. **Performance**, quando houver critério declarado.

## O que este projeto exige olhar sempre

- **D-1..D-17 obedecidas.** Uma decisão travada violada não é preferência de
  estilo, é defeito.
- **SQL fora de `storage/`** — violação de D-9.
- **Caminho do dinheiro.** Cálculo de custo, atribuição, base de faturamento e
  exportação merecem revisão mais dura que o resto. Erro ali cobra o cliente errado.
- **Custo equivalente apresentado como valor pago** — proibido, e é o erro mais
  fácil de cometer sem perceber.
- **Ausente virando zero.** Token não reportado e consumo zero são fatos
  distintos; confundi-los corrompe o ledger em silêncio.
- **Isolamento entre clientes por filtro em vez de por construção** — o spec
  exige construção justamente porque filtro é esquecível.
- **Consumo descartado** por não ter dono. Nada pode sumir; vira `unattributed`.

## Como tratar achados

Um achado é hipótese até ser verificado. Classifique:

| Classe | Ação |
|---|---|
| Confirmado | corrigir |
| Provável | investigar antes de mexer |
| Precisa investigação | investigar se for materialmente importante |
| Falso positivo | descartar, sem alterar código |

**Não altere código apenas porque um revisor automatizado apontou algo.**
Revisão de IA informa a decisão; não a substitui.

## Ferramentas

Uso e hierarquia de autoridade quando ferramentas divergem:
`Prompts/ToolingAndContextPolicy.md`.

Em resumo: comportamento aprovado em OpenSpec vence contrato executável, que vence
código atual, que vence decisão de arquitetura, que vence índice ou grafo, que vence
diagrama, que vence auditoria de IA, que vence suposição do agente.

## Revisão humana obrigatória

Independente do que a revisão automatizada disser, exigem olho humano:
qualquer mudança no caminho do dinheiro, em segurança ou secrets, em contrato de
CLI, em esquema de dados com dados reais, e qualquer decisão RED.
