**smiscasm - Stupid Mixed Instruction Set Computer Assembly**

Assembles smisc assembly code to something smiscld can link to an actual binary.

**Options:**
* `--generate-instruction-table`:
    Generates data for the CU's decoding memory based on all the instructions in the instructions folder.
* `-o` or `--output`:
    Specifies the output file's name and folder. If this option isn't used, the input name will be the output name (except for the suffix).
* `-h` or `-help` prints this screen.