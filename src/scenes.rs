use std::path::{Path, PathBuf};

use bevy::{asset::uuid::Uuid, log::warn, platform::collections::HashMap, prelude::Resource};
use serde::Deserialize;

use crate::components::ArmorSlot;

#[cfg(debug_assertions)]
const SCENE_PATH: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/assets/scenes");
#[cfg(not(debug_assertions))]
const SCENE_PATH: &str = "assets/scenes";

#[derive(
    Deserialize, Debug, Hash, Clone, PartialEq, Eq, derive_more::From, derive_more::Display,
)]
pub struct CharacterId(String);

#[derive(
    Deserialize, Debug, Hash, Clone, PartialEq, Eq, derive_more::From, derive_more::Display,
)]
pub struct SceneSectionId(String);

#[derive(Deserialize, Debug, Clone, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct Scene {
    id: String,
    music: String,
    characters: HashMap<CharacterId, Character>,
    dialogue: HashMap<String, Dialogue>,
}

#[derive(Deserialize, Debug, Clone, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct Character {
    name: String,
    image: PathBuf,
    voice: String,
}

#[derive(Deserialize, Debug, Clone, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct CharacterUpdate {
    name: Option<String>,
    image: Option<PathBuf>,
    voice: Option<String>,
}

#[derive(Deserialize, Debug, Clone, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub struct Dialogue {
    lines: Vec<Line>,
    #[serde(alias = "resp")]
    responses: Option<Vec<Response>>,
    #[serde(flatten)]
    commands: Option<SceneCommands>,
}

#[derive(Deserialize, Debug, Clone, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub struct Line {
    from: CharacterId,
    #[serde(alias = "txt")]
    text: String,
    #[serde(flatten)]
    commands: Option<SceneCommands>,
}

#[derive(Deserialize, Debug, Clone, PartialEq, Eq)]
#[serde(rename_all = "kebab-case", deny_unknown_fields)]
pub struct Response {
    #[serde(alias = "txt")]
    text: String,
    #[serde(alias = "lnk")]
    link: SceneSectionId,
    skill_check: Option<SkillCheck>,
}

#[derive(Deserialize, Debug, Clone, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct SkillCheck {
    #[serde(alias = "lnk-fail")]
    link_fail: SceneSectionId,
    #[serde(alias = "lnk-crit-fail")]
    link_crit_fail: Option<SceneSectionId>,
    modifier: Option<i32>,
    check: Skill,
}

// TODO: replace with actual skill enum from `components.rs` when that is added
#[derive(Deserialize, Debug, Clone, PartialEq, Eq, derive_more::From)]
#[serde(rename_all = "lowercase")]
pub enum Skill {
    Strength,
    Perception,
    Endurance,
    Charisma,
    Intelligence,
    Agility,
    Luck,
}

#[derive(Deserialize, Debug, Clone, PartialEq, Eq, derive_more::From)]
#[serde(rename_all = "kebab-case")]
#[serde(deny_unknown_fields)]
pub struct SceneCommands {
    reward_gold: Option<RewardGoldCommand>,
    update_characters: Option<HashMap<CharacterId, CharacterUpdate>>,
    scene_entry: Option<HashMap<String, String>>,
}

#[derive(Deserialize, Debug, Clone, PartialEq, Eq, derive_more::From)]
#[serde(deny_unknown_fields)]
pub struct RewardGoldCommand {
    amount: u32,
    from: CharacterId,
}

#[derive(Resource, Debug, Clone, Default)]
pub struct SceneManager {
    scenes: HashMap<String, Scene>,
}

impl SceneManager {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn load_scene<P: AsRef<Path>>(&mut self, path: P) -> anyhow::Result<()> {
        let path = Path::new(SCENE_PATH).join(path);
        let scene_json = std::fs::read_to_string(path)?;
        let scene: Scene = serde_json::from_str(&scene_json)?;
        self.scenes.insert(scene.id.clone(), scene);
        Ok(())
    }

    pub fn with_load_scene<P: AsRef<Path>>(mut self, path: P) -> anyhow::Result<Self> {
        self.load_scene(path)?;
        Ok(self)
    }

    pub fn get_scene(&self, id: &str) -> Option<&Scene> {
        self.scenes.get(id)
    }
}
