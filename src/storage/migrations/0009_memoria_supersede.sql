-- Migração 9: cadeia de substituição append-only de notas de memória
-- (continuity/memory-supersede, blueprint §36). Aditiva: nenhuma nota
-- existente é editada -- só ganha um ponteiro opcional para a nota que a
-- substituiu.

ALTER TABLE memory_note ADD COLUMN superseded_by TEXT REFERENCES memory_note(id);
