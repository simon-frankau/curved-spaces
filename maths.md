# The maths of curved manifolds

This subject is well-established, but I've never learnt it properly,
and I decided to re-derive it from scratch myself. This file documents
this work.

## The basics of curvature

Let's say we have a curve $P(t) = (x_1, \ldots, x_n)$, $0 \le t \le 1$
in $n$ dimensional space, representing a path between $P(0)$ and
$P(1)$. The length of the path is

<!-- I'm using blocks as sometimes GH mis-identifies $$-delimited
maths blocks. -->

```math
L = \int_{t=0}^{1} \left| \frac{\mathrm{d} P(t)}{\mathrm{d} t}
\right| \ \mathrm{d}t = \int_{t=0}^1 \sqrt{\sum_{i=1}^n
\left(\frac{\textrm{d} x_i}{\textrm{d} t}\right)^2} \ \mathrm{d}t
```

We'll write $L' = \sqrt{\sum_{i=1}^n \left(\frac{\textrm{d}
x_i}{\textrm{d} t}\right)^2}$ to be the local "speed" along the path -
we'll use this later.

We want to show that deviations from this curve lead to a longer
length, so we create a distortion $H(t) = (\eta_1, \ldots, \eta_n)$,
$0 \le t \le 1$, $H(0) = H(1) = 0$. Then we look at the lengths of the
curves $P + \delta H$ (which we'll call $L_H(\delta)$ ). If $P$ is the
minimal length curve, both positive and negative values of $\delta$
will increase the curve length. In other words we want to show

<!-- I'd prefer eqnarray, but GH MD doesn't support it. -->

```math
\frac{\mathrm{d}L_H}{\mathrm{d}\delta} =
\left. \frac{\mathrm{d}}{\mathrm{d} \delta} \int_{t = 0}^1 \| P(t) + \delta
H(t) \| \ \mathrm{d}t \ \right|_{\delta = 0} = 0
```

So, let's evaluate it!

```math
\frac{\mathrm{d}L_H}{\mathrm{d}\delta} =
\left. \frac{\mathrm{d}}{\mathrm{d} \delta} \int_{t=0}^1
\sqrt{\sum_{i=1}^n \left(\frac{\textrm{d} x_i}{\textrm{d} t} + \delta
\frac{\textrm{d} \eta_i}{\textrm{d} t}\right)^2} \ \mathrm{d}t
\ \right|_{\delta = 0}
```

```math
= \left. \int_{t=0}^1 \frac{\mathrm{d}}{\mathrm{d} \delta}
\sqrt{\sum_{i=1}^n \left(\frac{\textrm{d} x_i}{\textrm{d} t} + \delta
\frac{\textrm{d} \eta_i}{\textrm{d} t}\right)^2} \ \mathrm{d}t
\ \right|_{\delta = 0}
```

```math
= \left. \int_{t=0}^1 \frac{1}{2} {{ \frac{\mathrm{d}}{\mathrm{d}
\delta} \sum_{i=1}^n \left(\frac{\textrm{d} x_i}{\textrm{d} t} +
\delta \frac{\textrm{d} \eta_i}{\textrm{d} t}\right)^2 } \over {
\sqrt{\sum_{i=1}^n \left(\frac{\textrm{d} x_i}{\textrm{d} t} + \delta
\frac{\textrm{d} \eta_i}{\textrm{d} t}\right)^2} }} \ \mathrm{d}t
\ \right|_{\delta = 0}
```

```math
= \left. \int_{t=0}^1 {{ \sum_{i=1}^n \frac{\mathrm{d}
\eta_i}{\mathrm{d} t} (\frac{\textrm{d} x_i}{\textrm{d} t} + \delta
\frac{\textrm{d} \eta_i}{\textrm{d} t}) } \over { \sqrt{\sum_{i=1}^n
\left(\frac{\textrm{d} x_i}{\textrm{d} t} + \delta \frac{\textrm{d}
\eta_i}{\textrm{d} t}\right)^2} }} \ \mathrm{d}t \ \right|_{\delta =
0}
```

```math
= \int_{t=0}^1 {{ \sum_{i=1}^n \frac{\mathrm{d} \eta_i}{\mathrm{d}
t} \frac{\textrm{d} x_i}{\textrm{d} t} } \over { \sqrt{\sum_{i=1}^n
\left(\frac{\textrm{d} x_i}{\textrm{d} t}\right)^2} }} \ \mathrm{d}t
```

```math
= \int_{t=0}^1 \frac{1}{L'} \sum_{i=1}^n \frac{\mathrm{d}
\eta_i}{\mathrm{d} t} \frac{\textrm{d} x_i}{\textrm{d} t} \ \mathrm{d}t
```

As described in [Feynman's lecture on the Principle of Least
Action](https://www.feynmanlectures.caltech.edu/II_19.html), we'd like
this equation in terms of $H$, rather than its derivatives, so we can
integrate by parts and use the fact that $H(0) = H(1) = 0$:

```math
\frac{\mathrm{d}L_H}{\mathrm{d}\delta} = \int_{t=0}^1 \frac{1}{L'}
\sum_{i=1}^n \frac{\mathrm{d} \eta_i}{\mathrm{d} t} \frac{\textrm{d}
x_i}{\textrm{d} t} \ \mathrm{d}t
```

```math
= \left. \frac{1}{L'} \sum_{i=1}^n \eta_i \frac{\textrm{d} x_i}{\textrm{d}
t} \right|_{t = 0}^1 - \int_{t=0}^1 \sum_{i=1}^n \eta_i
\frac{\mathrm{d}}{\mathrm{d} t} \left(\frac{1}{L'} \frac{\textrm{d}
x_i}{\textrm{d} t}\right) \ \textrm{d} t
```

```math
= - \int_{t=0}^1 \sum_{i=1}^n \eta_i \frac{\mathrm{d}}{\mathrm{d} t}
\left(\frac{1}{L'} \frac{\textrm{d} x_i}{\textrm{d} t}\right)
\ \textrm{d} t
```

At this point, life is a lot easier if we reparameterise $L$ to move
at a constant speed - that is, $L'(t) = c$ for some $c$. We can do
this without loss of generality. This gives:

```math
\frac{\mathrm{d}L_H}{\mathrm{d}\delta} = - \int_{t=0}^1 \sum_{i=1}^n
\eta_i \frac{\mathrm{d}}{\mathrm{d} t} \left(\frac{1}{c}
\frac{\textrm{d} x_i}{\textrm{d} t}\right) \ \textrm{d} t = -
\frac{1}{c} \int_{t=0}^1 \sum_{i=1}^n \eta_i \frac{\mathrm{d}^2
x_i}{\mathrm{d} t^2} {\textrm{d} t}
```

If there were no constraints between the $\eta_i$, the only way
to have this be zero for all possible $\eta_i$ is to have
$\frac{\mathrm{d}^2 x_i}{\mathrm{d} t^2} = 0$. And that's just
saying the path should be a straight line!

If we're trying to deal with a curved manifold embedded in a
higher-dimension space, however, we've got a bunch of constraints
between the $\eta_i$.

Pedantry time: We expect our embedded manifold to be curved, so that
if $P(t)$ forms a path in that manifold, $P(t) + \delta H(t)$ won't,
for general $\delta$. What we can do is replace it with $P(t) + H(t,
\delta)$, where the latter is a function that makes the sum a curve
inside the manifold, and whose magnitude is bounded by $\delta$, so
that the limit still works, albeit with more tedious analysis than
simply taking the derivative.

I've been lead to believe people use "Lagrange multipliers" to handle
this constraint, but I'll do it my own way:

If the manifold is of dimension $m$, we'll say that locally there's a
smooth 1-1 mapping $E$ from $\mathbb{R}^m$ to the $\mathbb{R}^n$ that
embeds the manifold, representing the points in the manifold, $(x_1,
\ldots, x_n) = E(\overline{x}_1, \ldots, \overline{x}_m)$. We'll then
make $(\overline{x}_1, \ldots, \overline{x}_m) = \overline{P}(t)$ into
a path in the $m$ dimensional space such that $P = E \circ
\overline{P}$.

Similarly, we can define $\overline{H}$ as the $m$ dimensional
equivalent of $H$. As $H$ is a perturbation of $P$, the transformation
from $\overline{H}$ to $H$ is a little more complicated:

```math
\eta_i = \sum_{j = 1}^m \frac{\textrm{d} x_i}{\textrm{d}
\overline{x}_j} \overline{\eta}_j + O(\overline{\eta}_j^2)
```

Assuming quadratic terms don't matter and plugging this back into the
previous equation gives

```math
\frac{\mathrm{d}L_H}{\mathrm{d}\delta} = - \frac{1}{c}
\int_{t=0}^1 \sum_{i=1}^n \eta_i \frac{\mathrm{d}^2 x_i}{\mathrm{d}
t^2} \ \textrm{d} t
```

```math
= - \frac{1}{c} \int_{t=0}^1 \sum_{i=1}^n \sum_{j = 1}^m
\frac{\textrm{d} x_i}{\textrm{d} \overline{x}_j} \overline{\eta}_j
\frac{\mathrm{d}^2 x_i}{\mathrm{d} t^2} \ \textrm{d} t
```

```math
= - \frac{1}{c} \int_{t=0}^1 \sum_{j = 1}^m \overline{\eta}_j
\sum_{i=1}^n \frac{\textrm{d} x_i}{\textrm{d} \overline{x}_j}
\frac{\mathrm{d}^2 x_i}{\mathrm{d} t^2} \ \textrm{d} t
```

As the $\overline{\eta}_j$ are properly independent, the
constraint on our curve is

```math
\sum_{i=1}^n \frac{\textrm{d} x_i}{\textrm{d} \overline{x}_j}
\frac{\mathrm{d}^2 x_i}{\mathrm{d} t^2} = 0
```

(with $L' = 1$).

Put another way, for each direction within the locally-flat region,
the curvature is zero. All curvature is kind of invisible from the
"inside", which I guess makes sense?

## Why does "extrapolate and find nearest point" work?

My initial "intuitive" approach to plotting a geodesic on a 2D curved
surface embedded in $\mathbb{R}^3$ was to extrapolate the line from
the last 2 points in 3D, and then move the point found to the nearest
point on the surface.

Moving to the nearest point on the surface moves it along a vector
perpendicular to the surface, so that the curvature being introduced
is outside the "locally Euclidean" surface.

## Following the curvature of a height map

The solution we have at the moment works for a general $m$ dimensional
manifold embedded in $n$ dimensional space. What about the simple
special case where we have a 2D surface in 3D space, and the Z
component is simply a (uni-valued) function of X and Y?

Let's work it out!

Let's say $(x, y, z) = E(\overline{x}, \overline{y})$. We'll set $x =
\overline{x}$ and $y = \overline{y}$. Then taking our constraint above
and expanding out the summation, we get:

```math
\begin{array}{ccc}
x: & \frac{\mathrm{d} x}{\mathrm{d} \overline{x}} \frac{\mathrm{d}^2 x}{\mathrm{d} t^2} + \frac{\mathrm{d} y}{\mathrm{d} \overline{x}} \frac{\mathrm{d}^2 y}{\mathrm{d} t^2} + \frac{\mathrm{d} z}{\mathrm{d} \overline{x}} \frac{\mathrm{d}^2 z}{\mathrm{d} t^2} = 0& \frac{\mathrm{d}^2 x}{\mathrm{d} t^2} +  \frac{\mathrm{d} z}{\mathrm{d} \overline{x}} \frac{\mathrm{d}^2 z}{\mathrm{d} t^2} = 0 \\
y: & \frac{\mathrm{d} x}{\mathrm{d} \overline{y}} \frac{\mathrm{d}^2 x}{\mathrm{d} t^2} + \frac{\mathrm{d} y}{\mathrm{d} \overline{y}} \frac{\mathrm{d}^2 y}{\mathrm{d} t^2} + \frac{\mathrm{d} z}{\mathrm{d} \overline{y}} \frac{\mathrm{d}^2 z}{\mathrm{d} t^2} = 0& \frac{\mathrm{d}^2 y}{\mathrm{d} t^2} +  \frac{\mathrm{d} z}{\mathrm{d} \overline{y}} \frac{\mathrm{d}^2 z}{\mathrm{d} t^2} = 0
\end{array}
```

We can rearrange these as the constraints

```math
\frac{\mathrm{d}^2 z}{\mathrm{d} t^2} =
- \frac{\mathrm{d}^2 x}{\mathrm{d} t^2} / \frac{\mathrm{d} z}{\mathrm{d} \overline{x}} =
- \frac{\mathrm{d}^2 y}{\mathrm{d} t^2} / \frac{\mathrm{d} y}{\mathrm{d} \overline{x}} 
```

Combined with our earlier constraint that $L' = 1$, this gives us what
we need to plot a path step-by-step (or check that a path locally is
minimal).

## TODOs

 * Future work: understand generalised curvature.
