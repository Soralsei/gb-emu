# Design note: what codegen emits for the CPU

Status: **not implemented** in the generator. The `backend` side has started:
`W`/`Z` are in the register file and `Src`/`Dst` are deleted, so
`backend/src/cpu/operations.rs` does not currently build. This replaces the
micro-op sequencer design that stood here; `backend/ASYNC_CLOCK.md` §"The CPU as
a task" is the other half and the two are meant to be read together.

## What this replaces

The note that was here wanted a generated schedule of micro-ops per opcode — a
`[MicroOp; 8]` queue, a `BusAction` enum, a sequencer draining it front to back.
That existed to make an instruction suspendable between M-cycles.

If the CPU is a task, suspension is free: an `await` *is* a resume point, and the
instruction body's straight-line control flow stays the state machine. The queue,
the cursor, the `BusAction` enum and the schedule-shape table all drop out.

What does not drop out is the research. The per-M-cycle bus schedule for every
opcode is the same knowledge either way, expressed as await placement rather than
as a table of ops. The sections below are that knowledge, carried over.

## What must be accurate, and what need not be

Nothing on the machine can observe the CPU's registers mid-instruction, so
internal computation does not need micro-architectural fidelity.

Bus accesses *are* observable, and their exact M-cycle is what the timing tests
measure. `gb-test-roms/mem_timing/source/` has three sub-tests that check
precisely this: `01-read_timing`, `02-write_timing`, and `03-modify_timing`.

`03` is the one that rules out running an instruction body atomically and merely
delaying around it. `inc (hl)` is 3 M-cycles — fetch, **read**, **write** — with
the read on cycle 2 and the write on cycle 3. An atomic body puts both on the
same cycle. Likewise `ld (a16),sp` writes on cycles 4 and 5, and `push bc` on
cycles 3 and 4. A one-cycle shift is visible: DIV advances between a
read-modify-write pair, and a VRAM write landing a cycle later can cross into
mode 3 and be blocked.

So: **what the generator emits is a bus access schedule**, not a
micro-architecture. ALU work, flag computation and register writes happen in one
lump attached to whichever step carries the relevant bus access.

## The `W`/`Z` change this depends on

`Src` and `Dst` used to do two unrelated jobs in one call:

```rust
impl Src<u8> for Mem<Reg16> {
    fn read(self, cpu: &mut Cpu) -> u8 {
        let addr = reg.read(cpu);   // where the data lives
        cpu.mmu.read(addr)          // when the bus cycle happens
    }
}
```

The second half is the schedule's decision, not theirs. Rather than make
`Src`/`Dst` generic enough to place cycles, the bus comes out of them entirely:
`Imm8`, `Imm16`, `Mem<T>` and `DMem<T>` are deleted and replaced by a `W`/`Z`
latch pair in the register file, with `WZ` as the 16-bit view. This is what the
hardware has, and it is why the decomposition below is transcription rather than
invention.

With the bus gone, `Src`/`Dst` abstract over `Reg8` and `Reg16` and nothing else
— which `Registers::read_u8`/`write_u8`/`read_u16`/`write_u16` already are, so
both traits go.

### The operations go one level further: flags only

Having taken the bus out, take the register file out too. Every synchronous
operation reduces to values in, value out, flags touched:

| operation | signature |
| --- | --- |
| `inc` `dec` `rlc` `rl` `rr` `rrc` `sla` `sra` `srl` `swap` `daa` `cpl` `rlca` `rla` `rrca` `rra` | `(&mut Flags, u8) -> u8` |
| `add` `adc` `sub` `sbc` `and` `xor` `or` | `(&mut Flags, u8, u8) -> u8` |
| `cp` `bit` | `(&mut Flags, u8, u8)` |
| `res` `set` | `(u8, u8) -> u8` — no flags |
| `scf` `ccf` | `(&mut Flags)` |
| `add16` | `(&mut Flags, u16, u16) -> u16` |
| `offset_sp` | `(&mut Flags, u16, u8) -> u16` — serves `add sp,r8` and `ld hl,sp+r8` both |

