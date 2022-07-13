use crate::Weights;
use once_cell::sync::Lazy;
use std::collections::{HashMap, HashSet};
use tinyvec::ArrayVec;

//
// Const
//

pub const ASCII_AN: [u32; 62] = [
    48, 49, 50, 51, 52, 53, 54, 55, 56, 57, // Digits
    65, 66, 67, 68, 69, 70, 71, 72, 73, 74, 75, 76, 77, 78, 79, 80, 81, 82, 83, 84, 85, 86, 87, 88,
    89, 90, // A-Z
    97, 98, 99, 100, 101, 102, 103, 104, 105, 106, 107, 108, 109, 110, 111, 112, 113, 114, 115,
    116, 117, 118, 119, 120, 121, 122, // a-z
];

pub const NEED_THREE: [u32; 4] = [3_270, 3_545, 4_018, 4_019];

pub const NEED_TWO: [u32; 59] = [
    76, 108, 1_048, 1_080, 1_575, 1_608, 1_610, 2_503, 2_887, 2_962, 3_014, 3_015, 3_142, 3_263,
    3_274, 3_398, 3_399, 3_548, 3_648, 3_649, 3_650, 3_651, 3_652, 3_661, 3_776, 3_777, 3_778,
    3_779, 3_780, 3_789, 3_953, 4_133, 6_581, 6_582, 6_583, 6_586, 6_917, 6_919, 6_921, 6_923,
    6_925, 6_929, 6_970, 6_972, 6_974, 6_975, 6_978, 43_701, 43_702, 43_705, 43_707, 43_708,
    69_937, 69_938, 70_471, 70_841, 71_096, 71_097, 71_989,
];

pub const INCLUDED_UNASSIGNED: [u32; 4] = [177_977, 178_206, 183_970, 191_457];

//
// Static
//

pub static DECOMP: Lazy<HashMap<u32, Vec<u32>>> = Lazy::new(|| {
    let data = include_bytes!("bincode/decomp");
    let decoded: HashMap<u32, Vec<u32>> = bincode::deserialize(data).unwrap();
    decoded
});

pub static JAMO: Lazy<HashSet<u32>> = Lazy::new(|| {
    let data = include_bytes!("bincode/jamo");
    let decoded: HashSet<u32> = bincode::deserialize(data).unwrap();
    decoded
});

pub static FCD: Lazy<HashMap<u32, u16>> = Lazy::new(|| {
    let data = include_bytes!("bincode/fcd");
    let decoded: HashMap<u32, u16> = bincode::deserialize(data).unwrap();
    decoded
});

pub(crate) static SING: Lazy<HashMap<u32, Vec<Weights>>> = Lazy::new(|| {
    let data = include_bytes!("bincode/singles");
    let decoded: HashMap<u32, Vec<Weights>> = bincode::deserialize(data).unwrap();
    decoded
});

pub(crate) static MULT: Lazy<HashMap<ArrayVec<[u32; 3]>, Vec<Weights>>> = Lazy::new(|| {
    let data = include_bytes!("bincode/multis");
    let decoded: HashMap<ArrayVec<[u32; 3]>, Vec<Weights>> = bincode::deserialize(data).unwrap();
    decoded
});

pub(crate) static SING_CLDR: Lazy<HashMap<u32, Vec<Weights>>> = Lazy::new(|| {
    let data = include_bytes!("bincode/singles_cldr");
    let decoded: HashMap<u32, Vec<Weights>> = bincode::deserialize(data).unwrap();
    decoded
});

pub(crate) static MULT_CLDR: Lazy<HashMap<ArrayVec<[u32; 3]>, Vec<Weights>>> = Lazy::new(|| {
    let data = include_bytes!("bincode/multis_cldr");
    let decoded: HashMap<ArrayVec<[u32; 3]>, Vec<Weights>> = bincode::deserialize(data).unwrap();
    decoded
});
