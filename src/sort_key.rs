use crate::cea::get_cea;
use crate::CollationOptions;
use tinyvec::ArrayVec;

pub(crate) fn nfd_to_sk(nfd: &mut Vec<u32>, opt: CollationOptions) -> Vec<u16> {
    let collation_element_array = get_cea(nfd, opt);
    get_sort_key(&collation_element_array, opt.shifting)
}

fn get_sort_key(collation_element_array: &[ArrayVec<[u16; 4]>], shifting: bool) -> Vec<u16> {
    let max_level = if shifting { 4 } else { 3 };
    let mut sort_key = Vec::new();

    for i in 0..max_level {
        if i > 0 {
            sort_key.push(0);
        }

        for elem in collation_element_array {
            if elem[i] != 0 {
                sort_key.push(elem[i]);
            }
        }
    }

    sort_key
}
