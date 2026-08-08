-- Migração 1: esquema inicial do ledger (client-cost-attribution, v0.0).
-- Referência: BRIAN-BLUEPRINT-V1.md §60.

CREATE TABLE client (
    id TEXT PRIMARY KEY
);

CREATE TABLE provider (
    id TEXT PRIMARY KEY
);

-- Custos em micro-unidades da moeda (ver src/domain.rs::Money) — sem ponto
-- flutuante no caminho do dinheiro, nem em disco.
CREATE TABLE usage_record (
    id                       TEXT PRIMARY KEY,
    dedup_key                TEXT NOT NULL UNIQUE,
    provider_id              TEXT NOT NULL REFERENCES provider(id),
    model                    TEXT NOT NULL,

    -- Ausente (NULL) e zero são fatos distintos — nunca colapsar um no outro.
    tokens_input              INTEGER,
    tokens_cache              INTEGER,
    tokens_output             INTEGER,
    tokens_reasoning          INTEGER,

    -- Os dois custos coexistem (§42): pago é base de custo, equivalente é
    -- base de faturamento. Nenhum é derivado do outro em SQL.
    custo_pago_micros         INTEGER,
    custo_equivalente_micros  INTEGER,

    billing_mode              TEXT NOT NULL,
    usage_source               TEXT NOT NULL,
    cost_source                TEXT NOT NULL,

    client_id                 TEXT REFERENCES client(id),
    attribution_status        TEXT NOT NULL DEFAULT 'unattributed',

    occurred_at                INTEGER NOT NULL
);

CREATE INDEX idx_usage_client_time ON usage_record(client_id, occurred_at);
CREATE INDEX idx_usage_provider_time ON usage_record(provider_id, occurred_at);
CREATE INDEX idx_usage_unattributed ON usage_record(attribution_status, occurred_at)
    WHERE attribution_status = 'unattributed';

-- Versionado por vigência (design.md): uma atualização de preço fecha o
-- intervalo anterior em vez de sobrescrevê-lo, para que o equivalente de um
-- consumo passado permaneça reproduzível.
CREATE TABLE price_catalog (
    model              TEXT NOT NULL,
    preco_micros       INTEGER NOT NULL,
    vigente_desde      INTEGER NOT NULL,
    vigente_ate        INTEGER,
    PRIMARY KEY (model, vigente_desde)
);

-- Supersessão auditável (D-14): quando o custo pago ou a atribuição de um
-- registro mudam depois de gravados, o valor anterior fica aqui em vez de
-- ser sobrescrito sem rastro.
CREATE TABLE usage_record_revisao (
    id                  INTEGER PRIMARY KEY AUTOINCREMENT,
    usage_record_id     TEXT NOT NULL REFERENCES usage_record(id),
    campo               TEXT NOT NULL,
    valor_anterior      TEXT,
    revisado_em         INTEGER NOT NULL
);
