use bevy::{prelude::*, window::close_on_esc};
use bevy_inspector_egui::quick::WorldInspectorPlugin;
use bevy_xpbd_3d::{
    components::{CollisionLayers, LayerMask, LockedAxes, RigidBody},
    plugins::{collision::Collider, PhysicsDebugPlugin, PhysicsPlugins},
    prelude::PhysicsLayer,
};
use leafwing_input_manager::{plugin::InputManagerPlugin, Actionlike};

use crate::player::{PlayerBundle, PlayerPlugin, Turret, TurretBundle};

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
            .add_systems(Update, close_on_esc);
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

    let ground_material = materials.add(StandardMaterial::default());
    let ground_mesh = meshes.add(Plane3d::default().mesh().size(15.0, 15.0));
    commands.spawn((
        PbrBundle {
            mesh: ground_mesh,
            material: ground_material.clone(),
            ..Default::default()
        },
        RigidBody::Static,
        Collider::cuboid(15.0, 0.01, 15.0),
        CollisionLayers::new(GamePhysicsLayer::Ground, LayerMask::ALL),
    ));

    let turret_mesh: Handle<Mesh> = asset_server.load("tank.glb#Mesh1/Primitive0");
    let turret_material = materials.add(StandardMaterial {
        base_color: Color::YELLOW_GREEN,
        ..Default::default()
    });
    let turret = commands
        .spawn(TurretBundle {
            turret: Turret,
            pbr_bundle: PbrBundle {
                mesh: turret_mesh,
                material: turret_material,
                ..Default::default()
            },
        })
        .id();
    let player_mesh: Handle<Mesh> = asset_server.load("tank.glb#Mesh0/Primitive0");
    let player_material = materials.add(StandardMaterial {
        base_color: Color::LIME_GREEN,
        ..Default::default()
    });
    let player_collider = commands
        .spawn((Collider::default(), Transform::from_xyz(0.0, 0.0, 0.0)))
        .id();
    let base = commands
        .spawn(PlayerBundle {
            pbr_bundle: PbrBundle {
                mesh: player_mesh,
                material: player_material,
                transform: Transform::from_xyz(0.0, 0.0, 0.0),
                ..Default::default()
            },
            rigidbody: RigidBody::Kinematic,
            locked_axes: LockedAxes::from_bits(0b000_111),
            ..Default::default()
        })
        .add_child(turret)
        .add_child(player_collider)
        .id();

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
