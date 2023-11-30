use paste::paste;
use std::ops::*;

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

#[derive(Default, Clone, Copy)]
pub struct Vec3 {
    pub x: f64,
    pub y: f64,
    pub z: f64,
}

impl Vec3 {
    pub const U: Self = Self {
        x: 1.0,
        y: 0.0,
        z: 0.0,
    };

    pub const V: Self = Self {
        x: 0.0,
        y: 1.0,
        z: 0.0,
    };

    pub const W: Self = Self {
        x: 0.0,
        y: 0.0,
        z: 1.0,
    };

    pub const UVW: Self = Self {
        x: 1.0,
        y: 1.0,
        z: 1.0,
    };

    pub fn new(x: f64, y: f64, z: f64) -> Self {
        Self { x, y, z }
    }

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
