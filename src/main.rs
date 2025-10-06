use colorize::AnsiColor;
use clap::Parser;
use std::env;
use std::fmt::Debug;
use std::fs;
use std::path::PathBuf;
use crate::instruction::instruction::{Instruction, micro_operation_at};
use crate::assembler::assembler::assemble;
use crate::help::help::{print_help, print_instruction_help};
use std::fs::File;
use std::io::prelude::*;
use crate::util::exit::{exit, ExitCode};

mod util;
mod instruction;
mod assembler;
mod help;
mod config;

#[derive(Debug, PartialEq, Parser)]
pub struct ArgumentList{
    pub file: Option<String>,

    #[clap(short, long)]
    pub help: bool,                                 // -h or --help

    #[clap(long)]
    pub instruction_help: Option<Option<String>>,   // --instruction-help

    #[clap(short, long)]
    pub output_name: Option<Option<String>>,        // -o or --output

    #[clap(short, long, num_args = 0..=1)]
    pub get_micro_operation: Option<Option<String>>,// --get-micro-operation

    #[clap(short, long)]
    pub generate_instruction_table: bool,           // --generate-instructions-table
}

impl ArgumentList{
    pub fn new() -> ArgumentList{
        ArgumentList{file: None, help: false, instruction_help: None, output_name: None, generate_instruction_table: false, get_micro_operation: None}
    }

    /// Checks whether the current amount of data is enough (0) or the file name is missing (1)
    pub fn needs_input_file(&self) -> bool{
        let is_ok = self.help || self.generate_instruction_table || self.file.is_some() || self.get_micro_operation.is_some() || self.instruction_help.is_some();
        !is_ok
    }
}

#[tokio::main]
async fn main() {
    // Retrieve arguments from the terminal first
    let cli_args: Vec<String> = env::args().collect();

    // Generate a reasonable argument list
    let mut args = ArgumentList::parse();

    if args.help { print_help(args); return; }

    if args.instruction_help.is_some() { print_instruction_help(args.instruction_help.unwrap().unwrap()); return; }

    if args.generate_instruction_table { generate_instruction_table(); return; }

    if args.get_micro_operation.is_some() { get_micro_operation(args.get_micro_operation.unwrap().unwrap().to_string()); return;}

    // There is something to assemble

    // Load the instructions
    let instructions = instruction::instruction::get_all_instructions();

    // Load the file

    if let Some(relative_path) = &args.file.clone(){

        let path = expand_path(relative_path).unwrap();
        let input_file = fs::read_to_string(path.clone());


        if input_file.is_err() {
            exit(format!("Input file not found: {}", path.to_str().unwrap().to_string()), ExitCode::BadArgument);
        }

        let input_file = input_file.unwrap();

        // Check the file is in the same dir

        // The amount of dirs between PWD and file
        let amount_of_subdirs = path.to_str().unwrap().split('/').count() - 1;

        if amount_of_subdirs > 0 {
            exit(format!("Input file must be directly beneath the working directory, but there are {} directories in between.", amount_of_subdirs), ExitCode::BadArgument);

    }



        let binary = assemble(input_file, instructions).await;

        // Generate the output file name in case it doesn't exist.
        if args.output_name.is_none(){
            args.output_name = Some(Some(args.file.clone().unwrap().to_string().clone().strip_suffix(".s").unwrap().to_string()));
            args.output_name = Some(Some(args.output_name.unwrap().unwrap().clone() + ".o"));
        }

        let mut file = File::create(args.output_name.clone().unwrap().unwrap()).unwrap();

        file.write_all(binary.as_slice()).unwrap();

        return
    }

    println!("{}", "Nothing to do".to_string().red());
}

fn get_micro_operation(idx: String) {
    let idx_int = idx.parse::<usize>();

    if idx_int.is_err() {
        exit(format!("String \"{}\" is not a valid Micro Operation idx.", idx), ExitCode::BadArgument);
    }

    println!("That would be: {}", micro_operation_at(idx_int.unwrap()));
}

// This function is slow as fuck, I know that and I got ideas on how to solve it.
// But it's not slow enough and doesn't take up too much RAM (on my machine at least; lol) for me to seriously care about it.
fn generate_instruction_table() {
    let instructions = instruction::instruction::get_all_instructions();
    // All instructions' control words; position in vector counts as address/caller.
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
    let file1 = File::create("smiscasm_instructions-1-MSB.o");
    let file2 = File::create("smiscasm_instructions-2.o");
    let file3 = File::create("smiscasm_instructions-3.o");
    let file4 = File::create("smiscasm_instructions-4.o");
    let file5 = File::create("smiscasm_instructions-5.o");
    let file6 = File::create("smiscasm_instructions-6.o");
    let file7 = File::create("smiscasm_instructions-7.o");
    let file8 = File::create("smiscasm_instructions-8-LSB.o");

    if file1.is_err() || file2.is_err() || file3.is_err() || file4.is_err() || file5.is_err() || file6.is_err() || file7.is_err() || file8.is_err(){
        exit("Couldn't create file 'smiscasm_instructions-x.o' to store instructions in.".to_string(), ExitCode::ReadWriteError);
    }

    // Convert the list of u64 to a list of u8.
    // The output is needed in 8 separate files for each EEPROM.
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

        control_words_u8[0][i] = part_1;
        control_words_u8[1][i] = part_2;
        control_words_u8[2][i] = part_3;
        control_words_u8[3][i] = part_4;
        control_words_u8[4][i] = part_5;
        control_words_u8[5][i] = part_6;
        control_words_u8[6][i] = part_7;
        control_words_u8[7][i] = part_8;
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
