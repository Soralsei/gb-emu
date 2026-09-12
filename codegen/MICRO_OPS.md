# Design note: generating micro-op schedules

Status: **not implemented**. This records a design worked out in discussion so it
can be picked up later. Nothing here is in the generator yet.

## Why

Today the CPU pulls the clock: `Mmu::read`/`Mmu::write` call `Clock::tick`, so
bus cycles land at the right offset inside an instruction because the CPU's own
straight-line control flow acts as the state machine. That is what commit
`1d6f3bb` introduced, and it is why `mem_timing` passes.

The hardware works the other way round — one oscillator drives the CPU, PPU,
timer and DMA alike, and the CPU is just another consumer. Inverting the
direction so `Clock` drives `Cpu::step` would:

- make HALT and STOP ordinary "this device did not advance" cases instead of
  special ones, and remove the `Clock::stopped` / `Cpu::mode` duplication;
- let OAM DMA and the CPU genuinely interleave, rather than DMA being stepped
  from inside a CPU bus access;
- make double speed uniform, since the CPU becomes a `cpu_clock` device like
  the timer.

The cost is that an instruction has to be suspendable between M-cycles, which
means the generator emits a schedule of micro-ops per opcode rather than a
single closure.

### When it is worth doing

Not yet. The current model already passes `mem_timing` and `instr_timing`. The
limit to watch for: `spend()` ticks an instruction's remaining cycles *after*
its effects are applied, so writes land at the wrong M-cycle offset. That is
invisible to the tests that pass today but will show up in PPU-timing tests and
anywhere OAM DMA contends with a CPU access mid-instruction.

Do not undertake this to remove the `Clock::stopped` duplication — that is a
two-line deletion, not a rewrite.

## What must be accurate, and what need not be

Nothing on the machine can observe the CPU's registers mid-instruction, so
internal computation does not need micro-architectural fidelity. There is no
need to model the real `W`/`Z` latches as such; any scratch field will do.

Bus accesses *are* observable, and their exact M-cycle is what the timing tests
measure. `gb-test-roms/mem_timing/source/` has three sub-tests that check
precisely this: `01-read_timing`, `02-write_timing`, and `03-modify_timing`.

`03` is the one that rules out running an instruction body atomically and
merely delaying around it. `inc (hl)` is 3 M-cycles — fetch, **read**, **write**
— with the read on cycle 2 and the write on cycle 3. An atomic body puts both on
the same cycle. Likewise `ld (a16),sp` writes on cycles 4 and 5, and `push bc`
on cycles 3 and 4. A one-cycle shift is visible: DIV advances between a
read-modify-write pair, and a VRAM write landing a cycle later can cross into
mode 3 and be blocked.

So: **the micro-op sequence is the bus access schedule**, not a
micro-architecture. ALU work, flag computation and register writes can happen in
one lump attached to whichever step carries the relevant bus access.

## Sequencer

A small queue of pending micro-ops, drained front to back. Ops are only appended
during the current instruction and two instructions are never in flight, so a
fixed array with a cursor is enough — no ring buffer, no wraparound. Longest
sequence is `call cc` taken at 6 M-cycles, interrupt dispatch is 5, so `[MicroOp;
8]` has headroom.

```
queue empty -> run one Fetch M-cycle (PC++), decode, push the remaining ops
```

### Cycle accounting

The cycle counts in `instructions.yml` **include the instruction's own opcode
fetch** — `nop` is 4 T-cycles because its single M-cycle *is* that fetch. Since
the sequencer performs the fetch itself when the queue empties, a decoded
instruction pushes `cycles / 4 - 1` ops, not `cycles / 4`.

Getting this wrong makes every instruction run 4 T-cycles long. The timing tests
catch it, but it presents as a sequencer bug rather than an off-by-one in the
convention.

Useful invariant for the generator: `1 + pushed_ops == cycles / 4`. It holds for
the awkward opcodes too — `push bc` 16 = fetch/internal/write/write, `pop bc`
12 = fetch/read/read, `ld (a16),sp` 20 = fetch/fetch/fetch/write/write,
`inc (hl)` 12 = fetch/read/write, `add sp,r8` 16 = fetch/fetch/internal/internal.

### PC

