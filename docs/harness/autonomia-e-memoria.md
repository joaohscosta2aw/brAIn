# Autonomia e Memória

Governa toda sessão de trabalho no BrIAn, humana ou de agente.
Contexto mínimo do projeto: `AGENTS.md`.

## Autonomia

Três classes. Na dúvida entre duas, escolha a mais restritiva.

### GREEN — decida sozinho

Reversível em minutos, sem efeito externo.

- Nomes locais, organização interna de módulo, extração de função
- Formatação, comentários, mensagens de log internas
- Testes adicionais sobre comportamento já especificado
- Escolha entre construções equivalentes da linguagem

Não precisa registrar. Registrar isso é ruído.

### YELLOW — decida e registre

Tem consequência para quem vier depois, mas não muda o que o produto faz.

- Nova abstração ou mudança de fronteira entre módulos
- Nova dependência não crítica
- Decisão de performance com trade-off real
- Mudança na forma de um erro ou saída que alguém possa estar lendo
- Interpretação de um requisito ambíguo do spec

Registre onde a decisão vive: no `design.md` da change se for escopada a ela,
em `docs/DECISIONS.md` se influenciar decisões futuras fora dela.

### RED — pare e pergunte

Confirme com humano antes de agir. Nenhuma pressa justifica pular isto.

- Tocar qualquer decisão travada **D-1..D-17**
- Mudar comportamento descrito em spec aprovada
- **Qualquer coisa no caminho do dinheiro**: cálculo de custo, atribuição a
  cliente, base de faturamento, exportação usada para cobrar
- Contrato de CLI: nome de comando, nome ou semântica de flag, formato de saída
  que alguém automatize
- Esquema de dados depois que houver dados reais
- Secrets, credenciais, permissões
- Nova dependência crítica ou que execute código de terceiros
- Enviar, publicar ou expor qualquer dado para fora da máquina
- Alterar requisito do blueprint

O caminho do dinheiro é RED neste projeto porque erro ali não é defeito de
relatório: é cobrança errada de cliente.

## Memória

### O que fica sempre carregado — L0

`AGENTS.md`, e só. Identidade, as duas leis, a fronteira D-10, ordem de
construção, invariantes, e o índice de onde achar o resto.

Se algo não é necessário em praticamente toda tarefa, não entra aqui.

### Conhecimento estável do projeto — L1

`docs/DECISIONS.md` — decisões travadas.
`docs/PREMISSAS-BASICAS.md` — lei do repo.

Lido quando a tarefa envolve decisão de produto ou arquitetura.

### Contexto de domínio — L2

`openspec/specs/<capability>/` — comportamento aprovado daquela capability.
`openspec/changes/<change>/` — proposta, design e tasks da change ativa.

Lido apenas ao trabalhar naquele domínio ou naquela change.

### Conhecimento profundo sob demanda — L3

`BRIAN-BLUEPRINT-V1.md` — arquitetura completa, schema, glossário.
`BRIAN-BLUEPRINT.md` — v0.1-draft histórico, só para entender o porquê de uma decisão.
`docs/harness/` — protocolos de implementação e revisão.

Consultado por busca dirigida. Nunca carregado inteiro.

## O que vale persistir

Memória boa decide o futuro. Memória ruim narra o passado.

**Persista:** princípios, invariantes, razões, fronteiras, restrições, e decisões
que um agente futuro poderia razoavelmente tomar diferente sem conhecer o histórico.

**Não persista:** o que o código já diz, o que o git já registra, cronologia de
mudanças, nomes de funções criadas, ou qualquer fato que uma busca no repositório
responde em segundos.

Teste antes de escrever: *isto muda uma decisão futura, ou só conta o que aconteceu?*
Se só conta, não escreva.

## Onde uma decisão nova mora

| Alcance da decisão | Destino |
|---|---|
| Vale para o produto inteiro, influencia decisões futuras | `docs/DECISIONS.md` |
| Escopada a uma change | `design.md` daquela change |
| Muda comportamento do produto | não é decisão — é spec; volte para o OpenSpec |
| Trivial e reversível | lugar nenhum |

Não existe sistema de ADR paralelo neste repositório. `DECISIONS.md` já é isso.
