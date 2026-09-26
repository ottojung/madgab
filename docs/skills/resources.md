# Durable host resources

Use this skill when scheduled or agentic work creates, uses, or hands off a durable host filesystem location. A resource is a host and canonical absolute POSIX path, such as `lubko://marceline-dev` plus `/workspace/project-worktree`; the path may be a file or directory.

Resource registration is dependency and liveness metadata, not exclusive ownership or a lock. Multiple open Antonina issues may depend on one resource. Every number in these commands is an **Antonina BOARD issue number**, never a GitHub issue number.

## Garbage collection semantics

A deterministic garbage collector may remove unprotected host paths. When the Antonina board is available, protect durable paths with open issue dependencies rather than relying on timing.

## Commands

```sh
antonina board resource list [--host HOST] [--issue NUMBER]
antonina board resource add ISSUE HOST PATH
antonina board resource remove ISSUE HOST PATH
```

`--host` and `--issue` narrow inspection; verify registrations rather than assuming a command succeeded.

## Safe handoff

1. Inspect the old issue's resources.
2. Add the follow-up open issue dependency first.
3. Verify the resource list output includes the follow-up dependency and shows it protected.
4. Only then close the old issue or remove the old dependency.

Host values are `lubko://<server>` with no trailing slash; paths are canonical absolute POSIX paths.

If the current host does not have Antonina board credentials, do not fabricate them. Keep work in explicit, well-named paths and let the itinerary/repository state remain the durable coordination source until board access is configured.
