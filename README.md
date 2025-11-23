# Another-Custom-ISA
Another custom ISA, this time an 8-bit RISC style one targeted for a possible Minecraft computer build.

The documention is here:

https://docs.google.com/spreadsheets/d/1EU0HqW1YHJIZ-7ZNyb768a9Qf3rkegkdOBP46fiWrRU/edit?usp=sharing

It includes both the general planned layout, and the individual instructions for the ISA (including the binary encoding for those instructions)


The assembler supports headers (can be invoked under a couple names) and macros.
> Macros are created by doing:
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
```
> Headers can be defined by doing any of the following (the different names all operate the same, and are really just syntax sugar for the same thing):
```
!header header_name

!loop loop_name

; note that this is different from the !end used in !macro as that is spliced out before headers are parsed
!end header_end_name

; anywhere the header is used, the parser automatically expands it into the raw byte index
; so, you could do:
Ldi header_name   ; and this is valid, as it's loading the line the header originates on
```
> Comments are done as following:
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
```

