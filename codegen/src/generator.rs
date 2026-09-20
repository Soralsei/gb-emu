use crate::emitter::{ConstEmitter, RustEmitter};
use crate::types::*;
use serde::Serialize;
use std::error::Error;
use std::fs;
use tera::{Context, Tera};

#[derive(Default)]
struct Body {
    lines: Vec<String>,
    fetches: usize,
    loads: usize,
    stores: usize,
}

/// Control flow is irregular and hand-written verbatim in the template; the
/// generator skips these opcodes when emitting execution arms (the disasm table
/// still lists them, since operator/operands/cycles are regular data).
fn is_control(op: &str) -> bool {
    matches!(
        op,
        "jr" | "jp" | "call" | "ret" | "reti" | "rst" | "push" | "pop" | "stop"
    )
}

impl Body {
    fn fetch(&mut self, dst: Reg8) {
        self.fetches += 1;
        self.lines
            .push(format!("cpu.fetch({}).await;", dst.emit_rust()));
    }
    fn load(&mut self, dst: Reg8, addr: &str) {
        self.loads += 1;
        self.lines
            .push(format!("cpu.load({}, {addr}).await;", dst.emit_rust()));
    }
    fn store(&mut self, addr: &str, src: Reg8) {
        self.stores += 1;
        self.lines
            .push(format!("cpu.store({addr}, {}).await;", src.emit_rust()));
    }
    fn store16(&mut self, addr: &str, src: Reg16) {
        self.stores += 2;
        self.lines
            .push(format!("cpu.store16({addr}, {}).await;", src.emit_rust()));
    }
    fn line(&mut self, s: impl Into<String>) {
        self.lines.push(s.into());
    }
    fn bus(&self) -> usize {
        self.fetches + self.loads + self.stores
    }
}

fn is_mem(o: &Operand) -> bool {
    matches!(
        o,
        Operand::Mem(_) | Operand::MemImm16 | Operand::HighC | Operand::HighImm8
    )
}

/// Push any pre-fetches a memory operand needs, return its address expression.
fn addr_of(o: &Operand, b: &mut Body) -> String {
    match o {
        Operand::Mem(r) => format!("cpu.addr({})", r.emit_rust()),
        Operand::MemImm16 => {
            b.fetch(Reg8::Z);
            b.fetch(Reg8::W);
            "cpu.addr(Reg16::WZ)".into()
        }
        Operand::HighC => "cpu.high(Reg8::C)".into(),
        Operand::HighImm8 => {
            b.fetch(Reg8::Z);
            "cpu.high(Reg8::Z)".into()
        }
        other => unreachable!("addr_of on {other:?}"),
    }
}

fn r8(o: Option<&Operand>) -> String {
    match o {
        Some(Operand::Reg8(r)) => r.emit_rust(),
        other => panic!("expected r8, got {other:?}"),
    }
}
fn r16(o: Option<&Operand>) -> String {
    match o {
        Some(Operand::Reg16(r)) => r.emit_rust(),
        other => panic!("expected r16, got {other:?}"),
    }
}

