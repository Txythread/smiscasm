use std::process::exit;
use colorize::*;

#[allow(dead_code)]
pub fn tokenize(input: Vec<String>) -> Vec<Vec<String>> {
    // The remaining spaces always start a new token, but are themselves to be ignored.
    // '.',  '@',  ':',  '0x',  '0b', '0o'  '[',  ']',  '(',  ')',  ',',  '<',  '>',  '+',  '-',  '*',  '/',  '&'  &  '%'
    // also separate strings into tokens (except when in string/char literals), but also serve as tokens themselves.

    let mut all_tokens: Vec<Vec<String>> = vec![];
    let mut current_line_tokens: Vec<String> = vec![];

    // Having more than two characters is not allowed
    let token_markers = [".".to_string(), "@".to_string(), ":".to_string(), "0x".to_string(), "0b".to_string(), "0o".to_string(), "[".to_string(), "]".to_string(), "(".to_string(), ")".to_string(), "<".to_string(), ">".to_string(), ",".to_string(), "+".to_string(), "-".to_string(), "*".to_string(), "/".to_string(), "%".to_string(), "&".to_string()];

    for line in input.iter().enumerate() {
        let line_number = line.0;
        let line = line.1.clone();

        let mut last_character: char = 'x';
        let mut current_token: String = String::new();
        let mut in_string_literal = false;

        for i in line.chars().enumerate() {
            let char = i.1.clone();

            if !in_string_literal {
                // Check for whitespaces, especially \t and ' '
                if char == ' ' || char == '\t' {
                    if current_token.is_empty()/*current_token == ""*/ {
                        //current_line_tokens.push(current_token.clone());
                        current_token = String::new();
                    } else {
                        current_line_tokens.push(current_token.clone());
                        current_token = String::new();
                    }
                    last_character = char;
                    continue;
                }

                // Check for single-character token markers
                if token_markers.contains(&char.to_string()) {
                    if !current_token.is_empty() {
                        current_line_tokens.push(current_token.clone());
                        current_token = String::new();
                    }
                    current_line_tokens.push(char.to_string());
                    last_character = char;
                    continue;
                }

                // Check for double-character token markers
                if token_markers.contains(&(last_character.to_string() + char.to_string().as_str())) {
                    if !current_token.is_empty() {
                        // Remove the duplicate char
                        current_token.remove(current_token.len() - 1);
                        if !current_token.is_empty() {
                            current_line_tokens.push(current_token.clone());
                            current_token = String::new();
                        }
                    }
                    current_line_tokens.push(last_character.to_string() + char.to_string().as_str());
                    last_character = char;
                    continue;
                }
            }

            // Check if a string literal starts/ends
            if (char == '\"' || char == '\'') && last_character != '\\' {
                if in_string_literal {
                    // Check the literal is valid
                    if current_token.is_empty() {
                        let error = format!("String or character literal at line {} is empty", line_number).red().to_string();
                        eprintln!("{}", error);
                        exit(105);
                    }
                    // Add the current token & the ''' or the '"'.
                    current_line_tokens.push(current_token.clone());
                    current_token = String::new();
                    current_line_tokens.push(char.to_string());
                } else {
                    if !current_token.is_empty() {
                        current_line_tokens.push(current_token.clone());
                        current_token = String::new();
                    }
                    current_line_tokens.push(char.to_string());
                }

                in_string_literal = !in_string_literal;
                last_character = char;
                continue;
            }

            // Since it is neither a whitespace nor a special character, just add it to the current token.
            current_token.push(char);
            last_character = char;
        }

        if in_string_literal {
            let error = format!("String or character literal at line {} lacks trailing.", line_number).red().to_string();
            eprintln!("{}", error);
            exit(105);
        }

        if !current_token.is_empty() {
            current_line_tokens.push(current_token.clone());
        }

        if !current_line_tokens.is_empty() {
            all_tokens.push(current_line_tokens.clone());
            current_line_tokens = Vec::new();
        }
    }

    all_tokens
}

#[cfg(test)]
mod tests {
    use crate::assembler::tokenizer::tokenize;

    #[test]
    fn test_tokenize() {
        let input = vec![
            ".section \"CODE\"",
            "main:",
            "adrp x0, msg@PAGE",
            "add x0, msg@PAGEOFF",
            "adrp x1, msg_end@PAGE",
            "add x1, msg_end@PAGEOFF",
            "sub x1, 0x1",
            "loop:",
            "lb x2, x0",
            "add x0, 1",
            "out x2",
            "mov x0, x3",
            "sub x3, x1",
            "jmpz x3, end",
            "jmp loop",
            "end:",
            "hlt",
            ".section \"DATA\"",
            "msg:",
            ".ascii \"Hello, world!\"",
            ".msg_end [$ - 1]",
        ];

        let mut input2: Vec<String> = vec![];

        for i in input{
            input2.push(i.to_string());
        }

        let input = input2;

        let expected = vec![
            vec![".", "section",  "\"", "CODE", "\""],
            vec!["main", ":"],
            vec!["adrp", "x0", ",", "msg", "@", "PAGE"],
            vec!["add", "x0", ",", "msg", "@", "PAGEOFF"],
            vec!["adrp", "x1", ",", "msg_end", "@", "PAGE"],
            vec!["add", "x1", ",", "msg_end", "@", "PAGEOFF"],
            vec!["sub",  "x1", ",", "0x", "1"],
            vec!["loop", ":"],
            vec!["lb",  "x2", ",", "x0"],
            vec!["add", "x0", ",", "1"],
            vec!["out", "x2"],
            vec!["mov", "x0", ",", "x3"],
            vec!["sub", "x3", ",", "x1"],
            vec!["jmpz", "x3", ",", "end"],
            vec!["jmp", "loop"],
            vec!["end", ":"],
            vec!["hlt"],
            vec![".", "section", "\"", "DATA", "\""],
            vec!["msg", ":"],
            vec![".", "ascii", "\"", "Hello, world!", "\""],
            vec![".", "msg_end", "[", "$", "-", "1", "]"]
        ];

        let result = tokenize(input);

        for i in 0..result.len(){
            for j in 0..expected[i].len(){
                assert_eq!(expected[i][j], result[i][j]);
            }
        }
    }
}