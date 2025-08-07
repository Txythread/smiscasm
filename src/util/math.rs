use colorize::*;
use crate::util::operation::Operation;
use crate::util::replacement::Replacement;

/// Turns a string like "1 + 2" to "3"
pub(crate) fn resolve_string(string: String, replacements: Vec<Replacement>) -> String {
    // Tokenize
    let tokens: Vec<&str> = string.split(' ').collect();
    let mut operand_1: Option<i64> = None;
    let mut operand_2: Option<i64> = None;
    let mut operation: Option<Operation> = None;

    for token in tokens {
        let mut token = token.to_string();
        if let Some(op) = Operation::from_string(&token) {
            operation = Some(op);
            continue;
        }
        // Apply all replacements
        for replacement in replacements.clone() {
            let replacement = token.replace(&replacement.get_name(), &replacement.get_value());
            token = replacement;
        }

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
        let error = format!("No operation specified to resolve ({})", string).red();
        panic!("{}", error);
    }
}

#[cfg(test)]
mod tests {
    use crate::util::math::resolve_string;

    #[test]
    fn test_resolve_string() {
        assert_eq!(resolve_string(String::from("3 * 8"), vec![]), "24");
        assert_eq!(resolve_string(String::from("15 + 3"), vec![]), "18");
        assert_eq!(resolve_string(String::from("\"Hello world\""), vec![]), "");
    }
}