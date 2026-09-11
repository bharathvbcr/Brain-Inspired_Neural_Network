# Visual study companion verification

Verified on 2026-09-10. These checks validate the explanatory artifacts, not new model training or biological claims.

## Interactive study lab

- Output: [study-lab.html](../study-lab.html)
- SHA-256: `5b9766ef6fb1d0e2b7b6f4befc0b017de6eccfcf80645cef7953b65d01c2a2b9`
- All 48 embedded accuracies and source-file SHA-256 values checked against the saved Wave 29 cells.
- Browser interaction checks passed: clustered versus spaced inputs; play/pause/step; five recording conditions; all 48 seed-specific chart values; mean values; the seed that misses the growth threshold; theme switching; keyboard tab navigation; reduced motion; twelve-row data table.
- All three lessons fit horizontally at 360, 390, 680, 768, 1024, 1440 and 1920 pixels: 21 viewport/lesson checks. The document intentionally scrolls vertically.
- Zero JavaScript page errors and zero external network requests during the browser test; tested from a local file.
- Perceptual review: light desktop neuron and recording lessons, dark desktop evidence lesson, and phone-width neuron/evidence screenshots inspected. Text and controls fit without visible overlap.
- Teaching animations are synthetic. Only the third lesson displays measured experiment data. Training was not rerun.

## Experiment map receipt

```text
diagram_type: workflow
output: /Users/bharath/Code/research/BINN/docs/diagrams/experiment-path.html
specification_sha256: ea374e17d1ddd6cde68b2b910b6e753d5ac4cea313306e7d174f07afe0012883
artifact_sha256: 6dbe306bfa53600ced6d9c30ea4b8abac42186727198c01abca554ada99ed6d0
validation: 9/9 showcase, 0 errors, 0 warnings
browser_evidence: passed
visual_review: passed
correction_rounds: 2
```

- Specification: [experiment-path.workflow.json](experiment-path.workflow.json), 2,304 bytes.
- Artifact: [experiment-path.html](experiment-path.html), 704,879 bytes.
- Automatic browser evidence: [receipt](experiment-path.visual-check.json) and [contact sheet](experiment-path.visual-check.html).
- No horizontal or vertical page overflow at 1440×900, 1600×1000, 1920×1080 and 2048×1320.
- Both endpoint sizes captured in light and dark themes. Perceptual review inspected the final large light and smaller dark screenshots; labels and relationships are readable and unoccluded.
- The first spacing adjustment passed artifact validation but failed browser containment (scroll heights 1425–1582 px). Reducing that offset resolved all four viewport failures; the final receipt above binds the corrected artifact.
- Deterministic delivery, automated browser evidence and perceptual review are separate checks. The automatic receipt correctly retains `visualReview: pending`; this document records the subsequent image review.
- Arrows without labels connect stages whose names already state the operation; the branch endpoints identify attention and the addition of its class scores.

The original ASCII overview remains in the written notes for text-only reading. The new files add interactive teaching views and an explorable map without changing experiment code, datasets or result records.
