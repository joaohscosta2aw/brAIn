-- Migração 8: experimento H-1 do Context Governor (context-governor-
-- experiment, blueprint §18.3). Aditiva: nenhuma coluna existente muda.
-- Cada braço é uma execução real (tabela `run`, já existente) -- esta
-- migração só liga cada execução ao case sintético e ao braço que a gerou.

CREATE TABLE experimento_execucao (
    id         TEXT PRIMARY KEY,
    case_id    TEXT NOT NULL,
    braco      TEXT NOT NULL,
    run_id     TEXT NOT NULL REFERENCES run(id),
    started_at INTEGER NOT NULL
);

CREATE INDEX idx_experimento_execucao_case ON experimento_execucao(case_id);
