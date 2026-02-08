use super::addressing::{EffectiveAddress, IndexRegister};
use super::instruction::*;
use super::variants::CpuVariant;

/// Errors during instruction decoding.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DecodeError {
    /// Not enough bytes to decode the instruction.
    UnexpectedEof { address: u32, needed: usize },
    /// Unrecognized opcode word.
    UnknownOpcode { address: u32, opcode: u16 },
    /// Invalid effective address mode/register combination.
    InvalidEa { address: u32, mode: u8, reg: u8 },
}

impl std::fmt::Display for DecodeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DecodeError::UnexpectedEof { address, needed } => {
                write!(f, "at ${address:08X}: need {needed} more bytes")
            }
            DecodeError::UnknownOpcode { address, opcode } => {
                write!(f, "at ${address:08X}: unknown opcode ${opcode:04X}")
            }
            DecodeError::InvalidEa { address, mode, reg } => {
                write!(
                    f,
                    "at ${address:08X}: invalid EA mode={mode} reg={reg}"
                )
            }
        }
    }
}

impl std::error::Error for DecodeError {}

/// Internal decode state wrapping a byte slice with position tracking.
struct DecodeCtx<'a> {
    data: &'a [u8],
    base_address: u32,
    pos: usize,
    start: usize,
    cpu: CpuVariant,
}

impl<'a> DecodeCtx<'a> {
    fn new(data: &'a [u8], offset: usize, base_address: u32, cpu: CpuVariant) -> Self {
        DecodeCtx {
            data,
            base_address,
            pos: offset,
            start: offset,
            cpu,
        }
    }

    fn address(&self) -> u32 {
        self.base_address + self.start as u32
    }

    fn current_pc(&self) -> u32 {
        self.base_address + self.pos as u32
    }

    fn bytes_consumed(&self) -> usize {
        self.pos - self.start
    }

    fn raw_bytes(&self) -> Vec<u8> {
        self.data[self.start..self.pos].to_vec()
    }

    fn read_u16(&mut self) -> Result<u16, DecodeError> {
        if self.pos + 2 > self.data.len() {
            return Err(DecodeError::UnexpectedEof {
                address: self.address(),
                needed: 2,
            });
        }
        let val = u16::from_be_bytes([self.data[self.pos], self.data[self.pos + 1]]);
        self.pos += 2;
        Ok(val)
    }

    fn read_u32(&mut self) -> Result<u32, DecodeError> {
        if self.pos + 4 > self.data.len() {
            return Err(DecodeError::UnexpectedEof {
                address: self.address(),
                needed: 4,
            });
        }
        let val = u32::from_be_bytes([
            self.data[self.pos],
            self.data[self.pos + 1],
            self.data[self.pos + 2],
            self.data[self.pos + 3],
        ]);
        self.pos += 4;
        Ok(val)
    }

    /// Decode a standard 6-bit effective address field.
    ///
    /// The 68k encodes EA as mode(3 bits) + register(3 bits). For most
    /// instructions, these come from bits 5-3 (mode) and 2-0 (register)
    /// of the opcode word. Extension words follow as needed.
    fn decode_ea(&mut self, mode: u8, reg: u8, size: Size) -> Result<EffectiveAddress, DecodeError> {
        match mode {
            0 => Ok(EffectiveAddress::DataDirect(reg)),
            1 => Ok(EffectiveAddress::AddressDirect(reg)),
            2 => Ok(EffectiveAddress::AddressIndirect(reg)),
            3 => Ok(EffectiveAddress::AddressPostIncrement(reg)),
            4 => Ok(EffectiveAddress::AddressPreDecrement(reg)),
            5 => {
                let disp = self.read_u16()? as i16;
                Ok(EffectiveAddress::AddressDisplacement(reg, disp))
            }
            6 => {
                let ext = self.read_u16()?;
                Ok(self.decode_index_ea_reg(reg, ext))
            }
            7 => match reg {
                0 => {
                    let addr = self.read_u16()?;
                    Ok(EffectiveAddress::AbsoluteShort(addr))
                }
                1 => {
                    let addr = self.read_u32()?;
                    Ok(EffectiveAddress::AbsoluteLong(addr))
                }
                2 => {
                    let disp = self.read_u16()? as i16;
                    Ok(EffectiveAddress::PcDisplacement(disp))
                }
                3 => {
                    let ext = self.read_u16()?;
                    Ok(self.decode_index_ea_pc(ext))
                }
                4 => {
                    let imm = match size {
                        Size::Byte => {
                            let w = self.read_u16()?;
                            (w & 0xFF) as u32
                        }
                        Size::Word => self.read_u16()? as u32,
                        Size::Long => self.read_u32()?,
                    };
                    Ok(EffectiveAddress::Immediate(imm))
                }
                _ => Err(DecodeError::InvalidEa {
                    address: self.address(),
                    mode,
                    reg,
                }),
            },
            _ => Err(DecodeError::InvalidEa {
                address: self.address(),
                mode,
                reg,
            }),
        }
    }

    fn decode_index_ea_reg(&self, base_reg: u8, ext: u16) -> EffectiveAddress {
        let index_reg_num = ((ext >> 12) & 0x7) as u8;
        let index_is_addr = (ext & 0x8000) != 0;
        let index_size = if (ext & 0x0800) != 0 { Size::Long } else { Size::Word };
        let scale = ((ext >> 9) & 0x3) as u8;
        let disp = (ext & 0xFF) as i8;
        let index_reg = if index_is_addr {
            IndexRegister::Address(index_reg_num)
        } else {
            IndexRegister::Data(index_reg_num)
        };
        EffectiveAddress::AddressIndex {
            reg: base_reg,
            index_reg,
            index_size,
            scale: 1 << scale,
            displacement: disp,
        }
    }

    fn decode_index_ea_pc(&self, ext: u16) -> EffectiveAddress {
        let index_reg_num = ((ext >> 12) & 0x7) as u8;
        let index_is_addr = (ext & 0x8000) != 0;
        let index_size = if (ext & 0x0800) != 0 { Size::Long } else { Size::Word };
        let scale = ((ext >> 9) & 0x3) as u8;
        let disp = (ext & 0xFF) as i8;
        let index_reg = if index_is_addr {
            IndexRegister::Address(index_reg_num)
        } else {
            IndexRegister::Data(index_reg_num)
        };
        EffectiveAddress::PcIndex {
            index_reg,
            index_size,
            scale: 1 << scale,
            displacement: disp,
        }
    }

    fn make_inst(
        &self,
        mnemonic: Mnemonic,
        size: Option<Size>,
        condition: Option<Condition>,
        operands: Vec<Operand>,
    ) -> Instruction {
        Instruction {
            address: self.address(),
            size_bytes: self.bytes_consumed() as u8,
            raw_bytes: self.raw_bytes(),
            mnemonic,
            size,
            condition,
            operands,
            cpu_required: CpuVariant::M68000,
        }
    }
}

