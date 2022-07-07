use std::cmp::Ordering;
use std::iter::zip;

pub type ByteKey = Vec<u8>;

pub trait ARTKey {
    type Bytes: AsRef<[u8]>;

    fn convert_to_bytes(self) -> Self::Bytes;
}

impl ARTKey for String {
    type Bytes = Vec<u8>;

    fn convert_to_bytes(self) -> Self::Bytes {
        self.into_bytes()
    }
}

macro_rules! ArtKeyNumImpl {
    ($sty: ty) => {
        impl ARTKey for $sty {
            type Bytes = [u8; std::mem::size_of::<$sty>()];
            fn convert_to_bytes(self) -> Self::Bytes {
                self.to_be_bytes()
            }
        }
    };
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

            match len_1.cmp(&len_2) {
                Ordering::Equal => LeafKeyComp::FullMatch,
                Ordering::Less => LeafKeyComp::CompleteMatchLeft(len_1),
                Ordering::Greater => LeafKeyComp::CompleteMatchRight(len_2),
            }
        }
        Some(pos) => LeafKeyComp::PartialMatch(pos),
    }
}

pub fn compare_pkeys(pkey_1: &[u8], pkey_2: &[u8]) -> PartialKeyComp {
    match zip(pkey_1, pkey_2).position(|(a, b)| a != b) {
        None => PartialKeyComp::FullMatch(std::cmp::min(pkey_1.len(), pkey_2.len())),
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
