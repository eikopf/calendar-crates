#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
FIXTURES_DIR="$SCRIPT_DIR/fixtures"
TEMP_DIR="$(mktemp -d)"

cleanup() { rm -rf "$TEMP_DIR"; }
trap cleanup EXIT

echo "Fetching iCalendar test fixtures..."

# --- ical4j (Java) ---
echo "  Cloning ical4j/ical4j..."
git clone --depth 1 --filter=blob:none --sparse \
    https://github.com/ical4j/ical4j.git "$TEMP_DIR/ical4j" 2>/dev/null
(cd "$TEMP_DIR/ical4j" && git sparse-checkout set src/test/resources/samples 2>/dev/null)
mkdir -p "$FIXTURES_DIR/ical4j"
find "$TEMP_DIR/ical4j/src/test/resources/samples/valid" -name '*.ics' \
    -exec cp {} "$FIXTURES_DIR/ical4j/" \;
echo "    $(ls "$FIXTURES_DIR/ical4j/" | wc -l | tr -d ' ') files (valid only)"

TOTAL=$(find "$FIXTURES_DIR" -name '*.ics' | wc -l | tr -d ' ')
echo "Done. $TOTAL total .ics files in $FIXTURES_DIR"
