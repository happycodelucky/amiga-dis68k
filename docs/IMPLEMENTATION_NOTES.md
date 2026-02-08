# Implementation Notes

Technical decisions, gotchas, and patterns discovered during implementation. This document captures knowledge that isn't obvious from reading the code or the research docs.

## Hunk Parser

### Cursor Pattern

The parser uses a custom `Cursor<'a>` struct over `&[u8]` rather than `std::io::Cursor`. This avoids pulling in `std::io` traits, keeping the library portable to `no_std`/WASM. The cursor provides checked big-endian reads that return `HunkError` on bounds violations.

### Hunk Type Masking

The hunk type word carries memory flags in the upper bits. Always mask with `& 0x3FFFFFFF` before comparing to hunk type constants. The `HunkType::from_raw()` method handles this.

```
Raw word: 0x400003E9  (HUNK_CODE with CHIP memory flag)
Masked:   0x000003E9  (HUNK_CODE)
```

### HUNK_RELOC32SHORT Alignment

RELOC32SHORT uses 16-bit values for count, target hunk, and offsets. After the terminating zero count, the data must be padded to a longword (4-byte) boundary. The parser calls `cursor.align_to_longword()` after processing.

### Hunk Metadata Attachment

The file interleaves content hunks (CODE/DATA/BSS) with metadata hunks (RELOC32, SYMBOL, DEBUG). Metadata always attaches to the most recently parsed content hunk. The parser uses `hunks.last_mut()` for this. Order within the file is:

```
HUNK_CODE (or DATA or BSS)
  HUNK_RELOC32      (optional, attaches to above)
  HUNK_SYMBOL       (optional)
  HUNK_DEBUG        (optional)
HUNK_END
```

### HUNK_EXT Skipping

HUNK_EXT entries have a different structure from HUNK_SYMBOL — the upper 8 bits of the name-length word encode an entry type that determines what follows. Types < 128 are definitions (name + value), types >= 128 are references (name + count + offsets). The common symbol types (130, 137) add an extra size word. The parser currently skips HUNK_EXT entirely; Phase 3 will parse it.

## 68k Instruction Decoder

### Two-Level Dispatch

Bits 15-12 of the opcode word divide instructions into 16 groups (0x0-0xF). The decoder first switches on these 4 bits, then each group handler uses specific bit patterns to identify the instruction. This is faster than scanning a single flat table and keeps each group handler manageable.

### MOVE Destination EA Reversal

The MOVE instruction (groups 1-3) is unique: the **destination** EA field has register and mode bits **reversed** compared to every other instruction:

```
MOVE: bits 11-9 = dst register, bits 8-6 = dst mode
All other instructions: bits 5-3 = mode, bits 2-0 = register
```

This means `decode_ea(dst_mode, dst_reg, ...)` where `dst_mode` comes from bits 8-6 and `dst_reg` from bits 11-9 — the opposite of the source EA field. Getting this wrong produces subtly incorrect disassembly (e.g., confusing `(A0)` with `D0`).

### Size Encoding Variants

There are two common ways sizes are encoded:

1. **Standard 2-bit** (most instructions): `00=byte, 01=word, 10=long`
2. **MOVE encoding**: `01=byte, 11=word, 10=long` (different mapping!)

The MOVE size is encoded in bits 13-12 and decoded by the group dispatcher (group 1=byte, 2=long, 3=word).

### Effective Address Mode 7 Overloading

Mode 7 uses the register field to select among five different addressing modes:

| Register | Mode |
|----------|------|
| 0 | Absolute Short (xxx).W |
| 1 | Absolute Long (xxx).L |
| 2 | PC Displacement d16(PC) |
| 3 | PC Index d8(PC,Xn) |
| 4 | Immediate #imm |
| 5-7 | Invalid (used in 68020+ for some instructions) |

### Immediate Byte Values

When reading a byte-size immediate, the 68k still uses a full 16-bit extension word — the byte value is in the low 8 bits, high 8 bits are ignored. The decoder reads `ctx.read_u16()` and masks with `& 0xFF`.

### Branch Displacement Calculation

For Bcc/BRA/BSR, the displacement is relative to `PC + 2` (the PC has already advanced past the opcode word when the displacement is applied):

```
Target = instruction_address + 2 + displacement
```

For 8-bit displacements (short branches), the value is in the low byte of the opcode word. A value of 0 indicates a 16-bit displacement follows in the next word. On 68020+, a value of 0xFF indicates a 32-bit displacement follows.

