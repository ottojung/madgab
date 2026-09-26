# Madgab recurring improvement itinerary

Work on the `ottojung/madgab` repository and keep making concrete improvements to the generator.

Read [scheduled.md](scheduled.md) before doing work. Use Antonina agents through subprocesses for substantive agentic work.

## Active branch

The current accumulation branch is:

```text
post-milestone-acceptance
```

Scheduled work must **never** be pushed or merged directly into `main`.

Create focused issue/work branches and worktrees from the active accumulation branch when useful. Review and validate the work, then integrate it into `post-milestone-acceptance`. Push that branch and its focused child branches as needed. Do not create a competing accumulation branch unless a human explicitly changes this itinerary.

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

1. Inspect the current `post-milestone-acceptance` state and any already-running Antonina agents/worktrees before starting duplicate work.
2. Reproduce the current executable behavior for the two primary examples.
3. Identify the highest-value general reason a desired proposal is absent, pruned, or ranked out.
4. Delegate substantial investigation or implementation to Antonina agents in isolated worktrees.
5. Prefer small coherent changes over large rewrites.
6. Validate with relevant Rust tests and direct executable runs.
7. Add regression tests that express useful external behavior without baking in implementation details or phrase-specific production hacks.
8. Review completed work before integrating it into the accumulation branch.
9. Keep the accumulation branch usable and push progress there; never merge scheduled work into `main`.

If both primary examples are reliably generated, continue improving general approximate-search quality, ranking, runtime, and maintainability rather than stopping the recurring loop.

## Completion checks for the primary milestone

The primary milestone is reached when all of the following are true on the active accumulation branch:

- the executable's approximate mode can produce `wreck a nice beach` for `recognize speech`;
- the executable's approximate mode can produce `Hits Justice Dupe Hid Came` for `It's just a stupid game`;
- the behavior is covered by regression tests at an appropriate API or executable boundary;
- the implementation does not contain phrase-specific hard-coding for these examples;
- relevant tests pass;
- the integrated work is on `post-milestone-acceptance`, not `main`.

After that milestone, continue with broadly useful quality improvements unless a human changes this itinerary.
