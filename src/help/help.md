**smiscasm - Stupid Mixed Instruction Set Computer Assembler**

Assembles and links SMISC assembly code.

**Options:**
* Arguments that are not flags and do not belong to any will be treated as input files.
* `--generate-instruction-table`:
    Generates data for the CU's decoding memory based on all the instructions in the instructions folder.
* `-o` or `--output`:
    Specifies the output file's name and folder. If this option isn't used, the input name will be the output name (except for the suffix).
* `--get-micro-operation`:
    Gets a micro operation's name by its index (starting at 0).
* `--instruction-help`:
    Gets help for an instruction (from assembly) if the help file exists. Usage: `smiscasm --instruction-help add`
* `-h` or `-help` prints this screen or other help screens if accompanied by the flags listed above.