fn lower(i: &Instruction) -> Body {
    let mut b = Body::default();
    let op = i.operator.as_str();

    match op {
        "nop" => {}

        "ld" | "ldi" | "ldd" => {
            move_(i, &mut b);
            match op {
                "ldi" => b.line("cpu.alu16(Reg16::HL, |_, v| v.wrapping_add(1));"),
                "ldd" => b.line("cpu.alu16(Reg16::HL, |_, v| v.wrapping_sub(1));"),
                _ => {}
            }
        }

        "add" | "adc" | "sub" | "sbc" | "and" | "or" | "xor" if i.bits == 8 => {
            let src = alu_source(i, &mut b);
            b.line(format!("cpu.alu2(Reg8::A, {src}, {op});"));
        }
        "cp" => {
            let src = alu_source(i, &mut b);
            b.line(format!("cpu.test2(Reg8::A, {src}, cp);"));
        }

        "add" => match i.operands.first() {
            Some(Operand::Reg16(Reg16::SP)) => {
                b.fetch(Reg8::Z); // add sp,r8
                b.line("cpu.alu16_8(Reg16::SP, Reg16::SP, Reg8::Z, offset_sp);");
            }
            Some(Operand::Reg16(Reg16::HL)) => {
                b.line(format!(
                    "cpu.alu16_2(Reg16::HL, Reg16::HL, {}, add16);",
                    r16(i.last())
                ));
            }
            other => panic!("bad 16-bit add {other:?}"),
        },
        "ldhl" => {
            b.fetch(Reg8::Z); // ld hl,sp+r8
            b.line("cpu.alu16_8(Reg16::HL, Reg16::SP, Reg8::Z, offset_sp);");
        }

        "inc" | "dec" if i.bits == 8 => rmw(i, &mut b, op.into()),
        "swap" | "srl" | "sra" | "sla" | "rrc" | "rr" | "rlc" | "rl" => rmw(i, &mut b, op.into()),
        "res" | "set" => {
            let n = i.index().expect("res/set need a bit index");
            rmw(i, &mut b, format!("|_, v| {op}({n}, v)"));
        }

        "inc" | "dec" => {
            let delta = if op == "inc" { "add" } else { "sub" };
            b.line(format!(
                "cpu.alu16({}, |_, v| v.wrapping_{delta}(1));",
                r16(i.operands.first())
            ));
        }

        "bit" => {
            let n = i.index().expect("bit needs an index");
            let target = i.last().unwrap();
            let reg = if is_mem(target) {
                let a = addr_of(target, &mut b);
                b.load(Reg8::Z, &a);
                "Reg8::Z".into()
            } else {
                r8(Some(target))
            };
            b.line(format!("cpu.test({reg}, |f, v| bit(f, {n}, v));"));
        }

        "rlca" => b.line("cpu.alu(Reg8::A, rlca);"),
        "rla" => b.line("cpu.alu(Reg8::A, rla);"),
        "rrca" => b.line("cpu.alu(Reg8::A, rrca);"),
        "rra" => b.line("cpu.alu(Reg8::A, rra);"),
        "daa" => b.line("cpu.alu(Reg8::A, da);"),
        "cpl" => b.line("cpu.alu(Reg8::A, cpl);"),
        "scf" => b.line("cpu.flags(scf);"),
        "ccf" => b.line("cpu.flags(ccf);"),
        "ei" => b.line("cpu.ei();"),
        "di" => b.line("cpu.di();"),
        "halt" => b.line("cpu.halt();"),

        c if is_control(c) => unreachable!("control op '{c}' is hand-written in the template"),

        other => panic!("no lowering for '{other}' (0x{:02X})", i.byte()),
    }

    derive_timing(i, &mut b);
    b
}

fn move_(i: &Instruction, b: &mut Body) {
    let (dst, src) = (&i.operands[0], &i.operands[1]);
    match (dst, src) {
        (Operand::Reg8(d), Operand::Reg8(s)) => {
            b.line(format!("cpu.mov({}, {});", d.emit_rust(), s.emit_rust()))
        }
        (Operand::Reg16(d), Operand::Reg16(s)) => {
            b.line(format!("cpu.mov16({}, {});", d.emit_rust(), s.emit_rust()))
        }
        (Operand::Reg8(d), Operand::Imm8) => b.fetch(*d), // ld r,d8 — fetch straight in
        (Operand::Reg16(d), Operand::Imm16) => {
            b.fetch(Reg8::Z);
            b.fetch(Reg8::W);
            b.line(format!("cpu.mov16({}, Reg16::WZ);", d.emit_rust()));
        }
        (Operand::Reg8(d), s) if is_mem(s) => {
            let a = addr_of(s, b);
            b.load(*d, &a);
        }
        (d, Operand::Reg8(s)) if is_mem(d) => {
            let a = addr_of(d, b);
            b.store(&a, *s);
        }
        (d, Operand::Reg16(Reg16::SP)) if is_mem(d) => {
            let a = addr_of(d, b); // ld (a16),sp
            b.store16(&a, Reg16::SP);
        }
        (d, Operand::Imm8) if is_mem(d) => {
            let a = addr_of(d, b); // ld (hl),d8
            b.fetch(Reg8::Z);
            b.store(&a, Reg8::Z);
        }
        other => panic!("no move lowering for {other:?}"),
    }
}

fn alu_source(i: &Instruction, b: &mut Body) -> String {
    let src = i.last().expect("alu needs a source");
    if is_mem(src) {
        let a = addr_of(src, b);
        b.load(Reg8::Z, &a);
        return "Reg8::Z".into();
    }
    match src {
        Operand::Reg8(_) => r8(Some(src)),
        Operand::Imm8 => {
            b.fetch(Reg8::Z);
            "Reg8::Z".into()
        }
        other => panic!("bad alu source {other:?}"),
    }
}

/// Read-modify-write on the last operand. `op` is the fn name or a closure.
fn rmw(i: &Instruction, b: &mut Body, op: String) {
    let target = i.last().unwrap();
    if is_mem(target) {
        let a = addr_of(target, b);
        b.load(Reg8::Z, &a);
        b.line(format!("cpu.alu(Reg8::Z, {op});"));
        b.store(&a, Reg8::Z);
    } else {
        b.line(format!("cpu.alu({}, {op});", r8(Some(target))));
    }
}

