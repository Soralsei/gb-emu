use crate::types::*;
use phf::phf_map;
use std::{error::Error, fs::File};
use tera::{Context, Tera};

static OPERAND_PATTERNS: phf::Map<&'static str, &'static str> = phf_map! {
    "af" => "Reg16::AF",
    "bc" => "Reg16::BC",
    "de" => "Reg16::DE",
    "hl" => "Reg16::HL",
    "sp" => "Reg16::SP",
    "pc" => "Reg16::PC",
    "a" => "Reg8::A",
    "b" => "Reg8::B",
    "c" => "Reg8::C",
    "d" => "Reg8::D",
    "e" => "Reg8::E",
    "h" => "Reg8::H",
    "l" => "Reg8::L",
    "r8" => "Imem8",
    "d8" => "Imem8",
    "d16" => "Imem16",
    "a16" => "Imem16",
    "nc"=> "Condition::NotCarry",
    "cf"=> "Condition::Carry",
    "nz"=> "Condition::NotZero",
    "z"=> "Condition::Zero",
};

fn get_function_call(instruction: &Instruction) -> String {
    let mut params: Vec<String> = vec!["cpu".to_string()];
    for operand in &instruction.operands {
        let op = operand.replace(&['(', ')'], "");
        let param: &str = OPERAND_PATTERNS.get(&op).unwrap_or(&op.as_str());
        let template =        if operand.contains('(') || operand.contains(')') {
            format!("Mem({})", param)
        } else {
            format!("{}", param)
        };
        params.push(template);
    }
    // TODO

    "".to_string()
}

impl From<&Instruction> for InstructionTemplate {
    fn from(instruction: &Instruction) -> Self {
        let operands = instruction.operands.join(", ");
        let mut mnemonic = instruction.operator.clone();
        if !operands.is_empty() {
            mnemonic.push(' ');
            mnemonic.push_str(&operands);
        }
        let cycles = match &instruction.time {
            Time::One(c) => format!("Cycles::Unconditional({})", c),
            Time::Two(conditional_time) => format!(
                "Cycles::Conditional(ConditionCycles {{
                    not_taken: {},
                    taken: {},
                }})",
                conditional_time.not_taken, conditional_time.taken
            ),
        };
        InstructionTemplate {
            code: format!("{:02X}", instruction.code & 0xFF),
            mnemonic,
            cycles,
            call: get_function_call(instruction),
        }
    }
}

fn instr_template_from_raw(instr: &[Instruction]) -> (Vec<InstructionTemplate>, Vec<InstructionTemplate>)
{
    let (raw_unprefixed, raw_prefixed): (Vec<_>, Vec<_>) = instr
        .into_iter()
        .partition(|i| i.code & 0xFF00 != 0xCB00);

    let unprefixed: Vec<_> = raw_unprefixed
        .into_iter()
        .map(|i| InstructionTemplate::from(i))
        .collect();
    let prefixed: Vec<_> = raw_prefixed
        .into_iter()
        .map(|i| InstructionTemplate::from(i))
        .collect();

    (unprefixed, prefixed)
}

pub fn run(args: &GenerateArgs) -> Result<(), Box<dyn Error>> {
    println!("{:#?}", args);

    let mut tera = Tera::default();
    tera.load_from_glob(&format!(
        "{}/**/*",
        args.template_path.to_str().unwrap_or("templates")
    ))?;

    let file = File::open(&args.oplist).expect("Op list not found");
    let raw_instructions: Vec<Instruction> =
        serde_yaml_ng::from_reader(file).expect("Unpack error");

    let (instructions, prefixed_instructions) = instr_template_from_raw(&raw_instructions);

    let mut context = Context::new();
    context.insert("instructions", &instructions);
    context.insert("prefixed_instructions", &prefixed_instructions);

    Ok(())
}
