use bevy::prelude::*;

use crate::{
    Battle, Character, CharacterUpdate, GameState, ItemManager, SceneBookmark, SceneId,
    SceneManager, ScenePlayer, StateManager, StaticCommands, components::*, utils,
};

#[derive(Event)]
pub struct AttackEvent {
    pub attacker: Entity,
    pub victim: Entity,
}

impl AttackEvent {
    pub fn handler(
        query: Query<(Entity, &RpgEntity)>,
        mut attack_events: EventReader<AttackEvent>,
        mut damage_event: EventWriter<DamageEvent>,
        item_manager: Res<ItemManager>,
    ) {
        for &AttackEvent { attacker, victim } in attack_events.read() {
            let [attacker, victim] = query.get_many([attacker, victim]).unwrap();
            let damage = attacker.1.attack_damage(&item_manager);
            info!(
                "{:?} attacked {:?} for {damage:?} damage",
                attacker.1.name(),
                victim.1.name()
            );

            damage_event.write(DamageEvent {
                victim: victim.0,
                damage,
            });
        }
    }
}

#[derive(Event)]
pub struct DamageEvent {
    pub victim: Entity,
    pub damage: f32,
}

impl DamageEvent {
    pub fn handler(
        mut query: Query<(Entity, &mut RpgEntity)>,
        mut damage_events: EventReader<DamageEvent>,
        mut death_event: EventWriter<DeathEvent>,
    ) {
        for &DamageEvent { victim, damage } in damage_events.read() {
            let mut victim = query.get_mut(victim).unwrap();
            let DamageResult {
                reduced_damage,
                life_status,
            } = victim.1.apply_damage(damage);
            info!(
                "{:?} received {damage:?} (reduced: {reduced_damage:?}) damage, health is now: {:?}",
                victim.1.name(),
                victim.1.health()
            );
            if life_status.is_dead() {
                death_event.write(DeathEvent(victim.0));
            }
        }
    }
}

#[derive(Event)]
pub struct DeathEvent(pub Entity);

impl DeathEvent {
    pub fn handler(query: Query<&RpgEntity>, mut death_events: EventReader<DeathEvent>) {
        for &DeathEvent(entity) in death_events.read() {
            let victim = query.get(entity).unwrap();
            info!("{:?} has died", victim.name());
        }
    }
}

#[derive(Event)]
pub struct PlaySceneEvent(pub SceneId);

impl PlaySceneEvent {
    pub fn handler(
        mut commands: Commands,
        scene_manager: Res<SceneManager>,
        mut play_scene_events: EventReader<PlaySceneEvent>,
        mut spawn_npc_event: EventWriter<SpawnNpcEvent>,
        mut state_manager: ResMut<StateManager>,
    ) {
        let play_scene_events = play_scene_events.read();
        if play_scene_events.len() > 1 {
            warn!("more than one play scene event is queued")
        }
        if let Some(play_scene_event) = play_scene_events.last() {
            if let Some(scene_player) =
                scene_manager.play_scene(play_scene_event.0.clone(), &mut spawn_npc_event)
            {
                info!("playing scene: {:?}", play_scene_event.0);
                commands.insert_resource(scene_player);
                state_manager.push(&mut commands, GameState::Dialogue);
            } else {
                warn!("was not able to play scene: {:?}", play_scene_event.0)
            }
        }
    }
}

#[derive(Event)]
pub struct EndSceneEvent;

impl EndSceneEvent {
    pub fn handler(
        mut commands: Commands,
        mut end_scene_events: EventReader<EndSceneEvent>,
        mut state_manager: ResMut<StateManager>,
    ) {
        let end_scene_events = end_scene_events.read();
        if end_scene_events.len() > 0 {
            if end_scene_events.len() > 1 {
                warn!("more than one end scene event is queued")
            }
            assert!(matches!(
                state_manager.pop(&mut commands),
                Some(GameState::Dialogue)
            ));
            commands.remove_resource::<ScenePlayer>();
        }
    }
}

#[derive(Event)]
pub struct StaticCommandsEvent(pub SceneBookmark, pub StaticCommands);

