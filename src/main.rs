use bevy::{image::ImageSamplerDescriptor, prelude::*};

enum LifeStatus {
    Alive,
    Dead,
}

#[derive(Component)]
struct Player;

#[derive(Component)]
struct Npc;

#[derive(Component)]
struct RpgEntity {
    name: &'static str,
    damage: f32,
}

impl RpgEntity {
    pub fn new(name: &'static str) -> Self {
        Self { name, damage: 0.0 }
    }

    pub fn apply_damage(&mut self, damage: f32) -> LifeStatus {
        self.damage += damage;

        if self.damage < self.max_health() {
            return LifeStatus::Alive;
        }
        self.damage = self.max_health();
        LifeStatus::Dead
    }

    pub fn max_health(&self) -> f32 {
        20.0
    }

    pub fn health(&self) -> f32 {
        self.max_health() - self.damage
    }
}

impl std::fmt::Display for RpgEntity {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.name)
    }
}

#[derive(Event)]
struct AttackEvent {
    attacker: Entity,
    victim: Entity,
}

impl AttackEvent {
    fn handler(
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
struct DamageEvent {
    victim: Entity,
    damage: f32,
}

impl DamageEvent {
    fn handler(
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
struct DeathEvent(Entity);

impl DeathEvent {
    fn handler(
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

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(ImagePlugin {
            default_sampler: ImageSamplerDescriptor::nearest(),
        }))
        .add_event::<AttackEvent>()
        .add_event::<DamageEvent>()
        .add_event::<DeathEvent>()
        .add_systems(Startup, setup)
        .add_systems(
            Update,
            (
                debug_attack_system,
                debug_show_all_entities_system,
                exit_on_esc,
            ),
        )
        // event handlers
        .add_systems(
            Update,
            (
                AttackEvent::handler,
                DamageEvent::handler,
                DeathEvent::handler,
            ),
        )
        .run();
}

fn setup(mut commands: Commands) {
    commands.spawn(Camera2d);

    commands.spawn((Player, RpgEntity::new("Jake")));
    commands.spawn((Npc, RpgEntity::new("Boba Fett")));
}

fn debug_attack_system(
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

fn debug_show_all_entities_system(
    query: Query<&RpgEntity>,
    keyboard_input: Res<ButtonInput<KeyCode>>,
) {
    if keyboard_input.just_pressed(KeyCode::Backspace) {
        for rpg_entity in query.iter() {
            info!("{rpg_entity} still exists");
        }
    }
}

fn exit_on_esc(keyboard_input: Res<ButtonInput<KeyCode>>, mut exit: EventWriter<AppExit>) {
    if keyboard_input.just_pressed(KeyCode::Escape) {
        exit.write(AppExit::Success);
    }
}