/// Decode a single instruction from the given byte slice.
///
/// - `data`: the raw bytes of the code hunk
/// - `offset`: byte offset into `data` where decoding starts
/// - `base_address`: the logical address of the start of `data` (for PC-relative calculations)
/// - `cpu`: the CPU variant, controlling which instructions are recognized
///
/// Returns the decoded `Instruction` or a `DecodeError`. On error,
/// the caller should emit a `dc.w` for the unrecognized word and
/// advance by 2 bytes.
pub fn decode_instruction(
    data: &[u8],
    offset: usize,
    base_address: u32,
    cpu: CpuVariant,
) -> Result<Instruction, DecodeError> {
    let mut ctx = DecodeCtx::new(data, offset, base_address, cpu);
    let opcode = ctx.read_u16()?;

    // Two-level dispatch: first on bits 15-12
    match (opcode >> 12) & 0xF {
        0x0 => decode_group0(&mut ctx, opcode),
        0x1 => decode_move(&mut ctx, opcode, Size::Byte),
        0x2 => decode_move(&mut ctx, opcode, Size::Long),
        0x3 => decode_move(&mut ctx, opcode, Size::Word),
        0x4 => decode_group4(&mut ctx, opcode),
        0x5 => decode_group5(&mut ctx, opcode),
        0x6 => decode_group6(&mut ctx, opcode),
        0x7 => decode_moveq(&mut ctx, opcode),
        0x8 => decode_group8(&mut ctx, opcode),
        0x9 => decode_group9(&mut ctx, opcode),
        0xA => Ok(ctx.make_inst(Mnemonic::Dc, Some(Size::Word), None, vec![
            Operand::Ea(EffectiveAddress::Immediate(opcode as u32)),
        ])),
        0xB => decode_group_b(&mut ctx, opcode),
        0xC => decode_group_c(&mut ctx, opcode),
        0xD => decode_group_d(&mut ctx, opcode),
        0xE => decode_group_e(&mut ctx, opcode),
        0xF => Ok(ctx.make_inst(Mnemonic::Dc, Some(Size::Word), None, vec![
            Operand::Ea(EffectiveAddress::Immediate(opcode as u32)),
        ])),
        _ => unreachable!(),
    }
}

// ─── Group 0: Immediate operations + bit ops ─────────────────────

fn decode_group0(ctx: &mut DecodeCtx<'_>, opcode: u16) -> Result<Instruction, DecodeError> {
    // Check for dynamic bit operations first (bit 8 set = register bit number)
    if (opcode & 0x0100) != 0 && (opcode & 0x0038) != 0x0008 {
        // BTST/BCHG/BCLR/BSET Dn,<ea>
        return decode_bit_dynamic(ctx, opcode);
    }

    // MOVEP: 0000_rrr_1mm_001_aaa (bit 8 set, EA mode = 001)
    if (opcode & 0x0100) != 0 && (opcode & 0x0038) == 0x0008 {
        return decode_movep(ctx, opcode);
    }

    // Static bit ops or immediate ops based on bits 11-9
    let sub = (opcode >> 9) & 0x7;
    match sub {
        0b000 => decode_ori(ctx, opcode),
        0b001 => decode_andi(ctx, opcode),
        0b010 => decode_subi(ctx, opcode),
        0b011 => decode_addi(ctx, opcode),
        0b100 => decode_bit_static(ctx, opcode),
        0b101 => decode_eori(ctx, opcode),
        0b110 => decode_cmpi(ctx, opcode),
        _ => Ok(make_dc_word(ctx, opcode)),
    }
}

fn decode_ori(ctx: &mut DecodeCtx<'_>, opcode: u16) -> Result<Instruction, DecodeError> {
    let size = decode_size_2bit((opcode >> 6) & 0x3)?;
    let mode = ((opcode >> 3) & 0x7) as u8;
    let reg = (opcode & 0x7) as u8;

    // ORI to CCR
    if mode == 7 && reg == 4 && size == Size::Byte {
        let imm = ctx.read_u16()? & 0xFF;
        return Ok(ctx.make_inst(Mnemonic::Ori, Some(Size::Byte), None, vec![
            Operand::Ea(EffectiveAddress::Immediate(imm as u32)),
            Operand::Ccr,
        ]));
    }
    // ORI to SR
    if mode == 7 && reg == 4 && size == Size::Word {
        let imm = ctx.read_u16()?;
        return Ok(ctx.make_inst(Mnemonic::Ori, Some(Size::Word), None, vec![
            Operand::Ea(EffectiveAddress::Immediate(imm as u32)),
            Operand::Sr,
        ]));
    }

    let imm = read_immediate(ctx, size)?;
    let ea = ctx.decode_ea(mode, reg, size)?;
    Ok(ctx.make_inst(Mnemonic::Ori, Some(size), None, vec![
        Operand::Ea(EffectiveAddress::Immediate(imm)),
        Operand::Ea(ea),
    ]))
}

fn decode_andi(ctx: &mut DecodeCtx<'_>, opcode: u16) -> Result<Instruction, DecodeError> {
    let size = decode_size_2bit((opcode >> 6) & 0x3)?;
    let mode = ((opcode >> 3) & 0x7) as u8;
    let reg = (opcode & 0x7) as u8;

    if mode == 7 && reg == 4 && size == Size::Byte {
        let imm = ctx.read_u16()? & 0xFF;
        return Ok(ctx.make_inst(Mnemonic::Andi, Some(Size::Byte), None, vec![
            Operand::Ea(EffectiveAddress::Immediate(imm as u32)),
            Operand::Ccr,
        ]));
    }
    if mode == 7 && reg == 4 && size == Size::Word {
        let imm = ctx.read_u16()?;
        return Ok(ctx.make_inst(Mnemonic::Andi, Some(Size::Word), None, vec![
            Operand::Ea(EffectiveAddress::Immediate(imm as u32)),
            Operand::Sr,
        ]));
    }

    let imm = read_immediate(ctx, size)?;
    let ea = ctx.decode_ea(mode, reg, size)?;
    Ok(ctx.make_inst(Mnemonic::Andi, Some(size), None, vec![
        Operand::Ea(EffectiveAddress::Immediate(imm)),
        Operand::Ea(ea),
    ]))
}

fn decode_subi(ctx: &mut DecodeCtx<'_>, opcode: u16) -> Result<Instruction, DecodeError> {
    let size = decode_size_2bit((opcode >> 6) & 0x3)?;
    let mode = ((opcode >> 3) & 0x7) as u8;
    let reg = (opcode & 0x7) as u8;
    let imm = read_immediate(ctx, size)?;
    let ea = ctx.decode_ea(mode, reg, size)?;
    Ok(ctx.make_inst(Mnemonic::Subi, Some(size), None, vec![
        Operand::Ea(EffectiveAddress::Immediate(imm)),
        Operand::Ea(ea),
    ]))
}

fn decode_addi(ctx: &mut DecodeCtx<'_>, opcode: u16) -> Result<Instruction, DecodeError> {
    let size = decode_size_2bit((opcode >> 6) & 0x3)?;
    let mode = ((opcode >> 3) & 0x7) as u8;
    let reg = (opcode & 0x7) as u8;
    let imm = read_immediate(ctx, size)?;
    let ea = ctx.decode_ea(mode, reg, size)?;
    Ok(ctx.make_inst(Mnemonic::Addi, Some(size), None, vec![
        Operand::Ea(EffectiveAddress::Immediate(imm)),
        Operand::Ea(ea),
    ]))
}

fn decode_eori(ctx: &mut DecodeCtx<'_>, opcode: u16) -> Result<Instruction, DecodeError> {
    let size = decode_size_2bit((opcode >> 6) & 0x3)?;
    let mode = ((opcode >> 3) & 0x7) as u8;
    let reg = (opcode & 0x7) as u8;

    if mode == 7 && reg == 4 && size == Size::Byte {
        let imm = ctx.read_u16()? & 0xFF;
        return Ok(ctx.make_inst(Mnemonic::Eori, Some(Size::Byte), None, vec![
            Operand::Ea(EffectiveAddress::Immediate(imm as u32)),
            Operand::Ccr,
        ]));
    }
    if mode == 7 && reg == 4 && size == Size::Word {
        let imm = ctx.read_u16()?;
        return Ok(ctx.make_inst(Mnemonic::Eori, Some(Size::Word), None, vec![
            Operand::Ea(EffectiveAddress::Immediate(imm as u32)),
            Operand::Sr,
        ]));
    }

    let imm = read_immediate(ctx, size)?;
    let ea = ctx.decode_ea(mode, reg, size)?;
    Ok(ctx.make_inst(Mnemonic::Eori, Some(size), None, vec![
        Operand::Ea(EffectiveAddress::Immediate(imm)),
        Operand::Ea(ea),
    ]))
}

