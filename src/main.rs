use std::rc::Rc;

use camera::Config;
use material::{Lambertian, Metal};
use vec3::Vec3;

use crate::{hit_list::HitList, sphere::Sphere};

mod camera;
mod hit;
mod hit_list;
mod interval;
mod material;
mod ray;
mod sphere;
mod util;
mod vec3;

macro_rules! make {
    (material <$kind:ty>::albedo( $r:expr, $g:expr, $b:expr )) => {
        Rc::new(<$kind>::new().albedo(Vec3::new().x($r).y($g).z($b)))
    };
    (sphere $r:expr,  ($x:expr, $y:expr, $z:expr), $material:ident) => {
        Rc::new(Sphere::new(Vec3::new().x($x).y($y).z($z), $r, $material))
    };
}

fn main() {
    let material_ground = make!(material <Lambertian>::albedo(0.8, 0.8, 0.0));
    let material_center = make!(material <Lambertian>::albedo(0.7, 0.3, 0.3));
    let material_left = make!(material <Metal>::albedo(0.8, 0.8, 0.8));
    let material_right = make!(material <Metal>::albedo(0.8, 0.6, 0.2));

    let mut world = HitList::new();
    world.add(make!(sphere 100.0, (0.0, -100.5, -1.0), material_ground));
    world.add(make!(sphere 0.5, (0.0, 0.0, -1.0), material_center));
    world.add(make!(sphere 0.5, (-1.0, 0.0, -1.0), material_left));
    world.add(make!(sphere 0.5, (1.0, 0.0, -1.0), material_right));

    Config::new()
        .image_width(800)
        .max_depth(50)
        .samples_per_pixel(20)
        .camera()
        .render(&world);
}
