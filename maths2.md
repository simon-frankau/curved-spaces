# Curved manifolds without an embedding space

Following on from [maths.md](./maths.md), I want to do a quick
exploration of what the maths for curved spaces look like if you don't
want to work with an embedding $\mathbb{R}^n$ space.

I'm not going for the full generality or anything incredibly deep. I
just want to see if you can take an arbitrary embedded manifold and
represent it without referencing the embedding space. There are
probably constraints on smoothness etc., but I'm not going to go into
that. This is a very intuitive approach.

I never did anything like this for undergrad, but my goal is to derive
something that I can then look up e.g. the Wikipedia page on General
Relativity and see if I can find parallels with what I've derived.

While my eventual goal is to find equations describing the local
constraints on geodesics, I'm going to take a step back and start by
trying to find an embedding-free metric.

## Metrics

When you have a manifold embedded in $\mathbb{R}^n$, the metric is
inherited from $\mathbb{R}^n$: The idea of "length" in the manifold is
just the length of the curve as you follow it along the manifold in
the embedded space.

As described in [maths.md](./maths.md), the length along a path is

```math
L = \int_{t=0}^1 \sqrt{\sum_i \left(\frac{\textrm{d} x_i}{\textrm{d} t}\right)^2} \ \mathrm{d}t
```

where the $x_i$ are coordinates in $\mathbb{R}^n$.

and we'll use $L' = \sqrt{ \sum_i \left( \frac{\textrm{d}
x_i}{\textrm{d} t} \right)^2}$ as a way of describing the metric
locally.

However, this metric is in terms of coordinates in the embedding
Euclidean space, which is pretty exactly what we don't want. Instead,
let's assume we've got a local basis $\overline{x}_j$, $1 \le j \le m$
in the embedded space which has a nice, smooth mapping to the $x_i$,
and try to find a metric in terms of the $\overline{x}_j$.

Starting with $\frac{\mathrm{d} x_i}{\mathrm{d} t} = \sum_j
\frac{\mathrm{d} x_i}{\mathrm{d} \overline{x}_j} \frac{\mathrm{d}
\overline{x}_j}{\mathrm{d} t} $, we get

```math
L' = \sqrt{\sum_i \left(
\sum_j \frac{\mathrm{d} x_i}{\mathrm{d} \overline{x}_j} \frac{\mathrm{d}
\overline{x}_j}{\mathrm{d} t}
\right)^2}
```
Expanding this and switching to (subscript-only) [summation
convention](https://en.wikipedia.org/wiki/Einstein_notation), we get:

```math
L' = \sqrt{\sum_i \left(
\sum_j \frac{\mathrm{d} x_i}{\mathrm{d} \overline{x}_j} \frac{\mathrm{d}
\overline{x}_j}{\mathrm{d} t}
\right)
\left(
\sum_k \frac{\mathrm{d} x_i}{\mathrm{d} \overline{x}_k} \frac{\mathrm{d}
\overline{x}_k}{\mathrm{d} t}
\right)}
```

```math
L' = \sqrt{
\frac{\mathrm{d} x_i}{\mathrm{d} \overline{x}_j} \frac{\mathrm{d}
\overline{x}_j}{\mathrm{d} t}
\frac{\mathrm{d} x_i}{\mathrm{d} \overline{x}_k} \frac{\mathrm{d}
\overline{x}_k}{\mathrm{d} t}
}
```

```math
L' = \sqrt{
\frac{\mathrm{d} \overline{x}_j}{\mathrm{d} t}
\left(
\frac{\mathrm{d} x_i}{\mathrm{d} \overline{x}_j} 
\frac{\mathrm{d} x_i}{\mathrm{d} \overline{x}_k}
\right)
\frac{\mathrm{d} \overline{x}_k}{\mathrm{d} t}
}
```

```math
L' = \sqrt{
\frac{\mathrm{d} \overline{x}_j}{\mathrm{d} t}
g_{jk}
\frac{\mathrm{d} \overline{x}_k}{\mathrm{d} t}
}
```

where $g_{jk} = \frac{\mathrm{d} x_i}{\mathrm{d} \overline{x}_j}
\frac{\mathrm{d} x_i}{\mathrm{d} \overline{x}_k}$. The name " $g$ "
for this matrix has been shamelessly stolen from my later reading on
General Relativity, where it appears to be the [metric
tensor](https://en.m.wikipedia.org/wiki/Metric_tensor_(general_relativity)).
Admittedly there it's a Lorentzian manifold, which is slightly more
complicated (being based on a Minkowski metric), but I'm still quite
happy to have reinvented this in some way.

What does this mean? It means the concept of distance in curved space
is the square root of a symmetric bilinear form of the movement
vector, a fairly straightforward extension of the usual Euclidean
distance metric as the square root of the vector dotted with itself
(which is what you get if $g$ is the identity matrix).

The other fun thing is that for an embedded space of dimension $m$,
the local curvature can be represented by a symmetric $m$ by $m$
matrix, independent of the dimension of the embedding space. Your
surface could be embedded in a 4D space or a 100D space, the
information required to represent the local curvature is the same!

I think we've now got the hang of the metric, and can use it to start
working out how to create geodesics without reference to the embedding
space.

## Geodesics

**TODO!**
