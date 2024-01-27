from yaml import safe_load

operand_mnemonics = {
    'a16': 'NN',
    'd16': 'NN',
    'r8' : "N",
    'd8' : "N",
}

arithmetic16 = ['inc', 'dec', 'add']
jump = ['jr', 'jp', 'ret', 'reti']
conditions = ["Condition::NotCarry", "Condition::Carry", "Condition::NotZero", "Condition::Zero"]

operand_patterns = {
    'af' : 'Reg16::AF',
    'bc' : 'Reg16::BC',
    'de' : 'Reg16::DE',
    'hl' : 'Reg16::HL',
    'sp' : 'Reg16::SP',
    'pc' : 'Reg16::PC',
    'a' : 'Reg8::A',
    'b' : 'Reg8::B',
    'c' : 'Reg8::C',
    'd' : 'Reg8::D',
    'e' : 'Reg8::E',
    'h' : 'Reg8::H',
    'l' : 'Reg8::L',
    'r8' : "Imem8",
    'd8' : "Imem8",
    'd16' : "Imem16",
    'a16' : "Imem16",
    'nc': "Condition::NotCarry",
    'cf': "Condition::Carry",
    'nz': "Condition::NotZero",
    'z': "Condition::Zero",
}

template = """0x{:02X} => Some(&Instruction {{
    c_cycles: {:d},
    conditional_c_cycles: {},
    mnemonic: \"{}\",
    execute: |cpu: &mut Cpu| {}
}})"""

def get_function_call(instruction: str):
    params = ['cpu']
    for operand in instruction["operands"]:
        operand = str(operand)
        template = '{}'
        if '(' in operand or ')' in operand:
            template = 'Mem({})'
        operand = operand.replace("(", '').replace(')', '')
        param = operand_patterns.get(operand, operand)
        params.append(template.format(param))
    function_name = instruction['operator']
    if function_name == 'stop':
        return " {cpu.stop(); Timing::Normal}"
    else:
        if len(params) > 1 and "16" in params[1] and function_name in arithmetic16:
            function_name+='16'
        elif len(params) <= 1 and function_name in jump :
            params.append('Condition::Unconditional')
        elif function_name in jump and not any(param in conditions for param in params):
            params.insert(1, 'Condition::Unconditional')
        params = ', '.join(params)
    return f"{function_name}({params})"
     

instructions = None
with open("instructions.yml", 'r') as f:
    instructions = safe_load(f)

if instructions is not None:
    contents = []
    
    for instruction in instructions:
        # print(instruction)
        code = instruction['code']
        
        c_cycles = instruction['time']
        conditional_cycles = "None"
        if isinstance(c_cycles, list):
            conditional_cycles, c_cycles = c_cycles
            conditional_cycles = f"Some({conditional_cycles})"
            
        operands = [str(operand) for operand in instruction['operands']]
        operands = [operand_mnemonics.get(operand.replace('(', '').replace(')', ''), operand) for operand in operands]
        operands = ",".join(operands)
        mnemonic = instruction['operator'] + (' ' if operands else '') + operands
        mnemonic = mnemonic.upper()
        execute = get_function_call(instruction)
        
        contents.append(template.format(instruction['code'], c_cycles, conditional_cycles, mnemonic, execute))
        
    print(contents[20])
    with open("parsed_instructions.txt", 'w') as f:
        f.write(',\n'.join(contents))