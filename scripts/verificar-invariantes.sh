#!/usr/bin/env bash
# Invariantes do BrIAn que podem ser verificadas por ferramenta em vez de prosa.
#
# O projeto decidiu em PREMISSAS-BASICAS.md §22 preferir regra executável a
# documentação. Este script existe para cumprir isso: cada verificação aqui
# substitui uma regra que antes dependia de o agente lembrar de lê-la.
#
# Passa trivialmente enquanto não houver código. Isso é esperado — o valor
# aparece quando o primeiro `.rs` existir, sem precisar lembrar de ligá-lo.

set -euo pipefail
cd "$(dirname "$0")/.."

falhas=0

erro() {
    echo "FALHA: $1" >&2
    falhas=$((falhas + 1))
}

ok() {
    echo "ok: $1"
}

# --- D-9: SQL do LEDGER DO BRIAN vive apenas em storage/ ------------------
# "SQL apenas em storage/, atrás de traits" — docs/DECISIONS.md
#
# D-9 protege a trocabilidade do backend de persistência DO BRIAN (D-1), não
# proíbe SQL em qualquer lugar do repositório. Adapters legitimamente parseiam
# formatos externos de terceiros (D-4) — para a maioria isso é JSON; para o
# Copilot, o próprio provider grava o histórico em SQLite. Ler o banco de
# outra ferramenta é a mesma categoria de "parsear formato externo" que ler
# um JSONL, não uma violação de D-9. src/adapters/ fica de fora do check.
verificar_d9() {
    local fora
    fora=$(git ls-files '*.rs' \
        | grep -v '^src/storage/' \
        | grep -v '^src/adapters/' \
        | xargs -r grep -lEi '\b(SELECT|INSERT INTO|UPDATE .* SET|DELETE FROM|CREATE TABLE)\b' \
        2>/dev/null || true)

    if [ -n "$fora" ]; then
        erro "D-9 violado — SQL fora de src/storage/:"
        echo "$fora" | sed 's/^/       /' >&2
    else
        ok "D-9: nenhum SQL fora de src/storage/"
    fi
}

# --- Custo equivalente nunca somado ao custo pago ------------------------
# BRIAN-BLUEPRINT-V1.md §42.2: "Custo equivalente em API != fatura real".
# Confundi-los é erro de dinheiro. Procuramos soma direta entre os dois campos.
verificar_custos_nao_somados() {
    local suspeitos
    suspeitos=$(git ls-files '*.rs' \
        | xargs -r grep -nE '(paid|pago).*\+.*(equiv|equivalent)|(equiv|equivalent).*\+.*(paid|pago)' \
        2>/dev/null || true)

    if [ -n "$suspeitos" ]; then
        erro "custo pago somado a custo equivalente — proibido por §42.2:"
        echo "$suspeitos" | sed 's/^/       /' >&2
    else
        ok "custos pago e equivalente não são somados"
    fi
}

# --- Especificação válida -------------------------------------------------
verificar_openspec() {
    if ! command -v openspec >/dev/null 2>&1; then
        erro "openspec não encontrado — requisito declarado em docs/harness/ambiente.md"
        return
    fi

    local changes
    changes=$(openspec list --json 2>/dev/null | grep -oE '"name"[[:space:]]*:[[:space:]]*"[^"]+"' | cut -d'"' -f4 || true)

    if [ -z "$changes" ]; then
        ok "openspec: nenhuma change ativa a validar"
        return
    fi

    local c
    for c in $changes; do
        if openspec validate "$c" --strict >/dev/null 2>&1; then
            ok "openspec: change '$c' válida"
        else
            erro "openspec: change '$c' inválida em modo estrito"
        fi
    done
}

verificar_d9
verificar_custos_nao_somados
verificar_openspec

echo
if [ "$falhas" -gt 0 ]; then
    echo "$falhas invariante(s) violada(s)." >&2
    exit 1
fi
echo "Todas as invariantes verificáveis passaram."
