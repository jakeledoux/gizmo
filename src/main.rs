#![allow(unused)]

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
    App::new()
        .insert_resource(ItemManager::new())
        .add_plugins(DefaultPlugins)
        .add_event::<AttackEvent>()
        .add_event::<DamageEvent>()
        .add_event::<DeathEvent>()
        .add_systems(Startup, setup)
        .add_systems(Update, (debug_attack, debug_show_all_entities, exit_on_esc))
        // event handlers
        .add_systems(
            Update,
            (
                AttackEvent::handler,
                DamageEvent::handler,
                DeathEvent::handler,
            ),
        )
        // .add_systems(Update, debug_quit_immediately)
        .run();

    Ok(())
}

fn setup(mut commands: Commands, mut item_manager: ResMut<ItemManager>) {
    commands.spawn(Camera2d);

    if let Err(e) = item_manager.load_items("skyrim.json") {
        warn!("could not load items: {e}")
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
