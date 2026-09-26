# Repository work items

Repository work items are the default ticket/task mechanism when recurring agents need durable work that does not depend on GitHub Issues or another external tracker.

A work item is a Markdown file committed to the shared accumulation branch. The itinerary defines the accepted work-item locations. New standalone tasks should normally use `docs/work/items/w-<six-hex>.md`. Existing continuation or handoff documents may also be work items.

## Metadata

Put a small YAML header at the top of every work item:

```yaml
---
work_item: true
id: w-a1b2c3
state: open
priority: normal
owner: null
updated: 2026-09-26T05:00:00Z
branch: null
worktree: null
---
```

Allowed states are `open`, `working`, `blocked`, `done`, and `superseded`.

The body should contain enough durable context to work without recovering a chat transcript. At minimum, give the goal, relevant context or references, concrete completion criteria, and a handoff/notes section.

## Submit a task

Create a new work-item file with a fresh `w-<six-hex>` ID, `state: open`, no owner, and clear completion criteria. Commit and push it to the shared accumulation branch. The successful push is the submission event.

A human request, PR review finding, CI regression, spec gap, or external issue can all be converted into a repository work item. Link the source when useful, but copy enough context into the work item that the external source is not required for discovery.

## Claim and update

To claim work, fetch the shared accumulation branch, update only the selected item to `state: working`, set a unique owner ID, refresh `updated`, and record the focused branch/worktree when known. Push the metadata change.

The claim is valid only if the push succeeds against current repository state. If it is rejected or another owner appears, re-read the item and do not overwrite the competing claim.

Update the item when durable state changes: useful commits, a new blocker, a handoff, completion, or a materially different next action. Do not create noisy heartbeat-only commits.

## Handoff

Before stopping unfinished work, make local work durable first: commit and push useful branches or archive experiments that should not be lost. Then update the work item with the objective state, relevant branch/commit names, validation results, blocker details, and the next action.

If a work item becomes blocked, set `state: blocked` and say what concrete condition would unblock it. A later coordinator may reopen it when that condition changes.

## Complete or supersede

Set `state: done` only after the stated completion criteria and itinerary checks are verified. Use `superseded` when another work item replaces it, and link the replacement.

Completed files may remain in place; discovery is based on metadata state, so there is no need to move them merely to keep the active queue small.

## External trackers

GitHub Issues and other trackers are optional mirrors or intake sources. If they are enabled and useful, link them from the repository work item. Unless the itinerary explicitly says otherwise, the repository work item remains the durable source that recurring agents must be able to discover.
