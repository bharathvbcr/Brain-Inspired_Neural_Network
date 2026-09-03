# §0's citations, read against their primary sources

**Date:** 2026-09-03. **Method:** each source downloaded as a PDF, text extracted
with `pdftotext -layout`, and the cited number located in the source's own words.
Bibliographic fields come from the arXiv API and Crossref, not from recollection.
Every consulted file is hashed below so this check can be re-run against the same
bytes. The gate is [`scripts/check_section0_citations.py`](../scripts/check_section0_citations.py).

**Why this document exists.** §0's literature positioning was assembled by a
search pass on 2026-08-27 and carried a standing disclosure that none of it had
been read. Every SHD number elsewhere in this paper is machine-checked against
cells on disk; §0's were the only load-bearing numbers in the manuscript with no
check at all, which is exactly the asymmetry the disclosure admitted. **Three
claims were wrong.** They are corrected in §0 and recorded here.

---

## Sources consulted

| Key | Work | Retrieved from | sha256 |
|---|---|---|---|
| `cramer2022` | Cramer, Stradmann, Schemmel, Zenke, IEEE TNNLS **33(7):2744–2757**, July 2022 | arXiv:1910.07407v3 | `a3102636b53dfa73` |
| `yao2021tasnn` | Yao et al., TA-SNN, ICCV 2021 | arXiv:2107.11711v1 | `ccf39d3affc1d3b7` |
| `yu2022stsc` | C. Yu et al., STSC-SNN, Front. Neurosci. **16**, 2022 | arXiv:2210.05241v1 | `a18b85ae6403e777` |
| `okabe2018` | Okabe, Koshinaka, Shinoda, Interspeech 2018, **pp. 2252–2256** | arXiv:1803.10963v2 | `88b8f1d645669451` |
| `hammouamri2024dcls` | Hammouamri, Khalfaoui-Hassani, Masquelier, **ICLR 2024** | arXiv:2306.17670v3 | `ccb68bf6d385f8fe` |
| `baronig2025` | Baronig, Ferrand, Sabathiel, Legenstein, **Nat. Commun. 16(1):5776**, July 2025 | arXiv:2408.07517v3 | `1aac46f717bfa932` |
| `yu2025timing` | Z. Yu, Sun, Akarca, Goodman, preprint | arXiv:2507.16043v3 | `cdcccf7fc90a7986` |
| `chen2025nsa` | Neuromorphic Sequential Arena, **IJCAI 2025** | arXiv:2505.22035 | `0424ce7510852004` |
| `soydan2024s7` | Soydan, Zubić, Messikommer, Mishra, Scaramuzza, S7 | arXiv:2410.03464v1 | `4467b8cd531acc61` |
| `bittar2022` | Bittar & Garner, Front. Neurosci. **16**:865897, 2022 | PMC9479696 | *(HTML; see note)* |

Bittar & Garner was read as the publisher's HTML record rather than a PDF, so it
carries no hash here. Its two numbers are quoted from Table 5 of that record and
are independently corroborated by `baronig2025` Table 2 and `hammouamri2024dcls`
Table 2, which both reproduce **94.62** for the same model.

---

## Verified as written

