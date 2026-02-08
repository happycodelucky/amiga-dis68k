/// Tests for 68020+ extended addressing modes:
/// - Scaled indexing (x1, x2, x4, x8)
/// - Base displacement (word and long)
/// - Memory indirect pre/post-indexed
/// - Base register suppress
/// - Index suppress

use dis68k::m68k::decode::decode_instruction;
use dis68k::m68k::variants::CpuVariant;
use dis68k::m68k::instruction::{Instruction, Mnemonic, Operand};
use dis68k::m68k::addressing::EffectiveAddress;
use dis68k::output::formatter::{format_instruction, FormatOptions};

fn decode_68020(bytes: &[u8]) -> Instruction {
    decode_instruction(bytes, 0, 0, CpuVariant::M68020).unwrap()
}

// ─── Scaled Index Tests ──────────────────────────────────────────────

#[test]
fn test_scaled_index_x2() {
    // LEA (0,A0,D0.l*2),A1: 0x43F0 (LEA) followed by full extension word
    // Full extension word format:
    //   Bits 15-12: 0000 (D0)
    //   Bit 11: 1 (long size)
    //   Bits 10-9: 01 (scale x2)
    //   Bit 8: 1 (full format)
    //   Bit 7: 0 (base not suppressed)
    //   Bit 6: 0 (index not suppressed)
    //   Bits 5-4: 01 (BD null)
    //   Bits 2-0: 000 (no memory indirect)
    // = 0000_1011_0001_0000 = 0x0B10
    let bytes = [
        0x43, 0xF0,  // LEA (xxx,A0,xxx),A1
        0x0B, 0x10,  // Full extension: D0.l*2, null BD, no indirect
    ];
    let inst = decode_68020(&bytes);

    assert_eq!(inst.mnemonic, Mnemonic::Lea);
    assert_eq!(inst.size_bytes, 4);
    assert_eq!(inst.cpu_required, CpuVariant::M68020);

    // Verify it's a base displacement EA
    match &inst.operands[0] {
        Operand::Ea(EffectiveAddress::AddressBaseDisplacement {
            reg, base_disp, index_reg, index_size, scale
        }) => {
            assert_eq!(*reg, 0);
            assert_eq!(*base_disp, 0);
            assert!(index_reg.is_some());
            assert!(index_size.is_some());
            assert_eq!(*scale, 2);
        }
        _ => panic!("Expected AddressBaseDisplacement, got {:?}", inst.operands[0]),
    }
}

#[test]
fn test_scaled_index_x4() {
    // LEA (0,A0,D1.l*4),A1
    // Full extension word with scale=10 (x4)
    // = 0001_1101_0001_0000 = 0x1D10
    let bytes = [
        0x43, 0xF0,  // LEA
        0x1D, 0x10,  // D1.l*4, null BD
    ];
    let inst = decode_68020(&bytes);

    match &inst.operands[0] {
        Operand::Ea(EffectiveAddress::AddressBaseDisplacement { scale, .. }) => {
            assert_eq!(*scale, 4);
        }
        _ => panic!("Expected AddressBaseDisplacement"),
    }
}

#[test]
fn test_scaled_index_x8() {
    // LEA (0,A0,D2.l*8),A1
    // Full extension word with scale=11 (x8)
    // = 0010_1111_0001_0000 = 0x2F10
    let bytes = [
        0x43, 0xF0,  // LEA
        0x2F, 0x10,  // D2.l*8, null BD
    ];
    let inst = decode_68020(&bytes);

    match &inst.operands[0] {
        Operand::Ea(EffectiveAddress::AddressBaseDisplacement { scale, .. }) => {
            assert_eq!(*scale, 8);
        }
        _ => panic!("Expected AddressBaseDisplacement"),
    }
}

// ─── Base Displacement Tests ─────────────────────────────────────────

#[test]
fn test_base_displacement_word() {
    // LEA (100,A0,D0.l),A1
    // Full extension word with BD size=10 (word), followed by word BD
    // = 0000_1001_0010_0000 = 0x0920
    let bytes = [
        0x43, 0xF0,  // LEA
        0x09, 0x20,  // D0.l, word BD
        0x00, 0x64,  // BD = 100
    ];
    let inst = decode_68020(&bytes);

    assert_eq!(inst.size_bytes, 6);
    match &inst.operands[0] {
        Operand::Ea(EffectiveAddress::AddressBaseDisplacement {
            base_disp, ..
        }) => {
            assert_eq!(*base_disp, 100);
        }
        _ => panic!("Expected AddressBaseDisplacement"),
    }
}

