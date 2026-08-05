# Invalidated BINN-Hybrid development artifacts

The following protocol-v1 development outputs are retained for provenance but
must not be used as scientific evidence:

- `h0_h1_feasibility_quick.md`
- `artifacts_quick/`

Protocol v1 had two sequencing defects:

1. it trained student artifacts before the H0 direct-teacher gate was decided;
2. its pilot evaluation used seeds from the then-proposed held-out family.

Protocol v2 fixes both defects, uses separate pilot and fresh held-out seed
families, increments the semantic protocol version, and emits no student
artifact when H0 returns `HYBRID_NO_GO`.

Protocol v3 additionally guarantees that a surrogate-only H1 result can never
emit a scientific `PROCEED` decision. The H0 measurements and `HYBRID_NO_GO`
outcome are unchanged from v2.

The authoritative current development result is:

- `binn-hybrid-h0-v3-caedeec1a47475a5.md`
