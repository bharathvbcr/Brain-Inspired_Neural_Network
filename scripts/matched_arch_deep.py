#!/usr/bin/env python3
"""Deep hypotheses on the matched-architecture forward.

Builds on scripts/matched_arch_experiments.py (imported as M). Questions:

A. DIRECTIONALITY  — is the RL reward failing for lack of *direction*, of
   *gradation*, or because Hebbian eligibility is the wrong credit? Ladder of
   reward variants that add one ingredient at a time.
B. NODE PERTURBATION — can a *scalar reward alone* teach the hidden layer locally
   via exploration/correlation, where Hebbian three-factor cannot?
C. NONLINEAR + DEPTH — does local-credit success survive a nonlinearly-separable
   temporal-XOR task and a 2-hidden-layer forward (the real BINN thesis: deep
   local credit, cf. C3 D*=3 vs 8)?
D. DEPTH_LOCALITY — compare DFA / rl_reinforce_fb to inflated readout_only under
   strong/mid/weak 2-layer init; report paired excess (see MATCHED_ARCH_DEPTH_LOCALITY.md).

All arms share ONE forward per experiment; only the update rule varies. NumPy
preview — not the Rust verdict.
"""
import argparse, json
import numpy as np
import scripts.matched_arch_experiments as M

T = M.T
ALPHA = M.ALPHA
THETA = M.THETA
sig = M.sigmoid
surr = M.surrogate
CHANCE = 0.5


# ----------------------------- tasks -----------------------------
def gen_coincidence(n, seed):
    return M.gen_examples(n, seed)


def gen_xor(n, seed):
    """Temporal XOR: each of two channels fires 'early' (t<T/2) or 'late'.
    label = early_A XOR early_B. Nonlinearly separable; readout-only must fail."""
    rng = np.random.default_rng(seed)
    X1 = np.zeros((n, T), np.float32); X2 = np.zeros((n, T), np.float32)
    Y = np.zeros(n, np.float32)
    half = T // 2
    for i in range(n):
        eA = rng.random() < 0.5; eB = rng.random() < 0.5
        tA = rng.integers(0, half) if eA else rng.integers(half, T)
        tB = rng.integers(0, half) if eB else rng.integers(half, T)
        X1[i, tA] = 1.0; X2[i, tB] = 1.0
        Y[i] = float(eA ^ eB)
    return X1, X2, Y


def gen_xor_coin(n, seed):
    """Deprecated alias — prefer gen_xor_thresh. Kept for older CLI invocations."""
    return gen_xor_thresh(n, seed, thresh=3)


def gen_xor_thresh(n, seed, thresh=3):
    """Second nonlinear task (P3): temporal XOR with a *different* early cut.

    Same Boolean structure as `gen_xor` (early_A ⊕ early_B) but early means
    `t < thresh` with default thresh=3 (vs T/2=4 in gen_xor). Confirms the
    locality flip is not idiosyncratic to one early-window definition:
    broadcast scalar error stays at chance; per-neuron DFA solves it.
    """
    rng = np.random.default_rng(seed)
    X1 = np.zeros((n, T), np.float32); X2 = np.zeros((n, T), np.float32)
    Y = np.zeros(n, np.float32)
    assert 1 <= thresh < T
    for i in range(n):
        eA = rng.random() < 0.5; eB = rng.random() < 0.5
        tA = int(rng.integers(0, thresh) if eA else rng.integers(thresh, T))
        tB = int(rng.integers(0, thresh) if eB else rng.integers(thresh, T))
        X1[i, tA] = 1.0; X2[i, tB] = 1.0
        Y[i] = float(eA ^ eB)
    return X1, X2, Y


def gen_xnor(n, seed):
    """Temporal XNOR: label = NOT (early_A XOR early_B). Boolean dual of XOR.

    Note: does *not* reproduce the locality flip (broadcast scalar also solves
    it) — useful contrast, not a confirmation of locality necessity.
    """
    X1, X2, Y = gen_xor(n, seed)
    return X1, X2, 1.0 - Y