### ADDQ/SUBQ Quick Value 0 = 8

When the 3-bit data field in ADDQ/SUBQ is 0, the actual value is 8 (not 0). Same rule applies to shift/rotate count immediates.

### MOVEM Register List Reversal for Predecrement

MOVEM has two register list formats:
- **Normal** (register-to-memory, memory-to-register): bit 0 = D0, ..., bit 7 = D7, bit 8 = A0, ..., bit 15 = A7
- **Predecrement -(An)**: bit 0 = A7, bit 1 = A6, ..., bit 7 = A0, bit 8 = D7, ..., bit 15 = D0

The formatter handles this by detecting predecrement mode and reversing the bits before formatting.

### A-Line and F-Line Traps

Groups 0xA and 0xF are reserved for A-line traps (used by Amiga for system calls via the exception mechanism) and F-line coprocessor instructions (FPU on 68020+). Both currently emit `dc.w` since they require Phase 3 (symbol resolution) and Phase 4 (FPU) respectively.

### Unknown Opcodes

When the decoder can't match an opcode, it returns the word as `Mnemonic::Dc` (data constant) with `Size::Word`. The listing generator also handles `DecodeError` by emitting `dc.w` and advancing 2 bytes, ensuring the decoder never gets stuck.

## Formatter

### A7 Display as SP

Address register 7 is the stack pointer. The formatter displays it as `sp` rather than `a7` in all contexts: `sp`, `(sp)`, `(sp)+`, `-(sp)`, `(disp,sp)`. This matches standard Amiga assembler conventions.

### Hex Prefix Convention

Motorola syntax uses `$` as the hex prefix (not `0x`). Immediate values use `#$XX`, addresses use `$XXXXXXXX`. The formatter scales hex digit count to value magnitude: `$0A` for bytes, `$1234` for words, `$00001000` for longwords.

### Branch Targets as Absolute Addresses

Branch displacements are displayed as absolute target addresses (e.g., `beq $0000001A`) rather than as relative offsets. This is more readable for the user. Once symbol resolution is added (Phase 3), these will be replaced with labels like `beq loc_001A`.

## Testing Strategy

### Decoder Tests

Each test encodes a known instruction as raw bytes and verifies the decoded `Instruction` fields. Test bytes are derived from:
- Motorola M68000 Programmer's Reference Manual opcode encodings
- Cross-referencing with existing assemblers/disassemblers (vasm, Musashi)

Example: `JSR (-552,A6)` = `0x4EAE 0xFDD8`
- `0x4EAE`: opcode word for JSR with EA mode=5 (address displacement), reg=6 (A6)
- `0xFDD8`: displacement -552 as signed 16-bit = 0xFDD8

### Test Fixture

`tests/fixtures/test_startup.exe` is a 64-byte synthetic binary created by a Python script (not a real Amiga compiler). It contains a single CODE hunk with a typical Amiga startup sequence:

```asm
movea.l ($0004).w,a6    ; load ExecBase
lea     (dosName,pc),a1 ; library name
moveq   #0,d0           ; any version
jsr     (-552,a6)       ; _LVOOpenLibrary
movea.l d0,a3           ; save base
tst.l   d0              ; check success
beq.w   .fail           ; branch if null
rts                     ; success return
moveq   #-1,d0          ; error code
rts                     ; error return
```

For testing with real Amiga binaries, place `.exe` files in `tests/fixtures/` and use `include_bytes!` in integration tests.

## Known Limitations

1. **HUNK_EXT not parsed** — External references are skipped. Needed for object files and linked executables with unresolved symbols.
2. **No symbol resolution** — `jsr (-552,a6)` is not annotated as `_LVOOpenLibrary`. Phase 3 will add this.
3. **No auto-labels** — Branch targets show as absolute addresses, not labels. Phase 3.
4. **68000 only** — 68020+ instructions (bit fields, 32-bit mul/div, full extension words) are decoded as `dc.w`. Phase 4.
5. **No FPU** — 68881/68882/68040 FPU instructions (F-line opcodes) are not decoded. Phase 4.
6. **No flow analysis** — The disassembler is linear (walks bytes sequentially). It doesn't follow jump targets or detect function boundaries. Phase 5.
7. **Relocation sites not annotated** — Data at relocation offsets should be shown as cross-hunk references. Phase 3.
