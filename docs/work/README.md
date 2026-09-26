# Madgab work queue

Madgab uses repository-native work items instead of depending on GitHub Issues.

The reusable protocol is [../skills/work-items.md](../skills/work-items.md). New standalone tasks should normally be created under `docs/work/items/` from [TEMPLATE.md](TEMPLATE.md). Continuation documents elsewhere under `docs/` are also valid work items when their YAML metadata contains `work_item: true`.

The active accumulation branch is defined by [../skills/itinerary-madgab.md](../skills/itinerary-madgab.md). Submit, claim, hand off, and complete work by committing the relevant work-item metadata/body changes to that branch.

Agents should not infer an unlimited backlog from TODO comments or old archive branches. Explicit work items and the itinerary are the queue; CI failures, reviews, specs, and human requests become recurring work by being written into a work item.
