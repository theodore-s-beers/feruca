use once_cell::sync::Lazy;
use rustc_hash::{FxHashMap, FxHashSet};
use std::collections::HashSet;

use crate::types::{MultisTable, SinglesTable};

//
// Const
//

// Unassigned code points that are erroneously included in one of the ranges of code points used to
// calculate implicit weights
pub const INCLUDED_UNASSIGNED: [u32; 4] = [0x2B73A, 0x2B81E, 0x2CEA2, 0x2EBE1];

// Code points that can start three-code-point sequences in the collation tables. These values don't
// "need" to be u32, but that's what they'll be compared against.
pub const NEED_THREE: [u32; 4] = [0x0CC6, 0x0DD9, 0x0FB2, 0x0FB3];

// Code points that can start two-code-point sequences in the collation tables. This used to include
// duplicate values from NEED_THREE, but that's unnecessary.
pub const NEED_TWO: [u32; 59] = [
    0x004C, 0x006C, 0x0418, 0x0438, 0x0627, 0x0648, 0x064A, 0x09C7, 0x0B47, 0x0B92, 0x0BC6, 0x0BC7,
    0x0C46, 0x0CBF, 0x0CCA, 0x0D46, 0x0D47, 0x0DDC, 0x0E40, 0x0E41, 0x0E42, 0x0E43, 0x0E44, 0x0E4D,
    0x0EC0, 0x0EC1, 0x0EC2, 0x0EC3, 0x0EC4, 0x0ECD, 0x0F71, 0x1025, 0x19B5, 0x19B6, 0x19B7, 0x19BA,
    0x1B05, 0x1B07, 0x1B09, 0x1B0B, 0x1B0D, 0x1B11, 0x1B3A, 0x1B3C, 0x1B3E, 0x1B3F, 0x1B42, 0xAAB5,
    0xAAB6, 0xAAB9, 0xAABB, 0xAABC, 0x11131, 0x11132, 0x11347, 0x114B9, 0x115B8, 0x115B9, 0x11935,
];

//
// Static
//

