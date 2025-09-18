// This code is here for displaying a/n error/warning to the user when there's a mistake in the code.
// Right now, this hardly does anything special, but highlighting and other stuff can be implemented in the future.

use colorize::AnsiColor;

/// Notify the user that a mistake is in the code.
pub fn display_code_error(kind: ErrorNotificationKind, line: i32, column: Option<u32>, underlined_length: Option<u32>, title: String, message: String, code: Vec<String>) {
    let formated_title: String;
    let formated_message: String;

    match kind {
        ErrorNotificationKind::Error => {
            formated_title = format!("Error: {}", title).red().bold().to_string();
            formated_message = message.red().to_string();
        }

        ErrorNotificationKind::Warning => {
            formated_title = format!("Warning: {}", title).yellow().bold().to_string();
            formated_message = message.yellow().to_string();
        }
    }

    // Print the title
    println!("{}", formated_title);

    // Print the affected line
    println!("{}:\t{}", line, code[line as usize].clone().to_string());

    if let Some(column) = column {
        // Generate an offset in spaces to underline/point to the wrong line
        let mut offset_string = "\t".to_string();

        for _ in 0..column {
            offset_string += " ";
        }

        print!("{}", offset_string);

        // The string that's shown at the position of the mistake.
        let mut hint_string = "^".to_string();

        // Look if a length is available, if yes, underline the rest.
        if let Some(underlined_length) = underlined_length {
            let mut underline_string = "".to_string();

            for _ in 0..underlined_length-1 {
                underline_string += "~";
            }

            hint_string += underline_string.as_str();
        }

        // Make the color appropriate
        match kind {
            ErrorNotificationKind::Error => hint_string = hint_string.red().to_string(),
            ErrorNotificationKind::Warning => hint_string = hint_string.yellow().to_string(),
        }

        println!("{}", hint_string);
    }

    println!("{}", formated_message);
}

#[derive(Clone)]
pub enum ErrorNotificationKind {
    Error,
    Warning
}