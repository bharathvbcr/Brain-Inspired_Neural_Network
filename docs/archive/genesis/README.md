# Genesis documents — archived 2026-09-05

Pre-build planning lineage and two external literature reviews, all dated
2026-08-04, all superseded by [`BINN_Agent_Build_Spec_v8.md`](../../../BINN_Agent_Build_Spec_v8.md),
which is the only specification the code cites (`README.md` §10,
`Cargo.toml` allowed-crates comment). Kept because v8 was argued into
existence by v1–v7 and the argument is not reproduced there.

Nothing here is evidence. The scientific record is `results/` (see
`results/INDEX.md`) and, for the successor line, `hybrid-results/`.

## Lineage

| v | document | what it did | what the next version found wrong |
|---|---|---|---|
| 1 | [Foundation_Reinvention_Plan.md](Foundation_Reinvention_Plan.md) | Biology-first substrate; Rust/C++ build plan | v2: an "honest re-audit" — the problem that mattered was credit assignment, not the substrate |
| 2 | [Foundation_Reinvention_v2.md](Foundation_Reinvention_v2.md) | Foundation rebuilt around credit assignment | v3: no scaling story |
| 3 | [Foundation_Scaling_Plan_v3.md](Foundation_Scaling_Plan_v3.md) | One node → trillions; LLM vs brain as target | v4: gaps v1–v3 never listed |
| 4 | [Foundation_Gap_Analysis_v4.md](Foundation_Gap_Analysis_v4.md) | Complete gap map, BINN vs ANN | v5: every gap must be owned, not inherited |
| 5 | [BINN_Build_From_Scratch_v5.md](BINN_Build_From_Scratch_v5.md) | Own the whole stack; each gap a component | v6: no executable plan |
| 6 | [BINN_Project_Plan_v6.md](BINN_Project_Plan_v6.md) | Modules, scopes, milestones, gates | v7: adversarial pass — what survives, what is overstated, what is wrong |
| 7 | [Hard_Audit_v7.md](Hard_Audit_v7.md) | Audit of v1–v6 claims | v8: the spec, with preregistered kill-gates; the crux (G2) later **failed** — `c1-118207fbc3eaba53` |

## External reviews (not authored here)

| document | source | note |
|---|---|---|
| [Grok_ANN_Report.md](Grok_ANN_Report.md) | Grok, 2026-08-04 | ANN vs brain literature review; input to v4 |
| [chatgpt_ANN_Report.md](chatgpt_ANN_Report.md) | ChatGPT, 2026-08-04 | Same brief; input to v4 |

Chatbot-generated. Citations inside were not verified at the time and are
not verified now. Do not cite from these; cite the primary source.

## How to read them

Read v7 first. It says which claims in v1–v6 survived, and v8 is the
version of those claims that was then tested. Cross-references between these
files are bare filenames and resolve within this directory.