// I think a hash set may perform better than an array, given the size (~400). But it could always
// be changed.
pub static JAMO_LV: Lazy<HashSet<u32>> = Lazy::new(|| {
    HashSet::from([
        0xAC00, 0xAC1C, 0xAC38, 0xAC54, 0xAC70, 0xAC8C, 0xACA8, 0xACC4, 0xACE0, 0xACFC, 0xAD18,
        0xAD34, 0xAD50, 0xAD6C, 0xAD88, 0xADA4, 0xADC0, 0xADDC, 0xADF8, 0xAE14, 0xAE30, 0xAE4C,
        0xAE68, 0xAE84, 0xAEA0, 0xAEBC, 0xAED8, 0xAEF4, 0xAF10, 0xAF2C, 0xAF48, 0xAF64, 0xAF80,
        0xAF9C, 0xAFB8, 0xAFD4, 0xAFF0, 0xB00C, 0xB028, 0xB044, 0xB060, 0xB07C, 0xB098, 0xB0B4,
        0xB0D0, 0xB0EC, 0xB108, 0xB124, 0xB140, 0xB15C, 0xB178, 0xB194, 0xB1B0, 0xB1CC, 0xB1E8,
        0xB204, 0xB220, 0xB23C, 0xB258, 0xB274, 0xB290, 0xB2AC, 0xB2C8, 0xB2E4, 0xB300, 0xB31C,
        0xB338, 0xB354, 0xB370, 0xB38C, 0xB3A8, 0xB3C4, 0xB3E0, 0xB3FC, 0xB418, 0xB434, 0xB450,
        0xB46C, 0xB488, 0xB4A4, 0xB4C0, 0xB4DC, 0xB4F8, 0xB514, 0xB530, 0xB54C, 0xB568, 0xB584,
        0xB5A0, 0xB5BC, 0xB5D8, 0xB5F4, 0xB610, 0xB62C, 0xB648, 0xB664, 0xB680, 0xB69C, 0xB6B8,
        0xB6D4, 0xB6F0, 0xB70C, 0xB728, 0xB744, 0xB760, 0xB77C, 0xB798, 0xB7B4, 0xB7D0, 0xB7EC,
        0xB808, 0xB824, 0xB840, 0xB85C, 0xB878, 0xB894, 0xB8B0, 0xB8CC, 0xB8E8, 0xB904, 0xB920,
        0xB93C, 0xB958, 0xB974, 0xB990, 0xB9AC, 0xB9C8, 0xB9E4, 0xBA00, 0xBA1C, 0xBA38, 0xBA54,
        0xBA70, 0xBA8C, 0xBAA8, 0xBAC4, 0xBAE0, 0xBAFC, 0xBB18, 0xBB34, 0xBB50, 0xBB6C, 0xBB88,
        0xBBA4, 0xBBC0, 0xBBDC, 0xBBF8, 0xBC14, 0xBC30, 0xBC4C, 0xBC68, 0xBC84, 0xBCA0, 0xBCBC,
        0xBCD8, 0xBCF4, 0xBD10, 0xBD2C, 0xBD48, 0xBD64, 0xBD80, 0xBD9C, 0xBDB8, 0xBDD4, 0xBDF0,
        0xBE0C, 0xBE28, 0xBE44, 0xBE60, 0xBE7C, 0xBE98, 0xBEB4, 0xBED0, 0xBEEC, 0xBF08, 0xBF24,
        0xBF40, 0xBF5C, 0xBF78, 0xBF94, 0xBFB0, 0xBFCC, 0xBFE8, 0xC004, 0xC020, 0xC03C, 0xC058,
        0xC074, 0xC090, 0xC0AC, 0xC0C8, 0xC0E4, 0xC100, 0xC11C, 0xC138, 0xC154, 0xC170, 0xC18C,
        0xC1A8, 0xC1C4, 0xC1E0, 0xC1FC, 0xC218, 0xC234, 0xC250, 0xC26C, 0xC288, 0xC2A4, 0xC2C0,
        0xC2DC, 0xC2F8, 0xC314, 0xC330, 0xC34C, 0xC368, 0xC384, 0xC3A0, 0xC3BC, 0xC3D8, 0xC3F4,
        0xC410, 0xC42C, 0xC448, 0xC464, 0xC480, 0xC49C, 0xC4B8, 0xC4D4, 0xC4F0, 0xC50C, 0xC528,
        0xC544, 0xC560, 0xC57C, 0xC598, 0xC5B4, 0xC5D0, 0xC5EC, 0xC608, 0xC624, 0xC640, 0xC65C,
        0xC678, 0xC694, 0xC6B0, 0xC6CC, 0xC6E8, 0xC704, 0xC720, 0xC73C, 0xC758, 0xC774, 0xC790,
        0xC7AC, 0xC7C8, 0xC7E4, 0xC800, 0xC81C, 0xC838, 0xC854, 0xC870, 0xC88C, 0xC8A8, 0xC8C4,
        0xC8E0, 0xC8FC, 0xC918, 0xC934, 0xC950, 0xC96C, 0xC988, 0xC9A4, 0xC9C0, 0xC9DC, 0xC9F8,
        0xCA14, 0xCA30, 0xCA4C, 0xCA68, 0xCA84, 0xCAA0, 0xCABC, 0xCAD8, 0xCAF4, 0xCB10, 0xCB2C,
        0xCB48, 0xCB64, 0xCB80, 0xCB9C, 0xCBB8, 0xCBD4, 0xCBF0, 0xCC0C, 0xCC28, 0xCC44, 0xCC60,
        0xCC7C, 0xCC98, 0xCCB4, 0xCCD0, 0xCCEC, 0xCD08, 0xCD24, 0xCD40, 0xCD5C, 0xCD78, 0xCD94,
        0xCDB0, 0xCDCC, 0xCDE8, 0xCE04, 0xCE20, 0xCE3C, 0xCE58, 0xCE74, 0xCE90, 0xCEAC, 0xCEC8,
        0xCEE4, 0xCF00, 0xCF1C, 0xCF38, 0xCF54, 0xCF70, 0xCF8C, 0xCFA8, 0xCFC4, 0xCFE0, 0xCFFC,
        0xD018, 0xD034, 0xD050, 0xD06C, 0xD088, 0xD0A4, 0xD0C0, 0xD0DC, 0xD0F8, 0xD114, 0xD130,
        0xD14C, 0xD168, 0xD184, 0xD1A0, 0xD1BC, 0xD1D8, 0xD1F4, 0xD210, 0xD22C, 0xD248, 0xD264,
        0xD280, 0xD29C, 0xD2B8, 0xD2D4, 0xD2F0, 0xD30C, 0xD328, 0xD344, 0xD360, 0xD37C, 0xD398,
        0xD3B4, 0xD3D0, 0xD3EC, 0xD408, 0xD424, 0xD440, 0xD45C, 0xD478, 0xD494, 0xD4B0, 0xD4CC,
        0xD4E8, 0xD504, 0xD520, 0xD53C, 0xD558, 0xD574, 0xD590, 0xD5AC, 0xD5C8, 0xD5E4, 0xD600,
        0xD61C, 0xD638, 0xD654, 0xD670, 0xD68C, 0xD6A8, 0xD6C4, 0xD6E0, 0xD6FC, 0xD718, 0xD734,
        0xD750, 0xD76C, 0xD788,
    ])
});

