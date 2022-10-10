//cargo build --release --target wasm32-unknown-unknown
// wasm-bindgen --out-dir ./out/ --target web ./target/wasm32-unknown-unknown/release/s_shaped_growth.wasm
use bevy::{
    math::Vec3Swizzles,
    prelude::*,
    sprite::{MaterialMesh2dBundle, Mesh2dHandle},
    time::FixedTimestep,
};

use rand::prelude::random;
use std::{f32::consts::PI, time::Duration};

use bevy_sepax2d::prelude::*;
use sepax2d::prelude::*;

const TIME_STEP: f32 = 1.0 / 60.0;
const BOUNDS: Vec2 = Vec2::new(1200.0, 640.0);

const SPECIAL_THANKS: &str = "Friends";

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_startup_system(setup)
        .add_system_set(
            SystemSet::new()
                .with_run_criteria(FixedTimestep::step(TIME_STEP as f64))
                .with_system(spawn_grasses)
                .with_system(count_hoppers)
                .with_system(eating_system)
                .with_system(hopper_movement_system),
        )
        .add_system(bevy::window::close_on_esc)
        .run();
}

#[derive(Component)]
struct Grass {}

#[derive(Component)]
struct Hopper {
    belly: f32,
}

#[derive(Component)]
struct HopperCount {
    count: usize,
    step: usize,
}

struct GrassSpawnConfig {
    /// How often to spawn a new grass? (repeating timer)
    timer: Timer,
}

struct HopperCountConfig {
    timer: Timer,
    step: usize,
}

/// Add the game's entities to our world and creates an orthographic camera for 2D rendering.
///
/// The Bevy coordinate system is the same for 2D and 3D, in terms of 2D this means that:
///
/// * `X` axis goes from left to right (`+X` points right)
/// * `Y` axis goes from bottom to top (`+Y` point up)
/// * `Z` axis goes from far to near (`+Z` points towards you, out of the screen)
///
/// The origin is at the center of the screen.
fn setup(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    let handle: Handle<Font> = asset_server.load("fonts/NotoSansMono-Regular.ttf");

    // 2D orthographic camera
    commands.spawn_bundle(Camera2dBundle::default());

    commands.insert_resource(GrassSpawnConfig {
        // create the repeating timer
        timer: Timer::new(Duration::from_millis(50), true),
    });

    commands.insert_resource(HopperCountConfig {
        timer: Timer::new(Duration::from_secs(1), true),
        step: 0,
    });

    let circle1 = Circle::new((0.0, 0.0), 10.0);
    for count in 0..1 {
        let random_radian = (360.0_f32 * random::<f32>()).to_radians();
        commands
            .spawn_bundle(MaterialMesh2dBundle {
                mesh: meshes.add(shape::Circle::new(10.).into()).into(),
                material: materials.add(ColorMaterial::from(Color::RED)),
                transform: Transform::from_translation(Vec3::new(
                    random::<f32>() * BOUNDS.x - BOUNDS.x / 2.,
                    random::<f32>() * BOUNDS.y - BOUNDS.y / 2.0,
                    0.,
                ))
                .with_rotation(Quat::from_rotation_z(random_radian)),
                ..default()
            })
            .insert(Hopper { belly: 80. })
            .insert(Sepax {
                convex: bevy_sepax2d::Convex::Circle(circle1),
            });
    }

    let font = asset_server.load("fonts/NotoSansMono-Regular.ttf");
    let text_style = TextStyle {
        font: font,
        font_size: 20.0,
        color: Color::WHITE,
    };
    commands.spawn_bundle(Text2dBundle {
        text: Text::from_section("Development: Fatih Pense", text_style.clone())
            .with_alignment(TextAlignment::CENTER),
        transform: Transform::from_translation(Vec3::new(
            -BOUNDS.x / 2. + 120.,
            BOUNDS.y / 2. - 5.0,
            1.,
        )),
        ..default()
    });

    commands.spawn_bundle(Text2dBundle {
        text: Text::from_section(
            "Special Thanks: ".to_owned() + SPECIAL_THANKS,
            text_style.clone(),
        )
        .with_alignment(TextAlignment::CENTER),
        transform: Transform::from_translation(Vec3::new(0., BOUNDS.y / 2. - 5.0, 1.)),
        ..default()
    });
}

fn spawn_grasses(
    mut commands: Commands,
    time: Res<Time>,
    mut config: ResMut<GrassSpawnConfig>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    // tick the timer
    config.timer.tick(time.delta());

    if config.timer.finished() {
        // commands.spawn().insert(Grass {}).with_children(|parent| {
        //     parent.spawn_bundle(MaterialMesh2dBundle {
        //         mesh: meshes.add(shape::Circle::new(5.).into()).into(),
        //         material: materials.add(ColorMaterial::from(Color::GREEN)),
        //         transform: Transform::from_translation(Vec3::new(0., 0., 0.)),
        //         ..default()
        //     });
        // });

        let radius = 5.;
        let new_x = random::<f32>() * BOUNDS.x - BOUNDS.x / 2.;
        let new_y = random::<f32>() * BOUNDS.y - BOUNDS.y / 2.;
        let circle1 = Circle::new((new_x, new_y), radius);

        commands
            .spawn_bundle(MaterialMesh2dBundle {
                mesh: meshes.add(shape::Circle::new(radius).into()).into(),
                material: materials.add(ColorMaterial::from(Color::GREEN)),
                transform: Transform::from_translation(Vec3::new(new_x, new_y, 0.)),
                ..default()
            })
            .insert(Grass {})
            .insert(Sepax {
                convex: bevy_sepax2d::Convex::Circle(circle1),
            });
    }
}

