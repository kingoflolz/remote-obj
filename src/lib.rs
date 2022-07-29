#![no_std]

use core::any::Any;
use core::hash::Hash;
use core::mem::size_of;
use core::ops::Index;
pub use remote_obj_derive::{RemoteSetter, RemoteGetter, setter, getter};
use bincode::{Encode, Decode};

pub mod prelude {
    pub use crate::{
        RemoteSetter, RemoteGetter, setter, getter, Setter, Getter, Value
    };
    pub use core::any::Any;
}

pub trait Value {
    fn dehydrate(&self, x: &mut [u8]) -> Option<usize>;
    fn as_float(&self) -> Option<f32> {
        None
    }
}

pub trait Setter {
    type SetterType: Default;
    fn set(&mut self, x: Self::SetterType) -> Result<(), ()>;
}

pub trait Getter {
    type ValueType: Value;
    type GetterType: Default + Hash + Eq + Clone + Copy;

    fn get(&self, x: Self::GetterType) -> Result<Self::ValueType, ()> ;

    fn hydrate(x: Self::GetterType, buf: &[u8]) -> Result<(Self::ValueType, usize), ()> ;
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

            fn hydrate(_: (), buf: &[u8]) -> Result<(Self::ValueType, usize), ()> {
                const SIZE: usize = size_of::<$t>();
                if buf.len() < SIZE {
                    return Err(());
                }
                Ok((<$t>::from_le_bytes(buf[0..SIZE].try_into().unwrap()), SIZE))
            }
        }
    }
}

// impl_primitive!(());
impl Setter for () {
    type SetterType = ();

    fn set(&mut self, x: Self::SetterType) -> Result<(), ()>{
        *self = x;
        Ok(())
    }
}

impl Getter for () {
    type ValueType = ();
    type GetterType = ();

    fn get(&self, _: ()) -> Result<Self::ValueType, ()> {
        Ok(*self)
    }

    fn hydrate(_: (), _: &[u8]) -> Result<(Self::ValueType, usize), ()> {
        Ok(((), 0))
    }
}

impl Value for () {
    fn dehydrate(&self, _: &mut [u8]) -> Option<usize> {
        Some(0)
    }
}

macro_rules! impl_int_primitive {
    ($t:ty) => {
        impl Value for $t {
            fn dehydrate(&self, x: &mut [u8]) -> Option<usize> {
                let buf = self.to_le_bytes();
                for (idx, i) in buf.iter().enumerate() {
                    x[idx] = *i;
                }
                Some(buf.len())
            }

            fn as_float(&self) -> Option<f32> {
                Some(*self as f32)
            }
        }
    }
}

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

impl_int_primitive!(i8);
impl_int_primitive!(i16);
impl_int_primitive!(i32);
impl_int_primitive!(i64);
impl_int_primitive!(isize);

impl_int_primitive!(u8);
impl_int_primitive!(u16);
impl_int_primitive!(u32);
impl_int_primitive!(u64);
impl_int_primitive!(usize);

impl_primitive!(f32);
impl_primitive!(f64);

impl_int_primitive!(f32);
impl_int_primitive!(f64);

// impl_primitive!(bool);

impl Value for bool {
    fn dehydrate(&self, x: &mut [u8]) -> Option<usize> {
        match *self {
            true => {
                x[0] = 0;
            },
            false => {
                x[0] = 1;
            }
        };
        Some(1)
    }
}

#[derive(Debug, Encode, Decode, Clone, Hash, PartialEq, Eq, Copy)]
pub struct ArrHelper<T> {
    r: T,
    idx: usize,
}

impl<T: Setter + Setter<SetterType = T> + Default + Hash + PartialEq + Eq> ArrHelper<T>  {
    pub fn arr_set<F>(self, idx: usize, func: F) -> Self where F: Fn(<T as Setter>::SetterType) -> <T as Setter>::SetterType {
        ArrHelper {
            r: func(<T as Setter>::SetterType::default()),
            idx
        }
    }
}

impl<T: Getter + Getter<GetterType = T> + Default + Hash + PartialEq + Eq> ArrHelper<T>  {
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

impl<T: Value> Value for ArrHelper<T> {
    fn dehydrate(&self, x: &mut [u8]) -> Option<usize> {
        self.r.dehydrate(x)
    }
    fn as_float(&self) -> Option<f32> {
        self.r.as_float()
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

    fn hydrate(x: Self::GetterType, buf: &[u8]) -> Result<(Self::ValueType, usize), ()> {
        let (val, len) = T::hydrate(x.r, buf)?;
        Ok((ArrHelper {
            r: val,
            idx: x.idx,
        }, len))
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

    fn hydrate(x: Self::GetterType, buf: &[u8]) -> Result<(Self::ValueType, usize), ()> {
        T::hydrate(x, buf)
    }
}

impl<T> Getter for &mut T where T: Getter,
{
    type ValueType = T::ValueType;
    type GetterType = T::GetterType;

    fn get(&self, x: Self::GetterType) -> Result<Self::ValueType, ()> {
        (**self).get(x)
    }

    fn hydrate(x: Self::GetterType, buf: &[u8]) -> Result<(Self::ValueType, usize), ()> {
        T::hydrate(x, buf)
    }
}