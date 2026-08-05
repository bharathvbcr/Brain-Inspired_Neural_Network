#!/usr/bin/env python3
"""Matched-architecture control — full experimental battery (protocol v4 preview).

All arms run on ONE identical dense-LIF forward graph (same as the Rust
`MatchedArch::forward`). Only the weight-update rule changes. This lets us not
just confirm the v2 negative but *decompose* it: a ladder of increasingly
capable but still-local credit rules shows exactly which ingredient (a reward
baseline, per-neuron feedback, weight transport) is needed to close the
gradient gap.

Rules on the ladder (all share the forward; listed weakest -> strongest):
  gradient   : SuperSpike BPTT (the ceiling; not local).
  broadcast  : production three-factor. Hidden synapses see ONE broadcast
               scalar reward M=+/-1 times local eligibility. No feedback, no
               transport. (This is the BINN production rule.)
  rpe        : broadcast + running-mean reward baseline  M = r - b  (soft RPE;
               the variance-reduction BINN "deferred").
  dfa        : direct feedback alignment. Per-neuron hidden credit via FIXED
               RANDOM feedback of the output error. Local (no weight transport),
               but per-neuron rather than one scalar.
  eprop      : e-prop / weight-transport. Hidden credit uses the real wout as
               feedback. Upper bound for "local" (peeks at forward weights).

Everything else (forward, width, encoding, epochs, data, seeds) is held
identical, so `gap_closed = (rule - 0.5)/(gradient - 0.5)` is a one-variable
contrast.

This is a PREVIEW harness (fast NumPy). The binding verdict is the Rust n=20
run under the protocol-v4 hash.
"""
import argparse
import json
import numpy as np

T = 8
N_IN = 2
TAU_M = 20.0
ALPHA = float(np.exp(-1.0 / TAU_M))
THETA = 1.0
VRESET = 0.0
BETA = 5.0
CHANCE = 0.5
# Feed-forward LIF for ALL arms (coincidence is detectable via membrane
# integration; recurrence adds cost without changing the comparison). Applied
# identically to every arm, so the match is preserved.
FEEDFORWARD = True
PLASTIC_REC = False


def surrogate(u_minus_theta, beta=BETA):
    d = 1.0 + beta * np.abs(u_minus_theta)
    return 1.0 / (d * d)


def sigmoid(z):
    return 1.0 / (1.0 + np.exp(-np.clip(z, -60, 60)))


def gen_examples(n, seed):
    """Coincidence task: label 1 iff the two one-hot spikes are within +/-1 frame."""
    rng = np.random.default_rng(seed)
    X1 = np.zeros((n, T), np.float32)
    X2 = np.zeros((n, T), np.float32)
    Y = np.zeros(n, np.float32)
    for i in range(n):
        t1 = int(rng.integers(T))
        X1[i, t1] = 1.0
        if rng.random() < 0.5:
            opts = [t for t in (t1 - 1, t1, t1 + 1) if 0 <= t < T]
            t2 = int(rng.choice(opts))
        else:
            t2 = int(rng.integers(T))
            while abs(t2 - t1) <= 1:
                t2 = int(rng.integers(T))
        X2[i, t2] = 1.0
        Y[i] = 1.0 if abs(t1 - t2) <= 1 else 0.0
    return X1, X2, Y


def init_arch(h, seed):
    rng = np.random.default_rng(seed ^ 0x5171)
    win = ((rng.random((h, N_IN)) * 2 - 1) * 0.5).astype(np.float32)
    wrec = ((rng.random((h, h)) * 2 - 1) * (0.3 / np.sqrt(h))).astype(np.float32)
    if FEEDFORWARD:
        wrec[:] = 0.0
    wout = ((rng.random(h) * 2 - 1) * 0.2).astype(np.float32)
    by = np.float32(0.0)
    return dict(win=win, wrec=wrec, wout=wout, by=by, h=h)


# ---- batched forward (over a batch B of examples) ----
def forward_batch(A, X1, X2):
    """Return u (B,h,T), s (B,h,T), rates (B,h), logit (B,)."""
    B = X1.shape[0]
    h = A["h"]
    u = np.zeros((B, h, T), np.float32)
    s = np.zeros((B, h, T), np.float32)
    for t in range(T):
        cur = np.outer(X1[:, t], A["win"][:, 0]) + np.outer(X2[:, t], A["win"][:, 1])  # B,h
        if t > 0 and not FEEDFORWARD:
            cur = cur + s[:, :, t - 1] @ A["wrec"].T
        uprev = u[:, :, t - 1] if t > 0 else np.full((B, h), VRESET, np.float32)
        sprev = s[:, :, t - 1] if t > 0 else np.zeros((B, h), np.float32)
        ui = ALPHA * uprev + cur - THETA * sprev
        u[:, :, t] = ui
        s[:, :, t] = (ui >= THETA).astype(np.float32)
    rates = s.sum(axis=2)  # B,h
    logit = A["by"] + rates @ A["wout"]  # B,
    return u, s, rates, logit


