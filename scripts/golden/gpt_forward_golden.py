#!/usr/bin/env python3
"""
Reference GPT forward for one step — matches ai_app_gpt.c (microgpt layout).
Fills weights from a deterministic LCG-style sequence for reproducibility.
"""
from __future__ import annotations

import math


def rmsnorm(x):
    n = len(x)
    ms = sum(t * t for t in x) / n
    scale = (ms + 1e-5) ** -0.5
    return [t * scale for t in x]


def softmax(logits):
    m = max(logits)
    exps = [math.exp(v - m) for v in logits]
    s = sum(exps)
    return [e / s for e in exps]


def linear(x, w):
    return [sum(wi * xi for wi, xi in zip(wo, x)) for wo in w]


def mat_rows(flat, n_out, n_in):
    return [flat[r * n_in : (r + 1) * n_in] for r in range(n_out)]


def fill_weights(seed: int, vocab, n_embd, block, n_head, n_layer):
    """Return flat arrays matching C row-major layout."""
    rng = seed
    out = {}

    def next_f():
        nonlocal rng
        rng = (rng * 1103515245 + 12345) & 0x7FFFFFFF
        return (rng / 0x7FFFFFFF) * 0.16 - 0.08

    def mat(n_out, n_in):
        return [next_f() for _ in range(n_out * n_in)]

    out["wte"] = mat(vocab, n_embd)
    out["wpe"] = mat(block, n_embd)
    out["lm_head"] = mat(vocab, n_embd)
    layers = []
    for _ in range(n_layer):
        layers.append(
            {
                "attn_wq": mat(n_embd, n_embd),
                "attn_wk": mat(n_embd, n_embd),
                "attn_wv": mat(n_embd, n_embd),
                "attn_wo": mat(n_embd, n_embd),
                "mlp_fc1": mat(4 * n_embd, n_embd),
                "mlp_fc2": mat(n_embd, 4 * n_embd),
            }
        )
    out["layers"] = layers
    return out


def gpt_step(weights, kv_k, kv_v, token_id, pos_id, vocab, n_embd, block, n_head, n_layer):
    head_dim = n_embd // n_head
    te = weights["wte"][token_id * n_embd : (token_id + 1) * n_embd]
    pe = weights["wpe"][pos_id * n_embd : (pos_id + 1) * n_embd]
    x = [t + p for t, p in zip(te, pe)]
    x = rmsnorm(x)

    for li in range(n_layer):
        lw = weights["layers"][li]
        x_residual = x
        x = rmsnorm(x)
        wq = mat_rows(lw["attn_wq"], n_embd, n_embd)
        wk = mat_rows(lw["attn_wk"], n_embd, n_embd)
        wv = mat_rows(lw["attn_wv"], n_embd, n_embd)
        wo = mat_rows(lw["attn_wo"], n_embd, n_embd)
        q = linear(x, wq)
        k = linear(x, wk)
        v = linear(x, wv)
        kv_k[li].append(k)
        kv_v[li].append(v)
        x_attn = []
        for h in range(n_head):
            hs = h * head_dim
            q_h = q[hs : hs + head_dim]
            attn_logits = []
            for t in range(len(kv_k[li])):
                k_h = kv_k[li][t][hs : hs + head_dim]
                attn_logits.append(sum(q_h[j] * k_h[j] for j in range(head_dim)) / math.sqrt(head_dim))
            attn_weights = softmax(attn_logits)
            head_out = []
            for j in range(head_dim):
                acc = 0.0
                for t in range(len(kv_v[li])):
                    v_h = kv_v[li][t][hs : hs + head_dim]
                    acc += attn_weights[t] * v_h[j]
                head_out.append(acc)
            x_attn.extend(head_out)
        x = linear(x_attn, wo)
        x = [a + b for a, b in zip(x, x_residual)]
        x_residual = x
        x = rmsnorm(x)
        fc1 = mat_rows(lw["mlp_fc1"], 4 * n_embd, n_embd)
        fc2 = mat_rows(lw["mlp_fc2"], n_embd, 4 * n_embd)
        h = linear(x, fc1)
        h = [max(0.0, t) for t in h]
        x = linear(h, fc2)
        x = [a + b for a, b in zip(x, x_residual)]

    lh = mat_rows(weights["lm_head"], vocab, n_embd)
    return linear(x, lh)


def main():
    vocab = 17
    n_embd = 16
    block = 16
    n_head = 4
    n_layer = 1
    seed = 1234567
    w = fill_weights(seed, vocab, n_embd, block, n_head, n_layer)
    kv_k = [[] for _ in range(n_layer)]
    kv_v = [[] for _ in range(n_layer)]
    logits = gpt_step(w, kv_k, kv_v, token_id=3, pos_id=0, vocab=vocab, n_embd=n_embd, block=block, n_head=n_head, n_layer=n_layer)
    print("first5_logits:", ", ".join(f"{v:.17g}" for v in logits[:5]))
    print("logits_len:", len(logits))


if __name__ == "__main__":
    main()
