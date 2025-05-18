use std::{
    collections::HashSet,
    ops::Add,
    path::{Path, PathBuf},
};

use bevy::{
    log::{error, info},
    platform::collections::HashMap,
    prelude::{EventWriter, Resource},
};
use serde::Deserialize;

use crate::{EndSceneEvent, SceneCommandsEvent};

#[allow(clippy::upper_case_acronyms)]
type TODO = serde_json::Value;

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

#[derive(Debug, Hash, Clone, PartialEq, Eq)]
pub struct SceneBookmark(String);

impl SceneBookmark {
    pub fn new(
        scene: &SceneId,
        section: Option<&SceneSectionId>,
        line: Option<usize>,
        response: Option<usize>,
    ) -> Self {
        let mut bookmark = String::from(&scene.0);
        if let Some(section) = section {
            bookmark.push_str(&format!(":section({})", section.0))
        };
        if let Some(line) = line {
            bookmark.push_str(&format!(":line({})", line))
        };
        if let Some(response) = response {
            bookmark.push_str(&format!(":response({})", response))
        };

        Self(bookmark)
    }
}

#[derive(Deserialize, Debug, Clone, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct Scene {
    id: SceneId,
    music: Option<String>,
    characters: Option<HashMap<CharacterId, Character>>,
    vendors: Option<TODO>,
    quests: Option<TODO>,
    dialogue: HashMap<SceneSectionId, Dialogue>,
    #[serde(flatten)]
    commands: Option<SceneCommands>, // TODO: execute these?
}

#[derive(Deserialize, Debug, Clone, PartialEq, Eq)]
pub struct Character {
    name: String,
    #[serde(default)]
    image: PathBuf,
    #[serde(default)]
    voice: String,
}

impl Default for Character {
    fn default() -> Self {
        Self {
            name: "?".to_string(),
            image: PathBuf::from("images/unknown.png"),
            voice: "default".to_string(),
        }
    }
}

#[derive(Deserialize, Debug, Clone, PartialEq, Eq)]
pub struct CharacterUpdate {
    name: Option<String>,
    image: Option<PathBuf>,
    voice: Option<String>,
}

#[derive(Deserialize, Debug, Clone, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub struct Dialogue {
    #[serde(default)]
    lines: Vec<Line>,
    #[serde(alias = "resp", default)]
    responses: Vec<Response>,
    #[serde(flatten)]
    commands: Option<SceneCommands>, // TODO: execute these?
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
    commands: Option<SceneCommands>, // TODO: execute these?
}

#[derive(Deserialize, Debug, Clone, PartialEq, Eq)]
#[serde(rename_all = "kebab-case", deny_unknown_fields)]
pub struct Response {
    #[serde(alias = "txt")]
    pub text: String,
    #[serde(alias = "lnk")]
    link: Option<SceneSectionId>,
    skill_check: Option<SkillCheck>,
    // TODO:
    #[serde(alias = "cond", default)]
    conditions: Vec<Condition>,
    #[serde(flatten)]
    commands: Option<SceneCommands>,
}
impl Response {
    fn evaluate_conditions(&self, scene_manager: &SceneManager) -> bool {
        self.conditions.iter().all(|c| c.evaluate(scene_manager))
    }
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

#[derive(Deserialize, Debug, Clone, PartialEq, Eq)]
#[serde(rename_all = "kebab-case", tag = "type")]
pub enum Condition {
    VarEquals {
        #[serde(alias = "var")]
        variable: String,
        value: String,
    },
    Any {
        #[serde(alias = "cond")]
        conditions: Vec<Condition>,
    },
    Not {
        #[serde(alias = "cond")]
        conditions: Vec<Condition>,
    },
    // TODO
    HasItem,
    QuestStage,
}
impl Condition {
    fn evaluate(&self, scene_manager: &SceneManager) -> bool {
        match self {
            Condition::VarEquals { variable, value } => scene_manager
                .get_variable(variable)
                .map(|v| v == value)
                .unwrap_or(false),
            Condition::Any { conditions } => conditions.iter().any(|c| c.evaluate(scene_manager)),
            Condition::Not { conditions } => conditions.iter().all(|c| !c.evaluate(scene_manager)),
            Condition::HasItem => todo!(),
            Condition::QuestStage => todo!(),
        }
    }
}

#[derive(Deserialize, Debug, Clone, PartialEq, Eq, derive_more::From)]
#[serde(rename_all = "kebab-case")]
#[serde(deny_unknown_fields)]
pub struct SceneCommands {
    reward_gold: Option<RewardGoldCommand>,
    update_characters: Option<HashMap<CharacterId, CharacterUpdate>>,
    scene_entry: Option<HashMap<SceneId, SceneSectionId>>,
    #[serde(alias = "vars")]
    variables: Option<HashMap<String, String>>,
    battle: Option<TODO>,
    kill_character: Option<TODO>,
    set_quest_stage: Option<TODO>,
    complete_quest: Option<TODO>,
}

impl SceneCommands {
    pub fn execute(self, scene_manager: &mut SceneManager) {
        // TODO: reward_gold
        // TODO: update_characters
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
        // TODO: battle
        // TODO: kill_character
        // TODO: set_quest_stage
        // TODO: complete_quest
    }
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
    executed_commands: HashSet<SceneBookmark>,
}

impl ScenePlayer {
    fn new(scene: SceneId) -> Self {
        Self {
            scene,
            // TODO: load current_key from save
            current_key: String::from("start").into(),
            current_line: 0,
            highlighted_response: 0,
            executed_commands: HashSet::default(),
        }
    }

