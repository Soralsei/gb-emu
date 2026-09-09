use crate::emitter::{MnemonicEmitter, RustEmitter};
use crate::types::*;
use std::{error::Error, fs::File, io::Write};
use tera::{Context, Tera};

/// Resolve an instruction's operand strings.
///
/// Most operands are self-describing, but the numeric ones are not: "0" is a
/// bit index for `bit`/`res`/`set`, a reset vector for `rst`, and a dummy for
/// `stop`. Those need the operator, so they can't come from `Operand::try_from`.
fn parse_operands(instruction: &Instruction) -> Result<Vec<Operand>, String> {
    let operator = instruction.operator.to_lowercase();
    instruction
        .operands
        .iter()
        .enumerate()
        .map(|(index, raw)| match (operator.as_str(), index) {
            ("bit" | "res" | "set", 0) => Operand::from_bit(raw),
            ("rst", 0) => {
                Operand::from_vector(raw).map_err(|e| format!("invalid rst vector '{raw}': {e}"))
            }
            ("stop", 0) => Ok(Operand::Dummy(0)),
            _ => Operand::try_from(raw.as_str()).map_err(|_| format!("unknown operand '{raw}'")),
        })
        .collect()
}

fn emit_mnemonic(instruction: &Instruction, operands: &[Operand]) -> String {
    let mut mnemonic = instruction.operator.to_lowercase();
    if !operands.is_empty() {
        let args: Vec<String> = operands.iter().map(|o| o.to_mnemonic()).collect();
        mnemonic.push(' ');
        mnemonic.push_str(&args.join(", "));
    }
    mnemonic
}

/// Build the `execute` closure for an instruction.
///
/// The yaml's operand list is not the operation's argument list: some operators
/// take an implicit register the yaml leaves out, some take a register the yaml
/// spells out but the signature hardcodes, and `bits` picks between same-named
/// 8- and 16-bit operations.
fn emit_call(instruction: &Instruction, operands: &[Operand]) -> Result<String, String> {
    // These three read their immediate internally, so their yaml operands have
    // nowhere to go in the call.
    let body = match instruction.code {
        0x10 => "{ cpu.stop(); Timing::Normal }".to_string(),
        0xE8 => "add_sp(cpu)".to_string(),
        0xF8 => "ldhl(cpu)".to_string(),
        _ => {
            let operator = instruction.operator.to_lowercase();
            let a: Vec<String> = operands.iter().map(|o| o.to_rust()).collect();
            match (operator.as_str(), instruction.bits, a.len()) {
                ("nop", _, _) => "nop()".to_string(),
                ("ret", _, 0) => "ret(cpu, Condition::Unconditional)".to_string(),
                (
                    "rlca" | "rla" | "rrca" | "rra" | "daa" | "cpl" | "scf" | "ccf" | "halt"
                    | "reti" | "ei" | "di",
                    _,
                    0,
                ) => format!("{operator}(cpu)"),

                // `bits` is the only thing separating these from their 8-bit
                // namesakes, so they have to match before the generic arms.
                ("inc" | "dec", 16, 1) => format!("{operator}16(cpu, {})", a[0]),
                ("add", 16, 2) => format!("add16(cpu, {}, {})", a[0], a[1]),

                ("ret", _, 1) => format!("ret(cpu, {})", a[0]),
                (
                    "inc" | "dec" | "rlc" | "rl" | "rr" | "rrc" | "sla" | "sra" | "srl" | "swap"
                    | "push" | "pop",
                    _,
                    1,
                ) => format!("{operator}(cpu, {})", a[0]),

                // A is implicit in the yaml but explicit in the signature --
                // except for `cp`, which takes no destination at all.
                ("sub" | "and" | "xor" | "or", _, 1) => {
                    format!("{operator}(cpu, Reg8::A, {})", a[0])
                }
                ("cp", _, 1) => format!("cp(cpu, {})", a[0]),

                ("rst", _, 1) => format!("rst(cpu, {})", a[0]),
                ("jr" | "jp" | "call", _, 1) => {
                    format!("{operator}(cpu, Condition::Unconditional, {})", a[0])
                }

                // The register to step is HL for every ldi/ldd on the DMG.
                ("ldi" | "ldd", _, 2) => {
                    format!("{operator}(cpu, {}, {}, Reg16::HL)", a[0], a[1])
                }
                ("ld" | "add" | "adc" | "sbc" | "bit" | "res" | "set", _, 2) => {
                    format!("{operator}(cpu, {}, {})", a[0], a[1])
                }
                ("jr" | "jp" | "call", _, 2) => format!("{operator}(cpu, {}, {})", a[0], a[1]),

                _ => {
                    return Err(format!(
                        "no call shape for 0x{:04X} '{}' {:?} (bits {})",
                        instruction.code, operator, instruction.operands, instruction.bits
                    ))
                }
            }
        }
    };

    let binding = if body == "nop()" { "_" } else { "cpu" };
    Ok(format!("|{binding}: &mut Cpu| {body}"))
}

