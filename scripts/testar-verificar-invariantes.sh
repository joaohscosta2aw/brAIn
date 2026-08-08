#!/usr/bin/env bash
# Meta-teste do guard (task 8.7): confirma que verificar-invariantes.sh
# passa limpo, E que ele de fato falha quando uma violação real é plantada.
#
# Um guard nunca testado contra uma violação real é teatro -- já aconteceu
# nesta change dele passar por vacuidade (nenhum código pra vigiar ainda) e,
# na direção oposta, disparar falso positivo no próprio teste que provava a
# invariante. Este script fecha os dois lados: precisa passar limpo E
# precisa falhar quando corrompido, ou o próprio meta-teste falha.

set -uo pipefail
cd "$(dirname "$0")/.."

falhas=0

verificar_limpo_passa() {
    if ./scripts/verificar-invariantes.sh >/dev/null 2>&1; then
        echo "ok: guard passa limpo (estado atual do repositório)"
    else
        echo "FALHA: guard deveria passar limpo e não passou" >&2
        falhas=$((falhas + 1))
    fi
}

verificar_planta_e_pega() {
    local descricao="$1" arquivo="$2" conteudo="$3"

    echo "$conteudo" > "$arquivo"
    git add -N "$arquivo" 2>/dev/null

    if ./scripts/verificar-invariantes.sh >/dev/null 2>&1; then
        echo "FALHA: guard não pegou violação plantada ($descricao)" >&2
        falhas=$((falhas + 1))
    else
        echo "ok: guard pega violação plantada ($descricao)"
    fi

    rm -f "$arquivo"
    git rm --cached -q "$arquivo" 2>/dev/null
}

verificar_limpo_passa

verificar_planta_e_pega \
    "D-9, SQL fora de storage/" \
    "src/vazamento_meta_teste.rs" \
    'pub fn q() -> &'"'"'static str { "SELECT id FROM usage_record" }'

verificar_planta_e_pega \
    "soma proibida de custo pago e equivalente" \
    "src/soma_proibida_meta_teste.rs" \
    'pub fn f(pago: i64, equivalente: i64) -> i64 { pago + equivalente }'

# Confirma que o repositório volta ao estado limpo depois dos plantios.
verificar_limpo_passa

echo
if [ "$falhas" -gt 0 ]; then
    echo "$falhas verificação(ões) do meta-teste falhou(aram)." >&2
    exit 1
fi
echo "Meta-teste do guard passou: limpo passa, corrompido falha."
