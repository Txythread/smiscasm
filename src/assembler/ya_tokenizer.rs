use std::process::exit;
use colorize::AnsiColor;
use crate::util::replacement::Replacement;
use crate::assembler::valuerepl::{LineKind, ValueReplResult};
use crate::assembler::ya_tokenizer::InstructionArgs::{Global, Immediate, Register};

// Yet another tokenizer
/// Tokenizes ValueReplResult into YATokenizerResult
pub fn tokenize_ya_time(from: ValueReplResult) -> YATokenizerResult {
    let mut result = YATokenizerResult {
        code: vec![],
        global_constants: vec![],
        sections: from.sections.clone(),
        line_mapping: from.line_mapping.clone(),
    };

    // Copy all global constants
    result.global_constants = from.global_constants.clone().iter().filter(|&x| x.get_is_global() || x.get_is_function()).map(|x| x.clone()).collect();

    // Go through all the lines
    for i in from.code.iter().enumerate() {
        let line_number = i.0;

        // Get real line number
        let real_line_number = from.line_mapping.iter().find(| &&x | x.0 == line_number);

        if real_line_number.is_none() {
            let error = format!("Internal error with line mapping likely caused by \"valuegen\" or \"valuerepl\"., the line number in it's resulting code is: {}.", line_number).red().to_string();
            eprintln!("{}", error);
            exit(299);
        }

        let real_line_number = real_line_number.unwrap().1;


        let line = i.1.clone();
        let kind = line.1.clone();
        let code = line.0;

        match kind {
            LineKind::ASCII => {
                result.code.push(Line::ASCII(code[0].clone()));
                continue;
            }

            LineKind::Code(_) => {
                let name = code[0].clone();

                let mut args: Vec<InstructionArgs> = vec![];
                for token in code[1..].iter() {
                    // If it can be passed immediately, it's just an immediate value.
                    if let Some(value) = token.parse::<i32>().ok(){
                        if value % 2048 == 0 && value != 0{
                            let value = value >> 11; // Ignore the last bits

                            let immediate_value = (value as u16 & 0x7FF)| 0x1000;
                            args.push(Immediate(immediate_value))
                        }else {
                            // Correct the format
                            // Choose last 11 bits
                            let value_lsb: u16 = (value & 0x00_00_07_FF) as u16;

                            if value_lsb as i32 != value && value_lsb as i32 != -value {
                                let error = format!("The value in line {} can't fit into the 11b of an immediate value.", real_line_number).red().to_string();
                                eprintln!("{}", error);
                                exit(105);
                            }

                            let sign_bit_offset = 12;
                            let negative_prefix = (value.is_negative() as u16) << sign_bit_offset;

                            let immediate_value = value_lsb | negative_prefix;
                            args.push(Immediate(immediate_value));
                        }
                        continue;
                    }

                    // If the first char is an 'x' & it can be decoded, it's a register.
                    let first_char = token.clone().chars().nth(0).unwrap();
                    let mut token_except_first_char = token.to_string().clone();
                    token_except_first_char.remove(0);

                    if first_char == 'x'{
                        if let Some(value) = token_except_first_char.parse::<i32>().ok(){
                            if value > 31 || value < 0 {
                                let error = format!("Register number {} doesn't exist but was called at {}.", value, real_line_number).red().to_string();
                                eprintln!("{}", error);
                                exit(105);
                            }

                            args.push(Register(value as u8));
                            continue;
                        }
                    }

                    // If neither of the above way works, it's a global constant.
                    args.push(Global(token.clone()));
                }

                result.code.push(Line::Instruction(name, args.clone()));
            }
        }
    }
    result
}

pub struct YATokenizerResult {
    pub code: Vec<Line>,
    pub global_constants: Vec<Replacement>,
    pub sections: Vec<(String, u32)>,
    pub line_mapping: Vec<(usize, usize)>,
}

#[derive(Debug)]
pub enum Line{
    Instruction(String, Vec<InstructionArgs>),      // Name and args
    ASCII(String),                                  // ASCII Text
}

impl PartialEq for Line{
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Line::ASCII(a), Line::ASCII(b)) => a == b,
            (Line::Instruction(a, x), Line::Instruction(b, y)) => a == b && x == y,
            _ => false,
        }
    }
}

impl Clone for Line {
    fn clone(&self) -> Self {
        match self {
            Line::ASCII(a) => Line::ASCII(a.clone()),
            Line::Instruction(a, b) => Line::Instruction(a.clone(), b.clone()),
        }
    }
}

#[derive(Debug)]
pub enum InstructionArgs{
    Register(u8),                                   // Only least significant 6 bits
    Immediate(u16),                                 // Least significant 11 are value, the 12th one from the right is the sign, the 13th one is "shift by eleven bits"
    Global(String),                                 // The name of the global constant
}

impl PartialEq for InstructionArgs {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Register(a), Register(b)) => a == b,
            (Immediate(a), Immediate(b)) => a == b,
            (Global(a), Global(b)) => a == b,
            _ => false,
        }
    }
}

impl Clone for InstructionArgs {
    fn clone(&self) -> Self {
        match self{
            Immediate(immediate) => Immediate(immediate.clone()),
            Global(global) => Global(global.clone()),
            Register(register) => Register(*register),
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::assembler::valuerepl::{LineKind, ValueReplResult};
    use crate::assembler::ya_tokenizer::{tokenize_ya_time, InstructionArgs, Line};
    use crate::assembler::ya_tokenizer::YATokenizerResult;
    use crate::util::replacement::Replacement;

    #[test]
    fn test_tokenize_ya_time(){
        let mut input_constants = vec![
            Replacement::new("xyz".to_string(), "10".to_string(), false),
            Replacement::new("zyx".to_string(), "16".to_string(), false),
            Replacement::new("abc".to_string(), "20".to_string(), false),
            Replacement::new("main".to_string(), "CODE:0".to_string(), true),
            Replacement::new("msg".to_string(), "DATA:0".to_string(), true),
        ];

        input_constants[0].set_is_global(true);
        input_constants[3].set_is_global(true);

        let input_sections : Vec<(String, u32)>= vec![
            ("CODE".to_string(), 0),
            ("DATA".to_string(), 1),
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
            Line::Instruction("adrp".to_string(), vec![InstructionArgs::Register(0), InstructionArgs::Immediate(0x1001)]),
            Line::Instruction("add".to_string(), vec![InstructionArgs::Register(0), InstructionArgs::Immediate(0)]),
            Line::ASCII("Hi".to_string()),
        ];

        let result: YATokenizerResult = tokenize_ya_time(input);
        let output_code = result.code.clone();

        assert_eq!(output_code.len(), expected_output_code.len());

        for i in 0..expected_output_code.len() {
            let expected = expected_output_code[i].clone();
            let output = output_code[i].clone();

            assert_eq!(output, expected);
        }



    }
}