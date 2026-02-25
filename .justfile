fetch-fixtures:
    bash calico/tests/fetch-fixtures.sh

corpus-test: fetch-fixtures
    cargo test -p calico --test corpus -- --ignored --nocapture

test:
    cargo test --all-features
