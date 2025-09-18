// This is a help file for finding the correct line in the original file
// from another file in a later phase of assembly.

use std::collections::HashMap;
use crate::util::code_error::{display_code_error, ErrorNotificationKind};

pub struct LineMap{
    lines: Vec<LineInfo>,       // The lines that currently exist. Current & Real lines are not one-to-one.
    stop_after_step: bool,      // Stop after the current step has been executed, but finish this one.
    warnings_count: usize,      // The amount of warnings
    pub errors_count: usize,        // The amount of errors. If > 0, compilation will not succeed.
}

impl LineMap{
    pub fn new() -> LineMap{
        LineMap{lines: Vec::new(), stop_after_step: false, warnings_count: 0, errors_count: 0 }
    }

    /// Add a line in the next position
    pub fn add_line(&mut self, line: LineInfo){
        self.lines.push(line);
    }

    /// Print an error/warning for a line.
    pub fn print_notification(&mut self, kind: ErrorNotificationKind, line_number_in_current: u32, token_number: Option<u32>, title: String, message: String) {
        // Update the counter
        match kind {
            ErrorNotificationKind::Warning => {
                self.warnings_count += 1;
            },
            ErrorNotificationKind::Error => {
                self.errors_count += 1;
                println!("increased errors");
            }
        }



        let line_info = self.lines[line_number_in_current as usize].clone();

        let mut code: Vec<String> = vec![];

        // Create newlines except for the last (current) line

        for _ in 0..line_info.line_number{
            code.push(String::new());
        }

        code.push(line_info.contents);

        if let Some(token_number) = token_number{
            if let Some(token) = line_info.token_info.iter().nth(token_number as usize){
                let token = token.clone();

                display_code_error(kind, line_info.line_number as i32, Some(token.0), Some(token.1), title, message, code);
                return;
            }
        }

        display_code_error(kind.clone(), line_info.line_number as i32, None, None, title, message, code);
    }
}

#[derive(Clone)]
pub struct LineInfo {
    contents: String,           // The contents (without leading & trailing whitespaces)
    indent: u32,                // The indent (in spaces) this line has (for formatting)
    token_info: Vec<(u32, u32)>,// The start of a token and its length
    line_number: u32,           // The original line number
}

impl LineInfo{
    pub fn new(contents: String, indent: u32, token_info: Vec<(u32, u32)>, line_number: u32) -> Self{
        LineInfo { contents, indent, token_info, line_number }
    }

    /// Generate a new LineInfo with text only, without any info about tokens.
    pub fn new_no_info(line: String, line_number: u32) -> Self{
        let mut line_whitespace_length = 0u32;

        for i in 0..line.len(){
            match line.clone().chars().nth(i).unwrap() {
                ' ' => line_whitespace_length += 1,
                '\t' => line_whitespace_length += 4,
                _ => break,
            }
        }

        let contents = line.trim().to_string();

        LineInfo { contents, indent: line_whitespace_length, token_info: vec![], line_number }
    }
}