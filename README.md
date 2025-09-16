# SMISCasm - Stupid Mixed Instruction Set Computer Assembly

## Introduction
This project serves as an assembler and a linker for my SMISC (Stupid Mixed Insruction Set Computer) assembly language.
This language only serves one purpose: easy to implement (physically) while still allowing for complex instructions.

## Installation
### Install all of smisc (recommended)
To install smiscasm, smiscvm and smisc-connect all at once, execute the following command:
```
curl -s https://raw.githubusercontent.com/Txythread/smiscasm/master/install-smisc.sh | sh
```
*This might ask you for your password. This is requrired to move binaries into `/usr/local/bin`. If you don't want to enter your password in someones script, you can do this manually. Hint: If you want to do this, take a look at `production.sh` or `build.sh` in each of the downloaded directories if you wish to proceed this way.*

### Install smiscasm only
First, pull the repo.
```
git pull https://github.com/Txythread/smiscasm
```
Then, "cd" into it.
```
cd smiscasm
```
Lastly, execute the build script.
```
./production.sh
```
*This will ask you for your password. If you don't want this, cancel the script and execute `sudo mv target/debug/smiscasm /usr/bin/local` manually.*




## Assembly Basics
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
Memory Pages are 4096B in SMISC. To address a memory page, use label_in_said_page@PAGE. To access the offset of that label within that page, use the @PAGEOFF modifier instead.

### Registers
There are 32 regs called x<sub>n</sub> where n is between 0 and 31.
Some registers have special functionality or special naming.

| NAME(S) | Description                      | Calling behaviour | VM-only |
| ------- | -------------------------------- | ----------------- | ------- |
| x0 | Normal | Argument, return value | No |
| x1 | Normal | Argument, I/O | No |
| x2 | Normal | Argument, I/O | No |
| x3 | Normal | Argument, I/O | No |
| x4 | Normal | Argument, I/O | No |
| x5 | Normal | Argument, I/O | No |
| x6 | Normal | Argument, I/O | No |
| x7 | Normal | Argument, I/O | No |
| x8 | Normal | No-change in function | No |
| x9 | Normal | No-change in function | No |
| x10 | Normal | No-change in function | No |
| x11 | Normal | No-change in function | No |
| x12 | Normal | No-change in function | No |
| x13 | Normal | No-change in function | No |
| x14 | Normal | No-change in function | No |
| x15 | Normal | No-change in function | No |
| x16 | Normal | Changed in function | No |
| x17 | Normal | Changed in function | No |
| x18 | Normal | Changed in function | No |
| x19 | Normal | Changed in function | No |
| x20 | Normal | Changed in function | No |
| x21 | Normal | Changed in function | No |
| x22 | Normal | Changed in function | No |
| x23 | Normal | Changed in function | No |
| x24 | Reserved | Undecided | No |
| x25 | Reserved | Undecided | No |
| x26 | Reserved | Undecided | No |
| x27 | Reserved | Undecided | No |
| x28 | Reserved | Undecided | No |
| x29 | Reserved | Undecided | No |
| x30 | Reserved | Undecided | No |
| x31/sp | Stack Pointer; stores address of the lowest stack address | Reset at end | No |
| N/A | Zero flag / Flag #1 | Changed | No |
| N/A | Privileged Mode / Flag #2 | User specified | No |
| N/A | Memory Address Pointer; the address the memory is loading | Changed | No |
| N/A | Current Instruction | Changed | No |
| N/A | Micro operation counter | Changed | No |
| N/A | ALU arg 1; the first value of the ALU | Changed | No |
| N/A | ALU arg 2; the second value of the ALU | Changed | No |
| N/A | Program Counter; the address of the current/next instruction | Changed | No |
| N/A | Standard transmitter contents; the contents the stdtrans will send when sending / has received | Changed | No |
| N/A | Completed Clock Cycles; stored by VM for øIPC calculations | N/A | Yes |
| N/A | Completed Instruction; stored by VM for øIPC calculations | N/A | Yes |
| N/A | Halted; set by the vm when reset happens while op-counter contains 0 | N/A | Yes |


### Immediate Values
Immediate values are coded with **12b** (0...4096), 1 additional bit infront of that that represents the **sign** (everything is coded using two's complement). In assembly, an immediate value can be coded decimal, hexadecimal (when starting with 0x), octal (0o) or binary (0b).

### Labels
Labels (or functions) start with their name followed by a colon. They can later be **used like a constant**, but they refer to their position in bytes instead of a user-set value.  

```main:```
*Essentially creates a constant with the current position as the value.*

### Instructions
An instruction is coded using its name followed by its arguments (each seperated with a comma).
There might be an indent before and after the instruction.
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
* Then (, after a recommended empty line, ) add the @STAGE command to get from stage 0 (which should be empty usually) to stage 1.
* The following lines are just micro instructions. All the ones defined in `src/instruction/instruction.rs/OUTPUT_MAP_STRING` can be used for this and new ones can be added there. Seperate stages (for different clockcycles are defined with the @STAGE command. New versions of the same stage are defined with @VERSION. @ZF (zero flag) or @PM (privileged mode) can be used to make that version only apply when those flags are set.

## Generating Instruction Tables
Instruction tables contain data to write to EEPROMs or flash chips (or any persistent storage for that matter) from the CPU's CU to decode the commands.
Make sure you are in an empty directory before executing the command as it clutters the PWD.
This command gets invoked by `smiscvm` (during the build process), too.

You can generate them with the `--generate-instruction-table` flag.


## Actually Assembling
To run *smiscasm* with its standard functionality (assembling & linking), just run:  
```smiscasm my_code.s```

## Exit Codes
`smiscasm` has seperate exit codes for different kinds of errors.  
Additionally, rusts typical *101* might also appear (in which case, try rebuilding using *production.sh* in case you used *build.sh* previously, not at all guarenteed to work though).

| CODE | KIND        | DESCRIPTION                                                               |  
|------|-------------|---------------------------------------------------------------------------|  
| x00  | Arguments   | An input argument you wrote before execution is faulty                    |
| x03  | Input Files | An **implicit** input file (config file, instructions, etc.) is faulty    |
| x04  | Read/Write  | A file couldn't be opened/written. Check space left on disk & permissions |
| x05  | Input Code  | The code that should be assembled contains an error                       |
| x98  | Other       | Uncategorized error                                                       |
| x99  | Internal    | Internal issue; sorry. Please file a report                               |

The x means 1...9, usually it'll be a **1**. The other numbers are only there for debugging in case one issue might arise from different points in the code.

## Further Info
Run smiscasm with the `--help` flag for more info.  
You can also use help with other flags to get help for those flags (like `smiscasm --help --generate-instruction-table`).  
The `--instruction-help` command can give info on specific assembly instructions (like `smiscasm --instruction-help add`).
