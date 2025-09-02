use std::string::ToString;
use include_dir::{include_dir, Dir};
use crate::util::exit::{ exit, ExitCode };
use crate::util::remove_comments::remove_comments_in_line;

const INSTRUCTION_DIR: Dir = include_dir!("$CARGO_MANIFEST_DIR/instructions");
const OUTPUT_MAP_STRING: [&str; 29] = ["PC_OUT", "PC_IN", "PC_INC", "MEM_ADDR_PTR_IN", "ALU_IN_A", "ALU_IN_B", "CAL_REG_A_IN", "CAL_REG_B_IN", "CAL_REG_A_OUT", "CAL_REG_B_OUT", "IMMEDIATE_OUT", "INSTR_IN", "MEM_OUT", "PLUS_OUT", "RESET_MICRO", "STDTRANS_IN", "STDTRANS_OUT", "STDTRANS_SEND", "ZF_IN", "ZF_OUT", "SUB_OUT", "MEM_BYTE_OUT", "LSH_OUT", "SP_OUT", "SP_IN", "MEM_IN", "SP_DEC_DW", "SP_INC_DW", "MEM_B_IN"]; // The left-most string in the list will end up in the LSb of the control 'word'

#[derive(Debug)]
pub struct Instruction {
    pub name: String,
    pub format: Vec<bool>,              // A 0 is a register, a 1 an immediate value
    pub op_code: u16,                   // This is only 8 bits rn, but the encoding would allow expanding to up to 9 bits
    pub stages: Vec<(u16, u64)>
}

impl Instruction {
    pub fn new(name: String, format: Vec<bool>, stages: Vec<(u16, u64)>) -> Instruction {
        Instruction { name, format, op_code: 0, stages }
    }

    pub fn new_with_op(name: String, format: Vec<bool>, op_code: u16, stages: Vec<(u16, u64)>) -> Instruction {
        Instruction { name, format, op_code, stages }
    }

    /// Check whether an instruction's format is possible
    pub fn check(&self) -> bool /*should be zero*/{
        // Check length
        if self.format.iter().count() > 3{
            return true;
        }

        // Check whether any other argument other than the last one is an immediate value.
        let mut format = self.format.clone();
        if format.len() != 0{
            format.remove(0);
            for i in format.iter() {
                if *i { return false /* nope */ }
            }
        }

        false /*success*/
    }

    pub fn from_string(string: String) -> Instruction {
        // List of all available micro operations
        let output_map: Vec<String> = OUTPUT_MAP_STRING.clone().into_iter().map(|s| s.to_string()).collect();

        let mut lines = string.lines().collect::<Vec<&str>>();

        let mut name = remove_comments_in_line(lines.first().unwrap().to_string());
        name = name.split_whitespace().nth(0).unwrap().to_string();
        lines.remove(0);

        let format_string = remove_comments_in_line(lines.first().unwrap().to_string()).split_whitespace().collect::<Vec<&str>>().iter().map(|s| s.to_string()).collect::<Vec<String>>();
        let mut format_vec: Vec<bool> = Vec::new();
        lines.remove(0);

        for format_expression in format_string.iter() {
            match (*format_expression).as_str() {
                "x" => {
                    format_vec.push(false);
                }

                "i" => {
                    format_vec.push(true);
                }

                _ => {
                    exit(format!("Unknown expression ({}).\nShould be x for register or i for immediate value.\nThis was found in the code for the {} instruction.", format_expression, name), ExitCode::Other);
                }
            }
        }

        let op_code_string_with_whitespaces = remove_comments_in_line(lines.first().unwrap().to_string());
        let op_code_string = op_code_string_with_whitespaces.split_whitespace().nth(0);

        if op_code_string.is_none() {
            exit(format!("No op-code provided for instruction {}.", name), ExitCode::Other);
        }

        let op_code = u16::from_str_radix(op_code_string.unwrap(), 16);

        if op_code.is_err() {
            exit(format!("Couldn't generate number from string {} which should refer to {} instruction's OP-Code.", op_code_string.unwrap(), name), ExitCode::Other);
        }

        let op_code = op_code.unwrap();

        let mut result = Instruction::new(name, format_vec, Vec::new());
        result.op_code = op_code;

        let mut call_word: u16 = (op_code & 0x01FF) << 5;
        let mut current_stage_control_word: u64 = 0;

        for line in lines {
            let line_with_whitespaces = remove_comments_in_line(line.to_string());
            let line = line_with_whitespaces.split_whitespace().nth(0);

            if line.is_none() { continue; }
            let line = line.unwrap();

            match line {
                "@STAGE" | "@VERSION" => {
                    result.stages.push((call_word, current_stage_control_word));
                    current_stage_control_word = 0;
                    let mut current_stage = call_word & 0x1F; // The last five
                    if line == "@STAGE"{
                        current_stage += 1;
                    }
                    call_word = call_word & 0x3FE0; // remove the last five (current stage) & the first two (flags)
                    call_word = call_word | current_stage;
                }

                "@ZF" => {
                    // Enable the "Zero Flag" bit in the call_word
                    call_word = call_word | 0x4000u16;
                    continue;
                }

                "@PM" => {
                    // Just enable the "Privileged Mode Flag" bit in the call_word
                    call_word = call_word | 0x8000u16;
                    continue;
                }

                _ => {
                    // Go through every word and find the one that fits and add that info to the control word
                    for i in output_map.iter().enumerate(){
                        let instruction_name = i.1.to_string();

                        if instruction_name != line { continue; }

                        let i = i.0;
                        current_stage_control_word = current_stage_control_word | (1 << i);
                    }
                }
            }
        }

        result.stages.push((call_word, current_stage_control_word));

        result
    }
}

pub fn micro_operation_at(idx: usize) -> String {
    if idx >= OUTPUT_MAP_STRING.len() {
        return "OUT OF RANGE".to_string();
    }
    OUTPUT_MAP_STRING[idx].to_string()
}


impl Clone for Instruction {
    fn clone(&self) -> Instruction {
        Instruction::new_with_op(self.name.clone(), self.format.clone(), self.op_code, self.stages.clone())
    }
}

pub fn get_all_instructions() -> Vec<Instruction> {
    let files = INSTRUCTION_DIR.files();
    let mut instructions: Vec<Instruction> = Vec::new();

    for file in files.enumerate() {
        let file_contents = file.to_owned().1.contents_utf8().unwrap();

        let instruction = Instruction::from_string(file_contents.to_string());

        if instruction.check() {
            exit(format!("Instruction named {} didn't pass instruction check.", instruction.name), ExitCode::Other);
        }

        // Don't ask why
        instructions.push(Instruction {
            name: instruction.name.clone(),
            format: instruction.format.clone(),
            op_code: instruction.op_code.clone(),
            stages: instruction.stages.clone(),
        });
    }

    // Check whether there are instructions with the same op codes.

    let instruction_op_codes = instructions.iter().map(|i| i.op_code.clone()).collect::<Vec<u16>>();

    instructions.iter().for_each(| x | {
        let x = x.clone();

        // The amount of instructions with the same OP-Code (including the instruction itself)
        let op_code_frequency = instruction_op_codes.iter().filter(|&y| *y == x.op_code).count();

        if op_code_frequency != 1 {
            exit(format!("OP-Codes should be unique to each instruction, but there are {} more instructions using the same op-codes as the \"{}\" instruction.", op_code_frequency - 1, x.name), ExitCode::Other);
        }
    });

    instructions
}


#[cfg(test)]
mod tests {
    use crate::instruction::instruction::get_all_instructions;

    #[test]
    fn test_get_all_instructions() {
        let _ = get_all_instructions();
    }
}