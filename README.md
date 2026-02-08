# dis68k — Amiga 68k Hunk Executable Disassembler

A Rust-based disassembler for Commodore Amiga 68k executable files (hunk format). Reads compiled Amiga binaries and produces human-readable Motorola 68000 assembly output with line numbers, hex dumps, section headers, and symbol annotations.

## Features

- Parses the Amiga Hunk executable format (HUNK_CODE, HUNK_DATA, HUNK_BSS, HUNK_RELOC32, HUNK_SYMBOL, HUNK_DEBUG, and more)
- Decodes all core MC68000 instructions across all 14 addressing modes
- Motorola assembly syntax output with configurable formatting
- Line numbers, address column, hex byte dumps
- ASCII string detection in data sections
- Hunk structure inspection mode (`--hunk-info`)
- Library crate with no filesystem or network dependencies (WASM-ready design)

## Project Structure

```
dis68k/
  Cargo.toml                    # Workspace root
  crates/
    dis68k/                     # Library crate (no I/O, takes &[u8])
      src/
        lib.rs                  # Public API re-exports
        error.rs                # Unified Error enum
        hunk/                   # Amiga hunk file parser
          types.rs              # HunkFile, Hunk, Relocation, Symbol, etc.
          parser.rs             # Cursor<'a> + parse_hunk_file(&[u8])
          error.rs              # HunkError
        m68k/                   # 68k instruction decoder
          instruction.rs        # Instruction, Operand, Mnemonic, Size, Condition
          addressing.rs         # EffectiveAddress (14 addressing modes)
          decode.rs             # decode_instruction() — two-level dispatch decoder
          variants.rs           # CpuVariant enum (68000–68060)
        output/                 # Disassembly output formatting
          formatter.rs          # Instruction → Motorola syntax text
          listing.rs            # Full listing generator (walks hunks, formats output)
    dis68k-cli/                 # CLI binary
      src/
        main.rs                 # clap argument parsing, file I/O, output
  docs/
    research/                   # Technical reference documents
      amiga-hunk-format.md      # Hunk file format specification
      m68k-instruction-set.md   # 68k instruction encoding reference
      amiga-system-symbols.md   # Amiga OS LVO tables (exec, dos, etc.)
  tests/
    fixtures/
      test_startup.exe          # Synthetic 64-byte test binary
```

## Building

```sh
cargo build --release
```

The binary is produced at `target/release/dis68k`.

## Usage

```
dis68k [OPTIONS] <input-file>

Arguments:
  <input-file>              Amiga hunk executable to disassemble

Options:
  -o, --output <file>       Write output to file (default: stdout)
  -c, --cpu <variant>       CPU variant: 68000, 68010, 68020, 68030, 68040, 68060
                            (default: 68000)
      --hunk-info           Show hunk structure info only (no disassembly)
      --no-symbols          Disable Amiga OS symbol resolution
      --no-hex              Hide hex byte dump column
      --no-line-numbers     Hide line numbers
      --uppercase           Use uppercase mnemonics (MOVE instead of move)
  -v, --verbose             Show additional debug information
  -h, --help                Print help
  -V, --version             Print version
```

### Examples

Disassemble a binary:

```sh
dis68k program.exe
```

Output:

```
    1  ; Amiga Hunk Executable Disassembly
    2  ; Hunks: 1
    3
    4
    5  ; ──── SECTION hunk_0, CODE (hunk 0, 32 bytes, mem=ANY) ────
    6
    7  00000000  2C780004              movea.l  ($0004).w,a6
    8  00000004  43FA0014              lea.l    (20,pc),a1
    9  00000008  7000                  moveq    #0,d0
   10  0000000A  4EAEFDD8              jsr      (-552,a6)
   11  0000000E  2640                  movea.l  d0,a3
   12  00000010  4A80                  tst.l    d0
   13  00000012  67000006              beq      $0000001A
   14  00000016  4E75                  rts
   15  00000018  70FF                  moveq    #-1,d0
   16  0000001A  4E75                  rts
```

