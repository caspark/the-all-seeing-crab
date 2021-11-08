use crate::{
    aarect::{XyRect, XzRect, YzRect},
    box3d::Box3D,
    bvh_node::BvhNode,
    camera::CameraSettings,
    constant_medium::ConstantMedium,
    hittable::{Hittable, RotateY, Translate},
    material::{
        Dielectric, DiffuseLambertian, DiffuseLambertianTexture, DiffuseLight, Material, Metal,
    },
    perlin::Perlin,
    sphere::Sphere,
    texture::{
        CheckerTexture, ColorTexture, ImageTexture, MarbleTexture, NoiseTexture, TurbulenceTexture,
    },
    util::random_double,
    vec3::{Color, Point3, Vec3},
    World,
};

#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash, serde::Serialize, serde::Deserialize)]
pub(crate) enum RenderScene {
    ThreeBody,
    ManyBalls,
    CheckersColliding,
    PerlinNoise,
    EarthGlobe,
    LightDemo,
    CornelBox,
    CornelSmokeBox,
}

impl RenderScene {
    pub(crate) fn default_camera_settings(&self) -> CameraSettings {
        match self {
            RenderScene::ThreeBody => {
                let look_from = Point3::new(3.0, 3.0, 2.0);
                let look_at = Point3::new(0.0, 0.0, -1.0);
                CameraSettings {
                    look_from,
                    look_at,
                    vup: Vec3::new(0.0, 1.0, 0.0),
                    vfov: 20.0,
                    focus_dist: (look_from - look_at).length(),
                    aperture: 0.25,
                    time0: 0.0,
                    time1: 0.0,
                }
            }
            RenderScene::ManyBalls => CameraSettings {
                look_from: Point3::new(13.0, 2.0, 3.0),
                look_at: Point3::new(0.0, 0.0, 0.0),
                vup: Vec3::new(0.0, 1.0, 0.0),
                vfov: 20.0,
                focus_dist: 10.0,
                aperture: 0.1,
                time0: 0.0,
                time1: 1.0,
            },
            RenderScene::CheckersColliding => CameraSettings::default(),
            RenderScene::PerlinNoise => CameraSettings::default(),
            RenderScene::EarthGlobe => CameraSettings::default(),
            RenderScene::LightDemo => CameraSettings::default()
                .look_from(Point3::new(26.0, 3.0, 6.0))
                .look_at(Point3::new(0.0, 2.0, 0.0)),
            RenderScene::CornelBox | RenderScene::CornelSmokeBox => CameraSettings::default()
                .look_from(Point3::new(278.0, 278.0, -800.0))
                .look_at(Point3::new(278.0, 278.0, 0.0))
                .vfov(40.0),
        }
    }