#[test]
fn test_base_displacement_long() {
    // LEA (0x12345678,A0,D0.l),A1
    // Full extension word with BD size=11 (long)
    // = 0000_1001_0011_0000 = 0x0930
    let bytes = [
        0x43, 0xF0,        // LEA
        0x09, 0x30,        // D0.l, long BD
        0x12, 0x34, 0x56, 0x78,  // BD = 0x12345678
    ];
    let inst = decode_68020(&bytes);

    assert_eq!(inst.size_bytes, 8);
    match &inst.operands[0] {
        Operand::Ea(EffectiveAddress::AddressBaseDisplacement {
            base_disp, ..
        }) => {
            assert_eq!(*base_disp, 0x12345678u32 as i32);
        }
        _ => panic!("Expected AddressBaseDisplacement"),
    }
}

// ─── Index Suppress Tests ────────────────────────────────────────────

#[test]
fn test_index_suppress() {
    // LEA (100,A0),A1 — no index register
    // Full extension word with IS=1 (index suppress), BD size=10 (word)
    // = 0000_1001_0110_0000 = 0x0960
    let bytes = [
        0x43, 0xF0,  // LEA
        0x09, 0x60,  // Index suppressed, word BD
        0x00, 0x64,  // BD = 100
    ];
    let inst = decode_68020(&bytes);

    match &inst.operands[0] {
        Operand::Ea(EffectiveAddress::AddressBaseDisplacement {
            index_reg, base_disp, ..
        }) => {
            assert!(index_reg.is_none());
            assert_eq!(*base_disp, 100);
        }
        _ => panic!("Expected AddressBaseDisplacement with no index"),
    }
}

// ─── Memory Indirect Tests ───────────────────────────────────────────

#[test]
fn test_memory_indirect_preindexed() {
    // MOVE.L ([100,A0,D0.l],200),D1
    // Opcode: 0x2230 (MOVE.L from EA mode 6 reg 0 to D1)
    // Full extension word: pre-indexed, word BD, word OD
    // I/IS bits = 010 (pre-indexed, word OD)
    // = 0000_1001_0010_0010 = 0x0922
    let bytes = [
        0x22, 0x30,  // MOVE.L (xxx,A0,xxx),D1
        0x09, 0x22,  // D0.l, BD word, pre-indexed, OD word
        0x00, 0x64,  // BD = 100
        0x00, 0xC8,  // OD = 200
    ];
    let inst = decode_68020(&bytes);

    assert_eq!(inst.mnemonic, Mnemonic::Move);
    assert_eq!(inst.size_bytes, 8);

    match &inst.operands[0] {
        Operand::Ea(EffectiveAddress::AddressMemoryIndirectPre {
            base_disp, outer_disp, ..
        }) => {
            assert_eq!(*base_disp, 100);
            assert_eq!(*outer_disp, 200);
        }
        _ => panic!("Expected AddressMemoryIndirectPre, got {:?}", inst.operands[0]),
    }
}

#[test]
fn test_memory_indirect_postindexed() {
    // MOVE.L ([100,A0],D0.l,200),D1
    // I/IS bits = 110 (post-indexed, word OD)
    // = 0000_1001_0010_0110 = 0x0926
    let bytes = [
        0x22, 0x30,  // MOVE.L
        0x09, 0x26,  // D0.l, BD word, post-indexed, OD word
        0x00, 0x64,  // BD = 100
        0x00, 0xC8,  // OD = 200
    ];
    let inst = decode_68020(&bytes);

    match &inst.operands[0] {
        Operand::Ea(EffectiveAddress::AddressMemoryIndirectPost {
            base_disp, outer_disp, ..
        }) => {
            assert_eq!(*base_disp, 100);
            assert_eq!(*outer_disp, 200);
        }
        _ => panic!("Expected AddressMemoryIndirectPost, got {:?}", inst.operands[0]),
    }
}

// ─── PC-Relative Extended Addressing Tests ──────────────────────────

