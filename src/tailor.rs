use crate::consts::{DATA_MULT_CLDR, DATA_SING_CLDR};
use crate::types::{MultisTable, SinglesTable};
use once_cell::sync::Lazy;

pub static MULT_AR: Lazy<MultisTable> = Lazy::new(|| {
    let mut mult: MultisTable = bincode::deserialize(DATA_MULT_CLDR).unwrap();

    let data = include_bytes!("bincode/tailoring/arabic_script_multi");
    let extension: MultisTable = bincode::deserialize(data).unwrap();

    mult.extend(extension);
    mult
});

pub static SING_AR: Lazy<SinglesTable> = Lazy::new(|| {
    let mut sing: SinglesTable = bincode::deserialize(DATA_SING_CLDR).unwrap();

    let data = include_bytes!("bincode/tailoring/arabic_script_sing");
    let extension: SinglesTable = bincode::deserialize(data).unwrap();

    sing.extend(extension);
    sing
});
