#![no_std]

use core::ops::Index;
pub use remote_obj_derive::{RemoteSetter, RemoteGetter, setter, getter};
use bincode::{Encode, Decode};

pub mod prelude {
    pub use crate::{
        RemoteSetter, RemoteGetter, setter, getter, Setter, Getter
    };
}

pub trait Setter {
    type SetterType: Default;
    fn set(&mut self, x: Self::SetterType) -> Result<(), ()>;
}

pub trait Getter {
    type ValueType;
    type GetterType: Default;

    fn get(&self, x: Self::GetterType) -> Result<Self::ValueType, ()> ;
}

macro_rules! impl_primitive {
    ($t:ty) => {
        impl Setter for $t {
            type SetterType = $t;

            fn set(&mut self, x: Self::SetterType) -> Result<(), ()>{
                *self = x;
                Ok(())
            }
        }

        impl Getter for $t {
            type ValueType = $t;
            type GetterType = ();

            fn get(&self, _: ()) -> Result<Self::ValueType, ()> {
                Ok(*self)
            }
        }
    }
}

impl_primitive!(());

impl_primitive!(i8);
impl_primitive!(i16);
impl_primitive!(i32);
impl_primitive!(i64);
impl_primitive!(isize);

impl_primitive!(u8);
impl_primitive!(u16);
impl_primitive!(u32);
impl_primitive!(u64);
impl_primitive!(usize);

impl_primitive!(f32);
impl_primitive!(f64);

impl_primitive!(bool);

impl_primitive!(char);

#[derive(Debug, Encode, Decode, Copy, Clone)]
pub struct ArrHelper<T> {
    r: T,
    idx: usize,
}

impl<T: Setter + Setter<SetterType = T> + Default> ArrHelper<T>  {
    pub fn arr_set<F>(self, idx: usize, func: F) -> Self where F: Fn(<T as Setter>::SetterType) -> <T as Setter>::SetterType {
        ArrHelper {
            r: func(<T as Setter>::SetterType::default()),
            idx
        }
    }
}

impl<T: Getter + Getter<GetterType = T> + Default> ArrHelper<T>  {
    pub fn arr_get<F>(self, idx: usize, func: F) -> Self where F: Fn(<T as Getter>::GetterType) -> <T as Getter>::GetterType {
        ArrHelper {
            r: func(<T as Getter>::GetterType::default()),
            idx
        }
    }
}

impl<T> Index<usize> for ArrHelper<T> {
    type Output = T;

    fn index(&self, index: usize) -> &Self::Output {
        assert_eq!(index, self.idx);
        return &self.r;
    }
}

impl<T: Default> Default for ArrHelper<T> {
    fn default() -> Self {
        ArrHelper {
            r: T::default(),
            idx: 0,
        }
    }
}

impl<T, const N: usize> Setter for [T; N]
    where
        T: Setter + Default,
{
    type SetterType = ArrHelper<T::SetterType>;

    fn set(&mut self, x: Self::SetterType) -> Result<(), ()> {
        match self.get_mut(x.idx) {
            Some(v) => {
                v.set(x.r)?;
                Ok(())
            }
            None => Err(()),
        }
    }
}

impl<T, const N: usize> Getter for [T; N]
    where
        T: Getter + Default,
{
    type ValueType = ArrHelper<T::ValueType>;
    type GetterType = ArrHelper<T::GetterType>;

    fn get(&self, x: Self::GetterType) -> Result<Self::ValueType, ()> {
        match (self as &[T]).get(x.idx) {
            Some(v) => {
                Ok(ArrHelper {
                    r: v.get(x.r)?,
                    idx: x.idx,
                })
            }
            None => Err(()),
        }
    }
}


impl<T> Setter for &mut T where T: Setter,
{
    type SetterType = T::SetterType;

    fn set(&mut self, x: Self::SetterType) -> Result<(), ()> {
        (**self).set(x)
    }
}


impl<T> Getter for &T where T: Getter,
{
    type ValueType = T::ValueType;
    type GetterType = T::GetterType;

    fn get(&self, x: Self::GetterType) -> Result<Self::ValueType, ()> {
        (**self).get(x)
    }
}

impl<T> Getter for &mut T where T: Getter,
{
    type ValueType = T::ValueType;
    type GetterType = T::GetterType;

    fn get(&self, x: Self::GetterType) -> Result<Self::ValueType, ()> {
        (**self).get(x)
    }
}