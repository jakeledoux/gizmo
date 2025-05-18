use bevy::{
    log::error,
    prelude::{Entity, EventWriter, Query, Res, With},
};
use bevy_egui::egui::{
    self, Align2, CollapsingHeader, Color32, Context, Frame, Label, Margin, RichText, ScrollArea,
    SelectableLabel, Stroke, TextEdit, TextStyle, Ui, Widget, Window,
};

use crate::{
    AttackEvent, Battle, EndBattleEvent, Npc, PlaySceneEvent, Player, RpgEntity,
    SceneCommandsEvent, SceneManager, ScenePlayer, ScenePlayerInput, UiScenePart,
};

pub fn dialogue_ui(
    ctx: &mut Context,
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
        .show(ctx, |ui| {
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
                                let button = response_button(ui, &response.text, selected);
                                if button.clicked() {
                                    scene_player_input = Some(ScenePlayerInput::Select(i));
                                } else if button.hovered() {
                                    scene_player_input = Some(ScenePlayerInput::MoveTo(i))
                                }
                            })
                        }
                        _ => {
                            if response_button(ui, "<continue>", true).clicked() {
                                scene_player_input = Some(ScenePlayerInput::SelectCurrent);
                            };
                        }
                    })
                })
            })
        });

    scene_player_input
}

fn response_button(ui: &mut Ui, text: &str, selected: bool) -> egui::Response {
    ui.add_sized(
        [ui.available_width(), 24.0],
        SelectableLabel::new(selected, text),
    )
}

pub fn map_ui(
    ctx: &mut Context,
    play_scene_event: &mut EventWriter<PlaySceneEvent>,
    debug_new_scene_id: &mut String,
) {
    Window::new("Map Mode")
        .anchor(Align2::CENTER_CENTER, [0.0, 0.0])
        .show(ctx, |ui| {
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

pub fn battle_ui(
    ctx: &mut Context,
    player_query: &Query<Entity, With<Player>>,
    npc_query: &Query<(&Npc, &RpgEntity)>,
    battle: &Res<Battle>,
    attack_event: &mut EventWriter<AttackEvent>,
    end_battle_event: &mut EventWriter<EndBattleEvent>,
) {
    let opponent_entity = battle.0;
    let Ok((_opponent_npc, opponent_rpg_entity)) = npc_query.get(opponent_entity) else {
        error!("failed to get opponent");
        return;
    };
    let player = player_query.single().expect("failed to get player!");

    Window::new("Battle")
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
        .show(ctx, |ui| {
            if opponent_rpg_entity.is_alive() {
                ui.label(format!(
                    "{} HP: {}/{}",
                    opponent_rpg_entity.name(),
                    opponent_rpg_entity.health(),
                    opponent_rpg_entity.max_health()
                ));
                if ui.button("attack").clicked() {
                    // TODO: proper battle gameplay
                    attack_event.write(AttackEvent {
                        attacker: player,
                        victim: opponent_entity,
                    });
                }
            } else {
                ui.label(format!("{} is dead.", opponent_rpg_entity.name()));
                if ui.button("continue").clicked() {
                    end_battle_event.write(EndBattleEvent);
                }
            }
        });
}

pub fn debug_ui(ctx: &mut Context, entity_query: Query<(&Npc, &RpgEntity)>) {
    Window::new("Debug Panel").show(ctx, |ui| {
        CollapsingHeader::new("NPCs")
            .default_open(true)
            .show(ui, |ui| {
                entity_query.iter().for_each(|(npc, rpg_entity)| {
                    CollapsingHeader::new(format!(
                        "{}{} ({})",
                        if rpg_entity.is_dead() { "[DEAD] " } else { "" },
                        rpg_entity.name(),
                        npc.id,
                    ))
                    .show(ui, |ui| {
                        ui.label(format!("damage: {}", rpg_entity.damage()));
                        ui.label(format!("max health: {}", rpg_entity.max_health()));
                    });
                })
            })
    });
}
