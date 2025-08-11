use std::process::exit;
use colorize::AnsiColor;
use crate::assembler::ya_tokenizer::{InstructionArgs, Line, YATokenizerResult};
use crate::instruction::instruction::*;


// This name is really bad, ik
// I just want RustRover to sort all of them in a reasonable order.
// And it's alphabetically.
// The Z represents 'last'

pub fn perform_last_name(input: YATokenizerResult, instructions: Vec<Instruction>) -> Vec<u8> {
    let mut result: Vec<u8> = Vec::new();

    // Add the amount of sections
    append_u16_to_vec(&mut result, input.sections.len() as u16);
    // Add the amount of global constants
    append_u16_to_vec(&mut result, input.global_constants.len() as u16);

    // Add all the sections
    for section in input.sections.iter() {
        let name = section.clone().0;
        let start = section.clone().1;

        append_ascii_string_to_vec(&mut result, name);
        append_u8_to_vec(&mut result, 0xfa);
        append_u32_to_vec(&mut result, start);
        append_u8_to_vec(&mut result, 0xfb);
    }

    // Add all the constants
    for global in input.global_constants.iter() {
        let name = global.clone().get_name().clone();
        let value = global.clone().get_value();

        append_ascii_string_to_vec(&mut result, name);
        append_u8_to_vec(&mut result, 0xfa);
        if let Some(value) = value.parse::<i32>().ok() {
            append_u32_to_vec(&mut result, value as u32);
        }else{
            append_ascii_string_to_vec(&mut result, value.to_string());
        }
        append_u8_to_vec(&mut result, 0xfb);
    }

    // Add the amount of lines using a global constant
    let lines_with_global_values = lines_with_global_values(input.code.clone());
    append_u16_to_vec(&mut result, lines_with_global_values.len() as u16);

    // Add all the lines
    for line in lines_with_global_values.iter() {
        append_u32_to_vec(&mut result, *line);
    }

    // Add commands and data
    for line in input.code.iter().enumerate() {
        let i = line.0;
        let line = line.1.clone();


        // Get real line number
        let real_line_number = input.line_mapping.iter().find(| &&x | x.0 == i);

        if real_line_number.is_none() {
            let error = format!("Internal error with line mapping likely caused by \"valuegen\" or \"valuerepl\"., the line number in it's resulting code is: {}.", i).red().to_string();
            eprintln!("{}", error);
            exit(299);
        }

        let real_line_number = real_line_number.unwrap().1;


        match line {
            Line::Instruction(name, args) => {
                let name = name.clone();
                let args = args.clone();

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
                    let mut error = format!("Instruction '{}' from line {} with format (", name, real_line_number).to_string();
                    for format in format.iter() {
                        if *format {
                            error += ", Immediate Value";
                        } else {
                            error += "Register";
                        }
                    }
                    error += ") doesn't exist.";
                    error = error.red().to_string();
                    eprintln!("{}", error);
                    exit(105);
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

                append_u32_to_vec(&mut result, instruction_coded);

                if let Some(global) = global {
                    append_ascii_string_to_vec(&mut result, global);
                    append_u8_to_vec(&mut result, 0xff)
                }
            }

            Line::ASCII(text) => {
                append_ascii_string_to_vec(&mut result, text.clone());
            }
        }
    }

    result
}

fn lines_with_global_values(code: Vec<Line>) -> Vec<u32>{
    let mut result: Vec<u32> = Vec::new();

    let bytes_per_instruction = 4; // 32 bits
    let mut bytes_count = 0;

    for x in code.iter().enumerate() {
        let line = x.1.clone();

        match line {
            Line::Instruction(_, args) => {
                let global = args.iter().find(|&x| matches!(x.clone(), InstructionArgs::Global(_)));

                if global.is_some() {
                    result.push(bytes_count);
                }

                bytes_count += bytes_per_instruction;
            }
            Line::ASCII(text) => {
                bytes_count = bytes_count + text.len() as u32;
            }
        }
    }

    result
}

fn append_u8_to_vec(x: &mut Vec<u8>, data: u8){
    x.push(data);
}

fn append_u16_to_vec(x: &mut Vec<u8>, data: u16) {
    let msb: u8 = ((data & 0xFF00) >> 8) as u8;
    let lsb: u8 = ((data & 0x00FF) >> 0) as u8;
    x.push(msb);
    x.push(lsb);
}

fn append_u32_to_vec(x: &mut Vec<u8>, data: u32) {
    let msb: u8 = ((data & 0xFF000000) >> 24) as u8;
    let by1: u8 = ((data & 0x00FF0000) >> 16) as u8;
    let by2: u8 = ((data & 0x0000FF00) >> 8) as u8;
    let lsb: u8 = (data & 0x000000FF) as u8;
    x.push(msb);
    x.push(by1);
    x.push(by2);
    x.push(lsb);
}

fn append_ascii_string_to_vec(x: &mut Vec<u8>, string: String){
    for char in string.chars() {
        x.push(char as u8);
    }
}
