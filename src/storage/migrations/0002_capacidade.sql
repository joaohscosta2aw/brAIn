-- Migração 2: plano e sinal de quota por provider (capacity-windows-and-plans).
-- Aditiva: nenhuma coluna de usage_record muda (design.md).

-- Vigência própria, sem tabela de binding separada (design.md: só há um
-- plano vigente por provider por vez nesta versão).
-- verificado_em: instante da última consulta bem-sucedida à fonte, mesmo
-- quando o plano não mudou. Distinto de ativo_desde (que só avança quando o
-- plano relatado muda) -- sem isso, "consulta de plano falha" (spec
-- plan-catalog) não tinha como sinalizar informação potencialmente
-- desatualizada, só "mantém o último valor" sem dizer há quanto tempo.
CREATE TABLE provider_plan (
    provider_id   TEXT NOT NULL,
    billing_mode  TEXT NOT NULL,
    plan_label    TEXT,
    ativo_desde   INTEGER NOT NULL,
    ativo_ate     INTEGER,
    verificado_em INTEGER NOT NULL,
    PRIMARY KEY (provider_id, ativo_desde)
);

CREATE INDEX idx_provider_plan_vigente ON provider_plan(provider_id, ativo_desde DESC);

-- Percentual restante e reset não são consumo -- estado de cota à parte
-- (design.md). Upsert por (provider_id, bucket_id): sempre o sinal mais
-- recente, sem histórico de cada consulta.
CREATE TABLE quota_signal (
    provider_id       TEXT NOT NULL,
    bucket_id         TEXT NOT NULL,
    grupo             TEXT NOT NULL,
    remaining_percent REAL NOT NULL,
    reset_at          INTEGER,
    observed_at       INTEGER NOT NULL,
    PRIMARY KEY (provider_id, bucket_id)
);
