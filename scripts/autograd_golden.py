#!/usr/bin/env python3
"""
Emit C-style golden vectors for ai_app autograd unit tests.
Matches microgpt Value semantics (scripts/microgpt.py).
"""

from __future__ import annotations

import math
import random


class Value:
    __slots__ = ("data", "grad", "_children", "_local_grads")

    def __init__(self, data, children=(), local_grads=()):
        self.data = data
        self.grad = 0
        self._children = children
        self._local_grads = local_grads

    def __add__(self, other):
        other = other if isinstance(other, Value) else Value(other)
        return Value(self.data + other.data, (self, other), (1, 1))

    def __mul__(self, other):
        other = other if isinstance(other, Value) else Value(other)
        return Value(self.data * other.data, (self, other), (other.data, self.data))

    def exp(self):
        return Value(math.exp(self.data), (self,), (math.exp(self.data),))

    def __neg__(self):
        return self * -1

    def backward(self):
        topo = []
        visited = set()

        def build_topo(v):
            if v not in visited:
                visited.add(v)
                for child in v._children:
                    build_topo(child)
                topo.append(v)

        build_topo(self)
        self.grad = 1
        for v in reversed(topo):
            for child, local_grad in zip(v._children, v._local_grads):
                child.grad += local_grad * v.grad


def c_double(x: float) -> str:
    return f"{x:.17g}"


def emit_case(name: str, nodes: list[tuple[str, float]]) -> None:
    print(f"/* {name} */")
    for label, g in nodes:
        print(f"/* {label}_grad */ {c_double(g)},")


def main() -> None:
    random.seed(42)

    # Case A: mul + exp chain (shared subgraph later)
    a = Value(2.0)
    b = Value(3.0)
    c = a * b
    d = c.exp()
    d.backward()
    print("/* === Case A: d = exp(a*b), a=2, b=3 === */")
    emit_case(
        "A",
        [
            ("a", a.grad),
            ("b", b.grad),
            ("c", c.grad),
            ("d", d.grad),
        ],
    )
    print(f"/* A data: a b c d */ {c_double(a.data)}, {c_double(b.data)}, {c_double(c.data)}, {c_double(d.data)}")

    # Case B: add then mul
    x = Value(1.5)
    y = Value(2.5)
    z = x + y
    w = z * Value(4.0)
    w.backward()
    print("/* === Case B: w = (x+y)*4 === */")
    emit_case("B", [("x", x.grad), ("y", y.grad), ("z", z.grad), ("w", w.grad)])

    # Case C: shared node — u = a * b; v = u + u; loss = v (u used twice)
    p = Value(0.5)
    q = Value(1.25)
    u = p * q
    v = u + u
    v.backward()
    print("/* === Case C: v = u+u, u=p*q === */")
    emit_case("C", [("p", p.grad), ("q", q.grad), ("u", u.grad), ("v", v.grad)])

    # Case D: exp only
    t = Value(0.25)
    e = t.exp()
    e.backward()
    print("/* === Case D: e = exp(t), t=0.25 === */")
    emit_case("D", [("t", t.grad), ("e", e.grad)])


if __name__ == "__main__":
    main()
