## Purpose

H-1 é uma hipótese, não um pilar (D-5) — decidir se o Context Governor
deveria existir exige dado real de um experimento, nunca suposição. Esta
capability produz esse dado, honestamente, incluindo suas próprias
limitações declaradas.

## Requirements

### Requirement: Pacote curado é montado sem grafo de código real

O sistema SHALL montar um `ContextPackage::Curated` a partir de busca por
palavras-chave, diff recente e notas de memória já existentes — SHALL NOT
alegar ou depender de um grafo de código que não existe no Brian.

#### Scenario: Pacote curado não inventa fonte que não existe
- **GIVEN** uma tarefa de experimento no braço B ou C
- **WHEN** o pacote é montado
- **THEN** ele é composto só de busca por palavra-chave, diff recente e
  notas de memória — nenhum componente alega vir de um grafo de código

### Requirement: Cada braço formata a mesma tarefa de forma diferente

O sistema SHALL executar a mesma tarefa-base nos três braços (A, B, C), cada
um com a tarefa formatada de acordo com sua definição — A sem pacote, B com
pacote e instrução de uso exclusivo, C com pacote e instrução de ponto de
partida.

#### Scenario: Braço A não recebe pacote curado
- **GIVEN** uma execução no braço A
- **WHEN** a tarefa é montada
- **THEN** ela não contém nenhum pacote curado, só a descrição original

#### Scenario: Braços B e C recebem o mesmo pacote com instruções diferentes
- **GIVEN** a mesma tarefa-base rodando nos braços B e C
- **WHEN** as tarefas são montadas
- **THEN** ambas incluem o mesmo pacote curado, mas com textos de instrução
  diferentes entre si

### Requirement: Relatório nunca esconde que custo em USD não é medido

Quando o sistema reporta resultado do experimento, ele SHALL declarar
explicitamente que a métrica primária do blueprint (custo em USD) não é
medida nesta implementação, e que duração é usada como proxy.

#### Scenario: Relatório declara a limitação de métrica
- **GIVEN** um relatório do experimento
- **WHEN** ele é gerado
- **THEN** contém uma nota explícita dizendo que custo em USD não foi
  medido e duração é usada como proxy

### Requirement: Relatório nunca esconde o tamanho da população

O relatório SHALL expor quantas execuções por braço alimentaram cada
número reportado — mesma disciplina de `routing/historical-scoring`.

#### Scenario: n aparece junto a cada taxa e duração reportadas
- **GIVEN** um relatório com execuções de múltiplos braços
- **WHEN** o relatório é gerado
- **THEN** cada braço mostra quantas execuções o compõem, ao lado da taxa
  de sucesso e duração média
