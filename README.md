# Another-Custom-ISA
Another custom ISA, this time an 8-bit RISC style one targeted for a possible Minecraft computer build.

The computer is 8-bits, but has 16 bit program addressing using a paging system. There's also a custom emmulator and assembler built for it in rust, which supports macros, comments, and much, much more.

The documention is here:

https://docs.google.com/spreadsheets/d/1EU0HqW1YHJIZ-7ZNyb768a9Qf3rkegkdOBP46fiWrRU/edit?usp=sharing

It includes both the general planned layout, and the individual instructions for the ISA (including the binary encoding for those instructions)
Currenty, the emulator appears to be running a couple hundred million instructions per second, despite being interpreted, not Jit. Unless I messed up the timing, idk. Of coruse, that only matters so much as for a minecraft computer I'll be lucky to get even 0.5 seconds per instruction.

# Information on the Assembly Language/Assembler

The assembler supports headers (can be invoked under a couple names), pages, macros, and more.
* Macros are created by doing:
```
!macro macro_name arg1 arg2 arg3    ...(further args listed)   note that comments can't go on this line or it'll use them as args (so this is an invalid comment/macro)
    ; code can go here
    ; simply use the arg name and it'll be replaced when the macro is expanded:
    Ldi arg1    ; no special tokens, like a $, #, or {...} are necessary here
!end  ; ends the macro

; you could probably use valid operations as the name for args, or use an arg for the instruction, and it'd probably work
; this does seem to work, but wasn't entirely intentional, but should work, but if it doesn't it's not my fault
!macro other_macro_name rda arg2 Ldi
    ; this assumes Ldi is a valid instruction and is being provided valid inputs
    ; if that contract is broken, it'll either crash (failing to parse a non-integer into an integer) or ignore it as a comment (more on comments below)
    Ldi rda arg2
!end

; or something like

!macro other_other_macro_name instruction_name_arg instruction_arg1 instruction_arg2
    ; note that whitespace-based indentation is purely syntax sugar and doesn't matter, it really is only for readability here (and is recommended for that reason)
    instruction_name_arg instruction_arg1 instruction_arg_2
!end

; macros can be defined anywhere in the code (within reason.... duh, don't put it as an instruction's arg or it'll probably just crash)
; the parser simply splices them out before doing any further parsing, so they won't mess up byte indexes, or anything like that

; macros can also have -export added after !macro to make them globally accessible (i.e. in any page of the program; by default they only are accessible in the page they're created in, and local names across file don't collide when renammed):
!macro -export global_macro arg1
    ; macro code goes here
!end

; macros can also use other macros inside themselves, and the expansion will correctly expand it all
; however, if you have a macro calling itself inside the definition, it will result in the application freezing as it tries to infinitely expand the macro
!macro using_global_macro
    ; using the global macro defined above inside the macro
    global_macro arg1  ; local macros defined within the page could also be used here
!end

; note: you cannot create headers or pages within macros, as everytime it gets expanded, it'll mention the header
; with one exception though (as always):
; (unrelated) technically speaking, since the parser only divides tokens by space, you can include symbols and other things in the names
!macro new_mac! header_arg_name
    !header header_arg_name    ; this works as long as each time it's used it's given a unique header name; similar to before, the parser replaces the argument before doing any further parsing
    ; you could also use !loop, or other alias's for !header, but you cannot use !page, as the pages are created before macro expansion, and as such will both cut the macro in-half, and incorrectly parse everything
!end

; Note:

; Global macros are added in order as they're seen
; As such, if you call a global macro which is defined in a later page, you cannot access it
; However, if a macro is defined after a given line, but on the same page, that is valid
; The parser goes page by page, and for a given page first locates and slices any macros, than expands any mentions of it
; This means a macro can be defined anywhere on a page, but can only be used in pages at or beyond the current page
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

; Pages expand into their raw numerical index, with main being index 0. They can be used as a constant number, such as the following:

PshCon main
Ldi rda my_page
```
* Miscellaneous:
```
; Registers, similar to headers and pages, expand into a single raw constant representing their index
; By default, the registers are 'rda' - 'rdp'.
; 'rda' expands to 0, and as such could be used as following (an example, but can be used in other ways):

PshCon rda

; because teh registers expand into raw numbers, you can also technically avoid writing them out, and instead place the raw index instead (although it makes it harder to read):

Ldi 0 0   ; loading 0 into register 0 (rda)

; Technically, while not advised, you could do some odd things, such as the following, based on how the expansion works:

Ldi main header_name
; or
Ldi header_name main

; where header_name/main acts as the register index
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
* Current ABI:
```
; there isn't really an "offical" ABI, but the one I've been using in my scripts is as follows (simply using these macros will automatically make it work, unless you mess with the stack)

