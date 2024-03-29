#![no_std]

use core::fmt;
use core::fmt::{Display, Formatter};
use core::hash::Hash;
use core::mem::size_of;
use core::ops::Index;
pub use remote_obj_derive::{RemoteSetter, RemoteGetter, setter, getter};
use bincode::{Encode, Decode};

pub mod prelude {
    pub use crate::{
        RemoteSetter, RemoteGetter, setter, getter, Setter, Getter, Value, RemoteSet, RemoteGet, NullGetter, FieldsType
    };
    pub use core::any::Any;
}

#[derive(Hash, Eq, Clone, Copy, PartialEq, Debug)]
pub enum FieldsType {
    Fields(&'static [&'static str]),
    Arr(usize),
    Terminal
}

pub trait Value: Sized + Copy {
    fn dehydrate(&self, x: &mut [u8]) -> Option<usize>;
    fn as_float(&self) -> Option<f32> {
        None
    }
    fn parse_value<T: Sized>(self, x: &str) -> Option<T> {
        if x.is_empty() {
            assert_eq!(size_of::<Self>(), size_of::<T>(), "Value::parse_value: size mismatch");
            Some(unsafe { core::mem::transmute_copy::<Self, T>(&self) })
        } else {
            None
        }
    }
}

pub trait Setter: Default + Sized + Copy + Display {
    fn parse_setter<T: Sized>(&self, x: &str, set: T) -> Option<Self> {
        if x.is_empty() {
            assert_eq!(size_of::<Self>(), size_of::<T>(), "Setter::parse_setter: size mismatch");
            Some(unsafe { core::mem::transmute_copy::<T, Self>(&set) })
        } else {
            None
        }
    }

    fn parse_setter_numeric(&self, x: &str, set: f64) -> Option<Self>;
}

pub trait RemoteSet {
    type SetterType: Setter;
    fn set(&mut self, x: Self::SetterType) -> Result<(), ()>;

    fn dynamic_setter<T>(x: &str, set: T) -> Option<Self::SetterType>
    {
        Self::SetterType::parse_setter::<T>(&Self::SetterType::default(), x, set)
    }
    fn dynamic_setter_numeric(x: &str, set: f64) -> Option<Self::SetterType>
    {
        Self::SetterType::parse_setter_numeric(&Self::SetterType::default(), x, set)
    }
}

pub trait Getter: Default + Hash + Eq + Clone + Copy + Display {
    fn parse_getter(x: &str) -> Option<Self> {
        if x.is_empty() {
            Some(Self::default())
        } else {
            None
        }
    }

    fn get_fields(x: &str) -> Option<FieldsType> {
        if x.is_empty() {
            Some(FieldsType::Terminal)
        } else {
            None
        }
    }
}

pub trait RemoteGet {
    type ValueType: Value;
    type GetterType: Getter;

    fn get(&self, x: Self::GetterType) -> Result<Self::ValueType, ()> ;

    fn hydrate(x: Self::GetterType, buf: &[u8]) -> Result<(Self::ValueType, usize), ()> ;

    fn dynamic_getter(x: &str) -> Option<Self::GetterType>
    {
        Self::GetterType::parse_getter(x)
    }
}

#[derive(Encode, Decode, Default, Hash, Eq, Clone, Copy, PartialEq, Debug)]
pub struct NullGetter;

#[derive(Encode, Decode, Default, Hash, Eq, Clone, Copy, PartialEq, Debug)]
pub struct NullSetter;

impl Value for () {
    fn dehydrate(&self, _x: &mut [u8]) -> Option<usize> {
        Some(0)
    }
}

impl Display for NullSetter {
    fn fmt(&self, _: &mut Formatter<'_>) -> fmt::Result {
        Ok(())
    }
}

impl Setter for NullSetter {
    fn parse_setter_numeric(&self, _x: &str, _set: f64) -> Option<Self> {
        None
    }
}

impl Display for NullGetter {
    fn fmt(&self, _: &mut Formatter<'_>) -> fmt::Result {
        Ok(())
    }
}

impl Getter for NullGetter {}

macro_rules! impl_num_primitive {
    ($t:ty) => {
        impl RemoteSet for $t {
            type SetterType = $t;

            fn set(&mut self, x: Self::SetterType) -> Result<(), ()> {
                *self = x;
                Ok(())
            }
        }

        impl RemoteGet for $t {
            type ValueType = Self;
            type GetterType = NullGetter;

            fn get(&self, _: Self::GetterType) -> Result<Self::ValueType, ()> {
                Ok(*self)
            }

            fn hydrate(_: NullGetter, buf: &[u8]) -> Result<(Self::ValueType, usize), ()> {
                const SIZE: usize = size_of::<$t>();
                if buf.len() < SIZE {
                    return Err(());
                }
                Ok((<$t>::from_le_bytes(buf[0..SIZE].try_into().unwrap()), SIZE))
            }
        }

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

        impl Setter for $t {
            fn parse_setter_numeric(&self, x: &str, set: f64) -> Option<Self> {
                if x.is_empty() {
                    Some(set as Self)
                } else {
                    None
                }
            }
        }
    }
}

