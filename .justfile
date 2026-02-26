fetch-ics-fixtures:
    bash calico/tests/fetch-fixtures.sh

ics-corpus-test: fetch-ics-fixtures
    cargo test -p calico --test corpus -- --ignored --nocapture

test:
    cargo test --all-features
