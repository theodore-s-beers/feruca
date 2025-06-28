use std::sync::LazyLock;

use crate::consts::{MULT_CLDR_DATA, SING_CLDR_DATA, deserialize_bincode};
use crate::types::{MultisTable, SinglesTable};

const SING_AR_DATA: &[u8; 20_580] = include_bytes!("bincode/tailoring/arabic_script_sing");
pub static SING_AR: LazyLock<SinglesTable> = LazyLock::new(|| {
    let mut sing: SinglesTable = deserialize_bincode(SING_CLDR_DATA);
    let extension: SinglesTable = deserialize_bincode(SING_AR_DATA);

    sing.extend(extension);
    sing
});

const MULT_AR_DATA: &[u8; 148] = include_bytes!("bincode/tailoring/arabic_script_multi");
pub static MULT_AR: LazyLock<MultisTable> = LazyLock::new(|| {
    let mut mult: MultisTable = deserialize_bincode(MULT_CLDR_DATA);
    let extension: MultisTable = deserialize_bincode(MULT_AR_DATA);

    mult.extend(extension);
    mult
});

const SING_AR_I_DATA: &[u8; 14_652] = include_bytes!("bincode/tailoring/arabic_interleaved_sing");
pub static SING_AR_I: LazyLock<SinglesTable> = LazyLock::new(|| {
    let mut sing: SinglesTable = deserialize_bincode(SING_CLDR_DATA);
    let extension: SinglesTable = deserialize_bincode(SING_AR_I_DATA);

    sing.extend(extension);
    sing
});

const MULT_AR_I_DATA: &[u8; 92] = include_bytes!("bincode/tailoring/arabic_interleaved_multi");
pub static MULT_AR_I: LazyLock<MultisTable> = LazyLock::new(|| {
    let mut mult: MultisTable = deserialize_bincode(MULT_CLDR_DATA);
    let extension: MultisTable = deserialize_bincode(MULT_AR_I_DATA);

    mult.extend(extension);
    mult
});
