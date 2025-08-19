use termimad;
const HELP_STRING: &str = include_str!("help.md");

pub fn print_help() {
    // Termimad is for Markdown formatting in the terminal.
    println!("{}", termimad::inline(HELP_STRING));
}