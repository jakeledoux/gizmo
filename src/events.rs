use bevy::prelude::*;

use crate::{
    Battle, Character, GameState, ItemManager, SceneBookmark, SceneCommands, SceneId, SceneManager,
    ScenePlayer, StateManager, components::*, utils,
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
        if end_scene_events.len() > 1 {
            warn!("more than one end scene event is queued")
        }
        if end_scene_events.count() > 0 {
            commands.remove_resource::<ScenePlayer>();
            assert!(matches!(
                state_manager.pop(&mut commands),
                Some(GameState::Dialogue)
            ));
        }
    }
}

#[derive(Event)]
pub struct SceneCommandsEvent(pub SceneBookmark, pub SceneCommands);

impl SceneCommandsEvent {
    pub fn handler(
        scene_player: Option<ResMut<ScenePlayer>>,
        mut scene_manager: ResMut<SceneManager>,
        mut scene_commands_events: EventReader<SceneCommandsEvent>,
        mut start_battle_event: EventWriter<StartBattleEvent>,
    ) {
        let Some(mut scene_player) = scene_player else {
            // if no scene is currently playing then we shouldn't have any events to handle.
            //
            // there could potentially be an issue here if the scene is exited
            // before the final commands are executed. if that happens then this
            // handler should be set to run before the sceen exit handler.
            assert_eq!(scene_commands_events.len(), 0);
            return;
        };

        for SceneCommandsEvent(bookmark, commands) in scene_commands_events.read() {
            // TODO: I'd like to avoid cloning these
            scene_player.execute(
                bookmark.to_owned(),
                commands.to_owned(),
                &mut scene_manager,
                &mut start_battle_event,
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
        for SpawnNpcEvent(id, character) in spawn_npc_events.read() {
            if utils::spawn_npc(&mut commands, npc_query, id.to_owned(), character.clone()) {
                info!("spawned NPC: {id:?}");
            } else {
                info!("skipped spawning NPC: {id:?}");
            }
        }
    }
}