fn decode_cmpi(ctx: &mut DecodeCtx<'_>, opcode: u16) -> Result<Instruction, DecodeError> {
    let size = decode_size_2bit((opcode >> 6) & 0x3)?;
    let mode = ((opcode >> 3) & 0x7) as u8;
    let reg = (opcode & 0x7) as u8;
    let imm = read_immediate(ctx, size)?;
    let ea = ctx.decode_ea(mode, reg, size)?;
    Ok(ctx.make_inst(Mnemonic::Cmpi, Some(size), None, vec![
        Operand::Ea(EffectiveAddress::Immediate(imm)),
        Operand::Ea(ea),
    ]))
}

fn decode_bit_static(ctx: &mut DecodeCtx<'_>, opcode: u16) -> Result<Instruction, DecodeError> {
    // BTST/BCHG/BCLR/BSET #imm,<ea>
    let bit_op = (opcode >> 6) & 0x3;
    let mode = ((opcode >> 3) & 0x7) as u8;
    let reg = (opcode & 0x7) as u8;
    let bit_num = ctx.read_u16()? & 0xFF;

    let mnemonic = match bit_op {
        0 => Mnemonic::Btst,
        1 => Mnemonic::Bchg,
        2 => Mnemonic::Bclr,
        3 => Mnemonic::Bset,
        _ => unreachable!(),
    };

    // Bit ops on Dn are longword; on memory they're byte
    let size = if mode == 0 { Size::Long } else { Size::Byte };
    let ea = ctx.decode_ea(mode, reg, size)?;
    Ok(ctx.make_inst(mnemonic, None, None, vec![
        Operand::Ea(EffectiveAddress::Immediate(bit_num as u32)),
        Operand::Ea(ea),
    ]))
}

fn decode_bit_dynamic(ctx: &mut DecodeCtx<'_>, opcode: u16) -> Result<Instruction, DecodeError> {
    // BTST/BCHG/BCLR/BSET Dn,<ea>
    let dn = ((opcode >> 9) & 0x7) as u8;
    let bit_op = (opcode >> 6) & 0x3;
    let mode = ((opcode >> 3) & 0x7) as u8;
    let reg = (opcode & 0x7) as u8;

    let mnemonic = match bit_op {
        0 => Mnemonic::Btst,
        1 => Mnemonic::Bchg,
        2 => Mnemonic::Bclr,
        3 => Mnemonic::Bset,
        _ => unreachable!(),
    };

    let size = if mode == 0 { Size::Long } else { Size::Byte };
    let ea = ctx.decode_ea(mode, reg, size)?;
    Ok(ctx.make_inst(mnemonic, None, None, vec![
        Operand::Ea(EffectiveAddress::DataDirect(dn)),
        Operand::Ea(ea),
    ]))
}

fn decode_movep(ctx: &mut DecodeCtx<'_>, opcode: u16) -> Result<Instruction, DecodeError> {
    let dn = ((opcode >> 9) & 0x7) as u8;
    let an = (opcode & 0x7) as u8;
    let op_mode = (opcode >> 6) & 0x3;
    let disp = ctx.read_u16()? as i16;
    let size = if (op_mode & 1) != 0 { Size::Long } else { Size::Word };

    let (src, dst) = if (op_mode & 2) != 0 {
        // Register to memory
        (
            Operand::Ea(EffectiveAddress::DataDirect(dn)),
            Operand::Ea(EffectiveAddress::AddressDisplacement(an, disp)),
        )
    } else {
        // Memory to register
        (
            Operand::Ea(EffectiveAddress::AddressDisplacement(an, disp)),
            Operand::Ea(EffectiveAddress::DataDirect(dn)),
        )
    };
    Ok(ctx.make_inst(Mnemonic::Movep, Some(size), None, vec![src, dst]))
}

// ─── Groups 1-3: MOVE ────────────────────────────────────────────

fn decode_move(ctx: &mut DecodeCtx<'_>, opcode: u16, size: Size) -> Result<Instruction, DecodeError> {
    let src_mode = ((opcode >> 3) & 0x7) as u8;
    let src_reg = (opcode & 0x7) as u8;
    // MOVE destination has register and mode fields REVERSED!
    let dst_reg = ((opcode >> 9) & 0x7) as u8;
    let dst_mode = ((opcode >> 6) & 0x7) as u8;

    let src_ea = ctx.decode_ea(src_mode, src_reg, size)?;

    // MOVEA: destination is an address register (mode 1)
    if dst_mode == 1 {
        // MOVEA only supports word and long
        let movea_size = if size == Size::Byte {
            return Ok(make_dc_word(ctx, opcode));
        } else {
            size
        };
        return Ok(ctx.make_inst(Mnemonic::Movea, Some(movea_size), None, vec![
            Operand::Ea(src_ea),
            Operand::Ea(EffectiveAddress::AddressDirect(dst_reg)),
        ]));
    }

    let dst_ea = ctx.decode_ea(dst_mode, dst_reg, size)?;
    Ok(ctx.make_inst(Mnemonic::Move, Some(size), None, vec![
        Operand::Ea(src_ea),
        Operand::Ea(dst_ea),
    ]))
}

// ─── Group 4: Miscellaneous ──────────────────────────────────────

