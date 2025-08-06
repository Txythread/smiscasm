use std::process::exit;
use std::string::ToString;
use std::fs;
use colorize::AnsiColor;
use crate::util::remove_comments::remove_comments_in_line;
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

    /// Check whether a instruction's format is possible
    pub fn check(&self) -> bool /*should be zero*/{
        // Check length
        if self.format.iter().count() > 3{
            return false;
        }

        // Check whether any other argument than the last one is an immediate value
        let mut format = self.format.clone();
        format.remove(0);
        for i in format.iter() {
            if *i { return false /* nope */ }
        }

        false /*success*/
    }

    pub fn from_string(string: String) -> Instruction {
        /// List of all available micro operations
        let output_map_str: Vec<&str> = vec!["PC_OUT", "PC_IN", "PC_INC", "MEM_ADDR_PTR_IN"]; // The left-most string in the list will end up in the LSb of the control 'word'

        let output_map: Vec<String> = output_map_str.into_iter().map(|s| s.to_string()).collect();


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
                    let error = format!("Unknown expression ({}).\nShould be x for register or i for immediate value.\nThis was found in the code for the {} instruction.", format_expression, name).red().to_string();
                    eprintln!("{}", error);
                    exit(103);
                }
            }
        }

        let op_code_string_with_whitespaces = remove_comments_in_line(lines.first().unwrap().to_string());
        let op_code_string = op_code_string_with_whitespaces.split_whitespace().nth(0);

        if op_code_string.is_none() {
            let error = format!("No op-code provided for instruction {}.", name).red().to_string();
            eprintln!("{}", error);
            exit(103);
        }

        let op_code = u16::from_str_radix(op_code_string.unwrap(), 16);

        if op_code.is_err() {
            let error = format!("Couldn't generate number from string {} which should refer to {} instruction's OP-Code.", op_code_string.unwrap(), name).red().to_string();
            eprintln!("{}", error);
            exit(103);
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

            if line == "@STAGE" {
                result.stages.push((call_word, current_stage_control_word));
                current_stage_control_word = 0;
                let mut current_stage = call_word & 0x1F; // The last five
                current_stage += 1;
                call_word = call_word & 0xFFE0; // remove the last five (current stage)
                call_word = call_word | current_stage;
                continue;
            }

            // Go through every word and find the one that fits and add that info to the control word
            for i in output_map.iter().enumerate(){
                let instruction_name = i.1.to_string();

                if instruction_name != line { continue; }

                let i = i.0;
                current_stage_control_word = current_stage_control_word | (1 << i);
            }
        }

        result.stages.push((call_word, current_stage_control_word));

        result
    }
}


pub fn get_all_instructions(location: String) -> Vec<Instruction> {
    let paths = fs::read_dir(location).unwrap();
    let mut instructions: Vec<Instruction> = Vec::new();

    for path in paths {
        if path.is_err() { continue; }

        let path = path.unwrap().path();

        let file = fs::read_to_string(path.to_str().unwrap());

        if file.is_err() { continue; }
        let file = file.unwrap();

        let instruction = Instruction::from_string(file);

        if instruction.check() {
            let error = format!("Instruction named {} didn't pass instruction check.", instruction.name).red().to_string();
            eprintln!("{}", error);
            exit(103);
        }

        instructions.push(instruction);
    }

    instructions
}


#[cfg(test)]
mod tests {
    use crate::instruction::instruction::get_all_instructions;

    #[test]
    fn test_get_all_instructions() {
        let _ = get_all_instructions("/Users/michaelrudolf/Development/Rust/smiscasm/instructions/".to_string());
    }
}