/// Trailing idles are derived; `size` cross-checks the fetch count. Both are
/// asserts at generation time so a classification mistake fails here, loudly.
fn derive_timing(i: &Instruction, b: &mut Body) {
    let base = if i.prefixed() { 2 } else { 1 };
    assert_eq!(
        b.fetches,
        i.size - base,
        "0x{:02X} {}: {} fetches, size wants {}",
        i.byte(),
        i.operator,
        b.fetches,
        i.size - base
    );
    let idles = i.time.mcycles() as isize - base as isize - b.bus() as isize;
    assert!(
        idles >= 0,
        "0x{:02X} {}: negative idles",
        i.byte(),
        i.operator
    );
    for _ in 0..idles {
        b.line("cpu.idle().await;");
    }
}

// ===========================================================================
// Context for Tera. Everything render-shaped is a plain field; the template
// owns match layout, arm braces, tables and the header.
// ===========================================================================

#[derive(Serialize)]
struct Arm {
    code: String,
    mnemonic: String,
    body: Vec<String>,
}

#[derive(Serialize)]
struct Row {
    illegal: bool,
    operator: String,
    operands: Vec<String>,
    size: usize,
    cycles: String,
}

fn arm(i: &Instruction) -> Arm {
    Arm {
        code: format!("{:02X}", i.byte()),
        mnemonic: mnemonic(i),
        body: lower(i).lines,
    }
}

fn row(i: Option<&Instruction>) -> Row {
    match i {
        None => Row {
            illegal: true,
            operator: String::new(),
            operands: vec![],
            size: 1,
            cycles: String::new(),
        },
        Some(i) => Row {
            illegal: false,
            operator: i.operator.clone(),
            operands: i.operands.iter().map(ConstEmitter::emit_const).collect(),
            size: i.size,
            cycles: match &i.time {
                Time::One(t) => format!("Cycles::Unconditional({t})"),
                Time::Two(c) => format!(
                    "Cycles::Conditional {{ taken: {}, not_taken: {} }}",
                    c.taken, c.not_taken
                ),
            },
        },
    }
}

fn mnemonic(i: &Instruction) -> String {
    let ops: Vec<String> = i.operands.iter().map(text).collect();
    if ops.is_empty() {
        i.operator.clone()
    } else {
        format!("{} {}", i.operator, ops.join(","))
    }
}

fn text(o: &Operand) -> String {
    match o {
        Operand::Reg8(r) => r.emit_rust().trim_start_matches("Reg8::").to_lowercase(),
        Operand::Reg16(r) => r.emit_rust().trim_start_matches("Reg16::").to_lowercase(),
        Operand::Imm8 => "d8".into(),
        Operand::Rel8 => "r8".into(),
        Operand::Imm16 => "d16".into(),
        Operand::Mem(r) => format!(
            "({})",
            r.emit_rust().trim_start_matches("Reg16::").to_lowercase()
        ),
        Operand::MemImm16 => "(a16)".into(),
        Operand::HighC => "(0xff00+c)".into(),
        Operand::HighImm8 => "(0xff00+a8)".into(),
        Operand::Cond(c) => c
            .emit_rust()
            .trim_start_matches("Condition::")
            .to_lowercase(),
        Operand::Num(n) => n.to_string(),
    }
}

fn slots(instrs: &[Instruction], prefixed: bool) -> Vec<Option<&Instruction>> {
    let mut out: Vec<Option<&Instruction>> = vec![None; 256];
    for i in instrs {
        if i.operator != "prefix" && i.prefixed() == prefixed {
            out[i.byte() as usize] = Some(i);
        }
    }
    out
}

pub fn run(args: &GenerateArgs) -> Result<(), Box<dyn Error>> {
    let file = fs::File::open(&args.oplist)?;
    let instrs: Vec<Instruction> = serde_yaml_ng::from_reader(file)?;

    let unpref = slots(&instrs, false);
    let pref = slots(&instrs, true);

    // Execution arms: regular opcodes only (control flow is literal in the
    // template). Disasm tables: everything.
    let arms = |slots: &[Option<&Instruction>]| {
        slots
            .iter()
            .flatten()
            .filter(|i| !is_control(&i.operator))
            .map(|i| arm(i))
            .collect::<Vec<_>>()
    };

    let mut ctx = Context::new();
    ctx.insert("unprefixed", &arms(&unpref));
    ctx.insert("prefixed", &arms(&pref));
    ctx.insert(
        "unprefixed_table",
        &unpref.iter().map(|s| row(*s)).collect::<Vec<_>>(),
    );
    ctx.insert(
        "prefixed_table",
        &pref.iter().map(|s| row(*s)).collect::<Vec<_>>(),
    );

    let mut tera = Tera::default();
    tera.add_raw_template("instr", include_str!("../templates/instruction.rs.tera"))?;
    let rendered = tera.render("instr", &ctx)?;
    fs::write(&args.output, rendered)?;

    println!(
        "wrote {} ({} unprefixed, {} prefixed arms)",
        args.output.display(),
        unpref.iter().flatten().count(),
        pref.iter().flatten().count(),
    );
    Ok(())
}
