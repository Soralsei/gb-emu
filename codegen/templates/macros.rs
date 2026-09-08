{% component instruct_case(instruct) %}
            0x{{ instruct.code }} => Instruction {
                cycles: {{ instruct.cycles }},
                mnemonic: "{{ instruct.mnemonic }}",
                execute: {{ instruct.call }},
            },
{%- endcomponent instruct_case %}
