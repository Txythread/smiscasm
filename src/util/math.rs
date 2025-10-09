use colorize::*;
use crate::assembler::valuegen::Section;
use crate::util::operation::Operation;
use crate::util::replacement::Replacement;

pub fn resolve_argument(argument: String, replacements: Vec<Replacement>, sections: Vec<Section>) -> String {
    let replacements = create_string_resolving_replacements(replacements, sections);

    resolve_string(argument, replacements)
}

/// Creates things like page offsets (PAGEOFF), pages (@PAGE) etc.
fn create_string_resolving_replacements(replacements: Vec<Replacement>, sections: Vec<Section>) -> Vec<Replacement> {
    let mut output_replacements: Vec<Replacement> = Vec::new();

    for replacement in replacements {
        // Look if it's an address.
        // If yes, generate @PAGE and @PAGEOFF

        let page_name = replacement.get_value().split(':').map(|x|x.to_string().clone()).nth(0);
        let page_offset = replacement.get_value().split(':').map(|x|x.to_string().clone()).nth(1);

        // If both of the above values exist, then it's an address.
        if page_name.is_some() && page_offset.is_some() {
            let page_name = page_name.unwrap();
            let page_offset = page_offset.unwrap();

            if let Some(page_offset) = page_offset.parse::<i32>().ok() {
                // The how-many-th section fits the name?
                if let Some(page_start) = sections.clone().iter().enumerate().find(| &x | x.1.clone().name == page_name).map(| x | x.0){
                    output_replacements.push(Replacement::new(format!("{}@PAGE", replacement.get_name()), page_start.to_string(), replacement.get_is_function()));
                    output_replacements.push(Replacement::new(format!("{}@PAGEOFF", replacement.get_name()), page_offset.to_string(), replacement.get_is_function()));
                }
            }
        }


        // Look if it is an integer value
        if let Some(replacement_value) = replacement.get_value().parse::<i32>().ok() {
            // Generate bytes
            output_replacements.push(Replacement::new(format!("{}@LSB", replacement.get_name()), (replacement_value & 0x00_00_00_ff).to_string(), replacement.get_is_function()));
            output_replacements.push(Replacement::new(format!("{}@B1", replacement.get_name()), ((replacement_value & 0x00_00_ff_00) >> 8).to_string(), replacement.get_is_function()));
            output_replacements.push(Replacement::new(format!("{}@B2", replacement.get_name()), ((replacement_value & 0x00_ff_00_00) >> 16).to_string(), replacement.get_is_function()));
            output_replacements.push(Replacement::new(format!("{}@MSB", replacement.get_name()), ((replacement_value & 0xff_00_00_00u32 as i32) >> 24).to_string(), replacement.get_is_function()));
        }

        // Add the normal replacements to the map, too
        output_replacements.push(replacement);
    }


    output_replacements
}

/// Turns a string like "1 + 2" to "3"
pub fn resolve_string(string: String, replacements: Vec<Replacement>) -> String {
    // Tokenize
    let mut tokens: Vec<&str> = split_with_delimiters(&*string, &[' ', '+', '*', '/', '-']);
    // Remove empty tokens
    tokens = tokens.iter().filter(|&x| !(*x).is_empty() && *x != " ").map(|&x| x).collect();

    let mut operand_1: Option<i64> = None;
    let mut operand_2: Option<i64> = None;
    let mut operation: Option<Operation> = None;


    for token in tokens {
        let mut token = token.to_string().trim().to_string();
        if let Some(op) = Operation::from_string(&token) {
            operation = Some(op);
            continue;
        }
        // Apply all replacements
        for replacement in replacements.clone() {
            if token == replacement.get_name(){
                token = replacement.get_value().to_string();
            }
        }

        // Fix the raw number format if possible (convert to decimal)
        let token = convert_to_decimal(token);

        if operand_1.is_none() {

            let op_1 = token.parse::<i64>();

            if op_1.is_err() { return "".to_string(); }

            operand_1 = Some(op_1.unwrap());
            continue;
        }

        let op_2 = token.parse::<i64>();

        if op_2.is_err() { return "".to_string(); }

        operand_2 = Some(op_2.unwrap());
        break;
    }

    if let Some(operation) = operation {
        if let Some(operand_1) = operand_1 {
            if let Some(operand_2) = operand_2 {
                let result = operation.perform(operand_1, operand_2).to_string();
                return result;
            }
            let error = format!("Only 1/2 operands specified to resolve ({})", string).red();
            panic!("{}", error);
        }
        let error = format!("No operand specified to resolve ({})", string).red();
        panic!("{}", error);
    }else{
        // Check if the string only contains one operand.
        if operand_2.is_none() && operand_1.is_some() {
            return operand_1.unwrap().to_string();
        }


        let error = format!("No operation specified to resolve ({})", string).red();
        panic!("{}", error);
    }
}

