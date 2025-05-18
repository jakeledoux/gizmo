use bevy::prelude::*;

use crate::{
    GameState, ItemManager, SceneBookmark, SceneCommands, SceneId, SceneManager, ScenePlayer,
    components::*,
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
    pub fn handler(
        query: Query<&RpgEntity>,
        mut commands: Commands,
        mut death_events: EventReader<DeathEvent>,
    ) {
        for &DeathEvent(victim_id) in death_events.read() {
            let victim = query.get(victim_id).unwrap();
            info!("{:?} has died", victim.name());
            commands.entity(victim_id).despawn();
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
    ) {
        let play_scene_events = play_scene_events.read();
        if play_scene_events.len() > 1 {
            warn!("more than one play scene event is queued")
        }
        if let Some(play_scene_event) = play_scene_events.last() {
            if let Some(scene) = scene_manager.play_scene(play_scene_event.0.clone()) {
                info!("playing scene: {:?}", play_scene_event.0);
                commands.insert_resource(scene);
                commands.set_state(GameState::Dialogue)
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
        scene_manager: Res<SceneManager>,
        mut end_scene_events: EventReader<EndSceneEvent>,
    ) {
        let end_scene_events = end_scene_events.read();
        if end_scene_events.len() > 1 {
            warn!("more than one end scene event is queued")
        }
        if end_scene_events.count() > 0 {
            commands.remove_resource::<ScenePlayer>();
            commands.set_state(GameState::Map);
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
            scene_player.execute(bookmark.to_owned(), commands.to_owned(), &mut scene_manager);
        }
    }
}
