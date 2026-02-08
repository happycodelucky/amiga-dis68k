/// Tests for 68020+ bit field instructions:
/// BFTST, BFEXTU, BFCHG, BFEXTS, BFCLR, BFFFO, BFSET, BFINS

use dis68k::m68k::decode::decode_instruction;
use dis68k::m68k::variants::CpuVariant;
use dis68k::m68k::instruction::{Instruction, Mnemonic, Operand, BitFieldParam};
use dis68k::m68k::addressing::EffectiveAddress;
use dis68k::output::formatter::{format_instruction, FormatOptions};

fn decode_68000(bytes: &[u8]) -> Instruction {
    decode_instruction(bytes, 0, 0, CpuVariant::M68000).unwrap()
}

fn decode_68020(bytes: &[u8]) -> Instruction {
    decode_instruction(bytes, 0, 0, CpuVariant::M68020).unwrap()
}

// ─── BFTST Tests ─────────────────────────────────────────────────

#[test]
fn test_bftst_immediate_offset_width() {
    // BFTST D0{4:12}: opcode 0xE8C0, ext 0x010C
    let bytes = [0xE8, 0xC0, 0x01, 0x0C];
    let inst = decode_68020(&bytes);

    assert_eq!(inst.mnemonic, Mnemonic::Bftst);
    assert_eq!(inst.size_bytes, 4);
    assert_eq!(inst.cpu_required, CpuVariant::M68020);
    assert_eq!(inst.operands.len(), 2);
    assert_eq!(inst.operands[0], Operand::Ea(EffectiveAddress::DataDirect(0)));
    assert_eq!(inst.operands[1], Operand::BitField {
        offset: BitFieldParam::Immediate(4),
        width: BitFieldParam::Immediate(12),
    });
}

#[test]
fn test_bftst_register_offset_width() {
    // BFTST D0{d1:d2}: opcode 0xE8C0, ext 0x0862
    let bytes = [0xE8, 0xC0, 0x08, 0x62];
    let inst = decode_68020(&bytes);

    assert_eq!(inst.mnemonic, Mnemonic::Bftst);
    assert_eq!(inst.operands[1], Operand::BitField {
        offset: BitFieldParam::Register(1),
        width: BitFieldParam::Register(2),
    });
}

#[test]
fn test_bftst_rejected_on_68000() {
    let bytes = [0xE8, 0xC0, 0x01, 0x0C];
    let inst = decode_68000(&bytes);
    assert_eq!(inst.mnemonic, Mnemonic::Dc);
}

// ─── BFEXTU Tests ────────────────────────────────────────────────

#[test]
fn test_bfextu_d0_to_d3() {
    // BFEXTU D0{4:8},D3: opcode 0xE9C0, ext 0x3108
    let bytes = [0xE9, 0xC0, 0x31, 0x08];
    let inst = decode_68020(&bytes);

    assert_eq!(inst.mnemonic, Mnemonic::Bfextu);
    assert_eq!(inst.operands.len(), 3);
    assert_eq!(inst.operands[0], Operand::Ea(EffectiveAddress::DataDirect(0)));
    assert_eq!(inst.operands[1], Operand::BitField {
        offset: BitFieldParam::Immediate(4),
        width: BitFieldParam::Immediate(8),
    });
    assert_eq!(inst.operands[2], Operand::Ea(EffectiveAddress::DataDirect(3)));
}

// ─── BFCHG Tests ─────────────────────────────────────────────────

#[test]
fn test_bfchg_width_zero_means_32() {
    // BFCHG D0{0:0}: opcode 0xEAC0, ext 0x0000
    // Width field of 0 means 32
    let bytes = [0xEA, 0xC0, 0x00, 0x00];
    let inst = decode_68020(&bytes);

    assert_eq!(inst.mnemonic, Mnemonic::Bfchg);
    assert_eq!(inst.operands[1], Operand::BitField {
        offset: BitFieldParam::Immediate(0),
        width: BitFieldParam::Immediate(0),  // 0 encodes as 32
    });
}

// ─── BFEXTS Tests ────────────────────────────────────────────────

