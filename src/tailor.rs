use crate::tables::CollationTable;
use std::sync::LazyLock;

const ARABIC_SCRIPT_DATA: &[u8] = include_bytes!("data/tailoring/arabic_script");
pub static ARABIC_SCRIPT: LazyLock<CollationTable> =
    LazyLock::new(|| postcard::from_bytes(ARABIC_SCRIPT_DATA).unwrap());

const ARABIC_INTERLEAVED_DATA: &[u8] = include_bytes!("data/tailoring/arabic_interleaved");
pub static ARABIC_INTERLEAVED: LazyLock<CollationTable> =
    LazyLock::new(|| postcard::from_bytes(ARABIC_INTERLEAVED_DATA).unwrap());
