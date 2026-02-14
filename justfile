set dotenv-load := true
set export := true

alias s := snapshot
alias c := check

default_root := env('PROJECT_ROOT', '../tf2/server')

check: clippy audit machete

machete:
    cargo machete  --with-metadata

clippy:
    cargo clippy

audit:
    cargo audit

snapshot:
    goreleaser release --snapshot --clean

install project_root=default_root:
    cargo run -- install -p {{ project_root }}

config project_root=default_root:
    cargo run -- install -p {{ project_root }}
