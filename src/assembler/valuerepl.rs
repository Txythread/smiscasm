use crate::assembler::valuegen::ValueGenResult;
use crate::util::code_error::ErrorNotificationKind;
use crate::util::exit::{exit, exit_with_variant, ExitCode};
use crate::util::line_mapping::LineMap;
use crate::util::replacement::Replacement;

/// Replace global constants in code
pub fn replace_values_in_code(code: ValueGenResult, mut input_line_mapping: LineMap) -> (ValueReplResult, LineMap) {
    let mut result: ValueReplResult = ValueReplResult {
        global_constants: vec![],
        sections: vec![],
        code: vec![],
        line_mapping: vec![],
    };

    // Generate the new line mapping for the output
    let mut output_line_mapping: LineMap = LineMap::new();

    result.global_constants = code.constants.clone();
    result.sections = code.sections.clone();
    result.line_mapping = code.line_mapping.clone();

    for i in code.code.iter().enumerate() {
        let line = i.1.clone();
        let line_number = i.0;

        let first_token = line.first();

        if first_token.is_none() { continue; }

        let first_token = first_token.unwrap();

        if first_token == "." {
            // At this stage, the only thing it could be is data.
            let second_token = line[1].clone();

            match second_token.as_str() {
                "ascii" => {
                    let text = line[2].clone();

                    if !text.is_ascii() {
                        input_line_mapping.print_notification(ErrorNotificationKind::Error, line_number as u32, Some(2), "Illegal Character in ASCII".to_string(), "There are illegal characters in this string. Those characters can't be passed as ASCII".to_string());
                        input_line_mapping.stop_after_step = true;
                        continue;
                    }

                    result.code.push((vec![text], LineKind::ASCII));

                    // Generate the new line mapping
                    let mut input_line_info = input_line_mapping.lines[line_number].clone();

                    // Though the tokens technically remain the same in the original line,
                    // the string should appear as a single token for highlighting.
                    // This is the same as taking the pos of the string.
                    let text_info = input_line_info.token_info[2];
                    input_line_info.token_info = vec![text_info];

                    output_line_mapping.add_line(input_line_info);

                    continue;
                }

                "stc" => {
                    let text = line[2].clone();

                    if !text.is_ascii() {
                        input_line_mapping.print_notification(ErrorNotificationKind::Error, line_number as u32, Some(2), "Illegal Character in STC".to_string(), "There are illegal characters in this string.\nThose characters can't be passed as STC.\n(Note: The string is only tested for non-ASCII characters.)".to_string());
                        input_line_mapping.stop_after_step = true;
                        continue;
                    }

                    result.code.push((vec![text], LineKind::STC));

                    // Generate the new line mapping
                    let mut input_line_info = input_line_mapping.lines[line_number].clone();

                    // Though the tokens technically remain the same in the original line,
                    // the string should appear as a single token for highlighting.
                    // This is the same as taking the pos of the string adding one to the end
                    // and subtracting one from the start for the quotation marks.
                    let mut text_info = input_line_info.token_info[2];
                    text_info.0 -= 1;
                    text_info.1 += 1;
                    input_line_info.token_info = vec![text_info];

                    output_line_mapping.add_line(input_line_info);

                    continue;
                }

                _ => {
                    input_line_mapping.print_notification(ErrorNotificationKind::Error, line_number as u32, None, "Unknown Assembler Instruction".to_string(), format!("No such assembler instruction: \"{}\".\nNote: This is most likely an internal error with data likely caused by \"valuegen\".", second_token));
                    continue;
                }
            }
        }

        // It's an instruction, not data

        // Split the arguments
        let mut args: Vec<Vec<String>> = vec![];
        let mut current_argument: Vec<String> = vec![];
        for i in 1..line.len() {
            let token = line[i].clone();

            if token == "," {
                args.push(current_argument.clone());
                current_argument = vec![];

                continue;
            }

            current_argument.push(token);
        }

        if !current_argument.is_empty() {
            args.push(current_argument.clone());
        }

        // Go through every argument and hold its final version until the value is immediate or refers to a global constant
        let mut final_args: Vec<String> = vec![];
        let mut arg_start_pos_in_tokens = 1u32;

        // The start and stop positions of the new tokens (instead of msg@PAGEOFF being three separate tokens, they should be only one in the end).
        let mut new_tokens: Vec<(u32, u32)> = vec![input_line_mapping.lines[line_number].token_info[0]]; // Initialize with position of instruction
        for arg in args {
            // Check if it's complex
            if arg.len() == 3{
                // This is most likely some-constant@PAGE(OFF)

                if arg[1] != "@" {
                    // This is supposed to contain an @ (format as commented above), but it doesn't. Or at least not at the second token.
                    input_line_mapping.print_notification(ErrorNotificationKind::Error, line_number as u32, Some(arg_start_pos_in_tokens + 1), "Argument Formatting Error".to_string(), "Couldn't infer the meaning of this argument.".to_string());
                    arg_start_pos_in_tokens += arg.len() as u32 + 1; // Add the amount of tokens plus one for the comma
                    input_line_mapping.stop_after_step = true;
                    continue;
                }

                // The constant whose value will be either addressed with PAGE or PAGEOFF
                let arg0_value = code.constants.iter().find(| &x | x.clone().get_name() == arg[0].as_str());

                if arg0_value.is_none() {
                    // The constant doesn't exist, probably a user misspelling.
                    input_line_mapping.print_notification(ErrorNotificationKind::Error, line_number as u32, Some(arg_start_pos_in_tokens), "Undefined constant".to_string(), "There is no declaration for this constant.".to_string());
                    arg_start_pos_in_tokens += arg.len() as u32 + 1; // Add the amount of tokens plus one for the comma
                    input_line_mapping.stop_after_step = true;
                    continue;
                }

                let arg0_value = arg0_value.unwrap().clone().get_value();

                // Split as the value should be formated like "CODE:0"
                let mut arg0_value_split = arg0_value.split(':');

                if arg0_value_split.clone().count() != 2{
                    // Expected an address before a modifier like @PAGE or @PAGEOFF, but another type of value was found.
                    input_line_mapping.print_notification(ErrorNotificationKind::Error, line_number as u32, Some(arg_start_pos_in_tokens), "Constant Should Contain Address".to_string(), "The constant was expected to contain an address, but another type of constant was found.".to_string());
                    arg_start_pos_in_tokens += arg.len() as u32 + 1; // Add the amount of tokens plus one for the comma
                    input_line_mapping.stop_after_step = true;
                    continue;
                }

                let arg0_page = arg0_value_split.next().unwrap();
                let arg0_pageoff = arg0_value_split.next().unwrap();

                match arg[2].as_str() {
                    "PAGE" => {
                        // Calculate the correct memory page
                        let section_idx = code.sections.clone().iter().enumerate().find(| &x | x.1.clone().0 == arg0_page).map(| x | x.0);

                        if section_idx.is_none() {
                            // This is strange as this should've been generated by valuegen correctly, but it might still be an error in the assembly code.
                            input_line_mapping.print_notification(ErrorNotificationKind::Error, line_number as u32, Some(arg_start_pos_in_tokens), "No Such Section".to_string(), format!("There is no section named\"{}\".\nThis might be an internal issue. In this case: sorry.", arg0_page));
                            arg_start_pos_in_tokens += arg.len() as u32 + 1; // Add the amount of tokens plus one for the comma
                            input_line_mapping.stop_after_step = true;
                            continue;
                        }

                        let section_idx = section_idx.unwrap().clone();
                        let memory_page_start = section_idx/* * MEMORY_PAGE_SIZE*/;

                        final_args.push(memory_page_start.to_string());

                        // Add the new token to the line mapping.
                        let start_pos = input_line_mapping.lines[line_number].token_info[arg_start_pos_in_tokens as usize].0;
                        let end_token = input_line_mapping.lines[line_number].token_info[arg_start_pos_in_tokens as usize + arg.len() - 1];
                        let end_pos = end_token.0 + end_token.1;
                        let in_between_tokens_length = end_pos - start_pos;
                        new_tokens.push((start_pos, in_between_tokens_length));
                    }

                    "PAGEOFF" => {
                        final_args.push(arg0_pageoff.to_string());

                        // Add the new token to the line mapping.
                        // Add the new token to the line mapping.
                        let start_pos = input_line_mapping.lines[line_number].token_info[arg_start_pos_in_tokens as usize].0;
                        let end_token = input_line_mapping.lines[line_number].token_info[arg_start_pos_in_tokens as usize + arg.len() - 1];
                        let end_pos = end_token.0 + end_token.1;
                        let in_between_tokens_length = end_pos - start_pos;
                        new_tokens.push((start_pos, in_between_tokens_length));
                    }

                    _ => {
                        input_line_mapping.print_notification(ErrorNotificationKind::Error, line_number as u32, Some(arg_start_pos_in_tokens + 2), "Unknown Modifier".to_string(), "There is no such modifier.".to_string());
                        arg_start_pos_in_tokens += arg.len() as u32 + 1; // Add the amount of tokens plus one for the comma
                        input_line_mapping.stop_after_step = true;
                        continue;
                    }
                }
                continue;
            }

            // If not, just replace values normally
            if arg.len() != 1{
                input_line_mapping.print_notification_multiple_faulty_tokens(ErrorNotificationKind::Error, line_number as u32, arg_start_pos_in_tokens, arg_start_pos_in_tokens + arg.len() as u32, "Faulty Argument Format".to_string(), "Couldn't infer the meaning of this argument.".to_string());
                arg_start_pos_in_tokens += arg.len() as u32 + 1; // Add the amount of tokens plus one for the comma
                input_line_mapping.stop_after_step = true;
                continue;
            }

            let arg_0 = arg[0].clone();
            let arg_c1 = arg_0.clone().chars().nth(0).unwrap();
            let arg_except_first_char = arg_0[1..].to_string();

            // Check if it is an immediate value or a register
            if arg_0.parse::<i32>().is_ok() || (arg_c1 == 'x' && arg_except_first_char.parse::<i32>().is_ok()){
                // Just a normal value, nothing to replace
                final_args.push(arg_0);

                // Add the new token to the line mapping.
                let start_pos = input_line_mapping.lines[line_number].token_info[arg_start_pos_in_tokens as usize].0;
                let end_token = input_line_mapping.lines[line_number].token_info[arg_start_pos_in_tokens as usize + arg.len() - 1];
                let end_pos = end_token.0 + end_token.1;
                let in_between_tokens_length = end_pos - start_pos;
                new_tokens.push((start_pos, in_between_tokens_length));

                arg_start_pos_in_tokens += arg.len() as u32 + 1; // Add the amount of tokens plus one for the comma
                continue;
            }

            // Find a replacement that matches the name of the constant
            let replacement = code.constants.iter().find(| &x | x.get_name() == arg_0);

            if replacement.is_none() {
                // Look if it's in a known list
                match arg_0.as_str() {
                    "sp" => break,
                    _ => {
                        // The user probably misspelled.
                        input_line_mapping.print_notification_multiple_faulty_tokens(ErrorNotificationKind::Error, line_number as u32, arg_start_pos_in_tokens, arg_start_pos_in_tokens + arg.len() as u32, "Invalid Argument".to_string(), "This argument is neither an immediate value nor is it a constant.".to_string());
                        arg_start_pos_in_tokens += arg.len() as u32 + 1; // Add the amount of tokens plus one for the comma
                        input_line_mapping.stop_after_step = true;
                        continue;
                    }
                }
            }

            let replacement = replacement.unwrap().clone();

            // Add the new token to the line mapping.
            let start_pos = input_line_mapping.lines[line_number].token_info[arg_start_pos_in_tokens as usize].0;
            let end_token = input_line_mapping.lines[line_number].token_info[arg_start_pos_in_tokens as usize + arg.len() - 1];
            let end_pos = end_token.0 + end_token.1;
            let in_between_tokens_length = end_pos - start_pos;
            new_tokens.push((start_pos, in_between_tokens_length));

            if replacement.get_is_global(){
                final_args.push("@".to_owned() + arg_0.clone().as_str()); // Has to be changed in a later stage
            } else {
                final_args.push(replacement.get_value());
            }
        }

        // Add the instruction back to the top of the chain
        final_args.insert(0, first_token.to_string());

        // Add to the output line map, but modify the token positions
        let mut original_line = input_line_mapping.lines[line_number].clone();
        original_line.token_info = new_tokens;

        output_line_mapping.add_line(original_line);

        result.code.push((final_args, LineKind::Code(false)));
    }

    // TODO: Delete this fucking loop after the update of ya_tokenizer
    for _ in 0..100{
        result.line_mapping.push((0, 0));
    }


    output_line_mapping.warnings_count = input_line_mapping.warnings_count;
    output_line_mapping.errors_count = input_line_mapping.errors_count;

    (result, output_line_mapping)
}