// Map a code point to its canonical decomposition (if any)
const DECOMP_DATA: &[u8; 38_364] = include_bytes!("bincode/decomp");
pub static DECOMP: Lazy<SinglesTable> = Lazy::new(|| {
    let decoded: SinglesTable = bincode::deserialize(DECOMP_DATA).unwrap();
    decoded
});

// Map a code point to the first and last CCCs (two u8s packed into a u16) of its canonical
// decomposition (if any)
const FCD_DATA: &[u8; 12_374] = include_bytes!("bincode/fcd");
pub static FCD: Lazy<FxHashMap<u32, u16>> = Lazy::new(|| {
    let decoded: FxHashMap<u32, u16> = bincode::deserialize(FCD_DATA).unwrap();
    decoded
});

// Map a low code point to its collation weights (DUCET)
const LOW_DATA: &[u8; 1_456] = include_bytes!("bincode/low");
pub static LOW: Lazy<FxHashMap<u32, u32>> = Lazy::new(|| {
    let decoded: FxHashMap<u32, u32> = bincode::deserialize(LOW_DATA).unwrap();
    decoded
});

// Map a single code point to its collation weights (DUCET)
const SING_DATA: &[u8; 553_020] = include_bytes!("bincode/singles");
pub static SING: Lazy<SinglesTable> = Lazy::new(|| {
    let decoded: SinglesTable = bincode::deserialize(SING_DATA).unwrap();
    decoded
});

// Map a sequence of code points to its collation weights (DUCET)
const MULT_DATA: &[u8; 29_856] = include_bytes!("bincode/multis");
pub static MULT: Lazy<MultisTable> = Lazy::new(|| {
    let decoded: MultisTable = bincode::deserialize(MULT_DATA).unwrap();
    decoded
});

// Map a low code point to its collation weights (CLDR)
const LOW_CLDR_DATA: &[u8; 1_456] = include_bytes!("bincode/low_cldr");
pub static LOW_CLDR: Lazy<FxHashMap<u32, u32>> = Lazy::new(|| {
    let decoded: FxHashMap<u32, u32> = bincode::deserialize(LOW_CLDR_DATA).unwrap();
    decoded
});

// Map a single code point to its collation weights (CLDR)
pub const SING_CLDR_DATA: &[u8; 552_948] = include_bytes!("bincode/singles_cldr");
pub static SING_CLDR: Lazy<SinglesTable> = Lazy::new(|| {
    let decoded: SinglesTable = bincode::deserialize(SING_CLDR_DATA).unwrap();
    decoded
});

// Map a sequence of code points to its collation weights (CLDR)
pub const MULT_CLDR_DATA: &[u8; 30_192] = include_bytes!("bincode/multis_cldr");
pub static MULT_CLDR: Lazy<MultisTable> = Lazy::new(|| {
    let decoded: MultisTable = bincode::deserialize(MULT_CLDR_DATA).unwrap();
    decoded
});

// A hash set of code points that have either a variable weight, or a primary weight of zero
const VARIABLE_DATA: &[u8; 44_488] = include_bytes!("bincode/variable");
pub static VARIABLE: Lazy<FxHashSet<u32>> = Lazy::new(|| {
    let decoded: FxHashSet<u32> = bincode::deserialize(VARIABLE_DATA).unwrap();
    decoded
});
