use bevy::{
    prelude::TextBundle,
    text::{TextSection, TextStyle},
};

use crate::*;
#[derive(Component)]
pub struct GUIText;

#[derive(Bundle)]
pub struct GUIBundle {
    pub text_bundle: TextBundle,
    text: GUIText,
}

impl GUIBundle {
    pub fn new(font: Handle<Font>) -> Self {
        Self {
            text_bundle: TextBundle::from_sections([
                TextSection::new(
                    "Placing: ",
                    TextStyle {
                        font: font.clone(),
                        font_size: SCOREBOARD_FONT_SIZE,
                        color: TEXT_COLOR,
                    },
                ),
                TextSection::from_style(TextStyle {
                    font,
                    font_size: SCOREBOARD_FONT_SIZE,
                    color: SCORE_COLOR,
                }),
            ])
            .with_style(Style {
                position_type: PositionType::Absolute,
                position: UiRect {
                    bottom: SCOREBOARD_TEXT_PADDING,
                    left: SCOREBOARD_TEXT_PADDING,
                    ..default()
                },
                ..default()
            }),
            text: GUIText,
        }
    }
}

#[derive(Component)]
pub struct FPSText;

#[derive(Bundle)]
pub struct FPSBundle {
    text_bundle: TextBundle,
    text: FPSText,
}

impl FPSBundle {
    pub fn new(font: Handle<Font>) -> Self {
        Self {
            text_bundle: TextBundle::from_sections([
                TextSection::new(
                    "fps: ",
                    TextStyle {
                        font: font.clone(),
                        font_size: SCOREBOARD_FONT_SIZE,
                        color: TEXT_COLOR,
                    },
                ),
                TextSection::from_style(TextStyle {
                    font,
                    font_size: SCOREBOARD_FONT_SIZE,
                    color: SCORE_COLOR,
                }),
            ])
            .with_style(Style {
                position_type: PositionType::Absolute,
                position: UiRect {
                    top: SCOREBOARD_TEXT_PADDING,
                    right: SCOREBOARD_TEXT_PADDING,
                    ..default()
                },
                ..default()
            }),
            text: FPSText,
        }
    }
}
