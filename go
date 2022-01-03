#!/bin/bash
set -e

SCRIPT_DIR="$( cd "$( dirname "${BASH_SOURCE[0]}" )" >/dev/null && pwd )"

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

function cmd_test {
    RUST_LOG="" cargo test "$@"
}

function cmd_os {
  cargo run --bin=repl -- --program testcases/assembly/os.asm --entrypoint 0x200 "$@"
}

function cmd_bench {
  cargo build --release

  export RUST_LOG=${RUST_LOG:-info}
  if [[ "$1" == "flamegraph" ]]; then
    sudo flamegraph target/release/repl --program testcases/assembly/divisible.asm --entrypoint 0x3000
  else
    target/release/repl --program testcases/assembly/divisible.asm --entrypoint 0x3000
  fi
}

function cmd_usage {
    echo "
./go [cmd]

  test              TODO
  bench             TODO
  os                Runs the 'LC3 OS' test case in interactive mode

    ";
}

ensure_env

command=""
if (( $# > 0 )); then
    command="${1}"
    shift
fi

case "${command}" in
    test) cmd_test "$@" ;;
    bench) cmd_bench "$@" ;;
    os)  cmd_os "$@" ;;
    *) cmd_usage
esac
