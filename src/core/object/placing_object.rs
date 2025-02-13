use std::{
    f32::consts::{FRAC_PI_4, PI},
    fmt::Debug,
};

use bevy::{asset::HandleId, math::Vec3Swizzles, prelude::*, window::PrimaryWindow};
use bevy_rapier3d::prelude::*;
use leafwing_input_manager::common_conditions::action_just_pressed;

use crate::core::{
    action::Action,
    asset_metadata::{self, ObjectMetadata},
    city::CityMode,
    collision_groups::LifescapeGroupsExt,
    component_commands::ComponentCommandsExt,
    cursor_hover::CursorHover,
    family::FamilyMode,
    game_state::GameState,
    object::{ObjectDespawn, ObjectEventConfirmed, ObjectMove, ObjectPath, ObjectSpawn},
    player_camera::PlayerCamera,
    ready_scene::ReadyScene,
    wall::{WallEdges, WallObject, HALF_WIDTH},
};

pub(super) struct PlacingObjectPlugin;

impl Plugin for PlacingObjectPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            OnExit(CityMode::Objects),
            Self::cancel_system.pipe(Self::cleanup_system),
        )
        .add_systems(
            OnExit(FamilyMode::Building),
            Self::cancel_system.pipe(Self::cleanup_system),
        )
        .add_systems(
            Update,
            (
                (
                    Self::init_system,
                    Self::scene_init_system,
                    Self::picking_system
                        .run_if(action_just_pressed(Action::Confirm))
                        .run_if(not(any_with_component::<PlacingObject>())),
                    Self::confirmation_system
                        .after(Self::collision_system)
                        .run_if(action_just_pressed(Action::Confirm)),
                    Self::despawn_system.run_if(action_just_pressed(Action::Delete)),
                    Self::cancel_system.pipe(Self::cleanup_system).run_if(
                        action_just_pressed(Action::Cancel)
                            .or_else(on_event::<ObjectEventConfirmed>()),
                    ),
                ),
                (
                    Self::rotation_system.run_if(action_just_pressed(Action::RotateObject)),
                    Self::movement_system,
                    Self::snapping_system,
                    Self::collision_system,
                    Self::material_system,
                )
                    .chain(),
            )
                .run_if(
                    in_state(GameState::City)
                        .and_then(in_state(CityMode::Objects))
                        .or_else(
                            in_state(GameState::Family).and_then(in_state(FamilyMode::Building)),
                        ),
                ),
        )
        .add_systems(
            PostUpdate,
            Self::exclusive_system.pipe(Self::cleanup_system),
        );
    }
}

impl PlacingObjectPlugin {
    fn picking_system(
        mut commands: Commands,
        hovered_objects: Query<(Entity, &Parent), (With<ObjectPath>, With<CursorHover>)>,
        children: Query<&Children>,
        mut groups: Query<&mut CollisionGroups>,
    ) {
        if let Ok((placing_entity, parent)) = hovered_objects.get_single() {
            commands.entity(**parent).with_children(|parent| {
                parent.spawn(PlacingObject::moving(placing_entity));
            });

            // To exclude from collision with the placing object.
            for child_entity in children.iter_descendants(placing_entity) {
                if let Ok(mut group) = groups.get_mut(child_entity) {
                    group.memberships ^= Group::OBJECT;
                }
            }
        }
    }

