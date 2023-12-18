use ::rand::thread_rng;
use glam::DVec3;
use rand::{distributions::uniform::SampleRange, Rng};

pub fn random_range<R>(range: R) -> DVec3
where
    R: Clone + SampleRange<f64>,
{
    DVec3::new(
        thread_rng().gen_range(range.clone()),
        thread_rng().gen_range(range.clone()),
        thread_rng().gen_range(range),
    )
}

pub fn random_unit() -> DVec3 {
    random_within_unit_sphere().normalize()
}

pub fn random_within_unit_sphere() -> DVec3 {
    loop {
        let p = random_range(-1.0..=1.0);
        if p.length_squared() < 1.0 {
            return p;
        }
    }
}

pub fn random_within_unit_disk() -> DVec3 {
    loop {
        let p = DVec3::new(
            thread_rng().gen_range(-1.0..=1.0),
            thread_rng().gen_range(-1.0..=1.0),
            0.0,
        );
        if p.length_squared() < 1.0 {
            return p;
        }
    }
}

// pub fn random_on_hemisphere(normal: DVec3) -> DVec3 {
//     let p = random_unit();
//     if normal.dot(p) > 0.0 {
//         // Is the same hemisphere as the normal
//         p
//     } else {
//         -p
//     }
// }

pub fn near_zero(v: DVec3) -> bool {
    v.abs_diff_eq(DVec3::ZERO, 1e-6)
}

pub fn reflect(v: DVec3, n: DVec3) -> DVec3 {
    v - 2.0 * v.dot(n) * n
}

pub fn refract(v: DVec3, n: DVec3, etai_over_etat: f64) -> DVec3 {
    let cos_theta = (-v).dot(n).min(1.0);
    let r_out_perp = etai_over_etat * (v + cos_theta * n);
    let r_out_parallel = -(1.0 - r_out_perp.length_squared()).abs().sqrt() * n;
    r_out_perp + r_out_parallel
}