fn count_hoppers(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    asset_server: Res<AssetServer>,
    time: Res<Time>,
    mut config: ResMut<HopperCountConfig>,
    hoppers: Query<(&Hopper,)>,
) {
    // tick the timer
    config.timer.tick(time.delta());

    if config.timer.finished() {
        let font = asset_server.load("fonts/NotoSansMono-Regular.ttf");

        let text_style = TextStyle {
            font: font,
            font_size: 15.0,
            color: Color::WHITE,
        };

        config.step += 1;
        let count = hoppers.iter().count();
        commands
            .spawn_bundle(MaterialMesh2dBundle {
                mesh: meshes
                    .add(
                        shape::Quad::new(Vec2 {
                            x: 10.,
                            y: count as f32,
                        })
                        .into(),
                    )
                    .into(),
                material: materials.add(ColorMaterial::from(Color::RED)),
                transform: Transform::from_translation(Vec3::new(
                    config.step as f32 * 20. - BOUNDS.x / 2.,
                    (count as f32) / 2. - BOUNDS.y / 2.0,
                    1.,
                )),
                ..default()
            })
            .insert(HopperCount {
                count: count,
                step: config.step,
            })
            .with_children(|parent| {
                parent.spawn_bundle(Text2dBundle {
                    text: Text::from_section(count.to_string(), text_style.clone())
                        .with_alignment(TextAlignment::CENTER),
                    transform: Transform::from_translation(Vec3::new(
                        0.,
                        -(count as f32) / 2. - 5.0,
                        0.,
                    ))
                    .with_rotation(Quat::from_rotation_z(PI / 4.0)),

                    ..default()
                });
            });
    }
}

fn eating_system(
    mut commands: Commands,
    mut hoppers: Query<(
        &mut Hopper,
        &Sepax,
        &Handle<ColorMaterial>,
        Entity,
        &Transform,
    )>,
    targets: Query<(Entity, &Grass, &Sepax)>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    mut meshes: ResMut<Assets<Mesh>>,
) {
    for (mut h, hs, colorh, hopper_entity, hopper_transform) in hoppers.iter_mut() {
        for (ge, g, gs) in targets.iter() {
            if sat_overlap(hs.shape(), gs.shape()) {
                commands.entity(ge).despawn();
                h.belly += 40.;
            }
        }

        h.belly -= 0.2;

        if h.belly < 0. {
            commands.entity(hopper_entity).despawn();
            continue;
        }

        let pinkness = 1. - h.belly.min(100.) / 100.;
        let some_color = &mut materials.get_mut(colorh).unwrap().color;
        some_color.set_g(pinkness);
        some_color.set_b(pinkness);

        if h.belly > 100. {
            for count in 0..2 {
                let random_radian = (360.0_f32 * random::<f32>()).to_radians();
                let circle1 = Circle::new((0.0, 0.0), 10.0);
                commands
                    .spawn_bundle(MaterialMesh2dBundle {
                        mesh: meshes.add(shape::Circle::new(10.).into()).into(),
                        material: materials.add(ColorMaterial::from(Color::RED)),
                        transform: Transform::from_translation(Vec3::new(
                            hopper_transform.translation.x,
                            hopper_transform.translation.y,
                            0.,
                        ))
                        .with_rotation(Quat::from_rotation_z(random_radian)),
                        ..default()
                    })
                    .insert(Hopper { belly: 30. })
                    .insert(Sepax {
                        convex: bevy_sepax2d::Convex::Circle(circle1),
                    });
            }
            h.belly -= 60.0;
        }
    }
}

fn hopper_movement_system(mut query: Query<(&Hopper, &mut Transform, &mut Sepax)>) {
    for (_h, mut t, mut sepax) in query.iter_mut() {
        if t.translation.x < -BOUNDS.x / 2. {
            // t.rotation = Quat::from_rotation_z(0.);

            // let r = PI - t.rotation.z;
            // t.rotation = Quat::from_rotation_z(r);

            let random_radian = (90.0 - 90.0_f32 * random::<f32>()).to_radians();
            t.rotation = Quat::from_rotation_z(0. + random_radian);
        }

        if t.translation.x > BOUNDS.x / 2. {
            // t.rotation = Quat::from_rotation_z(180.0_f32.to_radians());

            // let r = PI - t.rotation.z;
            // t.rotation = Quat::from_rotation_z(r);

            let random_radian = (90.0 - 90.0_f32 * random::<f32>()).to_radians();
            t.rotation = Quat::from_rotation_z(PI + random_radian);
        }

        if t.translation.y > BOUNDS.y / 2. {
            // t.rotation = Quat::from_rotation_z(270.0_f32.to_radians());

            // let r = -t.rotation.z;
            // t.rotation = Quat::from_rotation_z(r);

            let random_radian = (90.0 - 90.0_f32 * random::<f32>()).to_radians();
            t.rotation = Quat::from_rotation_z(3.0 * PI / 2. + random_radian);
        }

        if t.translation.y < -BOUNDS.y / 2. {
            t.rotation = Quat::from_rotation_z(90.0_f32.to_radians());

            // let r = -t.rotation.z;
            // t.rotation = Quat::from_rotation_z(r);

            let random_radian = (90.0 - 90.0_f32 * random::<f32>()).to_radians();
            t.rotation = Quat::from_rotation_z(PI / 2. + random_radian);
        }

        let movement_distance = 300. * TIME_STEP;
        let translation_delta = t.rotation * Vec3::X * movement_distance;
        t.translation += translation_delta;
        sepax
            .shape_mut()
            .set_position((t.translation.x, t.translation.y));

        let random_f32 = (120.0_f32 * random::<f32>());
        if random_f32 < 1.0 {
            let random_radian = (360.0_f32 * random::<f32>()).to_radians();
            t.rotation = Quat::from_rotation_z(random_radian);
        }
    }
}
