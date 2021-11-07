use std::sync::Arc;

use crate::{
    aarect::{XyRect, XzRect, YzRect},
    hittable::{Hittable, HittableList},
    material::Material,
    vec3::Point3,
};

#[derive(Debug)]
pub(crate) struct Box3D {
    min: Point3,
    max: Point3,
    sides: HittableList,
    material: Arc<dyn Material>,
}

impl Box3D {
    pub(crate) fn new<M>(min: Point3, max: Point3, material: M) -> Self
    where
        M: 'static + Material,
    {
        let mat = std::sync::Arc::<dyn Material>::from(Box::new(material) as Box<dyn Material>);

        Self {
            min,
            max,
            sides: {
                let mut sides = HittableList::new();

                sides.add(Box::new(XyRect::new(
                    min.x,
                    max.x,
                    min.y,
                    max.y,
                    max.z,
                    Box::new(mat.clone()),
                )));
                sides.add(Box::new(XyRect::new(
                    min.x,
                    max.x,
                    min.y,
                    max.y,
                    min.z,
                    Box::new(mat.clone()),
                )));

                sides.add(Box::new(XzRect::new(
                    min.x,
                    max.x,
                    min.z,
                    max.z,
                    max.y,
                    Box::new(mat.clone()),
                )));
                sides.add(Box::new(XzRect::new(
                    min.x,
                    max.x,
                    min.z,
                    max.z,
                    min.y,
                    Box::new(mat.clone()),
                )));

                sides.add(Box::new(YzRect::new(
                    min.y,
                    max.y,
                    min.z,
                    max.z,
                    max.x,
                    Box::new(mat.clone()),
                )));
                sides.add(Box::new(YzRect::new(
                    min.y,
                    max.y,
                    min.z,
                    max.z,
                    min.x,
                    Box::new(mat.clone()),
                )));

                sides
            },
            material: mat,
        }
    }
}

impl Hittable for Box3D {
    fn hit(
        &self,
        r: crate::ray::Ray,
        t_min: f64,
        t_max: f64,
    ) -> Option<crate::hittable::HitRecord> {
        self.sides.hit(r, t_min, t_max)
    }

    fn bounding_box(&self, _time0: f64, _time1: f64) -> Option<crate::aabb::Aabb> {
        Some(crate::aabb::Aabb::new(self.min, self.max))
    }
}