fn decode_group4(ctx: &mut DecodeCtx<'_>, opcode: u16) -> Result<Instruction, DecodeError> {
    // Single-word encodings first
    match opcode {
        0x4E70 => return Ok(ctx.make_inst(Mnemonic::Reset, None, None, vec![])),
        0x4E71 => return Ok(ctx.make_inst(Mnemonic::Nop, None, None, vec![])),
        0x4E73 => return Ok(ctx.make_inst(Mnemonic::Rte, None, None, vec![])),
        0x4E75 => return Ok(ctx.make_inst(Mnemonic::Rts, None, None, vec![])),
        0x4E76 => return Ok(ctx.make_inst(Mnemonic::Trapv, None, None, vec![])),
        0x4E77 => return Ok(ctx.make_inst(Mnemonic::Rtr, None, None, vec![])),
        0x4AFC => return Ok(ctx.make_inst(Mnemonic::Illegal, None, None, vec![])),
        _ => {}
    }

    // STOP #imm
    if opcode == 0x4E72 {
        let imm = ctx.read_u16()?;
        return Ok(ctx.make_inst(Mnemonic::Stop, None, None, vec![
            Operand::Ea(EffectiveAddress::Immediate(imm as u32)),
        ]));
    }

    // TRAP #vector
    if (opcode & 0xFFF0) == 0x4E40 {
        let vector = (opcode & 0xF) as u8;
        return Ok(ctx.make_inst(Mnemonic::Trap, None, None, vec![
            Operand::TrapVector(vector),
        ]));
    }

    // LINK An,#disp
    if (opcode & 0xFFF8) == 0x4E50 {
        let an = (opcode & 0x7) as u8;
        let disp = ctx.read_u16()? as i16;
        return Ok(ctx.make_inst(Mnemonic::Link, Some(Size::Word), None, vec![
            Operand::Ea(EffectiveAddress::AddressDirect(an)),
            Operand::Ea(EffectiveAddress::Immediate(disp as u16 as u32)),
        ]));
    }

    // UNLK An
    if (opcode & 0xFFF8) == 0x4E58 {
        let an = (opcode & 0x7) as u8;
        return Ok(ctx.make_inst(Mnemonic::Unlk, None, None, vec![
            Operand::Ea(EffectiveAddress::AddressDirect(an)),
        ]));
    }

    // MOVE An,USP / MOVE USP,An
    if (opcode & 0xFFF0) == 0x4E60 {
        let an = (opcode & 0x7) as u8;
        let dir = (opcode >> 3) & 1;
        if dir == 0 {
            return Ok(ctx.make_inst(Mnemonic::MoveUsp, None, None, vec![
                Operand::Ea(EffectiveAddress::AddressDirect(an)),
                Operand::Usp,
            ]));
        } else {
            return Ok(ctx.make_inst(Mnemonic::MoveUsp, None, None, vec![
                Operand::Usp,
                Operand::Ea(EffectiveAddress::AddressDirect(an)),
            ]));
        }
    }

    let mode = ((opcode >> 3) & 0x7) as u8;
    let reg = (opcode & 0x7) as u8;

    // SWAP Dn
    if (opcode & 0xFFF8) == 0x4840 {
        return Ok(ctx.make_inst(Mnemonic::Swap, Some(Size::Word), None, vec![
            Operand::Ea(EffectiveAddress::DataDirect(reg)),
        ]));
    }

    // PEA <ea>
    if (opcode & 0xFFC0) == 0x4840 && mode != 0 {
        let ea = ctx.decode_ea(mode, reg, Size::Long)?;
        return Ok(ctx.make_inst(Mnemonic::Pea, Some(Size::Long), None, vec![
            Operand::Ea(ea),
        ]));
    }

    // EXT.W Dn or EXT.L Dn
    if (opcode & 0xFEB8) == 0x4880 && mode == 0 {
        let size = if (opcode & 0x0040) != 0 { Size::Long } else { Size::Word };
        return Ok(ctx.make_inst(Mnemonic::Ext, Some(size), None, vec![
            Operand::Ea(EffectiveAddress::DataDirect(reg)),
        ]));
    }

    // MOVEM
    if (opcode & 0xFB80) == 0x4880 {
        return decode_movem(ctx, opcode);
    }

    // LEA <ea>,An
    if (opcode & 0xF1C0) == 0x41C0 {
        let an = ((opcode >> 9) & 0x7) as u8;
        let ea = ctx.decode_ea(mode, reg, Size::Long)?;
        return Ok(ctx.make_inst(Mnemonic::Lea, Some(Size::Long), None, vec![
            Operand::Ea(ea),
            Operand::Ea(EffectiveAddress::AddressDirect(an)),
        ]));
    }

    // CHK <ea>,Dn
    if (opcode & 0xF1C0) == 0x4180 {
        let dn = ((opcode >> 9) & 0x7) as u8;
        let ea = ctx.decode_ea(mode, reg, Size::Word)?;
        return Ok(ctx.make_inst(Mnemonic::Chk, Some(Size::Word), None, vec![
            Operand::Ea(ea),
            Operand::Ea(EffectiveAddress::DataDirect(dn)),
        ]));
    }

    // JMP / JSR
    if (opcode & 0xFFC0) == 0x4EC0 {
        let ea = ctx.decode_ea(mode, reg, Size::Long)?;
        return Ok(ctx.make_inst(Mnemonic::Jmp, None, None, vec![Operand::Ea(ea)]));
    }
    if (opcode & 0xFFC0) == 0x4E80 {
        let ea = ctx.decode_ea(mode, reg, Size::Long)?;
        return Ok(ctx.make_inst(Mnemonic::Jsr, None, None, vec![Operand::Ea(ea)]));
    }

    // NBCD <ea>
    if (opcode & 0xFFC0) == 0x4800 {
        let ea = ctx.decode_ea(mode, reg, Size::Byte)?;
        return Ok(ctx.make_inst(Mnemonic::Nbcd, Some(Size::Byte), None, vec![
            Operand::Ea(ea),
        ]));
    }

    // TAS <ea>
    if (opcode & 0xFFC0) == 0x4AC0 {
        let ea = ctx.decode_ea(mode, reg, Size::Byte)?;
        return Ok(ctx.make_inst(Mnemonic::Tas, Some(Size::Byte), None, vec![
            Operand::Ea(ea),
        ]));
    }

    // MOVE from SR: 0100_0000_11_mmmrrr
    if (opcode & 0xFFC0) == 0x40C0 {
        let ea = ctx.decode_ea(mode, reg, Size::Word)?;
        return Ok(ctx.make_inst(Mnemonic::MoveFromSr, Some(Size::Word), None, vec![
            Operand::Sr,
            Operand::Ea(ea),
        ]));
    }

    // MOVE to CCR: 0100_0100_11_mmmrrr
    if (opcode & 0xFFC0) == 0x44C0 {
        let ea = ctx.decode_ea(mode, reg, Size::Word)?;
        return Ok(ctx.make_inst(Mnemonic::MoveToCcr, None, None, vec![
            Operand::Ea(ea),
            Operand::Ccr,
        ]));
    }

    // MOVE to SR: 0100_0110_11_mmmrrr
    if (opcode & 0xFFC0) == 0x46C0 {
        let ea = ctx.decode_ea(mode, reg, Size::Word)?;
        return Ok(ctx.make_inst(Mnemonic::MoveToSr, Some(Size::Word), None, vec![
            Operand::Ea(ea),
            Operand::Sr,
        ]));
    }

    // NEG, NEGX, NOT, CLR, TST: 0100_ooo_ss_mmmrrr
    let sub_op = (opcode >> 9) & 0x7;
    let size_bits = (opcode >> 6) & 0x3;
    if size_bits != 3 {
        if let Ok(size) = decode_size_2bit(size_bits) {
            let mnemonic = match sub_op {
                0 => Mnemonic::Negx,
                1 => Mnemonic::Clr,
                2 => Mnemonic::Neg,
                3 => Mnemonic::Not,
                5 => Mnemonic::Tst,
                _ => return Ok(make_dc_word(ctx, opcode)),
            };
            let ea = ctx.decode_ea(mode, reg, size)?;
            return Ok(ctx.make_inst(mnemonic, Some(size), None, vec![Operand::Ea(ea)]));
        }
    }

    Ok(make_dc_word(ctx, opcode))
}

fn decode_movem(ctx: &mut DecodeCtx<'_>, opcode: u16) -> Result<Instruction, DecodeError> {
    let size = if (opcode & 0x0040) != 0 { Size::Long } else { Size::Word };
    let direction = (opcode >> 10) & 1; // 0 = reg-to-mem, 1 = mem-to-reg
    let mode = ((opcode >> 3) & 0x7) as u8;
    let reg = (opcode & 0x7) as u8;

    let register_mask = ctx.read_u16()?;
    let ea = ctx.decode_ea(mode, reg, size)?;

    if direction == 0 {
        Ok(ctx.make_inst(Mnemonic::Movem, Some(size), None, vec![
            Operand::RegisterList(register_mask),
            Operand::Ea(ea),
        ]))
    } else {
        Ok(ctx.make_inst(Mnemonic::Movem, Some(size), None, vec![
            Operand::Ea(ea),
            Operand::RegisterList(register_mask),
        ]))
    }
}

// ─── Group 5: ADDQ / SUBQ / Scc / DBcc ──────────────────────────

