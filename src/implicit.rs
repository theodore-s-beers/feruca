use crate::weights::pack_weights;

mod generated {
    include!("data/implicit_weights.rs");
}

pub fn implicit_a(cp: u32) -> u32 {
    let (a, _) = generated::implicit_weights(cp);
    pack_weights(false, u16::try_from(a).unwrap(), 32, 2)
}

pub fn implicit_b(cp: u32) -> u32 {
    let (_, b) = generated::implicit_weights(cp);
    pack_weights(false, u16::try_from(b).unwrap(), 0, 0)
}
