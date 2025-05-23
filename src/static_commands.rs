use std::path::PathBuf;

use bevy::{log::info, platform::collections::HashMap, prelude::EventWriter};
use serde::Deserialize;

use crate::{
    Character, NpcId, SceneId, SceneManager, SceneSectionId, SpawnNpcEvent, StartBattleEvent, TODO,
    UpdateNpcEvent,
};

#[derive(Deserialize, Debug, Clone, PartialEq, Eq, derive_more::From)]
#[serde(rename_all = "kebab-case")]
#[serde(deny_unknown_fields)]
pub struct Definitions {
    characters: Option<HashMap<NpcId, Character>>,
    vendors: Option<TODO>,
    quests: Option<TODO>,
}

impl Definitions {
    pub fn create(&self, spawn_npc_event: &mut EventWriter<SpawnNpcEvent>) {
        if let Some(characters) = &self.characters {
            for (character_id, character) in characters.iter() {
                spawn_npc_event.write(SpawnNpcEvent(character_id.to_owned(), character.to_owned()));
            }
        }
        if let Some(_vendors) = &self.vendors {
            todo!()
        }
        if let Some(_quests) = &self.quests {
            todo!()
        }
    }
}

#[derive(Deserialize, Debug, Clone, PartialEq, Eq)]
pub struct CharacterUpdate {
    pub name: Option<String>,
    pub image: Option<PathBuf>,
    pub voice: Option<String>,
}

#[derive(Deserialize, Debug, Clone, PartialEq, Eq, derive_more::From)]
#[serde(deny_unknown_fields)]
pub struct RewardGoldCommand {
    amount: u32,
    from: NpcId,
}

#[derive(Deserialize, Debug, Clone, PartialEq, Eq, derive_more::From)]
#[serde(rename_all = "kebab-case")]
#[serde(deny_unknown_fields)]
pub struct StaticCommands {
    reward_gold: Option<RewardGoldCommand>,
    update_characters: Option<HashMap<NpcId, CharacterUpdate>>,
    scene_entry: Option<HashMap<SceneId, SceneSectionId>>,
    #[serde(alias = "vars")]
    variables: Option<HashMap<String, String>>,
    battle: Option<NpcId>,
    kill_character: Option<TODO>,
    set_quest_stage: Option<TODO>,
    complete_quest: Option<TODO>,
}

impl StaticCommands {
    pub fn execute(
        self,
        scene_manager: &mut SceneManager,
        start_battle_event: &mut EventWriter<StartBattleEvent>,
        update_npc_event: &mut EventWriter<UpdateNpcEvent>,
    ) {
        // TODO: reward_gold
        if let Some(update_characters) = self.update_characters {
            for (npc_id, character_update) in update_characters {
                update_npc_event.write(UpdateNpcEvent(npc_id, character_update));
            }
        }
        if let Some(scene_entry) = self.scene_entry {
            info!("updating scene entry points: {scene_entry:?}");
            scene_entry.into_iter().for_each(|(scene, key)| {
                scene_manager.update_scene_entry(scene, key);
            });
        }
        if let Some(variables) = self.variables {
            info!("updating variables: {variables:?}");
            scene_manager.update_variables(variables);
        }
        if let Some(battle) = self.battle {
            start_battle_event.write(StartBattleEvent(battle));
        }
        // TODO: kill_character
        // TODO: set_quest_stage
        // TODO: complete_quest
    }
}
