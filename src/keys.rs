use std::rc::Rc;
use std::iter::zip;

pub trait ARTKey {
    fn convert_to_bytes(self) -> Vec<u8>;
}

impl ARTKey for String {
    fn convert_to_bytes(self) -> Vec<u8> {
        self.into_bytes()
    }
}

macro_rules! ArtKeyNumImpl {
	($sty: ty) => {
        impl ARTKey for $sty {
            fn convert_to_bytes(self) -> Vec<u8> {
                self.to_be_bytes().to_vec()
            }
        }
	}
}

ArtKeyNumImpl!(u16);
ArtKeyNumImpl!(u32);
ArtKeyNumImpl!(u64);
ArtKeyNumImpl!(i16);
ArtKeyNumImpl!(i32);
ArtKeyNumImpl!(i64);
ArtKeyNumImpl!(usize);
ArtKeyNumImpl!(isize);
ArtKeyNumImpl!(f32);
ArtKeyNumImpl!(f64);

pub type ByteKey = Rc<Vec<u8>>;

pub enum PartialKeyComp {
    PartialMatch(usize),
    FullMatch(usize),
}

pub enum LeafKeyComp {
    PartialMatch(usize),
    FullMatch,
    CompleteMatchLeft(usize),
    CompleteMatchRight(usize),
}

pub fn compare_leaf_keys(key_1: &[u8], key_2: &[u8]) -> LeafKeyComp {
    match zip(key_1, key_2).position(|(a, b)| a != b) {
        None => {
            let len_1 = key_1.len();
            let len_2 = key_2.len();

            // if len_1 == len_2 {
            //     return leafkeycomp::fullmatch;
            // } else if len_1 < len_2 {
            //     return leafkeycomp::completematchleft(len_1);
            // } else {
            //     return leafkeycomp::completematchright(len_2);
            // }
            use std::cmp::Ordering;

            match len_1.cmp(&len_2) {
                Ordering::Equal => {
                    LeafKeyComp::FullMatch
                }
                Ordering::Less => {
                    LeafKeyComp::CompleteMatchLeft(len_1)
                }
                Ordering::Greater => {
                    LeafKeyComp::CompleteMatchRight(len_2)
                }
            }
        }
        Some(pos) => LeafKeyComp::PartialMatch(pos),
    }
}


pub fn compare_pkeys(pkey_1: &[u8], pkey_2: &[u8]) -> PartialKeyComp {
    match zip(pkey_1, pkey_2).position(|(a, b)| a != b) {
        None => {
            PartialKeyComp::FullMatch(std::cmp::min(pkey_1.len(), pkey_2.len()))
        }
        Some(pos) => PartialKeyComp::PartialMatch(pos),
    }
}

// pub fn compare_leaf_keys(pkey_1: &[u8], pkey_2: &[u8]) -> PartialKeyComp {
//     let len1 = pkey_1.len();
//     let len2 = pkey_2.len();
//     if len1 != len2 {
//         return PartialKeyComp::PartialMatch(std::cmp::min(len1, len2));
//     }

//     match zip(pkey_1, pkey_2).position(|(a, b)| a != b) {
//         Some(pos) => PartialKeyComp::PartialMatch(pos),
//         None => PartialKeyComp::FullMatch(len1),
//     }
// }