PC advances on fetches only — the opcode fetch and operand fetches. Internal
cycles and memory accesses leave it alone. `inc bc` is 8 cycles = fetch plus one
internal cycle (the 16-bit increment unit costs a cycle with no bus access), and
PC moves once.

### Conditionals

A conditional step appends a `&'static [MicroOp]` — up to three ops, and not all
of them are internal delays. The unconditional case is an empty slice, so there
is no special case to write.

| opcode        | not taken | taken   | appended on taken            |
| ------------- | --------- | ------- | ---------------------------- |
| `jr cc,r8`    | 8 (2 M)   | 12 (3 M)| `Internal`                   |
| `jp cc,a16`   | 12 (3 M)  | 16 (4 M)| `Internal`                   |
| `ret cc`      | 8 (2 M)   | 20 (5 M)| `Read`, `Read`, `Internal`   |
| `call cc,a16` | 12 (3 M)  | 24 (6 M)| `Internal`, `Write`, `Write` |

Note `ret cc` costs an extra internal cycle even when not taken.

Interrupt dispatch becomes a 5-op sequence pushed when the queue is empty, so it
stops being a special case. The EI delay becomes a pending flag applied the next
time the queue empties, which is exactly "after the following instruction".

## Shape of a micro-op

```rust
struct MicroOp {
    bus: BusAction,
    effect: Option<fn(&mut Cpu)>,
}

enum BusAction {
    Fetch,                              // PC++, into the latch
    Read  { addr: AddrMode, into: Half },
    Write { addr: AddrMode, from: Half },
    Internal,
}
```

A step performs its bus action, then runs its effect. `Half` (low/high of the
latch or of a 16-bit register) exists because `ld (a16),sp` writes two bytes on
two separate cycles. That payload is the irreducible part; everything else stays
data.

## The `Src`/`Dst` change this depends on

This is the part that looks fiddly and is not, once the cause is named. `Src`
and `Dst` currently do two unrelated jobs in one call:

```rust
impl Src<u8> for Mem<Reg16> {
    fn read(self, cpu: &mut Cpu) -> u8 {
        let addr = reg.read(cpu);   // where the data lives
        cpu.mmu.read(addr)          // when the bus cycle happens
    }
}
```

Under a sequencer the second half is the schedule's decision, not theirs. Do not
try to make `Src`/`Dst` generic enough to place cycles. Take the bus out of them:

```rust
trait Src<T> { fn read(self, cpu: &Cpu) -> T; }
trait Dst<T> { fn write(self, cpu: &mut Cpu, val: T); }
```

`&Cpu` on the read side is a forcing function: with no `&mut`, `Src::read`
cannot reach the MMU, and the compiler flags every place that needs a bus cycle
— which is precisely the list the schedule must own.

The operand types then split into two families that `Src` currently conflates:

- **Addressing modes** (`Mem<Reg16>`, `Mem<Imm16>`, `DMem<Reg8>`) resolve to a
  `u16` address and nothing more. The access is a `BusAction`.
- **Data sources** (`Reg8`, `Reg16`, latches) read directly, no bus, no cycle.

`Imm8`/`Imm16` move from the first family to the second: the fetch steps fill the
latch and the operand becomes a latch read, which drops their `&mut`.

The bodies in `backend/src/cpu/operations.rs` largely survive — they keep taking
`Src`/`Dst`, without a bus underneath.

## Generator work

`Instruction` in `types.rs` already carries `operands: Vec<String>`, `size` and
`time`, which is enough to classify most opcodes into a schedule shape:

- `reg,reg` — 1 M (fetch only)
- `reg,d8` — fetch, fetch
- `reg,(hl)` — fetch, read
- `(hl),reg` — fetch, write
- `(hl)` read-modify-write — fetch, read, write
- `a,(a16)` — fetch, fetch, fetch, read

with an override table for the ones that do not fit: `push`/`pop`,
`add sp,r8`, `ld hl,sp+r8`, `ld (a16),sp`, and the conditionals.

Rather than deriving the awkward cases by hand, gekkio's *Game Boy: Complete
Technical Reference* lists the exact per-M-cycle decomposition for every opcode.
Generating from that turns this into careful transcription rather than research.
