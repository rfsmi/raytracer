use std::rc::Rc;

use camera::Config;
use material::*;
use vector::{P3, V3};

use crate::{hit_list::HitList, sphere::Sphere};

mod camera;
mod hit;
mod hit_list;
mod interval;
mod material;
mod ray;
mod sphere;
mod util;
mod vector;

macro_rules! make {
    (Metal albedo( $r:expr, $g:expr, $b:expr ) fuzz($f:expr)) => {
        Box::new(Metal::new().fuzz($f).albedo(V3::new().x($r).y($g).z($b))) as Box<dyn Material>
    };
    (Lambertian albedo( $r:expr, $g:expr, $b:expr )) => {
        Box::new(Lambertian::new().albedo(V3::new().x($r).y($g).z($b))) as Box<dyn Material>
    };
    (Dielectric ir( $ir:expr )) => {
        Box::new(Dielectric::new().ir($ir)) as Box<dyn Material>
    };
    (sphere $r:expr,  ($x:expr, $y:expr, $z:expr), $material:ident) => {
        Box::new(Sphere::new(P3::new().x($x).y($y).z($z), $r, &*$material))
    };
}

fn main() {
    let material_ground = make!(Lambertian albedo(0.8, 0.8, 0.0));
    let material_center = make!(Lambertian albedo(0.1, 0.2, 0.5));
    let material_left = make!(Dielectric ir(1.5));
    let material_right = make!(Metal albedo(0.8, 0.6, 0.2) fuzz(0.0));

    let mut world = HitList::new();
    world.add(make!(sphere 100.0, (0.0, -100.5, -1.0), material_ground));
    world.add(make!(sphere 0.5, (0.0, 0.0, -1.0), material_center));
    world.add(make!(sphere 0.5, (-1.0, 0.0, -1.0), material_left));
    world.add(make!(sphere - 0.4, (-1.0, 0.0, -1.0), material_left));
    world.add(make!(sphere 0.5, (1.0, 0.0, -1.0), material_right));

    let config = Config::new()
        .image_width(800)
        .aspect_ratio(16.0 / 9.0)
        .vfov(20.0)
        .lookfrom(P3::new().x(-2.0).y(2.0).z(1.0))
        .lookat(P3::new().z(-1.0))
        .vup(V3::new().y(1.0))
        .defocus_angle(1.0)
        .focus_dist(3.4)
        .samples_per_pixel(500)
        .max_depth(50);
    config.camera().render(&world);
}
