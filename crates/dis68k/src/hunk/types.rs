/// Memory type flags from the upper 2 bits of the hunk size word.
///
/// The Amiga had separate memory regions: "chip" RAM was accessible by the
/// custom chips (Agnus/Alice) for DMA operations (graphics, audio, disk),
/// while "fast" RAM was CPU-only and typically faster.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MemoryType {
    /// No preference — allocate from any available memory.
    Any,
    /// Must be in chip RAM (DMA-accessible for graphics/audio/disk).
    Chip,
    /// Prefer fast RAM (CPU-only, not DMA-accessible).
    Fast,
    /// Extended memory attributes (both bits set, followed by additional spec).
    Extended(u32),
}

impl MemoryType {
    /// Decode memory type from the upper 2 bits of a hunk size or type word.
    pub fn from_flags(word: u32) -> Self {
        match (word >> 30) & 0x3 {
            0 => MemoryType::Any,
            1 => MemoryType::Fast,
            2 => MemoryType::Chip,
            _ => MemoryType::Extended(word),
        }
    }
}

/// Identifies the type of a hunk block in the executable.
///
/// Each hunk in an Amiga executable is tagged with a type ID (a 32-bit word
/// whose lower 30 bits identify the type). The primary content hunks are
/// CODE, DATA, and BSS; everything else attaches metadata to them.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HunkType {
    /// HUNK_HEADER (0x3F3) — file header, only at the start of an executable.
    Header,
    /// HUNK_CODE (0x3E9) — executable machine code.
    Code,
    /// HUNK_DATA (0x3EA) — initialized data (globals, constants).
    Data,
    /// HUNK_BSS (0x3EB) — uninitialized memory reservation (zeroed at load).
    Bss,
    /// HUNK_RELOC32 (0x3EC) — 32-bit absolute relocation table.
    Reloc32,
    /// HUNK_RELOC32SHORT (0x3FC) — compact 32-bit relocations with 16-bit offsets.
    Reloc32Short,
    /// HUNK_RELRELOC32 (0x3FD) — 32-bit PC-relative relocations (AmigaOS 2.0+).
    RelReloc32,
    /// HUNK_RELRELOC16 (0x3ED) — 16-bit PC-relative relocations.
    RelReloc16,
    /// HUNK_RELRELOC8 (0x3EE) — 8-bit PC-relative relocations.
    RelReloc8,
    /// HUNK_DREL32 (0x3F7) — 32-bit data-relative relocations.
    DReloc32,
    /// HUNK_DREL16 (0x3F8) — 16-bit data-relative relocations.
    DReloc16,
    /// HUNK_DREL8 (0x3F9) — 8-bit data-relative relocations.
    DReloc8,
    /// HUNK_SYMBOL (0x3F0) — symbol name/value pairs for debugging.
    Symbol,
    /// HUNK_EXT (0x3EF) — external references and definitions.
    Ext,
    /// HUNK_DEBUG (0x3F1) — debug information (line numbers, etc.).
    Debug,
    /// HUNK_END (0x3F2) — marks the end of a hunk.
    End,
    /// HUNK_OVERLAY (0x3F5) — overlay table for demand-loading.
    Overlay,
    /// HUNK_BREAK (0x3F6) — overlay break marker.
    Break,
    /// HUNK_NAME (0x3E8) — optional name for the hunk.
    Name,
    /// HUNK_UNIT (0x3E7) — unit marker in object files.
    Unit,
    /// HUNK_LIB (0x3FA) — library marker.
    Lib,
    /// HUNK_INDEX (0x3FB) — library index.
    Index,
    /// HUNK_ABSRELOC16 (0x3FE) — 16-bit absolute relocations.
    AbsReloc16,
}

/// Raw hunk type ID constants.
pub mod hunk_ids {
    pub const HUNK_UNIT: u32 = 0x3E7;
    pub const HUNK_NAME: u32 = 0x3E8;
    pub const HUNK_CODE: u32 = 0x3E9;
    pub const HUNK_DATA: u32 = 0x3EA;
    pub const HUNK_BSS: u32 = 0x3EB;
    pub const HUNK_RELOC32: u32 = 0x3EC;
    pub const HUNK_RELRELOC16: u32 = 0x3ED;
    pub const HUNK_RELRELOC8: u32 = 0x3EE;
    pub const HUNK_EXT: u32 = 0x3EF;
    pub const HUNK_SYMBOL: u32 = 0x3F0;
    pub const HUNK_DEBUG: u32 = 0x3F1;
    pub const HUNK_END: u32 = 0x3F2;
    pub const HUNK_HEADER: u32 = 0x3F3;
    pub const HUNK_OVERLAY: u32 = 0x3F5;
    pub const HUNK_BREAK: u32 = 0x3F6;
    pub const HUNK_DREL32: u32 = 0x3F7;
    pub const HUNK_DREL16: u32 = 0x3F8;
    pub const HUNK_DREL8: u32 = 0x3F9;
    pub const HUNK_LIB: u32 = 0x3FA;
    pub const HUNK_INDEX: u32 = 0x3FB;
    pub const HUNK_RELOC32SHORT: u32 = 0x3FC;
    pub const HUNK_RELRELOC32: u32 = 0x3FD;
    pub const HUNK_ABSRELOC16: u32 = 0x3FE;
}