#[test]
fn test_bfexts() {
    // BFEXTS D0{4:8},D3: opcode 0xEBC0, ext 0x3108
    let bytes = [0xEB, 0xC0, 0x31, 0x08];
    let inst = decode_68020(&bytes);
    assert_eq!(inst.mnemonic, Mnemonic::Bfexts);
}

// ─── BFCLR Tests ─────────────────────────────────────────────────

#[test]
fn test_bfclr() {
    // BFCLR D0{4:12}: opcode 0xECC0, ext 0x010C
    let bytes = [0xEC, 0xC0, 0x01, 0x0C];
    let inst = decode_68020(&bytes);
    assert_eq!(inst.mnemonic, Mnemonic::Bfclr);
}

// ─── BFFFO Tests ─────────────────────────────────────────────────

#[test]
fn test_bfffo() {
    // BFFFO D0{d2:8},D4: opcode 0xEDC0, ext 0x4888
    let bytes = [0xED, 0xC0, 0x48, 0x88];
    let inst = decode_68020(&bytes);

    assert_eq!(inst.mnemonic, Mnemonic::Bfffo);
    assert_eq!(inst.operands[1], Operand::BitField {
        offset: BitFieldParam::Register(2),
        width: BitFieldParam::Immediate(8),
    });
    assert_eq!(inst.operands[2], Operand::Ea(EffectiveAddress::DataDirect(4)));
}

// ─── BFSET Tests ─────────────────────────────────────────────────

#[test]
fn test_bfset() {
    // BFSET D0{4:12}: opcode 0xEEC0, ext 0x010C
    let bytes = [0xEE, 0xC0, 0x01, 0x0C];
    let inst = decode_68020(&bytes);
    assert_eq!(inst.mnemonic, Mnemonic::Bfset);
}

// ─── BFINS Tests ─────────────────────────────────────────────────

#[test]
fn test_bfins_dn_source_first() {
    // BFINS D5,D0{0:16}: opcode 0xEFC0, ext 0x5010
    let bytes = [0xEF, 0xC0, 0x50, 0x10];
    let inst = decode_68020(&bytes);

    assert_eq!(inst.mnemonic, Mnemonic::Bfins);
    assert_eq!(inst.operands.len(), 3);
    // BFINS has source register first
    assert_eq!(inst.operands[0], Operand::Ea(EffectiveAddress::DataDirect(5)));
    assert_eq!(inst.operands[1], Operand::Ea(EffectiveAddress::DataDirect(0)));
    assert_eq!(inst.operands[2], Operand::BitField {
        offset: BitFieldParam::Immediate(0),
        width: BitFieldParam::Immediate(16),
    });
}

// ─── Formatter Tests ─────────────────────────────────────────────

#[test]
fn test_format_bftst() {
    let bytes = [0xE8, 0xC0, 0x01, 0x0C];
    let inst = decode_68020(&bytes);
    let formatted = format_instruction(&inst, &FormatOptions::default());

    assert_eq!(formatted.mnemonic, "bftst");
    assert_eq!(formatted.operands, "d0{4:12}");
}

#[test]
fn test_format_bfextu() {
    let bytes = [0xE9, 0xC0, 0x31, 0x08];
    let inst = decode_68020(&bytes);
    let formatted = format_instruction(&inst, &FormatOptions::default());

    assert_eq!(formatted.mnemonic, "bfextu");
    assert_eq!(formatted.operands, "d0{4:8},d3");
}

#[test]
fn test_format_bfins() {
    let bytes = [0xEF, 0xC0, 0x50, 0x10];
    let inst = decode_68020(&bytes);
    let formatted = format_instruction(&inst, &FormatOptions::default());

    assert_eq!(formatted.mnemonic, "bfins");
    assert_eq!(formatted.operands, "d5,d0{0:16}");
}

#[test]
fn test_format_bftst_register_params() {
    let bytes = [0xE8, 0xC0, 0x08, 0x62];
    let inst = decode_68020(&bytes);
    let formatted = format_instruction(&inst, &FormatOptions::default());

    assert_eq!(formatted.operands, "d0{d1:d2}");
}
