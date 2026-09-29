#!/usr/bin/env bash
# agents.sh -- the MadGab agent census, as a RUNNING instrument.
#
# WHY THIS FILE EXISTS. at-risk.sh and content-sweep.sh were written for the same
# reason: a standing figure that every pass re-derives by hand is a figure, not a
# measurement, and 291 passes have shown the hand procedure fails in a fail-open
# direction often enough to be worth scripting. The agent census is the one
# standing fact still derived by hand, and pass 292 caught it failing live:
#
#   `antonina agent list --json` emits a field named STATE. A filter written
#   against `r.status` reads `undefined` for EVERY row, so
#
#       rows.filter(r => !["succeeded","failed","stopped"].includes(r.status))
#
#   is true for all 131 MadGab rows. Pass 292 published a candidate reading of
#   "131 non-terminal MadGab agents" -- an alarming false positive in the exact
#   direction that would make a pass launch agents onto a PAUSED repository.
#   Compare the true reading, 0.
#
# The shape is rules 14/14b/14g/182 and the 14aa family: a plausible number, a
# clean exit, and no abort, produced by an instrument that never checked it was
# looking at the right field. It is also exactly what this log's own rule 14r
# warns about from the other side -- a filter whose subject is undefined is not a
# filter that found nothing wrong.
#
# So this script REFUSES to report a number whose subject field it has not
# verified, in the same spirit as at-risk.sh refusing an unvalidated census.
#
# USAGE
#   docs/work/paused-recon/agents.sh          # report
#
# Exit status: 0 on a reported number, non-zero on instrument failure.
set -uo pipefail

JQ_KEYS_PROBE=state

json=$(antonina agent list --json 2>/dev/null) || {
  echo "agents.sh: 'antonina agent list --json' failed -- refusing to report" >&2
  exit 3
}

printf '%s' "$json" | node -e '
let s = "";
process.stdin.on("data", d => s += d).on("end", () => {
  let rows;
  try { const a = JSON.parse(s); rows = Array.isArray(a) ? a : (a.agents || []); }
  catch (e) {
    console.error("agents.sh: agent list --json is not parseable JSON -- refusing to report");
    process.exit(3);
  }
  if (!rows.length) {
    console.error("agents.sh: agent list returned ZERO rows -- a real host always has rows, so this is instrument failure, not an empty board");
    process.exit(3);
  }

  // REFUSAL 1: the state field must EXIST on the observed rows. This is the
  // check pass 292 lacked. `state` is the observed name; if the host ever
  // renames it, this aborts rather than silently reading undefined.
  const FIELD = "'"$JQ_KEYS_PROBE"'";
  const have = rows.filter(r => Object.prototype.hasOwnProperty.call(r, FIELD)).length;
  if (have !== rows.length) {
    console.error("agents.sh: " + (rows.length - have) + " of " + rows.length +
      " agent rows have no \"" + FIELD + "\" field -- refusing to report rather than counting undefined as non-terminal");
    console.error("agents.sh: observed keys: " + Object.keys(rows[0]).join(","));
    process.exit(4);
  }

  // REFUSAL 2: a madgab row must be recognised by CWD. Measured against this
  // repo, which has 131 such rows; a population of 0 is instrument failure.
  const MAD = /^\/workspace\/(madgab|c1d3a7|floor-|m9f1c05|probe-|ref-)/;
  const mad = rows.filter(r => MAD.test(String(r.cwd || "")));
  if (!mad.length) {
    console.error("agents.sh: 0 rows match the MadGab cwd pattern -- refusing to report (the log records 131)");
    process.exit(5);
  }

  const TERMINAL = ["succeeded", "failed", "stopped"];
  const nt = mad.filter(r => !TERMINAL.includes(r[FIELD]));
  const hist = {};
  mad.forEach(r => { hist[r[FIELD]] = (hist[r[FIELD]] || 0) + 1; });

  console.log("agents: host rows=" + rows.length + "  madgab cwd rows=" + mad.length);
  console.log("  madgab state  " + JSON.stringify(hist));
  console.log("  non-terminal madgab agents = " + nt.length +
    (nt.length ? "  " + JSON.stringify(nt.map(r => [r.id, r[FIELD], r.cwd])) : ""));

  // Other-repository agents are reported for visibility and NEVER touched.
  const running = rows.filter(r => r[FIELD] === "running");
  console.log("  host running (other repos, left running untouched) = " + running.length +
    "  " + JSON.stringify(running.map(r => [r.id, r.cwd])));
  const idle = rows.filter(r => r[FIELD] === "idle");
  console.log("  host idle = " + idle.length +
    "  " + JSON.stringify(idle.map(r => [r.id, r.cwd])));
});
'