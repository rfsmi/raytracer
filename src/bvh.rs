use crate::{
    aabb::AABB,
    hit::{Hit, HitList, HitRecord},
    ray::{Interval, Ray},
    vector::{P3, V3},
};

#[derive(Clone, Copy)]
pub enum Axis {
    X,
    Y,
    Z,
}

impl Axis {
    const ALL: &'static [Axis] = &[Axis::X, Axis::Y, Axis::Z];

    fn v3(self, v: V3) -> f64 {
        match self {
            Axis::X => v.x,
            Axis::Y => v.y,
            Axis::Z => v.z,
        }
    }

    fn p3(self, p: P3) -> f64 {
        match self {
            Axis::X => p.x,
            Axis::Y => p.y,
            Axis::Z => p.z,
        }
    }
}

const PRIMITIVE_TEST_COST: f64 = 1.0;
const BVH_SPLIT_COST: f64 = 1.0 / 8.0;

pub enum BVH {
    Internal(AABB, Axis, Box<BVH>, Box<BVH>),
    Leaf(AABB, Box<dyn Hit>),
}

impl BVH {
    pub fn new(objects: impl IntoIterator<Item = Box<dyn Hit>>) -> Self {
        let mut primitives: Vec<_> = objects.into_iter().collect();
        let bounds = AABB::union(primitives.iter().map(|o| o.aabb()));

        if let Some((axis, split_i, cost)) = object_split::find_best(bounds, &primitives) {
            // Split if we must, or the cost is low enough
            if cost < PRIMITIVE_TEST_COST * primitives.len() as f64 {
                primitives.sort_by(|a, b| {
                    axis.p3(a.aabb().centroid().unwrap())
                        .total_cmp(&axis.p3(b.aabb().centroid().unwrap()))
                });
                let rest = primitives.split_off(split_i);
                return Self::internal(bounds, axis, primitives, rest);
            }
        };

        // Otherwise this is just a leaf node
        Self::leaf(bounds, primitives)
    }

    fn leaf(aabb: AABB, primitives: Vec<Box<dyn Hit>>) -> Self {
        Self::Leaf(aabb, Box::new(HitList::new(primitives)))
    }

    fn internal(aabb: AABB, axis: Axis, a: Vec<Box<dyn Hit>>, b: Vec<Box<dyn Hit>>) -> Self {
        Self::Internal(aabb, axis, Box::new(Self::new(a)), Box::new(Self::new(b)))
    }
}

impl Hit for BVH {
    fn aabb(&self) -> &AABB {
        match self {
            BVH::Internal(aabb, ..) => aabb,
            BVH::Leaf(aabb, ..) => aabb,
        }
    }

    fn hit<'a>(&'a self, r: &Ray, mut ray_t: Interval) -> Option<HitRecord<'a>> {
        let mut stack = vec![self];
        let mut best_hr = None;
        while let Some(bvh) = stack.pop() {
            if bvh.aabb().ray_intersection(r, ray_t).is_none() {
                continue;
            }

            match bvh {
                BVH::Internal(.., axis, a, b) => {
                    let next = if axis.v3(r.direction).is_sign_negative() {
                        [&**a, &**b]
                    } else {
                        [&**b, &**a]
                    };
                    stack.extend(next);
                }
                BVH::Leaf(.., primitive) => {
                    if let Some(hr) = primitive.hit(r, ray_t) {
                        ray_t.max = hr.t;
                        best_hr = Some(hr);
                    }
                }
            }
        }
        best_hr
    }
}

mod object_split {
    use super::*;

    const N_BUCKETS: usize = 12;

    #[derive(Clone, Copy)]
    struct Bucket {
        count: usize,
        aabb: AABB,
    }

    impl Bucket {
        fn new() -> Self {
            Bucket {
                count: 0,
                aabb: AABB::new(),
            }
        }

        fn add(&mut self, aabb: &AABB) {
            self.count += 1;
            self.aabb.extend(aabb);
        }
    }

    fn combine_buckets<'a>(buckets: impl IntoIterator<Item = &'a Bucket>) -> Bucket {
        buckets.into_iter().fold(Bucket::new(), |a, b| Bucket {
            count: a.count + b.count,
            aabb: AABB::union([&a.aabb, &b.aabb]),
        })
    }

    pub fn find_best(bounds: AABB, primitives: &[Box<dyn Hit>]) -> Option<(Axis, usize, f64)> {
        let centroids = primitives.iter().filter_map(|o| o.aabb().centroid());
        let centroid_bounds = AABB::bounding_box(centroids);
        Axis::ALL
            .iter()
            // All centroids are at the same point
            .filter(|axis| axis.v3(centroid_bounds.size()) > 0.0)
            .flat_map(move |&axis| {
                // Assign each primitive to its bucket
                let mut buckets = [Bucket::new(); N_BUCKETS];
                let bounds_min = axis.p3(centroid_bounds.min().unwrap());
                let bounds_max = axis.p3(centroid_bounds.max().unwrap());
                for primitive in primitives {
                    let centroid = axis.p3(primitive.aabb().centroid().unwrap());
                    let f = (centroid - bounds_min) / (bounds_max - bounds_min);
                    let mut i = (N_BUCKETS as f64 * f) as usize;
                    if i == N_BUCKETS {
                        i -= 1;
                    }
                    buckets[i].add(primitive.aabb());
                }
                // Split at each point
                (1..N_BUCKETS).map(move |i| {
                    let (part_a, part_b) = buckets.split_at(i);
                    let a = combine_buckets(part_a);
                    let b = combine_buckets(part_b);
                    (axis, a, b)
                })
            })
            .map(|(axis, a, b)| {
                // Determine the cost
                let cost = BVH_SPLIT_COST
                    + PRIMITIVE_TEST_COST
                        * (a.count as f64 * a.aabb.surface_area()
                            + b.count as f64 * b.aabb.surface_area())
                        / bounds.surface_area();
                (axis, a.count, cost)
            })
            .min_by(|(_, _, a), (_, _, b)| a.total_cmp(b))
    }
}
