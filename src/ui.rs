use bevy::{
    input::ButtonInput,
    prelude::{EventWriter, KeyCode, Res},
};
use bevy_egui::{
    EguiContexts, EguiInput,
    egui::{
        self, Color32, Frame, Label, Margin, RichText, ScrollArea, SelectableLabel, Stroke,
        TextStyle, Ui, Widget, Window, vec2,
    },
};

use crate::{PlaySceneEvent, SceneManager, ScenePlayer, UiScenePart};

pub fn dialogue_ui(
    mut contexts: EguiContexts,
    scene_player: &mut ScenePlayer,
    scene_manager: &SceneManager,
) {
    let UiScenePart { line, responses } = scene_player.get_current(scene_manager);

    Window::new("Dialogue")
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
                    ScrollArea::vertical().show(ui, |ui| {
                        if let Some(responses) = responses {
                            responses.iter().enumerate().for_each(|(i, response)| {
                                let selected = i == scene_player.highlighted_response();
                                response_button(ui, &response.text, selected)
                            })
                        } else {
                            response_button(ui, "<continue>", true)
                        }
                    })
                })
            })
        });
}

fn response_button(ui: &mut Ui, text: &str, selected: bool) {
    ui.add_sized(
        [ui.available_width(), 24.0],
        SelectableLabel::new(selected, text),
    );
}

pub fn map_ui(
    mut contexts: EguiContexts,
    play_scene_event: &mut EventWriter<PlaySceneEvent>,
    debug_new_scene_id: &mut String,
) {
    Window::new("Dialogue").show(contexts.ctx_mut(), |ui| {
        ui.label("Map UI");
        Frame::new().show(ui, |ui| {
            ui.label("Debug: Start Scene");
            ui.horizontal(|ui| {
                ui.text_edit_singleline(debug_new_scene_id);
                if ui.button("start scene").clicked() {
                    play_scene_event.write(PlaySceneEvent(debug_new_scene_id.to_owned().into()));
                }
            })
        })
    });
}