# ----------------------------- forwards -----------------------------
# 2-layer init presets. `strong` is the P1 default that makes BPTT trainable but
# inflates readout_only (~0.78). `mid` / `weak` are depth-locality probes that
# try to bring frozen features closer to chance while keeping a live forward.
INIT_PRESETS = {
    "strong": (1.5, 1.8),   # P1 default
    "mid":    (1.0, 1.2),
    "weak":   (0.5, 1.0),   # matched 1-layer in-scale; often silent deep path
}


def init(h, seed, layers=1, init_preset="strong"):
    """Init weights. For `layers==2`, use a stronger fan-in so layer-1 spikes on
    the one-hot input frame (win≈±1.5 crosses θ=1) and layer-2 is driven —
    otherwise the deep forward is silent and BPTT stays at chance (see
    MATCHED_ARCH_DEEP_FINDINGS §D / NEXT_PLAN P1).

    `init_preset` only affects layers==2 (`strong`/`mid`/`weak`); 1-layer always
    uses in_scale=0.5 (matched-arch)."""
    rng = np.random.default_rng(seed ^ 0x5171)
    A = {"h": h, "layers": layers, "init_preset": init_preset}
    if layers == 2:
        in_scale, w12_coef = INIT_PRESETS[init_preset]
    else:
        in_scale, w12_coef = 0.5, 1.8
    A["win"] = ((rng.random((h, 2)) * 2 - 1) * in_scale).astype(np.float32)
    if layers == 2:
        # Larger inter-layer scale so L1 spikes produce L2 spikes at t≈0.
        A["w12"] = ((rng.random((h, h)) * 2 - 1) * (w12_coef / np.sqrt(h))).astype(np.float32)
    A["wout"] = ((rng.random(h) * 2 - 1) * 0.2).astype(np.float32)
    A["by"] = np.float32(0.0)
    return A


def lif_layer(cur_seq):
    """cur_seq: (B,h,T) input current -> spikes (B,h,T), u (B,h,T), rates (B,h)."""
    B, h, _ = cur_seq.shape
    u = np.zeros((B, h, T), np.float32); s = np.zeros((B, h, T), np.float32)
    for t in range(T):
        uprev = u[:, :, t - 1] if t > 0 else np.zeros((B, h), np.float32)
        sprev = s[:, :, t - 1] if t > 0 else np.zeros((B, h), np.float32)
        ui = ALPHA * uprev + cur_seq[:, :, t] - THETA * sprev
        u[:, :, t] = ui; s[:, :, t] = (ui >= THETA).astype(np.float32)
    return u, s, s.sum(axis=2)


def forward(A, X1, X2):
    B = X1.shape[0]; h = A["h"]
    cur1 = np.zeros((B, h, T), np.float32)
    for t in range(T):
        cur1[:, :, t] = np.outer(X1[:, t], A["win"][:, 0]) + np.outer(X2[:, t], A["win"][:, 1])
    u1, s1, r1 = lif_layer(cur1)
    cache = {"u1": u1, "s1": s1, "r1": r1, "X1": X1, "X2": X2}
    if A["layers"] == 2:
        # layer-1 spikes drive layer-2 (per-timestep synaptic current)
        cur2 = np.einsum("bkt,hk->bht", s1, A["w12"]).astype(np.float32)
        u2, s2, r2 = lif_layer(cur2)
        cache.update({"u2": u2, "s2": s2, "r2": r2})
        rtop = r2
    else:
        rtop = r1
    logit = A["by"] + rtop @ A["wout"]
    cache["rtop"] = rtop; cache["logit"] = logit
    return cache


def eval_acc(A, X1, X2, Y, bs=256):
    c = 0
    for i in range(0, len(Y), bs):
        lg = forward(A, X1[i:i+bs], X2[i:i+bs])["logit"]
        c += int((np.abs((sig(lg) >= .5).astype(np.float32) - Y[i:i+bs]) < .5).sum())
    return c / len(Y)


