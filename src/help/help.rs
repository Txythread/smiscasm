use termimad;
const HELP_STRING: &str = include_str!("help.md");

pub fn print_help() {
    println!("{}", termimad::inline(HELP_STRING));
}