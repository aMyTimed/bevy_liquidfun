extern crate bevy;
extern crate bevy_liquidfun;

use std::f32::consts::PI;

use bevy::ecs::system::EntityCommands;
use bevy::prelude::*;

use bevy::window::PresentMode;
use bevy_liquidfun::dynamics::{
    b2BodyBundle, b2Fixture, b2FixtureDef, b2MouseJoint, b2MouseJointDef, CreateMouseJoint,
};
use bevy_liquidfun::plugins::{LiquidFunDebugDrawPlugin, LiquidFunPlugin};
use bevy_liquidfun::utils::DebugDrawFixtures;
use bevy_liquidfun::{
    collision::b2Shape,
    dynamics::{b2BodyDef, b2BodyType::Dynamic, b2World},
};

use bevy_liquidfun::dynamics::b2Body;
use bevy_liquidfun::dynamics::b2BodyType;

use bevy::render::pipelined_rendering::PipelinedRenderingPlugin;

#[derive(Component)]
struct InfoText;

fn main() {
    App::new()
        .add_plugins((
            DefaultPlugins
                .set(WindowPlugin {
                    primary_window: Some(Window {
                        #[cfg(target_arch = "wasm32")]
                        present_mode: PresentMode::default(), // wasm32-unknown-unknown doesn't support PresentMode::Immediate
                        // on everything other than wasm32-unknown-unknown, immediate can be used for less input latency since this is an interactive mouse demo
                        #[cfg(not(target_arch = "wasm32"))]
                        present_mode: PresentMode::Immediate,
                        ..Default::default()
                    }),
                    ..Default::default()
                })
                .build()
                .disable::<PipelinedRenderingPlugin>(), // we want less input latency since this is an interactive mouse demo
            LiquidFunPlugin::default(),
            LiquidFunDebugDrawPlugin,
        ))
        .add_systems(Startup, (setup_camera, setup_instructions))
        .add_systems(
            Startup,
            (
                setup_physics_world,
                setup_physics_bodies.after(setup_physics_world),
            ),
        )
        .add_systems(Update, check_keys)
        .run();
}

fn setup_camera(mut commands: Commands) {
    commands.spawn(Camera2dBundle {
        projection: OrthographicProjection {
            scale: 0.05,
            far: 1000.,
            near: -1000.,
            ..OrthographicProjection::default()
        },
        transform: Transform::from_translation(Vec3::new(0., 10., 0.)),
        ..Camera2dBundle::default()
    });
}

fn setup_instructions(mut commands: Commands) {
    commands.spawn((
        TextBundle::from_section(
            "Use mouse to drag it up",
            TextStyle {
                font_size: 20.0,
                color: Color::WHITE,
                ..default()
            },
        )
        .with_style(Style {
            position_type: PositionType::Absolute,
            top: Val::Px(5.0),
            left: Val::Px(15.0),
            ..default()
        }),
        InfoText,
    ));
}

fn setup_physics_world(world: &mut World) {
    let gravity = Vec2::new(0., -9.81);
    let b2_world = b2World::new(gravity);
    world.insert_non_send_resource(b2_world);
}

#[derive(Component)]
struct GroundBody;

#[derive(Component)]
struct BoxBody;

fn setup_physics_bodies(mut commands: Commands) {
    let ground_entity = create_ground(&mut commands);
    let box_entity_1 = create_box(&mut commands, -5., Dynamic);

    /*let joint_def = b2MouseJointDef {
        target: Vec2::new(-5., 20.),
        stiffness: 100.,
        damping: 0.1,
        max_force: f32::MAX,
        ..default()
    };

    commands.spawn_empty().add(CreateMouseJoint::new(
        ground_entity,
        box_entity_1,
        true,
        &joint_def,
    ));

    println!("Created mouse joint");*/
}

fn create_ground(commands: &mut Commands) -> Entity {
    let ground_entity = commands.spawn((b2BodyBundle::default(), GroundBody)).id();

    let shape = b2Shape::EdgeTwoSided {
        v1: Vec2::new(-40., 0.),
        v2: Vec2::new(40., 0.),
    };
    let fixture_def = b2FixtureDef::new(shape, 0.);
    commands.spawn((
        b2Fixture::new(ground_entity, &fixture_def),
        DebugDrawFixtures::default_static(),
    ));

    return ground_entity;
}

fn create_box(commands: &mut Commands, offset_x: f32, body_type: b2BodyType) -> Entity {
    let body_def = b2BodyDef {
        body_type,
        position: Vec2::new(offset_x, 20.),
        angle: 0.5 * PI,
        allow_sleep: false,
        ..default()
    };
    let box_entity = commands.spawn((b2BodyBundle::new(&body_def), BoxBody)).id();

    let box_shape = b2Shape::create_box(1.0, 1.0);
    let fixture_def = b2FixtureDef::new(box_shape, 1.);
    commands.spawn((
        b2Fixture::new(box_entity, &fixture_def),
        DebugDrawFixtures::default_dynamic(),
    ));

    return box_entity;
}

fn check_keys(
    input: Res<Input<MouseButton>>,
    mut joints: Query<&mut b2MouseJoint>,
    mut gizmos: Gizmos,
    camera_query: Query<(&Camera, &GlobalTransform)>,
    windows: Query<&Window>,
    box_query: Query<(&b2Body, Entity), With<BoxBody>>,
    ground_entity: Query<Entity, (With<GroundBody>, Without<BoxBody>)>,
    commands: &mut Commands,
) {
    let (camera, camera_transform) = camera_query.single();

    let Some(cursor_position) = windows.single().cursor_position() else {
        return;
    };

    if input.just_pressed(MouseButton::Left) {
        let joint_def = b2MouseJointDef {
            target: Vec2::new(-5., 20.),
            stiffness: 100.,
            damping: 0.1,
            max_force: f32::MAX,
            ..default()
        };

        commands.spawn_empty().add(CreateMouseJoint::new(
            ground_entity.iter().next().unwrap(),
            box_query.iter().next().unwrap().1,
            true,
            &joint_def,
        ));

        println!("Created mouse joint");
    }

    // Calculate a world position based on the cursor's position.
    let Some(point) = camera.viewport_to_world_2d(camera_transform, cursor_position) else {
        return;
    };

    // set target to mouse

    let mut joint_maybe = joints.get_single_mut();
    if !joint_maybe.is_ok() {
        return;
    }
    let joint = joint_maybe.unwrap();

    let mut end = point;
    for box_body in box_query.iter() {
        end = box_body.0.position;
    }
    gizmos.line_2d(point, end, Color::WHITE);
}
