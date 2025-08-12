use crate::assembler::preprocesser::preprocess;
use crate::assembler::tokenizer::tokenize;
use crate::assembler::valuegen::{gen_values, ValueGenResult};
use crate::assembler::valuerepl::{replace_values_in_code, ValueReplResult};
use crate::assembler::ya_tokenizer::{tokenize_ya_time, YATokenizerResult};
use crate::assembler::zstep::perform_last_name;
use crate::instruction::instruction::Instruction;

pub fn assemble(code: String, instructions: Vec<Instruction>) -> Vec<u8> {
    let preprocesed: Vec<String> = preprocess(code);
    let tokenized: Vec<Vec<String>> = tokenize(preprocesed);
    let value_gen_result: ValueGenResult = gen_values(tokenized);
    let value_repl_result: ValueReplResult = replace_values_in_code(value_gen_result);
    let tokenized: YATokenizerResult = tokenize_ya_time(value_repl_result);
    let binary: Vec<u8> = perform_last_name(tokenized, instructions);

    binary
}

#[cfg(test)]
mod tests {
    use crate::assembler::assembler::assemble;
    use crate::assembler::preprocesser::preprocess;
    use crate::instruction::instruction::get_all_instructions;

    #[test]
    fn test_assemble() {
        let instructions = get_all_instructions();
        let assembled = assemble("\n\
        \n\
        .section \"CODE\"\n\
main:\n\
        adrp x0, msg@PAGE\n\
        add x0, msg@PAGEOFF\n\
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
\n\
end:\n\
        hlt\n\
 \n\
.section \"DATA\"\n\
msg:\n\
    .ascii \"Hello, world!\" #Very important text\n\
msg_end:
    .ascii \"e\"\n\
\n\
        ".to_string(), instructions);


        assert_eq!(assembled, vec![0, 2, 0, 5, 67, 79, 68, 69, 250, 0, 0, 0, 0, 251, 68, 65, 84, 65, 250, 0, 0, 0, 44, 251, 109, 97, 105, 110, 250, 67, 79, 68, 69, 58, 48, 251, 108, 111, 111, 112, 250, 67, 79, 68, 69, 58, 50, 48, 251, 101, 110, 100, 250, 67, 79, 68, 69, 58, 52, 48, 251, 109, 115, 103, 250, 68, 65, 84, 65, 58, 48, 251, 109, 115, 103, 95, 101, 110, 100, 250, 68, 65, 84, 65, 58, 49, 51, 251, 0, 2, 0, 0, 0, 0, 0, 0, 0, 8, 97, 0, 0, 0, 64, 68, 65, 84, 65, 255, 80, 0, 0, 0, 97, 4, 0, 0, 64, 68, 65, 84, 65, 255, 80, 4, 0, 13, 81, 4, 0, 1, 88, 8, 0, 0, 80, 0, 0, 1, 89, 8, 0, 0, 96, 128, 96, 0, 81, 140, 32, 0, 125, 0, 0, 0, 72, 101, 108, 108, 111, 44, 32, 119, 111, 114, 108, 100, 33, 101]);
    }
}