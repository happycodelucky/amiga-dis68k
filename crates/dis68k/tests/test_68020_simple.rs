/// Tests for simple 68020 instructions: EXTB.L, LINK.L, TRAPcc, Bcc.L

use dis68k::m68k::decode::decode_instruction;
use dis68k::m68k::variants::CpuVariant;
use dis68k::m68k::instruction::{Instruction, Mnemonic, Operand, Size, Condition};
use dis68k::m68k::addressing::EffectiveAddress;

fn decode_68000(bytes: &[u8]) -> Instruction {
    decode_instruction(bytes, 0, 0, CpuVariant::M68000).unwrap()
}

fn decode_68020(bytes: &[u8]) -> Instruction {
    decode_instruction(bytes, 0, 0, CpuVariant::M68020).unwrap()
}

// ─── EXTB.L Tests ────────────────────────────────────────────────────

#[test]
fn test_extb_l_d0() {
    // EXTB.L d0: 0100_1000_11_000_000 = 0x49C0
    let bytes = [0x49, 0xC0];
    let inst = decode_68020(&bytes);

    assert_eq!(inst.mnemonic, Mnemonic::Extb);
    assert_eq!(inst.size, Some(Size::Long));
    assert_eq!(inst.size_bytes, 2);
    assert_eq!(inst.cpu_required, CpuVariant::M68020);
    assert_eq!(inst.operands.len(), 1);
    assert_eq!(inst.operands[0], Operand::Ea(EffectiveAddress::DataDirect(0)));
}

#[test]
fn test_extb_l_d7() {
    // EXTB.L d7: 0x49C7
    let bytes = [0x49, 0xC7];
    let inst = decode_68020(&bytes);

    assert_eq!(inst.mnemonic, Mnemonic::Extb);
    assert_eq!(inst.operands[0], Operand::Ea(EffectiveAddress::DataDirect(7)));
}

#[test]
fn test_extb_l_rejected_on_68000() {
    // EXTB.L should decode as dc.w on 68000
    let bytes = [0x49, 0xC0];
    let inst = decode_68000(&bytes);

    assert_eq!(inst.mnemonic, Mnemonic::Dc);
    assert_eq!(inst.size, Some(Size::Word));
    assert_eq!(inst.operands[0], Operand::Ea(EffectiveAddress::Immediate(0x49C0)));
}

// ─── LINK.L Tests ────────────────────────────────────────────────────

#[test]
fn test_link_l_a5() {
    // LINK.L a5,#-100: 0x4808 followed by 32-bit displacement
    let bytes = [
        0x48, 0x0D,              // LINK.L a5
        0xFF, 0xFF, 0xFF, 0x9C   // -100 as i32
    ];
    let inst = decode_68020(&bytes);

    assert_eq!(inst.mnemonic, Mnemonic::Link);
    assert_eq!(inst.size, Some(Size::Long));
    assert_eq!(inst.size_bytes, 6);
    assert_eq!(inst.cpu_required, CpuVariant::M68020);
    assert_eq!(inst.operands.len(), 2);
    assert_eq!(inst.operands[0], Operand::Ea(EffectiveAddress::AddressDirect(5)));
    assert_eq!(inst.operands[1], Operand::Ea(EffectiveAddress::Immediate(0xFFFFFF9C)));
}

#[test]
fn test_link_l_rejected_on_68000() {
    let bytes = [0x48, 0x08, 0x00, 0x00, 0x00, 0x10];
    let inst = decode_68000(&bytes);

    assert_eq!(inst.mnemonic, Mnemonic::Dc);
}

// ─── TRAPcc Tests ────────────────────────────────────────────────────

#[test]
fn test_trapcc_no_operand() {
    // TRAPeq (no operand): 0101_0111_11_111_100 = 0x57FC
    let bytes = [0x57, 0xFC];
    let inst = decode_68020(&bytes);

    assert_eq!(inst.mnemonic, Mnemonic::Trapcc);
    assert_eq!(inst.condition, Some(Condition::Eq));
    assert_eq!(inst.size_bytes, 2);
    assert_eq!(inst.cpu_required, CpuVariant::M68020);
    assert_eq!(inst.operands.len(), 0);
}

#[test]
fn test_trapcc_word_operand() {
    // TRAPne.W #$1234: 0101_0110_11_111_010 = 0x56FA
    let bytes = [0x56, 0xFA, 0x12, 0x34];
    let inst = decode_68020(&bytes);

    assert_eq!(inst.mnemonic, Mnemonic::Trapcc);
    assert_eq!(inst.condition, Some(Condition::Ne));
    assert_eq!(inst.size_bytes, 4);
    assert_eq!(inst.cpu_required, CpuVariant::M68020);
    assert_eq!(inst.operands.len(), 1);
    assert_eq!(inst.operands[0], Operand::Ea(EffectiveAddress::Immediate(0x1234)));
}

