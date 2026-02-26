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

# --- collective/icalendar (Python) ---
echo "  Cloning collective/icalendar..."
git clone --depth 1 --filter=blob:none --sparse \
    https://github.com/collective/icalendar.git "$TEMP_DIR/icalendar" 2>/dev/null
(cd "$TEMP_DIR/icalendar" && git sparse-checkout set src/icalendar/tests 2>/dev/null)
mkdir -p "$FIXTURES_DIR/icalendar"
# Copy .ics files preserving subdirectory structure, but skip vCard/jCal dirs
find "$TEMP_DIR/icalendar/src/icalendar/tests" -name '*.ics' \
    ! -path '*/rfc_6350_vcard/*' ! -path '*/rfc_7265_jcal/*' \
    -exec sh -c '
        for f; do
            rel="${f#'"$TEMP_DIR"'/icalendar/src/icalendar/tests/}"
            mkdir -p "'"$FIXTURES_DIR"'/icalendar/$(dirname "$rel")"
            cp "$f" "'"$FIXTURES_DIR"'/icalendar/$rel"
        done
    ' _ {} +
echo "    $(find "$FIXTURES_DIR/icalendar/" -name '*.ics' | wc -l | tr -d ' ') files"

TOTAL=$(find "$FIXTURES_DIR" -name '*.ics' | wc -l | tr -d ' ')
echo "Done. $TOTAL total .ics files in $FIXTURES_DIR"
