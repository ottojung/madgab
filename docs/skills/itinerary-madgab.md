# MadGab recurring improvement itinerary

## Status: accepted and paused

A human accepted the current MadGab state for release on September 27, 2026. The autonomous approximate-search research loop is **paused**.

Scheduled orchestrators must not create new MadGab work items, claim existing historical items, launch MadGab agents, or resume superseded fronts unless a human explicitly asks to reopen MadGab development.

The accepted state and its known limitation are documented in [../accepted-state-2026-09-27.md](../accepted-state-2026-09-27.md).

The important unresolved limitation is intentionally preserved rather than hidden: approximate mode can generate `wreck a nice beach` for `recognize speech`, but the production candidate pool still does not generate the classical `Hits Justice Dupe Hid Came` clue for `It's just a stupid game`. That limitation is acceptable for the current release.

## If development is explicitly reopened

Read [scheduled.md](scheduled.md), [work-items.md](work-items.md), the accepted-state document, and the historical reports under `docs/work/`. Use repository-native work items for durable new tasks. Do not re-run already priced negative fronts unless a new argument changes their assumptions.

Work on a fresh focused branch from `main`, validate general behavior rather than hard-coding canonical phrases, and integrate only reviewed changes. The historical `post-milestone-acceptance` branch is release history after this acceptance and is no longer an automatic accumulation target.
