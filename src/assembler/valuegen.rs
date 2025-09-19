use std::i32;
use crate::util::code_error::ErrorNotificationKind;
use crate::util::replacement::Replacement;
use crate::util::math::resolve_string;
use crate::util::exit::{exit, exit_with_variant, ExitCode};
use crate::util::line_mapping::LineMap;

/// Find global constant declarations and labels (function definitions) in code and separate them.
pub fn gen_values(code: Vec<Vec<String>>, input_line_map: LineMap) -> (ValueGenResult, LineMap){
    let mut result = ValueGenResult{ constants: vec![], sections: vec![], code: vec![], line_mapping: vec![] };
    let mut output_line_map = LineMap::new();
    output_line_map.errors_count = input_line_map.errors_count;
    output_line_map.warnings_count = input_line_map.warnings_count;

    let mut input_line_map = input_line_map;

    let bytes_per_command = 4; // 32 bit

    let mut bytes_count = 0;
    let mut current_section_start = 0;

    let mut global_constants_names: Vec<String> = Vec::new();

    // Go through the code line by line and resolve all the ones starting with a '.'.
    for i in code.iter().enumerate(){
        let line_number = i.0;
        let line = i.1.clone();

        let first_token = line.iter().nth(0);

        if first_token.is_none(){ continue; }

        if first_token.unwrap() == "." {
            let command = line.iter().nth(1);

            if command.is_none() {
                input_line_map.print_notification(ErrorNotificationKind::Warning, line_number as u32, Some(0), "Unused assembler command".to_string(), "Expected assembler command or constant name after \".\".".to_string());
                continue;
            }

            match command.cloned().unwrap().clone().as_str() {
                "section" => {
                    if line.len() != 5 {
                        input_line_map.print_notification_multiple_faulty_tokens(ErrorNotificationKind::Error, line_number as u32, 2 /* both . and section are fine*/, (line.len() - 1) as u32 /* all remaining tokens*/, "Compiler command formatting error".to_string(), "The compiler command \"section\" requires a string as an argument, but no string was found.".to_string());
                        continue;
                    }

                    if line[2] != "\"" || line[4] != "\""{
                        input_line_map.print_notification_multiple_faulty_tokens(ErrorNotificationKind::Error, line_number as u32, 2 /* both . and section are fine*/, (line.len() - 1) as u32 /* all remaining tokens*/, "Compiler command formatting error".to_string(), "The compiler command \"section\" requires a string as an argument, but no string was found.".to_string());
                        continue;
                    }

                    let name = line[3].clone();
                    let section: (String, u32) = (name, bytes_count as u32);
                    result.sections.push(section);
                    current_section_start = bytes_count;
                }

                "ascii" | "stc" => {
                    if line.len() != 5{
                        input_line_map.print_notification_multiple_faulty_tokens(ErrorNotificationKind::Error, line_number as u32, 2 /* both . and section are fine*/, (line.len() - 1) as u32 /* all remaining tokens*/, "Compiler command formatting error".to_string(), format!("The compiler command \"{}\" requires a string as an argument, but no string was found.", command.unwrap() /*ascii or stc*/).to_string());
                        continue;
                    }

                    let first_quote = line.iter().nth(2).unwrap();
                    let text = line.iter().nth(3).unwrap();
                    let second_quote = line.iter().nth(4).unwrap();

                    if first_quote != "\"" || second_quote != "\"" {
                        input_line_map.print_notification_multiple_faulty_tokens(ErrorNotificationKind::Error, line_number as u32, 2 /* both . and section are fine*/, (line.len() - 1) as u32 /* all remaining tokens*/, "Compiler command formatting error".to_string(), format!("The compiler command \"{}\" requires a string as an argument, but no string was found.", command.unwrap() /*ascii or stc*/).to_string());
                        continue;
                    }

                    // Increment the byte counter
                    bytes_count += text.len();

                    // Add only '.', 'ascii' and the text back, not the surrounding quotes.
                    result.code.push(vec![first_token.unwrap().to_string(), command.unwrap().to_string(), text.to_string()]);

                    // Add the correct line mapping
                    let line_number_in_result = result.code.len() - 1;
                    result.line_mapping.push((line_number_in_result, line_number));
                }

                "global" => {
                    if line.len() != 3{
                        input_line_map.print_notification_multiple_faulty_tokens(ErrorNotificationKind::Error, line_number as u32, 2 /* both . and global are fine*/, (line.len() - 1) as u32 /* all remaining tokens*/, "Compiler command formatting error".to_string(), format!("The compiler command \"{}\" requires a variable name but this was found instead.", command.unwrap() /*ascii or stc*/).to_string());
                        continue;
                    }
                    let global_variable_name = line[2].clone();
                    global_constants_names.push(global_variable_name.clone());
                }

                _ => {
                    // This (command) is a variable name.
                    let first_value_token = line.iter().nth(2);

                    // Turn numbers into decimal
                    let mut value: Option<String> = None;

                    if first_value_token.is_none(){
                        input_line_map.print_notification_multiple_faulty_tokens(ErrorNotificationKind::Error, line_number as u32, 0 /* start at the .*/, (line.len() - 1) as u32 /* all remaining tokens*/, "Compiler command formatting error".to_string(), format!("Can't create constant named \"{}\" without a value.", command.cloned().unwrap() /*ascii or stc*/).to_string());
                        continue;
                    }

                    // Try to do math operations
                    if first_value_token.unwrap() == "[" {
                        if line.iter().len() == 7{
                            if line.iter().nth(6).unwrap() != "]"{
                                input_line_map.print_notification_multiple_faulty_tokens(ErrorNotificationKind::Error, line_number as u32, 2 /* both . and name are fine*/, (line.len() - 1) as u32 /* all remaining tokens*/, "Compiler command formatting error".to_string(), format!("The value of the constant named \"{}\" couldn't be read.", command.cloned().unwrap() /*ascii or stc*/).to_string());
                                continue;
                            }

                            let math_op = vec![line.iter().nth(3).unwrap().to_string(), line.iter().nth(4).unwrap().to_string(), line.iter().nth(5).unwrap().to_string()].join(" ");
                            let constants = vec![result.constants.clone(), vec![Replacement::new("$".to_string(), bytes_count.to_string(), true)]].concat();
                            let result_str = resolve_string(math_op, constants);

                            if result_str == "" {
                                input_line_map.print_notification_multiple_faulty_tokens(ErrorNotificationKind::Error, line_number as u32, 2 /* both . and section are fine*/, (line.len() - 1) as u32 /* all remaining tokens*/, "Compiler command formatting error".to_string(), format!("The value of the constant named \"{}\" couldn't be calculated.", command.cloned().unwrap() /*ascii or stc*/).to_string());
                                continue;
                            }

                            value = Some(result_str);
                        }
                    }

                    // If those didn't succeed, try to just decode the value
                    if value.is_none(){
                        if line.iter().len() == 3{
                            let value_token = line.iter().nth(2).unwrap();
                            if let Some(value_i32) = value_token.parse::<i32>().ok(){
                                value = Some(value_i32.to_string());
                            } else {
                                input_line_map.print_notification_multiple_faulty_tokens(ErrorNotificationKind::Error, line_number as u32, 2 /* both . and name are fine*/, (line.len() - 1) as u32 /* all remaining tokens*/, "Compiler command formatting error".to_string(), format!("The value of the constant named \"{}\" couldn't be decoded as a base-10 integer value.", command.cloned().unwrap()).to_string());
                                continue;
                            }
                        } else if line.iter().len() == 4 {
                            let prefix = line.iter().nth(2).unwrap();
                            let number = line.iter().nth(3).unwrap();

                            match prefix.as_str(){
                                "0x" => {
                                    if let Some(value_i32) = i32::from_str_radix(number, 16).ok(){
                                        value = Some(value_i32.to_string());
                                    } else {
                                        input_line_map.print_notification_multiple_faulty_tokens(ErrorNotificationKind::Error, line_number as u32, 2 /* both . and name are fine*/, (line.len() - 1) as u32 /* all remaining tokens*/, "Compiler command formatting error".to_string(), format!("The value of the constant named \"{}\" couldn't be decoded as a hexadecimal integer value.", command.cloned().unwrap()).to_string());
                                        continue;
                                    }
                                }

                                "0o" => {
                                    if let Some(value_i32) = i32::from_str_radix(number, 8).ok(){
                                        value = Some(value_i32.to_string());
                                    } else {
                                        input_line_map.print_notification_multiple_faulty_tokens(ErrorNotificationKind::Error, line_number as u32, 2 /* both . and name are fine*/, (line.len() - 1) as u32 /* all remaining tokens*/, "Compiler command formatting error".to_string(), format!("The value of the constant named \"{}\" couldn't be decoded as an octal integer value.", command.cloned().unwrap()).to_string());
                                        continue;
                                    }
                                }

                                "0b" => {
                                    if let Some(value_i32) = i32::from_str_radix(number, 2).ok(){
                                        value = Some(value_i32.to_string());
                                    } else {
                                        input_line_map.print_notification_multiple_faulty_tokens(ErrorNotificationKind::Error, line_number as u32, 2 /* both . and name are fine*/, (line.len() - 1) as u32 /* all remaining tokens*/, "Compiler command formatting error".to_string(), format!("The value of the constant named \"{}\" couldn't be decoded as a binary integer value.", command.cloned().unwrap()).to_string());
                                        continue;
                                    }
                                }

                                _ => {
                                    input_line_map.print_notification_multiple_faulty_tokens(ErrorNotificationKind::Error, line_number as u32, 2 /* both . and name are fine*/, (line.len() - 1) as u32 /* all remaining tokens*/, "Compiler command formatting error".to_string(), format!("The value of the constant named \"{}\" couldn't be decoded.", command.cloned().unwrap()).to_string());
                                    continue;
                                }
                            }
                        } else {
                            input_line_map.print_notification_multiple_faulty_tokens(ErrorNotificationKind::Error, line_number as u32, 2 /* both . and name are fine*/, (line.len() - 1) as u32 /* all remaining tokens*/, "Compiler command formatting error".to_string(), format!("The value of the constant named \"{}\" couldn't be decoded.", command.cloned().unwrap()).to_string());
                            continue;
                        }
                    }



                    let replacement = Replacement::new(command.unwrap().to_string(), value.unwrap(), false);
                    result.constants.push(replacement);
                }
            }
            continue;
        }

        let last_token = line.last().unwrap().to_string();

        if last_token == ":" {
            let function_name = line.first().unwrap().to_string();
            let default = &("NOSEC".to_string(), 0);
            let current_section = result.sections.last().unwrap_or(default);
            let current_section_name = current_section.0.clone();
            let bytes_in_section = bytes_count - current_section_start;
            let value = current_section_name.clone() + ":" + bytes_in_section.to_string().as_str();

            let replacement = Replacement::new(function_name, value, true);
            result.constants.push(replacement);

            continue;
        }

        // Make global constants/variables global
        'outerloop: for global_constant_name in global_constants_names.clone() {
            for i in 0..result.constants.len(){
                if result.constants[i].get_name() == global_constant_name{
                    result.constants[i].set_is_global(true);
                    continue 'outerloop;
                }
            }
            result.constants.push(Replacement::new(global_constant_name, "".to_string(), false));
        }

        // Not an assembler command, just add it to the list.
        result.code.push(line.clone());

        let line_number_in_result = result.code.len() - 1;
        result.line_mapping.push((line_number_in_result, line_number));

        // Copy the input map's info to the output map
        let line_info_input = input_line_map.lines[line_number].clone();
        output_line_map.add_line(line_info_input);

        bytes_count += bytes_per_command;
    }



    (result, output_line_map)
}