    fn init_system(
        mut commands: Commands,
        asset_server: Res<AssetServer>,
        object_metadata: Res<Assets<ObjectMetadata>>,
        objects: Query<(&Transform, &Handle<Scene>, &ObjectPath)>,
        new_placing_objects: Query<(Entity, &PlacingObject), Added<PlacingObject>>,
    ) {
        for (placing_entity, placing_object) in &new_placing_objects {
            debug!("created placing object {placing_object:?}");

            let mut placing_entity = commands.entity(placing_entity);

            let (transform, scene_handle, object_metadata) = match placing_object.kind {
                PlacingObjectKind::Spawning(id) => {
                    let metadata_path = asset_server
                        .get_handle_path(id)
                        .expect("spawning object metadata should have a path");
                    let metadata_handle = asset_server.get_handle(id);
                    let object_metadata =
                        object_metadata.get(&metadata_handle).unwrap_or_else(|| {
                            panic!("{metadata_path:?} should correspond to metadata")
                        });
                    let scene_handle = asset_server.load(asset_metadata::scene_path(metadata_path));
                    placing_entity.insert(CursorOffset::default());
                    let transform = Transform::from_rotation(Quat::from_rotation_y(PI)); // Rotate towards camera.
                    (transform, scene_handle, object_metadata)
                }
                PlacingObjectKind::Moving(object_entity) => {
                    let (transform, scene_handle, object_path) = objects
                        .get(object_entity)
                        .expect("moving object should exist with these components");
                    let metadata_handle = asset_server.load(&*object_path.0);
                    let object_metadata = object_metadata
                        .get(&metadata_handle)
                        .unwrap_or_else(|| panic!("{object_path:?} should correspond to metadata"));

                    (*transform, scene_handle.clone(), object_metadata)
                }
            };

            placing_entity
                .insert((
                    Name::new("Placing object"),
                    AsyncSceneCollider::default(),
                    SceneBundle {
                        scene: scene_handle,
                        transform,
                        ..Default::default()
                    },
                ))
                .insert_reflect(
                    object_metadata
                        .components
                        .iter()
                        .map(|component| component.clone_value())
                        .collect::<Vec<_>>(),
                );
        }
    }

    fn scene_init_system(
        mut commands: Commands,
        placing_objects: Query<(Entity, &PlacingObject), Added<ReadyScene>>,
        chidlren: Query<&Children>,
        mut objects: Query<&mut Visibility>,
        meshes: Query<(), With<Handle<Mesh>>>,
    ) {
        for (scene_entity, placing_object) in &placing_objects {
            if let PlacingObjectKind::Moving(object_entity) = placing_object.kind {
                let mut visibility = objects
                    .get_mut(object_entity)
                    .expect("moving object reference a valid object");
                *visibility = Visibility::Hidden;
            }

            for child_entity in chidlren
                .iter_descendants(scene_entity)
                .filter(|&entity| meshes.get(entity).is_ok())
            {
                commands
                    .entity(child_entity)
                    .insert(CollisionGroups::new(Group::NONE, Group::NONE));
            }
        }
    }

    fn rotation_system(mut placing_objects: Query<&mut Transform, With<PlacingObject>>) {
        if let Ok(mut transform) = placing_objects.get_single_mut() {
            const ROTATION_STEP: f32 = -FRAC_PI_4;
            transform.rotate_y(ROTATION_STEP);
        }
    }

    fn movement_system(
        mut commands: Commands,
        rapier_ctx: Res<RapierContext>,
        windows: Query<&Window, With<PrimaryWindow>>,
        cameras: Query<(&GlobalTransform, &Camera), With<PlayerCamera>>,
        mut placing_objects: Query<
            (Entity, &mut Transform, Option<&CursorOffset>),
            With<PlacingObject>,
        >,
    ) {
        let Ok((entity, mut transform, cursor_offset)) = placing_objects.get_single_mut() else {
            return;
        };
        let Some(cursor_pos) = windows.single().cursor_position() else {
            return;
        };

        let (&camera_transform, camera) = cameras.single();
        let ray = camera
            .viewport_to_world(&camera_transform, cursor_pos)
            .expect("ray should be created from screen coordinates");

        let toi = rapier_ctx
            .cast_ray(
                ray.origin,
                ray.direction,
                f32::MAX,
                false,
                CollisionGroups::new(Group::ALL, Group::GROUND | Group::WALL).into(),
            )
            .map(|(_, toi)| toi)
            .unwrap_or_default();

        let mut ray_translation = ray.origin + ray.direction * toi;
        ray_translation.y = 0.0;
        let offset = cursor_offset.copied().unwrap_or_else(|| {
            let offset = CursorOffset(transform.translation.xz() - ray_translation.xz());
            commands.entity(entity).insert(offset);
            offset
        });
        transform.translation = ray_translation + Vec3::new(offset.x, 0.0, offset.y);
    }

