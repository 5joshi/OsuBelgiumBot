pub trait MultMapKey: Copy {
    fn index<const N: usize>(self) -> usize;
}

macro_rules! impl_separator {
    ($($ty:ty),*) => {
        $(
            impl MultMapKey for $ty {
                fn index<const N: usize>(self) -> usize {
                    self as usize % N
                }
            }
        )*
    };
}

impl_separator!(u8, u16, u32, u64, u128, usize, i8, i16, i32, i64, isize);