def elig_in(u1, X1, X2):
    """Input eligibility to layer-1 (B,h,2)."""
    B, h, _ = u1.shape
    ei = np.zeros((B, h, 2), np.float32)
    for t in range(T):
        sr = surr(u1[:, :, t] - THETA)
        ei[:, :, 0] = ALPHA * ei[:, :, 0] + sr * X1[:, t][:, None]
        ei[:, :, 1] = ALPHA * ei[:, :, 1] + sr * X2[:, t][:, None]
    return ei


def elig_layer(u_post, s_pre):
    """Inter-layer eligibility: post surrogate × pre spikes → (B,h_post,h_pre)."""
    B, h, _ = u_post.shape
    e = np.zeros((B, h, h), np.float32)
    for t in range(T):
        sr = surr(u_post[:, :, t] - THETA)  # B,h
        e = ALPHA * e + sr[:, :, None] * s_pre[:, None, :, t]
    return e


# ----------------------------- unified trainer -----------------------------
def train(A, rule, epochs, eta, X1, X2, Y, seed, bs=20, sigma=0.5, lam=0.0):
    h = A["h"]; rng = np.random.default_rng(seed ^ 0x3FAC70)
    Bfb = (rng.random(h) * 2 - 1).astype(np.float32)         # fixed random feedback (top->L1)
    Bfb2 = (rng.random(h) * 2 - 1).astype(np.float32)        # for 2-layer L2 feedback if needed
    base = 0.0
    idx = np.arange(len(Y))
    for _ in range(epochs):
        rng.shuffle(idx)
        for b0 in range(0, len(Y), bs):
            bi = idx[b0:b0+bs]; nb = len(bi)
            C = forward(A, X1[bi], X2[bi]); p = sig(C["logit"])
            rtop = C["rtop"]; ei = elig_in(C["u1"], X1[bi], X2[bi])

            # ---- readout signal + top-layer teaching signal 'teach' (B,) ----
            if rule.startswith("rl") or rule.startswith("np"):
                a = (rng.random(nb) < p).astype(np.float32)
                r = np.where(np.abs(a - Y[bi]) < .5, 1.0, -1.0).astype(np.float32)
                pcorr = np.where(Y[bi] > .5, p, 1 - p)        # graded correctness in [0,1]
                A["wout"] += eta * ((r * (a - p)) @ rtop) / nb
                A["by"] += eta * float(np.mean(r * (a - p)))
            else:  # error rules
                d = (p - Y[bi]).astype(np.float32)
                A["wout"] -= eta * (d @ rtop) / nb
                A["by"] -= eta * float(d.mean())

            # ---- hidden modulator for layer feeding the readout (mod: B,h) ----
            if rule == "rl_flat":
                mod = np.broadcast_to(r[:, None], (nb, h))
            elif rule == "rl_graded":
                mod = np.broadcast_to((pcorr - base)[:, None], (nb, h)); base = .9*base+.1*float(pcorr.mean())
            elif rule == "rl_reinforce":
                mod = np.broadcast_to((r * (a - p))[:, None], (nb, h))
            elif rule == "rl_reinforce_fb":
                mod = Bfb[None, :] * (r * (a - p))[:, None]
            elif rule == "rl_reinforce_wt":
                mod = A["wout"][None, :] * (r * (a - p))[:, None]
            elif rule == "err_broadcast":
                mod = np.broadcast_to((-d)[:, None], (nb, h))
            elif rule == "err_dfa":
                mod = Bfb[None, :] * (-d)[:, None]
            elif rule == "err_transport":
                mod = A["wout"][None, :] * (-d)[:, None]
            elif rule == "readout_only":
                mod = None
            elif rule == "freeze_l1":
                # Depth control: freeze win; teach L2 with supervised DFA error.
                # Isolates whether L2+readout adaptation alone beats readout_only.
                mod = Bfb[None, :] * (-d)[:, None]
            elif rule in ("np_reward", "np_graded"):
                # node perturbation on the readout-feeding layer's rates
                xi = rng.standard_normal((nb, h)).astype(np.float32) * sigma
                lg_p = A["by"] + (rtop + xi) @ A["wout"]
                if rule == "np_reward":
                    a2 = (rng.random(nb) < sig(lg_p)).astype(np.float32)
                    r2 = np.where(np.abs(a2 - Y[bi]) < .5, 1.0, -1.0).astype(np.float32)
                    dJ = (r2 - r)                             # scalar reward change
                else:
                    Jb = np.where(Y[bi] > .5, p, 1 - p)
                    Jp = np.where(Y[bi] > .5, sig(lg_p), 1 - sig(lg_p))
                    dJ = (Jp - Jb)                            # graded objective change
                mod = (dJ[:, None] * xi) / (sigma * sigma)   # correlate perturbation w/ reward
            else:
                raise ValueError(rule)

            if mod is not None:
                if A["layers"] == 1:
                    # `mod` teaches layer-1 (the readout-feeding layer) via input elig.
                    A["win"] += eta * np.einsum("bi,bij->ij", mod, ei) / nb - lam * A["win"]
                else:
                    # 2-layer: `mod` teaches L2 (readout-feeding); also teach L1+w12
                    # unless freeze_l1 (L2+readout only).
                    e12 = elig_layer(C["u2"], C["s1"])
                    A["w12"] += eta * np.einsum("bi,bij->ij", mod, e12) / nb - lam * A["w12"]
                    if rule == "freeze_l1":
                        top_scalar = None
                    elif rule.startswith("err"):
                        top_scalar = (-d).astype(np.float32)
                    elif rule.startswith("rl") or rule.startswith("np"):
                        # reuse the per-example modulator mean as a scalar teach
                        top_scalar = mod.mean(axis=1).astype(np.float32)
                    else:
                        top_scalar = None
                    if top_scalar is not None:
                        mod1 = Bfb2[None, :] * top_scalar[:, None]
                        A["win"] += eta * np.einsum("bi,bij->ij", mod1, ei) / nb - lam * A["win"]
    return A


