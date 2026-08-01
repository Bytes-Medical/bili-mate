#!/usr/bin/env bash
# Weekly NICE CG98 source monitor (spec 08 guideline surveillance).
#
# NOTIFY-ONLY by design: this script detects that a source may have changed
# and exits non-zero so the calling workflow can raise an alert for clinical
# review within one working day. It never modifies production rules — a
# change flows through the full governed rule-pack update process.
#
# Checks:
#   1. The recommendations page still carries the expected "Last updated"
#      date and is not marked withdrawn.
#   2. The treatment-threshold workbook still hashes to the value recorded
#      in the rule pack's source manifest.
set -uo pipefail

RECOMMENDATIONS_URL="https://www.nice.org.uk/guidance/cg98/chapter/recommendations"
OVERVIEW_URL="https://www.nice.org.uk/guidance/cg98"
WORKBOOK_URL="https://www.nice.org.uk/guidance/cg98/resources/treatment-threshold-graphs-excel-544300525"
EXPECTED_UPDATED_TEXT="31 October 2023"
# From spec/clinical/nice-cg98-2023-10-31.1.yaml sources[cg98-threshold-workbook].
EXPECTED_WORKBOOK_SHA256="4c9e896c074b6d8b15daf18192951b5d82d3412a85a57438ba0112c76b5ec5a1"

findings=0
note() { echo "FINDING: $1"; findings=$((findings + 1)); }

fetch() {
  curl --silent --show-error --location --max-time 60 \
    --user-agent "bili-mate-guideline-monitor (notify-only; spec 08)" "$1"
}

for url in "$RECOMMENDATIONS_URL" "$OVERVIEW_URL"; do
  page="$(fetch "$url")" || { note "could not fetch $url"; continue; }
  if ! grep -qi "$EXPECTED_UPDATED_TEXT" <<<"$page"; then
    note "$url no longer shows the expected last-updated date '$EXPECTED_UPDATED_TEXT' — CG98 may have been updated"
  fi
  if grep -qiE "this guidance has been (withdrawn|replaced)|guidance is under review" <<<"$page"; then
    note "$url contains withdrawal or replacement wording"
  fi
done

workbook_sha="$(fetch "$WORKBOOK_URL" | shasum -a 256 | cut -d' ' -f1)" || workbook_sha=""
if [ -z "$workbook_sha" ]; then
  note "could not fetch the treatment-threshold workbook"
elif [ "$workbook_sha" != "$EXPECTED_WORKBOOK_SHA256" ]; then
  note "threshold workbook hash changed: $workbook_sha (expected $EXPECTED_WORKBOOK_SHA256)"
fi

if [ "$findings" -eq 0 ]; then
  echo "OK: CG98 sources match the recorded state (last updated $EXPECTED_UPDATED_TEXT)."
  exit 0
fi
echo
echo "$findings finding(s). Place in clinical review within one working day (spec 08)."
echo "This monitor is notify-only; production rules are unchanged."
exit 1
