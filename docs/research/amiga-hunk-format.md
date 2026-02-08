# Amiga Hunk Executable File Format

## Overview

Amiga executables use the "Hunk" binary format. All multi-byte values are stored in **big-endian** byte order (Motorola 68000 native format). Data must be **longword-aligned** (4-byte) throughout the file.

## File Structure

### Magic Numbers
- **Loadable executables**: Begin with `0x000003F3` (HUNK_HEADER)
- **Object code and libraries**: Begin with `0x000003E7` (HUNK_UNIT)

### File Header (HUNK_HEADER)

| Offset | Size | Description |
|--------|------|-------------|
| 0x0000 | 4 bytes | Magic value `0x000003F3` |
| 0x0004 | 4 bytes | Library string count (typically 0) |
| 0x0008 | 4 bytes | Number of hunks |
| 0x000C | 4 bytes | First hunk index (typically 0) |
| 0x0010 | 4 bytes | Last hunk index |

After the fixed header:
1. Resident library names (null-terminated strings, terminated by empty string)
2. Table of hunk sizes (one entry per hunk)
3. The actual hunk data

### Hunk Size Encoding

Each size table entry is a 32-bit value:
- **Bits 31-30**: Memory type flags
- **Bits 29-0**: Size in **longwords** (multiply by 4 for bytes)

When memory flags = 3 (both bits set), an extended memory specification follows.

### Memory Flags

| Flag | Bit | Value | Purpose |
|------|-----|-------|---------|
| HUNKB_ADVISORY | 29 | 0x20000000 | Advisory flag (causes load failure if set) |
| HUNKB_CHIP | 30 | 0x40000000 | Must allocate in chip memory (DMA-accessible) |
| HUNKB_FAST | 31 | 0x80000000 | Prefer fast memory |

Extract memory flags: `(hunk_size & 0xC0000000) >> 29`

---

## Hunk Types

### Core Hunk Types

| Hunk Type | Decimal | Hex | Purpose |
|-----------|---------|-----|---------|
| HUNK_UNIT | 999 | 0x3E7 | Unit/module marker (object files) |
| HUNK_NAME | 1000 | 0x3E8 | Name block |
| HUNK_CODE | 1001 | 0x3E9 | Executable code section |
| HUNK_DATA | 1002 | 0x3EA | Initialized data section |
| HUNK_BSS | 1003 | 0x3EB | Uninitialized memory allocation |
| HUNK_RELOC32 | 1004 | 0x3EC | 32-bit absolute relocations |
| HUNK_RELRELOC16 | 1005 | 0x3ED | 16-bit PC-relative relocations |
| HUNK_RELRELOC8 | 1006 | 0x3EE | 8-bit PC-relative relocations |
| HUNK_EXT | 1007 | 0x3EF | External references and definitions |
| HUNK_SYMBOL | 1008 | 0x3F0 | Symbol table |
| HUNK_DEBUG | 1009 | 0x3F1 | Debug information |
| HUNK_END | 1010 | 0x3F2 | Hunk terminator (required) |
| HUNK_HEADER | 1011 | 0x3F3 | File header (loadable executables) |

### Extended Hunk Types

| Hunk Type | Decimal | Hex | Purpose |
|-----------|---------|-----|---------|
| HUNK_OVERLAY | 1013 | 0x3F5 | Overlay structure definition |
| HUNK_BREAK | 1014 | 0x3F6 | Overlay break marker |
| HUNK_DREL32 | 1015 | 0x3F7 | 32-bit data-relative relocation |
| HUNK_DREL16 | 1016 | 0x3F8 | 16-bit data-relative relocation |
| HUNK_DREL8 | 1017 | 0x3F9 | 8-bit data-relative relocation |
| HUNK_LIB | 1018 | 0x3FA | Library marker |
| HUNK_INDEX | 1019 | 0x3FB | Index hunk |
| HUNK_RELOC32SHORT | 1020 | 0x3FC | 32-bit relocations (word offsets) |
| HUNK_RELRELOC32 | 1021 | 0x3FD | 32-bit PC-relative (AmigaOS 2.0+) |
| HUNK_ABSRELOC16 | 1022 | 0x3FE | 16-bit absolute relocations |

### PowerPC Extensions

| Hunk Type | Decimal | Hex | Purpose |
|-----------|---------|-----|---------|
| HUNK_PPC_CODE | 1257 | 0x4E9 | PowerPC code section |
| HUNK_RELRELOC26 | 1260 | 0x4EC | 26-bit PC-relative (PPC) |

---

## Relocations

### HUNK_RELOC32 Structure

The most common relocation format:

```
Loop:
  Read 32-bit: Number of offsets (N)
  If N == 0: End of relocation block
  Read 32-bit: Target hunk number
  Read N x 32-bit values: Offsets within current hunk
  Repeat loop
```

**Relocation process**: For each offset in the current hunk, add the base address of the target hunk to the 32-bit value at that offset. This allows code to reference data/code in other hunks regardless of load address.

### HUNK_RELOC32SHORT

Same as HUNK_RELOC32 but uses **16-bit word offsets** instead of 32-bit (more compact). The count and target hunk number are also 16-bit. Padded to longword boundary at end.

### HUNK_RELRELOC32

