# Brand assets

**Mark:** Dendritic $k$-WTA & somatic spike · **Palette:** Dendritic Rose.

Four dendritic branches ($K=4$ compartments in `binn-engine::Cell`) enter from
the left: two remain muted under $k$-WTA lateral inhibition while two carry
staggered presynaptic spikes ($\Delta t$ for local STDP eligibility) into the
somatic comparator ($\theta(t)$), emitting a single discrete action-potential
spike to the right. Same plate geometry (`128×128` viewBox, rounded plate
`x=6..122, rx=26`) as sibling crates [tessl](https://github.com/bharathvbcr/tessl)
and [sparsl](https://github.com/bharathvbcr/sparsl).

| | |
| --- | --- |
| Base | Deep Dendritic Plum `#11070D` |
| Border & Soma Ring | `#3D162C` · `#1B0A14` |
| Active Dendrites | Rose `#EC4899` · Coral `#FB7185` |
| Peak Spike & Soma Core | Rose-White `#FFF0F7` |
| Wordmark on dark | Soft Rose `#FCE7F3` · on light: `#11070D` |

## Files

```
logo-mark.svg                                vector source (128×128)
png/logo-mark@{1024,512,256,128,64,32}.png   icon / avatar / favicon
png/logo@{2048,900}.png                      wordmark lockup for light pages
png/logo-dark@{2048,900}.png                 wordmark lockup for dark pages
build_logo.py                                regenerates every PNG above
```

```bash
python3 build_logo.py
```
