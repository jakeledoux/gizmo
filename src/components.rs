use std::path::PathBuf;

use bevy::{
    log::{info, warn},
    platform::collections::HashMap,
    prelude::Component,
    reflect::Reflect,
};
use serde::Deserialize;

use crate::{ItemInstance, ItemInstanceId, ItemKind, ItemManager, utils::*};

#[derive(Clone, Copy, Reflect, Debug, Hash, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ArmorSlot {
    Head,
    Body,
    Feet,
    Hands,
}

#[derive(Default, Debug, Hash, PartialEq, Eq)]
pub struct ArmorSlots {
    head: Option<ItemInstanceId>,
    body: Option<ItemInstanceId>,
    feet: Option<ItemInstanceId>,
    hands: Option<ItemInstanceId>,
}

impl ArmorSlots {
    pub fn get_mut(&mut self, slot: ArmorSlot) -> &mut Option<ItemInstanceId> {
        match slot {
            ArmorSlot::Head => &mut self.head,
            ArmorSlot::Body => &mut self.body,
            ArmorSlot::Feet => &mut self.feet,
            ArmorSlot::Hands => &mut self.hands,
        }
    }

    pub fn get(&self, slot: ArmorSlot) -> Option<&ItemInstanceId> {
        match slot {
            ArmorSlot::Head => self.head.as_ref(),
            ArmorSlot::Body => self.body.as_ref(),
            ArmorSlot::Feet => self.feet.as_ref(),
            ArmorSlot::Hands => self.hands.as_ref(),
        }
    }

    pub fn set(&mut self, slot: ArmorSlot, item: ItemInstanceId) -> Option<ItemInstanceId> {
        self.get_mut(slot).replace(item)
    }

    pub fn remove(&mut self, slot: ArmorSlot) -> Option<ItemInstanceId> {
        self.get_mut(slot).take()
    }

    pub fn damage_restistance(&self) {
        todo!()
    }
}

#[derive(Component)]
pub struct Player;

#[derive(Component, Debug, Hash, Clone, PartialEq, Eq)]
pub struct Npc {
    pub id: NpcId,
    pub image: NpcImage,
    pub voice: NpcVoice,
}

#[derive(Component, Debug, Hash, Clone, PartialEq, Eq, Deserialize)]
pub struct NpcImage(pub PathBuf);
impl Default for NpcImage {
    fn default() -> Self {
        Self("images/mysterious.png".into())
    }
}

#[derive(Component, Debug, Hash, Clone, PartialEq, Eq, Deserialize)]
pub struct NpcVoice(pub String);
impl Default for NpcVoice {
    fn default() -> Self {
        Self("default".into())
    }
}

#[derive(
    Debug, derive_more::Display, PartialEq, Eq, Deserialize, Clone, Hash, derive_more::From,
)]
pub struct NpcId(pub String);

#[derive(Component, Debug)]
pub struct RpgEntity {
    name: String,
    damage: f32,
    armor: ArmorSlots,
    weapon: Option<ItemInstanceId>,
    shield: Option<ItemInstanceId>,
    pub inventory: Inventory,
}

impl RpgEntity {
    pub fn new(name: Option<String>) -> Self {
        Self {
            name: name.unwrap_or_else(|| "?".to_string()),
            damage: 0.0,
            armor: ArmorSlots::default(),
            weapon: None,
            shield: None,
            inventory: Inventory::default(),
        }
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn set_name(&mut self, name: String) {
        self.name = name
    }

    pub fn equip(&mut self, instance_id: ItemInstanceId) -> bool {
        let Some(item_instance) = self.inventory.get(&instance_id) else {
            warn!("could not equip: {instance_id:?}");
            return false;
        };

        match item_instance.kind() {
            ItemKind::Apparel(slot) => {
                self.armor.set(*slot, instance_id);
            }
            ItemKind::Weapon => {
                self.weapon.insert(instance_id);
            }
            ItemKind::Shield => {
                self.shield.insert(instance_id);
            }
            ItemKind::Food | ItemKind::Potion => {
                return false;
            }
        };

        info!("{:?} equipped: {:?}", self.name, item_instance.item_id());
        true
    }

    pub fn apply_damage(&mut self, damage: f32) -> DamageResult {
        // TODO: apply damage resistance
        let reduced_damage = damage;
        self.damage += reduced_damage;

        DamageResult {
            reduced_damage,
            life_status: if self.damage < self.max_health() {
                LifeStatus::Alive
            } else {
                self.damage = self.max_health();
                LifeStatus::Dead
            },
        }
    }

    pub fn max_health(&self) -> f32 {
        20.0 // TODO
    }

    pub fn health(&self) -> f32 {
        self.max_health() - self.damage
    }

    pub fn attack_damage(&self, item_manager: &ItemManager) -> f32 {
        // TODO: adjust based on character stats
        if let Some(weapon) = get_weapon(self.weapon, &self.inventory, item_manager) {
            weapon.damage() as f32
        } else {
            1.0
        }
    }

    pub fn damage(&self) -> f32 {
        self.damage
    }

    // TODO: consider adding `Alive` and `Dead` resources to NPC bundles so you can query `With<Alive>`
    pub fn is_alive(&self) -> bool {
        self.health() > 0.0
    }

    pub fn is_dead(&self) -> bool {
        !self.is_alive()
    }

    pub fn equipped(&self) -> impl Iterator<Item = &ItemInstanceId> {
        self.weapon
            .iter()
            .chain(self.shield.iter())
            .chain(self.armor.head.iter())
            .chain(self.armor.body.iter())
            .chain(self.armor.hands.iter())
            .chain(self.armor.feet.iter())
    }

    pub fn is_equipped(&self, instance_id: &ItemInstanceId) -> bool {
        self.equipped()
            .any(|equipped_instance_id| equipped_instance_id == instance_id)
    }
}

impl std::fmt::Display for RpgEntity {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.name)
    }
}

#[derive(Component, Default, Debug)]
pub struct Inventory {
    pub(crate) items: HashMap<ItemInstanceId, ItemInstance>,
}

impl Inventory {
    pub fn get(&self, id: &ItemInstanceId) -> Option<&ItemInstance> {
        self.items.get(id)
    }

    pub fn insert(&mut self, instance: ItemInstance) -> ItemInstanceId {
        let instance_id = instance.instance_id();
        self.items.insert(instance_id, instance);
        instance_id
    }
}

pub enum LifeStatus {
    Alive,
    Dead,
}

impl LifeStatus {
    /// Returns `true` if the life status is [`Dead`].
    ///
    /// [`Dead`]: LifeStatus::Dead
    #[must_use]
    pub fn is_dead(&self) -> bool {
        matches!(self, Self::Dead)
    }
}

pub struct DamageResult {
    pub reduced_damage: f32,
    pub life_status: LifeStatus,
}
