/// CPU variant selection for instruction decoding.
///
/// Each successive variant is a superset of the previous one. When the
/// decoder encounters an instruction that requires a higher variant than
/// configured, it emits a `dc.w` data constant instead.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum CpuVariant {
    /// MC68000 — the original 16/32-bit processor.
    M68000,
    /// MC68010 — adds virtual memory support, MOVE from SR made privileged.
    M68010,
    /// MC68020 — full 32-bit, bit fields, 32-bit mul/div, enhanced addressing.
    M68020,
    /// MC68030 — on-chip MMU and data cache.
    M68030,
    /// MC68040 — integrated FPU, dual pipelines.
    M68040,
    /// MC68060 — superscalar, some instructions software-emulated.
    M68060,
}

impl std::fmt::Display for CpuVariant {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CpuVariant::M68000 => f.write_str("68000"),
            CpuVariant::M68010 => f.write_str("68010"),
            CpuVariant::M68020 => f.write_str("68020"),
            CpuVariant::M68030 => f.write_str("68030"),
            CpuVariant::M68040 => f.write_str("68040"),
            CpuVariant::M68060 => f.write_str("68060"),
        }
    }
}

impl CpuVariant {
    /// Parse a variant from a string like "68000" or "68020".
    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "68000" | "m68000" | "M68000" => Some(CpuVariant::M68000),
            "68010" | "m68010" | "M68010" => Some(CpuVariant::M68010),
            "68020" | "m68020" | "M68020" => Some(CpuVariant::M68020),
            "68030" | "m68030" | "M68030" => Some(CpuVariant::M68030),
            "68040" | "m68040" | "M68040" => Some(CpuVariant::M68040),
            "68060" | "m68060" | "M68060" => Some(CpuVariant::M68060),
            _ => None,
        }
    }
}
