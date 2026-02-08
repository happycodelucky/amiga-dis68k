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

    // ─── 68020+ Extended Addressing Modes ────────────────────────────────

    /// (bd,An,Xn.size*scale) — base displacement with scaled index (68020+)
    AddressBaseDisplacement {
        reg: u8,
        base_disp: i32,
        index_reg: Option<IndexRegister>,
        index_size: Option<Size>,
        scale: u8,
    },
    /// ([bd,An],Xn.size*scale,od) — memory indirect post-indexed (68020+)
    AddressMemoryIndirectPost {
        reg: Option<u8>,        // None = base register suppressed
        base_disp: i32,
        outer_disp: i32,
        index_reg: Option<IndexRegister>,
        index_size: Option<Size>,
        scale: u8,
    },
    /// ([bd,An,Xn.size*scale],od) — memory indirect pre-indexed (68020+)
    AddressMemoryIndirectPre {
        reg: Option<u8>,
        base_disp: i32,
        outer_disp: i32,
        index_reg: Option<IndexRegister>,
        index_size: Option<Size>,
        scale: u8,
    },
    /// (bd,PC,Xn.size*scale) — PC-relative base displacement with scaled index (68020+)
    PcBaseDisplacement {
        base_disp: i32,
        index_reg: Option<IndexRegister>,
        index_size: Option<Size>,
        scale: u8,
    },
    /// ([bd,PC],Xn.size*scale,od) — PC-relative memory indirect post-indexed (68020+)
    PcMemoryIndirectPost {
        base_disp: i32,
        outer_disp: i32,
        index_reg: Option<IndexRegister>,
        index_size: Option<Size>,
        scale: u8,
    },
    /// ([bd,PC,Xn.size*scale],od) — PC-relative memory indirect pre-indexed (68020+)
    PcMemoryIndirectPre {
        base_disp: i32,
        outer_disp: i32,
        index_reg: Option<IndexRegister>,
        index_size: Option<Size>,
        scale: u8,
    },
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

impl EffectiveAddress {
    /// Returns the minimum CPU variant required for this addressing mode.
    pub fn min_cpu(&self) -> super::variants::CpuVariant {
        use super::variants::CpuVariant;
        match self {
            // 68020+ extended addressing modes
            EffectiveAddress::AddressBaseDisplacement { .. }
            | EffectiveAddress::AddressMemoryIndirectPost { .. }
            | EffectiveAddress::AddressMemoryIndirectPre { .. }
            | EffectiveAddress::PcBaseDisplacement { .. }
            | EffectiveAddress::PcMemoryIndirectPost { .. }
            | EffectiveAddress::PcMemoryIndirectPre { .. } => CpuVariant::M68020,

            // All other modes are 68000
            _ => CpuVariant::M68000,
        }
    }
}
