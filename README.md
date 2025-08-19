# smiscasm


## Introduction
This project serves as an assembler and a linker for my SMISC (Stupid Mixed Insruction Set Computer) assembly language.
This language only serves one purpose: easy to implement (physically) while still allowing for complex instructions.

## Assembly basics
### Constants
In this assembly language, constants are defined with a simple `.`, immediately followed by the constant's name and its value:  

```.abc 10```  
*Define a constant named "abc" with an integer value of 10.*  

Please note that **some names are reserved** as they refer to assembler commands (e.g. `section`).  

In code, you can use a previously defined constant just by calling its name. It'll be turned into a regular immediate value during compile time.

### Globals
***Globals are currently of no use and SHOULD BE AVOIDED***
... but if you want to you can make a constant by writing `.global <constant name>` after it has been defined.

### Sections
Sections can be defined with:  

```.section "SECTION NAME"```
*Creates a new section named 'SECTION NAME'.*  

Sections are for primarily used to avoid issues with memory pages.
A new section always starts at a new memory page & no section should ever be longer than one memory page.
Though it currently has almost no effect, please name pages appropriately, common names include "CODE" and "DATA" (potentially followed by an index).


### Memory Pages
Memory Pages are 2048B in SMISC. To address a memory page, use label_in_said_page@PAGE. To access the offset of that label within that page, use the @PAGEOFF modifier instead.

### Registers
There are 32 regs called x<sub>n</sub> where n is between 0 and 31.

### Immediate Values
Immediate values are coded with **11b** (0...2047), 1 additional bit infront of that that marks shift-by-11 & one more additional bit in front of that one that accounts for the **sign** (everything is coded using two's complement). An immediate value can be coded decimal, hexadecimal (when starting with 0x), octal (0o) or binary (0b).

### Labels
Labels (or functions) start with their name followed by a colon. They can later be **used like a constant**, but they refer to their position in bytes instead of a user-set value.  

```main:```
*Essentially creates a constant with the current position as the value.*

### Instructions
An instruction is coded using its name followed by its arguments (each seperated with a comma).
Their might be an indent before and after the instruction.
Example instruction:  

```add x1, 5```


### Comments
Comments are defined using a `'#'`. They exclude everything after themselves (plus themselves) from the line in the early stages of assembling.

### Includes
More source code from elsewhere can be included in a file using the `!include` command. 
This command is then followed by either a file name from the same directory (`!include other_file.s`), an URL (`!include https://...`) or the name of a well-known library (`!include bscmath` (library not finished yet)). The include statement gets replaced with the code from the called file during assembly. Downloaded files will be stored for later in the pub-libs directory. Remove it if you want to re-download all sources.

## Coding New Instructions
Instructions must be stored in the instructions directory (as they are included when smiscasm is getting compiled). 
* The first line is the instruction's name as it's called in the code.
* The second line is the format (such as (Register, Immediate Value) or (Register, Register) or () (None)). A x means a register, an i means an immediate value. * * Multiple functions with the same name but different formats can co-exist.
* The third line is the address (only 8 bits). Please make sure it's unique.
* The following lines are just micro instructions. All the ones defined in `src/instruction/instruction.rs/OUTPUT_MAP_STRING` can be used for this and new ones can be added there. Seperate stages (for different clockcycles are defined with the @STAGE command. New versions of the same stage are defined with @VERSION. @ZF (zero flag) or @PM (privileged mode) can be used to make that version only apply when those flags are set.

## Generating Instruction Tables
Instruction tables contain data to write to EEPROMs or flash chips (or any persistent storage for that matter) from the CPU's CU to decode the commands.
Make sure you are in an empty directory before executing the command as it clutters the PWD.

You can generate them with the `--generate-instruction-table` flag.

## Actually Compiling
To run *smiscasm* with its standard functionality (assembling & linking), just run:  
```smiscasm my_code.s```

## Exit Codes
You can take a look at exit_codes.txt to see what all the exit codes (except for 0) mean. This only includes intentional exit codes, not Rust's 101.

## Further Info
Run smiscasm with the `--help` flag for more info on flags. Also check out the code & instructions for more. Some instructions are documented, some not really tbh.
