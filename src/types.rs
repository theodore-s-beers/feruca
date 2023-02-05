use rustc_hash::FxHashMap;
use serde::Deserialize;

// Aliases for annoying types
pub type SinglesTable = FxHashMap<u32, Vec<Weights>>;
pub type MultisTable = FxHashMap<Vec<u32>, Vec<Weights>>;

/// This enum provides for a choice of which collation tailoring (or table of character weights) to
/// use. With the CLDR table, there is a further choice of locale. (The `Root` locale represents the
/// table in its unmodified form.)
#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Clone, Copy, Hash)]
pub enum Tailoring {
    /// The table associated with the CLDR root collation order, and locale tailorings based thereon
    /// (recommended)
    Cldr(Locale),
    /// The default table for the Unicode Collation Algorithm
    Ducet,
}

impl Default for Tailoring {
    fn default() -> Self {
        Self::Cldr(Locale::default())
    }
}

/// This enum provides for a choice of which locale to use with the CLDR table of character weights.
/// The default, `Root`, represents the CLDR root collation order. At the moment, there is only one
/// other choice: `ArabicScript`. But the list should grow over time.
#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Clone, Copy, Hash)]
pub enum Locale {
    /// This locale defines a tailoring in which the Arabic script sorts before the Latin script. No
    /// more granular adjustments have been made.
    ArabicScript,
    /// The CLDR root collation order
    Root,
}

impl Default for Locale {
    fn default() -> Self {
        Self::Root
    }
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Clone, Copy, Hash, Default, Deserialize)]
pub struct Weights {
    pub variable: bool,
    pub primary: u16,
    pub secondary: u16,
    pub tertiary: u16,
    pub quaternary: u16,
}
