use std::path::Path;

use bevy::{
    asset::{Asset, uuid::Uuid},
    log::{info, warn},
    platform::collections::HashMap,
    prelude::Resource,
    reflect::Reflect,
};
use serde::Deserialize;

use crate::components::ArmorSlot;

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug, derive_more::From, derive_more::Display)]
pub struct ItemInstanceId(Uuid);

impl Default for ItemInstanceId {
    fn default() -> Self {
        Self::new()
    }
}

impl ItemInstanceId {
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }
}

#[derive(Clone, PartialEq, Eq, Debug, Hash)]
pub struct ItemInstance {
    instance_id: ItemInstanceId,
    item_id: ItemId,
    kind: ItemKind,
}

impl ItemInstance {
    pub fn instance_id(&self) -> ItemInstanceId {
        self.instance_id
    }

    pub fn item_id(&self) -> &ItemId {
        &self.item_id
    }

    pub fn kind(&self) -> &ItemKind {
        &self.kind
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug, Hash)]
pub enum ItemKind {
    Apparel(ArmorSlot),
    Weapon,
    Food,
    Potion,
    Shield,
}

#[derive(Deserialize, Debug, Clone, PartialEq, Eq)]
struct ItemFile {
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

#[derive(Deserialize, Reflect, Debug, Clone, PartialEq, Eq)]
pub struct Apparel {
    id: String,
    name: String,
    #[serde(alias = "limb")]
    slot: ArmorSlot,
    defense: u32,
    weight: u32,
    value: u32,
}

impl Apparel {
    pub fn id(&self) -> &str {
        &self.id
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn defense(&self) -> u32 {
        self.defense
    }

    pub fn weight(&self) -> u32 {
        self.weight
    }

    pub fn value(&self) -> u32 {
        self.value
    }
}

#[derive(Deserialize, Reflect, Debug, Clone, PartialEq, Eq)]
pub struct Weapon {
    id: String,
    name: String,
    damage: u32,
    weight: u32,
    value: u32,
}

impl Weapon {
    pub fn id(&self) -> &str {
        &self.id
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn damage(&self) -> u32 {
        self.damage
    }

    pub fn weight(&self) -> u32 {
        self.weight
    }

    pub fn value(&self) -> u32 {
        self.value
    }
}

#[derive(Deserialize, Reflect, Debug, Clone, PartialEq, Eq)]
pub struct Food {
    id: String,
    name: String,
    #[serde(alias = "heal_HP")]
    hp: u32,
    weight: u32,
    value: u32,
}

impl Food {
    pub fn id(&self) -> &str {
        &self.id
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn hp(&self) -> u32 {
        self.hp
    }

    pub fn weight(&self) -> u32 {
        self.weight
    }

    pub fn value(&self) -> u32 {
        self.value
    }
}

#[derive(Deserialize, Reflect, Debug, Clone, PartialEq, Eq)]
pub struct Potion {
    id: String,
    name: String,
    description: String,
    value: u32,
    effects: PotionEffects,
}

impl Potion {
    pub fn id(&self) -> &str {
        &self.id
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn description(&self) -> &str {
        &self.description
    }

    pub fn value(&self) -> u32 {
        self.value
    }

    pub fn effects(&self) -> &PotionEffects {
        &self.effects
    }
}

#[derive(Deserialize, Reflect, Debug, Clone, PartialEq, Eq)]
pub struct PotionEffects {
    health: Option<u32>,
}

#[derive(Deserialize, Reflect, Debug, Clone, PartialEq, Eq)]
pub struct Shield {
    id: String,
    name: String,
    weight: u32,
    defense: u32,
    value: u32,
}

impl Shield {
    pub fn id(&self) -> &str {
        &self.id
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn weight(&self) -> u32 {
        self.weight
    }

    pub fn defense(&self) -> u32 {
        self.defense
    }

    pub fn value(&self) -> u32 {
        self.value
    }
}

#[derive(
    Debug, Hash, Clone, PartialEq, Eq, PartialOrd, Ord, derive_more::From, derive_more::Display,
)]
pub struct ItemId(pub String);

impl ItemId {
    pub fn new(s: &str) -> Self {
        Self(s.to_owned())
    }
}

#[derive(Asset, Reflect, Deserialize, Debug, Clone, PartialEq, Eq, derive_more::From)]
#[serde(untagged)]
pub enum AnyItem {
    Apparel(Apparel),
    Weapon(Weapon),
    Food(Food),
    Potion(Potion),
    Shield(Shield),
}

impl AnyItem {
    pub fn id(&self) -> ItemId {
        match self {
            AnyItem::Apparel(i) => &i.id,
            AnyItem::Weapon(i) => &i.id,
            AnyItem::Food(i) => &i.id,
            AnyItem::Potion(i) => &i.id,
            AnyItem::Shield(i) => &i.id,
        }
        .to_owned()
        .into()
    }

    pub fn name(&self) -> &str {
        match self {
            AnyItem::Apparel(i) => &i.name,
            AnyItem::Weapon(i) => &i.name,
            AnyItem::Food(i) => &i.name,
            AnyItem::Potion(i) => &i.name,
            AnyItem::Shield(i) => &i.name,
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

    fn kind(&self) -> ItemKind {
        match self {
            AnyItem::Apparel(apparel) => ItemKind::Apparel(apparel.slot),
            AnyItem::Weapon(_) => ItemKind::Weapon,
            AnyItem::Food(_) => ItemKind::Food,
            AnyItem::Potion(_) => ItemKind::Potion,
            AnyItem::Shield(_) => ItemKind::Shield,
        }
    }
}

#[derive(Resource, Debug, Clone, Default)]
pub struct ItemManager {
    pub(crate) items: HashMap<ItemId, AnyItem>,
}

impl ItemManager {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn load_items<P: AsRef<Path>>(&mut self, path: P) -> anyhow::Result<()> {
        info!("loading items from file: {:?}", path.as_ref());
        let item_json = std::fs::read_to_string(path)?;
        let item_file: ItemFile = serde_json::from_str(&item_json)?;
        item_file.into_iter().for_each(|item| {
            let item_id: ItemId = item.id();
            self.items.insert(item_id, item);
        });
        Ok(())
    }

    pub fn with_load_items<P: AsRef<Path>>(mut self, path: P) -> anyhow::Result<Self> {
        self.load_items(path)?;
        Ok(self)
    }

    pub fn load_folder<P: AsRef<Path>>(&mut self, path: P) -> anyhow::Result<()> {
        path.as_ref()
            .read_dir()?
            .filter_map(Result::ok)
            .filter(|entry| {
                entry
                    .path()
                    .extension()
                    .map(|ext| ext == "json")
                    .unwrap_or(false)
            })
            .try_for_each(|entry| self.load_items(entry.path()))
    }

    pub fn with_load_folder<P: AsRef<Path>>(mut self, path: P) -> anyhow::Result<Self> {
        self.load_folder(path)?;
        Ok(self)
    }

    pub fn get_item(&self, id: &ItemId) -> Option<&AnyItem> {
        self.items.get(id)
    }

    pub fn spawn(&self, item_id: ItemId) -> Option<ItemInstance> {
        let Some(item) = self.get_item(&item_id) else {
            warn!("no item with ID: {item_id:?}");
            return None;
        };

        Some(ItemInstance {
            instance_id: ItemInstanceId::new(),
            item_id,
            kind: item.kind(),
        })
    }
}

#[cfg(test)]
mod test {
    use crate::ItemInstanceId;

    #[test]
    fn unique_instance_ids() {
        assert_ne!(ItemInstanceId::new(), ItemInstanceId::new())
    }
}
