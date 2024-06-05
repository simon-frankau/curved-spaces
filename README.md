# Simple Curved Space visualiser

![Obligatory screenshot goes here](curved-surfaces.png)

I'll come clean. My end goal is to visualise what a wormhole would
actually look like. Not a physically accurate one, but at least a
mathematically accurate one - what you'd get if you cut out a subspace
in a pair of $\mathbb{R}^3$ spaces and connected them with a
"tube". I'm nowhere near that, right now.

As a predecessor step, I'd like to do that for a pair of
$\mathbb{R}^2$ spaces. Stepping back further, I'd like to understand
how curved space behaves, mathematically, full stop. On that bit, I
feel I'm making progress.

More on that later. First, what is this, and how do you run it?

## What is this?

This program renders a curved 2D space (embedded in 3D, with a choice
of 2D surfaces), and lets you point a line through it, seeing how the
line curves over the space. There are various controls you can use,
I'm lazy (well, actually extremely low on energy) and I'll let you
figure them out by yourself, reading the source if necessary.

## Building and running

### Native

To run with glutin and winit:

```shell
cargo run --features=glutin_winit
```

To run with sdl2:

```shell
cargo run --features=sdl2
```

Running with sdl2 is not recommended, as the egui-based GUI is
disabled, effectively rendering it useless. This is really a stub for
future work.

### Web

To run with web-sys:

```shell
cargo build --target wasm32-unknown-unknown
mkdir -p generated
wasm-bindgen target/wasm32-unknown-unknown/debug/curved-space.wasm --out-dir generated --target web
cp index.html generated
```

`web.sh` has been provided to do this, for convenience. You may need
to do `cargo install wasm-bindgen-cli` first, if you haven't done wasm
work before.

CORS prevents you opening this as a file in a web browser, but you can
start a small local web browser, e.g. `python3 -m http.server 8080` in
the `generated` directory.

## The maths

There's a lot of maths behind differential geometry that I've never
really mastered. I have a copy of Darling's *Differential Forms and
Connections*, and the bits of it I've read are very good (and
differential forms are a pleasant generalisation of the 3D div, curl
and grad, and seem to have a lot in common with geometric algebra),
but I've not made it all the way through.

I felt that when it comes to understanding curved spaces, maybe I
should take a practical "work it out for yourself" approach.

I also still don't understand how General Relativity (GR) is supposed
to work, and understand the maths to be hard. Maybe I can try to
understand the behaviour of geodesics in curved space independent of
the physics?

So, here I am. The clever way of understanding curved space is as an
entity in itself. The dumb way is as a subspace of a higher-dimension
Euclidean space. IIUC they encompass the same spaces, but the former
is a much more elegant representation. I'm using the latter, hopefully
as a stepping stone to deeper understanding, as it's certainly the
most intuitive way to start.

### Starting off

Starting off, I ended up putting a lot of effort into just getting
anything to display in OpenGL, working on both web and native, with an
egui interface. The maths was very much secondary!

As such the kind of space we show is simply a uni-valued function in
the Z direction of values in the X and Y directions. As I go along,
I've been having ideas about how to generalise this to... more general
spaces. I have ideas, but I'm not clear yet. Do I want to use an atlas
of flattish pieces, or maybe an implicit surface?

**Note that my first cut of tracing an explicit representation has
been removed from `main`. If you want to see this phase of
development, look at the branch `explicit_and_implicit_surfaces`.**

Then, there's my approach for tracing a "geodesic" on the
surface. Scare quotes because I'm making this up as I go along, and
have no real mathematical basis so far.

