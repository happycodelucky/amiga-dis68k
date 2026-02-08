//! Symbol resolution for Amiga 68k disassembly.
//!
//! Provides LVO (Library Vector Offset) tables for Amiga OS libraries,
//! auto-generated labels for branch/jump targets, and a composable
//! resolver system for mapping addresses to symbolic names.

pub mod amiga;
pub mod labels;
pub mod resolver;

pub use resolver::{
    AutoLabelResolver, CompositeResolver, HunkSymbolResolver, LvoResolver, SymbolResolver,
};
pub use labels::collect_branch_targets;
