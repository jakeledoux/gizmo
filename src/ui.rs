use bevy::{
    log::error,
    prelude::{Entity, EventWriter, Query, Res, With},
};
use bevy_egui::egui::{
    self, Align2, CollapsingHeader, Color32, Context, Frame, Label, Margin, RichText, ScrollArea,
    SelectableLabel, Stroke, TextEdit, TextStyle, Ui, Widget, Window,
};

use crate::{
    AttackEvent, Battle, EndBattleEvent, ItemManager, Npc, PlaySceneEvent, Player, RpgEntity,
    SceneManager, ScenePlayer, ScenePlayerInput, StaticCommandsEvent, UiScenePart,
};

pub fn dialogue_ui(
    ctx: &mut Context,
    scene_player: &mut ScenePlayer,
    scene_manager: &mut SceneManager,
    scene_commands_events: &mut EventWriter<StaticCommandsEvent>,
    npc_query: Query<(&Npc, &RpgEntity)>,
) -> Option<ScenePlayerInput> {
    let mut scene_player_input = None;
    let Some(UiScenePart { line, responses }) =
        scene_player.get_current(scene_manager, scene_commands_events)
    else {
        return Some(ScenePlayerInput::SelectCurrent);
    };

    let Some((_speaker_npc, speaker_rpg_entity)) = npc_query
        .iter()
        .find(|(npc, _rpg_entity)| npc.id == line.from)
    else {
        error!("failed to get speaker NPC!");
        return None;
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
                            RichText::new(speaker_rpg_entity.name())
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
                    ui.add_sized(image_size, Label::new("<image here>"));
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

pub fn debug_ui(
    ctx: &mut Context,
    player_query: Query<&RpgEntity, With<Player>>,
    entity_query: Query<(&Npc, &RpgEntity)>,
    scene_manager: &SceneManager,
    item_manager: &ItemManager,
) {
    let player = player_query.single().expect("player must exist.");

    Window::new("Debug Panel").show(ctx, |ui| {
        ScrollArea::vertical().show(ui, |ui| {
            CollapsingHeader::new("Player")
                .default_open(true)
                .show(ui, |ui| {
                    player.show(ui);
                });
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
                            rpg_entity.show(ui);
                        });
                    })
                });
            CollapsingHeader::new("Save Data")
                .default_open(true)
                .show(ui, |ui| {
                    CollapsingHeader::new("Variables")
                        .default_open(true)
                        .show(ui, |ui| {
                            for (k, v) in scene_manager.variables.iter() {
                                ui.label(format!("{k:?}: {v:?}"));
                            }
                        });
                    CollapsingHeader::new("Scene Entry Points")
                        .default_open(true)
                        .show(ui, |ui| {
                            for (k, v) in scene_manager.entries.iter() {
                                ui.label(format!("{:?}: {:?}", k.0, v.0));
                            }
                        });
                });
            CollapsingHeader::new("Loaded Resources")
                .default_open(true)
                .show(ui, |ui| {
                    CollapsingHeader::new("Scenes").show(ui, |ui| {
                        for scene_id in scene_manager.scenes.keys() {
                            ui.label(&scene_id.0);
                        }
                    });
                    CollapsingHeader::new("Items").show(ui, |ui| {
                        for (item_id, any_item) in item_manager.items.iter() {
                            CollapsingHeader::new(&item_id.0).show(ui, |ui| {
                                ui.label(format!("{any_item:#?}"));
                            });
                        }
                    });
                })
        })
    });
}

trait DebugUi {
    fn show(&self, ui: &mut Ui);
}

impl DebugUi for RpgEntity {
    fn show(&self, ui: &mut Ui) {
        ui.label(format!("name: {}", self.name()));
        ui.label(format!("hp: {}/{}", self.health(), self.max_health()));
        CollapsingHeader::new("Inventory").show(ui, |ui| {
            for item_instance in self.inventory.items.values() {
                ui.label(format!(
                    "{}{}",
                    if self.is_equipped(&item_instance.instance_id()) {
                        "[X] "
                    } else {
                        ""
                    },
                    item_instance.item_id()
                ));
            }
        });
    }
}
