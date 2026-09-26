# Code review

Review the resulting implementation, not merely whether automation accepts it. Look for concrete soundness, completeness, regression-safety, maintainability, and performance problems.

Read the task or itinerary, the diff, relevant surrounding code, and tests. Tests are evidence of intended behavior, not proof of correctness.

For every blocking finding, state the concrete trigger, resulting problem, and required change. Prefer a few high-confidence findings over speculative comments.

For Madgab in particular, reject phrase-specific hard-coding of the canonical examples. A change that makes only the known example strings pass without improving the general phonetic/search model is not an acceptable implementation.

Keep the review read-only unless a separate fixing task explicitly asks for changes.
