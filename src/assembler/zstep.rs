use colorize::AnsiColor;
use crate::assembler::assembler::MEMORY_PAGE_SIZE;
use crate::assembler::ya_tokenizer::{InstructionArgs, Line, YATokenizerResult};
use crate::instruction::instruction::*;
use crate::util::exit::{exit, exit_with_variant, ExitCode};
// This name is really bad, ik
// I just want RustRover to sort all of them in a reasonable order.
// And it's alphabetically.
// The Z represents 'last'

/// Turns YATokenizerResult into the final binary
pub fn perform_last_step(input: YATokenizerResult, instructions: Vec<Instruction>) -> Vec<u8> {
    let mut result: Vec<u8> = Vec::new();
    let mut next_page_start = MEMORY_PAGE_SIZE;
    let mut section_starts_in_lines = input.sections.iter().map(|x| x.clone().1).collect::<Vec<u32>>(); // The lines' (in the input code) indexes that belong to a new section. The first element is always the next section.
    let mut current_section_name = input.sections.iter().nth(0).unwrap().clone().0;
    let mut actual_bytes_written: u32 = 0; // The bytes written that were not written for memory page aligning
    section_starts_in_lines.remove(0);

    // Add commands and data
    for line in input.code.iter().enumerate() {
        let i = line.0;
        let line = line.1.clone();

        // Check if the start of the next section is at the current byte
        if let Some(next_section_start) = section_starts_in_lines.iter().nth(0){
            let next_section_start = *next_section_start;
            if actual_bytes_written == next_section_start{
                // Look if the section was too long
                if result.len() > next_page_start {
                    // Too long, throw error
                    exit(format!("Section \"{}\"", current_section_name), ExitCode::BadCode);
                }
                // Start a new section
                // Find the section's name
                current_section_name = input.sections.iter().find(| &x | x.clone().1 == actual_bytes_written).unwrap().clone().0;
                // Fill the section with 0s
                while result.len() < next_page_start {
                    // Push without incrementing the actual_bytes_written
                    result.push(0);
                }
                // Change the next page's start
                next_page_start += MEMORY_PAGE_SIZE;
                // Change the section_starts_in_lines
                section_starts_in_lines.remove(0);
            }
        }


        // Get real line number
        let real_line_number = input.line_mapping.iter().find(| &&x | x.0 == i);

        if real_line_number.is_none() {
            exit_with_variant(format!("Internal error with line mapping likely caused by \"valuegen\" or \"valuerepl\"., the line number in it's resulting code is: {}.", i), ExitCode::Internal, 2);
        }

        let real_line_number = real_line_number.unwrap().1;

        // Actually convert & add instructions & data to binary.
        match line {
            Line::Instruction(name, args) => {
                let name = name.clone();
                let args = args.clone();

                // The format of the instruction
                // 0 means register and 1 means immediate value
                // This is to keep instructions with multiple versions (e.g. add reg, reg; add reg, imm) apart
                let mut format: Vec<bool> = Vec::new();

                for arg in args.iter() {
                    if matches!(arg, InstructionArgs::Register(_)) {
                        format.push(false);
                        continue;
                    }
                    format.push(true);
                }

                // Find a matching instruction
                let instruction = instructions.iter().find(| &x | x.clone().name == name && x.clone().format == format);

                if instruction.is_none(){
                    let mut error = format!("Instruction '{}' from line {} with format (", name, real_line_number);
                    let mut error_format_string = String::new();
                    for format in format.iter() {
                        // If there is something in the format string already, print a colon
                        if error_format_string != String::new(){
                            error_format_string.push_str(", ");
                        }
                        if *format {
                            error_format_string += "Immediate Value";
                        } else {
                            error_format_string += "Register";
                        }
                    }

                    error += error_format_string.as_str();
                    error += ") doesn't exist.";
                    error = error.red().to_string();

                    exit(error, ExitCode::BadCode);
                }

                let instruction = instruction.unwrap().clone();

                let op_code: u16 = instruction.op_code.into();
                let mut reg0 = 0u8;
                let mut reg1 = 0u8;
                let mut immediate = 0u16;
                let mut global: Option<String> = None;

                for x in args.iter().enumerate() {
                    let arg = x.1.clone();
                    let i = x.0.clone();

                    match arg{
                        InstructionArgs::Register(register) => {
                            if i == 0{
                                reg0 = register;
                            } else {
                                reg1 = register;
                            }
                        }

                        InstructionArgs::Immediate(value) => {
                            immediate = value;
                        }

                        InstructionArgs::Global(name) => {
                            global = Some(name.clone());
                        }
                    }
                }

                // The instruction ignoring the global constant.
                let mut instruction_coded: u32 = (op_code as u32) << 23;
                instruction_coded |= (reg0 as u32) << 18;
                instruction_coded |= (reg1 as u32) << 13;
                instruction_coded |= immediate as u32;

                append_u32_to_vec(&mut result, &mut actual_bytes_written, instruction_coded);

                if let Some(global) = global {
                    append_ascii_string_to_vec(&mut result, &mut actual_bytes_written, global);
                    append_u8_to_vec(&mut result, &mut actual_bytes_written, 0xff)
                }
            }

            Line::ASCII(text) => {
                append_ascii_string_to_vec(&mut result, &mut actual_bytes_written, text.clone());
            }
        }
    }

    result
}

fn append_u8_to_vec(x: &mut Vec<u8>, size: &mut u32, data: u8){
    x.push(data);
    *size += 1;
}

#[allow(dead_code)]
fn append_u16_to_vec(x: &mut Vec<u8>, size: &mut u32, data: u16) {
    let msb: u8 = ((data & 0xFF00) >> 8) as u8;
    let lsb: u8 = ((data & 0x00FF) >> 0) as u8;
    x.push(msb);
    x.push(lsb);
    *size += 2;
}

fn append_u32_to_vec(x: &mut Vec<u8>, size: &mut u32, data: u32) {
    let msb: u8 = ((data & 0xFF000000) >> 24) as u8;
    let by1: u8 = ((data & 0x00FF0000) >> 16) as u8;
    let by2: u8 = ((data & 0x0000FF00) >> 8) as u8;
    let lsb: u8 = (data & 0x000000FF) as u8;
    x.push(msb);
    x.push(by1);
    x.push(by2);
    x.push(lsb);
    *size += 4;
}

fn append_ascii_string_to_vec(x: &mut Vec<u8>, size: &mut u32, string: String){
    for char in string.chars() {
        x.push(char as u8);
    }
    *size += string.len() as u32;
}
