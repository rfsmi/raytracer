use std::{cell::RefCell, rc::Rc};

use camera::Config;
use hit::Hit;
use vec3::Vec3;

use crate::{hit_list::HitList, sphere::Sphere};

mod camera;
mod hit;
mod hit_list;
mod interval;
mod ray;
mod sphere;
mod util;
mod vec3;

fn shared<T: Hit + 'static>(object: T) -> Rc<RefCell<dyn Hit>> {
    Rc::new(RefCell::new(object))
}

fn main() {
    let mut world = HitList::new();
    world.add(shared(Sphere::new(Vec3::new().z(-1.0), 0.5)));
    world.add(shared(Sphere::new(Vec3::new().y(-100.5).z(-1.0), 100.0)));

    Config::new()
        .image_width(800)
        .max_depth(50)
        .samples_per_pixel(20)
        .camera()
        .render(&world);
}