    fn snapping_system(
        walls: Query<&WallEdges>,
        mut placing_objects: Query<(&mut Transform, &mut PlacingObject), With<WallObject>>,
    ) {
        let Ok((mut transform, mut placing_object)) = placing_objects.get_single_mut() else {
            return;
        };

        const SNAP_DELTA: f32 = 1.0;
        let translation_2d = transform.translation.xz();
        if let Some((edge, edge_point)) = walls
            .iter()
            .flat_map(|edges| edges.iter())
            .map(|&(a, b)| {
                let edge = b - a;
                (edge, closest_point(a, edge, translation_2d))
            })
            .find(|(_, edge_point)| edge_point.distance(translation_2d) <= SNAP_DELTA)
        {
            const GAP: f32 = 0.03; // A small gap between the object and wall to avoid collision.
            let sign = edge.perp_dot(translation_2d - edge_point).signum();
            let snap_point = edge_point + sign * edge.perp().normalize() * (HALF_WIDTH + GAP);
            let edge_angle = edge.angle_between(Vec2::X * sign);
            transform.translation.x = snap_point.x;
            transform.translation.z = snap_point.y;
            transform.rotation = Quat::from_rotation_y(edge_angle);
            if !placing_object.allowed_place {
                placing_object.allowed_place = true;
            }
        } else if placing_object.allowed_place {
            placing_object.allowed_place = false;
        }
    }

    fn collision_system(
        rapier_ctx: Res<RapierContext>,
        mut placing_objects: Query<(Entity, &mut PlacingObject)>,
        children: Query<&Children>,
        child_meshes: Query<(&Collider, &GlobalTransform)>,
    ) {
        let Ok((object_entity, mut placing_object)) = placing_objects.get_single_mut() else {
            return;
        };

        for (collider, transform) in children
            .iter_descendants(object_entity)
            .flat_map(|entity| child_meshes.get(entity))
        {
            let (_, rotation, translation) = transform.to_scale_rotation_translation();
            let mut intersects = false;
            rapier_ctx.intersections_with_shape(
                translation,
                rotation,
                collider,
                CollisionGroups::new(Group::ALL, Group::OBJECT | Group::WALL).into(),
                |_| {
                    intersects = true;
                    false
                },
            );
            if intersects {
                if !placing_object.collides {
                    placing_object.collides = true;
                }
                return;
            }
        }

        if placing_object.collides {
            placing_object.collides = false;
        }
    }

    fn material_system(
        mut materials: ResMut<Assets<StandardMaterial>>,
        placing_objects: Query<(Entity, &PlacingObject), Changed<PlacingObject>>,
        children: Query<&Children>,
        mut material_handles: Query<&mut Handle<StandardMaterial>>,
    ) {
        if let Ok((placing_entity, placing_object)) = placing_objects.get_single() {
            for child_entity in children.iter_descendants(placing_entity) {
                if let Ok(mut material_handle) = material_handles.get_mut(child_entity) {
                    let mut material = materials
                        .get(&material_handle)
                        .cloned()
                        .expect("material handle should be valid");

                    material.base_color =
                        if placing_object.collides || !placing_object.allowed_place {
                            Color::RED
                        } else {
                            Color::WHITE
                        };
                    *material_handle = materials.add(material);
                }
            }
            debug!("assigned material color for {placing_object:?}");
        }
    }

    fn confirmation_system(
        mut move_events: EventWriter<ObjectMove>,
        mut spawn_events: EventWriter<ObjectSpawn>,
        asset_server: Res<AssetServer>,
        placing_objects: Query<(&Transform, &PlacingObject)>,
    ) {
        if let Ok((transform, placing_object)) = placing_objects.get_single() {
            if !placing_object.collides && placing_object.allowed_place {
                debug!("confirmed placing object {placing_object:?}");
                match placing_object.kind {
                    PlacingObjectKind::Spawning(id) => {
                        let metadata_path = asset_server
                            .get_handle_path(id)
                            .expect("spawning object metadata should have a path");
                        spawn_events.send(ObjectSpawn {
                            metadata_path: metadata_path.path().to_path_buf(),
                            position: transform.translation.xz(),
                            rotation: transform.rotation,
                        });
                    }
                    PlacingObjectKind::Moving(entity) => move_events.send(ObjectMove {
                        entity,
                        translation: transform.translation,
                        rotation: transform.rotation,
                    }),
                }
            }
        }
    }