fn decode_group5(ctx: &mut DecodeCtx<'_>, opcode: u16) -> Result<Instruction, DecodeError> {
    let size_bits = (opcode >> 6) & 0x3;

    if size_bits == 3 {
        // Scc / DBcc
        let condition = Condition::from_bits(((opcode >> 8) & 0xF) as u8);
        let mode = ((opcode >> 3) & 0x7) as u8;
        let reg = (opcode & 0x7) as u8;

        if mode == 1 {
            // DBcc Dn,<displacement>
            let disp = ctx.read_u16()? as i16;
            return Ok(ctx.make_inst(Mnemonic::Dbcc, Some(Size::Word), Some(condition), vec![
                Operand::Ea(EffectiveAddress::DataDirect(reg)),
                Operand::Displacement16(disp),
            ]));
        }

        // Scc <ea>
        let ea = ctx.decode_ea(mode, reg, Size::Byte)?;
        return Ok(ctx.make_inst(Mnemonic::Scc, Some(Size::Byte), Some(condition), vec![
            Operand::Ea(ea),
        ]));
    }

    // ADDQ / SUBQ
    let size = decode_size_2bit(size_bits)?;
    let mut quick_val = ((opcode >> 9) & 0x7) as u8;
    if quick_val == 0 {
        quick_val = 8;
    }
    let mode = ((opcode >> 3) & 0x7) as u8;
    let reg = (opcode & 0x7) as u8;
    let ea = ctx.decode_ea(mode, reg, size)?;

    let mnemonic = if (opcode & 0x0100) == 0 {
        Mnemonic::Addq
    } else {
        Mnemonic::Subq
    };

    Ok(ctx.make_inst(mnemonic, Some(size), None, vec![
        Operand::QuickImmediate(quick_val),
        Operand::Ea(ea),
    ]))
}

// ─── Group 6: Bcc / BRA / BSR ────────────────────────────────────

fn decode_group6(ctx: &mut DecodeCtx<'_>, opcode: u16) -> Result<Instruction, DecodeError> {
    let cond_bits = ((opcode >> 8) & 0xF) as u8;
    let disp8 = (opcode & 0xFF) as i8;

    let (mnemonic, condition) = match cond_bits {
        0 => (Mnemonic::Bra, None),
        1 => (Mnemonic::Bsr, None),
        _ => (Mnemonic::Bcc, Some(Condition::from_bits(cond_bits))),
    };

    if disp8 == 0 {
        // 16-bit displacement follows
        let disp16 = ctx.read_u16()? as i16;
        Ok(ctx.make_inst(mnemonic, Some(Size::Word), condition, vec![
            Operand::Displacement16(disp16),
        ]))
    } else {
        // 8-bit displacement in the opcode word
        Ok(ctx.make_inst(mnemonic, Some(Size::Byte), condition, vec![
            Operand::Displacement8(disp8),
        ]))
    }
}

// ─── Group 7: MOVEQ ─────────────────────────────────────────────

fn decode_moveq(ctx: &mut DecodeCtx<'_>, opcode: u16) -> Result<Instruction, DecodeError> {
    if (opcode & 0x0100) != 0 {
        return Ok(make_dc_word(ctx, opcode));
    }
    let dn = ((opcode >> 9) & 0x7) as u8;
    let data = (opcode & 0xFF) as i8;
    Ok(ctx.make_inst(Mnemonic::Moveq, Some(Size::Long), None, vec![
        Operand::MoveqImmediate(data),
        Operand::Ea(EffectiveAddress::DataDirect(dn)),
    ]))
}

// ─── Group 8: OR / DIVU / DIVS / SBCD ───────────────────────────

fn decode_group8(ctx: &mut DecodeCtx<'_>, opcode: u16) -> Result<Instruction, DecodeError> {
    let dn = ((opcode >> 9) & 0x7) as u8;
    let op_mode = (opcode >> 6) & 0x7;
    let mode = ((opcode >> 3) & 0x7) as u8;
    let reg = (opcode & 0x7) as u8;

    // SBCD
    if op_mode == 4 && mode <= 1 {
        let (src, dst) = if mode == 0 {
            (
                Operand::Ea(EffectiveAddress::DataDirect(reg)),
                Operand::Ea(EffectiveAddress::DataDirect(dn)),
            )
        } else {
            (
                Operand::Ea(EffectiveAddress::AddressPreDecrement(reg)),
                Operand::Ea(EffectiveAddress::AddressPreDecrement(dn)),
            )
        };
        return Ok(ctx.make_inst(Mnemonic::Sbcd, Some(Size::Byte), None, vec![src, dst]));
    }

    // DIVU <ea>,Dn
    if op_mode == 3 {
        let ea = ctx.decode_ea(mode, reg, Size::Word)?;
        return Ok(ctx.make_inst(Mnemonic::Divu, Some(Size::Word), None, vec![
            Operand::Ea(ea),
            Operand::Ea(EffectiveAddress::DataDirect(dn)),
        ]));
    }

    // DIVS <ea>,Dn
    if op_mode == 7 {
        let ea = ctx.decode_ea(mode, reg, Size::Word)?;
        return Ok(ctx.make_inst(Mnemonic::Divs, Some(Size::Word), None, vec![
            Operand::Ea(ea),
            Operand::Ea(EffectiveAddress::DataDirect(dn)),
        ]));
    }

    // OR <ea>,Dn / OR Dn,<ea>
    let size = match op_mode & 0x3 {
        0 => Size::Byte,
        1 => Size::Word,
        2 => Size::Long,
        _ => return Ok(make_dc_word(ctx, opcode)),
    };
    let ea = ctx.decode_ea(mode, reg, size)?;
    let direction = (op_mode >> 2) & 1;
    if direction == 0 {
        Ok(ctx.make_inst(Mnemonic::Or, Some(size), None, vec![
            Operand::Ea(ea),
            Operand::Ea(EffectiveAddress::DataDirect(dn)),
        ]))
    } else {
        Ok(ctx.make_inst(Mnemonic::Or, Some(size), None, vec![
            Operand::Ea(EffectiveAddress::DataDirect(dn)),
            Operand::Ea(ea),
        ]))
    }
}

// ─── Group 9: SUB / SUBA / SUBX ─────────────────────────────────

fn decode_group9(ctx: &mut DecodeCtx<'_>, opcode: u16) -> Result<Instruction, DecodeError> {
    let dn = ((opcode >> 9) & 0x7) as u8;
    let op_mode = (opcode >> 6) & 0x7;
    let mode = ((opcode >> 3) & 0x7) as u8;
    let reg = (opcode & 0x7) as u8;

    // SUBX
    if matches!(op_mode, 4 | 5 | 6) && (mode == 0 || mode == 1) {
        let size = match op_mode {
            4 => Size::Byte,
            5 => Size::Word,
            6 => Size::Long,
            _ => unreachable!(),
        };
        let (src, dst) = if mode == 0 {
            (
                Operand::Ea(EffectiveAddress::DataDirect(reg)),
                Operand::Ea(EffectiveAddress::DataDirect(dn)),
            )
        } else {
            (
                Operand::Ea(EffectiveAddress::AddressPreDecrement(reg)),
                Operand::Ea(EffectiveAddress::AddressPreDecrement(dn)),
            )
        };
        return Ok(ctx.make_inst(Mnemonic::Subx, Some(size), None, vec![src, dst]));
    }

    // SUBA <ea>,An
    if op_mode == 3 {
        let ea = ctx.decode_ea(mode, reg, Size::Word)?;
        return Ok(ctx.make_inst(Mnemonic::Suba, Some(Size::Word), None, vec![
            Operand::Ea(ea),
            Operand::Ea(EffectiveAddress::AddressDirect(dn)),
        ]));
    }
    if op_mode == 7 {
        let ea = ctx.decode_ea(mode, reg, Size::Long)?;
        return Ok(ctx.make_inst(Mnemonic::Suba, Some(Size::Long), None, vec![
            Operand::Ea(ea),
            Operand::Ea(EffectiveAddress::AddressDirect(dn)),
        ]));
    }

    // SUB <ea>,Dn / SUB Dn,<ea>
    let size = match op_mode & 0x3 {
        0 => Size::Byte,
        1 => Size::Word,
        2 => Size::Long,
        _ => return Ok(make_dc_word(ctx, opcode)),
    };
    let ea = ctx.decode_ea(mode, reg, size)?;
    let direction = (op_mode >> 2) & 1;
    if direction == 0 {
        Ok(ctx.make_inst(Mnemonic::Sub, Some(size), None, vec![
            Operand::Ea(ea),
            Operand::Ea(EffectiveAddress::DataDirect(dn)),
        ]))
    } else {
        Ok(ctx.make_inst(Mnemonic::Sub, Some(size), None, vec![
            Operand::Ea(EffectiveAddress::DataDirect(dn)),
            Operand::Ea(ea),
        ]))
    }
}

