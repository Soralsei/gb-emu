# Design note: the clock as an async executor

Status: **partially implemented**. Steps 1 to 3 of "Order of work" have landed.
`Timeline`, `Wait`, `Domain`, `spawn` and `poll_tasks` are in
`backend/src/clock.rs`; the PPU, timer, serial and OAM DMA all run as tasks
spawned from `System::new`, and the `Clocked` trait is deleted. The divider is
derived on the clock. `Clock::tick` advances one T-cycle and takes no `elapsed`,
so the batching fix in "What batching does lose" is in.

Not done: the CPU still drives the clock rather than being a task — see "The CPU
as a task", which is step 5 and the last of the migration — and nothing in
"Savestates" exists. Step 5 is underway: `W`/`Z` are in the register file and
`Src`/`Dst` are gone, which leaves `operations.rs` not building until 5.3. The
rest records a design worked out in discussion so it can be picked up later.

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

## The CPU as a task

The CPU is the last device, and the one the rest of this note was shaped around.
`codegen/DISPATCH.md` covers what the generator emits; this covers what the
`backend` side has to become. The two are meant to be read together.

### The sequencer is not needed

`codegen/MICRO_OPS.md` — now `DISPATCH.md` — wanted to invert the clock direction
and said the cost was a generated micro-op schedule per opcode, because an
instruction has to be suspendable between M-cycles.

If the CPU is a task, suspension is free. The instruction body's straight-line
control flow stays the state machine, as it is today, but becomes resumable:

```rust
0x34 => {                                    // inc (hl)
    let addr = cpu.addr(Reg16::HL);
    cpu.load(Reg8::Z, addr).await;
    cpu.alu(Reg8::Z, inc);
    cpu.store(addr, Reg8::Z).await;
}
```

The read and write land on separate M-cycles because there are two awaits, not
because a schedule placed them. That also fixes the ordering defect flagged
below and in the old note — `spend()` ticking an instruction's remaining cycles
*after* its effects are applied — because there are no remaining cycles to tick.
`mem_timing/03-modify_timing` is the check.

### The bus comes out of `Src`/`Dst`

Those traits used to resolve *where* a value lives and decide *when* the bus cycle
happens, in one call. The second job belongs to the schedule. So `Imm8`, `Imm16`,
`Mem<T>` and `DMem<T>` are deleted and replaced by a `W`/`Z` scratch pair in the
register file, with `WZ` as the 16-bit view — the latches the hardware has.

`Src`/`Dst` then abstract over `Reg8` and `Reg16` and nothing else, which
`Registers::read_u8`/`read_u16` already are, so both traits go too.

Then take the register file out as well. Every synchronous operation reduces to
`(&mut Flags, values) -> value`, and the ones that need more — `jr`, `jp`, `call`,
`ret`, `reti`, `rst`, `push`, `pop`, `ei`, `di`, `halt`, `stop` — are exactly the
set becoming `async fn` on `&Cpu` anyway, because they move PC, SP or IME. Nothing
is left over. `DISPATCH.md` has the signature table.

`operations.rs` becomes an ALU module that knows nothing about a `Cpu`, a bus or a
register file, which makes `daa`, `adc`, `sbc` and `add16` testable on a bare
`Flags`. Those four are where flag bugs live, and testing one currently means
constructing a machine.

Note the `W`/`Z` decision cuts against "nothing that must survive a savestate may
live only in a local across `.await`" in the opposite direction from usual — the
operand latch was the example of something that *may* be a local, and making it a
register field is strictly safer. Take the safer side; the cost is two bytes in a
snapshot.

### Shape

The CPU joins the other devices: `Rc<Cpu>` with interior mutability, `&self`
methods, `Cpu::task(this, timeline)`.

Not `Rc<RefCell<Cpu>>`. The task would hold the borrow across awaits, and the
debugger's disassembly panel repaints *while the task is parked mid-instruction*
— see `DEBUGGER.md` §"Repaint pacing". That is a `RefCell` panic waiting for
someone to open the panel while paused. Registers live in a `RefCell<Registers>`
borrowed per operation call and never across an await.

```rust
pub struct Cpu {
    registers: RefCell<Registers>,
    ime: Cell<bool>,
    instr: Cell<u64>,          // step boundary, ring-buffer sequence, replay resume point
    bus: CpuBus,
    timeline: Timeline,        // Domain::Cpu
    fixed: Timeline,           // Domain::Fixed -- the speed-switch stall, and only that
    clock_control: ClockControl,
}
```

