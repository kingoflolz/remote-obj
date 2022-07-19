#![no_std]
pub use remote_obj_derive::{RemoteSetter, RemoteGetter, setter, getter};

pub trait Setter {
    type SetterType: Default;
    fn set(&mut self, x: Self::SetterType) -> Result<(), ()>;
}

pub trait Getter {
    type ValueType;
    type GetterType;

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
