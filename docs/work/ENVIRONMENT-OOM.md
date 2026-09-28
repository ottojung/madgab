# Host OOM: the recurring SIGKILL on this queue, measured

Recorded 2026-09-28T04:20Z by coordination pass `coord-c4e1`. This exists because three
separate fronts on this queue have now died to SIGKILL, each rediscovering the cause from
scratch, and at least one nearly reported it as a priced negative.

## The measurement

Read directly off the host, no inference:

| quantity | value | source |
|---|---|---|
| cgroup memory limit | **30 GiB** (32212254720) | `/sys/fs/cgroup/memory.max` |
| host RAM total / used / available | 62 GiB / 49 GiB / **12 GiB** | `free -g` |
| swap | **0** | `free -g` |
| `oom` events | **4345** | `/sys/fs/cgroup/memory.events` |
| `oom_kill` events | **410** | `/sys/fs/cgroup/memory.events` |
| CPUs | 32 | `nproc` |

The host reports 62 GiB but the cgroup caps this workspace at 30 GiB, and **49 GiB is already
in use with only 12 GiB available and no swap**. Any build that peaks above the remaining
headroom is killed. `available`, not `total`, is the number that matters.

## Why it kills agents rather than builds

The shipped release profile is `-C lto=thin -C codegen-units=1` over a binary that embeds a
15 MB transcription table. A release link of that binary is the largest single allocation on
the box. With two fronts live, the second link crosses the limit and the kernel picks a
process — which has been `rustc` at link time on some runs and the **running test binary
itself** on others.

`agent-3a8f01` was killed twice on the same item:

* ~04:11Z — `rustc` SIGKILLed twice **while linking** a release example.
* ~04:19Z — after being resumed, `cargo test --release --lib` ran many tests green, then the
  test **binary** was killed: `madgab-e6de59af7a4e0bec (signal: 9, SIGKILL)`.

`agent-3a8f02` died in the same window. The second death is the important one: it shows the
kill can land on an *already-built* binary during execution, so "the build succeeded" is not
evidence the run will survive.

## Rules

1. **A SIGKILL is an environment fact. Never report it as a priced negative, a refutation, a
   negative result, or "no effect observed".** It means the measurement did not happen. Say
   the run was killed and name the cgroup numbers.
2. **Do not link a release binary or a release example.** Prefer a `#[cfg(test)]` test in
   `src/lib.rs` over a separate example binary — it reuses the library build and avoids the
   link peak entirely.
3. When a build is needed: `CARGO_PROFILE_RELEASE_LTO=false CARGO_PROFILE_RELEASE_CODEGEN_UNITS=16
   CARGO_PROFILE_RELEASE_DEBUG=false`.
4. **Reuse the worktree's existing warm `CARGO_TARGET_DIR`.** Do not create a new target dir
   per attempt; a cold target dir means a full rebuild, which is exactly the expensive step
   that gets killed.
5. `corpus_integration` needs `-- --test-threads=2` (already known, see OBSTRUCTION-MAP §4).
   Add `--test-threads=1` for any long `--lib` run.
6. **Serialize heavy builds across concurrent fronts.** Two live fronts each doing a release
   link on a 30 GiB cgroup with 12 GiB free is the configuration that produced 410 `oom_kill`
   events. Parallel *reasoning* is free; parallel *linking* is not.
7. Prefer a debug-profile `cargo test --lib` for shape checks when the release profile is not
   affordable, and **state in the report which profile produced each number**.

## What is still unverified

Whether the `--lib` release test suite completes at all on this host under rule 3 is **not
known**. No front has yet completed a full `cargo test --release --lib` green under the
memory-bounded profile. Until one does, treat a full release `--lib` run as aspirational on
this host and prefer per-test evidence.
