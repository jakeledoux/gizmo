mod components;
mod events;
mod items;
mod systems;

use bevy::prelude::*;

use components::*;
use events::*;
use items::*;
use systems::*;

fn main() -> anyhow::Result<()> {
    let mut item_manager: ItemManager = ItemManager::new();
    item_manager.load_items("skyrim.json")?;
    // App::new()
    //     .add_plugins(DefaultPlugins.set(ImagePlugin {
    //         default_sampler: ImageSamplerDescriptor::nearest(),
    //     }))
    //     .add_event::<AttackEvent>()
    //     .add_event::<DamageEvent>()
    //     .add_event::<DeathEvent>()
    //     .add_systems(Startup, setup)
    //     .add_systems(Update, (debug_attack, debug_show_all_entities, exit_on_esc))
    //     // event handlers
    //     .add_systems(
    //         Update,
    //         (
    //             AttackEvent::handler,
    //             DamageEvent::handler,
    //             DeathEvent::handler,
    //         ),
    //     )
    //     .run();

    Ok(())
}

fn setup(mut commands: Commands) {
    commands.spawn(Camera2d);

    commands.spawn((Player, RpgEntity::new("Jake")));
    commands.spawn((Npc, RpgEntity::new("Boba Fett")));
}
