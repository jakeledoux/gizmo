use bevy::prelude::*;

use crate::components::*;
use crate::events::*;

pub fn debug_attack(
    player_query: Query<Entity, With<Player>>,
    npc_query: Query<Entity, With<Npc>>,
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut attack_event: EventWriter<AttackEvent>,
) {
    let (Ok(player), Ok(npc)) = (player_query.single(), npc_query.single()) else {
        return;
    };

    if keyboard_input.just_pressed(KeyCode::Space) {
        attack_event.write(AttackEvent {
            attacker: player,
            victim: npc,
        });
    }
}

pub fn debug_show_all_entities(
    query: Query<&RpgEntity>,
    keyboard_input: Res<ButtonInput<KeyCode>>,
) {
    if keyboard_input.just_pressed(KeyCode::Backspace) {
        for rpg_entity in query.iter() {
            info!("{rpg_entity} still exists");
        }
    }
}

pub fn exit_on_esc(keyboard_input: Res<ButtonInput<KeyCode>>, mut exit: EventWriter<AppExit>) {
    if keyboard_input.just_pressed(KeyCode::Escape) {
        exit.write(AppExit::Success);
    }
}
