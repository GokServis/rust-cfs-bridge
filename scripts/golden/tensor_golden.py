#!/usr/bin/env python3
"""Golden outputs for ai_app_tensor.c unit tests."""
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


def linear(w_rows, x):
    return [sum(wi * xi for wi, xi in zip(row, x)) for row in w_rows]


def main():
    x = [1.0, 0.0, -2.0, 0.5]
    print("rmsnorm:", ", ".join(f"{v:.17g}" for v in rmsnorm(x)))
    lg = [1.0, 2.0, -0.5, 0.25]
    sm = softmax(lg)
    print("softmax:", ", ".join(f"{v:.17g}" for v in sm))
    print("softmax_sum:", f"{sum(sm):.17g}")
    w = [[0.1, 0.2], [0.3, -0.1], [-0.2, 0.4]]
    xv = [2.0, -1.0]
    y = linear(w, xv)
    print("linear:", ", ".join(f"{v:.17g}" for v in y))


if __name__ == "__main__":
    main()