The awaiting surface is five methods, and they are the only async things in the
CPU outside the stack operations:

```rust
async fn fetch(&self, dst: Reg8);          // PC++, bus read -> dst
async fn load(&self, dst: Reg8, addr: u16);
async fn store(&self, addr: u16, src: Reg8);
async fn idle(&self);                      // an internal M-cycle
async fn cycle(&self);                     // timeline.wait(M_CYCLE), what the others are built on
```

`CpuBus::read`/`write` become **sync** and drop their `Rc<Clock>`: the wait moved
to `Cpu::cycle`, and arbitration is all that is left. One less `Rc` in
`System::new`.

### HALT and STOP stop being modes

This is the payoff predicted at the top of this note. `Cpu::Mode`, `is_stopped()`,
`resume()` and `spend()` all delete.

| was | becomes |
| --- | --- |
| `Mode::Halted` | `while !self.interrupt_pending() { self.idle().await }` |
| `Mode::Stopped` | `clock_control.stop()`, then await. `now_cpu` is frozen, so the `Wait` never completes and the task parks. `Clock::resume` restarts it. |
| `Mode::SwitchStalled` | `self.fixed.wait(2050 * M_CYCLE).await` |

Interrupt dispatch becomes a function with five awaits called at the top of the
loop, not a separate `handle_interrupts` the frontend has to remember to call.
The EI delay becomes a local in the loop rather than a flag, which is exactly
"after the following instruction".

### The one piece that needs new clock machinery

The speed-switch stall wants `now_cpu` frozen while `now_fixed` keeps running —
the PPU advances through the pause, DIV does not. `Clock::stop()` freezes both,
and the CPU can no longer call `tick_fixed()` itself from inside a task.

So: a `cpu_frozen: Cell<bool>` in `ClockState` honoured by `tick`, plus a way for
the CPU to hold a second `Timeline` on the other domain, since `spawn` mints
exactly one. The obvious shortcut — freeze `now_cpu` and have the CPU await on its
own timeline — does not work, because that is the timeline that stopped moving.

### Spawn order is load-bearing

`Clock::poll_tasks` polls in spawn order, and that order is part of the savestate
contract below. Where the CPU goes relative to the PPU and OAM DMA decides who
sees whose writes within a tick. Pick it deliberately and do not let it drift.

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

The timer keeps the `0xFF04`–`0xFF07` handler and takes a `ClockControl`: it
reads `clock.div()` and calls `clock.reset_div()` on a DIV write, then runs its
own edge detect. Nothing about it is a sequence, so it gains nothing from being a
task — but it became one anyway when `Clocked` was deleted, and as a task it is
three lines:

```rust
pub async fn task(this: Rc<Self>, timeline: Timeline) -> Infallible {
    loop {
        timeline.wait(M_CYCLE as Cycles).await;
        let mut state = this.state.borrow_mut();
        let counter = state.clock.div();
        state.advance_to(counter);   // TMA reload, then the falling edge, in that order
    }
}
```

Edge detection needs a before and an after, and the obvious move — keep the
previous sample in the timer — undoes the whole change: that sample goes stale on
a DIV reset, so the timer would have to be *told* about resets, which is the
notification being deleted. Derive the window instead. `advance_to` compares the
selected bit at `counter - M_CYCLE` against `counter`, and the wait guarantees
that is exactly the span since the last call.

An earlier draft of this had the timer walking `[div() - elapsed, div()]` in
M-cycle steps, because a tick could carry several M-cycles. `Clock::tick` is one
T-cycle now, so the walk collapsed to the single `advance_to` above.

Nothing counter-shaped is stored. `TimerState` keeps `tima`, `tma`, `tac` and
`overflowed` — all genuinely the timer's own registers — and the DIV write path
reads `clock.div()`, detects the edge against `0`, then calls
`clock.reset_div()`.

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

### What batching lost: ordering inside the lump

**Fixed.** `Clock::tick` advances one T-cycle and takes no `elapsed`; `CpuBus`
loops it `M_CYCLE` times. Recorded because the reasoning is the same one the
remaining CPU work turns on.

