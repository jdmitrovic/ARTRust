use crate::ARTKey;

impl ARTKey for String {
    fn convert_to_bytes(&self) -> Vec<u8> {
        self.clone().into_bytes()
    }
}

macro_rules! ArtKeyNumImpl {
	($sty: ty) => {
        impl ARTKey for $sty {
            fn convert_to_bytes(&self) -> Vec<u8> {
                self.to_be_bytes().to_vec()
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