pub struct ValueReplResult{
    pub global_constants: Vec<Replacement>,
    pub sections: Vec<(String, u32)>,           // Name of the section followed by the correct line (starting at 0) from the resulting code.
    pub code: Vec<(Vec<String>, LineKind)>,     // The lines of code and if they contain immediate values encoded in global constants.
    pub line_mapping: Vec<(usize, usize)>,      // How the new line number in the resulting code above (.0) refers to the original line number (.1)
}

#[derive(Debug, Clone, PartialEq)]
pub enum LineKind {
    Code(bool),     // bool: immediate value?
    ASCII,
    STC
}



/*
#[cfg(test)]
mod tests {
    use crate::assembler::valuegen::ValueGenResult;
    use crate::assembler::valuerepl::{replace_values_in_code, LineKind};
    use crate::util::line_mapping::{LineInfo, LineMap};
    use crate::util::replacement::Replacement;

    #[test]
    fn test_replace_values_in_code(){
        let mut input_constants = vec![
            Replacement::new("xyz".to_string(), "10".to_string(), false),
            Replacement::new("zyx".to_string(), "16".to_string(), false),
            Replacement::new("abc".to_string(), "20".to_string(), false),
            Replacement::new("main".to_string(), "CODE:0".to_string(), true),
            Replacement::new("msg".to_string(), "DATA:0".to_string(), true),
        ];

        input_constants[0].set_is_global(true);
        input_constants[3].set_is_global(true);

        let input_sections : Vec<(String, u32)>= vec![
            ("CODE".to_string(), 0),
            ("DATA".to_string(), 1),
        ];

        let input_code = vec![
            vec!["adrp".to_string(), "x0".to_string(), ",".to_string(), "msg".to_string(), "@".to_string(), "PAGE".to_string()],
            vec!["add".to_string(), "x0".to_string(), ",".to_string(), "msg".to_string(), "@".to_string(), "PAGEOFF".to_string()],
            vec![".".to_string(), "ascii".to_string(), "Hi".to_string()],
        ];

        let expected_output_code: Vec<(Vec<String>, LineKind)> = vec![
            (vec!["adrp".to_string(), "x0".to_string(), "1".to_string()], LineKind::Code(false)),
            (vec!["add".to_string(), "x0".to_string(), "0".to_string()], LineKind::Code(false)),
            (vec!["Hi".to_string()], LineKind::ASCII)
        ];



        let line_mapping_input = vec![(0, 1), (1, 1), (2, 2)];

        let input = ValueGenResult {
            constants: input_constants,
            sections: input_sections,
            code: input_code,
            line_mapping: line_mapping_input,
        };

        let mut line_map = LineMap::new();

        for i in 0..5{
            line_map.add_line(LineInfo::new("as as as as".to_string(), 0, vec![(0, 0), (0, 0), (0, 0), (0, 0), (0, 0), (0, 0), (0, 0), (0, 0), (0, 0), (0, 0), ], i))
        }


        let result = replace_values_in_code(input, line_map);

        assert_eq!(result.code.len(), expected_output_code.len());

        for i in 0..result.code.len() {
            let expected = expected_output_code[i].clone();
            let actual = result.code[i].clone();

            assert_eq!(expected, actual);
        }


    }
}*/