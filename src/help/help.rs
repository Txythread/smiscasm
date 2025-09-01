// Termimad is for Markdown formatting in the terminal.
use termimad;
use include_dir::{include_dir, Dir};
use crate::ArgumentList;
use crate::util::exit::{exit, ExitCode};

const HELP_STRING: &str = include_str!("help.md");
const COMMAND_HELP_FILES: Dir = include_dir!("$CARGO_MANIFEST_DIR/src/help/commands");
const INSTRUCTION_HELP_FILES: Dir = include_dir!("$CARGO_MANIFEST_DIR/src/help/instructions");


pub fn print_help(arguments: ArgumentList) {
    if arguments.generate_instruction_table{
        print_help_file("generate-instruction-table".to_string());
        return;
    }

    if arguments.get_micro_operation.is_some() {
        print_help_file("get-micro-operation".to_string());
        return;
    }

    if arguments.instruction_help.is_some() {
        print_help_file("instruction-help".to_string());
        return;
    }

    if arguments.output_name.is_some() {
        print_help_file("output".to_string());
    }

    println!("{}", termimad::inline(HELP_STRING));
}

pub fn print_instruction_help(named: String){
    if let Some(contents) = INSTRUCTION_HELP_FILES.get_file(named.clone() + ".md"){
        println!("{}", termimad::inline(contents.contents_utf8().unwrap()));
    }else{
        exit(format!("There is no help file for an instruction named {}.", named), ExitCode::BadArgument);
    }
}

fn print_help_file(name: String) {
    let contents = COMMAND_HELP_FILES.get_file(format!("{}.md", name));

    if let Some(contents) = contents {
        println!("{}", termimad::inline(contents.contents_utf8().unwrap()));
    }else{
        exit(format!("Tried to open help file commands/{}, but failed.", name), ExitCode::Internal);
    }
}