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
    id: String,
    name: String,
    #[serde(alias = "limb")]
    slot: ArmorSlot,
    defense: u32,
    weight: u32,
    value: u32,
}

#[derive(Deserialize, Debug, Clone, PartialEq, Eq)]
pub struct Weapon {
    id: String,
    name: String,
    damage: u32,
    weight: u32,
    value: u32,
}

#[derive(Deserialize, Debug, Clone, PartialEq, Eq)]
pub struct Food {
    id: String,
    name: String,
    #[serde(alias = "heal_HP")]
    hp: u32,
    weight: u32,
    value: u32,
}

#[derive(Deserialize, Debug, Clone, PartialEq, Eq)]
pub struct Potion {
    id: String,
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
    id: String,
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
    pub fn item_id(&self) -> ItemId {
        let id = match self {
            AnyItem::Apparel(i) => &i.id,
            AnyItem::Weapon(i) => &i.id,
            AnyItem::Food(i) => &i.id,
            AnyItem::Potion(i) => &i.id,
            AnyItem::Shield(i) => &i.id,
        };
        FixedHasher::default().hash_one(id).into()
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
}
