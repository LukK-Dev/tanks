use bevy::{prelude::*, window::close_on_esc};
use bevy_inspector_egui::quick::WorldInspectorPlugin;
use bevy_xpbd_3d::prelude::*;
use leafwing_input_manager::{plugin::InputManagerPlugin, Actionlike};
use tanks::cycle_fullscreen_on_f11;

use crate::{
    player::{PlayerBundle, PlayerPlugin},
    turret::{Turret, TurretBundle},
};

#[derive(Actionlike, PartialEq, Eq, Hash, Clone, Copy, Debug, Reflect)]
pub enum Action {
    TurnLeft,
    TurnRight,
    MoveForward,
    MoveBackward,
    Shoot,
    Aim,
}

#[derive(PhysicsLayer, Default)]
pub enum GamePhysicsLayer {
    #[default]
    Default,
    Ground,
}

pub struct GamePlugin;

impl Plugin for GamePlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(DefaultPlugins)
            .add_plugins((
                #[cfg(debug_assertions)]
                WorldInspectorPlugin::default(),
                PhysicsPlugins::default(),
                #[cfg(debug_assertions)]
                PhysicsDebugPlugin::default(),
                InputManagerPlugin::<Action>::default(),
            ))
            .add_plugins(PlayerPlugin)
            .add_systems(Startup, setup)
            .add_systems(Update, (close_on_esc, cycle_fullscreen_on_f11));
    }
}

fn setup(
    mut commands: Commands,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut meshes: ResMut<Assets<Mesh>>,
    asset_server: Res<AssetServer>,
) {
    commands.spawn(Camera3dBundle {
        transform: Transform::from_translation(Vec3::new(0.0, 8.5, 10.5))
            .with_rotation(Quat::from_rotation_x(-0.75)),
        ..Default::default()
    });

    commands.spawn((
        RigidBody::Dynamic,
        Collider::sphere(1.0),
        PbrBundle {
            mesh: meshes.add(Cuboid::default()),
            material: materials.add(StandardMaterial::default()),
            transform: Transform::from_xyz(3.0, 3.0, 1.0),
            ..Default::default()
        },
    ));

    let ground_material = materials.add(StandardMaterial::default());
    let ground_mesh = meshes.add(Plane3d::default().mesh().size(15.0, 15.0));
    commands.spawn((
        PbrBundle {
            mesh: ground_mesh.clone(),
            material: ground_material.clone(),
            ..Default::default()
        },
        RigidBody::Static,
        Collider::cuboid(15.0, 0.01, 15.0),
        CollisionLayers::new(GamePhysicsLayer::Ground, LayerMask::ALL),
    ));

    let gun_mesh: Handle<Mesh> = asset_server.load("tank.glb#Mesh2/Primitive0");
    let gun_material = materials.add(Color::PURPLE);
    let gun = commands
        .spawn(PbrBundle {
            mesh: gun_mesh,
            material: gun_material,
            ..Default::default()
        })
        .id();
    let turret_mesh: Handle<Mesh> = asset_server.load("tank.glb#Mesh1/Primitive0");
    let turret_material = materials.add(Color::YELLOW_GREEN);
    let turret = commands
        .spawn(TurretBundle {
            turret: Turret,
            pbr_bundle: PbrBundle {
                mesh: turret_mesh,
                material: turret_material,
                transform: Transform::from_xyz(0.0, 0.3, 0.0),
                ..Default::default()
            },
        })
        .add_child(gun)
        .id();
    let player_mesh: Handle<Mesh> = asset_server.load("tank.glb#Mesh0/Primitive0");
    let player_material = materials.add(Color::LIME_GREEN);
    let player_collider = commands
        .spawn((
            Collider::cuboid(0.7, 0.4, 1.0),
            Transform::from_xyz(0.0, 0.0, -0.1),
            CollisionLayers::default(),
        ))
        .id();
    commands
        .spawn((PlayerBundle {
            pbr_bundle: PbrBundle {
                mesh: player_mesh,
                material: player_material,
                transform: Transform::from_xyz(0.0, 3.0, 0.0),
                ..Default::default()
            },
            rigidbody: RigidBody::Dynamic,
            locked_axes: LockedAxes::from_bits(0b000_111),
            ..Default::default()
        },))
        .add_child(turret)
        .add_child(player_collider);

    commands.insert_resource(AmbientLight {
        color: Color::WHITE,
        brightness: 700.0,
    });

    commands.spawn(DirectionalLightBundle {
        directional_light: DirectionalLight {
            illuminance: 3000.0,
            shadows_enabled: true,
            ..default()
        },
        transform: Transform {
            translation: Vec3::new(1.0, 2.0, 1.0),
            ..default()
        }
        .looking_at(Vec3::ZERO, Vec3::Y),
        ..default()
    });
}
