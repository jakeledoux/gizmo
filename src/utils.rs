use bevy::prelude::{Commands, Query};

use crate::{
    AnyItem, Apparel, Character, Food, Inventory, ItemInstanceId, ItemManager, Npc, NpcId, Potion,
    RpgEntity, Shield, Weapon,
};

pub fn get_item<'a>(
    maybe_instance_id: Option<ItemInstanceId>,
    inventory: &Inventory,
    item_manager: &'a ItemManager,
) -> Option<&'a AnyItem> {
    maybe_instance_id
        .and_then(|instance_id| inventory.get(&instance_id))
        .and_then(|item_instance| item_manager.get_item(item_instance.item_id()))
}

pub fn get_apparel<'a>(
    maybe_instance_id: Option<ItemInstanceId>,
    inventory: &Inventory,
    item_manager: &'a ItemManager,
) -> Option<&'a Apparel> {
    get_item(maybe_instance_id, inventory, item_manager).and_then(AnyItem::as_apparel)
}

pub fn get_weapon<'a>(
    maybe_instance_id: Option<ItemInstanceId>,
    inventory: &Inventory,
    item_manager: &'a ItemManager,
) -> Option<&'a Weapon> {
    get_item(maybe_instance_id, inventory, item_manager).and_then(AnyItem::as_weapon)
}

pub fn get_food<'a>(
    maybe_instance_id: Option<ItemInstanceId>,
    inventory: &Inventory,
    item_manager: &'a ItemManager,
) -> Option<&'a Food> {
    get_item(maybe_instance_id, inventory, item_manager).and_then(AnyItem::as_food)
}

pub fn get_potion<'a>(
    maybe_instance_id: Option<ItemInstanceId>,
    inventory: &Inventory,
    item_manager: &'a ItemManager,
) -> Option<&'a Potion> {
    get_item(maybe_instance_id, inventory, item_manager).and_then(AnyItem::as_potion)
}

pub fn get_shield<'a>(
    maybe_instance_id: Option<ItemInstanceId>,
    inventory: &Inventory,
    item_manager: &'a ItemManager,
) -> Option<&'a Shield> {
    get_item(maybe_instance_id, inventory, item_manager).and_then(AnyItem::as_shield)
}

pub fn spawn_npc(
    commands: &mut Commands,
    npc_query: Query<&Npc>,
    id: NpcId,
    character: Character,
) -> bool {
    if npc_query.into_iter().any(|npc| npc.id == id) {
        return false;
    }
    commands.spawn((
        Npc {
            id,
            image: character.image,
            voice: character.voice,
        },
        RpgEntity::new(Some(character.name)),
    ));
    true
}
