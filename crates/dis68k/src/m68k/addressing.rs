use super::instruction::Size;

/// Effective address — represents one of the 68k's 14 addressing modes.
///
/// The 68k encodes effective addresses using a 3-bit mode field and a
/// 3-bit register field. Modes 0-6 use the register field directly;
/// mode 7 overloads the register field to select among absolute,
/// PC-relative, and immediate modes.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EffectiveAddress {
    /// Dn — data register direct
    DataDirect(u8),
    /// An — address register direct
    AddressDirect(u8),
    /// (An) — address register indirect
    AddressIndirect(u8),
    /// (An)+ — address register indirect with postincrement
    AddressPostIncrement(u8),
    /// -(An) — address register indirect with predecrement
    AddressPreDecrement(u8),
    /// (d16,An) — address register indirect with 16-bit displacement
    AddressDisplacement(u8, i16),
    /// (d8,An,Xn.size*scale) — address register indirect with index
    AddressIndex {
        reg: u8,
        index_reg: IndexRegister,
        index_size: Size,
        scale: u8,
        displacement: i8,
    },
    /// (xxx).W — absolute short (16-bit, sign-extended to 32)
    AbsoluteShort(u16),
    /// (xxx).L — absolute long (32-bit)
    AbsoluteLong(u32),
    /// (d16,PC) — PC-relative with 16-bit displacement
    PcDisplacement(i16),
    /// (d8,PC,Xn.size*scale) — PC-relative with index
    PcIndex {
        index_reg: IndexRegister,
        index_size: Size,
        scale: u8,
        displacement: i8,
    },
    /// #imm — immediate value
    Immediate(u32),
}

/// Identifies an index register used in indexed addressing modes.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IndexRegister {
    /// Data register D0-D7
    Data(u8),
    /// Address register A0-A7
    Address(u8),
}

impl std::fmt::Display for IndexRegister {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            IndexRegister::Data(n) => write!(f, "d{n}"),
            IndexRegister::Address(n) => write!(f, "a{n}"),
        }
    }
}
