# Madgab recurring improvement itinerary

Work on the `ottojung/madgab` repository and keep making concrete improvements to the generator.

Read [scheduled.md](scheduled.md) and [work-items.md](work-items.md) before doing work. Use Antonina agents through subprocesses for substantive agentic work.

## Active branch

The current accumulation branch is:

```text
post-milestone-acceptance
```

Scheduled work must **never** be pushed or merged directly into `main`.

Create focused task/work branches and worktrees from the active accumulation branch when useful. Review and validate the work, then integrate it into `post-milestone-acceptance`. Push that branch and its focused child branches as needed. Do not create a competing accumulation branch unless a human explicitly changes this itinerary.

## Work sources

Madgab does not depend on GitHub Issues as a work queue. Repository-native work items are the durable source of scheduled work.

On startup, discover work in this order:

1. Markdown work items under `docs/work/items/` whose metadata state is `open` or legitimately resumable.
2. Continuation/handoff documents under `docs/` whose YAML metadata contains `work_item: true`, including [../continuation-approximate-search.md](../continuation-approximate-search.md).
3. A `working` item already owned by this coordinator, or one that is clearly abandoned after inspecting its referenced branches, worktrees, and agents.
4. The standing primary goal below when no explicit work item remains.

GitHub Issues, PR comments, CI failures, review findings, specs, and human prompts are valid intake sources, but recurring work from them should be captured in a repository work item. Create new standalone tasks from [../work/TEMPLATE.md](../work/TEMPLATE.md) and follow [work-items.md](work-items.md).

Do not manufacture a backlog from old archive branches, stale TODO comments, or every possible cleanup. Explicit work items plus this itinerary define what scheduled agents should pursue.

## Primary goal

Improve Madgab's approximate search so the executable can actually generate these canonical near-homophonic spellings as proposals:

```text
recognize speech
→ wreck a nice beach

It's just a stupid game
→ Hits Justice Dupe Hid Came
```

The important property is that the requested clue appears in the generated approximate proposal set for its input. Do not overspecify an arbitrary exact rank unless ranking itself is the thing being improved.

Do not hard-code either sentence, clue, or phrase-specific exception. Improvements must come from general search, pronunciation, phonetic-distance, segmentation, scoring, or ranking behavior that can help other inputs too.

## Work loop

On every turn:

1. Inspect the current `post-milestone-acceptance` state, explicit work-item records, and any already-running Antonina agents/worktrees before starting duplicate work.
2. Select and claim the highest-value explicit work item according to [work-items.md](work-items.md), or fall back to the standing primary goal if no explicit item exists.
3. Reproduce the relevant current executable behavior before changing it.
4. Identify the highest-value general reason the desired behavior is absent, pruned, slow, or ranked out.
5. Delegate substantial investigation or implementation to Antonina agents in isolated worktrees.
6. Prefer small coherent changes over large rewrites.
7. Validate with relevant Rust tests and direct executable runs.
8. Add regression tests that express useful external behavior without baking in implementation details or phrase-specific production hacks.
9. Review completed work before integrating it into the accumulation branch.
10. Make useful local work durable by committing and pushing it before handoff.
11. Update the work item with objective state, validation, blockers, and next action; mark it `done` only when its completion criteria are verified.
12. Keep the accumulation branch usable and pushed; never merge scheduled work into `main`.

If both primary examples are reliably generated, continue with explicit repository work items and broadly useful approximate-search quality, ranking, runtime, and maintainability improvements rather than stopping the recurring loop.

## Completion checks for the primary milestone

The primary milestone is reached when all of the following are true on the active accumulation branch:

- the executable's approximate mode can produce `wreck a nice beach` for `recognize speech`;
- the executable's approximate mode can produce `Hits Justice Dupe Hid Came` for `It's just a stupid game`;
- the behavior is covered by regression tests at an appropriate API or executable boundary;
- the implementation does not contain phrase-specific hard-coding for these examples;
- relevant tests pass;
- the integrated work is on `post-milestone-acceptance`, not `main`.

After that milestone, continue with explicit work items and broadly useful quality improvements unless a human changes this itinerary.
