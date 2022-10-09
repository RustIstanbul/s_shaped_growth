//! Demonstrates rotating entities in 2D using quaternions.

use bevy::{math::Vec3Swizzles, prelude::*, sprite::MaterialMesh2dBundle, time::FixedTimestep};

use rand::prelude::random;
use std::time::Duration;

use bevy_sepax2d::prelude::*;
use sepax2d::prelude::*;

const TIME_STEP: f32 = 1.0 / 60.0;
const BOUNDS: Vec2 = Vec2::new(1200.0, 640.0);

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_startup_system(setup)
        .add_system_set(
            SystemSet::new()
                .with_run_criteria(FixedTimestep::step(TIME_STEP as f64))
                .with_system(spawn_grasses)
                .with_system(eating_system)
                .with_system(hopper_movement_system),
        )
        .add_system(bevy::window::close_on_esc)
        .run();
}

#[derive(Component)]
struct Grass {}

#[derive(Component)]
struct Hopper {}

struct GrassSpawnConfig {
    /// How often to spawn a new grass? (repeating timer)
    timer: Timer,
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
    // 2D orthographic camera
    commands.spawn_bundle(Camera2dBundle::default());

    commands.insert_resource(GrassSpawnConfig {
        // create the repeating timer
        timer: Timer::new(Duration::from_millis(100), true),
    });

    let circle1 = Circle::new((0.0, 0.0), 10.0);

    commands
        .spawn_bundle(MaterialMesh2dBundle {
            mesh: meshes.add(shape::Circle::new(10.).into()).into(),
            material: materials.add(ColorMaterial::from(Color::RED)),
            transform: Transform::from_translation(Vec3::new(
                random::<f32>() * BOUNDS.x - BOUNDS.x / 2.,
                random::<f32>() * BOUNDS.y - BOUNDS.y / 2.0,
                0.,
            ))
            .with_rotation(Quat::from_rotation_z((20.0_f32).to_radians())),
            ..default()
        })
        .insert(Hopper {})
        .insert(Sepax {
            convex: bevy_sepax2d::Convex::Circle(circle1),
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

fn eating_system(
    mut commands: Commands,
    mut hoppers: Query<(&Hopper, &Sepax)>,
    targets: Query<(Entity, &Grass, &Sepax)>,
) {
    for (_h, hs) in hoppers.iter() {
        for (ge, g, gs) in targets.iter() {
            if sat_overlap(hs.shape(), gs.shape()) {
                commands.entity(ge).despawn();
            }
        }
    }
}

fn hopper_movement_system(mut query: Query<(&Hopper, &mut Transform, &mut Sepax)>) {
    for (_h, mut t, mut sepax) in query.iter_mut() {
        if t.translation.x < -BOUNDS.x / 2. {
            t.rotation = Quat::from_rotation_z(0.);
        }

        if t.translation.x > BOUNDS.x / 2. {
            t.rotation = Quat::from_rotation_z(180.0_f32.to_radians());
        }

        if t.translation.y > BOUNDS.y / 2. {
            t.rotation = Quat::from_rotation_z(270.0_f32.to_radians());
        }

        if t.translation.y < -BOUNDS.y / 2. {
            t.rotation = Quat::from_rotation_z(90.0_f32.to_radians());
        }

        let movement_distance = 600. * TIME_STEP;
        let translation_delta = t.rotation * Vec3::X * movement_distance;
        t.translation += translation_delta;
        sepax
            .shape_mut()
            .set_position((t.translation.x, t.translation.y))
    }
}
