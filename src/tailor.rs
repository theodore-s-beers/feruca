use once_cell::sync::Lazy;

use crate::consts::{MULT_CLDR_DATA, SING_CLDR_DATA};
use crate::types::{MultisTable, SinglesTable};

const SING_AR_DATA: &[u8; 20_504] = include_bytes!("bincode/tailoring/arabic_script_sing");
pub static SING_AR: Lazy<SinglesTable> = Lazy::new(|| {
    let mut sing: SinglesTable = bincode::deserialize(SING_CLDR_DATA).unwrap();
    let extension: SinglesTable = bincode::deserialize(SING_AR_DATA).unwrap();

    sing.extend(extension);
    sing
});

const MULT_AR_DATA: &[u8; 148] = include_bytes!("bincode/tailoring/arabic_script_multi");
pub static MULT_AR: Lazy<MultisTable> = Lazy::new(|| {
    let mut mult: MultisTable = bincode::deserialize(MULT_CLDR_DATA).unwrap();
    let extension: MultisTable = bincode::deserialize(MULT_AR_DATA).unwrap();

    mult.extend(extension);
    mult
});