// ─── Group B: CMP / CMPA / CMPM / EOR ───────────────────────────

fn decode_group_b(ctx: &mut DecodeCtx<'_>, opcode: u16) -> Result<Instruction, DecodeError> {
    let dn = ((opcode >> 9) & 0x7) as u8;
    let op_mode = (opcode >> 6) & 0x7;
    let mode = ((opcode >> 3) & 0x7) as u8;
    let reg = (opcode & 0x7) as u8;

    // CMPA
    if op_mode == 3 {
        let ea = ctx.decode_ea(mode, reg, Size::Word)?;
        return Ok(ctx.make_inst(Mnemonic::Cmpa, Some(Size::Word), None, vec![
            Operand::Ea(ea),
            Operand::Ea(EffectiveAddress::AddressDirect(dn)),
        ]));
    }
    if op_mode == 7 {
        let ea = ctx.decode_ea(mode, reg, Size::Long)?;
        return Ok(ctx.make_inst(Mnemonic::Cmpa, Some(Size::Long), None, vec![
            Operand::Ea(ea),
            Operand::Ea(EffectiveAddress::AddressDirect(dn)),
        ]));
    }

    // CMPM (An)+,(An)+
    if matches!(op_mode, 4 | 5 | 6) && mode == 1 {
        let size = match op_mode {
            4 => Size::Byte,
            5 => Size::Word,
            6 => Size::Long,
            _ => unreachable!(),
        };
        return Ok(ctx.make_inst(Mnemonic::Cmpm, Some(size), None, vec![
            Operand::Ea(EffectiveAddress::AddressPostIncrement(reg)),
            Operand::Ea(EffectiveAddress::AddressPostIncrement(dn)),
        ]));
    }

    // EOR Dn,<ea>
    if matches!(op_mode, 4 | 5 | 6) {
        let size = match op_mode {
            4 => Size::Byte,
            5 => Size::Word,
            6 => Size::Long,
            _ => unreachable!(),
        };
        let ea = ctx.decode_ea(mode, reg, size)?;
        return Ok(ctx.make_inst(Mnemonic::Eor, Some(size), None, vec![
            Operand::Ea(EffectiveAddress::DataDirect(dn)),
            Operand::Ea(ea),
        ]));
    }

    // CMP <ea>,Dn
    let size = match op_mode & 0x3 {
        0 => Size::Byte,
        1 => Size::Word,
        2 => Size::Long,
        _ => return Ok(make_dc_word(ctx, opcode)),
    };
    let ea = ctx.decode_ea(mode, reg, size)?;
    Ok(ctx.make_inst(Mnemonic::Cmp, Some(size), None, vec![
        Operand::Ea(ea),
        Operand::Ea(EffectiveAddress::DataDirect(dn)),
    ]))
}

// ─── Group C: AND / MULU / MULS / ABCD / EXG ────────────────────

fn decode_group_c(ctx: &mut DecodeCtx<'_>, opcode: u16) -> Result<Instruction, DecodeError> {
    let dn = ((opcode >> 9) & 0x7) as u8;
    let op_mode = (opcode >> 6) & 0x7;
    let mode = ((opcode >> 3) & 0x7) as u8;
    let reg = (opcode & 0x7) as u8;

    // ABCD
    if op_mode == 4 && mode <= 1 {
        let (src, dst) = if mode == 0 {
            (
                Operand::Ea(EffectiveAddress::DataDirect(reg)),
                Operand::Ea(EffectiveAddress::DataDirect(dn)),
            )
        } else {
            (
                Operand::Ea(EffectiveAddress::AddressPreDecrement(reg)),
                Operand::Ea(EffectiveAddress::AddressPreDecrement(dn)),
            )
        };
        return Ok(ctx.make_inst(Mnemonic::Abcd, Some(Size::Byte), None, vec![src, dst]));
    }

    // EXG
    if op_mode == 5 && mode == 0 {
        // EXG Dx,Dy
        return Ok(ctx.make_inst(Mnemonic::Exg, Some(Size::Long), None, vec![
            Operand::Ea(EffectiveAddress::DataDirect(dn)),
            Operand::Ea(EffectiveAddress::DataDirect(reg)),
        ]));
    }
    if op_mode == 5 && mode == 1 {
        // EXG Ax,Ay
        return Ok(ctx.make_inst(Mnemonic::Exg, Some(Size::Long), None, vec![
            Operand::Ea(EffectiveAddress::AddressDirect(dn)),
            Operand::Ea(EffectiveAddress::AddressDirect(reg)),
        ]));
    }
    if op_mode == 6 && mode == 1 {
        // EXG Dx,Ay
        return Ok(ctx.make_inst(Mnemonic::Exg, Some(Size::Long), None, vec![
            Operand::Ea(EffectiveAddress::DataDirect(dn)),
            Operand::Ea(EffectiveAddress::AddressDirect(reg)),
        ]));
    }

    // MULU <ea>,Dn
    if op_mode == 3 {
        let ea = ctx.decode_ea(mode, reg, Size::Word)?;
        return Ok(ctx.make_inst(Mnemonic::Mulu, Some(Size::Word), None, vec![
            Operand::Ea(ea),
            Operand::Ea(EffectiveAddress::DataDirect(dn)),
        ]));
    }

    // MULS <ea>,Dn
    if op_mode == 7 {
        let ea = ctx.decode_ea(mode, reg, Size::Word)?;
        return Ok(ctx.make_inst(Mnemonic::Muls, Some(Size::Word), None, vec![
            Operand::Ea(ea),
            Operand::Ea(EffectiveAddress::DataDirect(dn)),
        ]));
    }

    // AND <ea>,Dn / AND Dn,<ea>
    let size = match op_mode & 0x3 {
        0 => Size::Byte,
        1 => Size::Word,
        2 => Size::Long,
        _ => return Ok(make_dc_word(ctx, opcode)),
    };
    let ea = ctx.decode_ea(mode, reg, size)?;
    let direction = (op_mode >> 2) & 1;
    if direction == 0 {
        Ok(ctx.make_inst(Mnemonic::And, Some(size), None, vec![
            Operand::Ea(ea),
            Operand::Ea(EffectiveAddress::DataDirect(dn)),
        ]))
    } else {
        Ok(ctx.make_inst(Mnemonic::And, Some(size), None, vec![
            Operand::Ea(EffectiveAddress::DataDirect(dn)),
            Operand::Ea(ea),
        ]))
    }
}

// ─── Group D: ADD / ADDA / ADDX ─────────────────────────────────

