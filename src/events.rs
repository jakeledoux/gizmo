use bevy::prelude::*;

use crate::components::*;

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
    ) {
        for &AttackEvent { attacker, victim } in attack_events.read() {
            let [attacker, victim] = query.get_many([attacker, victim]).unwrap();
            info!("{} attacked {}!", attacker.1, victim.1);

            damage_event.write(DamageEvent {
                victim: victim.0,
                damage: 10.0,
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
            match victim.1.apply_damage(damage) {
                LifeStatus::Alive => {
                    info!("{} health is now: {}", *victim.1, victim.1.health())
                }
                LifeStatus::Dead => {
                    death_event.write(DeathEvent(victim.0));
                }
            };
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
            info!("{} has died", victim);
            commands.entity(victim_id).despawn();
        }
    }
}
