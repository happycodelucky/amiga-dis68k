use super::addressing::EffectiveAddress;
use super::variants::CpuVariant;

/// Operation size suffix (.b, .w, .l).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Size {
    Byte,
    Word,
    Long,
}

impl Size {
    /// Size suffix for Motorola syntax.
    pub fn suffix(&self) -> &'static str {
        match self {
            Size::Byte => ".b",
            Size::Word => ".w",
            Size::Long => ".l",
        }
    }

    /// Number of bytes this size occupies.
    pub fn bytes(&self) -> u8 {
        match self {
            Size::Byte => 1,
            Size::Word => 2,
            Size::Long => 4,
        }
    }
}

/// Condition codes for Bcc, DBcc, Scc, and TRAPcc instructions.
///
/// These correspond to the 4-bit condition field (bits 11-8) in
/// the opcode word.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Condition {
    /// 0000 — True (always)
    True,
    /// 0001 — False (never)
    False,
    /// 0010 — High (!C & !Z)
    Hi,
    /// 0011 — Low or Same (C | Z)
    Ls,
    /// 0100 — Carry Clear / High or Same (!C)
    Cc,
    /// 0101 — Carry Set / Low (C)
    Cs,
    /// 0110 — Not Equal (!Z)
    Ne,
    /// 0111 — Equal (Z)
    Eq,
    /// 1000 — Overflow Clear (!V)
    Vc,
    /// 1001 — Overflow Set (V)
    Vs,
    /// 1010 — Plus (!N)
    Pl,
    /// 1011 — Minus (N)
    Mi,
    /// 1100 — Greater or Equal
    Ge,
    /// 1101 — Less Than
    Lt,
    /// 1110 — Greater Than
    Gt,
    /// 1111 — Less or Equal
    Le,
}

impl Condition {
    pub fn from_bits(bits: u8) -> Self {
        match bits & 0xF {
            0x0 => Condition::True,
            0x1 => Condition::False,
            0x2 => Condition::Hi,
            0x3 => Condition::Ls,
            0x4 => Condition::Cc,
            0x5 => Condition::Cs,
            0x6 => Condition::Ne,
            0x7 => Condition::Eq,
            0x8 => Condition::Vc,
            0x9 => Condition::Vs,
            0xA => Condition::Pl,
            0xB => Condition::Mi,
            0xC => Condition::Ge,
            0xD => Condition::Lt,
            0xE => Condition::Gt,
            0xF => Condition::Le,
            _ => unreachable!(),
        }
    }

    pub fn suffix(&self) -> &'static str {
        match self {
            Condition::True => "t",
            Condition::False => "f",
            Condition::Hi => "hi",
            Condition::Ls => "ls",
            Condition::Cc => "cc",
            Condition::Cs => "cs",
            Condition::Ne => "ne",
            Condition::Eq => "eq",
            Condition::Vc => "vc",
            Condition::Vs => "vs",
            Condition::Pl => "pl",
            Condition::Mi => "mi",
            Condition::Ge => "ge",
            Condition::Lt => "lt",
            Condition::Gt => "gt",
            Condition::Le => "le",
        }
    }
}