The operations that need more than flags are exactly the ones already becoming
`async fn` on `&Cpu`, because they move PC, SP or IME: `jr`, `jp`, `call`, `ret`,
`reti`, `rst`, `push`, `pop`, `ei`, `di`, `halt`, `stop`. Nothing is left over.

Nine drop out of `operations.rs` entirely, because with the plumbing emitted into
the arm there is nothing left in them:

| gone | was |
| --- | --- |
| `ld` `ld16` | a move; the arm already is one |
| `ldi` `ldd` | a move plus a 16-bit step on `HL` — two helper calls |
| `inc16` `dec16` | no flags, so `wrapping_add(1)` with a name on top |
| `add_sp` `ldhl` | `offset_sp` plus a store to `SP` / `HL`, which is the helper's job |
| `nop` | an empty arm |

`offset_sp` is the one that survives that group, and it is the only part that was
ever an operation: the flag computation, which is famously the part people get
wrong — `H` and `C` come from bit 4 and bit 8 of the *byte* addition, not the word.

The point is not tidiness. `daa`, `adc`, `sbc` and `add16` are where flag bugs
live, and on a bare `Flags` they are unit-testable without standing up a `Cpu`, a
bus or a clock. Today testing `daa` means constructing a machine.

So: the ALU does not know where its operands came from, any more than it knows
when the bus cycle happens. Both are the same deletion, one level apart.

One bound carried information worth not losing: `L: Dst<u8> + Src<u8> + Copy`
meant read-modify-write. It moves to the access-mode table below, which is the
only thing that consumes it.

## Operand lowering

One `Operand` no longer maps to one expression. It maps to **(pre-steps, operand,
post-steps)**, and the bus steps name a register directly:

```rust
async fn fetch(&self, dst: Reg8);            // PC++, read -> dst
async fn load(&self, dst: Reg8, addr: u16);
async fn store(&self, addr: u16, src: Reg8);
```

### `W`/`Z` are fetch latches first

An immediate has nowhere else to go: the bytes arrive one per cycle and the
operation needs them assembled. That is the whole job, and `d16`/`a16` assembling
into `WZ` is why the 16-bit view exists.

Memory accesses are a different question, and the tempting rule — *every memory
operand collapses to `Z`* — is wrong. It emits a pointless register move whenever
the other end is already a real register:

```rust
ld (hl),b     load B into Z, then store Z        // the move buys nothing
ld a,(hl)     load into Z, then move Z into A    // same
```

Since `load`/`store` name a register, both are a bus step and nothing else. `Z`
takes the value only when there is no register to name — a read-modify-write, or
a memory byte on its way to the ALU:

| yaml operand | pre | operand | post (when written) |
| --- | --- | --- | --- |
| `a`…`l`, `bc`…`sp` | — | `Reg8::A` / `Reg16::BC` | — |
| `d8` / `a8` / `r8` | `fetch(Z)` | `Reg8::Z` | — |
| `d16` / `a16` | `fetch(Z)`, `fetch(W)` | `Reg16::WZ` | — |
| `(bc)` `(de)` `(hl)`, plain `ld` to/from a register | — | — | the bus step *is* the instruction |
| `(bc)` `(de)` `(hl)`, ALU source | `load(Z, addr)` | `Reg8::Z` | — |
| `(bc)` `(de)` `(hl)`, read-modify-write | `load(Z, addr)` | `Reg8::Z` | `store(addr, Z)` |
| `(a16)` | `fetch(Z)`, `fetch(W)` | address is `WZ` | — |
| `(0xff00+c)` | — | address is `0xFF00 \| C` | — |
| `(0xff00+a8)` | `fetch(Z)` | address is `0xFF00 \| Z` | — |

Emitted, the common cases are one line:

