use std::{cmp::Ordering, panic};

use crate::{
    aabb::Aabb,
    hittable::{HitRecord, Hittable},
    ray::Ray,
    util::random_int,
};

#[derive(Debug)]
pub(crate) struct BvhNode {
    left: Box<dyn Hittable>,
    right_maybe: Option<Box<dyn Hittable>>,
    abox: Aabb,
}

impl BvhNode {
    pub(crate) fn new(mut objects: Vec<Box<dyn Hittable>>, time0: f64, time1: f64) -> BvhNode {
        let size = objects.len();

        let axis = random_int(0, 2);
        let comparator = match axis {
            0 => Self::box_compare_x,
            1 => Self::box_compare_y,
            _ => Self::box_compare_z,
        };

        let left;
        let right;
        assert!(objects.len() != 0);
        if objects.len() == 1 {
            left = objects.pop().unwrap();
            right = None;
        } else if objects.len() == 2 {
            let second = objects.pop().unwrap();
            let first = objects.pop().unwrap();

            if comparator(&first, &second) == Ordering::Less {
                left = first;
                right = Some(second);
            } else {
                left = second;
                right = Some(first);
            }
        } else {
            objects.sort_by(|a, b| comparator(a, b));

            let mid = objects.len() / 2;
            let half2 = objects.split_off(mid);

            left = Box::new(BvhNode::new(objects, time0, time1));
            right = Some(Box::new(BvhNode::new(half2, time0, time1)));
        }

        let abox = if let Some(ref right) = right {
            Aabb::surrounding_box(
                left.bounding_box(time0, time1).unwrap(),
                right.bounding_box(time0, time1).unwrap(),
            )
        } else {
            left.bounding_box(time0, time1).unwrap()
        };

        BvhNode {
            left,
            right_maybe: right,
            abox,
        }
    }

    fn box_compare_x(a: &Box<dyn Hittable>, b: &Box<dyn Hittable>) -> Ordering {
        Self::box_compare(a, b, 0)
    }
    fn box_compare_y(a: &Box<dyn Hittable>, b: &Box<dyn Hittable>) -> Ordering {
        Self::box_compare(a, b, 1)
    }
    fn box_compare_z(a: &Box<dyn Hittable>, b: &Box<dyn Hittable>) -> Ordering {
        Self::box_compare(a, b, 2)
    }

    fn box_compare(a: &Box<dyn Hittable>, b: &Box<dyn Hittable>, axis: usize) -> Ordering {
        let box_a = a
            .bounding_box(0.0, 0.0)
            .expect("A must have a bounding box");
        let box_b = b
            .bounding_box(0.0, 0.0)
            .expect("B must have a bounding box");

        box_a.min()[axis].partial_cmp(&box_b.min()[axis]).unwrap()
    }
}

impl Hittable for BvhNode {
    fn hit(&self, r: Ray, t_min: f64, t_max: f64) -> Option<HitRecord> {
        if !self.abox.hit(r, t_min, t_max) {
            return None;
        }

        let hit_left = self.left.hit(r, t_min, t_max);

        let hit_right = self.right_maybe.as_ref().and_then(|right| {
            let hit_left_time = hit_left.as_ref().map(|h| h.t).unwrap_or(t_max);

            right.hit(r, t_min, hit_left_time)
        });

        match (hit_left, hit_right) {
            (None, right) => right,
            (left, None) => left,
            (Some(left), Some(right)) => {
                if left.t < right.t {
                    Some(left)
                } else {
                    Some(right)
                }
            }
        }
    }

    fn bounding_box(&self, _time0: f64, _time1: f64) -> Option<Aabb> {
        Some(self.abox)
    }
}
