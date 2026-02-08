# CLAUDE.md — Project Instructions for dis68k

## What This Project Is

An Amiga 68k binary executable disassembler written in Rust. It reads Amiga hunk-format executables and produces human-readable Motorola 68000 assembly output. Structured as a workspace with a library crate (`dis68k`) and a CLI binary (`dis68k-cli`).

## Key Constraints

- **Library must have no I/O** — `dis68k` takes `&[u8]` input only. No `std::fs`, no `std::io::Read`, no `std::net`. This is intentional for future WASM targeting.
- **Big-endian everywhere** — Amiga/68k is big-endian. All binary parsing uses explicit `from_be_bytes()` / `to_be_bytes()`.
- **Test with `cargo test`** — 39 tests across hunk parser, decoder, and formatter. All must pass before committing.

## Project Structure

```
crates/dis68k/src/
  lib.rs            — public API re-exports
  error.rs          — unified Error enum wrapping module errors
  hunk/             — Amiga hunk file format parser
    types.rs        — HunkFile, Hunk, Relocation, Symbol, MemoryType, HunkType
    parser.rs       — Cursor<'a> + parse_hunk_file()
    error.rs        — HunkError (no std::io dependency)
  m68k/             — 68k instruction decoder
    instruction.rs  — Instruction, Operand, Mnemonic, Size, Condition
    addressing.rs   — EffectiveAddress enum (14 addressing modes)
    decode.rs       — decode_instruction() with two-level dispatch
    variants.rs     — CpuVariant enum
  output/           — text formatting
    formatter.rs    — Instruction → Motorola syntax
    listing.rs      — full listing generator

crates/dis68k-cli/src/
  main.rs           — clap args, file I/O, output routing
```

## Implementation Plan

See `.claude/plans/tender-finding-narwhal.md` for the full 5-phase plan.

**Current status:**
- Phase 1 (Hunk Parser): COMPLETE
- Phase 2 (68k Decoder + Formatter): COMPLETE
- Phase 3 (Symbol Resolution): NOT STARTED — next up
- Phase 4 (68020+ Extensions): NOT STARTED
- Phase 5 (Advanced Analysis): NOT STARTED

## Critical Technical Details

Read `docs/IMPLEMENTATION_NOTES.md` for detailed technical gotchas. The most important ones:

1. **MOVE destination EA is reversed** — bits 11-9=register, 8-6=mode (opposite of source EA). See `decode_move()` in `decode.rs`.
2. **Hunk type masking** — always `& 0x3FFFFFFF` before comparing to hunk type constants (upper bits are memory flags).
3. **Branch displacement** — target = `instruction_address + 2 + displacement` (PC already advanced past opcode word).
4. **ADDQ/SUBQ and shift counts** — a 3-bit data value of 0 means 8, not 0.
5. **MOVEM register list** — bit order reverses for predecrement mode `-(An)`.

## Research Documents

Detailed reference material in `docs/research/`:
- `amiga-hunk-format.md` — complete hunk file format spec
- `m68k-instruction-set.md` — 68k instruction encoding, all EA modes, CPU variants
- `amiga-system-symbols.md` — Amiga OS LVO tables (exec, dos, intuition, graphics)

## What Phase 3 Needs (Symbol Resolution)

The next phase adds:
1. `symbols/amiga.rs` — static LVO tables (from `docs/research/amiga-system-symbols.md`)
2. `symbols/resolver.rs` — `SymbolResolver` trait + `CompositeResolver` combining user symbols, LVO tables, and auto-labels
3. Auto-label generation at branch/jump targets (`loc_XXXX`)
4. Relocation site annotation as comments
5. Pass the resolver into the formatter/listing so `jsr (-552,a6)` becomes `jsr _LVOOpenLibrary(a6)`

## Commands

```sh
cargo build                    # build everything
cargo test                     # run all tests (must stay green)
cargo run -p dis68k-cli -- <file>              # disassemble
cargo run -p dis68k-cli -- --hunk-info <file>  # inspect hunk structure
```

## Test Fixtures

- `tests/fixtures/test_startup.exe` — synthetic 64-byte binary with a typical Amiga startup sequence (created by Python script, not a real compiler)
- For real Amiga binaries, place them in `tests/fixtures/` and reference via `include_bytes!`
