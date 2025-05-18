#![allow(unused)]
#![warn(unused_mut, unused_variables, unused_imports)]

mod components;
mod events;
mod items;
mod scenes;
mod systems;
mod ui;
mod utils;

use std::path::Path;

use bevy::prelude::*;
use bevy_egui::{EguiContextPass, EguiContexts, EguiPlugin};

pub use components::*;
pub use events::*;
pub use items::*;
pub use scenes::*;
pub use systems::*;
pub use ui::*;
pub use utils::*;

// TODO: use bevy asset loader somehow
#[cfg(debug_assertions)]
const ASSETS_PATH: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/assets");
#[cfg(not(debug_assertions))]
const ASSETS_PATH: &str = "assets";

#[derive(States, Debug, Clone, PartialEq, Eq, Hash)]
pub enum GameState {
    Map,
    Dialogue,
    Battle,
}

#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
pub struct MapSet;

#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
pub struct DialogueSet;

fn main() -> anyhow::Result<()> {
    App::new()
        // TODO: make Manager structs support hot-reloading
        .add_plugins((
            DefaultPlugins,
            EguiPlugin {
                enable_multipass_for_primary_context: true,
            },
        ))
        .insert_state(GameState::Map)
        .insert_resource(ItemManager::new())
        .insert_resource(SceneManager::new())
        .add_event::<AttackEvent>()
        .add_event::<DamageEvent>()
        .add_event::<DeathEvent>()
        .add_event::<PlaySceneEvent>()
        .add_event::<EndSceneEvent>()
        .add_event::<SceneCommandsEvent>()
        .configure_sets(
            Update,
            (
                MapSet.run_if(in_state(GameState::Map)),
                DialogueSet.run_if(in_state(GameState::Dialogue)),
            ),
        )
        .add_systems(Startup, setup)
        .add_systems(
            Update,
            (debug_attack, debug_show_all_entities).in_set(MapSet),
        )
        // event handlers
        .add_systems(
            Update,
            (
                // RPG events
                AttackEvent::handler,
                DamageEvent::handler,
                DeathEvent::handler,
                // meta events
                PlaySceneEvent::handler,
                EndSceneEvent::handler,
                SceneCommandsEvent::handler.before(EndSceneEvent::handler),
            ),
        )
        // rendering
        .add_systems(EguiContextPass, ui_system)
        // debug
        // .add_systems(Update, debug_quit_immediately)
        .add_systems(Update, exit_on_esc)
        .run();

    Ok(())
}

fn setup(
    mut commands: Commands,
    mut item_manager: ResMut<ItemManager>,
    mut scene_manager: ResMut<SceneManager>,
) {
    commands.spawn(Camera2d);

    if let Err(e) = item_manager.load_folder(Path::new(ASSETS_PATH).join("items")) {
        warn!("could not load items: {e}")
    };

    if let Err(e) = scene_manager.load_folder(Path::new(ASSETS_PATH).join("scenes")) {
        warn!("could not load scene: {e}")
    };

    let mut jake = RpgEntity::new("Jake");
    if let Some(vampire_gloves) =
        ItemInstance::spawn(ItemId::new("dragonbone-sword"), &item_manager)
    {
        let instance_id = jake.inventory.insert(vampire_gloves);
        jake.equip(instance_id);
    }
    commands.spawn((Player, jake));
    commands.spawn((Npc, RpgEntity::new("Boba Fett")));

    commands.insert_resource(DebugPlaySceneId::default())
    // play_scene_events.write(PlaySceneEvent(SceneId::new("mike")));
}

#[derive(Resource, Default)]
struct DebugPlaySceneId(String);

fn debug_quit_immediately(mut exit_event: EventWriter<AppExit>) {
    exit_event.write(AppExit::Success);
}

fn ui_system(
    contexts: EguiContexts,
    game_state: Res<State<GameState>>,
    mut scene_manager: ResMut<SceneManager>,
    mut scene_player: Option<ResMut<ScenePlayer>>,
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut play_scene_event: EventWriter<PlaySceneEvent>,
    mut end_scene_event: EventWriter<EndSceneEvent>,
    mut scene_command_event: EventWriter<SceneCommandsEvent>,
    mut debug_new_scene_id: ResMut<DebugPlaySceneId>,
) {
    let play_scene_event = &mut play_scene_event;
    let end_scene_event = &mut end_scene_event;
    let scene_command_event = &mut scene_command_event;

    match **game_state {
        GameState::Map => {
            map_ui(contexts, play_scene_event, &mut debug_new_scene_id.0);
        }
        GameState::Dialogue => {
            let Some(ref mut scene_player) = scene_player else {
                return;
            };

            if let Some(input) = dialogue_ui(
                contexts,
                scene_player,
                &mut scene_manager,
                scene_command_event,
            ) {
                scene_player.input(
                    input,
                    &mut scene_manager,
                    end_scene_event,
                    scene_command_event,
                )
            }

            if keyboard_input.just_pressed(KeyCode::KeyW)
                || keyboard_input.just_pressed(KeyCode::ArrowUp)
            {
                scene_player.input(
                    ScenePlayerInput::MoveUp,
                    &mut scene_manager,
                    end_scene_event,
                    scene_command_event,
                )
            }
            if keyboard_input.just_pressed(KeyCode::KeyS)
                || keyboard_input.just_pressed(KeyCode::ArrowDown)
            {
                scene_player.input(
                    ScenePlayerInput::MoveDown,
                    &mut scene_manager,
                    end_scene_event,
                    scene_command_event,
                )
            }
            if keyboard_input.just_pressed(KeyCode::KeyE)
                || keyboard_input.just_pressed(KeyCode::Enter)
            {
                scene_player.input(
                    ScenePlayerInput::SelectCurrent,
                    &mut scene_manager,
                    end_scene_event,
                    scene_command_event,
                )
            }
        }
        GameState::Battle => todo!(),
    }
}
