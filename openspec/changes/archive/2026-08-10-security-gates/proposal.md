## Why

Blueprint §30: Semgrep (SAST), OSV Scanner (dependências vulneráveis) e
secret scanner são as três ferramentas explicitamente nomeadas — secret
scanning é chamado de "o único achado que não tem correção barata depois
do fato" e deve rodar sempre, em qualquer workflow. Hoje Brian não tem
nenhuma checagem de segurança automática: um run pode terminar
`Concluido` mesmo tendo commitado uma credencial. Verificação real feita
antes de propor: instalei `semgrep`, `gitleaks`, `osv-scanner` via
Homebrew e testei os três contra fixtures reais com vulnerabilidade
conhecida (shell injection, chave de API hardcoded, dependência com CVE
público) — os três funcionam, produzem JSON parseável.

## What Changes

- `src/security.rs` (novo): `rodar_gitleaks`, `rodar_semgrep`,
  `rodar_osv_scanner` — cada um roda a ferramenta real contra um
  diretório e devolve uma lista de `Achado` (ferramenta, severidade,
  arquivo, linha, mensagem) a partir do JSON real de saída.
- **Secret scan (`gitleaks`) passa a ser obrigatório em todo `brian run`**:
  roda no worktree logo após o provider terminar, antes de decidir o
  status final do run. Se encontrar qualquer segredo, o run é marcado
  `Falhou` — mesmo que o provider e o gate tenham tido sucesso. Secret
  scan nunca é opcional nem configurável para desligar (mesma ênfase do
  blueprint: "roda sempre").
- `brian security scan --sast|--dependencies --path <dir>`: SAST
  (semgrep) e dependências (osv-scanner) expostos como comando manual do
  operador — não são automáticos em todo run (ver Não-objetivos).

## Capabilities

### New Capabilities
- `execution/security-gates`: scan de segredo obrigatório em todo run;
  SAST e dependências vulneráveis disponíveis sob demanda.

## Impact

- `src/security.rs` (novo).
- `src/execucao.rs`: `iniciar_run` ganha a chamada obrigatória a
  `security::rodar_gitleaks` entre a execução do provider e
  `decidir_status_final`.
- `src/comandos.rs`, `src/main.rs`: `brian security scan`.
- Sem tabela nova: achados de um run não são persistidos separadamente
  nesta v1 (ver Não-objetivos) — aparecem no motivo de falha do run,
  mesmo canal que qualquer outro erro (`RunRegistrado`/`EventoDeRun` já
  existentes).

## Não-objetivos

- **Sem baseline nem escopo de diff** (blueprint §30.1: primeira execução
  gera baseline completo, execuções seguintes só reportam achados novos
  vs. pré-existentes): isso exige um conceito de "baseline por Context"
  persistido, comparação achado-a-achado entre execuções — infraestrutura
  real, não uma extensão pequena. Nesta v1, SAST/dependências escaneiam o
  diretório inteiro a cada chamada manual, sem comparação com execução
  anterior — declarado explicitamente, não escondido atrás de um "TODO".
- **SAST e dependências não são automáticos em todo `brian run`**: rodar
  `semgrep --config auto` a cada run adicionaria segundos-a-minutos reais
  (rodou ~1.5s no fixture minúsculo de teste; um repositório real é mais
  lento) a todo run, sempre — custo real que o operador deveria escolher,
  não que Brian impõe por padrão. Secret scan é a única exceção porque o
  blueprint é explícito sobre isso ser inegociável; SAST/dependências
  ficam como comando `brian security scan` que o operador roda quando
  quiser (ex.: antes de um PR).
- **Sem SkillSpector nem `security.skill_scan`**: isso é sobre escanear
  skills/plugins de terceiros instalados no harness do assistente
  (blueprint §31) — Brian não tem conceito de "skill instalada", mesma
  categoria de fora-de-escopo já registrada para Graphify/grafo de
  código (`docs/DECISIONS.md`, nota de 2026-08-09).
- **Achados não são persistidos numa tabela própria**: o motivo de falha
  de um run já vai para `RunRegistrado`/`EventoDeRun` (campo `detalhe`)
  quando o secret scan reprova — suficiente para esta v1. Uma tabela de
  achados de segurança pesquisável fica para quando houver demanda real
  (ex.: relatório histórico de segurança por cliente).

## Conformidade — checklist §16

- **Honestidade de capability**: as três ferramentas foram instaladas e
  testadas de verdade contra fixtures com vulnerabilidade conhecida antes
  de qualquer linha de código — mesma disciplina de
  `PROVIDERS_EXECUCAO_VERIFICADA`.
- **D-12**: secret scan roda antes de o run ser considerado concluído —
  mesma disciplina de "persistir/decidir antes de qualquer efeito
  colateral assumido como sucesso".
- **Versão alvo**: blueprint §85 (v0.4) lista Security Gates junto com
  memória e providers adicionais.
