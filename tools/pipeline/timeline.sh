#!/usr/bin/env bash
# Where did a pipeline session's time go? (owner 2026-11-11)
# Usage: tools/pipeline/timeline.sh .pipeline/runs/<PHASE>_implement_r1.log [N]
# Reads the "<epoch> <stream-json line>" file the runner writes next to the log (<log base>.timeline)
# and prints: total wall-clock time, time inside tool calls vs. the model, and the N slowest
# commands (default 15) with their duration.
set -euo pipefail
log="${1:?log file}"; n="${2:-15}"
tl="${log%.log}.timeline"
[[ -f "$tl" ]] || { echo "no timeline: $tl" >&2; exit 1; }

# One TSV row per completed tool call: seconds, tool, command-or-path (first 140 chars).
rows="$(while IFS= read -r line; do
  t="${line%% *}"; j="${line#* }"
  jq -rc --argjson t "$t" '
    if .type == "assistant" then
      (.message.content // [])[] | select(.type == "tool_use")
      | ["S", $t, .id, .name, ((.input.command // .input.file_path // .input.pattern // "") | tostring | gsub("[\t\n]"; " ") | .[0:140])] | @tsv
    elif .type == "user" then
      (.message.content // []) | if type == "array" then .[] else empty end | select(.type == "tool_result")
      | ["E", $t, .tool_use_id] | @tsv
    elif .type == "item.started" and .item.type == "command_execution" then
      ["S", $t, .item.id, "Bash", (.item.command | gsub("[\t\n]"; " ") | .[0:140])] | @tsv
    elif .type == "item.completed" and .item.type == "command_execution" then
      ["E", $t, .item.id] | @tsv
    else empty end' <<<"$j" 2>/dev/null || true
done < "$tl" | awk -F'\t' '
  $1 == "S" { start[$3] = $2; tool[$3] = $4; what[$3] = $5 }
  $1 == "E" && ($3 in start) { printf "%d\t%s\t%s\n", $2 - start[$3], tool[$3], what[$3]; delete start[$3] }')"

first="$(head -1 "$tl" | cut -d' ' -f1)"; last="$(tail -1 "$tl" | cut -d' ' -f1)"
total=$((last - first))
in_tools="$(awk -F'\t' '{s += $1} END {print s + 0}' <<<"$rows")"
fmt() { printf '%dh %02dm %02ds' $(($1 / 3600)) $(($1 % 3600 / 60)) $(($1 % 60)); }
echo "session:      $(basename "$log")"
echo "wall-clock:   $(fmt "$total")"
echo "in commands:  $(fmt "$in_tools")"
echo "model/other:  $(fmt $((total - in_tools)))"
echo
echo "by kind (seconds, calls):"
awk -F'\t' '{
  k = $2; c = $3
  if (k == "Bash") {
    if (c ~ /cargo (test|nextest)/) k = "cargo test"; else if (c ~ /cargo clippy/) k = "cargo clippy";
    else if (c ~ /cargo (build|check)/) k = "cargo build/check"; else if (c ~ /vitest|npm (test|run test)/) k = "UI tests";
    else if (c ~ /typecheck|tsc/) k = "typecheck"; else k = "other shell" }
  s[k] += $1; n[k]++ } END { for (k in s) printf "  %6d s  %4d  %s\n", s[k], n[k], k }' <<<"$rows" | sort -rn
echo
echo "slowest $n commands:"
sort -t$'\t' -k1,1rn <<<"$rows" | head -"$n" | awk -F'\t' '{ printf "  %6d s  %-5s %s\n", $1, $2, $3 }'
