-- Migração 3: perfis de identidade, contexto ativo e referências de credencial
-- (context-and-identity-switching). Aditiva: nenhuma coluna de usage_record,
-- provider_plan ou quota_signal muda.

-- Um perfil por (client_id, project) -- project pode ser NULL quando o cliente
-- tem um único perfil "padrão" sem distinção de projeto. Ambiguidade de
-- `connect` sem --project é detectada contando perfis distintos do cliente.
CREATE TABLE identity_profile (
    id                TEXT PRIMARY KEY,
    client_id         TEXT NOT NULL REFERENCES client(id),
    project           TEXT,
    git_author_name   TEXT,
    git_author_email  TEXT,
    github_org        TEXT,
    created_at        INTEGER NOT NULL,
    UNIQUE (client_id, project)
);

CREATE INDEX idx_identity_profile_client ON identity_profile(client_id);

-- config_home: caminho isolado que vira variável de ambiente do provider
-- (CODEX_HOME=<config_home>, etc.) -- design.md, "Identidade Git via variável
-- de ambiente" e §6.1 do blueprint.
CREATE TABLE provider_binding (
    identity_profile_id TEXT NOT NULL REFERENCES identity_profile(id),
    provider_id          TEXT NOT NULL,
    config_home           TEXT NOT NULL,
    PRIMARY KEY (identity_profile_id, provider_id)
);

-- Singleton: no máximo uma linha, id fixo em 1 (design.md: "contexto ativo
-- persistido no SQLite, não em arquivo solto").
CREATE TABLE active_context (
    id                   INTEGER PRIMARY KEY CHECK (id = 1),
    client_id            TEXT NOT NULL,
    project               TEXT,
    identity_profile_id TEXT NOT NULL,
    connected_at         INTEGER NOT NULL
);

-- Nenhuma coluna de valor -- só referência ao item do Keychain e metadados
-- (design.md: "referência + metadados, nunca o valor").
CREATE TABLE credential_ref (
    id                TEXT PRIMARY KEY,
    label             TEXT NOT NULL,
    keychain_service  TEXT NOT NULL,
    keychain_account  TEXT NOT NULL,
    class             TEXT NOT NULL,
    created_at        INTEGER NOT NULL,
    expires_at        INTEGER,
    last_used_at      INTEGER,
    rotation_policy   TEXT,
    UNIQUE (keychain_service, keychain_account)
);
