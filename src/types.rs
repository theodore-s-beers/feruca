use serde::Deserialize;

/// This enum provides for a choice of which table of character weights to use.
#[derive(Copy, Clone, PartialEq, Eq, Ord, PartialOrd, Hash, Debug)]
pub enum KeysSource {
    /// The table associated with the CLDR root collation order (recommended)
    Cldr,
    /// The default table for the Unicode Collation Algorithm
    Ducet,
}

#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug, Default, Deserialize)]
pub struct Weights {
    pub variable: bool,
    pub primary: u16,
    pub secondary: u16,
    pub tertiary: u16,
}
