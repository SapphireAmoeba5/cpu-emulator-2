# Overview
This is the assembler for the custom architecture. 
It uses a syntax very similar to Intel syntax in x86 assembly, 
but with some minor architecture specific differences

Currently very heavily WIP
## Basic example:

```asm
; This is required. It will tell the assembler to create a new "container" to emit bytes into. 
; All sections are fully isolated from other sections to allow them to be relocated as necessary by the linker.
; In the future you will be able to write a custom linker script to describe the layout of your program.
; By default the `.entry` section will always be placed first in the final linked program,
; And all other sections are placed after in a non-determinate order (using the `*` syntax)
.section .entry 

; Move the immediate value `0` into the register `r0`
mov r0, 100

; Save the location of the next bytes to be emitted in the symbol `_start`
_start: 

; Subtact the immediate value `1` from `r0`, equivalent to r0 = r0 - 1
sub r0, 1 

; Conditionally jump back up to `_start` if the result of the previous calculation was not zero
jnz _start
```
