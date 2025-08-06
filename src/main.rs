use std::env;
use std::fmt::{Debug, Formatter};
use std::process::exit;
use std::fs;
use std::path::PathBuf;
use colorize;
use colorize::AnsiColor;
use crate::instruction::instruction::Instruction;
use std::fs::File;
use std::io::prelude::*;

mod util;
mod instruction;
mod assembler;

pub struct ArgumentList{
    file: Option<String>,
    test: bool,                     // -t or --test
    output_name: Option<String>,    // -o or --output
    generate_instruction_table: bool,
}

impl ArgumentList{
    pub fn new() -> ArgumentList{
        ArgumentList{file: None, test: false, output_name: None, generate_instruction_table: false}
    }
}

impl Debug for ArgumentList{
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ArgumentList")
            .field("file", &self.file)
            .field("test", &self.test)
            .field("output_name", &self.output_name)
            .finish()
    }
}

impl PartialEq for ArgumentList{
    fn eq(&self, other: &Self) -> bool{
        self.file == other.file && self.test == other.test
    }
}


fn main() {
    let instructions_location = "/Users//Development/Rust/smiscasm/instructions".to_string();

    // Retrieve arguments from the terminal first
    let cli_args: Vec<String> = env::args().collect();

    // Generate a reasonable argument list
    let args = get_arguments_from_list(cli_args);

    if args.generate_instruction_table{ generate_instruction_table(instructions_location); return; }

    // Load the instructions
    let instructions = instruction::instruction::get_all_instructions(instructions_location);

    // Load the file
    let path = expand_path(&args.file.unwrap()).unwrap();
    let input_file = fs::read_to_string(path).unwrap();


    println!("{}", input_file);



}


// This function is slow as fuck, I know that and I got ideas on how to solve it.
// But it's not slow enough and doesn't take up too much RAM (on my machine at least; lol) for me to seriously care about it.
fn generate_instruction_table(location: String) {
    let instructions = instruction::instruction::get_all_instructions(location);
    for instruction in instructions.iter() {
        println!("--- {} ---", instruction.name);
        println!("Format: {:?}", instruction.format);
        println!("OP-Code: {:09b}", instruction.op_code);

        for stage in instruction.stages.iter() {
            let caller = stage.0;
            let control_word = stage.1;

            println!("{:016b}: {:064b}", caller, control_word);
        }
    }

    /// All instructions' control words; position in vector counts as address/caller.
    let mut all_control_words: Vec<u64> = vec![];

    for i in 0..65536{
        fn find_matching_stage(i: u16, instructions: Vec<Instruction>) -> Option<u64> {
            for instruction in instructions.iter() {
                for stage in instruction.stages.iter() {
                    let caller = stage.0;

                    if caller == i {
                        return Some(stage.1);
                    }
                }
            }
            None
        }

        // Look for the best match, then look if there are versions without the Z/PM flags set.
        if let Some(matching_stage) = find_matching_stage(i as u16, instructions.clone()){
            all_control_words.push(matching_stage);
            continue;
        }

        // Since there was no corresponding instruction found,
        // i might be above 32768, in which case the MSb refers to the PM (privileged mode) flag.
        // Some instructions only have one (privilege-independent) version tho.
        // So this one might be used, too.
        if let Some(matching_stage) = find_matching_stage((i as u16) & 0x7FFF, instructions.clone()){
            all_control_words.push(matching_stage);
            continue;
        }

        // Since there was no corresponding instruction found,
        // i might be above 16384, in which case the two MSBs (bits not bytes) refer to the PM flag and the ZF.
        // Most instructions are ZF-independent, so ignore it and retry.

        if let Some(matching_stage) = find_matching_stage((i as u16) & 0xBFFF, instructions.clone()){
            all_control_words.push(matching_stage);
            continue;
        }

        // Same as above.
        // Some instructions (if not most of them still) are both ZF and PM flag independent.
        // Rerun ignoring both.

        if let Some(matching_stage) = find_matching_stage((i as u16) & 0x3FFF, instructions.clone()){
            all_control_words.push(matching_stage);
            continue;
        }

        // No matching stage found, so write something with the MSb and only the MSb being a 1.
        // This prevents damage that may arise from multiple buses outputting at once and is a clear signal.
        // The MSb is the least likely to be used for anything, so this fits it perfectly.
        // Thought about it, just push zero for now.
        all_control_words.push(0x0);
    }

    // Now store the list
    let mut file1 = File::create("smiscasm_instructions-1-MSB.o");
    let mut file2 = File::create("smiscasm_instructions-2.o");
    let mut file3 = File::create("smiscasm_instructions-3.o");
    let mut file4 = File::create("smiscasm_instructions-4.o");
    let mut file5 = File::create("smiscasm_instructions-5.o");
    let mut file6 = File::create("smiscasm_instructions-6.o");
    let mut file7 = File::create("smiscasm_instructions-7.o");
    let mut file8 = File::create("smiscasm_instructions-8-LSB.o");

    if file1.is_err() || file2.is_err() || file3.is_err() || file4.is_err() || file5.is_err() || file6.is_err() || file7.is_err() || file8.is_err(){
        let error = "Couldn't create file 'smiscasm_instructions-x.o' to store instructions in.".red().to_string();
        eprintln!("{}", error);
        exit(104);
    }

    // Convert the list of u64 to a list of u8.
    // The output is needed in 8 seperate files for each EEPROM.
    // File one represents the MSB (byte not bit) while file eight represents the LSB.
    let mut control_words_u8: [[u8; 65536]; 8] = [[0; 65536]; 8];

    for i in all_control_words.iter().enumerate() {
        let control_word_u64 = *i.1;
        let i = i.0;
        let part_1 = ((control_word_u64 & 0xFF00000000000000) >> 56) as u8;
        let part_2 = ((control_word_u64 & 0x00FF000000000000) >> 48) as u8;
        let part_3 = ((control_word_u64 & 0x0000FF0000000000) >> 40) as u8;
        let part_4 = ((control_word_u64 & 0x000000FF00000000) >> 32) as u8;
        let part_5 = ((control_word_u64 & 0x00000000FF000000) >> 24) as u8;
        let part_6 = ((control_word_u64 & 0x0000000000FF0000) >> 16) as u8;
        let part_7 = ((control_word_u64 & 0x000000000000FF00) >> 8) as u8;
        let part_8 =  (control_word_u64 & 0x00000000000000FF) as u8;

        control_words_u8[0][i as usize] = part_1;
        control_words_u8[1][i as usize] = part_2;
        control_words_u8[2][i as usize] = part_3;
        control_words_u8[3][i as usize] = part_4;
        control_words_u8[4][i as usize] = part_5;
        control_words_u8[5][i as usize] = part_6;
        control_words_u8[6][i as usize] = part_7;
        control_words_u8[7][i as usize] = part_8;
    }


    file1.unwrap().write_all(&control_words_u8[0]).unwrap();
    file2.unwrap().write_all(&control_words_u8[1]).unwrap();
    file3.unwrap().write_all(&control_words_u8[2]).unwrap();
    file4.unwrap().write_all(&control_words_u8[3]).unwrap();
    file5.unwrap().write_all(&control_words_u8[4]).unwrap();
    file6.unwrap().write_all(&control_words_u8[5]).unwrap();
    file7.unwrap().write_all(&control_words_u8[6]).unwrap();
    file8.unwrap().write_all(&control_words_u8[7]).unwrap();
}


