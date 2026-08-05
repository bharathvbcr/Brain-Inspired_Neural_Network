#!/usr/bin/env python3
from pathlib import Path
import re

path = Path("binn-lab/experiments/c1.rs")
text = path.read_text()


def must_replace(old: str, new: str, label: str) -> None:
    global text
    if old not in text:
        raise SystemExit(f"missing: {label}")
    text = text.replace(old, new, 1)
    print("ok", label)


must_replace(
    """        if config.is_structured_fb_teach_protocol() {
            " (structured B × target teach; does not remassage v15 / reopen v2)"
        } else if config.is_elig_rfb_protocol() {""",
    """        if config.is_structured_fb_cont_protocol() {
            " (continuous structured B; does not remassage v15 / reopen v2)"
        } else if config.is_structured_fb_finth_protocol() {
            " (finite-theta under SFB; does not remassage v15 / reopen v2)"
        } else if config.is_structured_fb_soft_protocol() {
            " (soft-WTA x structured B; does not remassage v15 / reopen v2)"
        } else if config.is_dfa_live_protocol() {
            " (live graded-DFA transfer; does not remassage matched-dfa / reopen v2)"
        } else if config.is_structured_fb_teach_protocol() {
            " (structured B × target teach; does not remassage v15 / reopen v2)"
        } else if config.is_elig_rfb_protocol() {""",
    "banner",
)

must_replace(
    """        let default_name = if config.is_structured_fb_teach_protocol() {
            if config.quick {
                "results/c1_sfb_teach_quick.md"
            } else {
                "results/c1_sfb_teach.md"
            }
        } else if config.is_elig_rfb_protocol() {""",
    """        let default_name = if config.is_structured_fb_cont_protocol() {
            if config.quick {
                "results/c1_sfb_cont_quick.md"
            } else {
                "results/c1_sfb_cont.md"
            }
        } else if config.is_structured_fb_finth_protocol() {
            if config.quick {
                "results/c1_sfb_finth_quick.md"
            } else {
                "results/c1_sfb_finth.md"
            }
        } else if config.is_structured_fb_soft_protocol() {
            if config.quick {
                "results/c1_sfb_soft_quick.md"
            } else {
                "results/c1_sfb_soft.md"
            }
        } else if config.is_dfa_live_protocol() {
            if config.quick {
                "results/c1_dfa_live_quick.md"
            } else {
                "results/c1_dfa_live.md"
            }
        } else if config.is_structured_fb_teach_protocol() {
            if config.quick {
                "results/c1_sfb_teach_quick.md"
            } else {
                "results/c1_sfb_teach.md"
            }
        } else if config.is_elig_rfb_protocol() {""",
    "default_name",
)

m = re.search(
    r"\|\| config\.is_structured_fb_teach_protocol\(\)\s*\n\s*\{\s*\n\s*println!\(\s*\n\s*\"note: project[^\"]+\"\s*\n\s*\);\s*\n\s*\}",
    text,
)
if not m:
    raise SystemExit("note block missing")
text = (
    text[: m.start()]
    + """|| config.is_structured_fb_teach_protocol()
        || config.is_dfa_live_protocol()
        || config.is_structured_fb_soft_protocol()
        || config.is_structured_fb_finth_protocol()
        || config.is_structured_fb_cont_protocol()
    {
        println!(
            "note: project/spike/isolation/sensitivity/reinforce-fb/rfb-epoch/sfb*/elig-rfb/dfa-live results do not reopen protocol-v2 hash c1-118207fbc3eaba53"
        );
    }"""
    + text[m.end() :]
)
print("ok note")

must_replace(
    """struct ProtocolFlags<'a> {
    reinforce_fb: bool,
    rfb_epoch: bool,
    structured_fb: bool,
    structured_fb_epoch: bool,
    structured_fb_capacity: bool,
    elig_rfb: bool,
    structured_fb_teach: bool,
    project: bool,
    spike_s: bool,
    spike: bool,
    isolation: bool,
    sensitivity: Option<&'a str>,
}""",
    """struct ProtocolFlags<'a> {
    reinforce_fb: bool,
    rfb_epoch: bool,
    structured_fb: bool,
    structured_fb_epoch: bool,
    structured_fb_capacity: bool,
    elig_rfb: bool,
    structured_fb_teach: bool,
    dfa_live: bool,
    structured_fb_soft: bool,
    structured_fb_finth: bool,
    structured_fb_cont: bool,
    project: bool,
    spike_s: bool,
    spike: bool,
    isolation: bool,
    sensitivity: Option<&'a str>,
}""",
    "ProtocolFlags",
)

must_replace(
    """            ProtocolFlags {
                reinforce_fb,
                rfb_epoch,
                structured_fb,
                structured_fb_epoch,
                structured_fb_capacity,
                elig_rfb,
                structured_fb_teach,
                project,
                spike_s,
                spike,
                isolation,
                sensitivity: sensitivity.as_deref(),
            },""",
    """            ProtocolFlags {
                reinforce_fb,
                rfb_epoch,
                structured_fb,
                structured_fb_epoch,
                structured_fb_capacity,
                elig_rfb,
                structured_fb_teach,
                dfa_live,
                structured_fb_soft,
                structured_fb_finth,
                structured_fb_cont,
                project,
                spike_s,
                spike,
                isolation,
                sensitivity: sensitivity.as_deref(),
            },""",
    "ProtocolFlags ctor",
)

