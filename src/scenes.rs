use std::{
    ops::Add,
    path::{Path, PathBuf},
};

use bevy::{
    asset::uuid::Uuid,
    log::warn,
    platform::collections::HashMap,
    prelude::{EventWriter, Resource},
};
use serde::Deserialize;

use crate::{EndSceneEvent, components::ArmorSlot};

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

#[derive(
    Deserialize, Debug, Hash, Clone, PartialEq, Eq, derive_more::From, derive_more::Display,
)]
pub struct SceneId(String);

impl SceneId {
    pub fn new(s: &str) -> Self {
        Self(s.to_string())
    }
}

#[derive(Deserialize, Debug, Clone, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct Scene {
    id: SceneId,
    music: String,
    characters: HashMap<CharacterId, Character>,
    dialogue: HashMap<SceneSectionId, Dialogue>,
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
    #[serde(alias = "cont")]
    continue_to: Option<SceneSectionId>,
}

#[derive(Deserialize, Debug, Clone, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub struct Line {
    pub from: CharacterId,
    #[serde(alias = "txt")]
    pub text: String,
    #[serde(flatten)]
    commands: Option<SceneCommands>,
}

#[derive(Deserialize, Debug, Clone, PartialEq, Eq)]
#[serde(rename_all = "kebab-case", deny_unknown_fields)]
pub struct Response {
    #[serde(alias = "txt")]
    pub text: String,
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

#[derive(Resource, Debug, Clone)]
pub struct ScenePlayer {
    scene: SceneId,
    current_key: SceneSectionId,
    current_line: usize,
    highlighted_response: usize,
}

impl ScenePlayer {
    fn new(scene: SceneId) -> Self {
        Self {
            scene,
            // TODO: load current_key from save
            current_key: String::from("start").into(),
            current_line: 0,
            highlighted_response: 0,
        }
    }

    fn get_scene<'a>(&self, scene_manager: &'a SceneManager) -> &'a Scene {
        scene_manager.scenes.get(&self.scene).unwrap_or_else(|| {
            panic!("no scene with ID: {:?}", self.scene);
        })
    }

    fn get_dialogue<'a>(&self, scene_manager: &'a SceneManager) -> &'a Dialogue {
        let scene = self.get_scene(scene_manager);
        let Some(dialogue) = scene.dialogue.get(&self.current_key) else {
            panic!(
                "no such dialogue: {:?} in scene: {:#?}",
                self.current_key, self.scene
            )
        };

        dialogue
    }

    fn reset_line(&mut self) {
        self.current_line = 0;
        self.highlighted_response = 0;
    }

    fn advance_line(&mut self) {
        self.current_line += 1;
        self.highlighted_response = 0;
    }

    fn set_key(&mut self, key: SceneSectionId) {
        self.current_key = key;
        self.reset_line()
    }

    pub fn get_current<'a>(&'a self, scene_manager: &'a SceneManager) -> UiScenePart<'a> {
        let dialogue = self.get_dialogue(scene_manager);
        let line = &dialogue.lines[self.current_line];
        let responses = if dialogue.lines.len() - 1 == self.current_line {
            dialogue.responses.as_ref()
        } else {
            None
        };
        UiScenePart { line, responses }
    }

    pub fn input(
        &mut self,
        input: ScenePlayerInput,
        scene_manager: &SceneManager,
        end_scene_event: &mut EventWriter<EndSceneEvent>,
    ) {
        let dialogue = self.get_dialogue(scene_manager);
        match input {
            ScenePlayerInput::MoveUp => {
                self.highlighted_response = self.highlighted_response.saturating_sub(1);
            }
            ScenePlayerInput::MoveDown => {
                self.highlighted_response = self.highlighted_response.add(1).min(
                    dialogue
                        .responses
                        .as_ref()
                        .map(|responses| responses.len() - 1)
                        .unwrap_or(0),
                );
            }
            ScenePlayerInput::Select => {
                // execute response
                if let Some(response) = dialogue
                    .responses
                    .as_ref()
                    .map(|responses| &responses[self.highlighted_response])
                {
                    // TODO: commands, skill check, etc.
                    self.set_key(response.link.clone());
                }
                // continue
                else {
                    if dialogue.lines.len() - 1 > self.current_line {
                        self.advance_line();
                    } else {
                        if let Some(ref section) = dialogue.continue_to {
                            self.set_key(section.to_owned())
                        } else {
                            end_scene_event.write(EndSceneEvent);
                        }
                    }
                }
            }
        }
    }

    pub fn highlighted_response(&self) -> usize {
        self.highlighted_response
    }
}

#[derive(Clone, Copy, Debug, Hash, PartialEq, Eq)]
pub enum ScenePlayerInput {
    MoveUp,
    MoveDown,
    Select,
}

pub struct UiScenePart<'a> {
    pub line: &'a Line,
    pub responses: Option<&'a Vec<Response>>,
}

#[derive(Resource, Debug, Clone, Default)]
pub struct SceneManager {
    scenes: HashMap<SceneId, Scene>,
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

    pub fn play_scene(&self, scene: SceneId) -> Option<ScenePlayer> {
        self.scenes
            .contains_key(&scene)
            .then(|| ScenePlayer::new(scene))
    }
}
