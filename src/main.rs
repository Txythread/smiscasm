use std::env;
use std::fmt::{Debug, Formatter};
use std::process::exit;
use std::fs;
use std::path::PathBuf;
use colorize;
use colorize::AnsiColor;

pub struct ArgumentList{
    file: Option<String>,
    test: bool,                     // -t or --test
    output_name: Option<String>,    // -o or --output
}

impl ArgumentList{
    pub fn new() -> ArgumentList{
        ArgumentList{file: None, test: false, output_name: None}
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
    // Retrieve arguments from the terminal first
    let cli_args: Vec<String> = env::args().collect();

    // Generate a reasonable argument list
    let args = get_arguments_from_list(cli_args);

    // Load the file
    let path = expand_path(&args.file.unwrap()).unwrap();
    let input_file = fs::read_to_string(path).unwrap();


    println!("{}", input_file);
}

fn expand_path(path_str: &str) -> Option<PathBuf> {
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