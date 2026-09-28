# Loose blob content recovered - 2026-09-28

Branch `recovery/loose-blob-content-2026-09-28`. Recovered by coordinator pass `coord-5e3a`.

## What this is

`git fsck --unreachable --dangling` reports **205 unreachable commits, 398 unreachable
trees and 227 unreachable blobs** on this repository. Every earlier pass (rules 13 and 28)
classified the unreachable set by walking **commits** - `git diff-tree` and
`git ls-tree -r <commit>` - and `coord-9c31` concluded that 39 of them carried unique
content, archived as `recovery/unreachable-merge-content-2026-09-28` (`134c0ed`). A loose
blob that belongs to no commit is invisible to both spellings: `rev-list` and
`diff-tree` are commit-rooted.

**185 blobs** are held by *nothing*: no commit, no ref, no reflog
(`git rev-list --objects --all --reflog` does not contain them), and no remote head (the
durable set of 5,576 objects over the 188 fetched `refs/remotes/audit/*` tips does not
contain them). They are the most fragile object class this repository has produced, and
`git gc` prunes them without warning. All 185 are archived here except the two oversize
binaries noted below.

## Composition

| kind | count | bytes |
|---|---|---|
| prof harness output (results/scale dump) | 8 | 129,865 |
| instrumented src/lib.rs copy | 110 | 20,590,048 |
| other text | 65 | 2,464,207 |
| SKIPPED - 30 MB instrumented ELF binary, deliberately not archived | 2 | 60,240,720 |

Total across the 185: **83,424,840 bytes**. The two excluded blobs are
the `madgab-approx-runtime/prof/madgab-{baseline,prof}` instrumented binaries, 30,129,432
and 30,111,288 bytes. They are **deliberately not archived**: prior passes established
that `prof/run.sh` plus `prof/summarize.py` and all three harness inputs are durable at
`docs/work/probe-inputs/`, so the binaries are regenerable, and 60 MB of ELF output is
exactly the bulk rule 41 says never to push.

The **instrumented `src/lib.rs` copies** are the material item. Several passes recorded as
a standing loose end that `prof/README.md` documents a real `prune_partials` change
(cache `metrics` instead of recomputing per comparison) that existed in no branch and no
commit. It is in this set.

## Verification

Each archived file was re-hashed with `git hash-object` and every one reproduced its blob
sha (183 ok, 0 mismatches). The recovery was run with the input bracketed per rules 35 and
37: 227 blob-class entries in, 42 already durable on remote refs, 185 written, 2 skipped
by the size rule.

## Fence

These files are `src/lib.rs` copies from measurement probes. Several will contain canonical
phrase literals. They sit under `docs/`, which `tests/no_phrase_hard_coding.rs` does not
scan, and `ALLOWLIST_CAPS` is unchanged. **If any of this is ever promoted, its phrase
literals must be removed as part of that promotion, not waived.**

## Reproduction

```sh
git fetch origin "+refs/heads/*:refs/remotes/audit/*"
REFS=$(git for-each-ref refs/remotes/audit/ --format="%(refname)")
git rev-list --objects $REFS | awk '{print $1}' | sort -u > durable.txt
git fsck --unreachable --dangling          # blob-class lines only
# field-1 membership against durable.txt, both sides sort -u (rules 17 and 22)
git for-each-ref --format="%(objectname)" refs/remotes/audit/ | wc -l   # expect 188
```

## Files