def run(rule, seed, task, layers, h=128, epochs=80, ntr=160, nte=100, eta=0.05,
        init_preset="strong"):
    if task == "coin":
        gen = gen_coincidence
    elif task == "xor":
        gen = gen_xor
    elif task == "xor_coin" or task == "xor_thresh":
        gen = lambda n, s: gen_xor_thresh(n, s, thresh=3)
    elif task == "xnor":
        gen = gen_xnor
    else:
        raise ValueError(task)
    X1, X2, Y = gen(ntr, seed ^ 0xA1); TX1, TX2, TY = gen(nte, seed ^ 0xB2)
    A = init(h, seed, layers, init_preset=init_preset)
    if rule == "gradient":
        return train_bptt(A, epochs, eta if eta > 0 else .05, X1, X2, Y, seed, TX1, TX2, TY)
    train(A, rule, epochs, eta, X1, X2, Y, seed)
    return eval_acc(A, TX1, TX2, TY)


def train_bptt(A, epochs, lr, X1, X2, Y, seed, TX1, TX2, TY, bs=16):
    """Minibatch BPTT ceiling for 1 or 2 layers (autodiff-free, hand-rolled)."""
    h = A["h"]; rng = np.random.default_rng(seed ^ 0x6DAD); idx = np.arange(len(Y))
    for _ in range(epochs):
        rng.shuffle(idx)
        for b0 in range(0, len(Y), bs):
            bi = idx[b0:b0+bs]; nb = len(bi)
            C = forward(A, X1[bi], X2[bi]); p = sig(C["logit"]); d = (p - Y[bi]) / nb
            rtop = C["rtop"]
            A["wout"] -= lr * (d @ rtop); A["by"] -= lr * d.sum()
            g_rtop = np.outer(d, A["wout"])                  # dL/drtop (nb,h)
            if A["layers"] == 2:
                du2 = bptt_layer_grad(A, C["u2"], C["s2"], g_rtop, A, "w12", src=C["s1"], upd=True, lr=lr)
                # backprop to layer1 rates via w12
                g_r1 = du2 @ A["w12"]                        # (nb,h)
                bptt_layer_grad_input(A, C["u1"], g_r1, X1[bi], X2[bi], lr)
            else:
                bptt_layer_grad_input(A, C["u1"], g_rtop, X1[bi], X2[bi], lr)
    return eval_acc(A, TX1, TX2, TY)


