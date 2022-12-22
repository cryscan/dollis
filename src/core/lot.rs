pub(crate) mod spawning_lot;

use bevy::prelude::*;
use bevy_polyline::prelude::*;
use bevy_renet::renet::RenetServer;
use iyes_loopless::prelude::*;
use serde::{Deserialize, Serialize};

use super::{
    game_world::GameWorld,
    network::network_event::{
        client_event::{ClientEvent, ClientEventAppExt},
        server_event::{SendMode, ServerEvent, ServerEventAppExt},
    },
};
use spawning_lot::SpawningLotPlugin;

pub(super) struct LotPlugin;

impl Plugin for LotPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugin(SpawningLotPlugin)
            .add_client_event::<LotSpawn>()
            .add_server_event::<LotSpawnConfirmed>()
            .add_system(Self::init_system.run_if_resource_exists::<GameWorld>())
            .add_system(Self::vertices_update_system.run_if_resource_exists::<GameWorld>())
            .add_system(Self::spawn_system.run_if_resource_exists::<RenetServer>());
    }
}

impl LotPlugin {
    fn init_system(
        lot_material: Local<LotMaterial>,
        mut commands: Commands,
        mut polylines: ResMut<Assets<Polyline>>,
        spawned_lots: Query<(Entity, &LotVertices), Added<LotVertices>>,
    ) {
        for (entity, vertices) in &spawned_lots {
            commands.entity(entity).insert(PolylineBundle {
                polyline: polylines.add(Polyline {
                    vertices: vertices.0.clone(),
                }),
                material: lot_material.0.clone(),
                ..Default::default()
            });
        }
    }

    fn vertices_update_system(
        mut polylines: ResMut<Assets<Polyline>>,
        changed_lots: Query<(&Handle<Polyline>, &LotVertices, ChangeTrackers<LotVertices>)>,
    ) {
        for (polyline_handle, vertices, changed_vertices) in &changed_lots {
            if changed_vertices.is_changed() && !changed_vertices.is_added() {
                let polyline = polylines
                    .get_mut(polyline_handle)
                    .expect("polyline should be spawned on init");
                polyline.vertices = vertices.0.clone();
            }
        }
    }

    fn spawn_system(
        mut commands: Commands,
        mut spawn_events: EventReader<ClientEvent<LotSpawn>>,
        mut confirm_events: EventWriter<ServerEvent<LotSpawnConfirmed>>,
    ) {
        for ClientEvent { client_id, event } in spawn_events.iter().cloned() {
            commands.spawn(LotVertices(event.0));
            confirm_events.send(ServerEvent {
                mode: SendMode::Direct(client_id),
                event: LotSpawnConfirmed,
            });
        }
    }
}

#[derive(Component, Deref, DerefMut)]
struct LotVertices(Vec<Vec3>);

/// Stores a handle for the lot line material.
#[derive(Resource)]
struct LotMaterial(Handle<PolylineMaterial>);

impl FromWorld for LotMaterial {
    fn from_world(world: &mut World) -> Self {
        let mut polyline_materials = world.resource_mut::<Assets<PolylineMaterial>>();
        let material_handle = polyline_materials.add(PolylineMaterial {
            color: Color::WHITE,
            perspective: true,
            ..Default::default()
        });
        Self(material_handle)
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
struct LotSpawn(Vec<Vec3>);

#[derive(Debug, Deserialize, Serialize)]
struct LotSpawnConfirmed;
