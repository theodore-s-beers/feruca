use crate::tables::{CollationTable, DecompTable, FcdTable, VariableTable};
use std::sync::LazyLock;

//
// Const
//

// Unassigned code points that are erroneously included in one of the ranges of code points used to
// calculate implicit weights
pub const INCLUDED_UNASSIGNED: [u32; 4] = [0x2B73A, 0x2B81E, 0x2CEA2, 0x2EBE1];

//
// Static
//

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
#[allow(clippy::unreadable_literal)]
pub const LOW_DUCET: [u32; 183] = [
    0, 0, 0, 0, 0, 0, 0, 0, 0, 33653792, 33719328, 33784864, 33850400, 33915936, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 34178080, 40469536, 54166560, 63538208, 558433312,
    63603744, 63341600, 53969952, 54363168, 54428704, 62817312, 112493600, 36013088, 34440224,
    42107936, 63144992, 561841184, 561906720, 561972256, 562037792, 562103328, 562168864,
    562234400, 562299936, 562365472, 562431008, 37913632, 37520416, 112821280, 112886816,
    112952352, 40928288, 62751776, 595595296, 597299232, 599003168, 600444960, 602345504,
    606212128, 607195168, 609751072, 611520544, 613355552, 614993952, 0, 620105760, 621088800,
    623644704, 626790432, 628166688, 629411872, 633737248, 636555296, 638849056, 641994784,
    643174432, 643829792, 644616224, 646058016, 54494240, 63210528, 54559776, 82019360, 34309152,
    81822752, 595592224, 597296160, 599000096, 600441888, 602342432, 606209056, 607192096,
    609748000, 611517472, 613352480, 614990880, 0, 620102688, 621085728, 623641632, 626787360,
    628163616, 629408800, 633734176, 636552224, 638845984, 641991712, 643171360, 643826720,
    644613152, 646054944, 54625312, 113083424, 54690848, 113214496, 0, 0, 0, 0, 0, 0, 33981472, 0,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 34190880, 40535072,
    558367776, 558498848, 558302240, 558564384, 113148960, 62358560, 82281504, 102794272,
    595601440, 54232096, 113017888, 0, 102925344, 82084896, 90407968, 112624672, 561981472,
    562047008, 81888288, 657590304, 62489632,
];

// Map non-low code points to their single-code-point weights and contraction metadata (DUCET)
const DUCET_DATA: &[u8] = include_bytes!("data/ducet");
pub static DUCET: LazyLock<CollationTable> =
    LazyLock::new(|| postcard::from_bytes(DUCET_DATA).unwrap());

// Map a low code point to its collation weights (CLDR)
// Code points are used to index into this array
#[allow(clippy::unreadable_literal)]
pub const LOW_CLDR: [u32; 183] = [
    0, 0, 0, 0, 0, 0, 0, 0, 0, 33653792, 33719328, 33784864, 33850400, 33915936, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 34178080, 40469536, 54166560, 63538208, 558433312,
    63603744, 63341600, 53969952, 54363168, 54428704, 62817312, 112460832, 36013088, 34440224,
    42107936, 63144992, 561841184, 561906720, 561972256, 562037792, 562103328, 562168864,
    562234400, 562299936, 562365472, 562431008, 37913632, 37520416, 112788512, 112854048,
    112919584, 40928288, 62751776, 662704160, 664473632, 666177568, 667619360, 669519904,
    673386528, 674369568, 676859936, 678629408, 680464416, 682102816, 0, 687214624, 688197664,
    690753568, 693899296, 695275552, 696520736, 700846112, 703664160, 705957920, 709103648,
    710283296, 710938656, 711725088, 713166880, 54494240, 63210528, 54559776, 81986592, 34309152,
    81789984, 662701088, 664470560, 666174496, 667616288, 669516832, 673383456, 674366496,
    676856864, 678626336, 680461344, 682099744, 0, 687211552, 688194592, 690750496, 693896224,
    695272480, 696517664, 700843040, 703661088, 705954848, 709100576, 710280224, 710935584,
    711722016, 713163808, 54625312, 113050656, 54690848, 113181728, 0, 0, 0, 0, 0, 0, 33981472, 0,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 34190880, 40535072,
    558367776, 558498848, 558302240, 558564384, 113116192, 62358560, 82248736, 102761504,
    662710304, 54232096, 112985120, 0, 102892576, 82052128, 90375200, 112591904, 561981472,
    562047008, 81855520, 724699168, 62489632,
];

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