def eval_acc(A, X1, X2, Y, bs=256):
    correct = 0
    for i in range(0, len(Y), bs):
        _, _, _, logit = forward_batch(A, X1[i:i + bs], X2[i:i + bs])
        pred = (sigmoid(logit) >= 0.5).astype(np.float32)
        correct += int((np.abs(pred - Y[i:i + bs]) < 0.5).sum())
    return correct / len(Y)


# ---- gradient ceiling: minibatch SuperSpike BPTT ----
def train_gradient(A, epochs, lr, X1, X2, Y, bs=16, seed=0):
    h = A["h"]
    rng = np.random.default_rng(seed ^ 0x6DAD)
    idx = np.arange(len(Y))
    for _ in range(epochs):
        rng.shuffle(idx)
        for b0 in range(0, len(Y), bs):
            bi = idx[b0:b0 + bs]
            _train_gradient_batch(A, lr, X1[bi], X2[bi], Y[bi])


def _train_gradient_batch(A, lr, X1, X2, Y):
        h = A["h"]
        B = len(Y)
        u, s, rates, logit = forward_batch(A, X1, X2)
        dlogit = (sigmoid(logit) - Y) / B  # B,
        dwout = rates.T @ dlogit  # h,
        dby = dlogit.sum()
        g_r = np.outer(dlogit, A["wout"])  # B,h
        du_next = np.zeros((B, h), np.float32)
        dwin = np.zeros((h, N_IN), np.float32)
        dwrec = np.zeros((h, h), np.float32)
        for t in range(T - 1, -1, -1):
            ds = g_r - du_next + du_next @ A["wrec"]  # B,h
            surr = surrogate(u[:, :, t] - THETA)
            du = ds * surr + ALPHA * du_next  # B,h
            dwin[:, 0] += du.T @ X1[:, t]
            dwin[:, 1] += du.T @ X2[:, t]
            if PLASTIC_REC and t > 0:
                dwrec += du.T @ s[:, :, t - 1]
            du_next = du
        A["win"] -= lr * dwin
        if PLASTIC_REC:
            A["wrec"] -= lr * dwrec
        A["wout"] -= lr * dwout
        A["by"] -= lr * dby


# ---- shared eligibility for a single example ----
def eligibility_single(A, u, s, x1, x2, plastic_rec=False):
    """Return e_in (h,N_IN), e_rec (h,h or None) for one example. u,s are (h,T).

    Recurrent weights are frozen by default (identically across ALL arms, so the
    comparison stays matched) — this removes the dominant h*h per-step cost and
    the coincidence task is feed-forward-solvable given the fixed recurrence.
    """
    h = A["h"]
    ei = np.zeros((h, N_IN), np.float32)
    erec = np.zeros((h, h), np.float32) if plastic_rec else None
    for t in range(T):
        surr = surrogate(u[:, t] - THETA)  # h,
        ei[:, 0] = ALPHA * ei[:, 0] + surr * x1[t]
        ei[:, 1] = ALPHA * ei[:, 1] + surr * x2[t]
        if plastic_rec:
            if t > 0:
                erec = ALPHA * erec + np.outer(surr, s[:, t - 1])
            else:
                erec = ALPHA * erec
    return ei, erec


# Named rules = (supervision signal, hidden credit locality, use baseline).
#   signal:  "rl"  -> sampled action + reward +/-1  (BINN production regime)
#            "sup" -> supervised output error  d = p - y
#   credit:  "broadcast" -> one scalar to all hidden units
#            "dfa"       -> fixed random per-neuron feedback (local, no transport)
#            "transport" -> real wout as feedback (e-prop / weight transport)
RULES = {
    # --- RL reward regime (BINN production family) ---
    "broadcast":     dict(signal="rl",  credit="broadcast", baseline=False),  # production
    "rpe":           dict(signal="rl",  credit="broadcast", baseline=True),   # + reward baseline
    "rl_dfa":        dict(signal="rl",  credit="dfa",       baseline=True),   # RL + per-neuron fb
    "rl_transport":  dict(signal="rl",  credit="transport", baseline=True),   # RL + weight transport
    # --- supervised error regime ---
    "broadcast_sup": dict(signal="sup", credit="broadcast", baseline=False),  # scalar error
    "dfa":           dict(signal="sup", credit="dfa",       baseline=False),  # fixed random fb
    "eprop":         dict(signal="sup", credit="transport", baseline=False),  # weight transport
    # --- control: is the task solvable by the readout alone? ---
    "readout_only":  dict(signal="sup", credit="frozen",    baseline=False),
}