```rust
0x7E => cpu.load(Reg8::A, cpu.addr(Reg16::HL)).await,      // ld a,(hl)
0x70 => cpu.store(cpu.addr(Reg16::HL), Reg8::B).await,     // ld (hl),b
0x36 => { cpu.fetch(Reg8::Z).await;                        // ld (hl),d8
          cpu.store(cpu.addr(Reg16::HL), Reg8::Z).await; }
0xFA => { cpu.fetch(Reg8::Z).await;                        // ld a,(a16)
          cpu.fetch(Reg8::W).await;
          cpu.load(Reg8::A, cpu.addr(Reg16::WZ)).await; }
```

### `Z` as scratch never collides with `Z` as address

Worth stating because the opposite looks true. The only operands that *write* `Z`
from the bus are read-modify-write and ALU sources, and those address exclusively
through `(hl)`, `(bc)`, `(de)` — never through `(a16)`. So no instruction holds an
address in `WZ` while something overwrites `Z`.

Resolving the address into a local before the step is still the right shape, and
`load`/`store` taking `addr: u16` keeps it structural — but it is a habit, not a
hazard being defended against.

### The one 16-bit memory operand

`0x08 ld (a16),sp` writes a `u16`, and `Z` already holds half the address. It
takes the two halves of `SP` directly, on consecutive cycles to `WZ` and `WZ + 1`,
low byte first:

```rust
0x08 => {
    cpu.fetch(Reg8::Z).await;
    cpu.fetch(Reg8::W).await;
    cpu.store16(cpu.addr(Reg16::WZ), Reg16::SP).await;   // two awaits inside
}
```

A `store16` helper rather than a `Half` enum: `SP` is one `u16` field, so its
halves have no `Reg8` name to pass, and inventing one would exist for this single
opcode.

## Access modes

Whether a memory operand emits `pre`, `post`, both or neither is a property of the
**operator**, not the operand. The generator needs this table; nothing in
`instructions.yml` carries it.

| operator | operand 0 | operand 1 |
| --- | --- | --- |
| `ld` `ldi` `ldd` | Write | Read |
| `inc` `dec` `rl` `rlc` `rr` `rrc` `sla` `sra` `srl` `swap` `res` `set` | ReadWrite | — |
| `bit` | — | Read |
| `add` `adc` `sub` `sbc` `and` `xor` `or` `cp` | (`A`, implicit) | Read |
| `jr` `jp` `call` | Read | Read |

Combined with whether the other operand is a register, that decides what comes out:

| mode | emits |
| --- | --- |
| Write, register source | `store(addr, src)` — no operation call |
| Read, plain `ld` into a register | `load(dst, addr)` — no operation call |
| Read, feeding the ALU | `load(Z, addr)`, then the operation on `Z` |
| ReadWrite | `load(Z, addr)`, operation on `Z`, `store(addr, Z)` |

`bit b,(hl)` is Read-only at 12 cycles and `res b,(hl)` is ReadWrite at 16. Today
that distinction falls out of trait bounds by accident; here it is a line in a
table.

## Two outputs

The generated file serves two consumers that want opposite things.

### The execution half

A single `async fn` per prefix, with a `match` over the opcode. **Do not emit one
`async fn` per opcode** — each would be a distinct future type, so dispatch would
need `Box<dyn Future>`, an allocation per instruction at ~1M instructions/second.
One `async fn` is one future type, one state machine, zero allocation; its size is
the max over all arms.

```rust
pub async fn execute_unprefixed(cpu: &Cpu, opcode: u8) {
    match opcode {
        0x01 => { cpu.fetch(Reg8::Z).await;                      // ld bc,d16
                  cpu.fetch(Reg8::W).await;
                  cpu.mov16(Reg16::BC, Reg16::WZ); }
        0x04 => cpu.alu(Reg8::B, inc),                           // inc b
        0x86 => { cpu.load(Reg8::Z, cpu.addr(Reg16::HL)).await;  // add a,(hl)
                  cpu.alu2(Reg8::A, Reg8::Z, add); }
        // ...
        _ => illegal(cpu),
    }
}
```