Inspect hunk structure without disassembly:

```sh
dis68k --hunk-info program.exe
```

```
Amiga Hunk Executable: program.exe
Hunks: 3 (first: 0, last: 2)

  Hunk  0: HUNK_CODE        mem=ANY    alloc=  2048 bytes  data=  2048 bytes
           relocations: 5 entries -> [hunk_1, hunk_2]
           symbols: 3
  Hunk  1: HUNK_DATA        mem=CHIP   alloc=   512 bytes  data=   512 bytes
  Hunk  2: HUNK_BSS         mem=ANY    alloc=  4096 bytes  data=     0 bytes
```

## Running Tests

```sh
cargo test
```

39 tests covering the hunk parser (8), instruction decoder (25), and formatter (6).

## Library Usage

The `dis68k` crate can be used independently of the CLI. All input is via `&[u8]` — no filesystem access in the library.

```rust
use dis68k::{parse_hunk_file, decode_instruction, CpuVariant};

// Parse a hunk file
let data = std::fs::read("program.exe").unwrap();
let hunk_file = parse_hunk_file(&data).unwrap();

// Decode a single instruction from raw bytes
let code = &[0x4E, 0x75]; // RTS
let inst = decode_instruction(code, 0, 0, CpuVariant::M68000).unwrap();
assert_eq!(inst.mnemonic, dis68k::Mnemonic::Rts);

// Generate a full listing
let options = dis68k::ListingOptions::default();
let listing = dis68k::generate_listing(&hunk_file, &options);
for line in &listing {
    println!("{}", line.text);
}
```

## Implementation Status

| Phase | Status | Description |
|-------|--------|-------------|
| 1. Hunk Parser | Done | Parse all common hunk types, relocations, symbols |
| 2. 68000 Decoder | Done | All core MC68000 instructions, 14 addressing modes |
| 2b. Formatter/Listing | Done | Motorola syntax, listing with addresses/hex/line numbers |
| 3. Symbol Resolution | Planned | Amiga OS LVO tables, auto-labels, relocation comments |
| 4. 68020+ Extensions | Planned | Bit fields, 32-bit mul/div, full extension words, FPU |
| 5. Advanced Analysis | Planned | Library base tracking, function detection, cross-refs |

## Architecture

The library separates concerns into four independent modules:

- **hunk** — Binary format parsing. Knows nothing about 68k instructions.
- **m68k** — Instruction decoding. Knows nothing about Amiga file formats.
- **output** — Text formatting. Converts structured data to display strings.
- **symbols** (planned) — Name resolution. Maps addresses/offsets to human-readable names.

The CLI is a thin wrapper that reads files and calls the library. This separation means the library can be reused in a WASM-based web disassembler, a GUI application, or as part of a larger analysis toolchain.

## Research Documents

Detailed technical references are in `docs/research/`:

- **amiga-hunk-format.md** — Complete hunk file format specification with all type IDs, relocation structures, symbol table encoding, overlay hunks, and memory flags
- **m68k-instruction-set.md** — 68k instruction encoding reference covering opcode grouping (bits 15-12), all 14 EA modes, extension word formats, condition codes, register set, and CPU variant differences (68000-68060)
- **amiga-system-symbols.md** — Amiga OS Library Vector Offset (LVO) tables for exec.library (~120 entries), dos.library (~60), intuition.library (~50), and graphics.library (~40)

## References

- [Motorola M68000 Family Programmer's Reference Manual](https://www.nxp.com/docs/en/reference-manual/M68000PRM.pdf)
- [Amiga Hunk File Format](http://amiga-dev.wikidot.com/file-format:hunk)
- [AmigaDOS Technical Reference Manual](https://archive.org/details/AmigaDOS_Technical_Reference_Manual_1985_Commodore)
- [resrc4](https://github.com/rolsen74/resrc4) — Existing Rust Amiga disassembler
- [Musashi](https://github.com/kstenerud/Musashi) — Comprehensive C 68k emulator/disassembler
