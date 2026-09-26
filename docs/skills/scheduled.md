# Scheduled Antonina agent work

This document is the reusable operating guide for recurring agents that work on Antonina-managed repositories. It defines work discovery, ownership, recovery, liveness, and completion for agentic work. The calling project itinerary supplies the target repository, work sources, branch policy, and completion predicate.

Read [work-items.md](work-items.md) for the repository-native task protocol.

## Contract

Keep these resources distinct:

- **Antonina** — the preferred coding-agent runtime used to perform substantive repository work. Antonina is a capable agent and should be used for work that benefits from judgment, context, iteration, or multiple steps.
- **Coordinating agent** — the agent following this document. It selects work, launches subprocesses, observes progress, and keeps the overall loop moving.
- **Target repository** — the repository, task records, branches, pull requests, validation requirements, and other durable state selected by the itinerary.

The coordinating agent should use Antonina by launching `antonina agent ...` commands as subprocesses. The coordinating agent coordinates; Antonina agents do the substantive agentic work. Do not replace Antonina with ad-hoc direct model calls when an Antonina agent is appropriate.

Every invocation of the coordinating agent is disposable and should assume no useful conversational continuity from prior invocations. Repository work-item records, Antonina agent state and logs, repository state, CI, pull requests, and other explicit host state are the sources of truth; conversation memory is only context.

GitHub Issues are optional. Do not assume they exist or use them as the canonical queue unless the project itinerary explicitly says so.

## Coordinator pass

Treat each invocation as a fresh reconciliation pass, not as a long-lived supervisor.

Inspect durable state, existing Antonina agents, worktrees, branches, reviews, and blockers; take useful coordination actions; then exit promptly. Useful actions include claiming or recovering work, splitting work into independent fronts, starting or prompting agents, reviewing completed work, integrating validated work, and updating repository work-item handoffs.

Do not keep the coordinating invocation alive merely to wait for long-running Antonina agents. In particular, avoid multi-minute sleeps or long `antonina agent wait` calls whose only purpose is to poll later. Leave running agents running and let the next scheduled invocation inspect them afresh. A short wait is fine when a result is expected within seconds and immediately affects the current coordination decision.

The recurring scheduler should be able to start a fresh coordinating invocation at its intended cadence. Aim to finish the coordinating pass and exit before four minutes have elapsed. If that point is approaching, record durable state and return rather than continuing to supervise or starting more coordination work.

## Startup

1. Read the project itinerary, this document, and [work-items.md](work-items.md).
2. Discover work from the sources named by the itinerary. Prefer explicit repository work-item and continuation documents over inventing new work.
3. Read the selected work item's current state and handoff notes before claiming it.
4. Claim only work that is not actively owned. A repository work-item claim counts only after the ownership metadata update has been successfully pushed to the shared accumulation branch.
5. If a push races with another coordinator, fetch, re-read the work item, and yield or reconcile instead of overwriting the other claim.
6. Use a preassigned base-16 Antonina agent ID and an explicit target worktree cwd.
7. Launch and control Antonina agents through subprocesses.
8. Take the useful coordination actions available in this pass, record durable handoff state, and return without waiting for unrelated long-running work to finish.

## Work discovery and ownership

The itinerary defines which repository paths are work sources. The normal source is one Markdown file per task, as described in [work-items.md](work-items.md). A continuation or handoff document may itself be a work item when it carries the same metadata.

Prefer work in this order unless the itinerary says otherwise:

1. an open explicit work item;
2. an already-working item owned by this coordinator or clearly abandoned after inspecting its referenced branch, worktree, and agent state;
3. a continuation/handoff item with concrete unfinished acceptance criteria;
4. the itinerary's standing improvement goal when no explicit task remains.

Do not treat a timestamp alone as a lock or as proof of abandonment. Before taking over a working item, inspect its referenced resources and objective repository state. When taking over, update the owner and handoff notes in the same pushed change.

GitHub Issues, pull-request comments, CI failures, specs, checklists, and human prompts can all create or refine work, but durable recurring work should be captured in a repository work item so the next invocation does not depend on external UI state.

## Agent operation

Use Antonina for work requiring judgment, context, iteration, or multiple steps. The normal pattern is for the coordinating agent to spawn Antonina CLI subprocesses such as:

```sh
antonina agent new --id <agent-id> --cwd <worktree>
antonina agent prompt --id <agent-id> '<task>'
antonina agent status --id <agent-id>
antonina agent log --id <agent-id>
```

Exploit parallelism whenever useful. If several investigations, implementations, reviews, or other work items are materially independent, prefer running multiple Antonina agents concurrently in separate worktrees rather than serializing them without reason. Look for opportunities to split work into independent fronts, but avoid spawning agents that would merely duplicate the same work or contend on the same files.

Use direct shell only for tiny deterministic observations or coordination glue. Record the Antonina agent ID before invocation, retain durable logs, and inspect status and logs while work is nonterminal. Never treat a progress message or green test as completion by itself.

Before relying on a durable host path, follow [resources.md](resources.md): register it with `antonina board resource add`, verify it, and preserve open dependencies until handoff or completion when the Antonina board is available for the work.

## Handoff and completion

Keep the selected work item current enough that another invocation can resume from repository state alone. Before stopping, record concrete commits/branches, validation already run, unresolved blockers, and the next useful action. Push that handoff.

Define completion from the work item plus calling itinerary. It must include the requested repository result, required validation, review expectations, and no unresolved blockers. Mark a work item `done` only after those conditions are objectively verified.

A single coordinating invocation does not need to complete the selected work item. It is successful when it makes useful progress or a useful coordination decision and leaves enough durable state for a later fresh invocation to continue safely.

If new follow-up work is discovered while completing an item, either add it to the current item when it is part of the same acceptance criteria or submit a new repository work item. Do not leave important follow-up work only in chat, local notes, or an unpushed worktree.
