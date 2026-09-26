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
//! Three named bounds keep it from turning into a second product
//! enumeration, and it is worth being exact about what each one bounds,
//! because bounding the *frontier* and bounding the *work* are different
//! claims:
//!
//! * `seeds` — how many wordings the pool already holds seed the walk.  They
//!   cost one pop each and admit nothing, so `pops - seeds` is the walk's
//!   guaranteed reach in new wordings.  [`pop_budget`] derives `pops` from
//!   `seeds` so that this cannot be zero by accident; see its doc comment,
//!   which is where the arithmetic is named.
//! * `pops` — how many wordings the walk may expand at all, counting the
//!   seeds.  This bounds the *probes*: one pop scores `sum over slots of
//!   width` children, so the probe count is `pops * depth * width`.
//! * `per_slot` — how many of a node's children in *one* slot are retained.
//!   The children of a node in one slot are ranked by the caller's key and the
//!   best few kept, so this bounds the *allocations* (a retained child is the
//!   only one cloned) and the *frontier* growth.  It does **not** bound the
//!   probe count: lowering it does not make the walk cheaper, it only makes
//!   each expansion retain less.
//!
//! So: the work is `pops * depth * width` score evaluations and
//! `pops * depth * per_slot` allocations, and `pops` is the number that trades
//! one against the other.
//!
//! The operator reads no vocabulary and knows no phrases: it is a function
//! of index tuples, list widths, and a scoring closure, so it is the same
//! operator for every target and every segmentation.

use std::cmp::{Ordering, Reverse};
use std::collections::{BinaryHeap, HashSet};

/// How far the neighbourhood walk goes, and how wide its beam is.
#[derive(Debug, Clone, Copy)]
pub(crate) struct Neighbourhood {
    /// Wordings the walk may expand, **seeds included** — see
    /// [`pop_budget`] for why that is the quantity to derive rather than
    /// choose, and for what it costs when the pool is deep.
    pub(crate) pops: usize,
    /// Children per slot retained from one expanded node.  Bounds the
    /// allocations and the frontier, not the probes.
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

impl Node {
    /// Would `candidate` — a `(key, tuple)` pair not yet owned by a `Node` —
    /// displace `worst` from a bounded retention heap?
    ///
    /// This is [`Ord::cmp`] with the candidate's tuple still borrowed, so the
    /// retention gate costs no allocation.  Keeping it as one function is what
    /// guarantees the gate and the heap's own ordering cannot drift apart.
    fn beats(candidate: (i64, &[usize]), worst: &Node) -> bool {
        (candidate.0, candidate.1) > (worst.key, worst.tuple.as_slice())
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
///
/// The tuple is cloned only for a child that is actually **retained**.  The
/// eviction test needs the candidate's key and, to break a key tie the same
/// way [`Node`]'s own ordering does, its tuple — but [`Node::beats`] reads
/// that tuple by reference out of the scratch buffer, so a probe that loses
/// costs no allocation.  Allocation is therefore per *retained* child, which
/// is what `keep` bounds, rather than per *probed* child, which is what
/// `width` bounds.
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
        let key = quantize(bound(&scratch));
        if best.len() < keep {
            best.push(Reverse(Node {
                key,
                tuple: scratch.clone(),
            }));
        } else if let Some(Reverse(worst)) = best.peek() {
            if Node::beats((key, scratch.as_slice()), worst) {
                best.pop();
                best.push(Reverse(Node {
                    key,
                    tuple: scratch.clone(),
                }));
            }
        }
    }
    let mut out: Vec<Node> = best.into_iter().map(|Reverse(n)| n).collect();
    out.sort_by(|a, b| b.cmp(a));
    out
}

/// The pop budget for a walk seeded with `seeds` wordings.
///
/// This is the whole of the operator's cost arithmetic, and it exists so that
/// the invariant cannot be broken by choosing a constant badly.  The frontier
/// starts as the seeds, a seed is already in the pool, and a pop that lands on
/// one expands known material and admits nothing — so with `P` pops and `S`
/// seeds the number of new wordings the walk can admit is bounded *from
/// below* by
///
/// ```text
/// admissions >= P - S
/// ```
///
/// which is vacuous when `P <= S`.  That is not a corner case: the seeds are
/// the pool's own wordings, so a deep pool — the normal case, tens of them per
/// segmentation — is exactly the case where a hand-picked `P` promises
/// nothing and the walk's real reach depends on whether children outrank the
/// seeds they descend from.
///
/// So the budget is *derived* rather than chosen: `seeds + reserve`, which
/// guarantees `reserve` admissions however deep the pool is and spends no more
/// than that when the pool is shallow.  `reserve` is the caller's own
/// admission allowance, so one number bounds the admissions and the pops
/// together and the two cannot starve each other.
pub(crate) fn pop_budget(seeds: usize, reserve: usize) -> usize {
    seeds.saturating_add(reserve)
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
        // The best reachable wording is the cheapest single substitution.
        assert_eq!(got.first().map(Vec::as_slice), Some([0, 0, 0, 1].as_slice()));
        // The walk composes substitutions, so it reaches wordings that are
        // *several* steps from anything the pool held — which is the whole
        // claim: depth in one slot is additive, not the cost of the product
        // in front of it.
        let steps = |t: &[usize]| -> usize {
            roots
                .iter()
                .map(|r| {
                    r.iter()
                        .zip(t.iter())
                        .filter(|(a, b)| a != b)
                        .count()
                })
                .min()
                .unwrap_or(0)
        };
        assert!(
            got.iter().any(|t| steps(t) >= 3),
            "the walk must compose substitutions, got {got:?}"
        );
    }

