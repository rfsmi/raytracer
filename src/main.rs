use std::sync::Arc;

use bevy::app::App;
use bvh::BVH;
use camera::Config;
use glam::DVec3;
use material::*;
use rand::{random, thread_rng, Rng};

use crate::{hit::Hit, sphere::Sphere};

mod aabb;
mod app;
mod bvh;
mod camera;
mod hit;
mod material;
mod ray;
mod sphere;
mod util;
mod vector;

macro_rules! make {
    (Metal albedo( $r:expr, $g:expr, $b:expr ) fuzz($f:expr)) => {
        Arc::new(Metal::new().fuzz($f).albedo(DVec3::new($r, $g, $b))) as Arc<dyn Material>
    };
    (Lambertian albedo( $r:expr, $g:expr, $b:expr )) => {
        Arc::new(Lambertian::new().albedo(DVec3::new($r, $g, $b))) as Arc<dyn Material>
    };
    (Dielectric ir( $ir:expr )) => {
        Arc::new(Dielectric::new().ir($ir)) as Arc<dyn Material>
    };
    (sphere $r:expr, ($x:expr, $y:expr, $z:expr), $material:ident) => {
        Box::new(Sphere::new(
            DVec3::new($x, $y, $z),
            $r,
            Arc::clone(&$material),
        )) as Box<dyn Hit>
    };
}

fn main() {
    let mut objects = Vec::new();

    let ground_material = make!(Lambertian albedo(0.5, 0.5, 0.5));
    objects.push(make!(sphere 1000.0, (0.0, -1000.0, 0.0), ground_material));

    let material1 = make!(Dielectric ir(1.5));
    objects.push(make!(sphere 1.0, (0.0, 1.0, 0.0), material1));

    let material2 = make!(Lambertian albedo(0.4, 0.2, 0.1));
    objects.push(make!(sphere 1.0, (-4.0, 1.0, 0.0), material2));

    let material3 = make!(Metal albedo(0.7, 0.6, 0.5) fuzz(0.0));
    objects.push(make!(sphere 1.0, (4.0, 1.0, 0.0), material3));

    for a in -11..11 {
        for b in -11..11 {
            let center = DVec3::new(
                a as f64 + 0.9 * random::<f64>(),
                0.2,
                b as f64 + 0.9 * random::<f64>(),
            );
            let material = match thread_rng().gen_range(0..3) {
                // Diffuse
                0 => {
                    let albedo = random::<DVec3>() * random::<DVec3>();
                    Arc::new(Lambertian::new().albedo(albedo)) as Arc<dyn Material>
                }
                // Metal
                1 => {
                    let albedo = vector::random_range(0.5..=1.0);
                    let fuzz: f64 = random();
                    Arc::new(Metal::new().albedo(albedo).fuzz(fuzz)) as Arc<dyn Material>
                }
                // Glass
                2 => Arc::new(Dielectric::new().ir(1.5)) as Arc<dyn Material>,
                _ => unreachable!(),
            };
            objects.push(Box::new(Sphere::new(center, 0.2, material)));
        }
    }

    let world = BVH::new(objects);

    let config = Config::new()
        .aspect_ratio(16.0 / 9.0)
        .image_width(830)
        .vfov(20.0)
        .lookfrom(DVec3::new(13.0, 2.0, 3.0))
        .lookat(DVec3::ZERO)
        .vup(DVec3::Y)
        .defocus_angle(0.6)
        .focus_dist(10.0)
        .samples_per_pixel(750)
        .max_depth(50);

    config.camera().render(&world);
}
