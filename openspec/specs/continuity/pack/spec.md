## Purpose

Monta, a partir das notas registradas e do estado real do repositório, o artefato
denso que um próximo worker recebe para não recomeçar do zero — nunca o transcript
bruto, sempre orçado.

## Requirements

### Requirement: Pack montado a partir das notas do Context ativo

O sistema SHALL montar o Continuity Pack agrupando as notas do Context ativo por
categoria: objetivo, decisões (com motivo), análise, tentativas que falharam,
próximos passos.

#### Scenario: Pack com notas de todas as categorias
- **GIVEN** um Context com notas de várias categorias registradas
- **WHEN** o pack é montado
- **THEN** cada categoria aparece separadamente, nenhuma nota é omitida

#### Scenario: Pack de Context sem nenhuma nota
- **GIVEN** um Context ativo sem notas registradas
- **WHEN** o pack é montado
- **THEN** o pack é montado vazio nas categorias sem nota, não é um erro

### Requirement: Arquivos tocados vêm do repositório real, nunca inventados

O sistema SHALL derivar a seção de arquivos tocados do estado real do repositório
associado ao Context (via `git`) — nunca listar arquivo que a nota ou o pack não
possa apontar para uma origem real (critério de aceitação do blueprint: "o
Continuity Pack cita arquivos/símbolos reais do trabalho").

#### Scenario: Repositório com alterações não commitadas
- **GIVEN** o diretório do Context ativo com alterações não commitadas
- **WHEN** o pack é montado
- **THEN** os arquivos alterados aparecem na seção de arquivos tocados

#### Scenario: Repositório sem alterações
- **GIVEN** o diretório do Context ativo sem alterações pendentes
- **WHEN** o pack é montado
- **THEN** a seção de arquivos tocados vem vazia, não inventada

### Requirement: Pack nunca contém transcript bruto

O pack SHALL conter apenas notas estruturadas e o diff de arquivos tocados — SHALL
NOT incluir log de conversa integral com um provider.

#### Scenario: Composição do pack
- **WHEN** o pack é montado
- **THEN** todo o conteúdo vem de notas registradas explicitamente ou do diff real
  do repositório — nenhuma fonte de transcript bruto de provider é lida

### Requirement: Pack é orçado, avisa quando grande

O sistema SHALL sinalizar quando o pack monta acima de um tamanho de referência,
sem truncar conteúdo em silêncio.

#### Scenario: Pack dentro do orçamento
- **WHEN** o pack montado fica dentro do tamanho de referência
- **THEN** nenhum aviso aparece

#### Scenario: Pack acima do orçamento
- **WHEN** o pack montado excede o tamanho de referência
- **THEN** o sistema sinaliza isso explicitamente
- **AND** o conteúdo continua completo, não é cortado sem aviso
