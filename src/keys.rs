use crate::ARTKey;
use std::rc::Rc;
use std::iter::zip;

impl ARTKey for String {
    fn convert_to_bytes(self) -> Vec<u8> {
        self.into_bytes()
    }
}

macro_rules! ArtKeyNumImpl {
	($sty: ty) => {
        impl ARTKey for $sty {
            fn convert_to_bytes(self) -> Vec<u8> {
                self.to_ne_bytes().to_vec()
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

// pub fn compare_pkeys(pkey_1: &[u8], pkey_2: &[u8]) -> PartialKeyComp {
//     match zip(pkey_1, pkey_2).position(|(a, b)| a != b) {
//         None => PartialKeyComp::FullMatch(std::cmp::min(pkey_1.len(), pkey_2.len())),
//         Some(pos) => PartialKeyComp::PartialMatch(pos),
//     }
// }


pub fn compare_pkeys(pkey_1: &[u8], pkey_2: &[u8]) -> PartialKeyComp {
    match zip(pkey_1, pkey_2).position(|(a, b)| a != b) {
        None => {
            PartialKeyComp::FullMatch(std::cmp::min(pkey_1.len(), pkey_2.len()))
        }
        Some(pos) => PartialKeyComp::PartialMatch(pos),
    }
}

pub fn compare_leafkeys(pkey_1: &[u8], pkey_2: &[u8]) -> PartialKeyComp {
    match zip(pkey_1, pkey_2).position(|(a, b)| a != b) {
        Some(pos) => PartialKeyComp::PartialMatch(pos),
        None => {
            let len1 = pkey_1.len();
            let len2 = pkey_2.len();
            if len1 != len2 {
                PartialKeyComp::PartialMatch(std::cmp::min(len1, len2))
            } else {
                PartialKeyComp::FullMatch(len1)
            }
        }
    }
}