pub struct ValueGenResult{
    pub constants: Vec<Replacement>,
    pub sections: Vec<(String, u32)>, //Name of the section followed by the correct line (starting at 0) from the resulting code.
    pub code: Vec<Vec<String>>,
    pub line_mapping: Vec<(usize, usize)>, // How the new line number in the resulting code above (.0) refers to the original line number (.1)
}

#[cfg(test)]
mod tests{
    use crate::assembler::valuegen::gen_values;
    use crate::util::line_mapping::{LineInfo, LineMap};
    use crate::util::replacement::Replacement;

    #[test]
    fn test_gen_values(){
        let data = vec![
            vec![".".to_string(), "global".to_string(), "main".to_string()],
            vec![".".to_string(), "global".to_string(), "xyz".to_string()],
            vec![".".to_string(), "xyz".to_string(),  "10".to_string()],
            vec![".".to_string(), "zyx".to_string(), "0x".to_string(), "10".to_string()],
            vec![".".to_string(), "abc".to_string(),  "[".to_string(), "xyz".to_string(), "+".to_string(), "10".to_string(), "]".to_string()],
            vec![".".to_string(), "section".to_string(), "\"".to_string(), "CODE".to_string(), "\"".to_string()],
            vec!["main".to_string(), ":".to_string()],
            vec!["adrp".to_string(), "x0".to_string(), ",".to_string(), "msg".to_string(), "@".to_string(), "PAGE".to_string()],
            vec![".".to_string(), "section".to_string(), "\"".to_string(), "DATA".to_string(), "\"".to_string()],
            vec!["msg".to_string(), ":".to_string()],
            vec![".".to_string(), "ascii".to_string(), "\"".to_string(), "Hi".to_string(), "\"".to_string()],
        ];

        let mut expected_constants = vec![
            Replacement::new("xyz".to_string(), "10".to_string(), false),
            Replacement::new("zyx".to_string(), "16".to_string(), false),
            Replacement::new("abc".to_string(), "20".to_string(), false),
            Replacement::new("main".to_string(), "CODE:0".to_string(), true),
            Replacement::new("msg".to_string(), "DATA:0".to_string(), true),
        ];

        expected_constants[0].set_is_global(true);
        expected_constants[3].set_is_global(true);

        let expected_sections : Vec<(String, u32)>= vec![
            ("CODE".to_string(), 0),
            ("DATA".to_string(), 4),
        ];

        let expected_code = vec![
            vec!["adrp", "x0", ",", "msg", "@", "PAGE"],
            vec![".", "ascii", "Hi"],
        ];

        let expected_line_mapping = vec![(0, 7), (1, 10)];

        let mut line_map = LineMap::new();

        for _ in 0..100{
            line_map.add_line(LineInfo::new_no_info("".to_string(), 0))
        }

        let result = gen_values(data, line_map);

        assert_eq!(result.0.constants.len(), expected_constants.len());
        assert_eq!(result.0.code.len(), expected_code.len());
        assert_eq!(result.0.line_mapping.len(), expected_line_mapping.len());
        assert_eq!(result.0.sections.len(), expected_sections.len());


        for i in 0..expected_constants.len(){
            assert_eq!(result.0.constants[i], expected_constants[i]);
        }

        for i in 0..expected_code.len(){
            assert_eq!(result.0.code[i].len(), expected_code[i].len());
            for j in 0..expected_code[i].len(){
                assert_eq!(result.0.code[i][j], expected_code[i][j]);
            }
        }

        for i in 0..expected_line_mapping.len(){
            assert_eq!(result.0.line_mapping[i], expected_line_mapping[i]);
        }

        for i in 0..result.0.sections.len(){
            assert_eq!(result.0.sections[i], expected_sections[i]);
        }

    }
}