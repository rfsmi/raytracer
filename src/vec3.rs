use ::rand::thread_rng;
use paste::paste;
use rand::{distributions::uniform::SampleRange, random, Rng};
use std::ops::*;

use crate::util::default_struct;

macro_rules! element_wise {
    [[$($a:tt)+] [$($b:tt)+] [$($c:tt)+] []] => {
        Vec3 { x: $($a)+, y: $($b)+, z: $($c)+ }
    };
    [[$($a:tt)+] [$($b:tt)+] [$($c:tt)+] [$($op:tt)+]] => {
        $($a)+ $($op)+ $($b)+ $($op)+ $($c)+
    };
    [[$($a:tt)*] [$($b:tt)*] [$($c:tt)*] [] $name:ident $($rest:tt)*] => {
        element_wise![
            [$($a)* $name.x] [$($b)* $name.y] [$($c)* $name.z]
            []
            $($rest)*
        ]
    };
    [[$($a:tt)*] [$($b:tt)*] [$($c:tt)*] [] ($($op:tt)*) $($rest:tt)*] => {
        element_wise![
            [$($a)* $($op)*] [$($b)* $($op)*] [$($c)* $($op)*]
            []
            $($rest)*
        ]
    };
    [[$($a:tt)*] [$($b:tt)*] [$($c:tt)*] [] => ($($op:tt)*) $($rest:tt)*] => {
        element_wise![
            [$($a)*] [$($b)*] [$($c)*]
            [$($op)*]
            $($rest)*
        ]
    };
    ($($input:tt)*) => {
        element_wise![[] [] [] [] $($input)*]
    };
}

macro_rules! binary_op {
    [$trait:path, $op:tt, $rhs:ty] => {
        paste!{
            impl $trait<$rhs> for Vec3 {
                type Output = Vec3;
                fn [<$trait:lower>](self, rhs: $rhs) -> Self::Output {
                    element_wise!(self ($op rhs))
                }
            }
        }
        paste!{
            impl [<$trait Assign>]<$rhs> for Vec3 {
                fn [<$trait:lower _assign>](&mut self, rhs: $rhs) {
                    *self = element_wise!(self ($op rhs));
                }
            }
        }
        paste!{
            impl $trait<Vec3> for $rhs {
                type Output = Vec3;
                fn [<$trait:lower>](self, rhs: Vec3) -> Self::Output {
                    element_wise!((self $op) rhs)
                }
            }
        }
    };
    [$trait:path, $op:tt] => {
        paste!{
            impl [<$trait Assign>] for Vec3 {
                fn [<$trait:lower _assign>](&mut self, rhs: Self) {
                    *self = element_wise!(self ($op) rhs);
                }
            }
        }
        paste!{
            impl $trait for Vec3 {
                type Output = Vec3;
                fn [<$trait:lower>](self, rhs: Self) -> Self::Output {
                    element_wise!(self ($op) rhs)
                }
            }
        }
    };
    ($trait:path: $op:tt) => {
        binary_op![$trait, $op, f64];
        binary_op![$trait, $op];
    }
}

default_struct!(Vec3 {
    x: f64 = 0.0,
    y: f64 = 0.0,
    z: f64 = 0.0,
});

impl Vec3 {
    pub fn length_squared(&self) -> f64 {
        self.dot(self)
    }

    pub fn length(&self) -> f64 {
        self.length_squared().sqrt()
    }

    pub fn dot(&self, other: &Self) -> f64 {
        element_wise!(self (*) other => (+))
    }

    pub fn cross(&self, other: &Self) -> Self {
        Self {
            x: self.y * other.z - self.z * other.y,
            y: self.z * other.x - self.x * other.z,
            z: self.x * other.y - self.y * other.x,
        }
    }

    pub fn unit(&self) -> Self {
        *self / self.length()
    }

    pub fn random() -> Self {
        Vec3::new().x(random()).y(random()).z(random())
    }

    pub fn random_range<R>(range: R) -> Self
    where
        R: Clone + SampleRange<f64>,
    {
        Vec3::new()
            .x(thread_rng().gen_range(range.clone()))
            .y(thread_rng().gen_range(range.clone()))
            .z(thread_rng().gen_range(range))
    }

    fn random_within_unit_sphere() -> Self {
        loop {
            let p = Self::random_range(-1.0..=1.0);
            if p.length_squared() < 1.0 {
                return p;
            }
        }
    }

