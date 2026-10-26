#!/usr/bin/env bash
# Session Manager pipeline runner — the mechanical bridge (docs/process/flow.md §3).
# Usage:
#   tools/pipeline/run_phase.sh inventory
#   tools/pipeline/run_phase.sh plan
#   tools/pipeline/run_phase.sh phase <PHASE_ID> [--pause]
#   tools/pipeline/run_phase.sh all [--pause]
#   tools/pipeline/run_phase.sh archive <PHASE_ID>
# Exit codes: 0 approved/done · 10 orchestrator must look · 20 runner error
# Env: DRY_RUN=1 prints the commands instead of running sessions.
#      PSM_IMPL_DIR overrides docs/implementation (used by the dry run).
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
IMPL="${PSM_IMPL_DIR:-$ROOT/docs/implementation}"
RUNS="$ROOT/.pipeline/runs"
TPL="$ROOT/tools/pipeline/templates"
WRAP="$ROOT/tools/pipeline/wrappers/codex_implementer.md"
CWRAP="$ROOT/tools/pipeline/wrappers/claude_implementer.md"
# Implementer/fix runner: codex (default; owner 2026-10-26) or claude (PSM_IMPLEMENTER=claude: Opus for gpt-6-sol, Sonnet for gpt-6-luna)
IMPLEMENTER="${PSM_IMPLEMENTER:-codex}"
DATE="$(date +%F)"
DRY_RUN="${DRY_RUN:-0}"
CLAUDE_TOOLS="Bash Read Edit Write Grep Glob"
mkdir -p "$RUNS" "$IMPL/prompts/implementer" "$IMPL/prompts/reviewer" \
         "$IMPL/handoffs/implementer" "$IMPL/handoffs/reviewer" "$IMPL/narratives" "$IMPL/state"

log()  { printf '[%s] %s\n' "$(date +%H:%M:%S)" "$*" | tee -a "$RUNS/runner.log" >&2; }
die()  { log "ERROR: $*"; exit 20; }

# render TEMPLATE OUT KEY=VALUE...  → fills {{KEY}} placeholders
render() {
  local tpl="$1" out="$2"; shift 2
  python3 - "$tpl" "$out" "$@" <<'PY'
import sys, re
tpl, out, *kv = sys.argv[1:]
vals = dict(p.split('=', 1) for p in kv)
text = open(tpl).read()
missing = sorted(set(re.findall(r'{{(\w+)}}', text)) - set(vals))
if missing:
    sys.exit(f"template {tpl} missing values for: {missing}")
for k, v in vals.items():
    text = text.replace('{{' + k + '}}', v)
open(out, 'w').write(text)
PY
}

# frontmatter FILE KEY → value or empty
frontmatter() {
  python3 - "$1" "$2" <<'PY'
import sys, re
path, key = sys.argv[1:]
try: text = open(path).read()
except FileNotFoundError: sys.exit(0)
m = re.match(r'^---\n(.*?)\n---', text, re.S)
if not m: sys.exit(0)
for line in m.group(1).splitlines():
    if ':' in line:
        k, v = line.split(':', 1)
        if k.strip() == key: print(v.strip().strip('"').strip("'")); break
PY
}

# chatfield LASTMSG KEY → value from the first "HANDOFF: … | STATE: … | …" line
chatfield() {
  python3 - "$1" "$2" <<'PY'
import sys, re
path, key = sys.argv[1:]
try: text = open(path).read()
except FileNotFoundError: sys.exit(0)
for line in text.splitlines():
    if line.strip().startswith('HANDOFF:'):
        for seg in line.split('|'):
            if ':' in seg:
                k, v = seg.split(':', 1)
                if k.strip() == key: print(v.strip()); break
        break
PY
}

commit_artifacts() {  # commit_artifacts "message"
  if [[ "$DRY_RUN" == 1 ]]; then log "DRY commit: $1"; return; fi
  git -C "$ROOT" add -A -- "$IMPL" >/dev/null 2>&1 || true
  if ! git -C "$ROOT" diff --cached --quiet; then
    git -C "$ROOT" commit -q -m "$1" -m "Co-Authored-By: pipeline runner <noreply@local>" && log "committed: $1"
  fi
}

