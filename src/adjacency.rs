//! A neighbourhood (adjacency) operator over the wordings a traversal has
//! already emitted.
//!
//! ## Why this exists
//!
//! A best-first traversal reaches a wording by *cumulative* cost, so a
//! wording that is jointly excellent but locally expensive in **one** slot is
//! unreachable at any affordable budget: getting to it costs the sum of every
//! better-bound node standing in front of it.  Depth in one slot is
//! multiplicative in reach, so a product-space walk spends its whole
//! allowance on the cheap corner.
//!
//! The fix is not a better key — the key was refuted by measurement — and not
//! a bigger budget, which was measured inert.  The fix is a different
//! *neighbourhood*.  Where the traversal moves by extending a prefix, this
//! operator moves by **substituting one slot of a complete wording**, so a
//! wording `d` substitutions away from a parent the traversal did reach
//! costs `d` steps instead of the size of the product in front of it.  Depth
//! in one slot becomes **additive**.
//!
//! ## What it is, precisely
//!
//! A best-first walk over complete wordings, seeded with the wordings a
//! traversal has already emitted.  Each pop expands one slot substitution
//! away, re-scores the child with the caller's own key, and admits the child
//! to the pool.  Because the caller's key is an admissible upper bound on the
//! score of every completion (the same bound the traversal orders by), the
//! admission order is descending in score exactly as the traversal's is.
//!
//! Two named bounds keep it from turning into a second product enumeration:
//!
//! * `pops` — how many wordings the walk may expand.  Each pop emits at most
//!   one admission, so this also caps admissions.
//! * `per_slot` — how many of a node's children in *one* slot enter the
//!   frontier.  The children of a node in one slot are ranked by the caller's
//!   key, and the best few are kept; the walk is a bounded beam over
//!   substitutions rather than a full width sweep of the product.
//!
//! The operator reads no vocabulary and knows no phrases: it is a function
//! of index tuples, list widths, and a scoring closure, so it is the same
//! operator for every target and every segmentation.

use std::cmp::{Ordering, Reverse};
use std::collections::{BinaryHeap, HashSet};

/// How far the neighbourhood walk goes, and how wide its beam is.
#[derive(Debug, Clone, Copy)]
pub(crate) struct Neighbourhood {
    /// Wordings the walk may expand.  Each pop admits at most one new
    /// wording, so this bounds admissions too.
    pub(crate) pops: usize,
    /// Children per slot that enter the frontier from one expanded node.
    pub(crate) per_slot: usize,
}

/// One complete wording in the walk, ordered so that [`BinaryHeap`] — a max
/// heap — yields the best-scoring wording first, and ties break on the tuple
/// itself so the walk's order is total and reproducible.
#[derive(Debug, Eq, PartialEq)]
struct Node {
    key: i64,
    tuple: Vec<usize>,
}

impl Ord for Node {
    fn cmp(&self, other: &Self) -> Ordering {
        self.key
            .cmp(&other.key)
            .then_with(|| other.tuple.cmp(&self.tuple))
    }
}

