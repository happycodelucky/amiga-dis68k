use crate::m68k::addressing::EffectiveAddress;
use crate::m68k::instruction::*;

/// Options controlling assembly output formatting.
#[derive(Debug, Clone)]
pub struct FormatOptions {
    /// Use uppercase mnemonics (MOVE vs move).
    pub uppercase: bool,
}

impl Default for FormatOptions {
    fn default() -> Self {
        FormatOptions { uppercase: false }
    }
}

/// A formatted instruction ready for display.
pub struct FormattedInstruction {
    /// Hex bytes column (e.g., "4E75").
    pub hex_bytes: String,
    /// Mnemonic + size suffix (e.g., "move.l" or "rts").
    pub mnemonic: String,
    /// Operand string (e.g., "#$2A,d0").
    pub operands: String,
}

/// Format a decoded instruction into Motorola assembly syntax.
pub fn format_instruction(inst: &Instruction, opts: &FormatOptions) -> FormattedInstruction {
    let hex_bytes = inst
        .raw_bytes
        .iter()
        .map(|b| format!("{b:02X}"))
        .collect::<String>();

    let mut mnemonic = String::new();

    // Build mnemonic with condition and size suffix
    let base_name = inst.mnemonic.name();
    mnemonic.push_str(base_name);

    if inst.mnemonic.is_conditional() {
        if let Some(cond) = &inst.condition {
            mnemonic.push_str(cond.suffix());
        }
    }

    if let Some(size) = &inst.size {
        // Don't add size suffix for certain mnemonics where it's implicit
        if !matches!(
            inst.mnemonic,
            Mnemonic::Bra
                | Mnemonic::Bsr
                | Mnemonic::Bcc
                | Mnemonic::Dbcc
                | Mnemonic::Jmp
                | Mnemonic::Jsr
                | Mnemonic::Nop
                | Mnemonic::Rts
                | Mnemonic::Rte
                | Mnemonic::Rtr
                | Mnemonic::Trap
                | Mnemonic::Trapv
                | Mnemonic::Illegal
                | Mnemonic::Reset
                | Mnemonic::Unlk
                | Mnemonic::Moveq
        ) {
            mnemonic.push_str(size.suffix());
        }
    }

    if opts.uppercase {
        mnemonic = mnemonic.to_uppercase();
    }

    let operands = format_operands(inst, opts);

    FormattedInstruction {
        hex_bytes,
        mnemonic,
        operands,
    }
}

fn format_operands(inst: &Instruction, opts: &FormatOptions) -> String {
    if inst.operands.is_empty() {
        return String::new();
    }

    let parts: Vec<String> = inst
        .operands
        .iter()
        .map(|op| format_operand(op, inst, opts))
        .collect();

    parts.join(",")
}

fn format_operand(op: &Operand, inst: &Instruction, _opts: &FormatOptions) -> String {
    match op {
        Operand::Ea(ea) => format_ea(ea),
        Operand::RegisterList(mask) => format_register_list(*mask, inst),
        Operand::QuickImmediate(n) => format!("#{n}"),
        Operand::MoveqImmediate(n) => {
            if *n >= 0 {
                format!("#{n}")
            } else {
                // Show as signed decimal
                format!("#{n}")
            }
        }
        Operand::Displacement8(d) => {
            // Branch target = PC + 2 + displacement
            // (PC is address of opcode word + 2 at the time displacement is applied)
            let target = (inst.address as i32) + 2 + (*d as i32);
            format!("${target:08X}")
        }
        Operand::Displacement16(d) => {
            let target = (inst.address as i32) + 2 + (*d as i32);
            format!("${target:08X}")
        }
        Operand::TrapVector(n) => format!("#{n}"),
        Operand::Ccr => "ccr".to_string(),
        Operand::Sr => "sr".to_string(),
        Operand::Usp => "usp".to_string(),
    }
}

fn format_ea(ea: &EffectiveAddress) -> String {
    match ea {
        EffectiveAddress::DataDirect(n) => format!("d{n}"),
        EffectiveAddress::AddressDirect(n) => {
            if *n == 7 {
                "sp".to_string()
            } else {
                format!("a{n}")
            }
        }
        EffectiveAddress::AddressIndirect(n) => {
            if *n == 7 {
                "(sp)".to_string()
            } else {
                format!("(a{n})")
            }
        }
        EffectiveAddress::AddressPostIncrement(n) => {
            if *n == 7 {
                "(sp)+".to_string()
            } else {
                format!("(a{n})+")
            }
        }
        EffectiveAddress::AddressPreDecrement(n) => {
            if *n == 7 {
                "-(sp)".to_string()
            } else {
                format!("-(a{n})")
            }
        }
        EffectiveAddress::AddressDisplacement(n, disp) => {
            let reg = if *n == 7 {
                "sp".to_string()
            } else {
                format!("a{n}")
            };
            format!("({disp},{reg})")
        }
        EffectiveAddress::AddressIndex {
            reg,
            index_reg,
            index_size,
            scale,
            displacement,
        } => {
            let base = if *reg == 7 {
                "sp".to_string()
            } else {
                format!("a{reg}")
            };
            let idx = index_reg.to_string();
            let sz = index_size.suffix();
            if *scale > 1 {
                format!("({displacement},{base},{idx}{sz}*{scale})")
            } else {
                format!("({displacement},{base},{idx}{sz})")
            }
        }
        EffectiveAddress::AbsoluteShort(addr) => format!("(${addr:04X}).w"),
        EffectiveAddress::AbsoluteLong(addr) => format!("${addr:08X}"),
        EffectiveAddress::PcDisplacement(disp) => format!("({disp},pc)"),
        EffectiveAddress::PcIndex {
            index_reg,
            index_size,
            scale,
            displacement,
        } => {
            let idx = index_reg.to_string();
            let sz = index_size.suffix();
            if *scale > 1 {
                format!("({displacement},pc,{idx}{sz}*{scale})")
            } else {
                format!("({displacement},pc,{idx}{sz})")
            }
        }
        EffectiveAddress::Immediate(val) => {
            if *val <= 0xFF {
                format!("#${val:02X}")
            } else if *val <= 0xFFFF {
                format!("#${val:04X}")
            } else {
                format!("#${val:08X}")
            }
        }
    }
}

