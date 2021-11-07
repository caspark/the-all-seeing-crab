use crate::{
    aabb::Aabb,
    aarect::XyRect,
    hittable::{Hittable, HittableList},
    material::Material,
    vec3::Point3,
};

#[derive(Debug)]
pub(crate) struct Box3D {
    min: Point3,
    max: Point3,
    sides: HittableList,
}

impl Box3D {
    pub(crate) fn new(min: Point3, max: Point3, mat: &dyn Material) -> Self {
        Self {
            min,
            max,
            sides: {
                let mut sides = HittableList::new();

                let b = Box::new(mat);

                // let x = XyRect::new(min.x, max.x, min.y, max.y, max.z, b);
                sides.add(Box::new(XyRect::new(min.x, max.x, min.y, max.y, max.z, b)));
                //todo other sides

                sides
            },
        }
    }
}
