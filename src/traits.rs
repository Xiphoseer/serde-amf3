use super::Error;

pub(super) trait VisitInt {
    fn visit_int<'de, V: serde::de::Visitor<'de>>(visitor: V, v: u32) -> Result<V::Value, Error>;
}

macro_rules! impl_visit_int {
    ($f:ident $t:ty) => {
        impl VisitInt for $t {
            fn visit_int<'de, V: serde::de::Visitor<'de>>(
                visitor: V,
                v: u32,
            ) -> Result<V::Value, Error> {
                visitor.$f(v as $t)
            }
        }
    };
}

impl_visit_int!(visit_u8 u8);
impl_visit_int!(visit_u16 u16);
impl_visit_int!(visit_u32 u32);
impl_visit_int!(visit_u64 u64);
impl_visit_int!(visit_i8 i8);
impl_visit_int!(visit_i16 i16);
impl_visit_int!(visit_i32 i32);
impl_visit_int!(visit_i64 i64);
impl_visit_int!(visit_f32 f32);
impl_visit_int!(visit_f64 f64);

pub(super) trait VisitDouble {
    fn visit_double<'de, V: serde::de::Visitor<'de>>(visitor: V, v: f64)
        -> Result<V::Value, Error>;
}

macro_rules! impl_visit_double {
    ($f:ident $t:ty) => {
        impl VisitDouble for $t {
            fn visit_double<'de, V: serde::de::Visitor<'de>>(
                visitor: V,
                v: f64,
            ) -> Result<V::Value, Error> {
                visitor.$f(v as $t)
            }
        }
    };
}

impl_visit_double!(visit_f32 f32);
impl_visit_double!(visit_f64 f64);
impl_visit_double!(visit_i8 i8);
impl_visit_double!(visit_u8 u8);
impl_visit_double!(visit_i16 i16);
impl_visit_double!(visit_u16 u16);
impl_visit_double!(visit_i32 i32);
impl_visit_double!(visit_u32 u32);
impl_visit_double!(visit_i64 i64);
impl_visit_double!(visit_u64 u64);
