//! Auto-label generation by collecting branch and jump targets.
//!
//! Walks code sequentially, decodes each instruction, and extracts
//! target addresses from branch/jump operands to build a set of
//! addresses that should receive `loc_XXXX` labels.

use std::collections::BTreeSet;

use crate::m68k::addressing::EffectiveAddress;
use crate::m68k::decode::decode_instruction;
use crate::m68k::instruction::{Mnemonic, Operand};
use crate::m68k::variants::CpuVariant;

/// Scan a code section and collect all branch/jump target addresses.
///
/// Walks the code from the start, decoding each instruction. For any
/// instruction that transfers control (branches, jumps, subroutine calls),
/// extracts the target address and adds it to the result set.
///
/// # Arguments
/// - `data`: raw bytes of the code hunk
/// - `base_address`: the load address of this hunk (typically 0 for relative addressing)
/// - `cpu`: CPU variant to use for decoding
///
/// # Returns
/// A sorted set of target addresses within this hunk that should get labels.
pub fn collect_branch_targets(data: &[u8], base_address: u32, cpu: CpuVariant) -> BTreeSet<u32> {
    let mut targets = BTreeSet::new();
    let mut offset = 0usize;

    while offset < data.len() {
        match decode_instruction(data, offset, base_address, cpu) {
            Ok(inst) => {
                // TODO: Extract target addresses from this instruction.
                //
                // Check if the instruction is a control-flow transfer
                // (Bra, Bsr, Bcc, Jmp, Jsr, Dbcc) and extract the target
                // address from its operands.
                extract_targets(&inst.mnemonic, &inst.operands, inst.address, &mut targets);

                offset += inst.size_bytes as usize;
            }
            Err(_) => {
                // Skip undecodable words
                offset += 2.min(data.len() - offset);
            }
        }
    }

    // Only keep targets that fall within this hunk's address range
    let hunk_start = base_address;
    let hunk_end = base_address + data.len() as u32;
    targets.retain(|&addr| addr >= hunk_start && addr < hunk_end);

    targets
}

/// Extract branch/jump target addresses from an instruction's operands.
///
/// This function decides which instructions create auto-labels and how
/// to compute the target address from their operands.
fn extract_targets(
    mnemonic: &Mnemonic,
    operands: &[Operand],
    _address: u32,
    targets: &mut BTreeSet<u32>,
) {
    // Only consider control-flow instructions
    let is_control_flow = matches!(
        mnemonic,
        Mnemonic::Bra | Mnemonic::Bsr | Mnemonic::Bcc | Mnemonic::Jmp | Mnemonic::Jsr | Mnemonic::Dbcc
    );

    if !is_control_flow {
        return;
    }

    for op in operands {
        match op {
            Operand::Displacement8(d) => {
                // target = instruction_address + 2 + displacement
                let target = (_address as i32) + 2 + (*d as i32);
                if target >= 0 {
                    targets.insert(target as u32);
                }
            }
            Operand::Displacement16(d) => {
                let target = (_address as i32) + 2 + (*d as i32);
                if target >= 0 {
                    targets.insert(target as u32);
                }
            }
            Operand::Ea(EffectiveAddress::AbsoluteLong(addr)) => {
                targets.insert(*addr);
            }
            Operand::Ea(EffectiveAddress::AbsoluteShort(addr)) => {
                // Sign-extended to 32 bits
                targets.insert(*addr as i16 as i32 as u32);
            }
            _ => {}
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn collect_targets_from_branch() {
        // BEQ.S $+8 (branch forward 6 bytes from PC)
        // Opcode: 0x6706 — Bcc(eq) with 8-bit displacement of 6
        // At address 0: target = 0 + 2 + 6 = 8
        // Then NOP (0x4E71) x3 to pad, then NOP at target
        let code: Vec<u8> = vec![
            0x67, 0x06, // beq.s  $00000008
            0x4E, 0x71, // nop
            0x4E, 0x71, // nop
            0x4E, 0x71, // nop
            0x4E, 0x71, // nop (target at offset 8)
        ];
        let targets = collect_branch_targets(&code, 0, CpuVariant::M68000);
        assert!(targets.contains(&8), "should find branch target at 8, got: {:?}", targets);
    }

    #[test]
    fn collect_targets_filters_out_of_range() {
        // BRA.S to address past the hunk — should be excluded
        // 0x607E = BRA.S +126 → target = 0 + 2 + 126 = 128, but code is only 2 bytes
        let code: Vec<u8> = vec![0x60, 0x7E];
        let targets = collect_branch_targets(&code, 0, CpuVariant::M68000);
        assert!(targets.is_empty(), "out-of-range target should be excluded");
    }

    #[test]
    fn collect_targets_bsr() {
        // BSR.S $+4
        // 0x6102 = BSR.S +2 → target = 0 + 2 + 2 = 4
        let code: Vec<u8> = vec![
            0x61, 0x02, // bsr.s  $00000004
            0x4E, 0x71, // nop
            0x4E, 0x75, // rts (target at offset 4)
        ];
        let targets = collect_branch_targets(&code, 0, CpuVariant::M68000);
        assert!(targets.contains(&4));
    }
}
