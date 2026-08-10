-- Migração 6: máquina de estados de workflow (workflow-engine). Aditiva:
-- nenhuma coluna de tabela existente muda. Cada fase não-terminal do
-- workflow é um `run` real (tabela `run`, já existente) -- esta migração só
-- guarda o estado da máquina em si e o histórico de fases.

-- Persistido ANTES de qualquer fase rodar (D-12, mesma disciplina de `run`).
-- workflow_version é congelado no momento da criação -- nunca reavaliado
-- depois (design.md: "congelado no início do run"). definicao_json é o
-- conteúdo bruto do arquivo de definição no momento da criação -- toda
-- decisão de transição posterior (inclusive em `brian workflow approve`,
-- uma chamada de CLI separada) lê essa cópia, nunca o arquivo no disco de
-- novo, senão uma edição do arquivo entre `run` e `approve` mudaria o
-- comportamento de um run em andamento (violaria a garantia acima).
-- tarefa é gravada uma vez e reaproveitada em toda fase e em `approve`
-- (nunca re-digitada pelo operador, que poderia divergir da original).
CREATE TABLE workflow_run (
    id             TEXT PRIMARY KEY,
    client_id      TEXT NOT NULL REFERENCES client(id),
    project        TEXT,
    workflow_id    TEXT NOT NULL,
    workflow_version INTEGER NOT NULL,
    definicao_json TEXT NOT NULL,
    tarefa         TEXT NOT NULL,
    current_phase  TEXT NOT NULL,
    status         TEXT NOT NULL,
    pause_reason   TEXT,
    total_phases   INTEGER NOT NULL,
    started_at     INTEGER NOT NULL,
    finished_at    INTEGER
);

CREATE INDEX idx_workflow_run_status ON workflow_run(status);

-- Histórico de fases (blueprint §15.6: phase_history) -- run_id é nullable
-- porque fases terminais (done/escalate/fail) não executam `iniciar_run`.
CREATE TABLE workflow_phase_entry (
    id               TEXT PRIMARY KEY,
    workflow_run_id  TEXT NOT NULL REFERENCES workflow_run(id),
    phase_id         TEXT NOT NULL,
    run_id           TEXT REFERENCES run(id),
    outcome          TEXT,
    entrada_numero   INTEGER NOT NULL,
    started_at       INTEGER NOT NULL,
    ended_at         INTEGER
);

CREATE INDEX idx_workflow_phase_entry_run ON workflow_phase_entry(workflow_run_id, started_at);
