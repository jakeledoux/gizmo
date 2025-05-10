use bevy::prelude::Component;
use serde::Deserialize;

pub type ItemId = u64;

#[serde(rename_all = "lowercase")]
#[derive(Clone, Copy, Debug, Hash, PartialEq, Eq, Deserialize)]
pub enum ArmorSlot {
    Head,
    Body,
    Feet,
    Hands,
    Shield,
}

#[derive(Default, Debug, Hash, PartialEq, Eq)]
pub struct ArmorSlots {
    head: Option<ItemId>,
    body: Option<ItemId>,
    feet: Option<ItemId>,
    hands: Option<ItemId>,
    shield: Option<ItemId>,
}

impl ArmorSlots {
    pub fn get_mut(&mut self, slot: ArmorSlot) -> &mut Option<ItemId> {
        match slot {
            ArmorSlot::Head => &mut self.head,
            ArmorSlot::Body => &mut self.body,
            ArmorSlot::Feet => &mut self.feet,
            ArmorSlot::Hands => &mut self.hands,
            ArmorSlot::Shield => &mut self.shield,
        }
    }

    pub fn get(&self, slot: ArmorSlot) -> Option<ItemId> {
        match slot {
            ArmorSlot::Head => self.head,
            ArmorSlot::Body => self.body,
            ArmorSlot::Feet => self.feet,
            ArmorSlot::Hands => self.hands,
            ArmorSlot::Shield => self.shield,
        }
    }

    pub fn set(&mut self, slot: ArmorSlot, item: ItemId) -> Option<ItemId> {
        let previous_item = self.get(slot);
        _ = self.get_mut(slot).insert(item);
        previous_item
    }

    pub fn remove(&mut self, slot: ArmorSlot) -> Option<ItemId> {
        self.get_mut(slot).take()
    }

    pub fn damage_resistence(&self) {
        todo!()
    }
}

pub enum LifeStatus {
    Alive,
    Dead,
}

#[derive(Component)]
pub struct Player;

#[derive(Component)]
pub struct Npc;

#[derive(Component)]
pub struct RpgEntity {
    name: &'static str,
    damage: f32,
    armor: ArmorSlots,
}

impl RpgEntity {
    pub fn new(name: &'static str) -> Self {
        Self {
            name,
            damage: 0.0,
            armor: ArmorSlots::default(),
        }
    }

    pub fn apply_damage(&mut self, damage: f32) -> LifeStatus {
        // TODO: apply damage resistence
        self.damage += damage;

        if self.damage < self.max_health() {
            return LifeStatus::Alive;
        }
        self.damage = self.max_health();
        LifeStatus::Dead
    }

    pub fn max_health(&self) -> f32 {
        20.0 // TODO
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

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_armor_slots() {
        let mut armor_slots = ArmorSlots::default();
        assert_eq!(armor_slots.set(ArmorSlot::Head, 0), None);
        assert_eq!(armor_slots.set(ArmorSlot::Head, 1), Some(0));
        assert_eq!(armor_slots.remove(ArmorSlot::Head), Some(1));
        assert_eq!(armor_slots.get(ArmorSlot::Head), None);

        assert_eq!(armor_slots.set(ArmorSlot::Feet, 0), None);
        assert_eq!(armor_slots.set(ArmorSlot::Body, 2), None);
        assert_eq!(armor_slots.remove(ArmorSlot::Feet), Some(0));
        assert_eq!(armor_slots.remove(ArmorSlot::Body), Some(2));

        assert_eq!(armor_slots, ArmorSlots::default());
    }
}
