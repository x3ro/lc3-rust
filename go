#!/bin/bash
set -e

SCRIPT_DIR="$( cd "$( dirname "${BASH_SOURCE[0]}" )" >/dev/null && pwd )"
LC3AS_DIR="${SCRIPT_DIR}/vendor/lc3/lc3tools"
LC3AS="${LC3AS_DIR}/lc3as"

function error {
    echo -e >&2 "\033[31m${1}\033[0m";
    exit 1;
}

function notice {
    echo -e >&2 "\033[33m${1}\033[0m";
}

function ensure_env {
    git submodule update --init
    command -v make >/dev/null 2>&1 || error "Please install make (needed to build lc3tools)"
    command -v rustc >/dev/null 2>&1 || error "Please install Rust >= 1.33"
    command -v cargo >/dev/null 2>&1 || error "Please install Cargo (Rust's package manager)"
}

function ensure_lc3_tools {
    if [ ! -f "${LC3AS}" ]; then
        notice "LC3 assembler not found, trying to build"
        (cd "${LC3AS_DIR}" && ./configure && make)
    fi
}

function cmd_test {
    for asm_file in $SCRIPT_DIR/tests/*.asm; do
        lc3as "${asm_file}" 2>&1 1>/dev/null || error "Failed to assemble $(basename ${asm_file})!"
    done
    cargo test
}

function cmd_usage {
    echo "TODO usage";
}

ensure_env
ensure_lc3_tools

command=""
if (( $# > 0 )); then
    command="${1}"
fi

case "${command}" in
    test) cmd_test "$@" ;;
    *) cmd_usage
esac