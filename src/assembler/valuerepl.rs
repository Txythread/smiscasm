use crate::assembler::valuegen::{Section, ValueGenResult};
use crate::util::code_error::{display_code_error, ErrorNotificationKind};
use crate::util::line_mapping::LineMap;
use crate::util::math::resolve_argument;
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
    result.sections = code.sections.iter().clone().map(|x|x.clone()).collect();
    result.line_mapping = code.line_mapping.clone();

    for i in code.code.iter().enumerate() {
        let line = i.1.clone();
        let line_number = i.0;

        let first_token = line.first();

        if first_token.is_none() { continue; }

        let first_token = first_token.unwrap();

        let source_file_name = input_line_mapping.lines[line_number].clone().source_file_name;

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
            // Add the new token to the line mapping.
            let start_pos = input_line_mapping.lines[line_number].token_info[arg_start_pos_in_tokens as usize].0;
            let end_token = input_line_mapping.lines[line_number].token_info[arg_start_pos_in_tokens as usize + arg.len() - 1];
            let end_pos = end_token.0 + end_token.1;
            let in_between_tokens_length = end_pos - start_pos;
            new_tokens.push((start_pos, in_between_tokens_length));



            let argument_string = arg.join("");

            let mut all_constants = code.constants.clone();
            let mut line_specific_constants = input_line_mapping.lines[line_number].attributes.line_specific_constants.clone();
            all_constants.append(&mut line_specific_constants);

            let math_solution = resolve_argument(argument_string.clone(), all_constants, code.sections.iter().clone().map(|x|x.clone()).collect());

            if math_solution != ""{
                final_args.push(math_solution.clone());

                continue;
            }


            // Attempt decoding as a register
            let arg_0 = arg[0].clone();
            let arg_c1 = arg_0.clone().chars().nth(0).unwrap();
            let arg_except_first_char = arg_0[1..].to_string();

            // Check if it is an immediate value or a register
            if arg_0.parse::<i32>().is_ok() || (arg_c1 == 'x' && arg_except_first_char.parse::<i32>().is_ok()){
                // Just a normal value, nothing to replace
                final_args.push(arg_0);

                arg_start_pos_in_tokens += arg.len() as u32 + 1; // Add the amount of tokens plus one for the comma
                continue;
            }

            // None of the above decoding attempts succeeded, throw an error
            let mut code: Vec<String> = Vec::with_capacity(line.len());
            let real_line_number = input_line_mapping.lines[line_number].line_number;

            for _ in 0..real_line_number {
                code.push(String::new());
            }

            code.push(input_line_mapping.lines[line_number].contents.clone());


            display_code_error(ErrorNotificationKind::Error, real_line_number as i32, Some(start_pos), Some(in_between_tokens_length), "Argument Decoding Error".to_string(), "Can't decode this argument. Check whether all variables exist if applicable.".to_string(), code, source_file_name.clone());
            input_line_mapping.stop_after_step = true;
            input_line_mapping.errors_count += 1;
        }

        // Add the instruction back to the top of the chain
        final_args.insert(0, first_token.to_string());

        // Add to the output line map, but modify the token positions
        let mut original_line = input_line_mapping.lines[line_number].clone();
        original_line.token_info = new_tokens;

        output_line_mapping.add_line(original_line);

        result.code.push((final_args, LineKind::Code(false)));
    }


    output_line_mapping.warnings_count = input_line_mapping.warnings_count;
    output_line_mapping.errors_count = input_line_mapping.errors_count;
    output_line_mapping.stop_after_step = input_line_mapping.stop_after_step;


    output_line_mapping.exit_if_needed();

    (result, output_line_mapping)
}

#[derive(Debug, Clone)]
pub struct ValueReplResult{
    pub global_constants: Vec<Replacement>,
    pub sections: Vec<Section>,           // Name of the section followed by the correct line (starting at 0) from the resulting code.
    pub code: Vec<(Vec<String>, LineKind)>,     // The lines of code and if they contain immediate values encoded in global constants.
    pub line_mapping: Vec<(usize, usize)>,      // How the new line number in the resulting code above (.0) refers to the original line number (.1)
}

#[derive(Debug, Clone, PartialEq)]
pub enum LineKind {
    Code(bool),     // bool: immediate value?
    ASCII,
    STC
}



#[cfg(test)]
mod tests {
    use crate::assembler::valuegen::{Section, ValueGenResult};
    use crate::assembler::valuerepl::{replace_values_in_code, LineKind};
    use crate::util::line_mapping::LineMap;
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

        let input_sections : Vec<Section>= vec![
            Section { name: "CODE".to_string(), start_offset: 0, start_memory_page: 0, start_pos_bytes_original: 0 },
            Section { name: "DATA".to_string(), start_offset: 0, start_memory_page: 1, start_pos_bytes_original: 1 },
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

        let line_map = LineMap::test_map();


        let result = replace_values_in_code(input, line_map);
        let result = result.0;

        assert_eq!(result.code.len(), expected_output_code.len());

        for i in 0..result.code.len() {
            let expected = expected_output_code[i].clone();
            let actual = result.code[i].clone();

            assert_eq!(expected, actual);
        }


    }
}