run_codex() {  # run_codex MODEL EFFORT PROMPT LAST LOG
  local model="$1" effort="$2" prompt="$3" last="$4" logf="$5"
  log "codex $model/$effort ← $(basename "$prompt")"
  if [[ "$DRY_RUN" == 1 ]]; then
    echo "DRY: cat $WRAP $prompt | codex exec -C $ROOT -s workspace-write --add-dir $ROOT/.git --add-dir $HOME/.cargo --add-dir $HOME/.npm -m $model -c model_reasoning_effort=\"$effort\" -o $last --json -"
    printf 'HANDOFF: %s | STATE: DONE | OWNER_CARDS: 0\n' "$DRY_HANDOFF" > "$last"; return 0
  fi
  set +e
  cat "$WRAP" "$prompt" | codex exec -C "$ROOT" -s workspace-write --add-dir "$ROOT/.git" \
      --add-dir "$HOME/.cargo" --add-dir "$HOME/.npm" \
      -m "$model" -c model_reasoning_effort="\"$effort\"" -o "$last" --json - > "$logf" 2>&1
  local rc=$?; set -e
  log "codex exited $rc"
  # owner 2026-10-26: when gpt-6-sol is at capacity, use gpt-5.6-terra on high
  if [[ $rc -ne 0 && "$model" == gpt-6-sol ]] && grep -q 'at capacity' "$logf"; then
    log "gpt-6-sol at capacity → gpt-5.6-terra/high"
    run_codex gpt-5.6-terra high "$prompt" "$last" "$logf"; return $?
  fi
  return $rc
}

run_claude() {  # run_claude AGENT MODEL PROMPT LAST LOG [EFFORT]
  local agent="$1" model="$2" prompt="$3" last="$4" logf="$5" effort="${6:-}"
  local eff=(); [[ -n "$effort" ]] && eff=(--effort "$effort")
  log "claude --agent $agent --model $model${effort:+ --effort $effort} ← $(basename "$prompt")"
  if [[ "$DRY_RUN" == 1 ]]; then
    echo "DRY: claude -p --agent $agent --model $model ${eff[@]+"${eff[@]}"} --allowedTools \"$CLAUDE_TOOLS\" --permission-mode acceptEdits --no-session-persistence --output-format json < $prompt"
    printf 'HANDOFF: %s | STATE: DONE | VERDICT: %s | OWNER_CARDS: 0\n' "$DRY_HANDOFF" "${DRY_VERDICT:-APPROVED}" > "$last"; return 0
  fi
  set +e
  ( cd "$ROOT" && claude -p --agent "$agent" --model "$model" ${eff[@]+"${eff[@]}"} --allowedTools $CLAUDE_TOOLS \
      --permission-mode acceptEdits --no-session-persistence --output-format json < "$prompt" > "$logf" 2>&1 )
  local rc=$?; set -e
  python3 - "$logf" "$last" <<'PY'
import sys, json
logf, last = sys.argv[1:]
text = open(logf).read().strip()
result = ''
try:
    data = json.loads(text.splitlines()[-1]) if text else {}
    result = data.get('result') or ''
except Exception:
    result = text
open(last, 'w').write(result)
PY
  log "claude exited $rc"
  return $rc
}

run_impl() {  # run_impl CODEX_MODEL EFFORT PROMPT LAST LOG → codex, or claude pipeline-implementer
  local model="$1" effort="$2" prompt="$3" last="$4" logf="$5"
  if [[ "$IMPLEMENTER" == codex ]]; then run_codex "$@"; return $?; fi
  local cm=opus; [[ "$model" == gpt-6-luna ]] && cm=sonnet
  local full="$RUNS/$(basename "${prompt%.md}").full.md"
  cat "$CWRAP" "$prompt" > "$full"
  run_claude pipeline-implementer "$cm" "$full" "$last" "$logf" "$effort"
}

# session_state LAST HANDOFF → prints STATE, preferring the handoff frontmatter
session_state() {
  local st; st="$(frontmatter "$2" state)"; [[ -z "$st" ]] && st="$(chatfield "$1" STATE)"; echo "${st:-MISSING}"
}
session_verdict() {
  local v; v="$(frontmatter "$2" verdict)"; [[ -z "$v" ]] && v="$(chatfield "$1" VERDICT)"; echo "${v:-MISSING}"
}

set_state() { echo "{\"phase\":\"$1\",\"state\":\"$2\",\"date\":\"$DATE\",\"note\":\"$3\"}" > "$IMPL/state/$1.json"; }

