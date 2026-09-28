#!/usr/bin/env python3
"""TEMP: summarize prof/results.txt into per-target averaged phase fractions."""
import sys, collections

path = sys.argv[1] if len(sys.argv) > 1 else "prof/results.txt"
PHASES = [
    "00_from_json_total", "01_corpus_from_json", "02_build_lexicon",
    "10_gen_approx_total", "20_lattice_matches_at",
    "30_main_beam_loop",
    "31_beam_extend_fuzzy", "32_beam_overflow_prune", "33_final_prune_partials",
    "80_prune_total", "81_prune_dedup_loop", "83_prune_metrics_vector",
    "84_prune_cells_and_orders", "85_prune_final_sort_recompute",
    "90_partial_metrics", "92_boundary_novelty", "93_candidate_reuses_target", "94_familiarity_agg", "95_shape_quality_agg",
    "41_span_shortlist_loop", "51_structural_dp_loop", "50_seg_structural_dp", "60_lexical_heap_enum",
    "70_finish", "71_finish_into_clue", "72_finish_sort",
    "73_finish_dedup", "74_select_diverse",
    "A0_matches_at_total", "A1_matches_at_search_loop",
    "A2_matches_at_children_expand", "A3_matches_at_postprocess",
]
per = collections.defaultdict(lambda: collections.defaultdict(list))
stats = collections.defaultdict(lambda: collections.defaultdict(list))
cur = None
for line in open(path):
    line = line.rstrip("\n")
    if line.startswith("### "):
        cur = int(line.split("target=")[1].split()[0])
        continue
    if line.startswith("PROFSTAT "):
        kv = dict(p.split("=") for p in line.split()[1:] if "=" in p)
        for k, v in kv.items():
            stats[cur][k].append(int(v))
        continue
    if line.startswith("PROF\t"):
        _, name, secs, cnt = line.split("\t")
        per[cur][name].append((float(secs), int(cnt)))
    elif "corpus loaded" in line:
        ms = int(line.split("corpus loaded in ")[1].split("ms")[0])
        per[cur]["coarse_load_ms"].append((ms, 1))
        per[cur]["coarse_search_ms"].append(
            (int(line.split("search ")[1].split("ms")[0]), 1))

reps = max(len(v) for v in per.values() for k, v in per[1].items())
for t in sorted(per):
    tot = sum(v[0] for v in per[t]["10_gen_approx_total"]) / len(per[t]["10_gen_approx_total"])
    ld = sum(v[0] for v in per[t]["00_from_json_total"]) / len(per[t]["00_from_json_total"])
    print(f"\n=== target {t}  (load {ld:.2f}s  search {tot:.2f}s  total {ld+tot:.2f}s)  n={stats[t]['n'][0] if 'n' in stats[t] else '?'}"
          f"  lattice_entries={stats[t].get('lattice_entries',['?'])[0]}"
          f"  segmentations={stats[t].get('segmentations',['?'])[0]}"
          f"  finish_candidates={stats[t].get('finish_candidates',['?'])[0]}"
          f"  prune_calls={int(sum(per[t]['P1_prune_calls'][0][1] for _ in [0])/1)}")
    rows = []
    for p in PHASES:
        if p not in per[t]:
            continue
        secs = sum(x[0] for x in per[t][p]) / len(per[t][p])
        cnt = sum(x[1] for x in per[t][p]) / len(per[t][p])
        rows.append((secs, cnt, p))
    for secs, cnt, p in rows:
        print(f"   {p:32s} {secs:8.3f}s  {100*secs/tot:6.1f}%  calls/rep={cnt:12.1f}")