    fn despawn_system(
        mut commands: Commands,
        mut despawn_events: EventWriter<ObjectDespawn>,
        placing_objects: Query<(Entity, &PlacingObject)>,
    ) {
        if let Ok((entity, placing_object)) = placing_objects.get_single() {
            if let PlacingObjectKind::Moving(entity) = placing_object.kind {
                debug!("sent despawn event for placing object {placing_object:?}");
                despawn_events.send(ObjectDespawn(entity));
            } else {
                debug!("cancelled placing object {placing_object:?}");
                commands.entity(entity).despawn_recursive();
            }
        }
    }

    fn exclusive_system(
        new_placing_objects: Query<Entity, Added<PlacingObject>>,
        placing_objects: Query<(Entity, &PlacingObject)>,
    ) -> Vec<(Entity, PlacingObjectKind)> {
        if let Some(new_entity) = new_placing_objects.iter().last() {
            return placing_objects
                .iter()
                .filter(|&(entity, _)| entity != new_entity)
                .map(|(entity, placing_object)| (entity, placing_object.kind))
                .collect();
        }

        Vec::new()
    }

    fn cancel_system(
        placing_objects: Query<(Entity, &PlacingObject)>,
    ) -> Vec<(Entity, PlacingObjectKind)> {
        placing_objects
            .iter()
            .map(|(entity, placing_object)| (entity, placing_object.kind))
            .collect()
    }

    fn cleanup_system(
        In(placing_objects): In<Vec<(Entity, PlacingObjectKind)>>,
        mut commands: Commands,
        mut visibility: Query<&mut Visibility>,
        children: Query<&Children>,
        mut groups: Query<&mut CollisionGroups>,
    ) {
        for (placing_entity, kind) in placing_objects {
            debug!("despawned placing object {kind:?}");
            commands.entity(placing_entity).despawn_recursive();

            if let PlacingObjectKind::Moving(object_entity) = kind {
                // Object could be invalid in case of removal.
                if let Ok(mut visibility) = visibility.get_mut(object_entity) {
                    *visibility = Visibility::Visible;
                }

                // Restore object's collisions back.
                for child_entity in children.iter_descendants(object_entity) {
                    if let Ok(mut group) = groups.get_mut(child_entity) {
                        group.memberships |= Group::OBJECT;
                    }
                }
            }
        }
    }
}

/// Returns the minimal distance from point `p` to the segment defined by its `origin` and `displacement` vector.
fn closest_point(origin: Vec2, displacement: Vec2, p: Vec2) -> Vec2 {
    // Consider the line extending the segment, parameterized as `origin + t * displacement`.
    let t = (p - origin).dot(displacement) / displacement.length_squared();
    // We clamp `t` to handle points outside the segment.
    origin + t.clamp(0.0, 1.0) * displacement // Projection of point `p` onto the segment.
}

#[derive(Component, Debug, Clone, Copy)]
pub(crate) struct PlacingObject {
    kind: PlacingObjectKind,
    collides: bool,
    allowed_place: bool,
}

impl PlacingObject {
    pub(crate) fn moving(object_entity: Entity) -> Self {
        Self {
            kind: PlacingObjectKind::Moving(object_entity),
            collides: false,
            allowed_place: true,
        }
    }

    pub(crate) fn spawning(id: HandleId) -> Self {
        Self {
            kind: PlacingObjectKind::Spawning(id),
            collides: false,
            allowed_place: true,
        }
    }
}

/// Marks an entity as an object that should be moved with cursor to preview spawn position.
#[derive(Debug, Clone, Copy)]
pub(crate) enum PlacingObjectKind {
    Spawning(HandleId),
    Moving(Entity),
}

/// Contains an offset between cursor position on first creation and object origin.
#[derive(Clone, Component, Copy, Default, Deref)]
struct CursorOffset(Vec2);