stop_for_orchestrator() {  # phase reason
  set_state "$1" NEEDS_ORCHESTRATOR "$2"; commit_artifacts "pipeline: $1 needs orchestrator ($2)"
  log "STOP ($1): $2"; exit 10
}

run_phase() {
  local phase="$1" plan="$IMPL/plans/$1.md"
  [[ -f "$plan" ]] || die "plan file not found: $plan"
  local title difficulty projection effort
  title="$(frontmatter "$plan" title)"; difficulty="$(frontmatter "$plan" difficulty)"; projection="$(frontmatter "$plan" projection)"
  effort=medium; [[ "$difficulty" == high ]] && effort=high
  log "=== phase $phase: $title (difficulty=${difficulty:-normal}, projection=${projection:-no})"
  local H="$IMPL/handoffs" P="$IMPL/prompts" narrative="$IMPL/narratives/$phase.md"

  # 0. projection (opt-in)
  if [[ "$projection" == yes ]]; then
    local pp="$P/reviewer/${phase}_projection_r0.md" ph="$H/reviewer/handoff_${phase}_projection_r0.md"
    DRY_HANDOFF="$ph" DRY_VERDICT=PROJECTED_CLEAN
    render "$TPL/projection.md" "$pp" PHASE="$phase" DATE="$DATE" ROOT="$ROOT" PLAN_FILE="$plan" HANDOFF="$ph"
    commit_artifacts "pipeline: $phase projection prompt"
    run_claude pipeline-projectionist opus "$pp" "$RUNS/${phase}_projection_r0.last.md" "$RUNS/${phase}_projection_r0.log" || true
    commit_artifacts "pipeline: $phase projection handoff"
    [[ "$(session_verdict "$RUNS/${phase}_projection_r0.last.md" "$ph")" == PROJECTED_CLEAN ]] || stop_for_orchestrator "$phase" "projection requires amendments"
  fi

  # 1. implement
  local ip="$P/implementer/${phase}_implement_r1.md" ih="$H/implementer/handoff_${phase}_implement_r1.md"
  DRY_HANDOFF="$ih"
  render "$TPL/implement.md" "$ip" PHASE="$phase" ROUND=1 DATE="$DATE" ROOT="$ROOT" PLAN_FILE="$plan" HANDOFF="$ih"
  commit_artifacts "pipeline: $phase implement prompt"
  run_impl gpt-6-sol "$effort" "$ip" "$RUNS/${phase}_implement_r1.last.md" "$RUNS/${phase}_implement_r1.log" || true
  commit_artifacts "pipeline: $phase implement handoff"
  local st; st="$(session_state "$RUNS/${phase}_implement_r1.last.md" "$ih")"
  [[ "$st" == DONE ]] || stop_for_orchestrator "$phase" "implementer state $st"

  # 2. review
  local rp="$P/reviewer/${phase}_review_r1.md" rh="$H/reviewer/handoff_${phase}_review_r1.md"
  DRY_HANDOFF="$rh"; DRY_VERDICT="${DRY_VERDICT_R1:-APPROVED}"
  render "$TPL/review.md" "$rp" PHASE="$phase" ROUND=1 DATE="$DATE" ROOT="$ROOT" PLAN_FILE="$plan" IMPL_HANDOFF="$ih" HANDOFF="$rh" NARRATIVE="$narrative"
  commit_artifacts "pipeline: $phase review prompt"
  run_claude pipeline-reviewer opus "$rp" "$RUNS/${phase}_review_r1.last.md" "$RUNS/${phase}_review_r1.log" || true
  commit_artifacts "pipeline: $phase review r1"
  local v; v="$(session_verdict "$RUNS/${phase}_review_r1.last.md" "$rh")"
  st="$(session_state "$RUNS/${phase}_review_r1.last.md" "$rh")"
  [[ "$st" == DONE || "$st" == OWNER_DECISIONS_PENDING ]] || stop_for_orchestrator "$phase" "reviewer state $st"
  case "$v" in
    APPROVED|APPROVED_WITH_NOTES) approve "$phase" "$title" "$v" "$rh"; return 0 ;;
    CHANGES_REQUESTED) ;;
    *) stop_for_orchestrator "$phase" "review verdict $v" ;;
  esac

  # 3. fix 1
  local fp="$P/implementer/${phase}_fix_r2.md" fh="$H/implementer/handoff_${phase}_fix_r2.md"
  DRY_HANDOFF="$fh"
  render "$TPL/fix.md" "$fp" PHASE="$phase" ROUND=2 FIXN=1 DATE="$DATE" ROOT="$ROOT" PLAN_FILE="$plan" REVIEW_HANDOFF="$rh" IMPL_HANDOFF="$ih" HANDOFF="$fh"
  commit_artifacts "pipeline: $phase fix prompt"
  run_impl gpt-6-luna high "$fp" "$RUNS/${phase}_fix_r2.last.md" "$RUNS/${phase}_fix_r2.log" || true
  commit_artifacts "pipeline: $phase fix handoff"
  st="$(session_state "$RUNS/${phase}_fix_r2.last.md" "$fh")"
  [[ "$st" == DONE ]] || stop_for_orchestrator "$phase" "fix state $st"

  # 4. re-review
  local rp2="$P/reviewer/${phase}_review_r2.md" rh2="$H/reviewer/handoff_${phase}_review_r2.md"
  DRY_HANDOFF="$rh2"; DRY_VERDICT="${DRY_VERDICT_R2:-APPROVED_WITH_NOTES}"
  render "$TPL/rereview.md" "$rp2" PHASE="$phase" ROUND=2 FIXN=1 DATE="$DATE" ROOT="$ROOT" PLAN_FILE="$plan" PREV_REVIEW="$rh" FIX_HANDOFF="$fh" HANDOFF="$rh2" NARRATIVE="$narrative"
  commit_artifacts "pipeline: $phase re-review prompt"
  run_claude pipeline-reviewer opus "$rp2" "$RUNS/${phase}_review_r2.last.md" "$RUNS/${phase}_review_r2.log" || true
  commit_artifacts "pipeline: $phase review r2"
  v="$(session_verdict "$RUNS/${phase}_review_r2.last.md" "$rh2")"
  case "$v" in
    APPROVED|APPROVED_WITH_NOTES) approve "$phase" "$title" "$v" "$rh2"; return 0 ;;
  esac
  # 5. second fix only when the orchestrator authorised it in the plan's Review log
  grep -q 'FIX2_AUTHORISED:' "$plan" || stop_for_orchestrator "$phase" "re-review verdict $v (MUST items remain; no FIX2_AUTHORISED in plan)"
  local fp3="$P/implementer/${phase}_fix_r3.md" fh3="$H/implementer/handoff_${phase}_fix_r3.md"
  DRY_HANDOFF="$fh3"
  render "$TPL/fix.md" "$fp3" PHASE="$phase" ROUND=3 FIXN=2 DATE="$DATE" ROOT="$ROOT" PLAN_FILE="$plan" REVIEW_HANDOFF="$rh2" IMPL_HANDOFF="$ih" HANDOFF="$fh3"
  commit_artifacts "pipeline: $phase fix 2 prompt"
  run_impl gpt-6-luna high "$fp3" "$RUNS/${phase}_fix_r3.last.md" "$RUNS/${phase}_fix_r3.log" || true
  commit_artifacts "pipeline: $phase fix 2 handoff"
  st="$(session_state "$RUNS/${phase}_fix_r3.last.md" "$fh3")"; [[ "$st" == DONE ]] || stop_for_orchestrator "$phase" "fix 2 state $st"
  local rp3="$P/reviewer/${phase}_review_r3.md" rh3="$H/reviewer/handoff_${phase}_review_r3.md"
  DRY_HANDOFF="$rh3"; DRY_VERDICT="${DRY_VERDICT_R3:-APPROVED_WITH_NOTES}"
  render "$TPL/rereview.md" "$rp3" PHASE="$phase" ROUND=3 FIXN=2 DATE="$DATE" ROOT="$ROOT" PLAN_FILE="$plan" PREV_REVIEW="$rh2" FIX_HANDOFF="$fh3" HANDOFF="$rh3" NARRATIVE="$narrative"
  commit_artifacts "pipeline: $phase re-review 2 prompt"
  run_claude pipeline-reviewer opus "$rp3" "$RUNS/${phase}_review_r3.last.md" "$RUNS/${phase}_review_r3.log" || true
  commit_artifacts "pipeline: $phase review r3"
  v="$(session_verdict "$RUNS/${phase}_review_r3.last.md" "$rh3")"
  case "$v" in
    APPROVED|APPROVED_WITH_NOTES) approve "$phase" "$title" "$v" "$rh3" ;;
    *) stop_for_orchestrator "$phase" "re-review 2 verdict $v; no further fixes (flow.md §5)" ;;
  esac
}