def bptt_layer_grad_input(A, u, g_r, X1, X2, lr):
    """Accumulate + apply grad of a layer's win from dL/drate g_r (nb,h)."""
    nb, h, _ = u.shape; dwin = np.zeros((h, 2), np.float32); du_next = np.zeros((nb, h), np.float32)
    for t in range(T - 1, -1, -1):
        ds = g_r - du_next
        sd = surr(u[:, :, t] - THETA)
        du = ds * sd + ALPHA * du_next
        dwin[:, 0] += du.T @ X1[:, t]; dwin[:, 1] += du.T @ X2[:, t]
        du_next = du
    A["win"] -= lr * dwin


def bptt_layer_grad(A, u, s, g_r, Adummy, wkey, src, upd, lr):
    """Grad for layer-2 driven by src spikes through w[wkey]; returns du (nb,h) at t-sum. Applies w12 update."""
    nb, h, _ = u.shape; dw = np.zeros((h, h), np.float32); du_next = np.zeros((nb, h), np.float32)
    du_acc = np.zeros((nb, h), np.float32)
    for t in range(T - 1, -1, -1):
        ds = g_r - du_next
        sd = surr(u[:, :, t] - THETA)
        du = ds * sd + ALPHA * du_next
        # dW12[h,k] += sum_b du[b,h]*src[b,k,t]
        dw += np.einsum("bh,bk->hk", du, src[:, :, t])
        du_acc += du
        du_next = du
    A[wkey] -= lr * dw
    return du_acc


def summarize(accs, ceil):
    accs = np.asarray(accs); ceil = np.asarray(ceil); n = len(accs)
    gaps = np.array([np.clip((a-.5)/(c-.5), 0, 1) if c-.5 >= .15 else 0.0 for a, c in zip(accs, ceil)])
    se = gaps.std(ddof=1)/np.sqrt(n) if n > 1 else 0
    return accs.mean(), accs.std(ddof=1) if n > 1 else 0, gaps.mean(), gaps.mean()-1.96*se