def eligibility_batch(A, u, s, X1, X2, plastic_rec=False):
    """Batched eligibility. u,s: (B,h,T). Returns e_in (B,h,N_IN), e_rec (B,h,h|None)."""
    B, h, _ = u.shape
    ei = np.zeros((B, h, N_IN), np.float32)
    erec = np.zeros((B, h, h), np.float32) if plastic_rec else None
    for t in range(T):
        surr = surrogate(u[:, :, t] - THETA)  # B,h
        ei[:, :, 0] = ALPHA * ei[:, :, 0] + surr * X1[:, t][:, None]
        ei[:, :, 1] = ALPHA * ei[:, :, 1] + surr * X2[:, t][:, None]
        if plastic_rec:
            if t > 0:
                erec = ALPHA * erec + surr[:, :, None] * s[:, None, :, t - 1]
            else:
                erec = ALPHA * erec
    return ei, erec


def train_local(A, rule, epochs, eta, lam, X1, X2, Y, seed, feedback=None, bs=20):
    """Minibatch three-factor family. `rule` = (supervision signal) x (credit locality).

    Minibatched (like the gradient arm) for speed and a fair comparison; the
    credit structure — broadcast scalar vs per-neuron feedback vs transport, and
    RL reward vs supervised error — is exactly as in the online form.
    """
    spec = RULES[rule]
    signal, credit, use_bl = spec["signal"], spec["credit"], spec["baseline"]
    h = A["h"]
    rng = np.random.default_rng(seed ^ 0x3FAC70)
    if feedback is None:
        feedback = (rng.random(h) * 2 - 1).astype(np.float32)  # fixed random B for DFA
    baseline = 0.0
    bl_decay = 0.9
    idx = np.arange(len(Y))
    for _ in range(epochs):
        rng.shuffle(idx)
        for b0 in range(0, len(Y), bs):
            bi = idx[b0:b0 + bs]
            nb = len(bi)
            u, s, rates, logit = forward_batch(A, X1[bi], X2[bi])  # (nb,h,T),(nb,h),(nb,)
            p = sigmoid(logit)                                     # nb
            ei, erec = eligibility_batch(A, u, s, X1[bi], X2[bi], plastic_rec=PLASTIC_REC)

            if signal == "rl":
                a = (rng.random(nb) < p).astype(np.float32)
                reward = np.where(np.abs(a - Y[bi]) < 0.5, 1.0, -1.0).astype(np.float32)
                m = (reward - baseline) if use_bl else reward     # nb broadcast scalars
                drive = a - p                                     # nb
                A["wout"] += eta * ((m * drive) @ rates) / nb - lam * A["wout"]
                A["by"] += eta * np.mean(m * drive)
                if credit == "broadcast":
                    modb = np.broadcast_to((m)[:, None], (nb, h))
                elif credit == "dfa":
                    modb = feedback[None, :] * (m * drive)[:, None]
                elif credit == "frozen":
                    modb = None
                else:  # transport
                    modb = A["wout"][None, :] * (m * drive)[:, None]
                baseline = bl_decay * baseline + (1 - bl_decay) * float(reward.mean())
            else:  # supervised error d = p - y
                d = (p - Y[bi]).astype(np.float32)                # nb
                A["wout"] -= eta * (d @ rates) / nb + lam * A["wout"]
                A["by"] -= eta * float(d.mean())
                if credit == "broadcast":
                    modb = np.broadcast_to((-d)[:, None], (nb, h))
                elif credit == "dfa":
                    modb = feedback[None, :] * (-d)[:, None]
                elif credit == "frozen":
                    modb = None
                else:  # transport
                    modb = A["wout"][None, :] * (-d)[:, None]

            if modb is not None:
                A["win"] += eta * np.einsum("bi,bij->ij", modb, ei) / nb - lam * A["win"]
                if erec is not None:
                    A["wrec"] += eta * np.einsum("bi,bij->ij", modb, erec) / nb - lam * A["wrec"]
    return feedback


def run_condition(rule, seed, h=128, epochs=80, n_train=80, n_test=40,
                  eta=0.05, lam=0.0, lr=0.05):
    X1, X2, Y = gen_examples(n_train, seed ^ 0xA1)
    TX1, TX2, TY = gen_examples(n_test, seed ^ 0xB2)
    A = init_arch(h, seed)
    if rule == "gradient":
        train_gradient(A, epochs, lr, X1, X2, Y, seed=seed)
    else:
        train_local(A, rule, epochs, eta, lam, X1, X2, Y, seed)
    return eval_acc(A, TX1, TX2, TY)