fn decode_group_d(ctx: &mut DecodeCtx<'_>, opcode: u16) -> Result<Instruction, DecodeError> {
    let dn = ((opcode >> 9) & 0x7) as u8;
    let op_mode = (opcode >> 6) & 0x7;
    let mode = ((opcode >> 3) & 0x7) as u8;
    let reg = (opcode & 0x7) as u8;

    // ADDX
    if matches!(op_mode, 4 | 5 | 6) && (mode == 0 || mode == 1) {
        let size = match op_mode {
            4 => Size::Byte,
            5 => Size::Word,
            6 => Size::Long,
            _ => unreachable!(),
        };
        let (src, dst) = if mode == 0 {
            (
                Operand::Ea(EffectiveAddress::DataDirect(reg)),
                Operand::Ea(EffectiveAddress::DataDirect(dn)),
            )
        } else {
            (
                Operand::Ea(EffectiveAddress::AddressPreDecrement(reg)),
                Operand::Ea(EffectiveAddress::AddressPreDecrement(dn)),
            )
        };
        return Ok(ctx.make_inst(Mnemonic::Addx, Some(size), None, vec![src, dst]));
    }

    // ADDA
    if op_mode == 3 {
        let ea = ctx.decode_ea(mode, reg, Size::Word)?;
        return Ok(ctx.make_inst(Mnemonic::Adda, Some(Size::Word), None, vec![
            Operand::Ea(ea),
            Operand::Ea(EffectiveAddress::AddressDirect(dn)),
        ]));
    }
    if op_mode == 7 {
        let ea = ctx.decode_ea(mode, reg, Size::Long)?;
        return Ok(ctx.make_inst(Mnemonic::Adda, Some(Size::Long), None, vec![
            Operand::Ea(ea),
            Operand::Ea(EffectiveAddress::AddressDirect(dn)),
        ]));
    }

    // ADD <ea>,Dn / ADD Dn,<ea>
    let size = match op_mode & 0x3 {
        0 => Size::Byte,
        1 => Size::Word,
        2 => Size::Long,
        _ => return Ok(make_dc_word(ctx, opcode)),
    };
    let ea = ctx.decode_ea(mode, reg, size)?;
    let direction = (op_mode >> 2) & 1;
    if direction == 0 {
        Ok(ctx.make_inst(Mnemonic::Add, Some(size), None, vec![
            Operand::Ea(ea),
            Operand::Ea(EffectiveAddress::DataDirect(dn)),
        ]))
    } else {
        Ok(ctx.make_inst(Mnemonic::Add, Some(size), None, vec![
            Operand::Ea(EffectiveAddress::DataDirect(dn)),
            Operand::Ea(ea),
        ]))
    }
}

// ─── Group E: Shifts / Rotates ───────────────────────────────────

fn decode_group_e(ctx: &mut DecodeCtx<'_>, opcode: u16) -> Result<Instruction, DecodeError> {
    let size_bits = (opcode >> 6) & 0x3;

    if size_bits == 3 {
        // Memory shift/rotate (word-size, shift by 1)
        let mode = ((opcode >> 3) & 0x7) as u8;
        let reg = (opcode & 0x7) as u8;
        let shift_type = (opcode >> 9) & 0x3;
        let direction = (opcode >> 8) & 1;

        let mnemonic = match (shift_type, direction) {
            (0, 0) => Mnemonic::Asr,
            (0, 1) => Mnemonic::Asl,
            (1, 0) => Mnemonic::Lsr,
            (1, 1) => Mnemonic::Lsl,
            (2, 0) => Mnemonic::Roxr,
            (2, 1) => Mnemonic::Roxl,
            (3, 0) => Mnemonic::Ror,
            (3, 1) => Mnemonic::Rol,
            _ => unreachable!(),
        };

        let ea = ctx.decode_ea(mode, reg, Size::Word)?;
        return Ok(ctx.make_inst(mnemonic, Some(Size::Word), None, vec![Operand::Ea(ea)]));
    }

    // Register shift/rotate
    let size = decode_size_2bit(size_bits)?;
    let count_or_reg = ((opcode >> 9) & 0x7) as u8;
    let direction = (opcode >> 8) & 1;
    let ir = (opcode >> 5) & 1; // 0 = immediate count, 1 = register count
    let shift_type = (opcode >> 3) & 0x3;
    let reg = (opcode & 0x7) as u8;

    let mnemonic = match (shift_type, direction) {
        (0, 0) => Mnemonic::Asr,
        (0, 1) => Mnemonic::Asl,
        (1, 0) => Mnemonic::Lsr,
        (1, 1) => Mnemonic::Lsl,
        (2, 0) => Mnemonic::Roxr,
        (2, 1) => Mnemonic::Roxl,
        (3, 0) => Mnemonic::Ror,
        (3, 1) => Mnemonic::Rol,
        _ => unreachable!(),
    };

    let count_operand = if ir == 0 {
        let mut count = count_or_reg;
        if count == 0 {
            count = 8;
        }
        Operand::QuickImmediate(count)
    } else {
        Operand::Ea(EffectiveAddress::DataDirect(count_or_reg))
    };

    Ok(ctx.make_inst(mnemonic, Some(size), None, vec![
        count_operand,
        Operand::Ea(EffectiveAddress::DataDirect(reg)),
    ]))
}

// ─── Helpers ─────────────────────────────────────────────────────

fn decode_size_2bit(bits: u16) -> Result<Size, DecodeError> {
    match bits {
        0 => Ok(Size::Byte),
        1 => Ok(Size::Word),
        2 => Ok(Size::Long),
        _ => Err(DecodeError::UnknownOpcode {
            address: 0,
            opcode: 0,
        }),
    }
}

fn read_immediate(ctx: &mut DecodeCtx<'_>, size: Size) -> Result<u32, DecodeError> {
    match size {
        Size::Byte => {
            let w = ctx.read_u16()?;
            Ok((w & 0xFF) as u32)
        }
        Size::Word => Ok(ctx.read_u16()? as u32),
        Size::Long => ctx.read_u32(),
    }
}

fn make_dc_word(ctx: &DecodeCtx<'_>, opcode: u16) -> Instruction {
    ctx.make_inst(Mnemonic::Dc, Some(Size::Word), None, vec![
        Operand::Ea(EffectiveAddress::Immediate(opcode as u32)),
    ])
}

#[cfg(test)]
mod tests {
    use super::*;

    fn decode(bytes: &[u8]) -> Instruction {
        decode_instruction(bytes, 0, 0, CpuVariant::M68000).unwrap()
    }

    #[test]
    fn decode_nop() {
        let inst = decode(&[0x4E, 0x71]);
        assert_eq!(inst.mnemonic, Mnemonic::Nop);
        assert_eq!(inst.size_bytes, 2);
    }

    #[test]
    fn decode_rts() {
        let inst = decode(&[0x4E, 0x75]);
        assert_eq!(inst.mnemonic, Mnemonic::Rts);
        assert_eq!(inst.size_bytes, 2);
    }

    #[test]
    fn decode_rte() {
        let inst = decode(&[0x4E, 0x73]);
        assert_eq!(inst.mnemonic, Mnemonic::Rte);
    }

    #[test]
    fn decode_moveq_positive() {
        let inst = decode(&[0x70, 0x2A]);
        assert_eq!(inst.mnemonic, Mnemonic::Moveq);
        assert_eq!(inst.operands[0], Operand::MoveqImmediate(42));
        assert_eq!(
            inst.operands[1],
            Operand::Ea(EffectiveAddress::DataDirect(0))
        );
    }

    #[test]
    fn decode_moveq_negative() {
        // MOVEQ #-1,D3 = 0x76FF
        let inst = decode(&[0x76, 0xFF]);
        assert_eq!(inst.mnemonic, Mnemonic::Moveq);
        assert_eq!(inst.operands[0], Operand::MoveqImmediate(-1));
        assert_eq!(
            inst.operands[1],
            Operand::Ea(EffectiveAddress::DataDirect(3))
        );
    }

