#![allow(unused)]
#![warn(unused_mut, unused_variables, unused_imports)]

mod components;
mod events;
mod items;
mod maps;
mod scenes;
mod static_commands;
mod systems;
mod types;
mod ui;
mod utils;

use std::path::Path;

use bevy::prelude::*;
use bevy_egui::{EguiContextPass, EguiContexts, EguiPlugin};

pub use components::*;
pub use events::*;
pub use items::*;
pub use maps::*;
pub use scenes::*;
pub use static_commands::*;
pub use systems::*;
pub use types::*;
pub use ui::*;
pub use utils::*;

// TODO: use bevy asset loader somehow
#[cfg(debug_assertions)]
const ASSETS_PATH: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/assets");
#[cfg(not(debug_assertions))]
const ASSETS_PATH: &str = "assets";

#[cfg(debug_assertions)]
pub const DEBUG: bool = true;
#[cfg(not(debug_assertions))]
pub const DEBUG: bool = false;

#[derive(Resource, Debug, Clone, PartialEq, Eq, Hash)]
pub struct Battle(Entity);

#[derive(Resource, Debug, Clone, PartialEq, Eq, Hash, Default)]
pub struct StateManager(Vec<GameState>);

impl StateManager {
    fn new(state: GameState) -> Self {
        let mut stack = Vec::with_capacity(8);
        stack.push(state);
        Self(stack)
    }

    pub fn get(&self) -> Option<GameState> {
        self.0.last().copied()
    }

    pub fn push(&mut self, commands: &mut Commands, state: GameState) {
        commands.set_state(state);
        self.0.push(state)
    }

    pub fn pop(&mut self, commands: &mut Commands) -> Option<GameState> {
        let popped = self.0.pop();
        commands.set_state(self.get().expect("all states popped! oh no!"));
        popped
    }
}

// TODO: state should be a stack
#[derive(States, Debug, Clone, Copy, PartialEq, Eq, Hash)]
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
    let state_manager = StateManager::new(GameState::Map);
    let mut app = App::new();
    // TODO: make Manager structs support hot-reloading
    app.add_plugins((
        DefaultPlugins,
        EguiPlugin {
            enable_multipass_for_primary_context: true,
        },
    ))
    .insert_state(state_manager.get().expect("state exists."))
    .insert_resource(state_manager)
    .insert_resource(ItemManager::new())
    .insert_resource(MapManager::new())
    .insert_resource(SceneManager::new())
    .configure_sets(
        Update,
        (
            MapSet.run_if(in_state(GameState::Map)),
            DialogueSet.run_if(in_state(GameState::Dialogue)),
        ),
    )
    .add_systems(Startup, setup)
    .add_systems(EguiContextPass, ui_system)
    .add_systems(Update, exit_on_esc);

    register_events(&mut app);

    app.run();

    Ok(())
}

fn register_events(app: &mut App) {
    app.add_event::<AttackEvent>()
        .add_event::<DamageEvent>()
        .add_event::<DeathEvent>()
        .add_event::<PlaySceneEvent>()
        .add_event::<EndSceneEvent>()
        .add_event::<StaticCommandsEvent>()
        .add_event::<StartBattleEvent>()
        .add_event::<EndBattleEvent>()
        .add_event::<SpawnNpcEvent>()
        .add_event::<UpdateNpcEvent>()
        .add_systems(
            Update,
            (
                // RPG events
                AttackEvent::handler,
                DamageEvent::handler,
                DeathEvent::handler,
                // scene events
                PlaySceneEvent::handler,
                EndSceneEvent::handler,
                StaticCommandsEvent::handler.before(EndSceneEvent::handler),
                // battle events
                StartBattleEvent::handler,
                EndBattleEvent::handler,
                // meta events
                SpawnNpcEvent::handler,
                UpdateNpcEvent::handler,
            ),
        );
}

