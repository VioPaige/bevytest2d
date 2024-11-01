#![allow(non_snake_case)]

use std::{
    collections::HashMap,
    time::Duration,
};

use bevy::{
    dev_tools::fps_overlay::{
        FpsOverlayConfig, 
        FpsOverlayPlugin
    }, prelude::*, render::{view::RenderLayers, Render}, time::common_conditions::on_timer, window::{PresentMode, PrimaryWindow, WindowPlugin}
};
use bevy_rapier2d::prelude::*;
use rand::Rng;

#[derive(Component)]
struct Tank;

#[derive(Component)]
struct Bullet;

#[derive(Component)]
struct Object;

#[derive(Component)]
struct RealPosition(Vec2);

#[derive(Component)]
struct Velocity2d(Vec2);

fn main() {
    App::new()
        // .insert_resource(WindowDescriptor {
        //     present_mode: PresentMode::Immediate,
        //     ..default()
        // })
    
        .add_plugins((
            DefaultPlugins.set(WindowPlugin {
                primary_window: Some(Window {
                    present_mode: PresentMode::Immediate,
                    ..default()
                }),
                ..default()
            }),
            FpsOverlayPlugin {
                config: FpsOverlayConfig {
                    text_config: TextStyle {
                        font_size: 20.,
                        color: Color::srgb(1., 1., 1.),
                        ..default()
                    }
                }
            },
            RapierPhysicsPlugin::<()>::default(),
            RapierDebugRenderPlugin::default()
        ))

        .insert_resource(ClearColor(Color::srgb(0., 0., 0.)))
        
        .add_systems(Startup, setup)
        .add_systems(Update, (handleMovement, drawGrid, update, spawnBullets, collisionEvent).chain())
        .add_systems(Update, spawnObjects.run_if(on_timer(Duration::from_millis(250))))
        
        .run();
}

fn setup(
    mut configStore: ResMut<GizmoConfigStore>,
    mut commands: Commands,
    assetServer: Res<AssetServer>
) {
    configStore.config_mut::<DefaultGizmoConfigGroup>().0.render_layers = RenderLayers::layer(0);

    let tankTexture: Handle<Image> = assetServer.load("sprites/Tank.png");

    commands.spawn((Camera2dBundle::default(), RenderLayers::layer(0)));
    commands.spawn((
        SpriteBundle {
            transform: Transform::from_xyz(-0., -0., 2.),
            texture: tankTexture,
            sprite: Sprite {
                custom_size: Some(Vec2::new(50., 50.)),
                ..default()
            },
            ..default()
        },
        Tank,
        RealPosition(Vec2::new(0., 0.)),
        RenderLayers::layer(0)
    ));
}

fn update(
    mut bulletQuery: Query<(&mut Transform, &mut RealPosition, &Velocity2d), (With<Bullet>, Without<Tank>)>,
    mut objectQuery: Query<(&mut Transform, &RealPosition), (With<Object>, Without<Bullet>, Without<Tank>)>,
    mut tankQuery: Query<(&mut Transform, &RealPosition), With<Tank>>,
    time: Res<Time>,
    windows: Query<&Window, With<PrimaryWindow>>,
) {
    let dt = time.delta_seconds();
    let window = windows.single();

    let height = window.height();
    let width = window.width();

    
    for mut bullet in &mut bulletQuery {
        bullet.1.0 += (bullet.2.0) * dt;
        bullet.0.translation = Vec3::new(bullet.1.0.x - tankQuery.single().1.0.x, -bullet.1.0.y - tankQuery.single().1.0.y, 0.);
    }

    for mut object in &mut objectQuery {
        object.0.translation = Vec3::new(object.1.0.x - tankQuery.single().1.0.x, -object.1.0.y - tankQuery.single().1.0.y, 0.);
    }

    if let Some(position) = window.cursor_position() {
        let vec = Vec3::new(position.x - (width * 0.5), position.y - (height * 0.5), 0.);
        let rotation = Quat::from_rotation_z(-vec.y.atan2(vec.x) - std::f32::consts::FRAC_PI_2);
        for mut tank in &mut tankQuery {
            tank.0.rotation = rotation;
        }
    }

}

fn spawnBullets(
    mut commands: Commands,
    mouse: Res<ButtonInput<MouseButton>>,
    tankQuery: Query<&RealPosition, With<Tank>>,
    windows: Query<&Window, With<PrimaryWindow>>,

    assetServer: Res<AssetServer>
) {
    let bulletTexture: Handle<Image> = assetServer.load("sprites/Bullet.png");

    if mouse.just_pressed(MouseButton::Left) {
        let window = windows.single();

        let height = window.height();
        let width = window.width();

        if let Some(position) = window.cursor_position() {
            let velocity = Vec2::new(position.x - (width * 0.5), position.y - (height * 0.5)).normalize() * 600.;

            commands.spawn((
                SpriteBundle {
                    transform: Transform::from_xyz(0., 0., 0.),
                    texture: bulletTexture,
                    sprite: Sprite {
                        custom_size: Some(Vec2::new(10., 10.)),
                        ..default()
                    },
                    ..default()
                },
                // ActiveEvents::COLLISION_EVENTS,
                Collider::ball(10.),
                RigidBody::KinematicPositionBased,
                Bullet,
                RealPosition(Vec2::new(tankQuery.single().0.x, -tankQuery.single().0.y)),
                Velocity2d(velocity),
            // )).insert(ActiveEvents::COLLISION_EVENTS);
            )).insert(ActiveCollisionTypes::default() | ActiveCollisionTypes::KINEMATIC_STATIC).insert(ActiveEvents::COLLISION_EVENTS);
        }
    }
}

