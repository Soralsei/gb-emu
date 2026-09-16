# Design note: the clock as an async executor

Status: **partially implemented**. Steps 1 and 2 of "Order of work" have landed:
`Timeline`, `Wait`, `Domain`, `spawn` and `poll_tasks` are in
`backend/src/clock.rs`, and the PPU runs as a task spawned from `System::new`.
The timer, serial and OAM DMA are still `Clocked`, the CPU still drives the
clock, and nothing in "Savestates" exists. The rest records a design worked out
in discussion so it can be picked up later.

## Why

`Clock` is already an executor with the interesting part removed. `Clocked::step`
is `poll` without a `Waker` and without a compiler-generated resume point: every
device re-enters at the top of `step` and hand-restores its position from a mode
enum plus a cursor. That hand-restoring is the state machine the compiler will
write for you.

The payoff is not uniform. It lands on devices whose behaviour is a *sequence*:
the PPU mode walk, the pixel fetcher, OAM DMA, serial. It lands on nothing for
devices that are a counter with an edge detector — the timer gains zero.

```rust
async fn ppu(s: Rc<PpuState>, t: Timeline) {
    loop {
        for ly in 0..154 {
            s.ly.set(ly);
            if ly < 144 {
                s.mode(2); t.wait(80).await;                  // OAM scan
                s.mode(3); let dots = fetch_line(&s, &t).await;
                s.mode(0); t.wait(376 - dots).await;          // HBlank
            } else {
                s.mode(1); t.wait(456).await;
            }
        }
    }
}
```

There is no performance argument either way. The executor polls every task every
tick, which is exactly what `Clock::tick` does now; a suspended future's `poll`
costs about what an early-returning `step` costs. This is a legibility change,
not an optimisation — see "No scheduling at all" below for why the tempting
optimisation is not worth its complexity yet.

### Relationship to `codegen/MICRO_OPS.md`

That note wants to invert the clock direction (clock drives CPU) and says the
cost is that instructions must be suspendable between M-cycles, which forces a
generated micro-op schedule per opcode.

If the CPU is also a task, suspension is free and **the sequencer is not needed**.
The instruction body's straight-line control flow stays the state machine, as it
is today, but becomes resumable:

```rust
async fn cpu(...) {
    loop {
        let opcode = fetch(&bus).await;
        match opcode {
            0x34 => { let v = bus.read(hl).await; bus.write(hl, v.wrapping_add(1)).await; }
            /* ... */
        }
    }
}
```

`inc (hl)`'s read and write land on separate M-cycles because there are two
awaits, not because a schedule placed them. The `Src`/`Dst` split in MICRO_OPS.md
§"The `Src`/`Dst` change this depends on" is still worth doing — it is what makes
bus access an explicit, awaitable act — but the `MicroOp`/`BusAction` machinery
and the generator's schedule-shape table drop out.

What does *not* drop out is the research. The per-M-cycle bus schedule for every
opcode is the same knowledge either way, expressed as await placement rather than
as a `[MicroOp; 8]`. The invariant `1 + pushed_ops == cycles / 4` becomes "awaits
in the arm == `cycles / 4`", which the generator can assert against
`instructions.yml` rather than derive from it — a check it does not have today.

The conditionals table in MICRO_OPS.md stops needing to exist: `ret cc` at 8
cycles not taken and 20 taken falls out of an `if`, with the unconditional
internal cycle simply preceding it. Interrupt dispatch is a function with five
awaits instead of a pushed 5-op sequence, and the EI delay is a local in the
loop instead of a flag applied "next time the queue empties".

One constraint: do **not** emit one `async fn` per opcode. Each would be a
distinct future type, so dispatch needs `Box<dyn Future>` — an allocation per
instruction, at ~1M instructions/second. Emit a single `async fn` with a `match`
over the opcode. One future type, one state machine, zero allocation; its size is
the max over all arms.

