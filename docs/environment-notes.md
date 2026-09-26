# Scheduled-environment notes for Madgab

Objective facts about the environment the scheduled coordinator and its
Antonina agents run in. Recorded by a coordinator pass on 2026-09-26 so
later passes do not rediscover them, and do not report work as blocked on
commands that cannot run here at all.

## Validation commands that are unavailable

There is no `rustup`, and the installed toolchain was built from a source
tarball, so these cargo subcommands and components do not exist:

- `cargo fmt` / `cargo fmt --check` (no `rustfmt` component)
- `cargo clippy` (no `clippy` component)
- `cargo test --doc` / any doctest (no `rustdoc` binary, so
  `cargo test --no-fail-fast` always ends with
  `error: 2 targets failed: --test corpus_integration --doc`
  whenever a doc test would have run)

Consequence: the `cargo fmt --check` and `cargo clippy --all-targets
-- -D warnings` criteria in [continuation-approximate-search.md](continuation-approximate-search.md)
cannot be verified in this environment. Do not mark a work item `done`
while claiming they passed. Verify with
`cargo test --release --lib` and `cargo test --release --test <name>`
instead, and say plainly in the handoff which criteria were unverifiable
here.

A `wasm32-unknown-unknown` build is likewise not checkable, because targets
cannot be added without `rustup`.

## Git

- `commit.gpgsign` is `true` and there is no `gpg` binary, so plain
  `git commit` fails with `error: cannot run gpg`. Use
  `git -c commit.gpgsign=false commit ...`.
- `remote.origin.fetch` is narrowed to
  `refs/heads/post-milestone-acceptance:refs/remotes/origin/post-milestone-acceptance`,
  so `git fetch` alone will not see child or archive branches. Push and
  fetch them by explicit refspec, e.g.
  `git push origin <branch>:refs/heads/<branch>`.
- `git ls-remote --heads origin` lists roughly 40 `archive/*` branches;
  filter its output with node or a file, not with `grep`/`sed`/`awk`,
  which are absent from this environment.
- **Consequence of the narrowed fetch refspec: an absent remote-tracking ref
  does not mean a branch is unpushed.** Most local branches therefore *look*
  unpushed, and `git push --all` will report almost nothing. The honest
  durability check is to reconcile the two full lists:

  ```sh
  git ls-remote --heads origin > /tmp/lsr.txt
  node -e 'const fs=require("fs");
  const r=new Set(fs.readFileSync("/tmp/lsr.txt","utf8").split("\n").filter(Boolean)
    .map(l=>l.split("\t")[1].replace("refs/heads/","")));
  const cp=require("child_process");
  for(const b of cp.execSync("git for-each-ref --format=\"%(refname:short)\" refs/heads/")
    .toString().split("\n").filter(Boolean))
    if(!r.has(b)) console.log("NOT PUSHED: "+b);'
  ```

  Run this on every coordinator pass before concluding anything about
  durability, and `git push --all` afterwards if it reports anything. A
  single-copy source hazard on this repository has already happened twice
  (an agent finishing on a detached HEAD, and an unpushed implementation
  branch), and this check is what catches it.

## Shell

`grep`, `sed`, `awk` and `python3` are not on `PATH`. `node` is, and
`git grep` works. Use those for search and text processing.

## Shell portability note

Nothing above is a repository defect; it is only the shape of this host. If
the environment is later fixed, delete this file.