| §0 claim | Source's own words | Verdict |
|---|---|---|
| TA-SNN reports **91.08%** on SHD | Comparison table, "This work (SOTA) TA-SNN 91.08"; Table 7 gives 91.08 at dt=60 ms, T=10 | **CONFIRMED** |
| TA-SNN is squeeze-and-excitation attention over time | "Inspired by squeeze-and-excitation (SE) block[13], we design the…"; the paper applies "the squeeze step and excitation step" | **CONFIRMED** |
| STSC-SNN reports **92.36%** on SHD | "we acquire 92.36% accuracy on SHD" | **CONFIRMED** |
| Okabe et al. is attentive statistics pooling, Interspeech 2018 | Title verbatim; DOI 10.21437/Interspeech.2018-993 | **CONFIRMED** |
| Cramer et al. built spike-count-only variants that **could not exceed 60%** on SHD | "both linear and nonlinear classifiers trained on spike count patterns without temporal information were unable to surpass the 60 % accuracy mark for the SHD" | **CONFIRMED** |
| Cramer et al.'s best-effort recurrent baseline is **83.2 ± 1.3%** at 1024 neurons with augmentation | "(83.2 ± 1.3) % on the SHD test set (Table 1)"; Table 1 note 2 reads "1024 neurons + data augmentation" | **CONFIRMED** |
| Cramer et al.: 1024 neurons → **76.5%** | "improved validation and test accuracy up to (76.5 ± 1.0) % with 1024 neurons" | **CONFIRMED** |
| Cramer et al. is **IEEE TNNLS 33(7), 2022** | Crossref on DOI 10.1109/TNNLS.2020.3044364: vol. 33, issue 7, pp. 2744–2757, July 2022 | **CONFIRMED** |
| NSA removes temporal processing model-side; SHD falls **86.48 → 68.51** | Table S4: STBP 86.48, NoTD 68.51 (−17.97); NoTD "removes temporal propagation in both forward and" backward passes | **CONFIRMED** (the table is supplementary, and §0 now says so) |
| Bittar & Garner: 3×128 → 3×1024 improves **92.88 → 94.62** | Table 5, RadLIF, recurrent: 3 layers × 128 → 92.88; 3 layers × 1024 → 94.62 | **CONFIRMED** |
| Baronig et al. report **95.81 ± 0.56** validating on test and **93.79 ± 0.76** with a held-out split | Table 2: SHD SE-adLIF (2 layers) 95.81 ± 0.56; SHD\* 93.79 ± 0.76. The phrase "to ensure comparability" is the paper's own | **CONFIRMED** |

## Corrected

**1. The frontier band's upper bound was wrong and unsourced.** §0 read
"The SHD frontier is 95–96.4%". No source §0 cites reports 96.4 on SHD, and the
sources it names top out at **95.81**. The highest SHD accuracy this check could
verify against a primary is **96.3%** (S7, arXiv:2410.03464: "S7 achieved an
accuracy of 96.3%"). 96.4 appears in the literature as an **S-MNIST** result, not
an SHD one. The band is now stated as **95.07–96.3** with both endpoints named.

**2. The band's attribution was incomplete at its top.** §0 credited the frontier
to "learned delays (DCLS), adaptation (SE-adLIF), and spiking transformers".
DCLS's own paper reports **95.07 ± 0.24**, SE-adLIF **95.81 ± 0.56** — so neither
reaches the top of the band. The top is held by a **state-space model** whose SHD
entry uses data augmentation and continuously-valued output rather than spiking
output. "Spiking transformers" carried **no citation at all** and is withdrawn:
this check found no primary for it.

**3. Yu et al. (2025) was misdescribed, and correcting it helps this paper.**
§0 listed it as the third of "three independent destruction operators, one
conclusion". Its time reversal is not a third vote for order-dependence. Their
reversal "perturb[s] spatio-temporal spike patterns but leave[s] interspike
intervals and coincidence information unchanged" — the same information-preserving
property wave 24 built its own `reversed` control to have — and their finding is
that **"when axonal delays are not used, networks perform well under time
reversal, whereas networks trained with delays perform poorly."** This instrument
carries no delays and no temporal kernel, and its reversal difference-in-differences
is flat at all three points measured (−0.0222, −0.0040, −0.0099). So Yu et al. is
**external corroboration of §3.5's reversal control**, not prior work establishing
the shuffle result, and §0 now says that instead.

## Downgraded to the instrument's own record

§0 attributed "**8,156 training samples**" to Cramer et al. The arXiv version
states 10,420 total recordings and describes the split in prose; it does not print
8,156. The number does not need a citation: every cell record in this campaign
carries `"n_train": 8156, "n_test": 2264` as read from the dataset on disk, so it
is a **tier-A** fact about what this instrument trained on, and §0 now cites the
cells.

Relatedly, §0 said Cramer et al. "document [overfitting] as severe". They document
it repeatedly — a linear SVM "readily overfitted the data in the case of SHD",
"both kernel machines and LSTMs were affected by overfitting", and noise injection
"was effective in decreasing overfitting" — but "severe" is this paper's word, not
theirs. §0 now reports what they did rather than grading it.

## Still not checkable here

Four numbers encountered during the 2026-08-27 search pass came only from a
secondary comparison table — **Pfa-SNN 96.26, Event-SSMA 95.90, SpikeSCR 95.60,
d-cAdLIF 94.85**. They were excluded then and remain excluded now: this pass did
not retrieve their primaries either. None of them is used in any claim.