This note does not fix the ordering problem MICRO_OPS.md flags (effects applied
before the remaining cycles are ticked, so writes land at the wrong M-cycle
offset). That is the same question either way, and it is fixed by the direction
inversion, not by async.

## Runtime

Roughly 40 lines, no dependencies. `core::future`, `core::task`, `Pin<Box<dyn
Future>>` are all std — `Cargo.toml` stays as clean as it is now. No tokio, no
`futures` crate; a work-stealing multi-threaded runtime is the opposite of what
this needs.

### No scheduling at all

The executor keeps no deadlines and no ready queue. It advances `now` and polls
every task, unconditionally, exactly as `Clock::tick` loops over its device
vectors today. A suspended future's `poll` is a deref, a discriminant load and
one `u64` comparison — with ~5 tasks at ~1.05M M-cycles per second, the cost does
not register.

The waker is inert, so `Waker::noop()` does. It is stable since 1.85; on an older
toolchain it is ~15 lines of `RawWaker`/`RawWakerVTable`.

The reason to start here is *not* that per-task deadlines are awkward to build.
They are not — see below. It is only that there are five tasks and no measured
win. Revisit with a profile, or when the CPU becomes a task and the executor
free-runs, at which point it can jump `now` straight to the earliest deadline and
skip cycles outright, making HALT and VBlank genuinely free rather than cheap.

### If you add deadlines, use a real `Waker`

The tempting shortcut is a shared `next: Cell<Cycles>` that the executor clears
before `poll` and reads after. Do not. A rendezvous slot needs an *unset*
representation (a sentinel), needs a min rather than a store (under
`join!`/`select!` several children register in one poll and a store loses the
earliest), and gives no way to tell "registered nothing" from "finished".

A `Waker` removes all three, because registration becomes one-way — the future
pushes, the executor only pops:

```rust
fn poll(mut self: Pin<&mut Self>, cx: &mut Context) -> Poll<()> {
    if self.now.get() >= self.at {
        return Poll::Ready(());
    }
    if !self.armed {
        self.timers.borrow_mut().push((self.at, cx.waker().clone()));
        self.armed = true;
    }
    Poll::Pending
}
```

The executor advances `now`, drains entries with `at <= now`, calls `wake()` on
each — which sets a bit in a ready mask — and polls the tasks whose bit is set.
At five tasks a `Vec` with a linear scan beats a `BinaryHeap`.

Note what `Waker` is actually providing: not notification (the executor already
knows when every task becomes ready) but **task identity inside `poll`**. A boxed
future has no idea which slot it occupies, and `cx.waker()` is the only handle it
gets. That is what makes the push one-way.

One real caveat. `Waker` is unconditionally `Send + Sync`, so a
`RawWakerVTable` closing over `Rc`/`Cell` is unsound if a waker ever leaves the
thread. Either document a hard "wakers must not escape the executor thread"
invariant, which is what most single-threaded executors do, or use an `Arc`-based
waker and pay one uncontended atomic per registration.

Beyond timing, reach for real wakers if a task must wake on something that is
*not* a cycle count and cannot be expressed as shared state sampled at a known
cycle. STOP exit is `Clock::resume` at executor level, and the FIFO and the
fetcher live in the same task rather than signalling across two, so neither
qualifies.

The one that comes closest is serial idling between transfers: nothing to do
until the game sets SC bit 7, which is a bus write, not a cycle. Handle it with a
poll loop — `while !s.transfer_started() { t.wait(512).await }` costs ~8k polls
per second and needs no wake path. Reach for a real waker only if that pattern
starts appearing in several devices at once. The escape hatch is cheap either
way: `Timeline::wait` is the only thing that changes, and no device code moves.