impl TryFrom<&Instruction> for InstructionTemplate {
    type Error = String;

    fn try_from(instruction: &Instruction) -> Result<Self, Self::Error> {
        let operands = parse_operands(instruction)?;
        let cycles = match &instruction.time {
            Time::One(c) => format!("Cycles::Unconditional({})", c),
            Time::Two(conditional_time) => format!(
                "Cycles::Conditional(ConditionCycles {{ not_taken: {}, taken: {} }})",
                conditional_time.not_taken, conditional_time.taken
            ),
        };

        Ok(InstructionTemplate {
            code: format!("{:02X}", instruction.code & 0xFF),
            mnemonic: emit_mnemonic(instruction, &operands),
            cycles,
            call: emit_call(instruction, &operands)?,
        })
    }
}

fn instr_template_from_raw(
    instr: &[Instruction],
) -> Result<(Vec<InstructionTemplate>, Vec<InstructionTemplate>), String> {
    // 0xCB is consumed by the fetch loop before dispatch, so the yaml's `prefix`
    // entry has no operation behind it. It also sorts into the unprefixed half,
    // so drop it before partitioning rather than after.
    let (raw_unprefixed, raw_prefixed): (Vec<_>, Vec<_>) = instr
        .iter()
        .filter(|i| i.operator.to_lowercase() != "prefix")
        .partition(|i| i.code & 0xFF00 != 0xCB00);

    let unprefixed = raw_unprefixed
        .into_iter()
        .map(InstructionTemplate::try_from)
        .collect::<Result<Vec<_>, _>>()?;
    let prefixed = raw_prefixed
        .into_iter()
        .map(InstructionTemplate::try_from)
        .collect::<Result<Vec<_>, _>>()?;

    Ok((unprefixed, prefixed))
}

pub fn run(args: &GenerateArgs) -> Result<(), Box<dyn Error>> {
    let mut tera = Tera::default();
    tera.load_from_glob(&format!(
        "{}/**/*",
        args.template_path.to_str().unwrap_or("templates")
    ))?;

    let file = File::open(&args.oplist)?;
    let raw_instructions: Vec<Instruction> = serde_yaml_ng::from_reader(file)?;

    let (instructions, prefixed_instructions) = instr_template_from_raw(&raw_instructions)?;

    let mut context = Context::new();
    context.insert("instructions", &instructions);
    context.insert("prefixed_instructions", &prefixed_instructions);

    let rendered = tera.render("instruction.rs.tera", &context)?;
    File::create(&args.output)?.write_all(rendered.as_bytes())?;

    println!(
        "wrote {} ({} unprefixed, {} prefixed)",
        args.output.display(),
        instructions.len(),
        prefixed_instructions.len()
    );

    Ok(())
}