impl HunkType {
    /// Parse a hunk type from a raw 32-bit word, masking off memory flags.
    /// Returns `None` for unrecognized type IDs.
    pub fn from_raw(raw: u32) -> Option<Self> {
        // Mask off the upper 2 bits (memory type flags) and the advisory bit
        let id = raw & 0x3FFFFFFF;
        match id {
            hunk_ids::HUNK_UNIT => Some(HunkType::Unit),
            hunk_ids::HUNK_NAME => Some(HunkType::Name),
            hunk_ids::HUNK_CODE => Some(HunkType::Code),
            hunk_ids::HUNK_DATA => Some(HunkType::Data),
            hunk_ids::HUNK_BSS => Some(HunkType::Bss),
            hunk_ids::HUNK_RELOC32 => Some(HunkType::Reloc32),
            hunk_ids::HUNK_RELRELOC16 => Some(HunkType::RelReloc16),
            hunk_ids::HUNK_RELRELOC8 => Some(HunkType::RelReloc8),
            hunk_ids::HUNK_EXT => Some(HunkType::Ext),
            hunk_ids::HUNK_SYMBOL => Some(HunkType::Symbol),
            hunk_ids::HUNK_DEBUG => Some(HunkType::Debug),
            hunk_ids::HUNK_END => Some(HunkType::End),
            hunk_ids::HUNK_HEADER => Some(HunkType::Header),
            hunk_ids::HUNK_OVERLAY => Some(HunkType::Overlay),
            hunk_ids::HUNK_BREAK => Some(HunkType::Break),
            hunk_ids::HUNK_DREL32 => Some(HunkType::DReloc32),
            hunk_ids::HUNK_DREL16 => Some(HunkType::DReloc16),
            hunk_ids::HUNK_DREL8 => Some(HunkType::DReloc8),
            hunk_ids::HUNK_LIB => Some(HunkType::Lib),
            hunk_ids::HUNK_INDEX => Some(HunkType::Index),
            hunk_ids::HUNK_RELOC32SHORT => Some(HunkType::Reloc32Short),
            hunk_ids::HUNK_RELRELOC32 => Some(HunkType::RelReloc32),
            hunk_ids::HUNK_ABSRELOC16 => Some(HunkType::AbsReloc16),
            _ => None,
        }
    }

    /// Returns a human-readable name for this hunk type.
    pub fn name(&self) -> &'static str {
        match self {
            HunkType::Header => "HUNK_HEADER",
            HunkType::Code => "HUNK_CODE",
            HunkType::Data => "HUNK_DATA",
            HunkType::Bss => "HUNK_BSS",
            HunkType::Reloc32 => "HUNK_RELOC32",
            HunkType::Reloc32Short => "HUNK_RELOC32SHORT",
            HunkType::RelReloc32 => "HUNK_RELRELOC32",
            HunkType::RelReloc16 => "HUNK_RELRELOC16",
            HunkType::RelReloc8 => "HUNK_RELRELOC8",
            HunkType::DReloc32 => "HUNK_DREL32",
            HunkType::DReloc16 => "HUNK_DREL16",
            HunkType::DReloc8 => "HUNK_DREL8",
            HunkType::Symbol => "HUNK_SYMBOL",
            HunkType::Ext => "HUNK_EXT",
            HunkType::Debug => "HUNK_DEBUG",
            HunkType::End => "HUNK_END",
            HunkType::Overlay => "HUNK_OVERLAY",
            HunkType::Break => "HUNK_BREAK",
            HunkType::Name => "HUNK_NAME",
            HunkType::Unit => "HUNK_UNIT",
            HunkType::Lib => "HUNK_LIB",
            HunkType::Index => "HUNK_INDEX",
            HunkType::AbsReloc16 => "HUNK_ABSRELOC16",
        }
    }
}

impl std::fmt::Display for HunkType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.name())
    }
}

/// A single relocation group: all offsets within the current hunk that
/// need to be patched with the base address of `target_hunk`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Relocation {
    /// The hunk index whose load address gets added to each offset.
    pub target_hunk: u32,
    /// Byte offsets within the current hunk that need patching.
    pub offsets: Vec<u32>,
}

/// A debug symbol extracted from HUNK_SYMBOL.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Symbol {
    pub name: String,
    /// Byte offset within the hunk.
    pub value: u32,
}

/// A single parsed hunk (code, data, or BSS segment) with its
/// associated relocations, symbols, and debug data.
#[derive(Debug, Clone)]
pub struct Hunk {
    /// Index of this hunk in the executable (0-based).
    pub index: usize,
    /// Whether this is CODE, DATA, or BSS.
    pub hunk_type: HunkType,
    /// Memory allocation preference (chip, fast, any).
    pub memory_type: MemoryType,
    /// Total allocation size in bytes (may be larger than data.len() for BSS).
    pub alloc_size: u32,
    /// Raw bytes of the hunk content. Empty for BSS hunks.
    pub data: Vec<u8>,
    /// Relocation entries attached to this hunk.
    pub relocations: Vec<Relocation>,
    /// Symbols defined in this hunk.
    pub symbols: Vec<Symbol>,
    /// Optional hunk name (from HUNK_NAME).
    pub name: Option<String>,
    /// Raw debug data, if present.
    pub debug_data: Option<Vec<u8>>,
}

/// A fully parsed Amiga hunk executable.
#[derive(Debug, Clone)]
pub struct HunkFile {
    /// The content hunks (CODE, DATA, BSS) in load order.
    pub hunks: Vec<Hunk>,
    /// First hunk index from the header (usually 0).
    pub first_hunk: u32,
    /// Last hunk index from the header.
    pub last_hunk: u32,
}

impl std::fmt::Display for MemoryType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            MemoryType::Any => f.write_str("ANY"),
            MemoryType::Chip => f.write_str("CHIP"),
            MemoryType::Fast => f.write_str("FAST"),
            MemoryType::Extended(v) => write!(f, "EXT(0x{v:08X})"),
        }
    }
}