fn setup(
    mut commands: Commands,
    // server: ResMut<AssetServer>,
    mut item_manager: ResMut<ItemManager>,
    mut map_manager: ResMut<MapManager>,
    mut scene_manager: ResMut<SceneManager>,
    mut state_manager: ResMut<StateManager>,
) {
    commands.spawn(Camera2d);
    // value for text input for selecting scenes
    commands.insert_resource(DebugPlaySceneId::default());
    // start game in map mode
    state_manager.push(&mut commands, GameState::Map);

    // load assets
    if let Err(e) = item_manager.load_folder(Path::new(ASSETS_PATH).join("items")) {
        warn!("could not load items: {e}")
    };
    if let Err(e) = map_manager.load_folder(Path::new(ASSETS_PATH).join("maps")) {
        warn!("could not load maps: {e}")
    };
    if let Err(e) = scene_manager.load_folder(Path::new(ASSETS_PATH).join("scenes")) {
        warn!("could not load scene: {e}")
    };
    // server.load_folder(Path::new(ASSETS_PATH).join("images"));

    // spawn player
    utils::spawn_player(&mut commands, &item_manager, "Jake", &["dragonbone-sword"]);
}

#[derive(Resource, Default)]
struct DebugPlaySceneId(String);

fn debug_quit_immediately(mut exit_event: EventWriter<AppExit>) {
    exit_event.write(AppExit::Success);
}

#[allow(clippy::too_many_arguments)]
fn ui_system(
    mut contexts: EguiContexts,
    game_state: Res<State<GameState>>,
    mut scene_manager: ResMut<SceneManager>,
    item_manager: Res<ItemManager>,
    mut scene_player: Option<ResMut<ScenePlayer>>,
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut play_scene_event: EventWriter<PlaySceneEvent>,
    mut end_scene_event: EventWriter<EndSceneEvent>,
    mut static_command_event: EventWriter<StaticCommandsEvent>,
    mut attack_event: EventWriter<AttackEvent>,
    mut end_battle_event: EventWriter<EndBattleEvent>,
    mut debug_new_scene_id: ResMut<DebugPlaySceneId>,
    debug_player_query: Query<&RpgEntity, With<Player>>,
    npc_query: Query<(&Npc, &RpgEntity)>,
    battle_player_query: Query<Entity, With<Player>>,
    battle: Option<Res<Battle>>,
) {
    let play_scene_event = &mut play_scene_event;
    let end_scene_event = &mut end_scene_event;
    let static_command_event = &mut static_command_event;

    if DEBUG {
        debug_ui(
            contexts.ctx_mut(),
            debug_player_query,
            npc_query,
            &scene_manager,
            &item_manager,
        );
    }

    match **game_state {
        GameState::Map => {
            map_ui(
                contexts.ctx_mut(),
                play_scene_event,
                &mut debug_new_scene_id.0,
            );
        }
        GameState::Dialogue => {
            let Some(ref mut scene_player) = scene_player else {
                return;
            };

            if let Some(input) = dialogue_ui(
                contexts.ctx_mut(),
                scene_player,
                &mut scene_manager,
                static_command_event,
                npc_query,
            ) {
                scene_player.input(
                    input,
                    &mut scene_manager,
                    end_scene_event,
                    static_command_event,
                )
            }

            if keyboard_input.just_pressed(KeyCode::KeyW)
                || keyboard_input.just_pressed(KeyCode::ArrowUp)
            {
                scene_player.input(
                    ScenePlayerInput::MoveUp,
                    &mut scene_manager,
                    end_scene_event,
                    static_command_event,
                )
            }
            if keyboard_input.just_pressed(KeyCode::KeyS)
                || keyboard_input.just_pressed(KeyCode::ArrowDown)
            {
                scene_player.input(
                    ScenePlayerInput::MoveDown,
                    &mut scene_manager,
                    end_scene_event,
                    static_command_event,
                )
            }
            if keyboard_input.just_pressed(KeyCode::KeyE)
                || keyboard_input.just_pressed(KeyCode::Enter)
            {
                scene_player.input(
                    ScenePlayerInput::SelectCurrent,
                    &mut scene_manager,
                    end_scene_event,
                    static_command_event,
                )
            }
        }
        GameState::Battle => {
            battle_ui(
                contexts.ctx_mut(),
                &battle_player_query,
                &npc_query,
                &battle.expect("gamestate is battle but there's no battle!"),
                &mut attack_event,
                &mut end_battle_event,
            );
        }
    }
}