/// All 68k instruction mnemonics.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Mnemonic {
    // Data movement
    Move,
    Movea,
    Moveq,
    Movem,
    Movep,
    MoveFromSr,
    MoveToCcr,
    MoveToSr,
    MoveUsp,

    // Arithmetic
    Add,
    Adda,
    Addi,
    Addq,
    Addx,
    Sub,
    Suba,
    Subi,
    Subq,
    Subx,
    Muls,
    Mulu,
    Divs,
    Divu,
    Neg,
    Negx,
    Ext,
    Extb,  // 68020+ byte-to-long sign extend
    Clr,

    // Compare/test
    Cmp,
    Cmpa,
    Cmpi,
    Cmpm,
    Tst,

    // Logic
    And,
    Andi,
    Or,
    Ori,
    Eor,
    Eori,
    Not,

    // Shift/rotate
    Lsl,
    Lsr,
    Asl,
    Asr,
    Rol,
    Ror,
    Roxl,
    Roxr,

    // Bit manipulation
    Btst,
    Bset,
    Bclr,
    Bchg,

    // Bit field (68020+)
    Bftst,
    Bfextu,
    Bfchg,
    Bfexts,
    Bfclr,
    Bfffo,
    Bfset,
    Bfins,

    // BCD
    Abcd,
    Sbcd,
    Nbcd,

    // Program control
    Bra,
    Bsr,
    Bcc,
    Dbcc,
    Scc,
    Jmp,
    Jsr,
    Rts,
    Rte,
    Rtr,
    Nop,
    Illegal,
    Trap,
    Trapcc,  // 68020+ conditional trap
    Trapv,
    Stop,

    // Stack/frame
    Link,
    Unlk,
    Pea,
    Lea,

    // Miscellaneous
    Exg,
    Swap,
    Tas,
    Chk,
    Chk2,    // 68020+ check against bounds pair
    Cmp2,    // 68020+ compare against bounds pair
    Cas,     // 68020+ compare and swap
    Cas2,    // 68020+ compare and swap dual
    Pack,    // 68020+ BCD pack
    Unpk,    // 68020+ BCD unpack
    Movec,   // 68010+ move control register
    Moves,   // 68010+ move address space
    Rtd,     // 68010+ return and deallocate
    Reset,

    // Pseudo-instruction for unrecognized data
    Dc,
    TrapA,   // A-line trap (Amiga system calls)
}

impl Mnemonic {
    pub fn name(&self) -> &'static str {
        match self {
            Mnemonic::Move => "move",
            Mnemonic::Movea => "movea",
            Mnemonic::Moveq => "moveq",
            Mnemonic::Movem => "movem",
            Mnemonic::Movep => "movep",
            Mnemonic::MoveFromSr => "move",
            Mnemonic::MoveToCcr => "move",
            Mnemonic::MoveToSr => "move",
            Mnemonic::MoveUsp => "move",
            Mnemonic::Add => "add",
            Mnemonic::Adda => "adda",
            Mnemonic::Addi => "addi",
            Mnemonic::Addq => "addq",
            Mnemonic::Addx => "addx",
            Mnemonic::Sub => "sub",
            Mnemonic::Suba => "suba",
            Mnemonic::Subi => "subi",
            Mnemonic::Subq => "subq",
            Mnemonic::Subx => "subx",
            Mnemonic::Muls => "muls",
            Mnemonic::Mulu => "mulu",
            Mnemonic::Divs => "divs",
            Mnemonic::Divu => "divu",
            Mnemonic::Neg => "neg",
            Mnemonic::Negx => "negx",
            Mnemonic::Ext => "ext",
            Mnemonic::Extb => "extb",
            Mnemonic::Clr => "clr",
            Mnemonic::Cmp => "cmp",
            Mnemonic::Cmpa => "cmpa",
            Mnemonic::Cmpi => "cmpi",
            Mnemonic::Cmpm => "cmpm",
            Mnemonic::Tst => "tst",
            Mnemonic::And => "and",
            Mnemonic::Andi => "andi",
            Mnemonic::Or => "or",
            Mnemonic::Ori => "ori",
            Mnemonic::Eor => "eor",
            Mnemonic::Eori => "eori",
            Mnemonic::Not => "not",
            Mnemonic::Lsl => "lsl",
            Mnemonic::Lsr => "lsr",
            Mnemonic::Asl => "asl",
            Mnemonic::Asr => "asr",
            Mnemonic::Rol => "rol",
            Mnemonic::Ror => "ror",
            Mnemonic::Roxl => "roxl",
            Mnemonic::Roxr => "roxr",
            Mnemonic::Btst => "btst",
            Mnemonic::Bset => "bset",
            Mnemonic::Bclr => "bclr",
            Mnemonic::Bchg => "bchg",
            Mnemonic::Bftst => "bftst",
            Mnemonic::Bfextu => "bfextu",
            Mnemonic::Bfchg => "bfchg",
            Mnemonic::Bfexts => "bfexts",
            Mnemonic::Bfclr => "bfclr",
            Mnemonic::Bfffo => "bfffo",
            Mnemonic::Bfset => "bfset",
            Mnemonic::Bfins => "bfins",
            Mnemonic::Abcd => "abcd",
            Mnemonic::Sbcd => "sbcd",
            Mnemonic::Nbcd => "nbcd",
            Mnemonic::Bra => "bra",
            Mnemonic::Bsr => "bsr",
            Mnemonic::Bcc => "b",
            Mnemonic::Dbcc => "db",
            Mnemonic::Scc => "s",
            Mnemonic::Jmp => "jmp",
            Mnemonic::Jsr => "jsr",
            Mnemonic::Rts => "rts",
            Mnemonic::Rte => "rte",
            Mnemonic::Rtr => "rtr",
            Mnemonic::Nop => "nop",
            Mnemonic::Illegal => "illegal",
            Mnemonic::Trap => "trap",
            Mnemonic::Trapcc => "trap",
            Mnemonic::Trapv => "trapv",
            Mnemonic::Stop => "stop",
            Mnemonic::Link => "link",
            Mnemonic::Unlk => "unlk",
            Mnemonic::Pea => "pea",
            Mnemonic::Lea => "lea",
            Mnemonic::Exg => "exg",
            Mnemonic::Swap => "swap",
            Mnemonic::Tas => "tas",
            Mnemonic::Chk => "chk",
            Mnemonic::Chk2 => "chk2",
            Mnemonic::Cmp2 => "cmp2",
            Mnemonic::Cas => "cas",
            Mnemonic::Cas2 => "cas2",
            Mnemonic::Pack => "pack",
            Mnemonic::Unpk => "unpk",
            Mnemonic::Movec => "movec",
            Mnemonic::Moves => "moves",
            Mnemonic::Rtd => "rtd",
            Mnemonic::Reset => "reset",
            Mnemonic::Dc => "dc",
            Mnemonic::TrapA => "trapa",
        }
    }

    /// Returns true if this mnemonic takes a condition code suffix.
    pub fn is_conditional(&self) -> bool {
        matches!(self, Mnemonic::Bcc | Mnemonic::Dbcc | Mnemonic::Scc | Mnemonic::Trapcc)
    }
}

