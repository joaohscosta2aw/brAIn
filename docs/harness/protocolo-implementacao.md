# Protocolo de Implementação

Para quem vai escrever código. Você não precisa conhecer nenhuma conversa anterior.

## Antes de escrever qualquer linha

0. Confira `docs/harness/ambiente.md`. Se um pré-requisito estiver ausente,
   resolva antes — não improvise toolchain nem comando.
1. Leia `AGENTS.md`. São as leis, a fronteira e a ordem de construção.
2. Descubra a change ativa: `openspec list`.
3. Leia, naquela change: `proposal.md` (por quê), os arquivos em `specs/`
   (o que), `design.md` (como), `tasks.md` (em que ordem).
4. Leia `docs/harness/autonomia-e-memoria.md` para saber o que você pode
   decidir sozinho e o que exige perguntar.

Se a change não tem os quatro artefatos, ela não está pronta para implementação.
Não comece.

## Ao executar

Execute as tasks na ordem declarada. As dependências entre grupos estão escritas
em `tasks.md` e existem por um motivo.

Para cada task:

- Implemente **a solução mais simples que satisfaça o spec**. Não generalize antes
  de existir necessidade real, não crie abstração para possibilidade hipotética,
  não introduza infraestrutura para escala inexistente.
- Simplicidade não autoriza cortar validação em fronteira de confiança, tratamento
  de erro que evita perda de dado, nem segurança.
- Marque a task como concluída apenas quando ela for verificável — quando você
  consegue mostrar que está feita.
- Não refatore área não relacionada. Se encontrar algo errado fora do escopo,
  registre e siga.

## Ao terminar cada grupo de tasks

```
cargo test
./scripts/verificar-invariantes.sh
```

Confronte o resultado contra os cenários do spec, não contra sua própria
expectativa.

Um cenário do spec que você não consegue exercitar é sinal de uma destas três
coisas, e vale descobrir qual: o código está incompleto, o cenário está mal
escrito, ou o requisito é inverificável como está.

## Quando a realidade discordar do spec

Vai acontecer. Um provider não expõe o que o spec presumiu, um requisito se revela
ambíguo, uma decisão de design não sobrevive ao contato com o código.

**Não resolva silenciosamente no código.** A ordem é:

1. Pare a task.
2. Descreva a divergência: o que o spec diz, o que a realidade impõe.
3. Se for interpretação de ambiguidade sem mudança de comportamento → decida,
   registre no `design.md` da change, siga (YELLOW).
4. Se muda o comportamento especificado → volta para o spec. Isso é RED.

Comportamento aprovado é contrato. Código que diverge dele em silêncio é a falha
que este harness inteiro existe para impedir.

## Ao encerrar a change

- Todas as tasks marcadas e verificadas.
- Todos os cenários dos specs cobertos por teste, incluindo os unhappy paths.
- Decisões YELLOW registradas onde pertencem.
- Nenhuma decisão RED tomada sem confirmação humana.

Então sincronize os specs e arquive a change pelo fluxo do OpenSpec.
