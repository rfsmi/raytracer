use ::rand::thread_rng;
use paste::paste;
use rand::{distributions::uniform::SampleRange, random, Rng};
use std::ops::*;

use crate::bvh::Axis;
use crate::util::*;

default_struct!(P3 {
    x: f64 = 0.0,
    y: f64 = 0.0,
    z: f64 = 0.0,
});

default_struct!(V3 {
    x: f64 = 0.0,
    y: f64 = 0.0,
    z: f64 = 0.0,
});

define_ops!((V3) (+*-/) = (V3));
define_ops!((V3) (+*-/) = (f64));
define_ops!((V3) (+*-/) (V3) -> (V3));
define_ops!((V3) (+*-/) (f64) -> (V3));
define_ops!((f64) (+*-/) (V3) -> (V3));

define_ops!((P3) (+-) = (V3));
define_ops!((P3) (+-) = (f64));
define_ops!((P3) (-) (P3) -> (V3));
define_ops!((P3) (+-) (V3) -> (P3));
define_ops!((V3) (+-) (P3) -> (P3));
define_ops!((P3) (+-) (f64) -> (P3));
define_ops!((f64) (+-) (P3) -> (P3));

macro_rules! utility_funcs {
    ($type:ty) => {
        impl $type {
            pub fn all(v: f64) -> Self {
                Self::new().x(v).y(v).z(v)
            }

            pub fn axis(&self, axis: Axis) -> &f64 {
                match axis {
                    Axis::X => &self.x,
                    Axis::Y => &self.y,
                    Axis::Z => &self.z,
                }
            }

            pub fn axis_mut(&mut self, axis: Axis) -> &mut f64 {
                match axis {
                    Axis::X => &mut self.x,
                    Axis::Y => &mut self.y,
                    Axis::Z => &mut self.z,
                }
            }

            pub fn random() -> Self {
                Self {
                    x: random(),
                    y: random(),
                    z: random(),
                }
            }

            pub fn random_range<R>(range: R) -> Self
            where
                R: Clone + SampleRange<f64>,
            {
                Self {
                    x: thread_rng().gen_range(range.clone()),
                    y: thread_rng().gen_range(range.clone()),
                    z: thread_rng().gen_range(range),
                }
            }

            pub fn near_zero(&self) -> bool {
                const EPS: f64 = 1e-8;
                self.x.abs() < EPS && self.y.abs() < EPS && self.z.abs() < EPS
            }
        }
    };
}

utility_funcs!(P3);
utility_funcs!(V3);

impl V3 {
    pub fn length_squared(&self) -> f64 {
        self.dot(self)
    }

    pub fn length(&self) -> f64 {
        self.length_squared().sqrt()
    }

    pub fn dot(&self, other: &Self) -> f64 {
        self.x * other.x + self.y * other.y + self.z * other.z
    }

    pub fn cross(&self, other: &Self) -> Self {
        Self {
            x: self.y * other.z - self.z * other.y,
            y: self.z * other.x - self.x * other.z,
            z: self.x * other.y - self.y * other.x,
        }
    }

    pub fn unit(&self) -> Self {
        self / self.length()
    }

    pub fn random_unit() -> Self {
        Self::random_within_unit_sphere().unit()
    }

    fn random_within_unit_sphere() -> Self {
        loop {
            let p = Self::random_range(-1.0..=1.0);
            if p.length_squared() < 1.0 {
                return p;
            }
        }
    }

    pub fn random_within_unit_disk() -> Self {
        loop {
            let p = Self::new()
                .x(thread_rng().gen_range(-1.0..=1.0))
                .y(thread_rng().gen_range(-1.0..=1.0));
            if p.length_squared() < 1.0 {
                return p;
            }
        }
    }

    pub fn random_on_hemisphere(normal: &V3) -> Self {
        let p = Self::random_unit();
        if normal.dot(&p) > 0.0 {
            // Is the same hemisphere as the normal
            p
        } else {
            -p
        }
    }

    pub fn reflect(&self, n: &V3) -> Self {
        self - 2.0 * self.dot(n) * n
    }

    pub fn refract(&self, n: &V3, etai_over_etat: f64) -> V3 {
        let cos_theta = (-self).dot(n).min(1.0);
        let r_out_perp = etai_over_etat * (self + cos_theta * n);
        let r_out_parallel = -(1.0 - r_out_perp.length_squared()).abs().sqrt() * n;
        r_out_perp + r_out_parallel
    }
}

impl Neg for &V3 {
    type Output = V3;

    fn neg(self) -> Self::Output {
        V3::new() - self
    }
}

impl Neg for V3 {
    type Output = V3;

    fn neg(self) -> Self::Output {
        Self::new() - self
    }
}

macro_rules! element_wise_op {
    (binary $($func:tt)*) => {
        impl V3 {
            pub fn $($func)*(&self, other: Self) -> Self {
                Self::new()
                    .x(self.x.$($func)*(other.x))
                    .y(self.y.$($func)*(other.y))
                    .z(self.z.$($func)*(other.z))
            }
        }
    };
    ($($func:tt)*) => {
        impl V3 {
            pub fn $($func)*(&self) -> Self {
                Self::new()
                    .x(self.x.$($func)*())
                    .y(self.y.$($func)*())
                    .z(self.z.$($func)*())
            }
        }
    };
}

element_wise_op!(abs);
element_wise_op!(signum);
element_wise_op!(binary min);
element_wise_op!(binary max);