def summarize(accs, ceiling, min_ref_gap=0.15, z=1.96):
    accs = np.asarray(accs, float)
    ceiling = np.asarray(ceiling, float)
    gaps = []
    for a, c in zip(accs, ceiling):
        denom = c - CHANCE
        if denom < min_ref_gap:
            gaps.append(0.0)
        else:
            gaps.append(float(np.clip((a - CHANCE) / denom, 0, 1)))
    gaps = np.asarray(gaps)
    n = len(gaps)
    mean_gap = gaps.mean()
    se = gaps.std(ddof=1) / np.sqrt(n) if n > 1 else 0.0
    return dict(
        mean_acc=float(accs.mean()), std_acc=float(accs.std(ddof=1)) if n > 1 else 0.0,
        mean_gap=float(mean_gap), gap_lcb=float(mean_gap - z * se),
        floor_pass=bool(accs.mean() >= 0.65), gap_pass=bool(mean_gap - z * se > 0.5),
        per_seed_acc=[float(x) for x in accs], per_seed_gap=[float(x) for x in gaps],
    )


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--seeds", type=int, default=20)
    ap.add_argument("--h", type=int, default=128)
    ap.add_argument("--epochs", type=int, default=80)
    ap.add_argument("--n-train", type=int, default=80)
    ap.add_argument("--n-test", type=int, default=40)
    ap.add_argument("--rules", default="gradient,broadcast,rpe,rl_dfa,rl_transport,"
                                       "broadcast_sup,dfa,eprop,readout_only")
    ap.add_argument("--eta", type=float, default=0.05)
    ap.add_argument("--out", default=None)
    ap.add_argument("--jsonl", default=None,
                    help="append per-seed rows here (enables chunked runs)")
    ap.add_argument("--seed-start", type=int, default=None)
    ap.add_argument("--seed-end", type=int, default=None)
    args = ap.parse_args()

    rules = args.rules.split(",")
    if args.seed_start is not None:
        seeds = list(range(args.seed_start, args.seed_end))
    else:
        seeds = list(range(args.seeds))

    # chunked mode: compute the requested seeds, append rows, exit.
    if args.jsonl:
        import os
        done = set()
        if os.path.exists(args.jsonl):
            for ln in open(args.jsonl):
                try:
                    done.add(json.loads(ln)["seed"])
                except Exception:
                    pass
        with open(args.jsonl, "a") as f:
            for s in seeds:
                if s in done:
                    print(f"seed {s:2d}: (skip, already done)", flush=True)
                    continue
                row = {"seed": s}
                for r in rules:
                    row[r] = run_condition(r, s, h=args.h, epochs=args.epochs,
                                           n_train=args.n_train, n_test=args.n_test,
                                           eta=args.eta)
                f.write(json.dumps(row) + "\n"); f.flush()
                print("seed %2d: " % s + "  ".join(f"{r}={row[r]:.3f}" for r in rules),
                      flush=True)
        return

    results = {r: [] for r in rules}
    for s in seeds:
        for r in rules:
            acc = run_condition(r, s, h=args.h, epochs=args.epochs,
                                n_train=args.n_train, n_test=args.n_test, eta=args.eta)
            results[r].append(acc)
        line = "  ".join(f"{r}={results[r][-1]:.3f}" for r in rules)
        print(f"seed {s:2d}: {line}", flush=True)

    ceiling = results["gradient"]
    print("\n=== Matched-architecture ladder (n=%d, h=%d, epochs=%d) ===" %
          (args.seeds, args.h, args.epochs))
    print(f"{'rule':<11}{'mean_acc':>9}{'std':>7}{'gap':>7}{'gap_LCB':>9}  verdict")
    summary = {}
    for r in rules:
        srr = summarize(results[r], ceiling)
        summary[r] = srr
        verdict = "PASS" if (srr["floor_pass"] and srr["gap_pass"]) else "FAIL"
        tag = "(ceiling)" if r == "gradient" else verdict
        print(f"{r:<11}{srr['mean_acc']:>9.4f}{srr['std_acc']:>7.3f}"
              f"{srr['mean_gap']:>7.3f}{srr['gap_lcb']:>9.3f}  {tag}")

    if args.out:
        with open(args.out, "w") as f:
            json.dump(dict(config=vars(args), raw=results, summary=summary), f, indent=2)
        print(f"\nwrote {args.out}")


if __name__ == "__main__":
    main()
