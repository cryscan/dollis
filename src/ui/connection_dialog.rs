use bevy::prelude::*;
use bevy_replicon::{prelude::*, renet::transport::NetcodeClientTransport};

use super::{
    theme::Theme,
    widget::{button::TextButtonBundle, click::Click, ui_root::UiRoot, DialogBundle, LabelBundle},
};

pub(super) struct ConnectionDialogPlugin;

impl Plugin for ConnectionDialogPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (
                Self::setup_system.run_if(resource_added::<RenetClient>()),
                Self::button_system,
                Self::cleanup_system.run_if(client_just_diconnected()),
            ),
        );
    }
}

impl ConnectionDialogPlugin {
    fn setup_system(mut commands: Commands, theme: Res<Theme>, roots: Query<Entity, With<UiRoot>>) {
        commands.entity(roots.single()).with_children(|parent| {
            parent
                .spawn((ConnectionDialog, DialogBundle::new(&theme)))
                .with_children(|parent| {
                    parent
                        .spawn(NodeBundle {
                            style: Style {
                                flex_direction: FlexDirection::Column,
                                justify_content: JustifyContent::Center,
                                align_items: AlignItems::Center,
                                padding: theme.padding.normal,
                                row_gap: theme.gap.normal,
                                ..Default::default()
                            },
                            background_color: theme.panel_color.into(),
                            ..Default::default()
                        })
                        .with_children(|parent| {
                            parent.spawn(LabelBundle::normal(&theme, "Connecting to server"));
                            parent
                                .spawn((CancelButton, TextButtonBundle::normal(&theme, "Cancel")));
                        });
                });
        });
    }

    fn button_system(
        mut commands: Commands,
        mut click_events: EventReader<Click>,
        buttons: Query<(), With<CancelButton>>,
        dialogs: Query<Entity, With<ConnectionDialog>>,
    ) {
        for event in &mut click_events {
            if buttons.get(event.0).is_ok() {
                commands.remove_resource::<RenetClient>();
                commands.entity(dialogs.single()).despawn_recursive();
            }
        }
    }

    fn cleanup_system(mut commands: Commands, dialogs: Query<Entity, With<ConnectionDialog>>) {
        commands.entity(dialogs.single()).despawn_recursive();
    }
}

pub fn client_just_diconnected(
) -> impl FnMut(Local<bool>, Option<Res<NetcodeClientTransport>>) -> bool {
    |mut last_connected: Local<bool>, transport| {
        let disconnected = transport
            .map(|transport| transport.is_disconnected())
            .unwrap_or(true);

        let just_disconnected = *last_connected && disconnected;
        *last_connected = !disconnected;
        just_disconnected
    }
}

#[derive(Component)]
struct CancelButton;

#[derive(Component)]
struct ConnectionDialog;
