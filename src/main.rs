use std::sync::Arc;

use camera::Config;
use material::*;
use rand::{random, thread_rng, Rng};
use vector::{P3, V3};

use crate::{hit::HitList, sphere::Sphere};

mod camera;
mod hit;
mod material;
mod ray;
mod sphere;
mod util;
mod vector;

macro_rules! make {
    (Metal albedo( $r:expr, $g:expr, $b:expr ) fuzz($f:expr)) => {
        Arc::new(Metal::new().fuzz($f).albedo(V3::new().x($r).y($g).z($b))) as Arc<dyn Material>
    };
    (Lambertian albedo( $r:expr, $g:expr, $b:expr )) => {
        Arc::new(Lambertian::new().albedo(V3::new().x($r).y($g).z($b))) as Arc<dyn Material>
    };
    (Dielectric ir( $ir:expr )) => {
        Arc::new(Dielectric::new().ir($ir)) as Arc<dyn Material>
    };
    (sphere $r:expr,  ($x:expr, $y:expr, $z:expr), $material:ident) => {
        Box::new(Sphere::new(
            P3::new().x($x).y($y).z($z),
            $r,
            Arc::clone(&$material),
        ))
    };
}

fn main() {
    let ground_material = make!(Lambertian albedo(0.5, 0.5, 0.5));
    let mut world = HitList::new(make!(sphere 1000.0, (0.0, -1000.0, 0.0), ground_material));

    let material1 = make!(Dielectric ir(1.5));
    world.add(make!(sphere 1.0, (0.0, 1.0, 0.0), material1));

    let material2 = make!(Lambertian albedo(0.4, 0.2, 0.1));
    world.add(make!(sphere 1.0, (-4.0, 1.0, 0.0), material2));

    let material3 = make!(Metal albedo(0.7, 0.6, 0.5) fuzz(0.0));
    world.add(make!(sphere 1.0, (4.0, 1.0, 0.0), material3));

    for a in -11..11 {
        for b in -11..11 {
            let center = P3::new()
                .x(a as f64 + 0.9 * random::<f64>())
                .y(0.2)
                .z(b as f64 + 0.9 * random::<f64>());
            let material = match thread_rng().gen_range(0..3) {
                // Diffuse
                0 => {
                    let albedo = V3::random() * V3::random();
                    Arc::new(Lambertian::new().albedo(albedo)) as Arc<dyn Material>
                }
                // Metal
                1 => {
                    let albedo = V3::random_range(0.5..=1.0);
                    let fuzz: f64 = random();
                    Arc::new(Metal::new().albedo(albedo).fuzz(fuzz)) as Arc<dyn Material>
                }
                // Glass
                2 => Arc::new(Dielectric::new().ir(1.5)) as Arc<dyn Material>,
                _ => unreachable!(),
            };
            world.add(Box::new(Sphere::new(center, 0.2, material)));
        }
    }

    let config = Config::new()
        .aspect_ratio(16.0 / 9.0)
        .image_width(400)
        .vfov(20.0)
        .lookfrom(P3::new().x(13.0).y(2.0).z(3.0))
        .lookat(P3::new())
        .vup(V3::new().y(1.0))
        .defocus_angle(0.6)
        .focus_dist(10.0)
        .samples_per_pixel(50)
        // .samples_per_pixel(500)
        .max_depth(50);
    config.camera().render(&world);
}