The current approach is to trace the curve step by step by
extrapolating the line between the last two points to get an
approximate point in 3-space, likely off the 2-space surface (unless
it's locally perfectly flat), and then projecting that point down to
the nearest point on the 2-space surface (using the Euclidean 3-space
metric).

Why does this feel like it should work? Well, any three non-linear
points define a plane. By projecting that point from the linear
extrapolation down to the nearest point in the surface, I think we're
finding the plane that intersects the surface in the curve of least
curvature (i.e. closest to being a line).

Why do we want the least curved path? A straight line is the shortest
path between two points. I feel this is massively handwavey and
informal, yet enough to feel like something's about right here.

Qualitatively, it works, too! Positive curvature bends lines one way,
negative curvature the other. Hurrah.

### Checkpoint

Yet I've not got any real mathematical basis for this. It's just an
approach that feels right to me. Why am I even pushing this code in
this state? Well, my health is making me ultra-low-energy at this
point. I don't know how much progress I'll make in how long, so I want
to checkpoint my progress, such as it is. So, here we are.

### A failure to check the shortest path

The point of a geodesic is that it's the route with the shortest
length between the two end points. So, if I have an appropriximation
to that path made with a sequence of points (like I think I do), any
variation on those points should produce a longer path, right?

In the limit, dealing with the curve rather than a finite set of
points, this is the calculus of variations, but trying to check
optimality on a finite set of points will involve a lot less maths.

My idea is to perturb each point in turn along the path, and see if it
makes the path (locally) longer.

Does this work? No.

I'm approximating the curve with a series of line segments. If I move
a point perpendicular to the curve, and it's anywhere near optimal, a
movement of size $\delta$ leads to a change in length of size
$O(\delta^2)$. The size of the change of curve length is lost in the
error associated with the piecewise linear approximation to the curve.

Looks like proper maths will be needed after all.

### Success in checking the shortest path

Following the calculus of variations approach, I've come up with some
equations that represent the local version of "curve of minimal
length". This is going to be a well-established result, but it's fun
to rederive on your own.

As there's a bunch of equations and it goes on a bit, I've put my
derivation into [maths.md](./maths.md).

When I replace the naive curvature check with checking that the path
I'm generating matches these equations, it does look like the path
satisfies these equations, and a bad path does not. Success!

### More maths

I then took a slight detour in [maths2.md](./maths2.md) to explore the
structure of curved spaces without an embedding $\mathbb{R}^n$ space,
despite the fact it's not relevant to this coding project! This is
kind of a half-assed reinvention of the kind of maths used in General
Relativity.

### Following the geodesic equation

I then added code that attempts to follow the geodesic equation on the
explicit ( $z = f(x, y)$ ) formulation of the surface. Writing the
code, it's clear how similar it is to
extrapolate-and-find-nearest-point-in surface code that's already
implemented. In both cases, we take a linear extrapolation in the
embedding space, and then move the new point in the direction normal
to the surface until it hits the surface.

In the previous approach, the point was moved in a direction normal to
where the point is moved to. In this approach, the point is moved in
the direction normal to the point we extrapolated from.

In low curvature cases, both algorithms produce the same curve. In
high curvature, the change of normal within a step seems to be enough
to allow the curves to diverge. Or maybe there's a bug!

In either case, if the points aren't forced to be in the surface the
curves can become unstable and diverge from the surface. Forcing
points into the surface feels like a hacky mess, but seems necessary
unless I fix up my numerical methods.

Next, I plan to use an implicit representation for the surface, rather
than explicit. This will allow surfaces with overhangs. I will try to
get the numerical behaviour to work better in this domain, rather than
put much more effort into the explicit representation.

### Implicit surfaces

The next step was to support implicit surfaces, which would allow us
to create surfaces that curve 180 degrees, etc. It turns out that this
was easier to implement than expected, since it's uniform in all
dimensions, rather than having to special-case the Z direction.

The grids are created by tracing along lines of constant X and Y from
the edge of the grid (by constraining the curvature as the lines are
traced out). If the line curves around on one side and doubles back,
this can lead to a gap which needs to be addressed by other means.

At this point, I decided to remove support for explicit surfaces,
which simply represent development history, in order to simplify the
code. The point before explicit surfaces are removed is represented by
the branch `explicit_and_implicit_surfaces`.

### Numeric stability

Nice, bendy implicit surfaces allow for some fairly sharp curves that
can stop the solver from finding the next step within the surface. I
added some changes to stop you making the surfaces too messy, and
changed the solver to decrease the step size if there was too much
curvature otherwise, and this seems to have mostly done the trick. I
love half-baked numerical methods!

### And beyond

End goals are:

  * To be able to trace rays through a "wormhole" surface that can
    then be generalised into a curved space raytracer in 3D.

## Design choices on the coding side

I wanted to build something that could target web and native. It
didn't need to be particularly lightweight - on the web side, I just
wanted it to be accessible on the web, not neatly and lightly fitting
into some other web page. I wanted to use Rust as it's my favourite
programming language.

I wanted to use some form of modern OpenGL. I have experience of
old-school OpenGL, wanted the opportunity to learn the modern
pipeline, and felt it had a nice balance between being portable and
not requiring me to do wheel reinvention, as might be required for
more low-level graphics libraries.

Quite frankly, it probably makes more sense to do this kind of thing
in some kind of scripting language or even computational maths
environment. It would certainly allow me to cut to the chase. I just
like building things from the bottom up and having a lot of control. I
like to build production infra, and tend to work like that even for
projects that are entirely different.

When it comes to GL libraries, I chose
[glow](https://crates.io/crates/glow) as it seems to be simple,
popular, cross-platform (including wasm) and maintained.

For GUI control elements, I chose
[egui](https://crates.io/crates/egui), a nice immediate-mode
system. Previously I'd used it as the GUI framework for some tools
I've built, this time I'm using it as a library (i.e. it's no longer
"in control") to drive the UI elements.

Other than that, there are some pretty standard libraries, AFAICT.

There is something of a conspicuous lack of tests. It's not that the
code is untestable, it's more that it's more hassle than I want for a
simple hobby project like this. It's a bunch of UI code wrapping some
numerical code. Testing GUIs is a well-known pain. Testing numerical
code, in my experience, is usually very regression-testing-like:
checking that if you put these numbers in, you get these numbers
out. The problem is, as I toy around with that, trying to learn, I
don't really know what the right numbers are! So, no tests right now.
