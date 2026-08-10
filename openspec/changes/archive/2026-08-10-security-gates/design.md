## Context

Ferramentas instaladas via Homebrew e testadas ao vivo contra fixtures
reais antes de qualquer código (proposal.md). Contratos de saída
confirmados:

```text
gitleaks detect --source <dir> --no-git -f json -r <arquivo_tmp>
  exit 0, arquivo = []           -- nenhum achado
  exit 1, arquivo = [ {...} ]    -- achados (RuleID, StartLine, File, Description, Secret)
  -r /dev/stdout NÃO funciona (gitleaks recusa: "Report path is not writable")
  -> precisa escrever num arquivo temporário real e ler de volta

semgrep scan --config auto --json --quiet <dir>
  exit 0 sempre (não falha por achado, só por erro de execução)
  JSON no stdout: results[].{check_id, path, start.line, extra.{severity, message}}

osv-scanner scan source --format json -r <dir>
  exit 0 (limpo) / 1 (achados) / 128 (nenhum manifest de dependência encontrado)
  JSON no stdout quando há algo pra reportar; stdout vazio quando não há
  manifest nenhum -- tratado como "sem achados", não como erro
```

## Decisions

**`src/security.rs`, um `Achado` comum para as três ferramentas:**

```rust
pub struct Achado {
    pub ferramenta: String,   // "gitleaks" | "semgrep" | "osv-scanner"
    pub severidade: String,   // como a ferramenta relata, sem normalizar
    pub arquivo: String,
    pub linha: Option<u32>,
    pub mensagem: String,
}
```

Sem normalizar severidade entre ferramentas (cada uma usa escala
própria: gitleaks não tem severidade, semgrep usa ERROR/WARNING/INFO,
osv-scanner usa CVSS) -- inventar uma escala unificada seria fabricar
precisão que não existe. `severidade` é o texto cru de cada ferramenta.

**`rodar_gitleaks(dir)`**: escreve o relatório num arquivo temporário
único (`testutil`-like: `std::env::temp_dir()` + nome único), roda o
comando, lê o arquivo, deleta, parseia o array JSON. Ignora o exit code
(0 ou 1 são ambos "rodou com sucesso", só o conteúdo do array importa) --
só um erro de processo (comando não encontrado, etc.) é `ErroSecurity`.

**`rodar_semgrep(dir)`**: `semgrep scan --config auto --json --quiet
<dir>`, parseia `results` do stdout via `serde_json::Value` (mesmo padrão
de `router::carregar_regras`, sem derive).

**`rodar_osv_scanner(dir)`**: `osv-scanner scan source --format json -r
<dir>`, stdout vazio -> `Vec::new()` (spec: "nenhum manifest encontrado
não é achado nem erro"), stdout não-vazio -> parseia `results[].packages[].vulnerabilities[]`.

**Integração obrigatória em `execucao::iniciar_run`**: depois do gate
(ou da ausência dele), sempre que o worktree existe -- `rodar_gitleaks`
é chamado incondicionalmente, mesmo que o provider tenha falhado (spec:
"roda sempre, em todo run"). `decidir_status_final` ganha um parâmetro
`achados_secretos: &[Achado]` -- não vazio força `StatusRun::Falhou` com
o achado no motivo, sobrepondo qualquer resultado de provider/gate.
Evento `security.secrets.scan` registrado sempre (achados ou não), mesmo
padrão de `gate.run`.

**Falha da própria ferramenta (`gitleaks` não instalado, etc.) não
derruba o run.** Mesma disciplina de `aplicar_trailers_se_houver_commit_novo`:
registra evento `security.secrets.failed`, não bloqueia o run por uma
ferramenta de terceiro estar ausente -- mas **não silencia**: o evento
fica visível em `brian recover`/logs para o operador notar que o scan
não rodou de verdade.

**`brian security scan --sast|--dependencies --path <dir>`**: comando
novo, chama `rodar_semgrep`/`rodar_osv_scanner` diretamente, formata a
lista de achados. Sem persistência, sem comparação com execução
anterior (proposal.md, não-objetivos).

## Risks / Trade-offs

- **Sem baseline/diff**: primeira chamada de `brian security scan` num
  repositório legado pode reportar dezenas de achados pré-existentes --
  aceito e declarado (proposal.md); é exatamente o ruído que o blueprint
  descreve, mas resolver isso exige a infraestrutura de baseline que foi
  conscientemente deixada de fora desta v1.
- **Secret scan roda mesmo em run falho**: gasta o tempo do scan mesmo
  quando o provider já falhou e o run já seria `Falhou` de qualquer
  forma -- aceito, é o comportamento mais seguro (nunca perder um
  segredo commitado só porque o resto do run deu errado).
- **Regras padrão do `gitleaks` não cobriam sintaxe de declaração de
  constante do Rust -- achado real, corrigido, não uma suposição
  evitada.** `gitleaks detect` com as regras built-in
  (`generic-api-key`) detecta `API_KEY = "sk-live-..."` (estilo
  Python/shell/dotenv) mas **não** detectava `pub const API_KEY: &str =
  "sk-live-...";` (sintaxe típica de Rust) -- confirmado num `brian run`
  real na primeira tentativa: uma credencial hardcoded nessa forma
  sobreviveu ao secret scan e o run foi marcado `Concluido`
  incorretamente. Isolado depois: o mesmo worktree, com o mesmo segredo
  em sintaxe `.env` (`API_KEY=...`), foi detectado normalmente --
  confirmou que a integração (`rodar_gitleaks`, `decidir_status_final`)
  estava correta; a lacuna era só de cobertura de regras do `gitleaks`
  para Rust.

  **Corrigido com `security/gitleaks-rust.toml`** (novo arquivo,
  embutido no binário via `include_str!`, `useDefault = true` +
  `[[rules]] id = "rust-const-secret"`): regex cobrindo
  `const`/`static`/`let` com nome contendo
  `key`/`secret`/`token`/`password`/`credential`. Testado ao vivo contra
  o próprio código-fonte do Brian (113MB, todo o repositório) sem nenhum
  falso positivo, e um segundo `brian run` real repetindo exatamente o
  cenário do primeiro teste confirmou o run agora reprovando
  corretamente (`Falhou`, motivo: "segredo encontrado por gitleaks...
  rust-const-secret"), mesmo com o gate configurado passando.
