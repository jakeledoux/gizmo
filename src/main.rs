#![allow(unused)]

mod components;
mod events;
mod items;
mod scenes;
mod systems;
mod ui;
mod utils;

use bevy::{app::ScheduleRunnerPlugin, input::InputPlugin, prelude::*, state::app::StatesPlugin};
use bevy_egui::{EguiContextPass, EguiContexts, EguiPlugin};

use components::*;
use events::*;
use items::*;
use scenes::*;
use systems::*;
use ui::*;
use utils::*;

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
    let frame_time = std::time::Duration::from_secs_f32(1.0 / 60.0);
    App::new()
        // TODO: make Manager structs support hot-reloading
        .add_plugins((
            DefaultPlugins,
            EguiPlugin {
                enable_multipass_for_primary_context: true,
            },
        ))
        .insert_state(GameState::Dialogue)
        .insert_resource(ItemManager::new())
        .insert_resource(SceneManager::new())
        .add_event::<AttackEvent>()
        .add_event::<DamageEvent>()
        .add_event::<DeathEvent>()
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
                AttackEvent::handler,
                DamageEvent::handler,
                DeathEvent::handler,
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

    if let Err(e) = item_manager.load_items("skyrim.json") {
        warn!("could not load items: {e}")
    };

    if let Err(e) = scene_manager.load_scene("mike.json") {
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
}

fn debug_quit_immediately(mut exit_event: EventWriter<AppExit>) {
    exit_event.write(AppExit::Success);
}

fn ui_system(mut contexts: EguiContexts, game_state: Res<State<GameState>>) {
    match **game_state {
        GameState::Map => todo!(),
        GameState::Dialogue => {
            dialogue_ui(contexts);
        }
        GameState::Battle => todo!(),
    }
}