    fn get_scene<'a>(&self, scene_manager: &'a SceneManager) -> &'a Scene {
        scene_manager.scenes.get(&self.scene).unwrap_or_else(|| {
            panic!("no scene with ID: {:?}", self.scene);
        })
    }

    fn get_dialogue<'a>(
        &self,
        scene_manager: &'a SceneManager,
        scene_commands_events: &mut EventWriter<SceneCommandsEvent>,
    ) -> &'a Dialogue {
        let scene = self.get_scene(scene_manager);
        let Some(dialogue) = scene.dialogue.get(&self.current_key) else {
            panic!(
                "no such dialogue: {:?} in scene: {:#?}",
                self.current_key, self.scene
            )
        };

        if let Some(commands) = dialogue.commands.clone() {
            let bookmark = SceneBookmark::new(&self.scene, Some(&self.current_key), None, None);
            scene_commands_events.write(SceneCommandsEvent(bookmark, commands));
        }

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

    fn select(
        &mut self,
        dialogue: &Dialogue,
        end_scene_event: &mut EventWriter<EndSceneEvent>,
        scene_commands_events: &mut EventWriter<SceneCommandsEvent>,
    ) {
        if dialogue.lines.is_empty() {
            end_scene_event.write(EndSceneEvent);
            return;
        }

        let line = dialogue
            .lines
            .get(self.current_line)
            .expect("advanced past all lines in dialogue. this should not be possible");

        // execute line command
        if let Some(commands) = line.commands.clone() {
            let bookmark = SceneBookmark::new(
                &self.scene,
                Some(&self.current_key),
                Some(self.current_line),
                None,
            );
            scene_commands_events.write(SceneCommandsEvent(bookmark, commands));
        }

        // execute response
        if let Some(response) = dialogue.responses.get(self.highlighted_response) {
            if let Some(commands) = response.commands.clone() {
                let bookmark = SceneBookmark::new(
                    &self.scene,
                    Some(&self.current_key),
                    None,
                    Some(self.highlighted_response),
                );
                scene_commands_events.write(SceneCommandsEvent(bookmark, commands));
            }

            // TODO: skill check, etc.
            if let Some(link) = response.link.as_ref() {
                self.set_key(link.to_owned());
            } else {
                end_scene_event.write(EndSceneEvent);
            }
        }
        // continue
        else if dialogue.lines.len() - 1 > self.current_line {
            self.advance_line();
        } else if let Some(ref section) = dialogue.continue_to {
            self.set_key(section.to_owned())
        } else {
            end_scene_event.write(EndSceneEvent);
        }
    }

    pub fn get_current<'a>(
        &'a self,
        scene_manager: &'a SceneManager,
        scene_commands_events: &mut EventWriter<SceneCommandsEvent>,
    ) -> Option<UiScenePart<'a>> {
        let dialogue = self.get_dialogue(scene_manager, scene_commands_events);
        // some dialogues exist only to run commands and exit
        if dialogue.lines.is_empty() {
            return None;
        }
        let line = &dialogue.lines[self.current_line];

        // get responses if necessary
        let responses = {
            if dialogue.lines.len() - 1 > self.current_line {
                None
            } else {
                Some(
                    dialogue
                        .responses
                        .iter()
                        .filter(|resp| resp.evaluate_conditions(scene_manager))
                        .collect(),
                )
            }
        };
        Some(UiScenePart { line, responses })
    }

    pub fn input(
        &mut self,
        input: ScenePlayerInput,
        scene_manager: &mut SceneManager,
        end_scene_event: &mut EventWriter<EndSceneEvent>,
        scene_commands_events: &mut EventWriter<SceneCommandsEvent>,
    ) {
        // TODO: remove clone
        let dialogue = self
            .get_dialogue(scene_manager, scene_commands_events)
            .clone();
        match input {
            ScenePlayerInput::MoveUp => {
                self.highlighted_response = self.highlighted_response.saturating_sub(1);
            }
            ScenePlayerInput::MoveDown => {
                self.highlighted_response = self
                    .highlighted_response
                    .add(1)
                    .min(dialogue.responses.len().saturating_sub(1));
            }
            ScenePlayerInput::Select(i) => {
                self.highlighted_response = i;
                self.select(&dialogue, end_scene_event, scene_commands_events);
            }
            ScenePlayerInput::Select(_) | ScenePlayerInput::SelectCurrent => {
                self.select(&dialogue, end_scene_event, scene_commands_events);
            }
        }
    }

    pub fn highlighted_response(&self) -> usize {
        self.highlighted_response
    }

    pub fn execute(
        &mut self,
        bookmark: SceneBookmark,
        commands: SceneCommands,
        scene_manager: &mut SceneManager,
    ) {
        if self.executed_commands.insert(bookmark) {
            commands.execute(scene_manager);
        }
    }
}