Introduced in AmigaOS V39 (2.0+). Provides **PC-relative relocations** instead of absolute, allowing more position-independent code.

### HUNK_DREL32, HUNK_DREL16, HUNK_DREL8

**Data-relative relocations** with different offset sizes. Note: HUNK_DREL32 is **illegal in load files** (executables) and should only appear in object files.

---

## Symbol Tables

### HUNK_SYMBOL Structure

Symbol blocks contain a series of symbol entries:

```
Loop:
  Read 32-bit: Name length in longwords (N)
  If N == 0: End of symbol block
  Read N x 4 bytes: Symbol name (null-padded to longword boundary)
  Read 32-bit: Symbol value/offset
  Repeat
```

### HUNK_EXT (External References)

More complex than HUNK_SYMBOL, supports both definitions and references. Each entry has a type byte in the upper 8 bits of the name-length word.

#### EXT Sub-types

| Type | Value | Purpose |
|------|-------|---------|
| EXT_SYMB | 0 | Symbol table entry |
| EXT_DEF | 1 | Relocatable definition |
| EXT_ABS | 2 | Absolute definition |
| EXT_ABSREF32 | 129 | 32-bit absolute reference |
| EXT_RELREF32 | 136 | 32-bit PC-relative reference |
| EXT_ABSREF16 | 138 | 16-bit absolute reference |
| EXT_RELREF16 | 131 | 16-bit PC-relative reference |
| EXT_ABSREF8 | 139 | 8-bit absolute reference |
| EXT_RELREF8 | 132 | 8-bit PC-relative reference |
| EXT_ABSCOMMON | 130 | 32-bit common symbol (absolute) |
| EXT_RELCOMMON | 137 | 32-bit common symbol (relative) |
| EXT_DEXT32 | 133 | 32-bit data extension |
| EXT_DEXT16 | 134 | 16-bit data extension |
| EXT_DEXT8 | 135 | 8-bit data extension |
| EXT_RELREF26 | 229 | 26-bit PC-relative (PowerPC) |

---

## Debug Information

### HUNK_DEBUG Structure

```
Read 32-bit: Size in longwords (N)
Read N x 4 bytes: Debug data
```

### Common Debug Formats

**"LINE" format**: Contains filename and line number to offset mappings.

**"HCLN" format**: Variable-length encoding (1, 2, or 4 bytes) used by Devpac assembler.

As of dos.library v31, any hunk with ID > HUNK_ABSRELOC16 is treated as a debug hunk.

---

## Overlay Hunks

### HUNK_OVERLAY Structure

Used to reduce RAM requirements by loading program modules on-demand.

**Components**:
1. Tree Size: height + 1 (including root manager)
2. Tree Pointers: array tracking currently-loaded overlay nodes at each level
3. Terminator: zero longword
4. Overlay Table: 8-longword entries describing overlay references

**Overlay Table Entry (8 longwords)**:

| Field | Purpose |
|-------|---------|
| FILE_POSITION | Byte offset to module's HUNK_HEADER from file start |
| RESERVED (x2) | Unused, keep zero |
| OVERLAY_LEVEL | Tree depth (0 = root) |
| ORDINATE | Unique identifier within level |
| INITIAL_HUNK | First hunk number for module |
| SYMBOL_HUNK | Hunk containing referenced symbol |
| SYMBOL_OFFSET | Offset within hunk (add 4 for actual address) |

**Important**: Overlay executables cannot be crunched/compressed because FILE_POSITION contains absolute file offsets.

---

## Parsing Algorithm

1. Read and validate magic `0x000003F3`
2. Skip resident library names (loop reading longwords until 0)
3. Read hunk count, first_hunk, last_hunk
4. Read size table: for each hunk, extract upper 2 bits as memory type, lower 30 bits x 4 as byte size
5. For each hunk in sequence, read hunk type word (mask off `0xC0000000` memory bits), then dispatch:
   - `HUNK_CODE`/`HUNK_DATA`: read size longword, then that many longwords of data
   - `HUNK_BSS`: read size longword, store as alloc_size with empty data
   - `HUNK_RELOC32`: loop reading (count, target_hunk, offsets...) groups until count == 0
   - `HUNK_SYMBOL`: loop reading (name_len_in_longs, name_bytes, value) until name_len == 0
   - `HUNK_DEBUG`: read size, skip that many longwords
   - `HUNK_END`: finalize the current hunk, advance to next

---

## References

- [Amiga Hunk - Wikipedia](https://en.wikipedia.org/wiki/Amiga_Hunk)
- [Hunk File Format - Amiga Development](http://amiga-dev.wikidot.com/file-format:hunk)
- [AmigaDOS Technical Reference Manual (Internet Archive)](https://archive.org/details/AmigaDOS_Technical_Reference_Manual_1985_Commodore)
- [AmigaHunkParser (C)](https://github.com/emoon/AmigaHunkParser)
- [amitools (Python)](https://github.com/cnvogelg/amitools/blob/master/amitools/binfmt/hunk/Hunk.py)
- [vasm output_hunk.h](https://github.com/Leffmann/vasm/blob/master/output_hunk.h)
- [resrc4 (Rust Amiga disassembler)](https://github.com/rolsen74/resrc4)
