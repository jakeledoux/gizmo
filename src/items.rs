use std::{hash::BuildHasher, path::Path};

use bevy::platform::{collections::HashMap, hash::FixedHasher};
use serde::Deserialize;

use crate::components::ArmorSlot;

const ITEM_PATH: &str = "./assets/items";

// TODO: make not public
#[derive(Deserialize, Debug, Clone, PartialEq, Eq)]
pub struct ItemFile {
    apparel: Option<Vec<Apparel>>,
    weapon: Option<Vec<Weapon>>,
    food: Option<Vec<Food>>,
    potion: Option<Vec<Potion>>,
    shield: Option<Vec<Shield>>,
}

impl ItemFile {
    fn into_iter(self) -> impl Iterator<Item = AnyItem> {
        self.apparel
            .unwrap_or_default()
            .into_iter()
            .map(AnyItem::from)
            .chain(
                self.weapon
                    .unwrap_or_default()
                    .into_iter()
                    .map(AnyItem::from),
            )
            .chain(self.food.unwrap_or_default().into_iter().map(AnyItem::from))
            .chain(
                self.potion
                    .unwrap_or_default()
                    .into_iter()
                    .map(AnyItem::from),
            )
            .chain(
                self.shield
                    .unwrap_or_default()
                    .into_iter()
                    .map(AnyItem::from),
            )
    }
}

#[derive(Deserialize, Debug, Clone, PartialEq, Eq)]
pub struct Apparel {
    #[serde(alias = "id")]
    human_id: String,
    name: String,
    #[serde(alias = "limb")]
    slot: ArmorSlot,
    defense: u32,
    weight: u32,
    value: u32,
}

#[derive(Deserialize, Debug, Clone, PartialEq, Eq)]
pub struct Weapon {
    #[serde(alias = "id")]
    human_id: String,
    name: String,
    damage: u32,
    weight: u32,
    value: u32,
}

#[derive(Deserialize, Debug, Clone, PartialEq, Eq)]
pub struct Food {
    #[serde(alias = "id")]
    human_id: String,
    name: String,
    #[serde(alias = "heal_HP")]
    hp: u32,
    weight: u32,
    value: u32,
}

#[derive(Deserialize, Debug, Clone, PartialEq, Eq)]
pub struct Potion {
    #[serde(alias = "id")]
    human_id: String,
    name: String,
    description: String,
    value: u32,
    effects: PotionEffects,
}

#[derive(Deserialize, Debug, Clone, PartialEq, Eq)]
pub struct PotionEffects {
    health: Option<u32>,
}

#[derive(Deserialize, Debug, Clone, PartialEq, Eq)]
pub struct Shield {
    #[serde(alias = "id")]
    human_id: String,
    name: String,
    weight: u32,
    defense: u32,
    value: u32,
}

#[derive(Debug, Hash, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, derive_more::From)]
pub struct ItemId(u64);

impl std::fmt::Display for ItemId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:x}", self.0)
    }
}

#[derive(Deserialize, Debug, Clone, PartialEq, Eq, derive_more::From)]
#[serde(untagged)]
pub enum AnyItem {
    Apparel(Apparel),
    Weapon(Weapon),
    Food(Food),
    Potion(Potion),
    Shield(Shield),
}

impl AnyItem {
    pub fn human_id(&self) -> &str {
        match self {
            AnyItem::Apparel(i) => &i.human_id,
            AnyItem::Weapon(i) => &i.human_id,
            AnyItem::Food(i) => &i.human_id,
            AnyItem::Potion(i) => &i.human_id,
            AnyItem::Shield(i) => &i.human_id,
        }
    }

    pub fn weight(&self) -> u32 {
        match self {
            AnyItem::Apparel(i) => i.weight,
            AnyItem::Weapon(i) => i.weight,
            AnyItem::Food(i) => i.weight,
            AnyItem::Potion(_) => 0,
            AnyItem::Shield(i) => i.weight,
        }
    }

    pub fn value(&self) -> u32 {
        match self {
            AnyItem::Apparel(i) => i.value,
            AnyItem::Weapon(i) => i.value,
            AnyItem::Food(i) => i.value,
            AnyItem::Potion(i) => i.value,
            AnyItem::Shield(i) => i.value,
        }
    }

    pub fn item_id(&self) -> ItemId {
        hash_human_id(self.human_id())
    }

    pub fn is_apparel(&self) -> bool {
        matches!(self, Self::Apparel(_))
    }

    pub fn is_weapon(&self) -> bool {
        matches!(self, Self::Weapon(_))
    }

    pub fn is_food(&self) -> bool {
        matches!(self, Self::Food(_))
    }

    pub fn is_potion(&self) -> bool {
        matches!(self, Self::Potion(_))
    }

    pub fn is_shield(&self) -> bool {
        matches!(self, Self::Shield(_))
    }

    pub fn as_apparel(&self) -> Option<&Apparel> {
        if let Self::Apparel(apparel) = self {
            Some(apparel)
        } else {
            None
        }
    }

    pub fn as_weapon(&self) -> Option<&Weapon> {
        if let Self::Weapon(weapon) = self {
            Some(weapon)
        } else {
            None
        }
    }

    pub fn as_food(&self) -> Option<&Food> {
        if let Self::Food(food) = self {
            Some(food)
        } else {
            None
        }
    }

    pub fn as_potion(&self) -> Option<&Potion> {
        if let Self::Potion(potion) = self {
            Some(potion)
        } else {
            None
        }
    }

    pub fn as_shield(&self) -> Option<&Shield> {
        if let Self::Shield(shield) = self {
            Some(shield)
        } else {
            None
        }
    }
}

#[derive(Debug, Clone, Default)]
pub struct ItemManager {
    items: HashMap<ItemId, AnyItem>,
}

impl ItemManager {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn load_items<P: AsRef<Path>>(&mut self, path: P) -> anyhow::Result<()> {
        let item_json = std::fs::read_to_string(dbg!(Path::new(ITEM_PATH).join(path)))?;
        let item_file: ItemFile = serde_json::from_str(&item_json)?;
        item_file.into_iter().for_each(|item| {
            let item_id: ItemId = item.item_id();
            self.items.insert(item_id, item);
        });
        Ok(())
    }

    pub fn get_item(&self, id: ItemId) -> Option<&AnyItem> {
        self.items.get(&id)
    }

    pub fn get_item_by_human_id(&self, id: &str) -> Option<&AnyItem> {
        self.items.get(&hash_human_id(id))
    }
}

pub fn hash_human_id(id: &str) -> ItemId {
    FixedHasher::default().hash_one(id).into()
}
