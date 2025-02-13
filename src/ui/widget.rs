pub(super) mod button;
pub(super) mod checkbox;
pub(super) mod click;
pub(super) mod progress_bar;
pub(super) mod text_edit;
pub(super) mod ui_root;

use bevy::{prelude::*, ui::FocusPolicy};

use super::theme::Theme;
use button::ButtonPlugin;
use checkbox::CheckboxPlugin;
use click::ClickPlugin;
use progress_bar::ProgressBarPlugin;
use text_edit::TextEditPlugin;
use ui_root::UiRootPlugin;

pub(super) struct WidgetPlugin;

impl Plugin for WidgetPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((
            ButtonPlugin,
            CheckboxPlugin,
            ClickPlugin,
            ProgressBarPlugin,
            TextEditPlugin,
            UiRootPlugin,
        ));
    }
}

#[derive(Bundle)]
pub(super) struct LabelBundle {
    label: Label,
    text_bundle: TextBundle,
}

impl LabelBundle {
    pub(super) fn normal(theme: &Theme, text: impl Into<String>) -> Self {
        Self {
            label: Label,
            text_bundle: TextBundle::from_section(text, theme.label.normal.clone()),
        }
    }

    pub(super) fn large(theme: &Theme, text: impl Into<String>) -> Self {
        Self {
            label: Label,
            text_bundle: TextBundle::from_section(text, theme.label.large.clone()),
        }
    }

    pub(super) fn symbol(theme: &Theme, text: impl Into<String>) -> Self {
        Self {
            label: Label,
            text_bundle: TextBundle::from_section(text, theme.label.symbol.clone()),
        }
    }
}

#[derive(Bundle)]
pub(super) struct DialogBundle {
    dialog: Dialog,
    node_bundle: NodeBundle,
}

impl DialogBundle {
    pub(super) fn new(theme: &Theme) -> Self {
        Self {
            dialog: Dialog,
            node_bundle: NodeBundle {
                style: Style {
                    position_type: PositionType::Absolute,
                    width: Val::Percent(100.0),
                    height: Val::Percent(100.0),
                    align_items: AlignItems::Center,
                    justify_content: JustifyContent::Center,
                    ..Default::default()
                },
                focus_policy: FocusPolicy::Block,
                background_color: theme.modal_color.into(),
                ..Default::default()
            },
        }
    }
}

#[derive(Component)]
pub(super) struct Dialog;
