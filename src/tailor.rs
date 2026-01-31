use crate::consts::{MULT_CLDR_DATA, SING_CLDR_DATA};
use crate::types::{MultisTable, SinglesTable};
use std::sync::LazyLock;

const SING_AR_DATA: &[u8] = include_bytes!("bincode/tailoring/arabic_script_sing");
pub static SING_AR: LazyLock<SinglesTable> = LazyLock::new(|| {
    let mut sing: SinglesTable = postcard::from_bytes(SING_CLDR_DATA).unwrap();
    let extension: SinglesTable = postcard::from_bytes(SING_AR_DATA).unwrap();

    sing.extend(extension);
    sing
});

const MULT_AR_DATA: &[u8] = include_bytes!("bincode/tailoring/arabic_script_multi");
pub static MULT_AR: LazyLock<MultisTable> = LazyLock::new(|| {
    let mut mult: MultisTable = postcard::from_bytes(MULT_CLDR_DATA).unwrap();
    let extension: MultisTable = postcard::from_bytes(MULT_AR_DATA).unwrap();

    mult.extend(extension);
    mult
});

const SING_AR_I_DATA: &[u8] = include_bytes!("bincode/tailoring/arabic_interleaved_sing");
pub static SING_AR_I: LazyLock<SinglesTable> = LazyLock::new(|| {
    let mut sing: SinglesTable = postcard::from_bytes(SING_CLDR_DATA).unwrap();
    let extension: SinglesTable = postcard::from_bytes(SING_AR_I_DATA).unwrap();

    sing.extend(extension);
    sing
});

const MULT_AR_I_DATA: &[u8] = include_bytes!("bincode/tailoring/arabic_interleaved_multi");
pub static MULT_AR_I: LazyLock<MultisTable> = LazyLock::new(|| {
    let mut mult: MultisTable = postcard::from_bytes(MULT_CLDR_DATA).unwrap();
    let extension: MultisTable = postcard::from_bytes(MULT_AR_I_DATA).unwrap();

    mult.extend(extension);
    mult
});
