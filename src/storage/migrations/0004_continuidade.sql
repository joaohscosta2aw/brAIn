-- Migração 4: notas de memória para o Continuity Pack (continuity-pack-handoff).
-- Aditiva: nenhuma coluna de tabela existente muda.

-- Escopada por (client_id, project) -- os mesmos campos de `active_context`, não
-- um novo conceito de "contexto" (design.md: "Context de memória é
-- (client_id, project), reaproveitado de active_context"). `project` é NULL para
-- o perfil "padrão" do cliente, mesma convenção de `identity_profile`.
CREATE TABLE memory_note (
    id          TEXT PRIMARY KEY,
    client_id   TEXT NOT NULL REFERENCES client(id),
    project     TEXT,
    categoria   TEXT NOT NULL,
    texto       TEXT NOT NULL,
    rationale   TEXT,
    created_at  INTEGER NOT NULL
);

CREATE INDEX idx_memory_note_contexto ON memory_note(client_id, project, created_at);
