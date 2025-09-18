use crate::util::line_mapping::{LineInfo, LineMap};

// Removes comments, empty lines, splits lines into parts
/// Removes unnecessary information, splits text into lines and tokens
pub async fn preprocess(code: String, input_line_map: LineMap) -> (Vec<String>, LineMap) {
    let mut output_line_map = LineMap::new();
    output_line_map.warnings_count = input_line_map.warnings_count;
    output_line_map.errors_count = input_line_map.errors_count;


    // Split into lines at newline characters
    let mut lines: Vec<String> = code.lines().map(|x| x.to_string()).collect();

    // Remove comments
    let mut lines_without_comments: Vec<String> = vec![];

    for i in 0..lines.iter().count(){
        let line = lines[i].clone();
        let mut line_without_comment: String = String::new();

        for char in line.chars(){
            if char == '#' { /*Comment starts here*/ break; }

            // Char not part of a comment
            line_without_comment.push(char);
        }

        lines_without_comments.push(line_without_comment);
    }

    lines = lines_without_comments;

    // Split into lines at semicolons
    let lines_and_semicolons: Vec<Vec<String>> = lines.iter().map(|x| (*x.clone().split(';').map(|x| x.to_string()).collect::<Vec<String>>()).to_owned()).collect();
    lines = lines_and_semicolons.concat();

    // Remove empty lines, leading and trailing whitespaces
    let mut lines_with_content: Vec<String> = Vec::new();

    let mut line_number = 0;
    for line in lines.iter(){
        let line_in_input_map = input_line_map.lines[line_number as usize].clone();
        line_number += 1;

        // If the line without whitespaces is empty, it contains nothing but whitespaces.
        let line_without_whitespaces = line.split_whitespace().collect::<Vec<&str>>().join("");

        if !line_without_whitespaces.is_empty() {
            lines_with_content.push(line.trim().to_string());
            output_line_map.add_line(line_in_input_map.clone());
        }
    }

    lines = lines_with_content;



    (lines, output_line_map)
}



#[cfg(test)]
mod tests {
    use crate::assembler::preprocesser::preprocess;
    use crate::util::line_mapping::LineMap;

   /* #[tokio::test]
    async fn test_preprocesser() {
        let preprocessed_code = preprocess("\n\
        \n\
        .section \"CODE\"\n\
main:\n\
        adrp x0, msg@PAGE; add x0, msg@PAGEOFF\n\
        adrp x1, msg_end@PAGE\n\
        add x1, msg_end@PAGEOFF\n\
        sub x1, 1\n\
        \n\
loop:\n\
        lb x2, x0\n\
        add x0, 1\n\
        out x2\n\
        mov x0, x3\n\
        sub x3, x1\n\
        jmpz x3, end\n\
        jmp loop\n\
\n\
end:\n\
        hlt\n\
 \n\
.section \"DATA\"\n\
msg:\n\
        .ascii \"Hello, world!\" #Very important text\n\
.msg_end [$ - 1]\n\
        ".to_string(), LineMap::new());
        let expected_result = vec![
                                   ".section \"CODE\"",
                                   "main:",
                                   "adrp x0, msg@PAGE",
                                   "add x0, msg@PAGEOFF",
                                   "adrp x1, msg_end@PAGE",
                                   "add x1, msg_end@PAGEOFF",
                                   "sub x1, 1",
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



        for i in 0..preprocessed_code.await.0.len().clone(){
            assert_eq!(preprocessed_code.await.0[i].clone(), expected_result[i]);
        }
    }*/
}