pub fn expand_path(path_str: &str) -> Option<PathBuf> {
    let expanded = if path_str.starts_with("~/") {
        let home = env::var("HOME").ok()?;
        PathBuf::from(home).join(&path_str[2..])
    } else if path_str.starts_with("$PWD/") {
        let pwd = env::var("PWD").ok()?;
        PathBuf::from(pwd).join(&path_str[5..])
    } else {
        PathBuf::from(path_str)
    };

    Some(expanded)
}

fn get_arguments_from_list(args: Vec<String>) -> ArgumentList {
    // Remove the first argument as it's just the name of the bin
    let mut args = args;
    args.remove(0);

    // Make space for the result
    let mut result = ArgumentList::new();

    // Sort the arguments
    // The first out-of-context (not belonging or being connected to a flag (-)) is the input file
    let mut current_flag: Option<String> = None;

    for arg in args {
        if let Some(arg_first_char) = arg.chars().nth(0){
            // Check if this argument is necessary for the last flag
            if let Some(flag) = current_flag.clone(){
                let value = arg.clone();

                match flag.as_str() {
                    "-o" | "--output" => {
                        result.output_name = Some(value);
                    }

                    _=>{
                        let error = format!("Unknown flag {}.", flag).red().to_string();
                        eprintln!("{}", error);
                        exit(100)
                    }
                }

                current_flag = None;
                continue;
            }

            // Check if the argument is a flag
            if arg_first_char == '-' {
                // This is a flag
                // Therefore, look if the next argument also needs to be checked or the argument can be added right away

                match arg.as_str() {
                    "-t" | "--test" => {
                        result.test = true;
                    }

                    "--generate-instruction-table" => {
                        result.generate_instruction_table = true;
                    }
                    _=>{
                        current_flag = Some(arg);
                    }
                }
                continue;
            }

            // The argument is not a flag, nor is it used after a flag, ...
            // ... so it has to be the name of the file
            if result.file.is_some(){
                println!("Result: {:?}", result);
                let error = format!("\"{}\" and \"{:?}\" can't both be input files.", result.file.clone().unwrap(), arg).red().to_string();
                eprintln!("{}", error);
                exit(100);
            }

            // Isn't yet written, so add the file name
            result.file = Some(arg);
        }
    }

    if current_flag.is_some(){
        let error = "All flags that act like parameters must have their second part provided.".red().to_string();
        eprintln!("{}", error);
        exit(100);
    }

    if result.file.is_none(){
        let error = "No input files provided.".red().to_string();
        eprintln!("{}", error);
        exit(100);
    }

    result
}