def summarize_vs_readout(accs, readout):
    """Paired excess over inflated readout_only (depth-locality acceptance)."""
    accs = np.asarray(accs, dtype=np.float64)
    readout = np.asarray(readout, dtype=np.float64)
    n = len(accs)
    excess = accs - readout
    mean = float(excess.mean())
    sd = float(excess.std(ddof=1)) if n > 1 else 0.0
    se = sd / np.sqrt(n) if n > 1 else 0.0
    return dict(excess_mean=mean, excess_sd=sd, excess_lcb=mean - 1.96 * se,
                frac_above=float((excess > 0).mean()))


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--exp", required=True,
                    choices=["direction", "nodepert", "xor", "depth", "depth_locality",
                             "xor_coin", "xor_thresh", "xnor"])
    ap.add_argument("--seeds", type=int, default=12)
    ap.add_argument("--h", type=int, default=128)
    ap.add_argument("--epochs", type=int, default=80)
    ap.add_argument("--eta", type=float, default=0.05)
    ap.add_argument("--init-preset", default="strong", choices=list(INIT_PRESETS.keys()),
                    help="2-layer init scale (depth / depth_locality). strong=P1 default.")
    ap.add_argument("--out", default=None)
    a = ap.parse_args()

    if a.exp == "direction":
        rules = ["gradient", "rl_flat", "rl_graded", "rl_reinforce", "rl_reinforce_fb",
                 "rl_reinforce_wt", "err_broadcast", "err_dfa"]; task, layers = "coin", 1
    elif a.exp == "nodepert":
        rules = ["gradient", "rl_flat", "np_reward", "np_graded", "err_broadcast", "err_dfa"]; task, layers = "coin", 1
    elif a.exp == "xor":
        rules = ["gradient", "readout_only", "rl_flat", "rl_reinforce_wt",
                 "err_broadcast", "err_dfa", "np_graded"]; task, layers = "xor", 1
    elif a.exp in ("xor_coin", "xor_thresh"):
        # P3: temporal XOR with early cut thresh=3 (not T/2).
        rules = ["gradient", "readout_only", "err_broadcast", "err_dfa"]; task, layers = "xor_thresh", 1
    elif a.exp == "xnor":
        # Contrast: Boolean dual — broadcast often *also* solves (no locality flip).
        rules = ["gradient", "readout_only", "err_broadcast", "err_dfa"]; task, layers = "xnor", 1
    elif a.exp == "depth_locality":
        # Careful depth probe vs inflated readout_only (P1 strong init).
        # Includes DFA + rl_reinforce_fb (v12 family) and freeze_l1 control.
        rules = ["gradient", "readout_only", "freeze_l1", "err_broadcast", "err_dfa",
                 "rl_flat", "rl_reinforce_fb"]; task, layers = "xor", 2
    else:  # depth: 2-hidden-layer on XOR (legacy P1 table)
        rules = ["gradient", "readout_only", "err_broadcast", "err_dfa", "np_graded"]; task, layers = "xor", 2

    init_preset = a.init_preset if layers == 2 else "strong"
    res = {r: [] for r in rules}
    for s in range(a.seeds):
        for r in rules:
            res[r].append(run(r, s, task, layers, h=a.h, epochs=a.epochs, eta=a.eta,
                              init_preset=init_preset))
        print("seed %2d " % s + "  ".join(f"{r}={res[r][-1]:.2f}" for r in rules), flush=True)
    ceil = res["gradient"]
    print(f"\n=== exp={a.exp} task={task} layers={layers} init={init_preset} "
          f"n={a.seeds} h={a.h} ===")
    print(f"{'rule':<18}{'mean':>7}{'sd':>7}{'gap':>7}{'gapLCB':>8}", end="")
    if "readout_only" in res:
        print(f"{'exRO':>8}{'exROLCB':>9}")
    else:
        print()
    summary = {}
    for r in rules:
        m, sd, g, lcb = summarize(res[r], ceil)
        entry = dict(mean=m, sd=sd, gap=g, gap_lcb=lcb)
        if "readout_only" in res and r != "readout_only":
            vs = summarize_vs_readout(res[r], res["readout_only"])
            entry.update(vs)
            print(f"{r:<18}{m:>7.3f}{sd:>7.3f}{g:>7.3f}{lcb:>8.3f}"
                  f"{vs['excess_mean']:>8.3f}{vs['excess_lcb']:>9.3f}")
        elif "readout_only" in res:
            print(f"{r:<18}{m:>7.3f}{sd:>7.3f}{g:>7.3f}{lcb:>8.3f}{'—':>8}{'—':>9}")
        else:
            print(f"{r:<18}{m:>7.3f}{sd:>7.3f}{g:>7.3f}{lcb:>8.3f}")
        summary[r] = entry
    if a.out:
        payload = dict(exp=a.exp, task=task, layers=layers, init_preset=init_preset,
                       raw=res, summary=summary)
        json.dump(payload, open(a.out, "w"), indent=2)
        print("wrote", a.out)


if __name__ == "__main__":
    main()
