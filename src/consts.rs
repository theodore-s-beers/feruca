use crate::tables::{CollationTable, DecompTable, FcdTable, VariableTable};
use std::sync::LazyLock;

// Map a code point to its canonical decomposition (if any)
const DECOMP_DATA: &[u8] = include_bytes!("data/decomp");
pub static DECOMP: LazyLock<DecompTable> =
    LazyLock::new(|| postcard::from_bytes(DECOMP_DATA).unwrap());

// Map a code point to the first and last CCCs (two u8s packed into a u16) of its canonical
// decomposition (if any)
const FCD_DATA: &[u8] = include_bytes!("data/fcd");
pub static FCD: LazyLock<FcdTable> = LazyLock::new(|| postcard::from_bytes(FCD_DATA).unwrap());

// Map a low code point to its collation weights (DUCET)
// Code points are used to index into this array
include!("data/low_ducet.rs");

// Map non-low code points to their single-code-point weights and contraction metadata (DUCET)
const DUCET_DATA: &[u8] = include_bytes!("data/ducet");
pub static DUCET: LazyLock<CollationTable> =
    LazyLock::new(|| postcard::from_bytes(DUCET_DATA).unwrap());

// Map a low code point to its collation weights (CLDR)
// Code points are used to index into this array
include!("data/low_cldr.rs");

// Map non-low code points to their single-code-point weights and contraction metadata (CLDR)
const CLDR_ROOT_DATA: &[u8] = include_bytes!("data/cldr_root");
pub static CLDR_ROOT: LazyLock<CollationTable> =
    LazyLock::new(|| postcard::from_bytes(CLDR_ROOT_DATA).unwrap());

// CLDR root collation with Arabic-script characters sorted before Latin-script characters
const ARABIC_SCRIPT_DATA: &[u8] = include_bytes!("data/tailoring/arabic_script");
pub static ARABIC_SCRIPT: LazyLock<CollationTable> =
    LazyLock::new(|| postcard::from_bytes(ARABIC_SCRIPT_DATA).unwrap());

// CLDR root collation with Arabic-script characters interleaved among Latin-script characters
const ARABIC_INTERLEAVED_DATA: &[u8] = include_bytes!("data/tailoring/arabic_interleaved");
pub static ARABIC_INTERLEAVED: LazyLock<CollationTable> =
    LazyLock::new(|| postcard::from_bytes(ARABIC_INTERLEAVED_DATA).unwrap());

// Code points that have either a variable weight, or a primary weight of zero
const VARIABLE_DATA: &[u8] = include_bytes!("data/variable");
pub static VARIABLE: LazyLock<VariableTable> =
    LazyLock::new(|| postcard::from_bytes(VARIABLE_DATA).unwrap());
