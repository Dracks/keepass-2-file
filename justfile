test:
    cargo test
coverage:
    cargo llvm-cov
coverage-html:
    cargo llvm-cov --open
clippy:
    cargo clippy -- -D warnings