impl_num_primitive!(u8);
impl_num_primitive!(u16);
impl_num_primitive!(u32);
impl_num_primitive!(u64);

impl_num_primitive!(i8);
impl_num_primitive!(i16);
impl_num_primitive!(i32);
impl_num_primitive!(i64);

impl_num_primitive!(f32);
impl_num_primitive!(f64);

#[derive(Debug, Encode, Decode, Clone, Hash, PartialEq, Eq, Copy)]
pub struct ArrHelper<T, const N: usize> where T: Copy {
    r: T,
    idx: usize,
}

impl <T, const N: usize> ArrHelper<T, N> where T: Copy {
    pub fn new(r: T, idx: usize) -> Self {
        ArrHelper {
            r,
            idx,
        }
    }
}

impl<T, const N: usize> RemoteSet for [T; N] where T: RemoteSet,
{
    type SetterType = ArrHelper<T::SetterType, N>;

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

impl<T: Copy + Default, const N: usize> Default for ArrHelper<T, N> {
    fn default() -> Self {
        ArrHelper {
            r: T::default(),
            idx: 0,
        }
    }
}

impl<T: Copy + Default + Display, const N: usize> Display for ArrHelper<T, N> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "[{}]{}", self.idx, self.r)
    }
}

impl<T: Copy + Default + Setter, const N: usize> Setter for ArrHelper<T, N> {
    fn parse_setter<I: Sized>(&self, x: &str, set: I) -> Option<Self> {
        let l_bracket = x.find('[')?;
        let r_bracket = x.find(']')?;
        let idx = x[l_bracket + 1..r_bracket].parse::<usize>().ok()?;
        Some(ArrHelper {
            r: T::default().parse_setter(&x[r_bracket + 1..], set)?,
            idx,
        })
    }

    fn parse_setter_numeric(&self, x: &str, set: f64) -> Option<Self> {
        let l_bracket = x.find('[')?;
        let r_bracket = x.find(']')?;
        let idx = x[l_bracket + 1..r_bracket].parse::<usize>().ok()?;
        Some(ArrHelper {
            r: T::default().parse_setter_numeric(&x[r_bracket + 1..], set)?,
            idx,
        })
    }
}

impl<T, const N: usize> RemoteGet for [T; N] where T: RemoteGet,
{
    type ValueType = ArrHelper<T::ValueType, N>;
    type GetterType = ArrHelper<T::GetterType, N>;

    fn get(&self, x: Self::GetterType) -> Result<Self::ValueType, ()> {
        self[x.idx].get(x.r).map(|v| ArrHelper {
            r: v,
            idx: x.idx,
        })
    }

    fn hydrate(x: Self::GetterType, buf: &[u8]) -> Result<(Self::ValueType, usize), ()> {
        T::hydrate(x.r, buf).map(|(v, s)| (ArrHelper {
            r: v,
            idx: x.idx,
        }, s))
    }
}

impl<T: Copy + Value, const N: usize> Value for ArrHelper<T, N> {
    fn dehydrate(&self, x: &mut [u8]) -> Option<usize> {
        self.r.dehydrate(x)
    }

    fn as_float(&self) -> Option<f32> {
        self.r.as_float()
    }

    fn parse_value<I: Sized>(self, x: &str) -> Option<I> {
        let l_bracket = x.find('[')?;
        let r_bracket = x.find(']')?;
        let idx = x[l_bracket + 1..r_bracket].parse::<usize>().ok()?;
        if idx == self.idx {
            Some(self.r.parse_value(&x[r_bracket + 1..])?)
        } else {
            None
        }
    }
}

impl<T: Copy + Default + Getter, const N: usize> Getter for ArrHelper<T, N> {
    fn parse_getter(x: &str) -> Option<Self> {
        let l_bracket = x.find('[')?;
        let r_bracket = x.find(']')?;
        let idx = x[l_bracket + 1..r_bracket].parse::<usize>().ok()?;
        Some(ArrHelper {
            r: T::parse_getter(&x[r_bracket + 1..])?,
            idx,
        })
    }

    fn get_fields(x: &str) -> Option<FieldsType> {
        if x.is_empty() {
            return Some(FieldsType::Arr(N))
        }
        let l_bracket = x.find('[')?;
        let r_bracket = x.find(']')?;
        let idx = x[l_bracket + 1..r_bracket].parse::<usize>().ok()?;

        if idx >= N {
            return None;
        } else {
            T::get_fields(&x[r_bracket + 1..])
        }
    }
}


impl<T: Copy, const N: usize> Index<usize> for ArrHelper<T, N> {
    type Output = T;

    fn index(&self, index: usize) -> &Self::Output {
        assert_eq!(index, self.idx);
        return &self.r;
    }
}

impl<T: Getter, const N: usize> ArrHelper<T, N> {
    pub fn arr_get<F>(self, idx: usize, func: F) -> Self where F: Fn(T) -> T {
        ArrHelper::new(func(T::default()), idx)
    }
}

impl<T: Setter, const N: usize> ArrHelper<T, N> {
    pub fn arr_set<F>(self, idx: usize, func: F) -> Self where F: Fn(T) -> T {
        ArrHelper::new(func(T::default()), idx)
    }
}