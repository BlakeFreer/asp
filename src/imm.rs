#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Imm<T, const N: u8>
where
    T: ImmType<N>,
{
    value: T,
}

pub trait ImmType<const N: u8>:
    PartialOrd + PartialEq + TryInto<Imm<Self, N>> + Into<i32> + TryFrom<i32>
{
    const MIN: Self;
    const MAX: Self;
}
impl<const N: u8> ImmType<N> for u8 {
    const MIN: u8 = 0;
    const MAX: u8 = (1 << N) - 1;
}
impl<const N: u8> ImmType<N> for i8 {
    const MIN: i8 = -(1 << (N - 1));
    const MAX: i8 = (1 << (N - 1)) - 1;
}

impl<T: ImmType<N>, const N: u8> Imm<T, N> {
    pub fn get(self) -> T {
        self.value
    }
}

impl<T: ImmType<N>, const N: u8> Imm<T, N> {
    pub fn new(value: T) -> Option<Self> {
        if value >= T::MIN && value <= T::MAX {
            Some(Imm::<T, N> { value })
        } else {
            None
        }
    }
}

// is there a way to use generics here? getting issues with it
// overriding core's impl for Into
impl<const N: u8> TryFrom<u8> for Imm<u8, N> {
    type Error = ();

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        Self::new(value).ok_or(())
    }
}
impl<const N: u8> TryFrom<i8> for Imm<i8, N> {
    type Error = ();

    fn try_from(value: i8) -> Result<Self, Self::Error> {
        Self::new(value).ok_or(())
    }
}

pub type I5 = Imm<i8, 5>;
pub type U3 = Imm<u8, 3>;
pub type U4 = Imm<u8, 4>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_imm_i() {
        for i in -128..=127 {
            let x = Imm::<i8, 3>::new(i);
            if i >= -4 && i < 4 {
                assert!(x.is_some());
                assert_eq!(x.unwrap().get(), i);
            } else {
                assert!(x.is_none());
            }
        }
    }

    #[test]
    fn test_imm_u() {
        for i in 0..=255 {
            let x = Imm::<u8, 3>::new(i);
            if i < 8 {
                assert!(x.is_some());
                assert_eq!(x.unwrap().get(), i);
            } else {
                assert!(x.is_none());
            }
        }
    }
}
