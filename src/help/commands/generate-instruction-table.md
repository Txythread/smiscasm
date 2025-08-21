**Generate Instruction Table**

`--generate-instruction-table` generates 8 binary files containing control words that can be used for the CPU's CU's EEPROM/Flash/Other persistent storage chips or for **smiscvm** (which probably *doesn't exist yet*).
The position in the binary is the address in said chip. Control words are 8 bits.