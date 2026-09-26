# Code review

Review the resulting implementation, not merely whether automation accepts it. Look for concrete soundness, completeness, regression-safety, maintainability, and performance problems.

Read the selected repository work item or itinerary, its acceptance criteria and handoff notes, the diff, relevant surrounding code, and tests. Tests are evidence of intended behavior, not proof of correctness.

For every blocking finding, state the concrete trigger, resulting problem, and required change. Prefer a few high-confidence findings over speculative comments.

When a review discovers substantial follow-up work that should survive beyond the current invocation, add it to the current work item if it is part of the same completion criteria or submit a new repository work item using [work-items.md](work-items.md). Do not rely on a GitHub Issue being available.

For Madgab in particular, reject phrase-specific hard-coding of the canonical examples. A change that makes only the known example strings pass without improving the general phonetic/search model is not an acceptable implementation.

Keep the review read-only unless a separate fixing task explicitly asks for changes.
