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
        Rc::new(Metal::new().fuzz($f).albedo(V3::new().x($r).y($g).z($b))) as Rc<dyn Material>
    };
    (Lambertian albedo( $r:expr, $g:expr, $b:expr )) => {
        Rc::new(Lambertian::new().albedo(V3::new().x($r).y($g).z($b))) as Rc<dyn Material>
    };
    (Dielectric ir( $ir:expr )) => {
        Rc::new(Dielectric::new().ir($ir)) as Rc<dyn Material>
    };
    (sphere $r:expr,  ($x:expr, $y:expr, $z:expr), $material:ident) => {
        Rc::new(Sphere::new(
            P3::new().x($x).y($y).z($z),
            $r,
            Rc::clone(&$material),
        ))
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

    Config::new()
        .image_width(800)
        .max_depth(50)
        .samples_per_pixel(100)
        .camera()
        .render(&world);
}