With the operations on flags alone, the arm is the only thing that touches the
register file. A small family of helpers on `Cpu` reads, applies and writes back
under one `registers_mut()` borrow:

| helper | operations |
| --- | --- |
| `alu(Reg8, f(&mut Flags, u8) -> u8)` | `inc` `dec` `rlc` `rl` `rr` `rrc` `sla` `sra` `srl` `swap` `daa` `cpl` `res` `set` |
| `alu2(Reg8 dst, Reg8 src, f(&mut Flags, u8, u8) -> u8)` | `add` `adc` `sub` `sbc` `and` `xor` `or` |
| `test(Reg8, f(&mut Flags, u8))` | `bit` |
| `test2(Reg8, Reg8, f(&mut Flags, u8, u8))` | `cp` |
| `flags(f(&mut Flags))` | `scf` `ccf` |
| `alu16(Reg16, f(u16) -> u16)` | `inc bc`…`dec sp`, and the `HL` step of `ldi`/`ldd` |
| `alu16_2(Reg16 dst, Reg16 a, Reg16 b, f(&mut Flags, u16, u16) -> u16)` | `add hl,rr` |
| `alu16_8(Reg16 dst, Reg16 a, Reg8 b, f(&mut Flags, u16, u8) -> u16)` | `add sp,r8`, `ld hl,sp+r8` |
| `mov(Reg8, Reg8)` / `mov16(Reg16, Reg16)` | `ld` `ld16` |

`alu16_8` takes an explicit destination because `0xF8` is the one arm whose
destination is not one of its sources — it reads `SP` and `Z` and writes `HL`:

```rust
0xE8 => { cpu.fetch(Reg8::Z).await;                                  // add sp,r8
          cpu.alu16_8(Reg16::SP, Reg16::SP, Reg8::Z, offset_sp);
          cpu.idle().await; cpu.idle().await; }
0xF8 => { cpu.fetch(Reg8::Z).await;                                  // ld hl,sp+r8
          cpu.alu16_8(Reg16::HL, Reg16::SP, Reg8::Z, offset_sp);
          cpu.idle().await; }
```

The temptation is an `offset_sp_into(dst)` helper that reads `SP` and `Z` itself.
Don't: that puts knowledge of two specific opcodes into the plumbing, which is the
coupling this whole change removes. Helpers move bytes between the register file
and a function; which bytes is the arm's business. `alu16_2` takes an explicit
destination for the same reason, even though `add hl,rr` always passes `HL` twice.

Closures are zero-cost, so the ones carrying an operand pass one:
`cpu.alu(Reg8::Z, |_, v| res(0, v))`. The read-modify-write shape then reads
identically whether the operand is a register or memory — only the `load`/`store`
pair around it appears.

**These exist for borrow scoping, not brevity.** `registers_mut()` must be held
for one call and never across an `await`, and several arms interleave the two.
`inc bc` is a fetch and an *internal cycle*:

```rust
0x03 => { let mut r = cpu.registers_mut();          // borrow still live...
          let v = r.read_u16(Reg16::BC).wrapping_add(1);
          r.write_u16(Reg16::BC, v);
          cpu.idle().await; }                       // ...across this.

0x03 => { cpu.alu16(Reg16::BC, |v| v.wrapping_add(1));
          cpu.idle().await; }                       // borrow ends inside the call
```

The first form panics the moment anything else borrows — the debugger's
disassembly panel repainting while the task is parked, per `ASYNC_CLOCK.md`
§Shape. Emitting through the helpers makes the rule structural rather than
something the emitter has to remember at every site.

`Timing` does not survive. It existed only to pick a branch of `Cycles` for
`spend()`; conditional cycles now live inside the four operators that have them
(`jr`, `jp`, `call`, `ret`), as an `await` on the taken path. Operations return a
value or nothing — never a timing.

### The disassembly half