/// Format a MOVEM register list bitmask into d0-d7/a0-a7 notation.
///
/// The bitmask layout depends on whether the destination is predecrement:
/// - Normal (register-to-memory, memory-to-register):
///   bit 0=D0, bit 1=D1, ..., bit 7=D7, bit 8=A0, ..., bit 15=A7
/// - Predecrement -(An):
///   bit 0=A7, bit 1=A6, ..., bit 7=A0, bit 8=D7, ..., bit 15=D0
fn format_register_list(mask: u16, inst: &Instruction) -> String {
    // Check if this is a predecrement MOVEM (register-to-memory with -(An))
    let is_predecrement = inst.mnemonic == Mnemonic::Movem
        && inst.operands.len() == 2
        && matches!(
            inst.operands[1],
            Operand::Ea(EffectiveAddress::AddressPreDecrement(_))
        );

    let effective_mask = if is_predecrement {
        reverse_bits_16(mask)
    } else {
        mask
    };

    let mut parts = Vec::new();

    // Data registers (bits 0-7)
    format_reg_range(&mut parts, effective_mask & 0xFF, "d");
    // Address registers (bits 8-15)
    format_reg_range(&mut parts, (effective_mask >> 8) & 0xFF, "a");

    parts.join("/")
}

fn format_reg_range(parts: &mut Vec<String>, mask: u16, prefix: &str) -> () {
    let mut i = 0u8;
    while i < 8 {
        if (mask & (1 << i)) != 0 {
            let start = i;
            while i < 7 && (mask & (1 << (i + 1))) != 0 {
                i += 1;
            }
            if i > start {
                parts.push(format!("{prefix}{start}-{prefix}{i}"));
            } else {
                parts.push(format!("{prefix}{start}"));
            }
        }
        i += 1;
    }
}

fn reverse_bits_16(val: u16) -> u16 {
    let mut result = 0u16;
    for i in 0..16 {
        if (val & (1 << i)) != 0 {
            result |= 1 << (15 - i);
        }
    }
    result
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::m68k::variants::CpuVariant;

    fn make_inst(mnemonic: Mnemonic, size: Option<Size>, operands: Vec<Operand>) -> Instruction {
        Instruction {
            address: 0,
            size_bytes: 2,
            raw_bytes: vec![0; 2],
            mnemonic,
            size,
            condition: None,
            operands,
            cpu_required: CpuVariant::M68000,
        }
    }

    #[test]
    fn format_rts() {
        let inst = make_inst(Mnemonic::Rts, None, vec![]);
        let fmt = format_instruction(&inst, &FormatOptions::default());
        assert_eq!(fmt.mnemonic, "rts");
        assert_eq!(fmt.operands, "");
    }

    #[test]
    fn format_move_long() {
        let inst = make_inst(
            Mnemonic::Move,
            Some(Size::Long),
            vec![
                Operand::Ea(EffectiveAddress::DataDirect(0)),
                Operand::Ea(EffectiveAddress::DataDirect(1)),
            ],
        );
        let fmt = format_instruction(&inst, &FormatOptions::default());
        assert_eq!(fmt.mnemonic, "move.l");
        assert_eq!(fmt.operands, "d0,d1");
    }

    #[test]
    fn format_jsr_displacement() {
        let inst = make_inst(
            Mnemonic::Jsr,
            None,
            vec![Operand::Ea(EffectiveAddress::AddressDisplacement(6, -552))],
        );
        let fmt = format_instruction(&inst, &FormatOptions::default());
        assert_eq!(fmt.mnemonic, "jsr");
        assert_eq!(fmt.operands, "(-552,a6)");
    }

    #[test]
    fn format_absolute_short() {
        let inst = make_inst(
            Mnemonic::Movea,
            Some(Size::Long),
            vec![
                Operand::Ea(EffectiveAddress::AbsoluteShort(4)),
                Operand::Ea(EffectiveAddress::AddressDirect(6)),
            ],
        );
        let fmt = format_instruction(&inst, &FormatOptions::default());
        assert_eq!(fmt.operands, "($0004).w,a6");
    }

    #[test]
    fn format_uppercase() {
        let inst = make_inst(Mnemonic::Nop, None, vec![]);
        let fmt = format_instruction(&inst, &FormatOptions { uppercase: true });
        assert_eq!(fmt.mnemonic, "NOP");
    }

    #[test]
    fn format_a7_as_sp() {
        let inst = make_inst(
            Mnemonic::Move,
            Some(Size::Long),
            vec![
                Operand::Ea(EffectiveAddress::DataDirect(0)),
                Operand::Ea(EffectiveAddress::AddressPreDecrement(7)),
            ],
        );
        let fmt = format_instruction(&inst, &FormatOptions::default());
        assert_eq!(fmt.operands, "d0,-(sp)");
    }
}
