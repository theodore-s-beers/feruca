use crate::consts::INCLUDED_UNASSIGNED;
use crate::weights::pack_weights;

pub fn implicit_a(cp: u32) -> u32 {
    let aaaa = if INCLUDED_UNASSIGNED.contains(&cp) {
        0xFBC0 + (cp >> 15)
    } else {
        match cp {
            0x3400..=0x4DBF | 0x20000..=0x2A6DF | 0x2A700..=0x2EE5D | 0x30000..=0x323AF => {
                0xFB80 + (cp >> 15)
            } // CJK2
            0x4E00..=0x9FFF | 0xF900..=0xFAFF => 0xFB40 + (cp >> 15), // CJK1
            0x17000..=0x18AFF | 0x18D00..=0x18D8F => 0xFB00,          // Tangut
            0x18B00..=0x18CFF => 0xFB02,                              // Khitan
            0x1B170..=0x1B2FF => 0xFB01,                              // Nushu
            _ => 0xFBC0 + (cp >> 15),                                 // unass.
        }
    };

    #[allow(clippy::cast_possible_truncation)]
    pack_weights(false, aaaa as u16, 32, 2)
}

pub fn implicit_b(cp: u32) -> u32 {
    let mut bbbb = if INCLUDED_UNASSIGNED.contains(&cp) {
        cp & 0x7FFF
    } else {
        match cp {
            0x17000..=0x18AFF | 0x18D00..=0x18D8F => cp - 0x17000, // Tangut
            0x18B00..=0x18CFF => cp - 0x18B00,                     // Khitan
            0x1B170..=0x1B2FF => cp - 0x1B170,                     // Nushu
            _ => cp & 0x7FFF,                                      // CJK1, CJK2, unass.
        }
    };

    // BBBB always gets bitwise ORed with this value
    bbbb |= 0x8000;

    #[allow(clippy::cast_possible_truncation)]
    pack_weights(false, bbbb as u16, 0, 0)
}

pub fn remove_pulled(char_vals: &mut Vec<u32>, i: usize, input_length: &mut usize, try_two: bool) {
    char_vals.remove(i);
    *input_length -= 1;

    if try_two {
        char_vals.remove(i - 1);
        *input_length -= 1;
    }
}
