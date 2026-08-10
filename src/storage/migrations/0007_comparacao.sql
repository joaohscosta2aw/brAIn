-- Migração 7: comparação pareada (paired-comparison, blueprint §38.4).
-- Aditiva: nenhuma coluna de tabela existente muda. Cada candidato é um
-- `run` real (tabela `run`, já existente) -- esta migração só agrupa
-- candidatos sob uma comparação e guarda a escolha do operador.

-- Persistida ANTES de qualquer candidato rodar (D-12, mesma disciplina de
-- `run`/`workflow_run`). vencedor_provider_id fica NULL até o operador
-- escolher explicitamente (spec: "Escolha do vencedor é sempre uma ação
-- explícita separada") -- nunca preenchido pela execução dos candidatos.
CREATE TABLE comparacao (
    id                   TEXT PRIMARY KEY,
    client_id            TEXT NOT NULL REFERENCES client(id),
    project              TEXT,
    tarefa               TEXT NOT NULL,
    vencedor_provider_id TEXT,
    started_at           INTEGER NOT NULL,
    finished_at          INTEGER
);

-- run_id nullable só por simetria com workflow_phase_entry -- na prática
-- todo candidato desta change sempre tem run_id, já que a comparação
-- inteira falha antes de rodar qualquer candidato se algum provider for
-- inválido (spec: "Candidato inválido falha a comparação inteira").
CREATE TABLE comparacao_candidato (
    id             TEXT PRIMARY KEY,
    comparacao_id  TEXT NOT NULL REFERENCES comparacao(id),
    provider_id    TEXT NOT NULL,
    run_id         TEXT REFERENCES run(id)
);

CREATE INDEX idx_comparacao_candidato_comparacao ON comparacao_candidato(comparacao_id);