| blob | bytes | kind |
|---|---|---|
| `0243f828b936` | 14,858 | prof harness output (results/scale dump) |
| `02d230273d86` | 200,015 | instrumented src/lib.rs copy |
| `02ecde9d059e` | 22 | other text |
| `0343b27c0ee9` | 192,454 | instrumented src/lib.rs copy |
| `0396d5b2d8b1` | 24,681 | other text |
| `049801cd2282` | 214,048 | instrumented src/lib.rs copy |
| `0540cbecb898` | 336,381 | instrumented src/lib.rs copy |
| `05bcd5bf154d` | 12,157 | other text |
| `05c7e78ba65f` | 12,125 | other text |
| `05f4d06b4d43` | 27,856 | other text |
| `08d314ee2529` | 181,459 | instrumented src/lib.rs copy |
| `0d724a051a80` | 23,936 | other text |
| `105d9b3458d9` | 248,872 | instrumented src/lib.rs copy |
| `11388b7ee8b9` | 135,147 | instrumented src/lib.rs copy |
| `12e8d6f747ca` | 184,124 | instrumented src/lib.rs copy |
| `168607b20681` | 5,866 | other text |
| `1b602fc36b13` | 18,720 | prof harness output (results/scale dump) |
| `1b7147f73ea2` | 131,243 | instrumented src/lib.rs copy |
| `1c11b4e00405` | 8,706 | other text |
| `1ef98e850611` | 220,032 | instrumented src/lib.rs copy |
| `203db548b87c` | 217,372 | instrumented src/lib.rs copy |
| `209c7e19ecef` | 163,088 | instrumented src/lib.rs copy |
| `22aa194353e0` | 25 | other text |
| `24753c126de1` | 30,129,432 | SKIPPED - 30 MB instrumented ELF binary, deliberately not archived |
| `26d9d0aff27a` | 172,221 | instrumented src/lib.rs copy |
| `279b3c1ae846` | 190,607 | instrumented src/lib.rs copy |
| `292540ceab35` | 133,903 | instrumented src/lib.rs copy |
| `2aa68a547bb9` | 9,759 | other text |
| `2c6349d15d43` | 30,320 | other text |
| `31c3a535e56b` | 223,927 | instrumented src/lib.rs copy |
| `33796f9a4b33` | 314,150 | instrumented src/lib.rs copy |
| `33a4cc05ebb5` | 303,533 | instrumented src/lib.rs copy |
| `36fdde2eb9bc` | 196,027 | instrumented src/lib.rs copy |
| `3861fde83682` | 33,424 | other text |
| `3892c535130e` | 196,171 | instrumented src/lib.rs copy |
| `389a8b54ca6e` | 40,930 | other text |
| `38dc0f165326` | 195,667 | instrumented src/lib.rs copy |
| `3b8c5630585c` | 27,866 | other text |
| `3f34f6f59b8d` | 209,918 | instrumented src/lib.rs copy |
| `4029be173dfd` | 234,065 | instrumented src/lib.rs copy |
| `403bfbbb0d4a` | 17,015 | other text |
| `430aa65af2a9` | 194,853 | instrumented src/lib.rs copy |
| `440ea566f7b2` | 211,055 | instrumented src/lib.rs copy |
| `4601dc6e8685` | 161,844 | instrumented src/lib.rs copy |
| `463b5b8b5d32` | 184,100 | instrumented src/lib.rs copy |
| `46ec942d0bb2` | 200,813 | instrumented src/lib.rs copy |
| `483a6058203e` | 183,771 | instrumented src/lib.rs copy |
| `4a49f834febc` | 155,497 | instrumented src/lib.rs copy |
| `4de374621c48` | 153,910 | instrumented src/lib.rs copy |
| `5062dc86d7bd` | 192,125 | instrumented src/lib.rs copy |
| `50d0d035ecc7` | 165,784 | instrumented src/lib.rs copy |
| `510b6804d4e3` | 22,970 | other text |
| `51d364c48e49` | 195,249 | instrumented src/lib.rs copy |
| `52a6aabe22c5` | 186,369 | instrumented src/lib.rs copy |
| `52db9f7fd71b` | 142,666 | instrumented src/lib.rs copy |
| `542ab78b593b` | 14,831 | other text |
| `5532bd0f5b4f` | 203,449 | instrumented src/lib.rs copy |
| `579079494ba5` | 155,154 | instrumented src/lib.rs copy |
| `57bdd2722d6f` | 116,715 | other text |
| `587d0987e458` | 7,744 | other text |
| `5c5036a7eca7` | 30,762 | other text |
| `5d35dfb2e3a9` | 213,988 | instrumented src/lib.rs copy |
| `5eac2144ea31` | 235,442 | other text |
| `5ef8dee825cc` | 194,920 | instrumented src/lib.rs copy |
| `5f03a28dccb2` | 141,093 | instrumented src/lib.rs copy |
| `60a431b12b9a` | 157,408 | instrumented src/lib.rs copy |
| `60ebfbe498fa` | 28,166 | other text |
| `619bf1e31a6e` | 159,677 | instrumented src/lib.rs copy |
| `62d636a3a1c9` | 217,906 | instrumented src/lib.rs copy |
| `64b395ba5fbe` | 223,912 | instrumented src/lib.rs copy |
| `6986a80572d7` | 20,798 | other text |
| `698992843e62` | 15,956 | prof harness output (results/scale dump) |
| `6b23055b704c` | 158,652 | instrumented src/lib.rs copy |
| `705c0fa6fff5` | 26,688 | other text |
| `70c9f72742e2` | 23,792 | other text |
| `70f28b7ba6a5` | 37,194 | other text |
| `7135880f51b1` | 15,955 | prof harness output (results/scale dump) |
| `741f4f988d46` | 304,313 | instrumented src/lib.rs copy |
| `74d0256090f1` | 28,941 | other text |
| `7796cf9ed2fd` | 141,422 | instrumented src/lib.rs copy |
| `77ef786ffd60` | 183,099 | instrumented src/lib.rs copy |
| `77ff719acf45` | 24,651 | other text |
| `7a4c52de6c07` | 11,863 | instrumented src/lib.rs copy |
| `7b0c0f8b422f` | 166,809 | instrumented src/lib.rs copy |
| `8071780cad1b` | 159,046 | instrumented src/lib.rs copy |
| `815d1df5c521` | 163,016 | instrumented src/lib.rs copy |
| `8c2f96c50c9e` | 18,665 | other text |
| `8cca45bb7499` | 20,247 | other text |
| `8cdb68ca82fc` | 215,375 | instrumented src/lib.rs copy |
| `8e2ff2cc8275` | 198,594 | instrumented src/lib.rs copy |
| `8e94e5469bf6` | 185,344 | instrumented src/lib.rs copy |
| `8f69ffd9bb13` | 12,489 | other text |
| `9138dd610090` | 15,956 | prof harness output (results/scale dump) |
| `932f9ce748d8` | 212,471 | instrumented src/lib.rs copy |
| `94d6cc75493a` | 271,536 | instrumented src/lib.rs copy |
| `951e7bf00cb2` | 164,113 | instrumented src/lib.rs copy |
| `96cf46d77d2b` | 33,010 | other text |
| `97ba680f43b7` | 198,591 | instrumented src/lib.rs copy |
| `983a9e3fb50f` | 15,955 | prof harness output (results/scale dump) |
| `985cc78f47c2` | 190,211 | instrumented src/lib.rs copy |
| `99aec42558b7` | 10,945 | other text |
| `9ba9a72e97e2` | 181,526 | instrumented src/lib.rs copy |
| `9e3fa0d166c1` | 194,723 | instrumented src/lib.rs copy |
| `9fab8c4f9038` | 196,493 | instrumented src/lib.rs copy |
| `9fc8e034ce14` | 325,127 | instrumented src/lib.rs copy |
| `a29d95c9d9f8` | 18,982 | other text |
| `a43662d287b3` | 31,006 | other text |
| `a4a4415b94d6` | 198,240 | instrumented src/lib.rs copy |
| `a593efa36bf0` | 13,232 | other text |
| `a657a4d313f2` | 18,174 | other text |
| `a65bf10326fa` | 14,642 | instrumented src/lib.rs copy |
| `a70aa6560bbd` | 239,262 | instrumented src/lib.rs copy |
| `a76fcee55a41` | 164,211 | instrumented src/lib.rs copy |
| `aad1cab896ff` | 9,300 | instrumented src/lib.rs copy |
| `aad696a0d621` | 30,111,288 | SKIPPED - 30 MB instrumented ELF binary, deliberately not archived |
| `adcc00497a70` | 33,576 | other text |
| `add740517257` | 305,050 | instrumented src/lib.rs copy |
| `af4286c1bd41` | 12,314 | other text |
| `af8d316e0106` | 198,018 | instrumented src/lib.rs copy |
| `b1b7a79b443f` | 12,461 | other text |
| `b1ecc7ea9f9e` | 25,119 | instrumented src/lib.rs copy |
| `b1f960dc3694` | 15,192 | other text |
| `b2b33b44be94` | 181,855 | instrumented src/lib.rs copy |
| `b47b16588383` | 40,956 | other text |
| `b8eba7a7e9c1` | 28,169 | other text |
| `ba8ada0dec83` | 161,515 | instrumented src/lib.rs copy |
| `badf2583c287` | 13,670 | other text |
| `bca425cbdb98` | 205,694 | instrumented src/lib.rs copy |
| `bcb689417fa3` | 223,537 | instrumented src/lib.rs copy |
| `bcdee6d5b49b` | 143,691 | instrumented src/lib.rs copy |
| `bef7fdaadfa8` | 178,231 | instrumented src/lib.rs copy |
| `c1974e585a19` | 157,079 | instrumented src/lib.rs copy |
| `c223c1a8a390` | 29,702 | other text |
| `c30996f836d9` | 384,736 | instrumented src/lib.rs copy |
| `c6a8b25b2579` | 16,509 | prof harness output (results/scale dump) |
| `c6cf2506312b` | 24,682 | other text |
| `ca3e327544d1` | 231,405 | instrumented src/lib.rs copy |
| `cf4038249864` | 32,305 | other text |
| `cf7f35a12257` | 21,631 | other text |
| `cf95e237df19` | 196,766 | instrumented src/lib.rs copy |
| `d2e66f8252ad` | 156,179 | instrumented src/lib.rs copy |
| `d3c4b98add4a` | 211,328 | instrumented src/lib.rs copy |
| `d6b1f149dab0` | 242,682 | instrumented src/lib.rs copy |
| `d71b2c7f31ed` | 209,811 | instrumented src/lib.rs copy |
| `d9b167b85d51` | 130,631 | other text |
| `da1c957f64a2` | 15,956 | prof harness output (results/scale dump) |
| `da5e5ade8e98` | 26,426 | other text |
| `da72bb3170d8` | 12,123 | other text |
| `daf1b3e912c9` | 213,715 | instrumented src/lib.rs copy |
| `db60f7b9f40e` | 211,691 | instrumented src/lib.rs copy |
| `dc016e234066` | 192,124 | instrumented src/lib.rs copy |
| `dd633cbc7c95` | 28,192 | other text |
| `dd8732342cfd` | 30,320 | other text |
| `dd87d0027a16` | 17,818 | other text |
| `dde61a5827ab` | 193,698 | instrumented src/lib.rs copy |
| `df01f9706fa7` | 164,540 | instrumented src/lib.rs copy |
| `df236c96c062` | 100,112 | instrumented src/lib.rs copy |
| `e0387c8b0f57` | 26,574 | other text |
| `e0477a5ddfb8` | 29,830 | other text |
| `e0763899b8e4` | 28,175 | other text |
| `e2296f377af2` | 175,504 | instrumented src/lib.rs copy |
| `e51aad7634da` | 16,158 | other text |
| `e54b845c151d` | 193,971 | instrumented src/lib.rs copy |
| `e592e46c1648` | 216,843 | instrumented src/lib.rs copy |
| `e678c2c840a7` | 63,517 | other text |
| `e6b8b3fb14cd` | 183,704 | instrumented src/lib.rs copy |
| `e764b0acf35d` | 183,438 | instrumented src/lib.rs copy |
| `e9262ffd9d69` | 138,005 | instrumented src/lib.rs copy |
| `e943c60d1ff8` | 36,228 | instrumented src/lib.rs copy |
| `ea29dc7c4808` | 212,201 | instrumented src/lib.rs copy |
| `ea6e65f104ad` | 212,142 | instrumented src/lib.rs copy |
| `ebd4ab309e19` | 179,002 | instrumented src/lib.rs copy |
| `eda8bc1c30b9` | 21,601 | other text |
| `edcb646683fa` | 218,257 | instrumented src/lib.rs copy |
| `f0b22b70f598` | 13,425 | instrumented src/lib.rs copy |
| `f1311c6108db` | 133,574 | instrumented src/lib.rs copy |
| `f2a3a09afaa7` | 186,134 | instrumented src/lib.rs copy |
| `f524890de473` | 46,748 | other text |
| `f6394d13d4f7` | 14,160 | other text |
| `f6cea11c7b2b` | 580,632 | other text |
| `f6f4775a7447` | 28,347 | other text |
| `f79a5f2d897c` | 218,035 | instrumented src/lib.rs copy |
| `f801b044354b` | 209,482 | instrumented src/lib.rs copy |
| `f9bd5f751c02` | 153,581 | instrumented src/lib.rs copy |
| `fb6d346016e3` | 28,165 | other text |
