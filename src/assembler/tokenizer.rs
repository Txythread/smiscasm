use std::process::Command;
use crate::config::*;
use crate::util::replacement::Replacement;
use crate::assembler::valuerepl::{LineKind, ValueReplResult};
use crate::assembler::tokenizer::InstructionArgs::{Global, Immediate, Register};
use crate::assembler::valuegen::Section;
use crate::util::code_error::ErrorNotificationKind;
use crate::util::line_mapping::LineMap;

// Yet another tokenizer
/// Tokenizes ValueReplResult into YATokenizerResult
pub fn tokenize(from: ValueReplResult, mut input_line_map: LineMap) -> (TokenizerResult, LineMap) {
    let mut result = TokenizerResult {
        code: vec![],
        global_constants: vec![],
        sections: from.sections.iter().clone().map(|x|x.clone()).collect(),
    };
    let mut output_line_map = LineMap::new();

    // Copy all global constants
    result.global_constants = from.global_constants.clone().iter().filter(|&x| x.get_is_global() || x.get_is_function()).map(|x| x.clone()).collect();

    // Go through all the lines
    'line_loop: for i in from.code.iter().enumerate() {
        let line_number = i.0;

        let line = i.1.clone();
        let kind = line.1.clone();
        let code = line.0;

        // Modify this later to contain new token mapping if needed
        let line_info = input_line_map.lines[line_number].clone();

        match kind {
            LineKind::ASCII => {
                result.code.push(Line::RAW(code[0].clone().into_bytes()));
                continue;
            }

            LineKind::STC => {
                let text = code[0].clone();

                // Convert using smisc-connect as it has the required functionality
                // Then make a string from the output.
                let mut stc_values_string = Command::new("smisc-connect".to_string())
                    .arg("--convert-to-stc".to_string())
                    .arg(text.clone())
                    .output()
                    .expect("Failed to execute smisc-connect. Is smisc-connect installed?")
                    .stdout;

                stc_values_string.remove(stc_values_string.len() - 1);


                let stc_values_string =
                    stc_values_string
                    .iter().map(|x|x.clone() as char)
                    .map(|x|x.to_string())
                    .collect::<Vec<String>>()
                    .join("");


                // The output from smisc-connect should contain one value per line.
                // Separate each line to an element in a vector.
                let stc_values_array = stc_values_string.split(':').collect::<Vec<&str>>();

                let stc_values = stc_values_array.iter().map(|&x| x.parse().unwrap()).collect();

                result.code.push(Line::RAW(stc_values));
            }

            LineKind::Code(_) => {
                let name = code[0].clone();

                let mut args: Vec<InstructionArgs> = vec![];
                let mut current_token_index = 1u32;
                for token in code[1..].iter() {
                    current_token_index += 1;
                    // If it can be passed immediately, it's just an immediate value.
                    if let Some(value) = token.parse::<i32>().ok() {
                        if value.abs() > MEMORY_PAGE_SIZE as i32 {
                            input_line_map.print_notification(ErrorNotificationKind::Error, line_number as u32, Some(current_token_index), "Value Outside Of Immediate Range".to_string(), format!("Value of {} decimal is not in the range of an immediate value ({} decimal to {} decimal).", value, -(MEMORY_PAGE_SIZE as isize), MEMORY_PAGE_SIZE - 1));
                        }
                        // Correct the format
                        // Choose last 12 bits
                        let mut immediate_value: u16 = (value & 0x00_00_0F_FF) as u16;

                        if value.is_negative() {
                            immediate_value = immediate_value | 0x00_00_10_00;
                        }

                        args.push(Immediate(immediate_value));
                        continue;
                    }

                    // If the first char is an 'x' & it can be decoded, it's a register.
                    let first_char = token.clone().chars().nth(0).unwrap();
                    let mut token_except_first_char = token.to_string().clone();
                    token_except_first_char.remove(0);

                    if first_char == 'x'{
                        if let Some(value) = token_except_first_char.parse::<i32>().ok(){
                            if value > 31 || value < 0 {
                                input_line_map.print_notification(ErrorNotificationKind::Error, line_number as u32, Some(current_token_index - 1), "Unknown Register".to_string(), format!("No such register: \"{}\". Known registers include x0...x31 and sp (which links to x31).", token));
                                continue 'line_loop;
                            }


                            args.push(Register(value as u8));
                            continue;
                        }
                    }

                    // If neither of the above way works, it's a global constant.
                    args.push(Global(token.clone()));
                }


                output_line_map.add_line(line_info);
                result.code.push(Line::Instruction(name, args.clone()));
            }
        }
    }


    output_line_map.errors_count = input_line_map.errors_count;
    output_line_map.warnings_count = input_line_map.warnings_count;


    output_line_map.exit_if_needed();

    (result, output_line_map)
}