fn split_with_delimiters<'a>(s: &'a str, delims: &[char]) -> Vec<&'a str> {
    let mut out = Vec::new();
    let mut start = 0;

    for (i, c) in s.char_indices() {
        if delims.contains(&c) {
            if start != i {
                let text = &s[start..i];
                if text != " "{
                    out.push(text); // push text before delimiter
                }
            }
            out.push(&s[i..i + c.len_utf8()]);
            start = i + c.len_utf8();
        }
    }

    if start < s.len() {
        out.push(&s[start..]); // push the last part
    }

    out
}


/// Turns a string with a number in octal, binary or hexadecimal into decimal if possible, returns the input string otherwise.
fn convert_to_decimal(input: String) -> String {

    if let Some(hex_value) = input.strip_prefix("0x") {
        if let Some(value) = u32::from_str_radix(hex_value, 16).ok() {
            return value.to_string()
        }
    }

    if let Some(bin_value) = input.strip_prefix("0b") {
        if let Some(value) = u32::from_str_radix(bin_value, 2).ok() {
            return value.to_string()
        }
    }

    if let Some(octal_value) = input.strip_prefix("0o") {
        if let Some(value) = u32::from_str_radix(octal_value, 8).ok() {
            return value.to_string()
        }
    }

    // Couldn't convert from binary, hexadecimal and octal, just return the input string
    input
}

#[cfg(test)]
mod tests {
    use crate::assembler::valuegen::Section;
    use crate::util::math::{create_string_resolving_replacements, resolve_argument, resolve_string};
    use crate::util::replacement::Replacement;

    #[test]
    fn test_resolve_string() {
        assert_eq!(resolve_string(String::from("3 * 8"), vec![]), "24");
        assert_eq!(resolve_string(String::from("15 + 3"), vec![]), "18");

        let replacements = vec![Replacement::new("x".to_string(), "5".to_string(), false)];
        assert_eq!(resolve_string(String::from("10 - x"), replacements.clone()), "5");

        assert_eq!(resolve_string("x".to_string(), replacements), "5");

        assert_eq!(resolve_string(String::from("\"Hello world\""), vec![]), "");
    }

    #[test]
    fn test_resolve_string_resolving_replacements() {
        let replacements = vec![Replacement::new("msg".to_string(), "DATA:5".to_string(), false)];
        let sections = vec![
            Section { name: "CODE".to_string(), start_offset: 0, start_memory_page: 0, start_pos_bytes_original: 0 },
            Section { name: "DATA".to_string(), start_offset: 1, start_memory_page: 1, start_pos_bytes_original: 1 },
        ];

        let new_replacements = create_string_resolving_replacements(replacements, sections);


        assert_eq!(new_replacements[0], Replacement::new("msg@PAGE".to_string(), "1".to_string(), false));
        assert_eq!(new_replacements[1], Replacement::new("msg@PAGEOFF".to_string(), "5".to_string(), false));
        assert_eq!(new_replacements[2], Replacement::new("msg".to_string(), "DATA:5".to_string(), false));
    }

    #[test]
    fn test_resolve_argument() {
        let replacements = vec![
            Replacement::new("msg".to_string(), "DATA:5".to_string(), false),
            Replacement::new("msg_len".to_string(), "13".to_string(), false),
        ];
        let sections = vec![
            Section { name: "CODE".to_string(), start_offset: 0, start_memory_page: 0, start_pos_bytes_original: 0 },
            Section { name: "DATA".to_string(), start_offset: 1, start_memory_page: 1, start_pos_bytes_original: 1 },
        ];

        let argument_result = resolve_argument("msg@PAGEOFF + msg_len".to_string(), replacements, sections);

        assert_eq!(argument_result, "18");
    }
}