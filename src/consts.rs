use crate::types::Weights;
use once_cell::sync::Lazy;
use rustc_hash::{FxHashMap, FxHashSet};
use std::collections::HashSet;
use tinyvec::ArrayVec;

//
// Const
//

// Unassigned code points that are erroneously included in one of the ranges of code points used to
// calculate implicit weights
pub const INCLUDED_UNASSIGNED: [u32; 4] = [177_977, 178_206, 183_970, 191_457];

// Code points that can start three-code-point sequences in the collation tables. These values don't
// "need" to be u32, but that's what they'll be compared against
pub const NEED_THREE: [u32; 4] = [3_270, 3_545, 4_018, 4_019];

// Code points that can start two-code-point sequences in the collation tables. This used to include
// duplicate values from NEED_THREE, but that's unnecessary.
pub const NEED_TWO: [u32; 59] = [
    76, 108, 1_048, 1_080, 1_575, 1_608, 1_610, 2_503, 2_887, 2_962, 3_014, 3_015, 3_142, 3_263,
    3_274, 3_398, 3_399, 3_548, 3_648, 3_649, 3_650, 3_651, 3_652, 3_661, 3_776, 3_777, 3_778,
    3_779, 3_780, 3_789, 3_953, 4_133, 6_581, 6_582, 6_583, 6_586, 6_917, 6_919, 6_921, 6_923,
    6_925, 6_929, 6_970, 6_972, 6_974, 6_975, 6_978, 43_701, 43_702, 43_705, 43_707, 43_708,
    69_937, 69_938, 70_471, 70_841, 71_096, 71_097, 71_989,
];

//
// Static
//

// I think a hash set may perform better than an array, given the size (~400). But it could always
// be changed.
//
// I did go for u16 for this -- same with the jamo-related consts in the `normalize` module. That
// means casting the code point to u16. I wonder if it would be better to keep everything in u32.
pub static JAMO_LV: Lazy<HashSet<u16>> = Lazy::new(|| {
    HashSet::from([
        44_032, 44_060, 44_088, 44_116, 44_144, 44_172, 44_200, 44_228, 44_256, 44_284, 44_312,
        44_340, 44_368, 44_396, 44_424, 44_452, 44_480, 44_508, 44_536, 44_564, 44_592, 44_620,
        44_648, 44_676, 44_704, 44_732, 44_760, 44_788, 44_816, 44_844, 44_872, 44_900, 44_928,
        44_956, 44_984, 45_012, 45_040, 45_068, 45_096, 45_124, 45_152, 45_180, 45_208, 45_236,
        45_264, 45_292, 45_320, 45_348, 45_376, 45_404, 45_432, 45_460, 45_488, 45_516, 45_544,
        45_572, 45_600, 45_628, 45_656, 45_684, 45_712, 45_740, 45_768, 45_796, 45_824, 45_852,
        45_880, 45_908, 45_936, 45_964, 45_992, 46_020, 46_048, 46_076, 46_104, 46_132, 46_160,
        46_188, 46_216, 46_244, 46_272, 46_300, 46_328, 46_356, 46_384, 46_412, 46_440, 46_468,
        46_496, 46_524, 46_552, 46_580, 46_608, 46_636, 46_664, 46_692, 46_720, 46_748, 46_776,
        46_804, 46_832, 46_860, 46_888, 46_916, 46_944, 46_972, 47_000, 47_028, 47_056, 47_084,
        47_112, 47_140, 47_168, 47_196, 47_224, 47_252, 47_280, 47_308, 47_336, 47_364, 47_392,
        47_420, 47_448, 47_476, 47_504, 47_532, 47_560, 47_588, 47_616, 47_644, 47_672, 47_700,
        47_728, 47_756, 47_784, 47_812, 47_840, 47_868, 47_896, 47_924, 47_952, 47_980, 48_008,
        48_036, 48_064, 48_092, 48_120, 48_148, 48_176, 48_204, 48_232, 48_260, 48_288, 48_316,
        48_344, 48_372, 48_400, 48_428, 48_456, 48_484, 48_512, 48_540, 48_568, 48_596, 48_624,
        48_652, 48_680, 48_708, 48_736, 48_764, 48_792, 48_820, 48_848, 48_876, 48_904, 48_932,
        48_960, 48_988, 49_016, 49_044, 49_072, 49_100, 49_128, 49_156, 49_184, 49_212, 49_240,
        49_268, 49_296, 49_324, 49_352, 49_380, 49_408, 49_436, 49_464, 49_492, 49_520, 49_548,
        49_576, 49_604, 49_632, 49_660, 49_688, 49_716, 49_744, 49_772, 49_800, 49_828, 49_856,
        49_884, 49_912, 49_940, 49_968, 49_996, 50_024, 50_052, 50_080, 50_108, 50_136, 50_164,
        50_192, 50_220, 50_248, 50_276, 50_304, 50_332, 50_360, 50_388, 50_416, 50_444, 50_472,
        50_500, 50_528, 50_556, 50_584, 50_612, 50_640, 50_668, 50_696, 50_724, 50_752, 50_780,
        50_808, 50_836, 50_864, 50_892, 50_920, 50_948, 50_976, 51_004, 51_032, 51_060, 51_088,
        51_116, 51_144, 51_172, 51_200, 51_228, 51_256, 51_284, 51_312, 51_340, 51_368, 51_396,
        51_424, 51_452, 51_480, 51_508, 51_536, 51_564, 51_592, 51_620, 51_648, 51_676, 51_704,
        51_732, 51_760, 51_788, 51_816, 51_844, 51_872, 51_900, 51_928, 51_956, 51_984, 52_012,
        52_040, 52_068, 52_096, 52_124, 52_152, 52_180, 52_208, 52_236, 52_264, 52_292, 52_320,
        52_348, 52_376, 52_404, 52_432, 52_460, 52_488, 52_516, 52_544, 52_572, 52_600, 52_628,
        52_656, 52_684, 52_712, 52_740, 52_768, 52_796, 52_824, 52_852, 52_880, 52_908, 52_936,
        52_964, 52_992, 53_020, 53_048, 53_076, 53_104, 53_132, 53_160, 53_188, 53_216, 53_244,
        53_272, 53_300, 53_328, 53_356, 53_384, 53_412, 53_440, 53_468, 53_496, 53_524, 53_552,
        53_580, 53_608, 53_636, 53_664, 53_692, 53_720, 53_748, 53_776, 53_804, 53_832, 53_860,
        53_888, 53_916, 53_944, 53_972, 54_000, 54_028, 54_056, 54_084, 54_112, 54_140, 54_168,
        54_196, 54_224, 54_252, 54_280, 54_308, 54_336, 54_364, 54_392, 54_420, 54_448, 54_476,
        54_504, 54_532, 54_560, 54_588, 54_616, 54_644, 54_672, 54_700, 54_728, 54_756, 54_784,
        54_812, 54_840, 54_868, 54_896, 54_924, 54_952, 54_980, 55_008, 55_036, 55_064, 55_092,
        55_120, 55_148, 55_176,
    ])
});