must_replace(
    """    if flags.structured_fb_teach && !c.is_structured_fb_teach_protocol() {
        return Err(format!(
            "--structured-fb-teach refuses non-c1-sfb-teach config-hash `{h}` — \\
             use a structured-teach hash or omit --config-hash"
        ));
    }
    if flags.project && !c.is_project_protocol() {""",
    """    if flags.structured_fb_teach && !c.is_structured_fb_teach_protocol() {
        return Err(format!(
            "--structured-fb-teach refuses non-c1-sfb-teach config-hash `{h}` — \\
             use a structured-teach hash or omit --config-hash"
        ));
    }
    if flags.dfa_live && !c.is_dfa_live_protocol() {
        return Err(format!(
            "--dfa-live refuses non-c1-dfa-live config-hash `{h}` — \\
             use a live graded-DFA hash or omit --config-hash"
        ));
    }
    if flags.structured_fb_soft && !c.is_structured_fb_soft_protocol() {
        return Err(format!(
            "--structured-fb-soft refuses non-c1-sfb-soft config-hash `{h}` — \\
             use a soft-WTA structured-B hash or omit --config-hash"
        ));
    }
    if flags.structured_fb_finth && !c.is_structured_fb_finth_protocol() {
        return Err(format!(
            "--structured-fb-finth refuses non-c1-sfb-finth config-hash `{h}` — \\
             use a finite-theta SFB hash or omit --config-hash"
        ));
    }
    if flags.structured_fb_cont && !c.is_structured_fb_cont_protocol() {
        return Err(format!(
            "--structured-fb-cont refuses non-c1-sfb-cont config-hash `{h}` — \\
             use a continuous structured-B hash or omit --config-hash"
        ));
    }
    if flags.project && !c.is_project_protocol() {""",
    "validate",
)

must_replace(
    """    for (flag, name) in [
        (rfb_epoch, "--rfb-epoch"),
        (structured_fb, "--structured-fb"),
        (structured_fb_epoch, "--structured-fb-epoch"),
        (structured_fb_capacity, "--structured-fb-capacity"),
        (elig_rfb, "--elig-rfb"),
        (structured_fb_teach, "--structured-fb-teach"),
    ] {""",
    """    for (flag, name) in [
        (rfb_epoch, "--rfb-epoch"),
        (structured_fb, "--structured-fb"),
        (structured_fb_epoch, "--structured-fb-epoch"),
        (structured_fb_capacity, "--structured-fb-capacity"),
        (elig_rfb, "--elig-rfb"),
        (structured_fb_teach, "--structured-fb-teach"),
        (dfa_live, "--dfa-live"),
        (structured_fb_soft, "--structured-fb-soft"),
        (structured_fb_finth, "--structured-fb-finth"),
        (structured_fb_cont, "--structured-fb-cont"),
    ] {""",
    "conflict loop",
)

marker = '         Optional Tier-B sensitivities (protocol v3; new hashes):\\n\\'
idx = text.find(marker)
if idx < 0:
    raise SystemExit("usage marker missing")
insert = (
    "         Live graded-DFA transfer (protocol v20; new c1-dfa-live* hash):\\n\\\n"
    "           c1 --dfa-live [--quick] [--out results/c1_dfa_live.md]\\n\\\n"
    "         Graded error x FixedRandomFeedback on muted-theta/k-WTA C1.\\n\\\n"
    "         \\n\\\n"
    "         Soft-WTA x structured B (protocol v21; new c1-sfb-soft* hash):\\n\\\n"
    "           c1 --structured-fb-soft [--quick] [--out results/c1_sfb_soft.md]\\n\\\n"
    "         One disclosed temperature T=1.0; no grid.\\n\\\n"
    "         \\n\\\n"
    "         Matched undertrain adversarial (protocol v22):\\n\\\n"
    "           c1 --matched-arch --match-undertrain [--quick] [--out results/c1_match_ep4.md]\\n\\\n"
    "         4x epochs on matched broadcast three-factor; new c1-match-* hash.\\n\\\n"
    "         \\n\\\n"
    "         Finite-theta under SFB (protocol v23; new c1-sfb-finth* hash):\\n\\\n"
    "           c1 --structured-fb-finth [--quick] [--out results/c1_sfb_finth.md]\\n\\\n"
    "         Mute off under structured B; may INVALID_HARNESS.\\n\\\n"
    "         \\n\\\n"
    "         Continuous structured B (protocol v24; new c1-sfb-cont* hash):\\n\\\n"
    "           c1 --structured-fb-cont [--quick] [--out results/c1_sfb_cont.md]\\n\\\n"
    "         L2-normalized B proportional to (w1-w0); one construction.\\n\\\n"
    "         \\n\\\n"
)
text = text[:idx] + insert + text[idx:]
print("ok usage")

path.write_text(text)
print("done")
