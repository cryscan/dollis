use bevy::prelude::{shape::Plane, *};
use bevy_rapier3d::prelude::*;
use oxidized_navigation::NavMeshAffector;

use super::{
    city::{ActiveCity, CityPlugin},
    collision_groups::LifescapeGroupsExt,
    cursor_hover::Hoverable,
    game_state::GameState,
};

pub(super) struct GroundPlugin;

impl Plugin for GroundPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(GameState::City), Self::spawn_system)
            .add_systems(OnExit(GameState::City), Self::despawn_system)
            .add_systems(
                OnEnter(GameState::Family),
                Self::spawn_system.after(CityPlugin::activation_system),
            )
            .add_systems(OnExit(GameState::Family), Self::despawn_system);
    }
}

impl GroundPlugin {
    fn spawn_system(
        activated_cities: Query<Entity, Added<ActiveCity>>,
        mut commands: Commands,
        mut meshes: ResMut<Assets<Mesh>>,
        mut materials: ResMut<Assets<StandardMaterial>>,
    ) {
        commands
            .entity(activated_cities.single())
            .with_children(|parent| {
                parent.spawn(GroundBundle {
                    pbr_bundle: PbrBundle {
                        mesh: meshes.add(Mesh::from(Plane::from_size(GroundBundle::SIZE))),
                        material: materials.add(Color::rgb_u8(69, 108, 69).into()),
                        ..Default::default()
                    },
                    ..Default::default()
                });
                parent.spawn(DirectionalLightBundle {
                    directional_light: DirectionalLight {
                        illuminance: 6000.0,
                        shadows_enabled: true,
                        ..Default::default()
                    },
                    transform: Transform::from_xyz(4.0, 7.0, 5.0).looking_at(Vec3::ZERO, Vec3::Y),
                    ..Default::default()
                });
            });
    }

    fn despawn_system(
        mut commands: Commands,
        active_cities: Query<&Children, With<ActiveCity>>,
        direction_lights: Query<Entity, With<DirectionalLight>>,
        grounds: Query<Entity, With<Ground>>,
    ) {
        let children = active_cities.single();
        let light_entity = *children
            .iter()
            .find(|&&entity| direction_lights.get(entity).is_ok())
            .expect("deactivated city should have a children light");
        commands.entity(light_entity).despawn();

        let ground_entity = *children
            .iter()
            .find(|&&entity| grounds.get(entity).is_ok())
            .expect("deactivated city should have a children ground");
        commands.entity(ground_entity).despawn();
    }
}

#[derive(Bundle)]
struct GroundBundle {
    name: Name,
    collider: Collider,
    collision_groups: CollisionGroups,
    ground: Ground,
    hoverable: Hoverable,
    nav_mesh_affector: NavMeshAffector,
    pbr_bundle: PbrBundle,
}

impl GroundBundle {
    const SIZE: f32 = 50.0;
}

impl Default for GroundBundle {
    fn default() -> Self {
        Self {
            name: Name::new("Ground"),
            collider: Collider::cuboid(Self::SIZE / 2.0, 0.0, Self::SIZE / 2.0),
            collision_groups: CollisionGroups::new(Group::GROUND, Group::ALL),
            ground: Ground,
            hoverable: Hoverable,
            nav_mesh_affector: NavMeshAffector,
            pbr_bundle: Default::default(),
        }
    }
}

#[derive(Component)]
pub(super) struct Ground;
