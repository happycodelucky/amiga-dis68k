//! Symbol resolution trait and concrete implementations.
//!
//! The resolver system composes multiple symbol sources (hunk symbols,
//! auto-generated labels, LVO tables) into a single lookup interface.

use std::collections::BTreeMap;

use crate::hunk::types::Hunk;
use super::amiga;

/// Trait for resolving addresses and LVO offsets to symbolic names.
pub trait SymbolResolver {
    /// Resolve a library vector offset to a function name.
    fn resolve_lvo(&self, offset: i16) -> Option<String>;

    /// Resolve an address within a hunk to a label name.
    fn resolve_address(&self, address: u32) -> Option<String>;
}

/// Resolves symbols defined in HUNK_SYMBOL data.
pub struct HunkSymbolResolver {
    /// Map from address → symbol name.
    symbols: BTreeMap<u32, String>,
}

impl HunkSymbolResolver {
    pub fn from_hunk(hunk: &Hunk) -> Self {
        let mut symbols = BTreeMap::new();
        for sym in &hunk.symbols {
            symbols.insert(sym.value, sym.name.clone());
        }
        HunkSymbolResolver { symbols }
    }
}

impl SymbolResolver for HunkSymbolResolver {
    fn resolve_lvo(&self, _offset: i16) -> Option<String> {
        None
    }

    fn resolve_address(&self, address: u32) -> Option<String> {
        self.symbols.get(&address).cloned()
    }
}

/// Resolves LVO offsets using the static Amiga OS library tables.
///
/// For now, this resolves against a single library (typically "exec"
/// since A6 is assumed to be ExecBase without data-flow analysis).
pub struct LvoResolver {
    library_name: String,
}

impl LvoResolver {
    pub fn new(library_name: &str) -> Self {
        LvoResolver {
            library_name: library_name.to_string(),
        }
    }
}

impl SymbolResolver for LvoResolver {
    fn resolve_lvo(&self, offset: i16) -> Option<String> {
        amiga::lookup_lvo(&self.library_name, offset)
            .map(|s| s.to_string())
    }

    fn resolve_address(&self, _address: u32) -> Option<String> {
        None
    }
}

/// Resolves branch/jump target addresses to auto-generated labels.
///
/// Built by scanning all branch/jump targets in a first pass, then
/// assigned names like `loc_001A` based on the target address.
pub struct AutoLabelResolver {
    labels: BTreeMap<u32, String>,
}

impl AutoLabelResolver {
    /// Create from a set of target addresses.
    pub fn from_targets(targets: impl IntoIterator<Item = u32>) -> Self {
        let mut labels = BTreeMap::new();
        for addr in targets {
            labels.insert(addr, format!("loc_{:04X}", addr));
        }
        AutoLabelResolver { labels }
    }
}

impl SymbolResolver for AutoLabelResolver {
    fn resolve_lvo(&self, _offset: i16) -> Option<String> {
        None
    }

    fn resolve_address(&self, address: u32) -> Option<String> {
        self.labels.get(&address).cloned()
    }
}

/// Chains multiple resolvers, returning the first match.
pub struct CompositeResolver {
    resolvers: Vec<Box<dyn SymbolResolver>>,
}

impl CompositeResolver {
    pub fn new() -> Self {
        CompositeResolver {
            resolvers: Vec::new(),
        }
    }

    pub fn add(&mut self, resolver: Box<dyn SymbolResolver>) {
        self.resolvers.push(resolver);
    }
}

impl SymbolResolver for CompositeResolver {
    fn resolve_lvo(&self, offset: i16) -> Option<String> {
        for r in &self.resolvers {
            if let Some(name) = r.resolve_lvo(offset) {
                return Some(name);
            }
        }
        None
    }

    fn resolve_address(&self, address: u32) -> Option<String> {
        for r in &self.resolvers {
            if let Some(name) = r.resolve_address(address) {
                return Some(name);
            }
        }
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn auto_label_resolver() {
        let resolver = AutoLabelResolver::from_targets(vec![0x001A, 0x0042]);
        assert_eq!(resolver.resolve_address(0x001A), Some("loc_001A".to_string()));
        assert_eq!(resolver.resolve_address(0x0042), Some("loc_0042".to_string()));
        assert_eq!(resolver.resolve_address(0x0099), None);
    }

    #[test]
    fn lvo_resolver() {
        let resolver = LvoResolver::new("exec");
        assert_eq!(resolver.resolve_lvo(-552), Some("_LVOOpenLibrary".to_string()));
        assert_eq!(resolver.resolve_lvo(-999), None);
        assert_eq!(resolver.resolve_address(0x100), None);
    }

    #[test]
    fn composite_priority() {
        let mut composite = CompositeResolver::new();

        // Auto-labels take priority
        let auto = AutoLabelResolver::from_targets(vec![0x001A]);
        composite.add(Box::new(auto));

        // LVO resolver for exec
        let lvo = LvoResolver::new("exec");
        composite.add(Box::new(lvo));

        assert_eq!(composite.resolve_address(0x001A), Some("loc_001A".to_string()));
        assert_eq!(composite.resolve_lvo(-552), Some("_LVOOpenLibrary".to_string()));
    }

    #[test]
    fn hunk_symbol_resolver() {
        let hunk = Hunk {
            index: 0,
            hunk_type: crate::hunk::types::HunkType::Code,
            memory_type: crate::hunk::types::MemoryType::Any,
            alloc_size: 100,
            data: vec![],
            relocations: vec![],
            symbols: vec![
                crate::hunk::types::Symbol { name: "_main".to_string(), value: 0x0000 },
                crate::hunk::types::Symbol { name: "_exit".to_string(), value: 0x0020 },
            ],
            name: None,
            debug_data: None,
        };

        let resolver = HunkSymbolResolver::from_hunk(&hunk);
        assert_eq!(resolver.resolve_address(0x0000), Some("_main".to_string()));
        assert_eq!(resolver.resolve_address(0x0020), Some("_exit".to_string()));
        assert_eq!(resolver.resolve_address(0x0010), None);
    }
}
