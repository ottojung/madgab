# Durable host resources

Use this skill when scheduled or agentic work creates, uses, or hands off a durable host filesystem location. A resource is a host and canonical absolute POSIX path, such as `lubko://marceline-dev` plus `/workspace/project-worktree`; the path may be a file or directory.

Resource registration is dependency and liveness metadata, not exclusive ownership or a lock. Multiple open Antonina BOARD issues may depend on one resource. Every number in these commands is an **Antonina BOARD issue number**, never a GitHub issue number and never a repository work-item ID such as `w-a1b2c3`.

Repository work items own the durable task/handoff record. Antonina board issues are only an internal mechanism for protecting host resources when that board is available. If both are used, record useful board/resource references in the repository work item's handoff notes.

## Garbage collection semantics

A deterministic garbage collector may remove unprotected host paths. When the Antonina board is available, protect durable paths with open board-issue dependencies rather than relying on timing.

## Commands

```sh
antonina board resource list [--host HOST] [--issue NUMBER]
antonina board resource add ISSUE HOST PATH
antonina board resource remove ISSUE HOST PATH
```

`--host` and `--issue` narrow inspection; verify registrations rather than assuming a command succeeded.

## Safe handoff

1. Inspect the existing board issue's resources when one exists.
2. Add the follow-up open board issue dependency first.
3. Verify the resource list output includes the follow-up dependency and shows it protected.
4. Only then close the old board issue or remove the old dependency.
5. Independently update and push the repository work item with the canonical path, branch/commit state, and next action.

Host values are `lubko://<server>` with no trailing slash; paths are canonical absolute POSIX paths.

If the current host does not have Antonina board credentials, do not fabricate them. Keep work in explicit, well-named paths and use the pushed repository work item plus repository state as the durable coordination source until board access is configured.