```rust
type Cycles = u64;

/// A task's view of time. Holds the executor's counter for one time base, and
/// deliberately not the executor itself: the executor owns the tasks, so an
/// `Rc<Executor>` inside a task would be a reference cycle that never frees.
#[derive(Clone)]
struct Timeline(Rc<Cell<Cycles>>);

impl Timeline {
    fn wait(&self, n: Cycles) -> Wait {
        Wait { now: self.0.clone(), at: self.0.get() + n }
    }
}

struct Wait {
    now: Rc<Cell<Cycles>>,
    at: Cycles,
}

impl Future for Wait {
    type Output = ();
    fn poll(self: Pin<&mut Self>, _: &mut Context) -> Poll<()> {
        if self.now.get() >= self.at { Poll::Ready(()) } else { Poll::Pending }
    }
}
```

`at` is absolute and is resolved when `wait` is called, which is the resume
point — so it stays correct when the executor advances in batches larger than
`n`. `wait(0)` completes inside the same poll, which is what makes
`boundary(0, ..)` below cheap. `join!` and `select!` need no special handling:
each child compares against the same `now` and there is no shared slot to clobber.

Tasks never complete, and that is worth making a type-level fact rather than a
runtime convention. `Output = Infallible` with an infinite loop: the body has
type `!`, which coerces, so the compiler enforces that the loop has no exit.
There is no `Ready` arm to write — `let _ = poll(..)` is the whole call site,
because the value it would discard cannot exist.

A task is just a pinned future. The savestate boundary flag lives in the device's
own shared state (see "Resume descriptors"), which the savestate path already
reads, so the executor needs nothing from a task but the ability to poll it.

`spawn` takes a closure rather than a future because the `Timeline` has to exist
before the future that captures it, and the executor is the only thing that
should be minting `Timeline`s. `std::thread::spawn` has the same shape for the
same reason — it takes a closure, not a running thread.

```rust
struct Task(Pin<Box<dyn Future<Output = Infallible>>>);

fn spawn<F, Fut>(&mut self, domain: Domain, task: F)
where
    F: FnOnce(Timeline) -> Fut,
    Fut: Future<Output = Infallible> + 'static,
{
    let timeline = Timeline(match domain {
        Domain::Cpu => self.now_cpu.clone(),
        Domain::Fixed => self.now_fixed.clone(),
    });
    self.tasks.push(Task(Box::pin(task(timeline))));
}

// exec.spawn(Domain::Fixed, |t| ppu(ppu_state.clone(), t));

fn tick(&mut self, elapsed: u16) {
    if elapsed == 0 || self.stopped.get() {
        return;
    }
    let shift = self.speed.get() as u8;
    self.now_cpu.set(self.now_cpu.get() + elapsed as Cycles);
    self.now_fixed.set(self.now_fixed.get() + (elapsed >> shift) as Cycles);

    let mut cx = Context::from_waker(Waker::noop());
    for task in &mut self.tasks {
        let _ = task.0.as_mut().poll(&mut cx);
    }
}
```

Two time bases replace `attach` / `attach_fixed`, but only `spawn` needs to know
which is which: the choice is made once, into the `Timeline`, and `tick` treats
every task alike. Double speed stays a shift on one counter. `stop`, `resume`,
`switch_speed` and `key1` carry over from `Clock` unchanged; `reset_div` changes,
see below.

The other three shapes for handing a task its `Timeline` were considered and are
worse, so do not re-derive them:

- **caller constructs it** — puts executor plumbing in every call site, for a
  type the caller has no business knowing the innards of
- **ambient thread-local, rebound per poll** — what `tokio::time::sleep` does,
  and it is the most ergonomic, but it is mutable global state and makes
  "awaited outside a poll" a silent stale read rather than a compile error
- **executor hands one out inline**, `exec.spawn(ppu(s, exec.timeline(..)))` —
  compiles under two-phase borrows, but still leaves the plumbing in the
  argument list

### Invariants that are not optional

- **Poll order is fixed.** `Vec`, indexed, spawn order. Never a `HashMap`, never
  `FuturesUnordered` — it reorders on wake and would make the core
  non-deterministic. Determinism is load-bearing for the savestate scheme below.