; calls a given function (header/line index and page index/name)
!macro -export call function_arg page_arg current_page
    LdiR 5  ; loading 5 to the right alu register, to add to the line number (5 instructions are added after this
    PgcL    ; the left register contains the current line
    Add     ; adding the two together
    PshO    ; pushing the result onto the stack
    PshCon current_page  ; pushing the current page onto the stack
    Goto function_arg page_arg   ; this should actually jump to the function
    ; it should jump to here upon exit (i.e. the next line, which is outside this macro)
!end

; returns from a function, assuming the return address is on the top of the stack (page than line)
!macro -export ret
    TopL      ; load the left alu register with the top of the stack
    Pop       ; removing the top element (TopL doesn't pop, and pop doesn't return a value, so both are needed)
    ThruL     ; loading the return page into alu_out to get it into a register
    WrtO rda  ; writing the output to rda to use for setting the page
    SetPageReg rda   ; setting the page register to the return page
    ; repeating, but for the line number
    TopL      ; load the left alu register with the top of the stack
    Pop       ; removing the top element (TopL doesn't pop, and pop doesn't return a value, so both are needed)
    ThruL     ; loading the return address into alu_out to get it into a register
    WrtO rda  ; writing the output to rda to use for jumping
    JmpR rda  ; returning/jumping to the callers address
!end

; Note:

; Because "ret" uses rda, any arguments I try to pass in using rdb and onwards instead (you could also try using the stack, or ram)
; If using the stack for arguments, be very careful as the order will get messed up because the page and line are pushed afterwards
;   You will need to pop them but then push them back onto the stack for ret to correctly work (when possible, it's best to just
;   use registers rdb and beyond for arguments and returns to avoid these complex errors)

; This ABI does work for recursion, with the only complex part being arguments. If the arguments reference the same source, it's easy,
; just use the same register. However, if they need unique data each iteration, that's where you either need a pointer system (to ram,
; disc, registers, or whatever), or to use the stack. Ideally, you try to avoid using the stack, or tweak the algerithm, but there may be no alternatives.
```
* Formatting:
```
; I try to increment in sets of 4 spaces, but use an extra 2 spaces when there's a condition that jumps past a region, instead of a conditional that jumps to, which marks that it's part of the conditional:

!header my_function
    OvrFlow   ; some condition goes here
    Jnz header_name
      ; conditional code goes here, slightly indented
      Ldi rdb 0
  !header header_name     ; this is slightly indented backwards to mark it's not a seperate scope, but rather a branch within the scope
    ; continuation of normal code
    Ldi rda 0

; CamelCase, snake_case, or other naming convensions are all fine

; When a macro takes in a header name, I try to pass a very explict name to it to prevent namespace collisions:
my_macro MyMacro_HeaderName_Task_0
; I try to include the task it's doing, possibly the macro's name, and a post fix number, which is incremented each spot it's used to prevent collisions
my_macro MyMacro_HeaderName_Task_1    ; used again, but no collisions. And, with the very explicit and complex name, it's highly unlikely you'll simply redefine it elsewhere by accident
```

## Progress In Minecraft

### 11/24:
 Developed a 6-bit RGB screen inspired by the design of https://www.youtube.com/watch?v=USH-PME_rls&t=140s. The screen uses 3 sr-latches per pixel for each color channel, storing them as signal strengths. Trapdoors open/close to present different colors, with the number of trapdoors open being tied to the stored color. A 2D selector still needs to be added onto it so that 2 binary numbers can index into an arbitrary pixel.

## Other Progress

### Substantial Progress In Logisim-Evolution (12/2):

Note: There are some slight differences with the behavior between the planned MC version, and this circuit. However, these should only be in edge cases, and the overall function should be the same. Any inconsistencies arise mostly from the differences between redstone and actual circuitry.

![Screenshot Of Circuit (WIP)](https://github.com/AndrewDMorgan/Another-Custom-ISA/blob/main/circuit.png)

### It's Alive! (12/3):

The CPU can now correctly run the screen program. Paging, multi-page jumps, stack behavior, and some other instructions/operations haven't yet been tested and likely still contain bugs.

![Screenshot Of Circuit (It's Alive!)](https://github.com/AndrewDMorgan/Another-Custom-ISA/blob/main/circuit_working.png)