    pub(crate) fn create_world(&self) -> World {
        match self {
            RenderScene::ThreeBody => {
                let mut world: Vec<Box<dyn Hittable>> = Vec::new();

                let material_ground = Box::new(DiffuseLambertianTexture::new(Box::new(
                    CheckerTexture::from_colors(
                        10.0,
                        Color::new(0.2, 0.3, 0.1),
                        Color::new(0.9, 0.9, 0.9),
                    ),
                )));
                let material_center = Box::new(DiffuseLambertianTexture::new(Box::new(
                    ColorTexture::from_rgb(0.1, 0.2, 0.5),
                )));
                let material_left = Box::new(Dielectric::new(1.5));
                let material_right = Box::new(Metal::new(Color::new(0.8, 0.6, 0.2), 0.0));

                world.push(Box::new(Sphere::stationary(
                    Point3::new(0.0, -100.5, -1.0),
                    100.0,
                    material_ground,
                )));
                world.push(Box::new(Sphere::stationary(
                    Point3::new(0.0, 0.0, -1.0),
                    0.5,
                    material_center,
                )));
                world.push(Box::new(Sphere::stationary(
                    Point3::new(-1.0, 0.0, -1.0),
                    0.5,
                    material_left.clone(),
                )));
                world.push(Box::new(Sphere::stationary(
                    Point3::new(-1.0, 0.0, -1.0),
                    -0.45,
                    material_left,
                )));
                world.push(Box::new(Sphere::stationary(
                    Point3::new(1.0, 0.0, -1.0),
                    0.5,
                    material_right,
                )));

                BvhNode::new(world, 0.0, 0.0).into()
            }
            RenderScene::ManyBalls => {
                let mut world: Vec<Box<dyn Hittable>> = Vec::new();

                let material_ground = Box::new(DiffuseLambertian::new(Color::new(0.8, 0.8, 0.0)));
                world.push(Box::new(Sphere::stationary(
                    Point3::new(0.0, -1000.0, 0.0),
                    1000.0,
                    material_ground,
                )));

                for a in -11..11 {
                    for b in -11..11 {
                        let a = a as f64;
                        let b = b as f64;

                        let choose_mat = random_double(0.0, 1.0);
                        let center = Point3::new(
                            a + 0.9 * random_double(0.0, 1.0),
                            0.2,
                            b + 0.9 * random_double(0.0, 1.0),
                        );

                        if (center - Point3::new(4.0, 0.2, 0.0)).length() > 0.9 {
                            let material: Box<dyn Material + Send + Sync> = if choose_mat < 0.8 {
                                let albedo = Color::random(0.0, 1.0) * Color::random(0.0, 1.0);
                                Box::new(DiffuseLambertian::new(albedo))
                            } else if choose_mat < 0.95 {
                                let albedo = Color::random(0.5, 1.0);
                                let fuzz = random_double(0.0, 0.5);
                                Box::new(Metal::new(albedo, fuzz))
                            } else {
                                Box::new(Dielectric::new(1.5)) // 1.5 is glass
                            };
                            world.push(if choose_mat < 0.4 {
                                Box::new(Sphere::stationary(center, 0.2, material))
                            } else {
                                let center2 = center + Vec3::new(0.0, random_double(0.0, 0.5), 0.0);
                                Box::new(Sphere::moving(center, center2, 0.0, 1.0, 0.2, material))
                            });
                        }
                    }
                }

                world.push(Box::new(Sphere::stationary(
                    Point3::new(0.0, 1.0, 0.0),
                    1.0,
                    Box::new(Dielectric::new(1.5)),
                )));

                world.push(Box::new(Sphere::stationary(
                    Point3::new(-4.0, 1.0, 0.0),
                    1.0,
                    Box::new(DiffuseLambertian::new(Color::new(0.1, 0.2, 0.5))),
                )));
                world.push(Box::new(Sphere::stationary(
                    Point3::new(4.0, 1.0, 0.0),
                    1.0,
                    Box::new(Metal::new(Color::new(0.8, 0.6, 0.2), 0.0)),
                )));

                BvhNode::new(world, 0.0, 1.0).into()
            }
            RenderScene::CheckersColliding => {
                let mut world: Vec<Box<dyn Hittable>> = Vec::new();

                let light = Color::new(0.2, 0.3, 0.1);
                let dark = Color::new(0.9, 0.9, 0.9);

                let material_top = Box::new(DiffuseLambertianTexture::new(Box::new(
                    CheckerTexture::from_colors(10.0, light, dark),
                )));
                let material_bottom = Box::new(DiffuseLambertianTexture::new(Box::new(
                    CheckerTexture::from_colors(10.0, dark, light),
                )));

                world.push(Box::new(Sphere::stationary(
                    Point3::new(0.0, 10.0, 0.0),
                    10.0,
                    material_top,
                )));
                world.push(Box::new(Sphere::stationary(
                    Point3::new(0.0, -10.0, 0.0),
                    10.0,
                    material_bottom,
                )));

                BvhNode::new(world, 0.0, 0.0).into()
            }
            RenderScene::PerlinNoise => {
                let mut world = Vec::new();

                let noise = Perlin::new();

                let material_ground = Box::new(DiffuseLambertianTexture::new(Box::new(
                    NoiseTexture::new(noise.clone(), 4.0),
                )));
                let material_center = Box::new(DiffuseLambertianTexture::new(Box::new(
                    MarbleTexture::new(noise.clone(), 4.0, 5),
                )));
                let material_right = Box::new(DiffuseLambertianTexture::new(Box::new(
                    TurbulenceTexture::new(noise, 4.0, 5),
                )));

                world.push(Box::new(Sphere::stationary(
                    Point3::new(0.0, -1000.0, 0.0),
                    1000.0,
                    material_ground,
                )) as Box<dyn Hittable>);
                world.push(Box::new(Sphere::stationary(
                    Point3::new(0.0, 2.0, 0.0),
                    2.0,
                    material_center,
                )) as Box<dyn Hittable>);
                world.push(Box::new(Sphere::stationary(
                    Point3::new(-1.0, 2.0, -2.0),
                    2.0,
                    material_right,
                )) as Box<dyn Hittable>);

                BvhNode::new(world, 0.0, 0.0).into()
            }
            RenderScene::EarthGlobe => {
                let mut world = Vec::new();

                let material = Box::new(DiffuseLambertianTexture::new(Box::new(
                    ImageTexture::load_from_png("textures/earthmap.png")
                        .expect("expected earthmap.png texture to exist in textures/"),
                )));

                world.push(Box::new(Sphere::stationary(
                    Point3::new(0.0, 0.0, 0.0),
                    2.0,
                    material,
                )) as Box<dyn Hittable>);

                BvhNode::new(world, 0.0, 0.0).into()
            }
            RenderScene::LightDemo => World {
                background: Some(Color::new(0.0, 0.0, 0.0)),
                node: {
                    let mut world = Vec::new();

                    let noise = Perlin::new();

                    // ground
                    world.push(Box::new(Sphere::stationary(
                        Point3::new(0.0, -1000.0, 0.0),
                        1000.0,
                        Box::new(DiffuseLambertianTexture::new(Box::new(MarbleTexture::new(
                            noise.clone(),
                            4.0,
                            7,
                        )))),
                    )) as Box<dyn Hittable>);
                    // floating sphere
                    world.push(Box::new(Sphere::stationary(
                        Point3::new(0.0, 2.0, 0.0),
                        2.0,
                        Box::new(DiffuseLambertianTexture::new(Box::new(MarbleTexture::new(
                            noise, 4.0, 7,
                        )))),
                    )) as Box<dyn Hittable>);

                    // light
                    world.push(Box::new(XyRect::new(
                        3.0,
                        5.0,
                        1.0,
                        3.0,
                        -2.0,
                        Box::new(DiffuseLight::new(Box::new(ColorTexture::from_rgb(
                            4.0, 4.0, 4.0,
                        )))),
                    )) as Box<dyn Hittable>);

                    BvhNode::new(world, 0.0, 0.0)
                },
            },
            RenderScene::CornelBox => World {
                background: Some(Color::new(0.0, 0.0, 0.0)),
                node: {
                    let mut world: Vec<Box<dyn Hittable>> = cornell_box_walls();

                    // light
                    world.push(Box::new(XzRect::new(
                        213.0,
                        343.0,
                        227.0,
                        332.0,
                        554.0,
                        Box::new(DiffuseLight::new(Box::new(ColorTexture::from_rgb(
                            15.0, 15.0, 15.0,
                        )))),
                    )));

                    let white = Box::new(DiffuseLambertianTexture::new(Box::new(
                        ColorTexture::from_rgb(0.73, 0.73, 0.73),
                    )));
                    world.push(Box::new(Translate::new(
                        Point3::new(265.0, 0.0, 295.0),
                        RotateY::new(
                            15.0,
                            Box3D::new(
                                Point3::new(0.0, 0.0, 0.0),
                                Point3::new(165.0, 330.0, 165.0),
                                white.clone(),
                            ),
                        ),
                    )));
                    world.push(Box::new(RotateY::new(
                        -18.0,
                        Translate::new(
                            Vec3::new(130.0, 0.0, 65.0),
                            Box3D::new(
                                Point3::new(0.0, 0.0, 0.0),
                                Point3::new(165.0, 165.0, 165.0),
                                white,
                            ),
                        ),
                    )));

                    BvhNode::new(world, 0.0, 0.0)
                },
            },
            RenderScene::CornelSmokeBox => World {
                background: Some(Color::new(0.0, 0.0, 0.0)),
                node: {
                    let mut world: Vec<Box<dyn Hittable>> = cornell_box_walls();

                    // light (4x larger but half as bright as regular Cornell box)
                    world.push(Box::new(XzRect::new(
                        113.0,
                        443.0,
                        127.0,
                        432.0,
                        554.0,
                        Box::new(DiffuseLight::new(Box::new(ColorTexture::from_rgb(
                            7.0, 7.0, 7.0,
                        )))),
                    )));

                    let white = Box::new(DiffuseLambertianTexture::new(Box::new(
                        ColorTexture::from_rgb(0.73, 0.73, 0.73),
                    )));
                    world.push(Box::new(Translate::new(
                        Point3::new(265.0, 0.0, 295.0),
                        RotateY::new(
                            15.0,
                            ConstantMedium::new(
                                Box::new(Box3D::new(
                                    Point3::new(0.0, 0.0, 0.0),
                                    Point3::new(165.0, 330.0, 165.0),
                                    white.clone(),
                                )),
                                Box::new(ColorTexture::new(Color::new(0.0, 0.0, 0.0))),
                                0.01,
                            ),
                        ),
                    )));
                    world.push(Box::new(RotateY::new(
                        -18.0,
                        Translate::new(
                            Vec3::new(130.0, 0.0, 65.0),
                            ConstantMedium::new(
                                Box::new(Box3D::new(
                                    Point3::new(0.0, 0.0, 0.0),
                                    Point3::new(165.0, 165.0, 165.0),
                                    white,
                                )),
                                Box::new(ColorTexture::new(Color::new(1.0, 1.0, 1.0))),
                                0.01,
                            ),
                        ),
                    )));

                    BvhNode::new(world, 0.0, 0.0)
                },
            },
        }
    }
}

impl Default for RenderScene {
    fn default() -> Self {
        RenderScene::ThreeBody
    }
}

#[must_use]
fn cornell_box_walls() -> Vec<Box<dyn Hittable>> {
    let red = Box::new(DiffuseLambertianTexture::new(Box::new(
        ColorTexture::from_rgb(0.65, 0.05, 0.05),
    )));
    let white = Box::new(DiffuseLambertianTexture::new(Box::new(
        ColorTexture::from_rgb(0.73, 0.73, 0.73),
    )));
    let green = Box::new(DiffuseLambertianTexture::new(Box::new(
        ColorTexture::from_rgb(0.12, 0.45, 0.15),
    )));

    vec![
        // left side
        Box::new(YzRect::new(0.0, 555.0, 0.0, 555.0, 555.0, green)),
        // right side
        Box::new(YzRect::new(0.0, 555.0, 0.0, 555.0, 0.0, red)),
        Box::new(XzRect::new(0.0, 555.0, 0.0, 555.0, 0.0, white.clone())),
        Box::new(XzRect::new(0.0, 555.0, 0.0, 555.0, 555.0, white.clone())),
        Box::new(XyRect::new(0.0, 555.0, 0.0, 555.0, 555.0, white)),
    ]
}