#[derive(Clone, Copy, Debug, Hash, PartialEq, Eq)]
pub enum ScenePlayerInput {
    MoveUp,
    MoveDown,
    Select(usize),
    SelectCurrent,
}

pub struct UiScenePart<'a> {
    pub line: &'a Line,
    pub responses: Option<Vec<&'a Response>>,
}

#[derive(Resource, Debug, Clone, Default)]
pub struct SceneManager {
    scenes: HashMap<SceneId, Scene>,
    variables: HashMap<String, String>,
    entries: HashMap<SceneId, SceneSectionId>,
}

impl SceneManager {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn load_scene<P: AsRef<Path>>(&mut self, path: P) -> anyhow::Result<()> {
        info!("attempting to item file: {:?}", path.as_ref());
        let scene_json = std::fs::read_to_string(path)?;
        let scene: Scene = serde_json::from_str(&scene_json)?;
        self.scenes.insert(scene.id.clone(), scene);
        Ok(())
    }

    pub fn with_load_scene<P: AsRef<Path>>(mut self, path: P) -> anyhow::Result<Self> {
        self.load_scene(path)?;
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
            .for_each(|entry| {
                if let Err(e) = self.load_scene(entry.path()) {
                    error!("failed to load scene: {e}");
                }
            });
        Ok(())
    }

    pub fn with_load_folder<P: AsRef<Path>>(mut self, path: P) -> anyhow::Result<Self> {
        self.load_folder(path)?;
        Ok(self)
    }

    pub fn play_scene(&self, scene: SceneId) -> Option<ScenePlayer> {
        self.scenes.contains_key(&scene).then(|| {
            let scene_entry = self.entries.get(&scene);
            let mut scene_player = ScenePlayer::new(scene);
            if let Some(key) = scene_entry {
                scene_player.set_key(key.to_owned());
            }

            scene_player
        })
    }

    fn update_variables<U>(&mut self, variables: U)
    where
        U: IntoIterator<Item = (String, String)>,
    {
        self.variables.extend(variables)
    }

    fn get_variable(&self, variable: &str) -> Option<&String> {
        self.variables.get(variable)
    }

    fn update_scene_entry(
        &mut self,
        scene: SceneId,
        key: SceneSectionId,
    ) -> Option<SceneSectionId> {
        self.entries.insert(scene, key)
    }
}
