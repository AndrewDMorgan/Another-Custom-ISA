# Another-Custom-ISA
Another custom ISA, this time an 8-bit RISC style one targeted for a possible Minecraft computer build.

The documention is here:

https://docs.google.com/spreadsheets/d/1EU0HqW1YHJIZ-7ZNyb768a9Qf3rkegkdOBP46fiWrRU/edit?usp=sharing

It includes both the general planned layout, and the individual instructions for the ISA (including the binary encoding for those instructions)


The assembler supports headers (can be invoked under a couple names) and macros.
* Macros are created by doing:
```
!macro macro_name arg1 arg2 arg3    ...(further args listed)   note that comments can't go on this line or it'll use them as args
    ; code can go here
    ; simply use the arg name and it'll be replaced when the macro is expanded:
    Ldi arg1    ; no special tokens, like a $, #, or {...} are necessary here
!end  ; ends the macro

; you could probably use valid operations as the name for args, or use an arg for the instruction, and it'd probably word
; this does seem to work, but wasn't entirely intentional, but should work, but if it doesn't it's not my fault
!macro other_macro_name rda arg2 Ldi
    Ldi rda arg2   ; this assumes Ldi is a valid instruction and is being provided valid inputs
!end

; or something like

!macro other_other_macro_name instruction_name_arg instruction_arg1 instruction_arg2
    ; note that whitespace is purely syntax sugar and doesn't matter, it really is only for readability here (and is recommended for that reason)
    instruction_name_arg instruction_arg1 instruction_arg_2
!end

; macros can be defined anywhere in the code (within reason.... duh, don't put it as an instruction's arg or it'll probably just crash)
; the parser simply splices them out before doing any further parsing, so they won't mess up byte indexes, or anything like that

; macros can also have -export added after !macro to make them globally accessible (i.e. in any page of the program; by default they only are accessible in the page they're created in, and local names across file don't collide when renammed):
!macro -export global_macro arg1
    ; macro code goes here
!end
```
* Headers can be defined by doing any of the following (the different names all operate the same, and are really just syntax sugar for the same thing):
```
!header header_name

!loop loop_name

; note that this is different from the !end used in !macro as that is spliced out before headers are parsed
; note that header names, regardless of the name used for generating it, cannot overlap, and they also can't overlap with !page names (main is used as the default for the first page, and is therefore reserved)
!end header_end_name

; anywhere the header is used, the parser automatically expands it into the raw byte index
; so, you could do:
Ldi rda header_name   ; and this is valid, as it's loading the line the header originates on
```
* Comments are done as following:
```
; Semi-colons in traditional assembly fassion
# Python style comments
// Typical Comments
/* Long comments (only works for one line though) */
Or any non-valid instruction keyword for starting the line, i.e.:       ; fyi, this line in of itself is not a valid comment, as the first word, 'Or', is an actual operation name
    hello world    ; valid (the parser ignores this line as it's invalid)
    Ldi rda        ; invalid (would be parsed as an instruction)
    Ldi register   ; invalid (would be parsed as an instruction, and would crash due to an invalid register name)
; Tokens are broken up purely by spaces, unlike in many programming languages where most symbols, like +, -, etc. also break up tokens

; in simple words, the parser skips non-valid operation keywords (only looking at the first word/token of the line)
```
* Pages are another feature, although they're a bit more odd. Pages are to handle the fact that the ISA is 8-bit, but 256 instructions is hardly enough for most more complex applications. The solution, divide the read-only program memory into 256 pages of 256 instructions, providing 16-bits worth of instructions, a much more reasonable number. The downside, you have to manually index the pages, as you can't simply go to a 16-bit value from 8-bit registers and busses.
```
; the initial/main program goes here (this is where the computer begins reading; page 0 aka page 'main')

; page names can't overlap with header names, keep them seperate
!page PageName    ; this dictates a new page; the inital page is 'main' and where the program starts. Pages are sequential, so this page would be page 1 (the name is just an allias for the index, to simplify the code)

; more code can go here

; to jump between pages, there are currently four instructions: SetPage, Goto, SetPageReg, and GotoReg
Goto page_name/page_index header_name/line_number
SetPage 1   ; next time a Jmp instruction/branch instruction is used, this page will be appended to the new line number. However, it will only chage pages upon branching, until then it continues in the current page
GotoReg page_name/page_index register   ; the register contains the line number to jump to (useful for a function return with a known page, but unknown line number)
SetPageReg register    ; the register contains the page number (useful for function returns that return to an unknown page)
; if SetPageReg is used, it could than be used in combination with JmpR to jump to an arbitrary line and an arbitary page (the SetPage changes the page upon branching, but the branch still changes the line it jumps to within the given page)
```
* Flags are an essential feature of an ISA, as they allow for conditional branching. Below is information on the flags and nuances:
```
; the first flag, and the one used in all conditional branches, is the condition flag
; the condition flag is set by certain ALU operations, or logical operations (check the google sheets for more info)
; however, those same operations do NOT reset the flag, so something like `RsetC` would be necessary to correctly reset it after using it

; the other flag is the overflow flag
; the overflow flag doesn't directly act upon any conditional branching
; however, the `OvrFlow` instruction sets the condition flag to the overflow flag's current state, allowing it to be used
; also check the documention for the cases in which this happens
; this flag also does not reset, similar to the condition flag, but `RsetO` can be used to reset it to false
```
* The ALU is slightly different than many systems, in that you can't call an operation, like add, while providing where to gather the data:
```
; you may notice that the ALU operations have no arguments:
Add   ; this is the full instruciton
; that is because it instead uses the dedicated alu_left and alu_right registers, and it outputs to the alu_out register
; the alu_left/right registers cannot be written back to memory, so to retrieve the values, you'd need to use `ThruL` or `ThruR` to push it into the output
; alu_out can be pushed to memory, but not straight into the left/right alu registers
; example for subtracting registers rda from rdb
LodL rdb   ; moving rdb to alu left
LodR rda   ; moving rda to alu right
Sub        ; left - right, so rdb - rda; similar to the Add instruction, there are no arguments provided for this operation
WrtO rdc   ; storing the output from alu_out in teh rdc register
; while this can bloat binary sizes, it also makes the instruction set far simpler at the hardware level, allowing for more efficent compution
; for that same reason of simplicity, you also cannot write a register into another register directly, but instead need to use the alu input registers, and move it into the output before storing it again
```
* Example (below is the fibonacci sequence):
```
Ldi rda 0  ; previous result
Ldi rdb 1  ; current result
PshCon 0   ; the stack is purely for visualization
PshCon 1   ; this is just pushing the 0 and 1 onto it, which is pre-loaded into rda and rdb
!loop FibLoop
    LodL rda  ; Loading the previous result
    LodR rdb  ; Loading the current result
    Add       ; Adding them to get F(n+1)
    WrtO rdb  ; Writing the result to rdb, which is the register holding the current result
    PshO      ; Pushing the result to the stack for visualization
    ThruR     ; Brining the right alu register into the alu output register (the alu_right reg currently stores the previous rdb, before it was overwritten)
    WrtO rda  ; Writing the output (aka the old rdb, i.e. the previous result) to rda which represents the new previous result
    OvrFlow   ; setting the condition flag to the overflow flag's state
    Jnz FibLoop  ; jumping if the condition flag is true, aka if an overflow happened
Kill  ; ending the program
```