approve() {  # phase title verdict handoff
  local cards; cards="$(chatfield "$RUNS/${1}_review_r1.last.md" OWNER_CARDS)"
  for r in r2 r3; do [[ -f "$RUNS/${1}_review_$r.last.md" ]] && cards="$(chatfield "$RUNS/${1}_review_$r.last.md" OWNER_CARDS)"; done
  set_state "$1" APPROVED "$3; owner cards: ${cards:-0}"
  if [[ "$DRY_RUN" == 1 ]]; then log "DRY approval commit: APPROVED $1: $2"; else
    git -C "$ROOT" add -A -- "$IMPL" >/dev/null 2>&1 || true
    git -C "$ROOT" commit -q --allow-empty -m "APPROVED $1: $2 ($3)" -m "Co-Authored-By: pipeline runner <noreply@local>"
  fi
  log "APPROVED $1 ($3). Narrative: $IMPL/narratives/$1.md"
}

cmd="${1:-}"; shift || true
case "$cmd" in
  inventory)
    h="$IMPL/handoffs/reviewer/handoff_intention_inventory_r1.md"; p="$IMPL/prompts/reviewer/intention_inventory_r1.md"; DRY_HANDOFF="$h"
    render "$TPL/inventory.md" "$p" DATE="$DATE" ROOT="$ROOT" HANDOFF="$h"; commit_artifacts "pipeline: inventory prompt"
    run_claude pipeline-inventory opus "$p" "$RUNS/inventory_r1.last.md" "$RUNS/inventory_r1.log" || true
    git -C "$ROOT" add -A -- "$ROOT/docs/intention" >/dev/null 2>&1 || true; commit_artifacts "pipeline: inventory handoff and intention delta"
    st="$(session_state "$RUNS/inventory_r1.last.md" "$h")"; log "inventory state: $st"; [[ "$st" == DONE ]] || exit 10 ;;
  plan)
    h="$IMPL/handoffs/reviewer/handoff_master_plan_r1.md"; p="$IMPL/prompts/reviewer/master_plan_r1.md"; DRY_HANDOFF="$h"
    render "$TPL/plan.md" "$p" DATE="$DATE" ROOT="$ROOT" HANDOFF="$h"; commit_artifacts "pipeline: planning prompt"
    run_claude pipeline-planner fable "$p" "$RUNS/plan_r1.last.md" "$RUNS/plan_r1.log" || true
    commit_artifacts "pipeline: master plan and phase plans"
    st="$(session_state "$RUNS/plan_r1.last.md" "$h")"; log "plan state: $st"; [[ "$st" == DONE ]] || exit 10 ;;
  phase)
    [[ -n "${1:-}" ]] || die "phase id required"; run_phase "$1" ;;
  archive)
    [[ -n "${1:-}" ]] || die "phase id required"; ph="$1"; dest="$IMPL/archive/${ph}_$(date +%Y%m%d-%H%M%S)"; mkdir -p "$dest"
    for f in "$IMPL"/prompts/*/"${ph}"_*.md "$IMPL"/handoffs/*/handoff_"${ph}"_*.md "$IMPL/state/$ph.json" "$IMPL/narratives/$ph.md"; do
      [[ -e "$f" ]] && mv "$f" "$dest/"; done
    commit_artifacts "pipeline: archive attempt of $ph"; log "archived $ph → $dest" ;;
  all)
    [[ -f "$IMPL/phases.txt" ]] || die "phases.txt not found"
    pause=0; [[ "${1:-}" == --pause ]] && pause=1
    while read -r ph; do
      [[ -z "$ph" || "$ph" == \#* ]] && continue
      if [[ -f "$IMPL/state/$ph.json" ]] && grep -q '"state":"APPROVED"' "$IMPL/state/$ph.json"; then log "skip $ph (approved)"; continue; fi
      run_phase "$ph"
      [[ $pause == 1 ]] && { log "paused after $ph"; exit 0; }
    done < "$IMPL/phases.txt"
    exit 0 ;;  # every phase approved (the loop's last test would otherwise leak status 1)
  *) sed -n 2,10p "$0"; exit 20 ;;
esac