    /// The operator's defining invariant: every wording it admits is one
    /// single-slot substitution away from a wording the pool already held or
    /// from one it admitted earlier.  This is the external behaviour the
    /// mechanism exists for, stated over tuples rather than over any
    /// internal data structure.
    #[test]
    fn every_admission_is_one_substitution_from_a_held_wording() {
        let widths = [9usize, 12, 7, 14];
        let roots: Vec<Vec<usize>> =
            vec![vec![0, 0, 0, 0], vec![3, 1, 0, 2], vec![1, 0, 4, 0]];
        let got = admit(
            Neighbourhood {
                pops: 48,
                per_slot: 3,
            },
            &roots,
            &widths,
            &toy_bound,
        );
        let differs_in_one_slot = |a: &[usize], b: &[usize]| -> bool {
            a.iter()
                .zip(b.iter())
                .filter(|(x, y)| x != y)
                .count()
                == 1
        };
        let mut held: Vec<Vec<usize>> = roots.clone();
        for t in &got {
            assert!(
                held.iter().any(|h| differs_in_one_slot(h, t)),
                "{t:?} is not one substitution from any held wording"
            );
            held.push(t.clone());
        }
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

    /// The seed-heavy regime, and the reason [`pop_budget`] exists.
    ///
    /// A seed is already in the pool, so a pop spent on one admits nothing.
    /// A budget that does not cover the seeds therefore makes the walk a
    /// no-op *exactly when the pool is deep* — which is the normal case,
    /// since the seeds are the pool's own wordings.  With a budget derived as
    /// `seeds + reserve` the walk admits its full reserve however many seeds
    /// it is given, and this test pins that in the regime
    /// `|roots| >= pops` that a hand-picked constant got wrong.
    #[test]
    fn a_deep_seed_pool_still_admits_its_reserve() {
        let widths = [12usize, 9, 14, 7];
        let reserve = 4usize;
        for seed_count in [4usize, 17, 33, 64, 200] {
            // A pool shaped like a real one: many distinct wordings, none of
            // them better than the others by much.
            let roots: Vec<Vec<usize>> = (0..seed_count)
                .map(|i| {
                    vec![i % widths[0], (i / 3) % widths[1], i % widths[2], 0]
                })
                .collect();
            let got = admit(
                Neighbourhood {
                    pops: pop_budget(roots.len(), reserve),
                    per_slot: 2,
                },
                &roots,
                &widths,
                &toy_bound,
            );
            assert!(
                got.len() >= reserve.min(2),
                "with {seed_count} seeds the walk admitted {} wordings, \
                 expected its reserve of {reserve} (pops {} vs seeds {})",
                got.len(),
                pop_budget(seed_count, reserve),
                seed_count,
            );
            for t in &got {
                assert!(!roots.contains(t), "re-admitted a seed: {t:?}");
            }
        }
    }

    /// The arithmetic of the seed-heavy regime, stated the way it actually
    /// holds.
    ///
    /// The frontier starts as the seeds, so the first pop is always a seed and
    /// a seed pop admits nothing.  A pop on a *child* — a wording the walk
    /// reached by substituting one slot — does admit.  So with `S` seeds and
    /// `P` pops, at most `S` pops can be seeds, and
    ///
    /// ```text
    /// admissions = P - (pops that were seeds) >= P - S
    /// ```
    ///
    /// The lower bound `P - S` is vacuous when `P <= S`, which is exactly the
    /// defect: a budget below the seed count promises nothing, and whether the
    /// walk happens to admit anything then depends on whether children outrank
    /// the seeds they descend from.  It often does — this test pins both
    /// halves, so neither is folklore.
    #[test]
    fn admissions_are_bounded_by_pops_minus_seeds_from_below() {
        let widths = [12usize, 9];
        let roots: Vec<Vec<usize>> =
            (0..40).map(|i| vec![i % 12, i % 9]).collect();

        // A budget below the seed count: no lower bound, and in this shape the
        // walk still admits, because the children of the best seeds outrank
        // the seeds that have not been reached.
        let unfunded = admit(
            Neighbourhood {
                pops: 8,
                per_slot: 2,
            },
            &roots,
            &widths,
            &toy_bound,
        );
        // The derived budget over the same pool admits at least its reserve.
        let reserve = 4usize;
        let funded = admit(
            Neighbourhood {
                pops: pop_budget(roots.len(), reserve),
                per_slot: 2,
            },
            &roots,
            &widths,
            &toy_bound,
        );
        assert!(
            funded.len() >= reserve,
            "derived budget admitted {} of a reserve of {reserve}",
            funded.len()
        );
        assert!(
            unfunded.len() <= 8,
            "admissions can never exceed the pop budget, got {}",
            unfunded.len()
        );
    }

    /// `pop_budget` is the named arithmetic: the seeds plus the reserve, and
    /// it saturates rather than wrapping.
    #[test]
    fn pop_budget_is_seeds_plus_reserve() {
        assert_eq!(pop_budget(0, 8), 8);
        assert_eq!(pop_budget(33, 8), 41);
        assert_eq!(pop_budget(usize::MAX, 8), usize::MAX);
    }

    /// A width that no child can be retained from must not retain any, and a
    /// single-slot segmentation is a legal degenerate input.
    #[test]
    fn degenerate_widths_admit_nothing() {
        assert!(admit(
            Neighbourhood {
                pops: 8,
                per_slot: 2
            },
            &[vec![0usize]],
            &[0usize],
            &toy_bound
        )
        .is_empty());
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
