use bevy::prelude::*;

use crate::{ItemManager, components::*};

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