impl PartialOrd for Node {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

/// The same quantization the lexical traversal orders by, so the operator's
/// key is the traversal's key and the two are directly comparable.
fn quantize(score: f64) -> i64 {
    (score * 1_000_000_000.0).round() as i64
}

/// The best `keep` children of one node in one slot, by the caller's key.
///
/// A bounded min-heap: the worst retained child is the one evicted, and the
/// result comes back in descending key order so the frontier is filled
/// best-first whatever the list order was.
fn best_children(
    node: &Node,
    slot: usize,
    width: usize,
    keep: usize,
    bound: &dyn Fn(&[usize]) -> f64,
) -> Vec<Node> {
    let mut best: BinaryHeap<Reverse<Node>> = BinaryHeap::new();
    let mut scratch = node.tuple.clone();
    for i in 0..width {
        if i == node.tuple[slot] {
            continue;
        }
        scratch[slot] = i;
        let child = Node {
            key: quantize(bound(&scratch)),
            tuple: scratch.clone(),
        };
        if best.len() < keep {
            best.push(Reverse(child));
        } else if let Some(Reverse(worst)) = best.peek() {
            if child > *worst {
                best.pop();
                best.push(Reverse(child));
            }
        }
    }
    let mut out: Vec<Node> = best.into_iter().map(|Reverse(n)| n).collect();
    out.sort_by(|a, b| b.cmp(a));
    out
}

/// The wordings this operator adds to the pool, best first.
///
/// `roots` are the wordings already emitted (by the traversal and by whatever
/// else shares the pool); `widths` are the candidate-list lengths, one per
/// slot; `bound` is the caller's own re-score, which must be an admissible
/// upper bound on the score of a completion for the admission order to be
/// descending in score.
///
/// A wording already in `roots` is never re-admitted, and a wording admitted
/// here is never admitted twice, so the caller can spend its allowance on
/// these alone without re-checking anything.
pub(crate) fn admit(
    plan: Neighbourhood,
    roots: &[Vec<usize>],
    widths: &[usize],
    bound: &dyn Fn(&[usize]) -> f64,
) -> Vec<Vec<usize>> {
    let depth = widths.len();
    if depth == 0 || plan.pops == 0 || plan.per_slot == 0 {
        return Vec::new();
    }

    // `seen` is the frontier's visited set: a tuple is expanded at most once,
    // and a tuple already in the pool is never re-proposed.
    let mut seen: HashSet<Vec<usize>> =
        HashSet::with_capacity(roots.len() + plan.pops);
    let mut held: HashSet<Vec<usize>> = HashSet::with_capacity(roots.len());
    let mut heap: BinaryHeap<Node> = BinaryHeap::with_capacity(roots.len());

    for root in roots {
        if root.len() != depth {
            continue;
        }
        if seen.insert(root.clone()) {
            held.insert(root.clone());
            heap.push(Node {
                key: quantize(bound(root)),
                tuple: root.clone(),
            });
        }
    }

    let mut out: Vec<Vec<usize>> = Vec::new();
    for _ in 0..plan.pops {
        let Some(node) = heap.pop() else {
            break;
        };
        if node.tuple.len() != depth {
            continue;
        }

        // A node that is not already in the pool is one substitution away from
        // a wording the pool holds, and that is exactly the admission this
        // operator exists to make.
        if held.insert(node.tuple.clone()) {
            out.push(node.tuple.clone());
            if out.len() >= plan.pops {
                break;
            }
        }

        for slot in 0..depth {
            for child in best_children(
                &node,
                slot,
                widths[slot],
                plan.per_slot,
                bound,
            ) {
                if seen.insert(child.tuple.clone()) {
                    heap.push(child);
                }
            }
        }
    }

    out
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A toy score: each slot's index is a cost, and a wording's score falls
    /// with the total.  The best wording is therefore the one that is deep in
    /// exactly one slot and at 0 everywhere else — the shape a pure
    /// cost-ordered walk cannot reach inside a small allowance.
    fn toy_bound(tuple: &[usize]) -> f64 {
        1.0 - tuple.iter().sum::<usize>() as f64 / 100.0
    }

    #[test]
    fn reaches_a_wording_deep_in_one_slot() {
        // A pool that holds only the corner, and one word per slot.
        let widths = [16usize, 16, 16, 16];
        let roots = vec![vec![0, 0, 0, 0]];
        let got = admit(
            Neighbourhood {
                pops: 64,
                per_slot: 4,
            },
            &roots,
            &widths,
            &toy_bound,
        );
        // The best reachable wording is deep in the last slot.
        assert_eq!(got.first().map(Vec::as_slice), Some([0, 0, 0, 1].as_slice()));
        assert!(
            got.iter().any(|t| t.iter().copied().max() == Some(9)),
            "the walk must reach a slot index well past any small cap, got {got:?}"
        );
    }

    #[test]
    fn never_readmits_the_pool() {
        let widths = [8usize, 8];
        let roots = vec![vec![0, 0], vec![1, 1]];
        let got = admit(
            Neighbourhood {
                pops: 16,
                per_slot: 2,
            },
            &roots,
            &widths,
            &toy_bound,
        );
        for t in &got {
            assert!(
                !roots.contains(t),
                "re-admitted a wording already in the pool: {t:?}"
            );
        }
    }

    #[test]
    fn admits_each_wording_at_most_once() {
        let widths = [6usize, 6, 6];
        let roots = vec![vec![0, 0, 0]];
        let got = admit(
            Neighbourhood {
                pops: 48,
                per_slot: 3,
            },
            &roots,
            &widths,
            &toy_bound,
        );
        let mut sorted = got.clone();
        sorted.sort();
        sorted.dedup();
        assert_eq!(sorted.len(), got.len(), "duplicate admission: {got:?}");
    }

    #[test]
    fn admission_order_is_descending_in_score() {
        let widths = [12usize, 12];
        let roots = vec![vec![0, 0]];
        let got = admit(
            Neighbourhood {
                pops: 32,
                per_slot: 4,
            },
            &roots,
            &widths,
            &toy_bound,
        );
        for pair in got.windows(2) {
            assert!(
                toy_bound(&pair[0]) >= toy_bound(&pair[1]),
                "admission order is not descending in score: {got:?}"
            );
        }
    }

    #[test]
    fn is_deterministic() {
        let widths = [10usize, 10, 10, 10];
        let roots: Vec<Vec<usize>> =
            (0..4).map(|i| vec![i, 0, i, 1]).collect();
        let plan = Neighbourhood {
            pops: 40,
            per_slot: 3,
        };
        let first = admit(plan, &roots, &widths, &toy_bound);
        let second = admit(plan, &roots, &widths, &toy_bound);
        assert_eq!(first, second);
    }

    #[test]
    fn an_empty_plan_or_a_flat_segmentation_admits_nothing() {
        let widths = [4usize];
        let roots = vec![vec![0]];
        assert!(admit(
            Neighbourhood {
                pops: 0,
                per_slot: 4
            },
            &roots,
            &widths,
            &toy_bound
        )
        .is_empty());
        assert!(admit(
            Neighbourhood {
                pops: 8,
                per_slot: 0
            },
            &roots,
            &widths,
            &toy_bound
        )
        .is_empty());
        assert!(admit(
            Neighbourhood {
                pops: 8,
                per_slot: 4
            },
            &roots,
            &[],
            &toy_bound
        )
        .is_empty());
    }
}
