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

# --- ical.net (C#) ---
echo "  Cloning ical-org/ical.net..."
git clone --depth 1 --filter=blob:none --sparse \
    https://github.com/ical-org/ical.net.git "$TEMP_DIR/ical.net" 2>/dev/null
(cd "$TEMP_DIR/ical.net" && git sparse-checkout set Ical.Net.Tests/Calendars 2>/dev/null)
mkdir -p "$FIXTURES_DIR/ical.net"
find "$TEMP_DIR/ical.net/Ical.Net.Tests/Calendars" -name '*.ics' \
    -exec cp {} "$FIXTURES_DIR/ical.net/" \;
echo "    $(ls "$FIXTURES_DIR/ical.net/" | wc -l | tr -d ' ') files"

# --- allenporter/ical (Python) ---
echo "  Cloning allenporter/ical..."
git clone --depth 1 --filter=blob:none --sparse \
    https://github.com/allenporter/ical.git "$TEMP_DIR/ical" 2>/dev/null
(cd "$TEMP_DIR/ical" && git sparse-checkout set tests/testdata 2>/dev/null)
mkdir -p "$FIXTURES_DIR/allenporter-ical"
find "$TEMP_DIR/ical/tests/testdata" -name '*.ics' \
    -exec cp {} "$FIXTURES_DIR/allenporter-ical/" \;
echo "    $(ls "$FIXTURES_DIR/allenporter-ical/" | wc -l | tr -d ' ') files"

TOTAL=$(find "$FIXTURES_DIR" -name '*.ics' | wc -l | tr -d ' ')
echo "Done. $TOTAL total .ics files in $FIXTURES_DIR"