    pub fn random_unit() -> Self {
        Self::random_within_unit_sphere().unit()
    }

    pub fn random_on_hemisphere(normal: &Vec3) -> Self {
        let p = Self::random_unit();
        if normal.dot(&p) > 0.0 {
            // Is the same hemisphere as the normal
            p
        } else {
            -p
        }
    }

    pub fn reflect(&self, normal: &Vec3) -> Self {
        *self - 2.0 * self.dot(normal) * *normal
    }

    pub fn near_zero(&self) -> bool {
        element_wise!(self (< 1e-8) => (&&))
    }
}

impl Neg for Vec3 {
    type Output = Vec3;

    fn neg(self) -> Self::Output {
        element_wise!((-) self)
    }
}

binary_op!(Add: +);
binary_op!(Sub: -);
binary_op!(Mul: *);
binary_op!(Div: /);

// default_struct!(P3 {
//     x: f64 = 0.0,
//     y: f64 = 0.0,
//     z: f64 = 0.0,
// });

// default_struct!(V3 {
//     x: f64 = 0.0,
//     y: f64 = 0.0,
//     z: f64 = 0.0,
// });

// macro_rules! define_impl {
//     ([] [] [$($op:tt)*] [] f64, $($rest:tt)*) => {
//         define_impl!([] [f64, self, self, self] [$($op)*] [] $($rest)*)
//     };
//     ([] [] [$($op:tt)*] [] $lhs:ty, $($rest:tt)*) => {
//         define_impl!([] [$lhs, self.x, self.y, self.z] [$($op)*] [] $($rest)*)
//     };
//     ([] [$($lhs:tt)*] [$($op:tt)*] [] f64, $($rest:tt)*) => {
//         define_impl!([] [$($lhs)*] [$($op)*] [f64, self, self, self] $($rest)*)
//     };
//     ([] [$($lhs:tt)*] [$($op:tt)*] [] $rhs:ty, $($rest:tt)*) => {
//         define_impl!([] [$($lhs)*] [$($op)*] [$rhs, self.x, self.y, self.z] $($rest)*)
//     };
//     ([$trait:path] [$($lhs:tt)*] [$op:tt] [$($rhs:tt)*] $res:ty) => {
//         paste! {
//             impl $trait<$rhs> for $lhs {
//                 type Output = $rhs;
//                 fn [<$trait:lower>](self, rhs: $rhs) -> Self::Output {
//                     panic!()
//                 }
//             }
//         }
//     };
// }

// /*
//     algebra!{
//         +-   :: <P3,  V3> -> P3,
//         +*-/ :: <V3, f64> -> V3,
//     };
// */
// macro_rules! algebra {
//     { @ [$($ops:tt)*] + $($rest:tt)* } => {
//         algebra!{ @ [(Add, +) $($ops)*] $($rest)* }
//     };
//     { @ [$($ops:tt)*] - $($rest:tt)* } => {
//         algebra!{ @ [(Sub, -) $($ops)*] $($rest)* }
//     };
//     { @ [$($ops:tt)*] * $($rest:tt)* } => {
//         algebra!{ @ [(Mul, *) $($ops)*] $($rest)* }
//     };
//     { @ [$($ops:tt)*] / $($rest:tt)* } => {
//         algebra!{ @ [(Div, /) $($ops)*] $($rest)* }
//     };
//     {
//         @ []
//         :: < $lhs:ty, $rhs:ty > -> $res:ty ,
//         $($rest:tt)*
//     } => {
//         algebra!{ @ [] $($rest)* }
//     };
//     {
//         @ [($trait:path, $sym:tt) $($ops:tt)*]
//         :: < $lhs:ty, $rhs:ty > -> $res:ty ,
//         $($rest:tt)*
//     } => {
//         paste! {
//             impl $trait<$rhs> for $lhs {
//                 type Output = $rhs;
//                 fn [<$trait:lower>](self, rhs: $rhs) -> Self::Output {
//                     panic!()
//                 }
//             }
//         }
//         algebra!{
//             @ [$($ops)*]
//             :: < $lhs, $rhs > -> $res ,
//             $($rest)*
//         }
//     };
//     { @ [] } => {};
//     [$($input:tt)*] => {
//         algebra! { @ [] $($input)* }
//     };
// }

// algebra! [
//     +-   :: <P3,  V3> -> P3,
//     +*-/ :: <V3, f64> -> V3,
// ];