pub struct TokenizerResult {
    pub code: Vec<Line>,
    pub global_constants: Vec<Replacement>,
    pub sections: Vec<Section>,
}

#[derive(Debug, PartialEq, Clone)]
pub enum Line{
    Instruction(String, Vec<InstructionArgs>),      // Name and args
    RAW(Vec<u8>),                                   // Raw data (such as STC text values)
}


#[derive(Debug, Clone, PartialEq)]
pub enum InstructionArgs{
    Register(u8),                                   // Only least significant 6 bits
    Immediate(u16),                                 // Least significant 11 are value, the 12th one from the right is the sign, the 13th one is "shift by eleven bits"
    Global(String),                                 // The name of the global constant
}


#[cfg(test)]
mod tests {
    use crate::assembler::valuerepl::{LineKind, ValueReplResult};
    use crate::assembler::tokenizer::{tokenize, InstructionArgs, Line};
    use crate::assembler::valuegen::Section;
    use crate::util::line_mapping::LineMap;
    use crate::util::replacement::Replacement;

    #[test]
    fn test_tokenize(){
        let mut input_constants = vec![
            Replacement::new("xyz".to_string(), "10".to_string(), false),
            Replacement::new("zyx".to_string(), "16".to_string(), false),
            Replacement::new("abc".to_string(), "20".to_string(), false),
            Replacement::new("main".to_string(), "CODE:0".to_string(), true),
            Replacement::new("msg".to_string(), "DATA:0".to_string(), true),
        ];

        input_constants[0].set_is_global(true);
        input_constants[3].set_is_global(true);

        let input_sections : Vec<Section>= vec![
            Section { name: "CODE".to_string(), start_offset: 0, start_memory_page: 0, start_pos_bytes_original: 0 },
            Section { name: "DATA".to_string(), start_offset: 1, start_memory_page: 1, start_pos_bytes_original: 1 },
        ];


        let input_code: Vec<(Vec<String>, LineKind)> = vec![
            (vec!["adrp".to_string(), "x0".to_string(), "2048".to_string()], LineKind::Code(false)),
            (vec!["add".to_string(), "x0".to_string(), "0".to_string()], LineKind::Code(false)),
            (vec!["Hi".to_string()], LineKind::ASCII)
        ];



        let line_mapping_input = vec![(0, 1), (1, 1), (2, 2)];

        let input = ValueReplResult {
            global_constants: input_constants,
            sections: input_sections,
            code: input_code,
            line_mapping: line_mapping_input,
        };

        let expected_output_code: Vec<Line> = vec![
            Line::Instruction("adrp".to_string(), vec![InstructionArgs::Register(0), InstructionArgs::Immediate(2048)]),
            Line::Instruction("add".to_string(), vec![InstructionArgs::Register(0), InstructionArgs::Immediate(0)]),
            Line::RAW("Hi".to_string().into_bytes()),
        ];

        let line_map = LineMap::test_map();

        let result = tokenize(input, line_map);
        let output_code = result.0.code.clone();

        assert_eq!(output_code.len(), expected_output_code.len());

        for i in 0..expected_output_code.len() {
            let expected = expected_output_code[i].clone();
            let output = output_code[i].clone();

            assert_eq!(output, expected);
        }



    }
}