`DEBUGGER.md` §Disassembly needs the operand structure at runtime, in the yaml's
own vocabulary — `(hl)` and `d8` render as `(hl)` and `$93` regardless of how
execution lowers them to `W`/`Z`. So `Mem`, `HighMem`, `Imm8`, `Rel8` survive as
*data*, and the runtime enum mirrors `codegen::types::Operand` one to one.

With `execute` no longer a field, `Instruction` is fully const, so this stops
being a `match` returning by value and becomes indexable — which is what a
disassembly panel scrolling backwards over the trace bitmap wants:

```rust
pub struct Instruction {
    pub operator: &'static str,          // "ld" -- replaces the frozen `mnemonic`
    pub operands: &'static [Operand],
    pub size: u8,
    pub cycles: Cycles,
}

pub static UNPREFIXED: [Instruction; 256] = [ /* ... */ ];
```

Not feature-gated. `DEBUGGER.md` §"Attaching and detaching" turns on normal mode
costing nothing by construction, which is what makes a separate debug build
unnecessary; a `#[cfg(feature = "debug")]` table would give that up for a few KB.

**The array must be dense.** `instructions.yml` has ten holes — `0xD3`, `0xDB`,
`0xDD`, `0xE3`, `0xE4`, `0xEB`–`0xED`, `0xF4`, `0xFC`, `0xFD` — which the current
`_ => ILLEGAL` match arm covers implicitly. `instr_template_from_raw` must sort by
code, fill the gaps, and assert a length of 256. The execution-side match keeps
its `_` arm.

### Three emitter traits

| trait | fate |
| --- | --- |
| `RustEmitter for Operand` | becomes the lowering above — returns `(pre, expr, post)`, not a type name |
| `ConstEmitter for Operand` | new. `Operand::Mem(&Operand::R16(Reg16::HL))`, valid in const position by rvalue static promotion |
| `MnemonicEmitter` | deleted. The join moves to runtime `render`, so operands can resolve against real bytes |

`RustEmitter`/`ConstEmitter` for `Reg8`/`Reg16`/`Condition` carry over unchanged,
except that `Reg8` gains `W`/`Z` and `Reg16` gains `WZ` on the execution side only
— those are never yaml operands, so the disassembly table never mentions them.

One collapse to undo: `emitter.rs` currently has
`Operand::Imm8 | Operand::Rel8 => "Imm8"`. `ConstEmitter` must keep them apart,
because `Rel8` renders a resolved target (`jr nz, $032D`) and `Imm8` does not.

## Cycle accounting, and three asserts

The cycle counts in `instructions.yml` **include the instruction's own opcode
fetch** — `nop` is 4 T-cycles because its single M-cycle *is* that fetch. The task
loop performs that fetch before dispatch, so an arm emits `cycles / 4 - 1` steps,
not `cycles / 4`.

Getting this wrong makes every instruction run 4 T-cycles long. The timing tests
catch it, but it presents as a dispatch bug rather than an off-by-one in a
convention.

The invariant holds for the awkward opcodes too — `push bc` 16 =
fetch/internal/write/write, `pop bc` 12 = fetch/read/read, `ld (a16),sp` 20 =
fetch/fetch/fetch/write/write, `inc (hl)` 12 = fetch/read/write, `add sp,r8` 16 =
fetch/fetch/internal/internal.

Since the idles are derived from `cycles` rather than checked against it, that
invariant is a definition now and asserts nothing. The independent check is
against `size`, which is already deserialised into `codegen::types::Instruction`
and thrown away today; emitting it is one template field.

```
emitted fetch steps == size - base        // base: 1 unprefixed, 2 prefixed
derived idles       >= 0
```

`base` is where the CB convention gets pinned. Entries carry `size: 2` and a cycle
count that both include the prefix byte, but the task loop consumes `0xCB` before
dispatch, so a prefixed arm owns neither. Both checks pass for all 461 opcodes
with `base` defined this way; get it wrong and every CB instruction is off by one
fetch and 4 cycles at once.

