use rustc_hash::FxHashMap;

// Aliases for annoying types
pub type SinglesTable = FxHashMap<u32, Box<[u32]>>;
pub type MultisTable = FxHashMap<Box<[u32]>, Box<[u32]>>;

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
/// The default, `Root`, represents the CLDR root collation order. At the moment, there are only two
/// other choices: `ArabicScript` and `ArabicInterleaved`. But the list should grow over time.
#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Clone, Copy, Hash, Default)]
pub enum Locale {
    /// This locale defines a tailoring in which the Arabic script sorts before the Latin script. No
    /// more granular adjustments have been made.
    ArabicScript,
    /// This locale defines a tailoring in which Arabic-script characters are interleaved with
    /// Latin-script characters, so that _alif_ sorts between A and B, _bā’_ between B and C, etc.
    ArabicInterleaved,
    /// The CLDR root collation order
    #[default]
    Root,
}