- **Nothing that must survive a reset or savestate may live only in a local
  across `.await`.** The compiler will not check this. One forgotten local is a
  corrupt state restore, and it will present as a hardware-timing bug.
- **Big buffers stay in `Rc`, not in locals.** Every local held across an await
  lives in the future, and the future is sized for the worst path.

## State split

The win only materialises when transient state is *locals*. But MMU handlers read
PPU registers while the PPU is parked mid-scanline (`system.rs`, the `0xFF40`
ranges), so registers must stay reachable from outside the task.

So each device splits in two:

- **shared** — `Rc<State>` with interior mutability: registers, VRAM, OAM,
  framebuffer. Reachable by memory handlers, and the thing a savestate
  serialises.
- **transient** — locals in the async fn: fetcher step, latched tile bytes, the
  X counter, the mode cursor.

This is already half-done — `Ppu` is an `Rc` with `Cell`/`RefCell` inside. Async
makes the line explicit rather than implicit, and the split matches the hardware
(the register file is externally visible, the fetcher's internal latches are not).

## The divider

`Clocked::reset_div` does not survive, because a task is an opaque future with no
second method to call. The fix is not a wake channel — the divider is in the
wrong place today, independently of any of this.

There is one 16-bit counter, incremented at the T-cycle rate. DIV (`0xFF04`) is
its upper 8 bits, TIMA is a falling-edge detector on a selected bit of it, and
the APU frame sequencer is a second detector on bit 12 (13 in double speed). The
counter is system state with several consumers. It currently lives in
`TimerState.counter`, with `Clock` reaching in to zero it — which is precisely
why `reset_div` had to become a trait method broadcast over every CPU-clocked
device.

Move it onto the executor, where it is derived rather than stored:

```rust
div_epoch: Cell<Cycles>,

fn div(&self) -> u16 { self.now_cpu.get().wrapping_sub(self.div_epoch.get()) as u16 }
fn reset_div(&self) { self.div_epoch.set(self.now_cpu.get()); }
```

`Clocked::reset_div` is then deleted, and `Clock::stop` / the speed switch set
their own field instead of broadcasting. Consequences:

- no per-tick work for the divider at all; it is a subtraction on read
- double speed is already correct, because `now_cpu` is what runs at 2×
- STOP freezes DIV for free, since `now_cpu` stops advancing
- one `u64` to serialise, and the APU can share the counter later without a
  second copy or a second `reset_div` recipient

The timer keeps the `0xFF04`–`0xFF07` handler and takes an `Rc<Clock>`: it reads
`clock.div()` and calls `clock.reset_div()` on a DIV write, then runs its own
edge detect. It stays a `Clocked` device; nothing about it is a sequence, so it
gains nothing from being a task.

Edge detection needs a before and an after, and the obvious move — keep the
previous sample in the timer — undoes the whole change: that sample goes stale on
a DIV reset, so the timer would have to be *told* about resets, which is the
notification being deleted. Derive the window instead. The timer knows `elapsed`,
and a reset can only land between ticks (it comes from a bus write, STOP, or the
speed switch), so `[div() - elapsed, div()]` is exactly the span this tick covers:

```rust
impl Clocked for Timer {
    fn step(&self, elapsed: u16) {
        let end = self.clock.div();
        let start = end.wrapping_sub(elapsed);
        let mut state = self.state.borrow_mut();
        for k in (M_CYCLE..=elapsed).step_by(M_CYCLE as usize) {
            // Compares the selected bit at `c - M_CYCLE` against `c`, and
            // applies the pending TMA reload, in that order.
            state.advance_to(start.wrapping_add(k));
        }
    }
}
```

Nothing counter-shaped is stored. `TimerState` keeps `tima`, `tma`, `tac` and
`overflowed` — all genuinely the timer's own registers — and the DIV write path
reads `clock.div()`, detects the edge against `0`, then calls
`clock.reset_div()`.

The walk exists only because a tick can currently carry several M-cycles.
Once the CPU is clock-driven and every tick is exactly one M-cycle, the loop
collapses to a single `advance_to(clock.div())`.

Note an existing asymmetry worth preserving deliberately rather than by accident:
a write to `0xFF04` goes through `state_change(0, tac)` and so produces the
spurious TIMA increment, while STOP and the speed switch zero `counter` directly
and produce no edge. Keep that split when moving the counter.

## Cycle accuracy

Unaffected, and arguably helped. Deadlines *are* absolute cycle counts:
`t.wait(1).await` is one dot. When the executor advances `now` by 4 on an M-cycle
bus access, the PPU task runs four dot-iterations in one poll and parks. Timing
is identical to a hand-written dot loop.

The M-cycle is the smallest quantum at which any *write* is observable, because
every writer — the CPU and OAM DMA alike — moves at M-cycle granularity. Batching
4 dots loses nothing there.

### What batching does lose: ordering inside the lump

Cycles are conserved. `Timeline::wait` is cumulative (`at + n`, not `now + n`)
and `Wait::poll` is `now >= at`, so a task that wanted to wake at +2 wakes at +4
and its next `wait(2)` returns `Ready` in the same poll — the state machine runs
forward until it genuinely blocks. Nothing is dropped.

What is lost is the *interleaving*. `Clock::tick(4)` advances `now` by 4, steps
both device vectors by 4, then polls tasks once. So the PPU burns four dots in
one go and the CPU's bus access resolves against whatever state that left behind.

`CpuBus::read` makes it concrete: it calls `clock.tick(M_CYCLE)` and *then* asks
`bus_controller.conflict(address)`. The PPU has already run its four dots, so the
CPU samples the bus at the end of the M-cycle rather than at the dot its access
actually occupies. Any mode-3 edge that turns on when the VRAM lock is taken or
released relative to a CPU access is decided at the wrong resolution.

This is the same defect `codegen/MICRO_OPS.md` names from the CPU side — "`spend()`
ticks an instruction's remaining cycles *after* its effects are applied, so writes
land at the wrong M-cycle offset". One cause, two symptoms.

The fix is to make the tick atomic at its real granularity: `tick(1)`, four times,
rather than `tick(4)` once. Deadlines then land on the dot they name and devices
interleave as they do on the hardware. Cost is 4× the poll rate — `poll_tasks`
iterates every task plus both device vectors per tick, so ~4.19M polls/second
instead of ~1.05M. That is the "No scheduling at all" argument cashed at four
times the rate; it still holds at five tasks, and it is the first thing to revisit
with a profile.

Worth doing before the WX ≤ 7 and mode-3 penalty work, not after. Those are
exactly the quirks that live in the dots this flattens.

The genuinely hard PPU-timing cases are all "at dot N, sample a register, maybe
restart a 5-step sequence":

- mode 3 length penalties: `SCX & 7` discard, window trigger mid-line, sprite
  fetch abort and restart
- `LCDC` bit 4 / bit 0 flipped mid-scanline, observed at the exact dot the
  fetcher reads it
- fetcher stall while the FIFO holds ≥ 8 pixels; sprite fetch preempting a BG
  fetch mid-step
- STAT IRQ blocking, LY=LYC off-by-one-dot

Linear `await` code states each of those directly. In the mode-enum form you have
to prove every entry path restores the right cursor, which is where those bugs
live.

Cost is one poll per dot for the PPU — ~4.19M polls/second, each a discriminant
jump. Not a concern.

## Savestates

This is the real decision, and it is the reason to get the state split right
from day one. There are no savestates in the tree today (no `serde` in
`backend/Cargo.toml`, no snapshot code), so nothing is being broken — but the
choice is expensive to reverse.

**A compiler-generated future cannot be serialised. Ever.** No reflection,
unstable layout, self-referential when a borrow crosses an await. Byte-copying a
`Pin<Box<F>>` breaks absolute self-pointers unless restored at the same address.
Do not go down that road. Do not snapshot the future — *reconstruct* it.

### Scheme: rolling frame snapshot + deterministic replay

A savestate taken at cycle `T` is not a snapshot of `T`:

```
{ snapshot of shared state at frame boundary B, joypad deltas in [B, T), T - B }
```

Loading restores `B` and replays forward `T - B` cycles with the recorded input,
landing exactly on `T`. Exact by construction, because the core is deterministic.

| | cost |
| --- | --- |
| save | one memcpy of shared state per frame (~50 KB CGB) ≈ 3 MB/s amortised |
| load | replay ≤ 70224 cycles ≈ 1 ms |
| blob | snapshot + a handful of joypad events + a `u32` offset |

Loading stops being O(1) and becomes O(one frame of emulation) — 1/60th of the
work already done every frame. For a rewind ring, consecutive entries share the
frame snapshot; store offsets only.

Replay must be **whole-system**. The CPU writes VRAM between `B` and `T`, so the
PPU cannot be replayed in isolation against a stale VRAM. This is also why
shrinking the boundary to a scanline does not reduce the memcpy — it multiplies
it by 154. The frame boundary is the right point.

### Resume descriptors

The frame boundary must be universally quiescent, so anything that straddles one
needs a small serialisable descriptor written into shared state just before it
parks:

This is device state, not executor state — the descriptor is a field of the
device's own shared struct, which is exactly what the savestate path serialises
anyway. `Timeline` stays a bare counter and the executor learns nothing new:

```rust
// in the device's shared state
resume: Cell<Option<Resume>>,   // Some(..) == parked at a boundary, and where
```

The task sets it immediately before parking and clears it on the next `wait`:

```rust
s.resume.set(Some(Resume::Line(ly)));
t.wait(0).await;                       // the boundary
s.resume.set(None);
```

A snapshot is only taken once every device reports `Some(..)`. The frontend asks
for a savestate, the executor sets a flag and runs to quiescence. One field does
both jobs — whether it is safe to snapshot, and what to restore — so there is no
separate `safe` flag to keep in step with it.

| device | descriptor |
| --- | --- |
| OAM DMA | `Byte(u8)` — 160 M-cycles, straddles frames |
| serial | `Bit(u8)` |
| PPU | `Line(u8)` |
| CPU (if it becomes a task) | none — parks at an instruction boundary |
| timer, joypad | none — stays `Clocked`, plain counters |

The CPU needs no descriptor because the top of its loop *is* a natural boundary,
but that imposes two things. `B` becomes "the frame boundary, rounded up to the
next instruction boundary" — at most ~24 cycles later, since quiescence cannot be
declared mid-instruction. And the state split applies to the CPU too: the
register file, `SP`, `PC` and `IME` are shared state, only per-instruction
scratch (the operand latch, a decoded address) may be a local across an await.
`IME` is the trap — nothing on the bus can read it, so it looks like a local, and
losing it across a restore is silent.

The descriptors you hand-write are exactly the ones that were trivial anyway. The
pixel fetcher — the code this whole design is for — needs none, because you can
never snapshot inside it.

The CPU's mid-instruction state would otherwise be the worst offender here, and
replay is what makes an async CPU viable at all.

### Determinism requirements

Replay is only exact if the core is bit-deterministic: no `HashMap` iteration in
the tick path, no wall clock, no uninitialised reads, stable task poll order. All
of that is wanted for testing and TAS regardless.

## Order of work

1. Executor beside `Clock`, keeping `Clocked` for timer and joypad. Both models
   run at once; nothing has to be ported in one go.
2. PPU and `graphics/fifo.rs` first — biggest payoff, and the fetcher is
   unfinished, so it is new code rather than a rewrite of something that works.
3. Serial and OAM DMA, if it feels good.
4. `Resume` descriptors and the snapshot/replay path only when savestates are
   actually being added. That work is additive; the state split is not.
5. The CPU last, and only together with the direction inversion in
   `codegen/MICRO_OPS.md`.