#[test]
fn test_pc_base_displacement() {
    // LEA (1000,PC,D0.l*2),A0
    // Opcode: 0x41FB (LEA PC-relative mode 7 reg 3)
    // Full extension word with scale=01 (x2), BD word
    // = 0000_1011_0010_0000 = 0x0B20
    let bytes = [
        0x41, 0xFB,  // LEA (xxx,PC,xxx),A0
        0x0B, 0x20,  // D0.l*2, word BD
        0x03, 0xE8,  // BD = 1000
    ];
    let inst = decode_68020(&bytes);

    assert_eq!(inst.mnemonic, Mnemonic::Lea);
    match &inst.operands[0] {
        Operand::Ea(EffectiveAddress::PcBaseDisplacement {
            base_disp, scale, ..
        }) => {
            assert_eq!(*base_disp, 1000);
            assert_eq!(*scale, 2);
        }
        _ => panic!("Expected PcBaseDisplacement"),
    }
}

#[test]
fn test_pc_memory_indirect() {
    // LEA ([1000,PC,D0.l],2000),A0
    // Full extension: pre-indexed, BD word, OD word
    // = 0000_1001_0010_0010 = 0x0922
    let bytes = [
        0x41, 0xFB,  // LEA
        0x09, 0x22,  // D0.l, pre-indexed, BD word, OD word
        0x03, 0xE8,  // BD = 1000
        0x07, 0xD0,  // OD = 2000
    ];
    let inst = decode_68020(&bytes);

    match &inst.operands[0] {
        Operand::Ea(EffectiveAddress::PcMemoryIndirectPre {
            base_disp, outer_disp, ..
        }) => {
            assert_eq!(*base_disp, 1000);
            assert_eq!(*outer_disp, 2000);
        }
        _ => panic!("Expected PcMemoryIndirectPre"),
    }
}

// ─── Formatter Tests ─────────────────────────────────────────────────

#[test]
fn test_format_scaled_index() {
    let bytes = [0x43, 0xF0, 0x0B, 0x10];
    let inst = decode_68020(&bytes);
    let formatted = format_instruction(&inst, &FormatOptions::default());

    // Should format as: lea (0,a0,d0.l*2),a1
    assert!(formatted.mnemonic.contains("lea"));
    assert!(formatted.operands.contains("d0.l*2"));
}

#[test]
fn test_format_memory_indirect_pre() {
    let bytes = [0x22, 0x30, 0x09, 0x22, 0x00, 0x64, 0x00, 0xC8];
    let inst = decode_68020(&bytes);
    let formatted = format_instruction(&inst, &FormatOptions::default());

    // Should format as: move.l ([100,a0,d0.l],200),d1
    assert!(formatted.mnemonic.contains("move.l"));
    assert!(formatted.operands.contains("[100,a0,d0.l],200"));
}

#[test]
fn test_format_memory_indirect_post() {
    let bytes = [0x22, 0x30, 0x09, 0x26, 0x00, 0x64, 0x00, 0xC8];
    let inst = decode_68020(&bytes);
    let formatted = format_instruction(&inst, &FormatOptions::default());

    // Should format as: move.l ([100,a0],d0.l,200),d1
    assert!(formatted.mnemonic.contains("move.l"));
    assert!(formatted.operands.contains("[100,a0]"));
    assert!(formatted.operands.contains(",200"));
}

// ─── Backward Compatibility Tests ───────────────────────────────────

#[test]
fn test_brief_format_still_works() {
    // Simple 68000-style (d8,An,Xn) should still work on 68020
    // MOVE.L (4,A0,D0.w),D1: 0x2230 followed by brief extension
    // Brief extension: D0.w, disp=4: 0x0004
    let bytes = [
        0x22, 0x30,  // MOVE.L (xxx,A0),D1
        0x00, 0x04,  // Brief: D0.w, disp=4 (bit 8=0 means brief)
    ];
    let inst = decode_68020(&bytes);

    assert_eq!(inst.mnemonic, Mnemonic::Move);
    assert_eq!(inst.size_bytes, 4);

    // Should decode as brief AddressIndex, not extended addressing
    match &inst.operands[0] {
        Operand::Ea(EffectiveAddress::AddressIndex { displacement, .. }) => {
            assert_eq!(*displacement, 4);
        }
        _ => panic!("Expected AddressIndex (brief format)"),
    }
}
