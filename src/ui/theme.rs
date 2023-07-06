use bevy::prelude::*;

pub(crate) struct ThemePlugin;

impl Plugin for ThemePlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<Theme>()
            .add_startup_system(Self::clear_color_system);
    }
}

impl ThemePlugin {
    fn clear_color_system(mut commands: Commands, theme: Res<Theme>) {
        commands.insert_resource(ClearColor(theme.background_color));
    }
}

#[derive(Resource)]
pub(crate) struct Theme {
    pub(crate) button: ButtonTheme,
    pub(crate) label: LabelTheme,
    pub(crate) text_edit: TextEditTheme,
    pub(crate) checkbox: CheckboxTheme,
    pub(crate) gap: GapTheme,
    pub(crate) padding: PaddingTheme,
    pub(crate) background_color: Color,
    pub(crate) modal_color: Color,
    pub(crate) panel_color: Color,
}

impl FromWorld for Theme {
    fn from_world(world: &mut World) -> Self {
        let asset_server = world.resource::<AssetServer>();
        let text_handle = asset_server.load("base/fonts/FiraSans-Bold.ttf");
        let symbol_handle = asset_server.load("base/fonts/NotoEmoji-Regular.ttf");
        Self {
            button: ButtonTheme {
                normal: Style {
                    size: Size::new(Val::Px(170.0), Val::Px(40.0)),
                    justify_content: JustifyContent::Center,
                    align_items: AlignItems::Center,
                    ..Default::default()
                },
                large: Style {
                    size: Size::new(Val::Px(200.0), Val::Px(60.0)),
                    justify_content: JustifyContent::Center,
                    align_items: AlignItems::Center,
                    ..Default::default()
                },
                square: Style {
                    size: Size::new(Val::Px(40.0), Val::Px(40.0)),
                    justify_content: JustifyContent::Center,
                    align_items: AlignItems::Center,
                    ..Default::default()
                },
                image_button: Style {
                    size: Size::new(Val::Px(60.0), Val::Px(60.0)),
                    justify_content: JustifyContent::Center,
                    align_items: AlignItems::Center,
                    ..Default::default()
                },
                image: Style {
                    size: Size::all(Val::Px(50.0)),
                    ..Default::default()
                },
                normal_text: TextStyle {
                    font: text_handle.clone(),
                    font_size: 35.0,
                    color: Color::rgb(0.9, 0.9, 0.9),
                },
                large_text: TextStyle {
                    font: text_handle.clone(),
                    font_size: 40.0,
                    color: Color::rgb(0.9, 0.9, 0.9),
                },
                square_text: TextStyle {
                    font: symbol_handle,
                    font_size: 35.0,
                    color: Color::rgb(0.9, 0.9, 0.9),
                },
                normal_color: Color::rgb(0.15, 0.15, 0.15),
                hovered_color: Color::rgb(0.25, 0.25, 0.25),
                pressed_color: Color::rgb(0.35, 0.75, 0.35),
                hovered_pressed_color: Color::rgb(0.25, 0.65, 0.25),
            },
            label: LabelTheme {
                normal: TextStyle {
                    font: text_handle.clone(),
                    font_size: 35.0,
                    color: Color::rgb(0.1, 0.1, 0.1),
                },
                large: TextStyle {
                    font: text_handle.clone(),
                    font_size: 45.0,
                    color: Color::rgb(0.1, 0.1, 0.1),
                },
            },
            text_edit: TextEditTheme {
                style: Style {
                    min_size: Size::new(Val::Px(200.0), Val::Px(30.0)),
                    ..Default::default()
                },
                text: TextStyle {
                    font: text_handle,
                    font_size: 30.0,
                    color: Color::rgb(0.1, 0.1, 0.1),
                },
                normal_color: Color::rgb(0.5, 0.5, 0.5),
                hovered_color: Color::rgb(0.55, 0.55, 0.55),
                active_color: Color::rgb(0.6, 0.6, 0.6),
                hovered_active_color: Color::rgb(0.65, 0.65, 0.65),
            },
            checkbox: CheckboxTheme {
                node: Style {
                    gap: Size::width(Val::Px(10.0)),
                    flex_direction: FlexDirection::Row,
                    align_items: AlignItems::Center,
                    ..Default::default()
                },
                button: Style {
                    size: Size::all(Val::Px(25.0)),
                    justify_content: JustifyContent::Center,
                    align_items: AlignItems::Center,
                    ..Default::default()
                },
                tick: Style {
                    size: Size::all(Val::Px(15.0)),
                    ..Default::default()
                },
                tick_color: Color::rgb(0.35, 0.75, 0.35),
            },
            gap: GapTheme {
                normal: Size::all(Val::Px(15.0)),
                large: Size::all(Val::Px(25.0)),
            },
            padding: PaddingTheme {
                normal: UiRect::all(Val::Px(10.0)),
                global: UiRect::all(Val::Px(20.0)),
            },
            background_color: Color::rgb(0.9, 0.9, 0.9),
            modal_color: Color::rgba(0.0, 0.0, 0.0, 0.0),
            panel_color: Color::rgb(0.8, 0.8, 0.8),
        }
    }
}

pub(crate) struct ButtonTheme {
    pub(crate) normal: Style,
    pub(crate) large: Style,
    pub(crate) square: Style,
    pub(crate) image_button: Style,
    pub(crate) image: Style,
    pub(crate) normal_text: TextStyle,
    pub(crate) large_text: TextStyle,
    pub(crate) square_text: TextStyle,
    pub(crate) normal_color: Color,
    pub(crate) hovered_color: Color,
    pub(crate) pressed_color: Color,
    pub(crate) hovered_pressed_color: Color,
}

pub(crate) struct LabelTheme {
    pub(crate) normal: TextStyle,
    pub(crate) large: TextStyle,
}

pub(crate) struct TextEditTheme {
    pub(crate) style: Style,
    pub(crate) text: TextStyle,
    pub(crate) normal_color: Color,
    pub(crate) hovered_color: Color,
    pub(crate) active_color: Color,
    pub(crate) hovered_active_color: Color,
}

pub(crate) struct CheckboxTheme {
    pub(crate) node: Style,
    pub(crate) button: Style,
    pub(crate) tick: Style,
    pub(crate) tick_color: Color,
}

pub(crate) struct GapTheme {
    pub(crate) normal: Size,
    pub(crate) large: Size,
}

pub(crate) struct PaddingTheme {
    pub(crate) normal: UiRect,
    pub(crate) global: UiRect,
}
