// Candidate aggregation FORMS.  Each is a general re-expression of how the
// seven axis values are combined.  None reads a phrase, a word, a substring, a
// dictionary or an environment value: each is a function of the axis vector
// and (where the form needs a scale) of the pool's own per-axis statistics,
// which the production code already has per target.
import { W } from './forms.mjs';

const EPS = 1e-9;

// ---- F0 control: the production weighted sum ------------------------------
export const F0 = { name: 'F0 linear (production control)', make: () => r => W.reduce((s, w, i) => s + w * r.v[i], 0) };

// ---- F1 within-target z-normalisation, mass preserved ---------------------
// The item's hypothesis: a saturated axis (zero in-band variance) wastes its
// weight mass.  z-scoring every axis against the pool's own mean/sd makes a
// zero-variance axis contribute exactly 0 to every candidate, so its mass
// reverts to the axes that discriminate.  Affinely mapped back onto the
// production score's own [min,max] range so the result is in [0,1] and
// comparable with the printed scores.
export const F1 = {
  name: 'F1 within-target z-normalisation (mass preserved)',
  make(pool) {
    const mu = [0, 1, 2, 3, 4, 5, 6].map(i => pool.reduce((s, r) => s + r.v[i], 0) / pool.length);
    const sd = [0, 1, 2, 3, 4, 5, 6].map(i => {
      const m = mu[i]; return Math.sqrt(pool.reduce((s, r) => s + (r.v[i] - m) ** 2, 0) / pool.length);
    });
    const f = r => { let s = 0; for (let i = 0; i < 7; i++) s += W[i] * (sd[i] > EPS ? (r.v[i] - mu[i]) / sd[i] : 0); return s; };
    // affine map to [0,1] over the pool's own score range
    const vals = pool.map(f); const lo = Math.min(...vals), hi = Math.max(...vals);
    return r => { const x = f(r); return hi > lo ? (x - lo) / (hi - lo) : 0; };
  },
};

// ---- F2 within-target rank percentile ---------------------------------------
// The rank form of F1.  Immune to outliers and to the shape of each axis's
// distribution, which z-scoring is not.  A constant axis has every candidate
// at the same percentile, so it contributes its weight times a common constant
// and cancels in every comparison - the same cancellation F1 gets, reached
// without assuming any distribution.
export const F2 = {
  name: 'F2 within-target rank percentile',
  make(pool) {
    const pct = [0, 1, 2, 3, 4, 5, 6].map(i => {
      const a = pool.map(r => r.v[i]).sort((x, y) => x - y);
      return v => { let lo = 0, hi = a.length; while (lo < hi) { const m = (lo + hi) >> 1; if (a[m] < v) lo = m + 1; else hi = m; } return lo / (a.length - 1); };
    });
    return r => W.reduce((s, w, i) => s + w * pct[i](r.v[i]), 0);
  },
};

// ---- F3 geometric / soft-min ------------------------------------------------
// Sum -> product with the weights as exponents.  A sum lets a strong axis
// buy off a collapsed one; a product does not, which is the "stop one bad axis
// from being masked" property in its strongest form.  Reported in [0,1] by the
// production bound (each axis is in [0,1] and the weights sum to 1, so the
// weighted geometric mean of values in [0,1] is in [0,1]).
export const F3 = {
  name: 'F3 weighted geometric mean (soft-min)',
  make: () => r => {
    let lp = 0;
    for (let i = 0; i < 7; i++) {
      const v = i === 6 ? (1 - r.v[6]) : r.v[i];       // CLOSED is a penalty: use its complement
      lp += W[i] * Math.log(Math.max(v, 1e-6));
    }
    return Math.exp(lp);
  },
};

// ---- F4 tiered / gated ------------------------------------------------------
// A strong content-word signal dominates a saturated one.  Implemented as a
// hard gate: the score is the weighted sum restricted to axes that actually
// vary on this target, times a lexicographic gate on the highest-priority
// varying axis.  Gate = a step at the pool median of the primary axis, so the
// form is scale-free and needs no tuned constant.
export const F4 = {
  name: 'F4 tiered: primary varying axis gates, the rest break ties',
  make(pool) {
    const sd = [0, 1, 2, 3, 4, 5, 6].map(i => {
      const m = pool.reduce((s, r) => s + r.v[i], 0) / pool.length;
      return Math.sqrt(pool.reduce((s, r) => s + (r.v[i] - m) ** 2, 0) / pool.length);
    });
    const live = [0, 1, 2, 3, 4, 5, 6].filter(i => sd[i] > 1e-6);
    // fixed, input-independent priority order (documented, not tuned)
    const PRIO = [0, 3, 2, 5, 1, 4, 6];
    const primary = PRIO.find(i => live.includes(i));
    const rest = PRIO.filter(i => i !== primary && live.includes(i));
    const dead = [0, 1, 2, 3, 4, 5, 6].filter(i => !live.includes(i));
    const a = pool.map(r => r.v[primary]).sort((x, y) => x - y);
    const med = a[a.length >> 1];
    return r => {
      const gate = r.v[primary] >= med ? 1 : 0;
      let s = 0;
      for (const i of rest) s += W[i] * r.v[i];
      for (const i of dead) s += W[i] * r.v[i];
      // the gate owns the whole top of the range; the rest orders within it
      return gate * (0.5 + 0.5 * s / (1.0 + 1e-9)) + 0.0 * 0 + gate * 0;
    };
  },
};

// ---- F5 saturated-axis masking with NO rescaling of the live axes ------------
// The minimal form of the item's hypothesis: delete the mass of every axis
// that is constant on this target and renormalise the survivors.  Unlike F1 it
// does not rescale the surviving axes, so it isolates exactly one effect: how
// much of the 0.0958 was mass sitting on a saturated axis.
export const F5 = {
  name: 'F5 saturated-axis masking (drop dead mass, renormalise live axes)',
  make(pool) {
    const sd = [0, 1, 2, 3, 4, 5, 6].map(i => {
      const m = pool.reduce((s, r) => s + r.v[i], 0) / pool.length;
      return Math.sqrt(pool.reduce((s, r) => s + (r.v[i] - m) ** 2, 0) / pool.length);
    });
    const live = [0, 1, 2, 3, 4, 5, 6].filter(i => sd[i] > 1e-6);
    const tot = live.reduce((s, i) => s + Math.abs(W[i]), 0);
    return r => live.reduce((s, i) => s + (W[i] / tot) * r.v[i], 0);
  },
};

// ---- F6 hard gate on the content-word signal --------------------------------
// "A saturated axis masking a real signal" taken at its most literal: if the
// clue is a genuine resegmentation (no target word reused) it competes, else it
// does not.  Threshold 1.0 is the axis's own range, not a tuned constant.
export const F6 = {
  name: 'F6 hard gate on WORD_NOVELTY (full-resegmentation requirement)',
  make: () => r => (r.v[2] >= 1.0 - 1e-9 ? 1 : 0) * 0.5 + 0.5 * W.reduce((s, w, i) => s + w * r.v[i], 0),
};

export const FORMS = [F0, F1, F2, F3, F4, F5, F6];
