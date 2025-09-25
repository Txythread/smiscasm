# Assembler - Portion

**The assembler portion of this assembler consists of the following stages:**

1. **Include** - dependencies are added as code. A line mapping is created to trace the code back to its original position later.
1. **Preprocess** - Removes comments, empty lines, etc.
1. **Splitting** - Individual tokens won't get tokenized here, but they'll be separated from each other.
1. **Value Generation** - Creates constants and labels
1. **Value Replacement** - The constants and labels get inserted into the code where they are needed.
1. **Tokenization** - The lines get split into arguments, which then get split into immediate values and out
1. **Binary Translation** - The lines (in token form) get put into binary

*Those steps are executed (in series) by the `assembler.rs` file.*