    #[test]
    fn decode_move_d0_d1() {
        // MOVE.L D0,D1 = 0x2200
        let inst = decode(&[0x22, 0x00]);
        assert_eq!(inst.mnemonic, Mnemonic::Move);
        assert_eq!(inst.size, Some(Size::Long));
        assert_eq!(
            inst.operands[0],
            Operand::Ea(EffectiveAddress::DataDirect(0))
        );
        assert_eq!(
            inst.operands[1],
            Operand::Ea(EffectiveAddress::DataDirect(1))
        );
    }

    #[test]
    fn decode_movea() {
        // MOVEA.L D0,A0 = 0x2040
        let inst = decode(&[0x20, 0x40]);
        assert_eq!(inst.mnemonic, Mnemonic::Movea);
        assert_eq!(inst.size, Some(Size::Long));
    }

    #[test]
    fn decode_jsr_a6_displacement() {
        // JSR (-552,A6) = 0x4EAE 0xFDD8
        let inst = decode(&[0x4E, 0xAE, 0xFD, 0xD8]);
        assert_eq!(inst.mnemonic, Mnemonic::Jsr);
        assert_eq!(inst.size_bytes, 4);
        assert_eq!(
            inst.operands[0],
            Operand::Ea(EffectiveAddress::AddressDisplacement(6, -552))
        );
    }

    #[test]
    fn decode_lea_displacement() {
        // LEA (8,A5),A0 = 0x41ED 0x0008
        let inst = decode(&[0x41, 0xED, 0x00, 0x08]);
        assert_eq!(inst.mnemonic, Mnemonic::Lea);
        assert_eq!(
            inst.operands[0],
            Operand::Ea(EffectiveAddress::AddressDisplacement(5, 8))
        );
        assert_eq!(
            inst.operands[1],
            Operand::Ea(EffectiveAddress::AddressDirect(0))
        );
    }

    #[test]
    fn decode_bra_short() {
        // BRA.S *+$10 = 0x600E (8-bit displacement = 14)
        let inst = decode(&[0x60, 0x0E]);
        assert_eq!(inst.mnemonic, Mnemonic::Bra);
        assert_eq!(inst.size, Some(Size::Byte));
        assert_eq!(inst.operands[0], Operand::Displacement8(14));
    }

    #[test]
    fn decode_beq_word() {
        // BEQ.W *+$100 = 0x6700 0x00FE (16-bit displacement)
        let inst = decode(&[0x67, 0x00, 0x00, 0xFE]);
        assert_eq!(inst.mnemonic, Mnemonic::Bcc);
        assert_eq!(inst.condition, Some(Condition::Eq));
        assert_eq!(inst.operands[0], Operand::Displacement16(0xFE));
    }

    #[test]
    fn decode_addq_immediate() {
        // ADDQ.L #1,A7 = 0x5E8F
        let inst = decode(&[0x5E, 0x8F]);
        assert_eq!(inst.mnemonic, Mnemonic::Addq);
        assert_eq!(inst.size, Some(Size::Long));
        assert_eq!(inst.operands[0], Operand::QuickImmediate(7));
    }

    #[test]
    fn decode_trap() {
        // TRAP #0 = 0x4E40
        let inst = decode(&[0x4E, 0x40]);
        assert_eq!(inst.mnemonic, Mnemonic::Trap);
        assert_eq!(inst.operands[0], Operand::TrapVector(0));

        // TRAP #15 = 0x4E4F
        let inst2 = decode(&[0x4E, 0x4F]);
        assert_eq!(inst2.operands[0], Operand::TrapVector(15));
    }

    #[test]
    fn decode_link() {
        // LINK A6,#-4 = 0x4E56 0xFFFC
        let inst = decode(&[0x4E, 0x56, 0xFF, 0xFC]);
        assert_eq!(inst.mnemonic, Mnemonic::Link);
        assert_eq!(
            inst.operands[0],
            Operand::Ea(EffectiveAddress::AddressDirect(6))
        );
    }

    #[test]
    fn decode_unlk() {
        // UNLK A6 = 0x4E5E
        let inst = decode(&[0x4E, 0x5E]);
        assert_eq!(inst.mnemonic, Mnemonic::Unlk);
    }

    #[test]
    fn decode_clr() {
        // CLR.L D0 = 0x4280
        let inst = decode(&[0x42, 0x80]);
        assert_eq!(inst.mnemonic, Mnemonic::Clr);
        assert_eq!(inst.size, Some(Size::Long));
    }

    #[test]
    fn decode_tst() {
        // TST.W D0 = 0x4A40
        let inst = decode(&[0x4A, 0x40]);
        assert_eq!(inst.mnemonic, Mnemonic::Tst);
        assert_eq!(inst.size, Some(Size::Word));
    }

    #[test]
    fn decode_swap() {
        // SWAP D3 = 0x4843
        let inst = decode(&[0x48, 0x43]);
        assert_eq!(inst.mnemonic, Mnemonic::Swap);
    }

    #[test]
    fn decode_lsl_immediate() {
        // LSL.W #3,D0 = 0xE748
        let inst = decode(&[0xE7, 0x48]);
        assert_eq!(inst.mnemonic, Mnemonic::Lsl);
        assert_eq!(inst.size, Some(Size::Word));
        assert_eq!(inst.operands[0], Operand::QuickImmediate(3));
        assert_eq!(
            inst.operands[1],
            Operand::Ea(EffectiveAddress::DataDirect(0))
        );
    }

    #[test]
    fn decode_dbf() {
        // DBF D0,*-2 = 0x51C8 0xFFFC
        let inst = decode(&[0x51, 0xC8, 0xFF, 0xFC]);
        assert_eq!(inst.mnemonic, Mnemonic::Dbcc);
        assert_eq!(inst.condition, Some(Condition::False));
    }

    #[test]
    fn decode_move_abs_long() {
        // MOVE.L $4.w,A6 = 0x2C78 0x0004
        let inst = decode(&[0x2C, 0x78, 0x00, 0x04]);
        assert_eq!(inst.mnemonic, Mnemonic::Movea);
        assert_eq!(inst.size, Some(Size::Long));
        assert_eq!(
            inst.operands[0],
            Operand::Ea(EffectiveAddress::AbsoluteShort(4))
        );
    }

    #[test]
    fn decode_cmpi_byte() {
        // CMPI.B #$0A,D0 = 0x0C00 0x000A
        let inst = decode(&[0x0C, 0x00, 0x00, 0x0A]);
        assert_eq!(inst.mnemonic, Mnemonic::Cmpi);
        assert_eq!(inst.size, Some(Size::Byte));
        assert_eq!(
            inst.operands[0],
            Operand::Ea(EffectiveAddress::Immediate(0x0A))
        );
    }

    #[test]
    fn decode_movem_to_stack() {
        // MOVEM.L D0-D7/A0-A6,-(A7) = 0x48E7 0xFFFE
        let inst = decode(&[0x48, 0xE7, 0xFF, 0xFE]);
        assert_eq!(inst.mnemonic, Mnemonic::Movem);
        assert_eq!(inst.size, Some(Size::Long));
    }

    #[test]
    fn decode_add_immediate() {
        // ADDI.L #$1000,D0 = 0x0680 0x0000 0x1000
        let inst = decode(&[0x06, 0x80, 0x00, 0x00, 0x10, 0x00]);
        assert_eq!(inst.mnemonic, Mnemonic::Addi);
        assert_eq!(inst.size, Some(Size::Long));
        assert_eq!(
            inst.operands[0],
            Operand::Ea(EffectiveAddress::Immediate(0x1000))
        );
    }

    #[test]
    fn decode_unknown_returns_dc() {
        // A-line trap should be dc.w
        let inst = decode(&[0xA0, 0x00]);
        assert_eq!(inst.mnemonic, Mnemonic::Dc);
    }
}
