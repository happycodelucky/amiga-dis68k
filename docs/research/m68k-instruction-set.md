# Motorola 68000 Family Instruction Set Reference

## Overview

- Instructions are 2 to 10 bytes in length (1 to 5 words)
- The first 16-bit word is the opcode word, specifying the operation and addressing information
- Additional extension words provide operand details, displacements, and immediate values
- All instructions begin on even-byte boundaries (word-aligned)
- All values are big-endian

---

## Instruction Encoding

### Opcode Word Structure

The first word determines the instruction. Bits 15-12 provide the primary grouping:

| Bits 15-12 | Hex | Primary Instructions |
|------------|-----|---------------------|
| 0000 | 0 | ORI, ANDI, SUBI, ADDI, EORI, CMPI, BTST/BCHG/BCLR/BSET (static bit#), MOVEP, CMP2, CHK2, CAS, CAS2 |
| 0001 | 1 | MOVE.B |
| 0010 | 2 | MOVE.L, MOVEA.L |
| 0011 | 3 | MOVE.W, MOVEA.W |
| 0100 | 4 | Miscellaneous: NEG, NEGX, NOT, CLR, TST, TAS, EXT, SWAP, PEA, LEA, JMP, JSR, MOVEM, CHK, TRAP, LINK, UNLK, MOVE USP, RESET, NOP, STOP, RTE, RTS, TRAPV, RTR, ILLEGAL, MOVEC |
| 0101 | 5 | ADDQ, SUBQ, Scc, DBcc, TRAPcc |
| 0110 | 6 | Bcc, BRA, BSR |
| 0111 | 7 | MOVEQ |
| 1000 | 8 | OR, DIVU, DIVS, SBCD, PACK, UNPK |
| 1001 | 9 | SUB, SUBA, SUBX |
| 1010 | A | (Unassigned / A-line traps) |
| 1011 | B | CMP, CMPA, CMPM, EOR |
| 1100 | C | AND, MULU, MULS, ABCD, EXG |
| 1101 | D | ADD, ADDA, ADDX |
| 1110 | E | Shift/Rotate (ASL, ASR, LSL, LSR, ROL, ROR, ROXL, ROXR), Bit field (68020+) |
| 1111 | F | Coprocessor / F-line (FPU, MMU) |

### Decoding Strategy

Table-driven approach using mask/signature pairs:

```
(opcode & mask) == signature
```

Two-level dispatch recommended:
1. Switch on bits 15-12 to select one of 16 instruction groups
2. Linear scan within the group using mask/signature entries
3. Table ordered most-specific to least-specific masks
4. First match wins

---

## Addressing Modes

The 68000 has 14 addressing modes encoded using 6 bits: a 3-bit Mode field and a 3-bit Register field.

### Effective Address Encoding

| Mode | Register | Addressing Mode | Syntax | Extension Words |
|------|----------|----------------|--------|----------------|
| 000 | 0-7 | Data Register Direct | Dn | 0 |
| 001 | 0-7 | Address Register Direct | An | 0 |
| 010 | 0-7 | Address Register Indirect | (An) | 0 |
| 011 | 0-7 | Address Register Indirect with Postincrement | (An)+ | 0 |
| 100 | 0-7 | Address Register Indirect with Predecrement | -(An) | 0 |
| 101 | 0-7 | Address Register Indirect with Displacement | d16(An) | 1 |
| 110 | 0-7 | Address Register Indirect with Index | d8(An,Xn) | 1+ |
| 111 | 000 | Absolute Short | (xxx).W | 1 |
| 111 | 001 | Absolute Long | (xxx).L | 2 |
| 111 | 010 | PC Relative with Displacement | d16(PC) | 1 |
| 111 | 011 | PC Relative with Index | d8(PC,Xn) | 1+ |
| 111 | 100 | Immediate | #xxx | 1-2 |

### Extension Word Details

- **Immediate mode**: 1 extension word for byte/word, 2 for longword
- **Displacement modes**: 1 word for 16-bit signed displacement
- **Index modes (basic)**: 1 word encoding index register, size, and 8-bit signed displacement
- **Absolute short**: 1 word, sign-extended to 32 bits
- **Absolute long**: 2 words

### Brief Extension Word Format (Mode 110, 111/011)

```
Bit 15:    D/A (0=Data register, 1=Address register)
Bits 14-12: Index register number
Bit 11:    W/L (0=Word index, 1=Long index)
Bits 10-8:  Scale (68020+: 00=x1, 01=x2, 10=x4, 11=x8; always 00 on 68000)
Bit 8:     0 for brief format (68020+: 1 for full extension word)
Bits 7-0:  8-bit signed displacement
```

### Full Extension Word (68020+ only)

When bit 8 of the extension word is 1, this is a full extension word with much richer addressing:

```
Bit 15:    D/A
Bits 14-12: Index register number
Bit 11:    W/L
Bits 10-9:  Scale (x1, x2, x4, x8)
Bit 8:     1 (full extension word marker)
Bit 7:     BS (Base Register Suppress: 1 = suppress An)
Bit 6:     IS (Index Suppress: 1 = suppress index)
Bits 5-4:  BD Size (00=reserved, 01=null, 10=word, 11=long)
Bit 3:     0
Bits 2-0:  I/IS (Index/Indirect Selection)
```

This enables:
- **Base displacement**: `(bd,An)` or `(bd,An,Xn)` with word or longword displacement
- **Memory indirect pre-indexed**: `([bd,An,Xn],od)`
- **Memory indirect post-indexed**: `([bd,An],Xn,od)`

---

## Register Set

### Data Registers (D0-D7)
- Eight 32-bit general-purpose data registers
- Used for arithmetic, logic, and bit operations
- Accessible as byte (.B bits 0-7), word (.W bits 0-15), or longword (.L bits 0-31)

### Address Registers (A0-A7)
- Seven general-purpose address registers (A0-A6)
- A7 is the Stack Pointer (SP)
- Two A7 registers: User Stack Pointer (USP) and Supervisor Stack Pointer (SSP)
- Operations on address registers are always word or longword (never byte)

### Program Counter (PC)
- 32-bit register (only 24 bits used on 68000)
- Points to current instruction

### Status Register (SR) - 16 bits

```
Bit 15:   T1 (Trace)
Bit 14:   T0 (Trace - 68020+, reserved on 68000)
Bit 13:   S  (Supervisor/User state)
Bit 12:   M  (Master/Interrupt state - 68020+)
Bits 10-8: I2-I0 (Interrupt mask level 0-7)
Bits 7-5:  Reserved
Bit 4:    X (Extend)
Bit 3:    N (Negative)
Bit 2:    Z (Zero)
Bit 1:    V (Overflow)
Bit 0:    C (Carry)
```

Lower byte (bits 7-0) is the **Condition Code Register (CCR)** - accessible in user mode.

---

## Condition Codes

Used by Bcc, DBcc, Scc, and TRAPcc instructions:

| Code | Bits | Mnemonic | Condition | Test |
|------|------|----------|-----------|------|
| 0000 | 0 | T | True | 1 |
| 0001 | 1 | F | False | 0 |
| 0010 | 2 | HI | High | !C & !Z |
| 0011 | 3 | LS | Low or Same | C \| Z |
| 0100 | 4 | CC (HS) | Carry Clear | !C |
| 0101 | 5 | CS (LO) | Carry Set | C |
| 0110 | 6 | NE | Not Equal | !Z |
| 0111 | 7 | EQ | Equal | Z |
| 1000 | 8 | VC | Overflow Clear | !V |
| 1001 | 9 | VS | Overflow Set | V |
| 1010 | A | PL | Plus | !N |
| 1011 | B | MI | Minus | N |
| 1100 | C | GE | Greater or Equal | (N & V) \| (!N & !V) |
| 1101 | D | LT | Less Than | (N & !V) \| (!N & V) |
| 1110 | E | GT | Greater Than | (N & V & !Z) \| (!N & !V & !Z) |
| 1111 | F | LE | Less or Equal | Z \| (N & !V) \| (!N & V) |

---

## Key Instruction Groups

### Data Movement
- **MOVE** - Move data between locations (most used instruction)
- **MOVEA** - Move to address register
- **MOVEQ** - Move quick (8-bit immediate to data register, sign-extended to 32 bits)
- **MOVEM** - Move multiple registers (save/restore register sets)
- **MOVEP** - Move peripheral (alternating bytes for 8-bit I/O)
- **LEA** - Load effective address (compute address into An)
- **PEA** - Push effective address onto stack
- **EXG** - Exchange two registers
- **SWAP** - Swap upper and lower words of a data register

### Arithmetic
- **ADD/ADDA/ADDI/ADDQ/ADDX** - Addition variants
- **SUB/SUBA/SUBI/SUBQ/SUBX** - Subtraction variants
- **MULS/MULU** - Signed/unsigned multiply (16x16->32; 68020+: 32x32->32 or 32x32->64)
- **DIVS/DIVU** - Signed/unsigned divide (32/16->16q:16r; 68020+: 64/32->32)
- **NEG/NEGX** - Negate / negate with extend
- **EXT** - Sign extend (byte->word, word->long; 68020+: byte->long via EXTB.L)
- **CLR** - Clear (set to zero)
- **CMP/CMPA/CMPI/CMPM** - Compare variants
- **TST** - Test (compare against zero)

### Logic
- **AND/ANDI** - Bitwise AND
- **OR/ORI** - Bitwise OR
- **EOR/EORI** - Bitwise Exclusive OR
- **NOT** - Bitwise complement

### Shifts and Rotates
- **LSL/LSR** - Logical shift left/right (zero fill)
- **ASL/ASR** - Arithmetic shift left/right (sign-preserving right shift)
- **ROL/ROR** - Rotate left/right
- **ROXL/ROXR** - Rotate through extend bit

### Bit Manipulation
- **BTST** - Test a bit (sets Z flag)
- **BSET** - Test and set a bit
- **BCLR** - Test and clear a bit
- **BCHG** - Test and change (toggle) a bit

### BCD (Binary Coded Decimal)
- **ABCD** - Add BCD with extend
- **SBCD** - Subtract BCD with extend
- **NBCD** - Negate BCD

### Program Control
- **BRA** - Branch always (PC-relative)
- **BSR** - Branch to subroutine (PC-relative)
- **Bcc** - Conditional branch (14 conditions)
- **DBcc** - Decrement and branch (loop primitive)
- **JMP** - Jump (absolute)
- **JSR** - Jump to subroutine (absolute)
- **RTS** - Return from subroutine
- **RTE** - Return from exception (privileged)
- **RTR** - Return and restore condition codes
- **Scc** - Set byte conditionally (0xFF or 0x00)

### System Control
- **TRAP #n** - Software interrupt (vectors 32-47)
- **CHK** - Check register against bounds (trap on out of range)
- **TAS** - Test and set (atomic read-modify-write)
- **STOP** - Halt processor (privileged)
- **RESET** - Reset external devices (privileged)
- **LINK/UNLK** - Stack frame management
- **MOVE to/from SR** - Status register access (privileged for full SR)
- **MOVE to/from CCR** - Condition code register access
- **MOVE USP** - User stack pointer access (privileged)
- **ILLEGAL** - Illegal instruction trap
- **NOP** - No operation
- **TRAPV** - Trap on overflow

---

## CPU Variant Differences

### 68000 (Base)
- 16-bit external data bus, 24-bit address bus
- No virtual memory support
- All base instructions

### 68010
- Virtual memory support (instruction continuation after page fault)
- MOVE from SR made privileged (security fix)
- Added instructions:
  - **RTD** - Return and deallocate (RTS + adjust stack)
  - **MOVEC** - Move to/from control register
  - **MOVES** - Move to/from address space

### 68020
- Full 32-bit data and address buses
- Instruction cache (256 bytes)
- Coprocessor interface
- Added instructions:
  - **MULS.L/MULU.L** - 32x32->64 multiply
  - **DIVS.L/DIVU.L** - 64/32->32 divide
  - **EXTB.L** - Sign extend byte to longword
  - **Bit field operations**: BFCHG, BFCLR, BFEXTS, BFEXTU, BFFFO, BFINS, BFSET, BFTST
  - **CAS/CAS2** - Compare and swap (atomic, for multiprocessing)
  - **CHK2/CMP2** - Check/compare against bounds pair
  - **PACK/UNPK** - BCD pack/unpack
  - **TRAPcc** - Trap on condition
  - **CALLM/RTM** - Module call/return (removed in 68040)
  - **LINK.L** - Link with 32-bit displacement
- Enhanced addressing modes:
  - Scaled index (x1, x2, x4, x8)
  - Base displacement (word or longword)
  - Memory indirect with pre/post-indexing
  - 32-bit branch displacements

### 68030
- On-chip MMU and dual caches (data + instruction)
- Same instruction set as 68020
- Minor: PFLUSHA added for MMU

### 68040
- Integrated FPU
- Dual integer pipelines
- Hardware-implemented MOVE16
- Removed: CALLM, RTM
- Added: CINV, CPUSH (cache control)
- Some instructions execute differently (CLR does single write vs read-then-write)

### 68060
- Superscalar architecture (two integer pipelines)
- Branch prediction
- Some complex instructions trapped to software emulation for performance
- Further instruction removals (emulated in software)

---

## MOVE Instruction Encoding Example

The MOVE instruction demonstrates the general encoding pattern:

```
Bits 15-14: 00 (MOVE identifier)
Bits 13-12: SIZE (01=byte, 11=word, 10=long)
Bits 11-9:  Destination Register
Bits 8-6:   Destination Mode
Bits 5-3:   Source Mode
Bits 2-0:   Source Register
```

Note: For MOVE, the destination EA field has register and mode in **reversed** order compared to the source EA field. This is unique to MOVE.

---

## Variable-Length Instruction Decoding

### Algorithm

1. Read the first word (16-bit big-endian) at PC
2. Decode instruction type using bit pattern matching
3. Determine operand sizes and addressing modes from opcode bits
4. Calculate extension words needed based on addressing modes
5. Read additional words as needed
6. Total instruction length = 1 opcode word + extension words

### Extension Word Counts by EA Mode

| EA Mode | Extension Words |
|---------|----------------|
| Dn, An | 0 |
| (An), (An)+, -(An) | 0 |
| d16(An) | 1 |
| d8(An,Xn) | 1 (brief) or 1-5 (full, 68020+) |
| (xxx).W | 1 |
| (xxx).L | 2 |
| d16(PC) | 1 |
| d8(PC,Xn) | 1 (brief) or 1-5 (full, 68020+) |
| #imm | 1 (byte/word) or 2 (longword) |

---

## References

### Official Documentation
- [Motorola M68000 Family Programmer's Reference Manual (PDF)](https://www.nxp.com/docs/en/reference-manual/M68000PRM.pdf)
- [MC68000 User's Manual (PDF)](https://www.nxp.com/docs/en/reference-manual/MC68000UM.pdf)
- [M68000PRM on Internet Archive](https://archive.org/details/M68000PRM)

### Opcode Tables
- [M68k Opcodes v2.3 (PDF)](http://goldencrystal.free.fr/M68kOpcodes-v2.3.pdf)
- [68000 Instruction Set Summary](https://nguillaumin.github.io/perihelion-m68k-tutorials/appendixes/m68k-instruction-set.txt)

### Tutorials and Guides
- [MarkeyJester's 68k Tutorial](https://mrjester.hapisan.com/04_MC68/)
- [The Digital Cat - Motorola 68000 Addressing Modes](https://www.thedigitalcatonline.com/blog/2019/03/04/motorola-68000-addressing-modes/)
- [68000 Assembly - Wikibooks](https://en.wikibooks.org/wiki/68000_Assembly)

### Reference Implementations
- [Musashi (C)](https://github.com/kstenerud/Musashi) - comprehensive emulator/disassembler, used in MAME
- [m68000 Rust crate](https://crates.io/crates/m68000) - requires nightly Rust
- [resrc4 (Rust)](https://github.com/rolsen74/resrc4) - Amiga hunk disassembler
- [dis68k (C)](https://github.com/TomHarte/dis68k)
- [m68k-disasm (C)](https://github.com/Oxore/m68k-disasm)
- [BSVC decode.cpp (C++)](https://github.com/BSVC/bsvc/blob/master/src/M68k/sim68000/decode.cpp) - educational table-driven decoder
