#![allow(unused)]
#![warn(unused_mut, unused_variables, unused_imports)]

mod components;
mod events;
mod items;
mod maps;
mod pixels;
mod scenes;
mod static_commands;
mod systems;
mod types;
mod ui;
mod utils;

use std::path::Path;

use bevy::prelude::*;
use bevy_egui::{EguiContextPass, EguiPlugin};
use bevy_rand::prelude::*;

pub use components::*;
pub use events::*;
pub use items::*;
pub use maps::*;
pub use pixels::*;
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

#[derive(States, Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum GameState {
    Map,
    Dialogue,
    Battle,
}

pub type Rng<'w> = GlobalEntropy<'w, WyRand>;

fn main() -> anyhow::Result<()> {
    let state_manager = StateManager::new(GameState::Map);
    let mut app = App::new();
    // TODO: make Manager structs support hot-reloading
    app.add_plugins((
        DefaultPlugins,
        EguiPlugin {
            enable_multipass_for_primary_context: true,
        },
        EntropyPlugin::<WyRand>::default(),
    ))
    .insert_state(state_manager.get().expect("state exists."))
    .insert_resource(state_manager)
    .insert_resource(ItemManager::new())
    .insert_resource(MapManager::new())
    .insert_resource(SceneManager::new())
    .add_systems(Startup, (setup, setup_pixel_buffer))
    .add_systems(Update, exit_on_esc)
    .add_systems(Update, draw_random_pixels.run_if(in_state(GameState::Map)));

    register_events(&mut app);
    register_ui(&mut app);

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
            PostUpdate,
            (
                // RPG events
                AttackEvent::handler,
                DamageEvent::handler,
                DeathEvent::handler,
                // scene events
                PlaySceneEvent::handler,
                StaticCommandsEvent::handler,
                EndSceneEvent::handler,
                // battle events
                StartBattleEvent::handler,
                EndBattleEvent::handler,
                // meta events
                SpawnNpcEvent::handler,
                UpdateNpcEvent::handler,
            ),
        );
}

fn register_ui(app: &mut App) {
    app.add_systems(EguiContextPass, debug_ui.run_if(|| DEBUG))
        .add_systems(EguiContextPass, map_ui.run_if(in_state(GameState::Map)))
        .add_systems(
            EguiContextPass,
            (dialogue_ui, dialogue_ui_input).run_if(in_state(GameState::Dialogue)),
        )
        .add_systems(
            EguiContextPass,
            battle_ui.run_if(in_state(GameState::Battle)),
        );
}

fn setup(
    mut commands: Commands,
    // server: ResMut<AssetServer>,
    mut item_manager: ResMut<ItemManager>,
    mut map_manager: ResMut<MapManager>,
    mut scene_manager: ResMut<SceneManager>,
    mut state_manager: ResMut<StateManager>,
    npc_query: Query<&Npc>,
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

    utils::spawn_npc(
        &mut commands,
        npc_query,
        NpcId(String::from("narrator")),
        Character {
            name: String::from(""),
            ..default()
        },
    );
}

#[derive(Resource, Default)]
pub struct DebugPlaySceneId(String);

fn debug_quit_immediately(mut exit_event: EventWriter<AppExit>) {
    exit_event.write(AppExit::Success);
}