fn spawnObjects(
    mut commands: Commands,
    tankQuery: Query<&RealPosition, With<Tank>>,
    windows: Query<&Window, With<PrimaryWindow>>,

    assetServer: Res<AssetServer>
) {
    let objectTexture: Handle<Image> = assetServer.load("sprites/Object.png");
    let tank = tankQuery.single().0;

    let window = windows.single();
    let height = window.height();
    let width = window.width();

    let mut rng = rand::thread_rng();

    commands.spawn((
        SpriteBundle {
            transform: Transform::from_xyz(0., 0., 0.),
            texture: objectTexture,
            sprite: Sprite {
                custom_size: Some(Vec2::new(25., 25.)),
                ..default()
            },
            ..default()
        },
        // RigidBody::Fixed,
        Collider::ball(25.),
        RigidBody::KinematicPositionBased,
        Object,
        RealPosition(tank + Vec2::new(rng.gen_range(-width..width), rng.gen_range(-height..height))),
    // )).insert(ActiveEvents::COLLISION_EVENTS);
    )).insert(ActiveCollisionTypes::default() | ActiveCollisionTypes::KINEMATIC_STATIC).insert(ActiveEvents::COLLISION_EVENTS);
}

fn handleMovement(
    keys: Res<ButtonInput<KeyCode>>,
    mut tankQuery: Query<&mut RealPosition, With<Tank>>,
    time: Res<Time>,
) {
    let mut tank = tankQuery.single_mut();
    let dt = time.delta_seconds();

    let directions = HashMap::from([
        (KeyCode::KeyW, Vec2::new(0., 1.)),
        (KeyCode::KeyS, Vec2::new(0., -1.)),
        (KeyCode::KeyA, Vec2::new(-1., 0.)),
        (KeyCode::KeyD, Vec2::new(1., 0.)),
    ]);

    let mut vec = Vec2::new(0., 0.);
    for (key, direction) in directions {
        if keys.pressed(key) {
            vec += direction;
        }
    }
    
    // println!("{:?}, {:?}, {:?}", tank.0, vec, vec.normalize() * 2.);
    if vec.length() <= 0. { return }

    tank.0 +=vec.normalize() * 150. * dt;
}

// fn handleCollisions(
//     physicsContext: Res<RapierContext>,
//     bulletQuery: Query<Entity, With<Bullet>>,
//     objectQuery: Query<Entity, With<Object>>,
// ) {}

fn collisionEvent(
    mut commands: Commands,
    mut events: EventReader<CollisionEvent>,
    mut contactForceEvents: EventReader<ContactForceEvent>,
    bulletQuery: Query<Entity, With<Bullet>>,
    objectQuery: Query<Entity, With<Object>>,
) {
    for event in events.read() {
        println!("collision event");
        match event {
            CollisionEvent::Started(entity1, entity2, _) => {
                if bulletQuery.get(*entity1).is_ok() && objectQuery.get(*entity2).is_ok() {
                    commands.entity(*entity2).despawn();
                } else if bulletQuery.get(*entity2).is_ok() && objectQuery.get(*entity1).is_ok() {
                    commands.entity(*entity1).despawn();
                }
            }
            CollisionEvent::Stopped(entity1, entity2, _) => {}
        }
    }

    for event in contactForceEvents.read() {
        println!("contact force event");
    }
}

// replace for gizmos.draw_grid, that exists.. lol
fn drawGrid ( 
    // mut configStore: ResMut<GizmoConfigStore>,
    mut gizmos: Gizmos,
    tankQuery: Query<&RealPosition, With<Tank>>,
    windows: Query<&Window, With<PrimaryWindow>>,
) {

    let offset = 60.;
    let window = windows.single();
    let height = window.height();
    let width = window.width();
    let tank = tankQuery.single().0;

    let mut x = -60.;
    let mut y = -60.;
    
    while width >= x {
        gizmos.line_2d(
            Vec2::new(x - (width * 0.5) - (tank.x % offset), -height * 0.5),
            Vec2::new(x - (width * 0.5) - (tank.x % offset), height * 0.5),
            Color::srgba(0.5, 0.5, 0.5, 0.2),
        );

        x += offset;
    }

    while height >= y {
        gizmos.line_2d(
            Vec2::new(-width * 0.5, y - (height * 0.5) - (tank.y % offset)),
            Vec2::new(width * 0.5, y - (height * 0.5) - (tank.y % offset)),
            Color::srgba(0.5, 0.5, 0.5, 0.2),
        );

        y += offset;
    }
}
