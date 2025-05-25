use bevy::prelude::*;
use bevy_egui::{
    EguiContexts,
    egui::{
        self, Align2, CollapsingHeader, Color32, Frame, Label, Margin, RichText, ScrollArea,
        SelectableLabel, Stroke, TextEdit, TextStyle, Ui, Widget, Window,
    },
};

use crate::{
    AttackEvent, Battle, DebugPlaySceneId, EndBattleEvent, EndSceneEvent, ItemManager, Npc,
    PlaySceneEvent, Player, RpgEntity, SceneManager, ScenePlayer, ScenePlayerInput,
    StaticCommandsEvent, UiScenePart,
};

pub fn dialogue_ui(
    mut contexts: EguiContexts,
    mut scene_player: ResMut<ScenePlayer>,
    mut scene_manager: ResMut<SceneManager>,
    mut scene_commands_event: EventWriter<StaticCommandsEvent>,
    mut end_scene_event: EventWriter<EndSceneEvent>,
    npc_query: Query<(&Npc, &RpgEntity)>,
) {
    let ctx = contexts.ctx_mut();

    let mut scene_player_input = None;
    let Some(UiScenePart { line, responses }) =
        scene_player.get_current(&scene_manager, &mut scene_commands_event)
    else {
        scene_player.input(
            ScenePlayerInput::SelectCurrent,
            &mut scene_manager,
            &mut end_scene_event,
            &mut scene_commands_event,
        );
        return;
    };

    let fallback_name = &line.from.0;
    let speaker_rpg_entity = if let Some((_npc, rpg_entity)) = npc_query
        .iter()
        .find(|(npc, _rpg_entity)| npc.id == line.from)
    {
        Some(rpg_entity)
    } else {
        None
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
                            RichText::new(
                                speaker_rpg_entity
                                    .map(|e| e.name())
                                    .unwrap_or(fallback_name),
                            )
                            .text_style(TextStyle::Heading)
                            .color(Color32::WHITE),
                        );

                        Frame::dark_canvas(ui.style()).show(ui, |ui| {
                            // TODO: apply text formatting
                            ui.label(&line.text);
                        })
                    });

                    if let Some(_speaker_rpg_entity) = speaker_rpg_entity {
                        // right: speaker image
                        let image_size = egui::vec2(100.0, 60.0);
                        // TODO: get character image from character query
                        ui.add_sized(image_size, Label::new("<image here>"));
                    }
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

    if let Some(input) = scene_player_input {
        scene_player.input(
            input,
            &mut scene_manager,
            &mut end_scene_event,
            &mut scene_commands_event,
        );
    }
}

pub fn dialogue_ui_input(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut scene_manager: ResMut<SceneManager>,
    mut scene_player: Option<ResMut<ScenePlayer>>,
    mut end_scene_event: EventWriter<EndSceneEvent>,
    mut static_command_event: EventWriter<StaticCommandsEvent>,
) {
    let Some(ref mut scene_player) = scene_player else {
        return;
    };

    if keyboard_input.just_pressed(KeyCode::KeyW) || keyboard_input.just_pressed(KeyCode::ArrowUp) {
        scene_player.input(
            ScenePlayerInput::MoveUp,
            &mut scene_manager,
            &mut end_scene_event,
            &mut static_command_event,
        )
    }
    if keyboard_input.just_pressed(KeyCode::KeyS) || keyboard_input.just_pressed(KeyCode::ArrowDown)
    {
        scene_player.input(
            ScenePlayerInput::MoveDown,
            &mut scene_manager,
            &mut end_scene_event,
            &mut static_command_event,
        )
    }
    if keyboard_input.just_pressed(KeyCode::KeyE) || keyboard_input.just_pressed(KeyCode::Enter) {
        scene_player.input(
            ScenePlayerInput::SelectCurrent,
            &mut scene_manager,
            &mut end_scene_event,
            &mut static_command_event,
        )
    }
}

fn response_button(ui: &mut Ui, text: &str, selected: bool) -> egui::Response {
    ui.add_sized(
        [ui.available_width(), 24.0],
        SelectableLabel::new(selected, text),
    )
}

pub fn map_ui(
    mut contexts: EguiContexts,
    mut play_scene_event: EventWriter<PlaySceneEvent>,
    mut debug_new_scene_id: ResMut<DebugPlaySceneId>,
    // mut user_textures: ResMut<EguiUserTextures>,
    // pixels: Res<PixelBuffer>,
) {
    let ctx = contexts.ctx_mut();

    // let texture_id = user_textures
    //     .image_id(&pixels.handle)
    //     .unwrap_or_else(|| user_textures.add_image(pixels.handle.clone()));

    Window::new("Map Mode")
        .anchor(Align2::CENTER_CENTER, [0.0, 0.0])
        .show(ctx, |ui| {
            CollapsingHeader::new("Play Scene")
                .default_open(true)
                .show(ui, |ui| {
                    ui.horizontal(|ui| {
                        TextEdit::singleline(&mut debug_new_scene_id.0)
                            .hint_text("scene id")
                            .ui(ui);
                        if ui.button("play").clicked() {
                            play_scene_event
                                .write(PlaySceneEvent(debug_new_scene_id.0.clone().into()));
                        }
                    })
                });
            // ui.image(SizedTexture::new(texture_id, egui::Vec2::new(28.0, 28.0)));
        });
}

pub fn battle_ui(
    mut contexts: EguiContexts,
    player_query: Query<Entity, With<Player>>,
    npc_query: Query<(&Npc, &RpgEntity)>,
    battle: Res<Battle>,
    mut attack_event: EventWriter<AttackEvent>,
    mut end_battle_event: EventWriter<EndBattleEvent>,
) {
    let ctx = contexts.ctx_mut();

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
    mut contexts: EguiContexts,
    player_query: Query<&RpgEntity, With<Player>>,
    entity_query: Query<(&Npc, &RpgEntity)>,
    scene_manager: Res<SceneManager>,
    item_manager: Res<ItemManager>,
) {
    let ctx = contexts.ctx_mut();
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