Cycles were always conserved. `Timeline::wait` is cumulative (`at + n`, not
`now + n`) and `Wait::poll` is `now >= at`, so a task that wanted to wake at +2
woke at +4 and its next `wait(2)` returned `Ready` in the same poll — the state
machine runs forward until it genuinely blocks. Nothing was dropped.

What was lost is the *interleaving*. `tick(4)` advanced `now` by 4 and polled
once, so the PPU burned four dots in one go and the CPU's bus access resolved
against whatever state that left behind. Any mode-3 edge that turns on when the
VRAM lock is taken or released relative to a CPU access was decided at the wrong
resolution.

That was the same defect `codegen/DISPATCH.md` names from the CPU side —
`spend()` ticking an instruction's remaining cycles *after* its effects are
applied, so writes land at the wrong M-cycle offset. One cause, two symptoms, and
the CPU half is still open: it closes when the CPU becomes a task and there are no
remaining cycles to tick.

Cost of the fix was 4× the poll rate, ~4.19M polls/second instead of ~1.05M. That
is the "No scheduling at all" argument cashed at four times the rate; it still
holds at five tasks, and it is the first thing to revisit with a profile.

Done before the WX ≤ 7 and mode-3 penalty work rather than after, because those
are exactly the quirks that live in the dots it was flattening.

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
| CPU | none — parks at an instruction boundary |
| timer, joypad | none — plain counters |

The CPU needs no descriptor because the top of its loop *is* a natural boundary,
but that imposes two things. `B` becomes "the frame boundary, rounded up to the
next instruction boundary" — at most ~24 cycles later, since quiescence cannot be
declared mid-instruction. And the state split applies to the CPU too: the
register file, `SP`, `PC` and `IME` are shared state, only per-instruction scratch
may be a local across an await. `IME` is the trap — nothing on the bus can read
it, so it looks like a local, and losing it across a restore is silent.

The operand latch was the example of something that *may* be a local. It no longer
is: `W`/`Z` are register-file fields, for the reasons in "The bus comes out of
`Src`/`Dst`". Two bytes more in a snapshot, one less thing to get wrong.

The `instr` counter on the CPU earns its keep three times over — step granularity
for `DEBUGGER.md`, sequence numbers for its ring buffer, and a resume point for
replay that is better than a cycle count, which collides across a speed switch.

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

1. ~~Executor beside `Clock`, keeping `Clocked` for timer and joypad.~~ Done.
2. ~~PPU and `graphics/fifo.rs`~~ — done. Biggest payoff, and the fetcher was
   unfinished, so it was new code rather than a rewrite of something that works.
3. ~~Serial, OAM DMA and the timer; `Clocked` deleted.~~ Done.
4. ~~One T-cycle per `tick`.~~ Done — "What batching lost", above.
5. **The CPU**, with `codegen/DISPATCH.md`. Last, and the only step that touches
   the generator. In order:
   1. ~~`W`/`Z` in `Registers`; `Reg8::W`/`Z`, `Reg16::WZ`.~~ Done; inert on its own.
   2. ~~Delete `Imm8`, `Imm16`, `Mem<T>`, `DMem<T>`, and then `Src`/`Dst`
      themselves.~~ Done. Breaks the build until 5.3.
   3. **Here.** `operations.rs` becomes flags-only: `(&mut Flags, values) ->
      value`, `Timing` deleted. Nine drop out entirely — `ld` `ld16` `ldi` `ldd`
      `inc16` `dec16` `add_sp` `ldhl` `nop` — because once the plumbing is emitted
      into the arm there is nothing left in them. The twelve that move PC, SP or
      IME become `async fn` on `&Cpu`. `offset_sp` stops reading its own immediate
      and takes a `u8`, which is what empties the generator's override table.
   4. Codegen: lowering, access modes, hole-filling, the three asserts, both
      outputs. Build green again.
   5. `Rc<Cpu>`, the five bus steps, the task, interrupt dispatch. `Mode`,
      `spend`, `is_stopped` and `handle_interrupts` delete; `CpuBus` goes sync.
   6. `cpu_frozen` and the second `Timeline` for the speed-switch stall.
   7. `System::step` on the `instr` counter, per `DEBUGGER.md`.

   `mem_timing/03-modify_timing` is the check on 5.5 — `inc (hl)`'s read and
   write land on cycles 2 and 3 for the first time.
6. `Resume` descriptors and the snapshot/replay path only when savestates are
   actually being added. That work is additive; the state split is not.
