//! Generalization of numeric types for use in UI components.

use std::fmt::{Debug, Display};
use std::ops::{Add, Div, Mul, Sub};
use std::str::FromStr;

/// A trait for numeric values.
pub trait Number:
    PartialEq
    + PartialOrd
    + FromStr
    + ToString
    + Default
    + Clone
    + Copy
    + Display
    + Debug
    + Add<Self, Output = Self>
    + Sub<Self, Output = Self>
    + Mul<Self, Output = Self>
    + Div<Self, Output = Self>
{
    /// The minimum value for this number type.
    const NUMBER_MIN: Self;

    /// The maximum value for this number type.
    const NUMBER_MAX: Self;

    /// The typical step value for this number type.
    const NUMBER_STEP: Self;

    /// Can this number type represent decimal values?
    const DECIMAL: bool;
}

/// Implements the `Number` trait for integer primitives.
macro_rules! impl_number_int {
    ( $($ty:ty),* ) => {
        $(
            impl Number for $ty {
                const NUMBER_MIN: Self = Self::MIN;
                const NUMBER_MAX: Self = Self::MAX;
                const NUMBER_STEP: Self = 1;
                const DECIMAL: bool = false;
            }
        )*
    };
}

/// Implements the `Number` trait for floating point primitives.
macro_rules! impl_number_float {
    ( $($ty:ty),* ) => {
        $(
            impl Number for $ty {
                const NUMBER_MIN: Self = Self::MIN;
                const NUMBER_MAX: Self = Self::MAX;
                const NUMBER_STEP: Self = 1.0;
                const DECIMAL: bool = true;
            }
        )*
    };
}

impl_number_int!(i8, i16, i32, i64, i128, isize, u8, u16, u32, u64, u128, usize);

impl_number_float!(f32, f64);