That pair is worth more than it looks. `size` comes from the yaml and the fetch
count comes from the lowering table, so they are independent derivations of the
same fact — a mistake in the operand classification shows up here rather than
under `instr_timing` an hour later.

The taken branch of a conditional is not checkable from the emitted text, because
its extra awaits are inside the operation. Those four stay hand-verified against:

| opcode | not taken | taken | extra on taken |
| --- | --- | --- | --- |
| `jr cc,r8` | 8 (2 M) | 12 (3 M) | one internal |
| `jp cc,a16` | 12 (3 M) | 16 (4 M) | one internal |
| `ret cc` | 8 (2 M) | 20 (5 M) | two reads, one internal |
| `call cc,a16` | 12 (3 M) | 24 (6 M) | one internal, two writes |

`ret cc` costs an extra internal cycle **even when not taken**, which is why its
unconditional `idle` precedes the condition rather than sitting inside it.

## PC

PC advances on fetches only — the opcode fetch and operand fetches. Internal
cycles and memory accesses leave it alone. `inc bc` is 8 cycles = fetch plus one
internal cycle (the 16-bit increment unit costs a cycle with no bus access), and
PC moves once.

## Internal cycles are derived, not tabulated

For every arm the generator emits, the internal cycles are **trailing**. So they
need no table and no per-opcode knowledge:

```
idles = cycles / 4 - base - (fetches + loads + stores)
```

Checked against `instructions.yml` for all 461 generator-emitted opcodes: never
negative, no exceptions. Fifteen opcodes have any — the twelve 16-bit
`inc`/`dec`/`add hl,rr`, plus `add sp,r8` (two), `ld hl,sp+r8`, and `ld sp,hl`.

That last one is the argument for deriving rather than listing. `0xF9 ld sp,hl`
is 8 cycles: a fetch and one internal cycle, with no bus access and nothing in its
operands to suggest it. On a hand-maintained list it is exactly the entry that
gets missed, and it presents as one instruction running 4 cycles short.

**The condition this rests on:** internals are trailing *for generated arms*. They
are not in general — `push bc` is fetch/internal/write/write, with the internal in
the middle. `push` is a hand-written operation that owns its own awaits, so the
generator never places its cycles. If anything moves from the hand-written set
into the generated path, check that its internals are still trailing before
trusting the derivation.

## The one opcode that needs a hand-written arm

The generator has an override table today, and it is an artifact of the thing
being deleted. Its comment says why: *"These three read their immediate
internally, so their yaml operands have nowhere to go in the call."* `Imm8::read`
performed a hidden bus access, so `add_sp`, `ldhl` and `stop` consumed their own
operand and the emitter had nothing to hand them. Once `r8` lowers to an ordinary
`cpu.fetch(Reg8::Z)` in the pre-steps and the operation takes `Reg8::Z` like any
other, the table has one entry left.

`0x10` `stop`. It reads its own second byte *conditionally* — only when no
interrupt is pending — and what it does next depends on JOYP, IE/IF and whether a
speed switch is armed. That conditionality is real, not an artifact.
`backend/ASYNC_CLOCK.md` has the three outcomes.

Everything else is regular:

| looks special | actually |
| --- | --- |
| `0x08 ld (a16),sp` | `(a16)` written at `bits == 16` — the 16-bit row of the lowering table |
| `0xE8 add sp,r8` | `r8` is a plain fetch; the two internals derive |
| `0xF8 ld hl,sp+r8` | same, one internal |

`push`, `pop`, `call`, `ret`, `reti` and `rst` are hand-written *operations*,
not override-table entries: they move bytes through the stack, so they own their
awaits and the generator emits an ordinary call.

## Where the remaining research lives

gekkio's *Game Boy: Complete Technical Reference* lists the exact per-M-cycle
decomposition for every opcode. The lowering table above covers the regular cases;
for anything that looks irregular, transcribe from there rather than deriving it.