/// An operand of a decoded instruction.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Operand {
    /// An effective address operand.
    Ea(EffectiveAddress),
    /// A register list for MOVEM (bitmask of d0-d7/a0-a7).
    RegisterList(u16),
    /// Quick immediate for ADDQ/SUBQ (1-8).
    QuickImmediate(u8),
    /// MOVEQ immediate (signed 8-bit, sign-extended to 32).
    MoveqImmediate(i8),
    /// 8-bit branch displacement (byte-size Bcc/BRA/BSR).
    Displacement8(i8),
    /// 16-bit branch displacement.
    Displacement16(i16),
    /// 32-bit branch displacement (68020+ long branch).
    Displacement32(i32),
    /// TRAP vector number (0-15).
    TrapVector(u8),
    /// Condition code register (CCR).
    Ccr,
    /// Status register (SR).
    Sr,
    /// User stack pointer (USP).
    Usp,
    /// Bit field specifier {offset:width} (68020+).
    BitField {
        offset: BitFieldParam,
        width: BitFieldParam,
    },
}

/// A bit field parameter — either an immediate value or a data register.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BitFieldParam {
    /// Immediate value (0-31 for offset, 1-32 for width where 0 encodes 32).
    Immediate(u8),
    /// Data register D0-D7.
    Register(u8),
}

/// A fully decoded 68k instruction.
#[derive(Debug, Clone)]
pub struct Instruction {
    /// Address of this instruction within the hunk.
    pub address: u32,
    /// Total instruction length in bytes.
    pub size_bytes: u8,
    /// The raw instruction bytes.
    pub raw_bytes: Vec<u8>,
    /// The operation mnemonic.
    pub mnemonic: Mnemonic,
    /// Operation size (.b/.w/.l), if applicable.
    pub size: Option<Size>,
    /// Condition code for Bcc/Scc/DBcc.
    pub condition: Option<Condition>,
    /// Operands (typically 0-2).
    pub operands: Vec<Operand>,
    /// Minimum CPU variant required for this instruction.
    pub cpu_required: CpuVariant,
}
