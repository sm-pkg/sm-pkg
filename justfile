alias s := snapshot
alias c := check

check:
    cargo clippy
    cargo audit

snapshot:
    goreleaser release --snapshot --clean
