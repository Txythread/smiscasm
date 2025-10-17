use crate::util::code_error::{display_code_error, ErrorNotificationKind};
use crate::util::line_mapping::LineMap;

/// Split lines into tokens
pub fn split(input: Vec<String>, mut input_line_map: LineMap) -> (Vec<Vec<String>>, LineMap) {
    let mut output_line_map: LineMap = LineMap::new();

    // The remaining spaces always start a new token, but are themselves to be ignored.
    // '.',  '@',  ':',  '0x',  '0b', '0o'  '[',  ']',  '(',  ')',  ',',  '<',  '>',  '+',   '*',  '/',  '&'  &  '%'
    // also separate strings into tokens (except when in string/char literals), but also serve as tokens themselves.

    let mut all_tokens: Vec<Vec<String>> = vec![];
    let mut current_line_tokens: Vec<String> = vec![];
    let mut current_line_token_map: Vec<(u32, u32)> = vec![];

    // Having more than two characters is not allowed
    let token_markers = [".".to_string(), "@".to_string(), ":".to_string(), "0x".to_string(), "0b".to_string(), "0o".to_string(), "[".to_string(), "]".to_string(), "(".to_string(), ")".to_string(), "<".to_string(), ">".to_string(), ",".to_string(), "+".to_string(), "*".to_string(), "/".to_string(), "%".to_string(), "&".to_string()];


    for line in input.iter().enumerate() {
        let line_number = line.0;
        let line = line.1.clone();

        let mut last_character: char = 'x';
        let mut current_token: String = String::new();
        let mut in_string_literal = false;
        let mut current_token_start = 0;

        let mut current_char_count = -1i32;

        let file_name= input_line_map.lines[line_number].clone().source_file_name;

        for i in line.chars().enumerate() {
            let char = i.1.clone();
            current_char_count += 1;

            if !in_string_literal {
                // Check for whitespaces, especially \t and ' '
                if char == ' ' || char == '\t' {
                    if current_token.is_empty() {
                        //current_line_tokens.push(current_token.clone());
                        current_token = String::new();
                        current_token_start = current_char_count + 1;
                    } else {
                        current_line_token_map.push((current_token_start as u32, current_token.len() as u32));
                        current_token_start = current_char_count + 1;

                        current_line_tokens.push(current_token.clone());
                        current_token = String::new();
                    }
                    last_character = char;
                    continue;
                }

                // Check for single-character token markers
                if token_markers.contains(&char.to_string()) {
                    if !current_token.is_empty() {
                        // Replace internal constants
                        match current_token.as_str(){
                            "sp" => { current_token = "x31".to_string() },
                            _ => { /* Nothing to change */}
                        }
                        current_line_token_map.push((current_token_start as u32, current_token.len() as u32));
                        current_token_start = current_char_count;

                        current_line_tokens.push(current_token.clone());
                        current_token = String::new();
                    }
                    current_line_token_map.push((current_token_start as u32, 1u32));
                    current_token_start = current_char_count + 1;

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
                            current_line_token_map.push((current_token_start as u32, current_token.len() as u32));
                            current_token_start = current_char_count;

                            current_line_tokens.push(current_token.clone());
                            current_token = String::new();
                        }
                    }
                    current_line_token_map.push((current_token_start as u32, 2u32));
                    current_token_start = current_char_count + 1;

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
                        let mut code = vec![];

                        let real_line_number = input_line_map.lines[line_number].line_number;

                        for _ in 0..real_line_number {
                            code.push("".to_string());
                        }

                        code.push(line.clone());

                        display_code_error(ErrorNotificationKind::Error, real_line_number as i32, Some((current_token_start - 1) as u32), Some((current_token.len() + 2) as u32), "Empty String Literal".to_string(), "Empty string literals are not allowed, but an empty string literal was found here.".to_string(), code, file_name.clone());
                        input_line_map.errors_count += 1;
                        output_line_map.stop_after_step = true;
                    }
                    // Add the current token & the ''' or the '"'.
                    current_line_token_map.push((current_token_start as u32, current_token.len() as u32));
                    current_token_start = current_char_count;

                    current_line_tokens.push(current_token.clone());
                    current_token = String::new();


                    current_line_token_map.push((current_token_start as u32, 1u32));
                    current_token_start = current_char_count;

                    current_line_tokens.push(char.to_string());
                } else {
                    if !current_token.is_empty() {
                        current_line_token_map.push((current_token_start as u32, current_token.len() as u32));
                        current_token_start = current_char_count;

                        current_line_tokens.push(current_token.clone());
                        current_token = String::new();
                    }
                    current_line_token_map.push((current_token_start as u32, 1u32));
                    current_token_start = current_char_count + 1;

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
            let real_line_number = input_line_map.lines[line_number].line_number;

            let mut code: Vec<String> = vec![];

            for _ in 0..real_line_number {
                code.push("".to_string());
            }

            code.push(line.clone());


            display_code_error(ErrorNotificationKind::Error, real_line_number as i32, Some((current_token_start - 1) as u32), Some((current_char_count - current_token_start + 2) as u32), "Unterminated String Literal".to_string(), "String literals always need to be terminated, but this one wasn't closed.\nAdd the missing \".".to_string(), code, file_name);
            input_line_map.errors_count += 1;
            output_line_map.stop_after_step = true;
        }

        if !current_token.is_empty() {
            current_line_token_map.push((current_token_start as u32, current_token.len() as u32));
            current_token_start = current_char_count + 1;

            current_line_tokens.push(current_token.clone());
        }

        if !current_line_tokens.is_empty() {
            // Handle line in input map
            let mut line_in_input_map = input_line_map.lines[line_number].clone();

            // Add the token mapping
            line_in_input_map.token_info = current_line_token_map.clone();
            current_line_token_map = vec![];

            output_line_map.add_line(line_in_input_map);

            all_tokens.push(current_line_tokens.clone());
            current_line_tokens = Vec::new();
        }
    }


    output_line_map.warnings_count = input_line_map.warnings_count;
    output_line_map.errors_count = input_line_map.errors_count;


    output_line_map.exit_if_needed();

    (all_tokens, output_line_map)
}

#[cfg(test)]
mod tests {
    use crate::assembler::splitter::split;
    use crate::util::line_mapping::LineMap;

    #[test]
    fn test_splitting() {
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


        let line_map = LineMap::test_map();


        let result = split(input, line_map).0;

        for i in 0..result.len(){
            for j in 0..expected[i].len(){
                assert_eq!(expected[i][j], result[i][j]);
            }
        }
    }
}