// Map a code point to its canonical decomposition (if any)
pub static DECOMP: Lazy<FxHashMap<u32, Vec<u32>>> = Lazy::new(|| {
    let data = include_bytes!("bincode/decomp");
    let decoded: FxHashMap<u32, Vec<u32>> = bincode::deserialize(data).unwrap();
    decoded
});

// Map a code point to the first and last CCCs (two u8s packed into a u16) of its canonical
// decomposition (if any)
pub static FCD: Lazy<FxHashMap<u32, u16>> = Lazy::new(|| {
    let data = include_bytes!("bincode/fcd");
    let decoded: FxHashMap<u32, u16> = bincode::deserialize(data).unwrap();
    decoded
});

// Map a low code point to its collation weights (DUCET)
pub static LOW: Lazy<FxHashMap<u32, Weights>> = Lazy::new(|| {
    let data = include_bytes!("bincode/low");
    let decoded: FxHashMap<u32, Weights> = bincode::deserialize(data).unwrap();
    decoded
});

// Map a single code point to its collation weights (DUCET)
pub static SING: Lazy<FxHashMap<u32, Vec<Weights>>> = Lazy::new(|| {
    let data = include_bytes!("bincode/singles");
    let decoded: FxHashMap<u32, Vec<Weights>> = bincode::deserialize(data).unwrap();
    decoded
});

// Map a sequence of code points to its collation weights (DUCET)
pub static MULT: Lazy<FxHashMap<ArrayVec<[u32; 3]>, Vec<Weights>>> = Lazy::new(|| {
    let data = include_bytes!("bincode/multis");
    let decoded: FxHashMap<ArrayVec<[u32; 3]>, Vec<Weights>> = bincode::deserialize(data).unwrap();
    decoded
});

// Map a low code point to its collation weights (CLDR)
pub static LOW_CLDR: Lazy<FxHashMap<u32, Weights>> = Lazy::new(|| {
    let data = include_bytes!("bincode/low_cldr");
    let decoded: FxHashMap<u32, Weights> = bincode::deserialize(data).unwrap();
    decoded
});

// Map a single code point to its collation weights (CLDR)
pub static SING_CLDR: Lazy<FxHashMap<u32, Vec<Weights>>> = Lazy::new(|| {
    let data = include_bytes!("bincode/singles_cldr");
    let decoded: FxHashMap<u32, Vec<Weights>> = bincode::deserialize(data).unwrap();
    decoded
});

// Map a sequence of code points to its collation weights (CLDR)
pub static MULT_CLDR: Lazy<FxHashMap<ArrayVec<[u32; 3]>, Vec<Weights>>> = Lazy::new(|| {
    let data = include_bytes!("bincode/multis_cldr");
    let decoded: FxHashMap<ArrayVec<[u32; 3]>, Vec<Weights>> = bincode::deserialize(data).unwrap();
    decoded
});

// A hash set of code points that have either a variable weight, or a primary weight of zero
pub static VARIABLE: Lazy<FxHashSet<u32>> = Lazy::new(|| {
    let data = include_bytes!("bincode/variable");
    let decoded: FxHashSet<u32> = bincode::deserialize(data).unwrap();
    decoded
});