#[test]
fn test_trapcc_long_operand() {
    // TRAPhi.L #$12345678: 0101_0010_11_111_011 = 0x52FB
    let bytes = [0x52, 0xFB, 0x12, 0x34, 0x56, 0x78];
    let inst = decode_68020(&bytes);

    assert_eq!(inst.mnemonic, Mnemonic::Trapcc);
    assert_eq!(inst.condition, Some(Condition::Hi));
    assert_eq!(inst.size_bytes, 6);
    assert_eq!(inst.cpu_required, CpuVariant::M68020);
    assert_eq!(inst.operands.len(), 1);
    assert_eq!(inst.operands[0], Operand::Ea(EffectiveAddress::Immediate(0x12345678)));
}

#[test]
fn test_trapcc_rejected_on_68000() {
    let bytes = [0x57, 0xFC];
    let inst = decode_68000(&bytes);

    assert_eq!(inst.mnemonic, Mnemonic::Dc);
}

// ─── Bcc.L Tests ─────────────────────────────────────────────────────

#[test]
fn test_bra_long() {
    // BRA.L <disp>: 0110_0000_11111111 followed by 32-bit displacement
    let bytes = [
        0x60, 0xFF,              // BRA.L
        0x00, 0x01, 0x00, 0x00   // displacement = +65536
    ];
    let inst = decode_68020(&bytes);

    assert_eq!(inst.mnemonic, Mnemonic::Bra);
    assert_eq!(inst.size, Some(Size::Long));
    assert_eq!(inst.size_bytes, 6);
    assert_eq!(inst.cpu_required, CpuVariant::M68020);
    assert_eq!(inst.operands.len(), 1);
    assert_eq!(inst.operands[0], Operand::Displacement32(0x00010000));
}

#[test]
fn test_bsr_long() {
    // BSR.L <disp>: 0110_0001_11111111
    let bytes = [
        0x61, 0xFF,
        0xFF, 0xFF, 0xFF, 0xFE   // displacement = -2
    ];
    let inst = decode_68020(&bytes);

    assert_eq!(inst.mnemonic, Mnemonic::Bsr);
    assert_eq!(inst.size, Some(Size::Long));
    assert_eq!(inst.cpu_required, CpuVariant::M68020);
    assert_eq!(inst.operands[0], Operand::Displacement32(-2));
}

#[test]
fn test_bcc_long() {
    // Beq.L <disp>: 0110_0111_11111111
    let bytes = [
        0x67, 0xFF,
        0x00, 0x00, 0x10, 0x00   // displacement = +4096
    ];
    let inst = decode_68020(&bytes);

    assert_eq!(inst.mnemonic, Mnemonic::Bcc);
    assert_eq!(inst.condition, Some(Condition::Eq));
    assert_eq!(inst.size, Some(Size::Long));
    assert_eq!(inst.cpu_required, CpuVariant::M68020);
    assert_eq!(inst.operands[0], Operand::Displacement32(0x00001000));
}

#[test]
fn test_bcc_long_rejected_on_68000() {
    let bytes = [0x60, 0xFF, 0x00, 0x01, 0x00, 0x00];
    let inst = decode_68000(&bytes);

    // On 68000, 0xFF displacement should decode as dc.w
    assert_eq!(inst.mnemonic, Mnemonic::Dc);
}

// ─── Backward Compatibility Tests ───────────────────────────────────

#[test]
fn test_68000_instructions_unchanged_on_68020() {
    // Existing 68000 instructions should decode identically on 68020

    // MOVE.L d0,d1: 0x2200
    let bytes = [0x22, 0x00];
    let inst_68000 = decode_68000(&bytes);
    let inst_68020 = decode_68020(&bytes);

    assert_eq!(inst_68000.mnemonic, inst_68020.mnemonic);
    assert_eq!(inst_68000.size, inst_68020.size);
    assert_eq!(inst_68000.operands, inst_68020.operands);

    // RTS: 0x4E75
    let bytes = [0x4E, 0x75];
    let inst_68000 = decode_68000(&bytes);
    let inst_68020 = decode_68020(&bytes);

    assert_eq!(inst_68000.mnemonic, Mnemonic::Rts);
    assert_eq!(inst_68020.mnemonic, Mnemonic::Rts);
}
