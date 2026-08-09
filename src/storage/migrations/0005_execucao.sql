-- Migração 5: runs isolados por worktree (isolated-tracked-run). Aditiva:
-- nenhuma coluna de tabela existente muda.

-- Gravado ANTES de qualquer efeito colateral (D-12, design.md: "Run persistido
-- antes de qualquer efeito colateral"). pid é preenchido só depois de o
-- processo do provider existir -- por isso é nullable, não é conhecido no
-- instante da criação do registro.
CREATE TABLE run (
    id                        TEXT PRIMARY KEY,
    client_id                 TEXT NOT NULL REFERENCES client(id),
    project                   TEXT,
    base_commit                TEXT NOT NULL,
    worktree_path              TEXT NOT NULL,
    branch                     TEXT NOT NULL,
    provider_id                TEXT NOT NULL,
    pid                        INTEGER,
    status                     TEXT NOT NULL,
    custo_equivalente_micros  INTEGER,
    started_at                 INTEGER NOT NULL,
    finished_at                INTEGER
);

CREATE INDEX idx_run_status ON run(status);

-- Log de eventos local, não OTel completo (design.md, decisão "Log de eventos
-- local, não OTel completo") -- suficiente para reconstruir o que aconteceu
-- num run, sem o subsistema de spans do blueprint §39.
CREATE TABLE run_event (
    id           TEXT PRIMARY KEY,
    run_id       TEXT NOT NULL REFERENCES run(id),
    tipo         TEXT NOT NULL,
    detalhe      TEXT,
    ocorrido_em  INTEGER NOT NULL
);

CREATE INDEX idx_run_event_run ON run_event(run_id, ocorrido_em);