impl StaticCommandsEvent {
    pub fn handler(
        scene_player: Option<ResMut<ScenePlayer>>,
        mut scene_manager: ResMut<SceneManager>,
        mut static_commands_events: EventReader<StaticCommandsEvent>,
        mut start_battle_event: EventWriter<StartBattleEvent>,
        mut update_npc_event: EventWriter<UpdateNpcEvent>,
    ) {
        let Some(mut scene_player) = scene_player else {
            // if no scene is currently playing then we shouldn't have any events to handle.
            //
            // there could potentially be an issue here if the scene is exited
            // before the final commands are executed. if that happens then this
            // handler should be set to run before the screen exit handler.
            assert_eq!(static_commands_events.len(), 0);
            return;
        };

        for StaticCommandsEvent(bookmark, commands) in static_commands_events.read() {
            // TODO: I'd like to avoid cloning these
            scene_player.execute(
                bookmark.to_owned(),
                commands.to_owned(),
                &mut scene_manager,
                &mut start_battle_event,
                &mut update_npc_event,
            );
        }
    }
}

#[derive(Event)]
pub struct StartBattleEvent(pub NpcId);

impl StartBattleEvent {
    pub fn handler(
        mut commands: Commands,
        mut start_battle_events: EventReader<StartBattleEvent>,
        mut state_manager: ResMut<StateManager>,
        npc_query: Query<(Entity, &Npc)>,
    ) {
        let start_battle_events = start_battle_events.read();
        if start_battle_events.len() > 1 {
            warn!("more than one start battle event is queued")
        }
        if let Some(StartBattleEvent(npc_id)) = start_battle_events.last() {
            let Some((entity, _npc)) = npc_query.iter().find(|(_entity, npc)| &npc.id == npc_id)
            else {
                error!("cannot start battle. no such NPC with id: {npc_id}");
                return;
            };

            info!("starting battle with: {npc_id}");
            state_manager.push(&mut commands, GameState::Battle);
            commands.insert_resource(Battle(entity));
        }
    }
}

#[derive(Event)]
pub struct EndBattleEvent;

impl EndBattleEvent {
    pub fn handler(
        mut commands: Commands,
        mut end_battle_events: EventReader<EndBattleEvent>,
        mut state_manager: ResMut<StateManager>,
    ) {
        let end_battle_events = end_battle_events.read();
        if end_battle_events.len() > 1 {
            warn!("more than one end battle event is queued")
        }
        if end_battle_events.count() > 0 {
            info!("ending battle");
            assert!(matches!(
                state_manager.pop(&mut commands),
                Some(GameState::Battle)
            ));
        }
    }
}

#[derive(Event)]
pub struct SpawnNpcEvent(pub NpcId, pub Character);

impl SpawnNpcEvent {
    pub fn handler(
        mut commands: Commands,
        npc_query: Query<&Npc>,
        mut spawn_npc_events: EventReader<SpawnNpcEvent>,
    ) {
        for SpawnNpcEvent(npc_id, character) in spawn_npc_events.read() {
            if utils::spawn_npc(
                &mut commands,
                npc_query,
                npc_id.to_owned(),
                character.clone(),
            ) {
                info!("spawned NPC: {npc_id:?}");
            } else {
                info!("skipped spawning NPC: {npc_id:?}");
            }
        }
    }
}

#[derive(Event)]
pub struct UpdateNpcEvent(pub NpcId, pub CharacterUpdate);

impl UpdateNpcEvent {
    pub fn handler(
        mut mutable_npc_query: Query<(&mut Npc, &mut RpgEntity)>,
        mut update_npc_events: EventReader<UpdateNpcEvent>,
    ) {
        for UpdateNpcEvent(npc_id, character_update) in update_npc_events.read() {
            if let Some((mut npc, mut rpg_entity)) = mutable_npc_query
                .iter_mut()
                .find(|(npc, _rpg_entity)| &npc.id == npc_id)
            {
                info!("updating: {npc_id:?} ...");
                if let Some(new_image) = &character_update.image {
                    info!("    updated image to: {new_image:?}");
                    npc.image.0 = new_image.clone();
                }
                if let Some(new_name) = &character_update.name {
                    info!("    updated name to: {new_name:?}");
                    rpg_entity.set_name(new_name.clone());
                }
                if let Some(new_voice) = &character_update.voice {
                    info!("    updated voice to: {new_voice:?}");
                    npc.voice.0 = new_voice.clone();
                }
                info!("...done.");
            } else {
                error!("unable to update NPC: Could not find NPC with ID: {npc_id:?}")
            }
        }
    }
}
