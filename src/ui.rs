use bevy::prelude::EventWriter;
use bevy_egui::{
    EguiContexts,
    egui::{
        self, Align2, CollapsingHeader, Color32, Frame, Label, Margin, RichText, ScrollArea,
        SelectableLabel, Stroke, TextEdit, TextStyle, Ui, Widget, Window,
    },
};

use crate::{
    PlaySceneEvent, SceneCommandsEvent, SceneManager, ScenePlayer, ScenePlayerInput,
    UiScenePart,
};

pub fn dialogue_ui(
    mut contexts: EguiContexts,
    scene_player: &mut ScenePlayer,
    scene_manager: &mut SceneManager,
    scene_commands_events: &mut EventWriter<SceneCommandsEvent>,
) -> Option<ScenePlayerInput> {
    let mut scene_player_input = None;
    let Some(UiScenePart { line, responses }) =
        scene_player.get_current(scene_manager, scene_commands_events)
    else {
        return Some(ScenePlayerInput::SelectCurrent);
    };

    Window::new("Dialogue")
        .collapsible(false)
        .anchor(Align2::CENTER_CENTER, [0.0, 0.0])
        .frame(
            Frame::new()
                .fill(Color32::BLUE)
                .stroke(Stroke {
                    color: Color32::DARK_BLUE,
                    width: 10.0,
                })
                .inner_margin(Margin::same(10)),
        )
        .show(contexts.ctx_mut(), |ui| {
            ui.vertical(|ui| {
                // speaker row
                ui.horizontal(|ui| {
                    // left side: name and dialogue
                    ui.vertical(|ui| {
                        ui.label(
                            // TODO: query character name instead of showing ID
                            RichText::new(line.from.to_string())
                                .text_style(TextStyle::Heading)
                                .color(Color32::WHITE),
                        );

                        Frame::dark_canvas(ui.style()).show(ui, |ui| {
                            // TODO: apply text formatting
                            ui.label(&line.text);
                        })
                    });

                    // right: speaker image
                    let image_size = egui::vec2(100.0, 60.0);
                    // TODO: get character image from character query
                    ui.add_sized(image_size, Label::new("<image here>"))
                });

                // response row
                Frame::dark_canvas(ui.style()).show(ui, |ui| {
                    ScrollArea::vertical().show(ui, |ui| match responses {
                        Some(responses) if !responses.is_empty() => {
                            responses.iter().enumerate().for_each(|(i, response)| {
                                let selected = i == scene_player.highlighted_response();
                                if response_button(ui, &response.text, selected) {
                                    scene_player_input = Some(ScenePlayerInput::Select(i));
                                }
                            })
                        }
                        _ => {
                            if response_button(ui, "<continue>", true) {
                                scene_player_input = Some(ScenePlayerInput::SelectCurrent);
                            };
                        }
                    })
                })
            })
        });

    scene_player_input
}

fn response_button(ui: &mut Ui, text: &str, selected: bool) -> bool {
    ui.add_sized(
        [ui.available_width(), 24.0],
        SelectableLabel::new(selected, text),
    )
    .clicked()
}

pub fn map_ui(
    mut contexts: EguiContexts,
    play_scene_event: &mut EventWriter<PlaySceneEvent>,
    debug_new_scene_id: &mut String,
) {
    Window::new("Debug Panel")
        .anchor(Align2::CENTER_CENTER, [0.0, 0.0])
        .show(contexts.ctx_mut(), |ui| {
            CollapsingHeader::new("Play Scene")
                .default_open(true)
                .show(ui, |ui| {
                    ui.horizontal(|ui| {
                        TextEdit::singleline(debug_new_scene_id)
                            .hint_text("scene id")
                            .ui(ui);
                        if ui.button("play").clicked() {
                            play_scene_event
                                .write(PlaySceneEvent(debug_new_scene_id.to_owned().into()));
                        }
                    })
                })
        });
}
