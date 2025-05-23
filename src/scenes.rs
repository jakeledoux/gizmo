use std::{collections::HashSet, ops::Add, path::Path};

use bevy::{
    log::{error, info},
    platform::collections::HashMap,
    prelude::{EventWriter, Resource},
};
use serde::Deserialize;

use crate::{
    Definitions, EndSceneEvent, NpcId, NpcImage, NpcVoice, SpawnNpcEvent, StartBattleEvent,
    StaticCommands, StaticCommandsEvent, UpdateNpcEvent,
};

#[derive(
    Deserialize, Debug, Hash, Clone, PartialEq, Eq, derive_more::From, derive_more::Display,
)]
pub struct SceneSectionId(pub String);

impl Default for SceneSectionId {
    fn default() -> Self {
        Self("start".to_string())
    }
}

#[derive(
    Deserialize, Debug, Hash, Clone, PartialEq, Eq, derive_more::From, derive_more::Display,
)]
pub struct SceneId(pub String);

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
    #[serde(flatten)]
    definitions: Definitions,
    dialogue: HashMap<SceneSectionId, Dialogue>,
    #[serde(flatten)]
    commands: Option<StaticCommands>, // TODO: execute these?
}

#[derive(Deserialize, Debug, Clone, PartialEq, Eq)]
pub struct Character {
    pub name: String,
    #[serde(default)]
    pub image: NpcImage,
    #[serde(default)]
    pub voice: NpcVoice,
}

impl Default for Character {
    fn default() -> Self {
        Self {
            name: "?".to_string(),
            image: NpcImage::default(),
            voice: NpcVoice::default(),
        }
    }
}

#[derive(Deserialize, Debug, Clone, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub struct Dialogue {
    #[serde(default)]
    lines: Vec<Line>,
    #[serde(alias = "resp", default)]
    responses: Vec<Response>,
    #[serde(flatten)]
    commands: Option<StaticCommands>,
    #[serde(alias = "cont")]
    continue_to: Option<SceneSectionId>,
}

#[derive(Deserialize, Debug, Clone, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub struct Line {
    pub from: NpcId,
    #[serde(alias = "txt")]
    pub text: String,
    #[serde(flatten)]
    commands: Option<StaticCommands>,
}

#[derive(Deserialize, Debug, Clone, PartialEq, Eq)]
#[serde(rename_all = "kebab-case", deny_unknown_fields)]
pub struct Response {
    #[serde(alias = "txt")]
    pub text: String,
    #[serde(alias = "lnk")]
    link: Option<SceneSectionId>,
    skill_check: Option<SkillCheck>,
    #[serde(alias = "cond", default)]
    conditions: Vec<Condition>,
    #[serde(flatten)]
    commands: Option<StaticCommands>,
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

#[derive(Resource, Debug, Clone)]
pub struct ScenePlayer {
    scene: SceneId,
    current_key: SceneSectionId,
    current_line: usize,
    highlighted_response: usize,
    executed_commands: HashSet<SceneBookmark>,
}

impl ScenePlayer {
    fn new(scene: SceneId, start_key: Option<SceneSectionId>) -> Self {
        Self {
            scene,
            current_key: start_key.unwrap_or_default(),
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
        scene_commands_events: &mut EventWriter<StaticCommandsEvent>,
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
            scene_commands_events.write(StaticCommandsEvent(bookmark, commands));
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
        scene_commands_events: &mut EventWriter<StaticCommandsEvent>,
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
            scene_commands_events.write(StaticCommandsEvent(bookmark, commands));
        }

        // continue dialog
        if dialogue.lines.len() - 1 > self.current_line {
            self.advance_line();
        }
        // execute response
        else if let Some(response) = dialogue.responses.get(self.highlighted_response) {
            if let Some(commands) = response.commands.clone() {
                let bookmark = SceneBookmark::new(
                    &self.scene,
                    Some(&self.current_key),
                    None,
                    Some(self.highlighted_response),
                );
                scene_commands_events.write(StaticCommandsEvent(bookmark, commands));
            }

            // TODO: skill check, etc.
            if let Some(link) = response.link.as_ref() {
                self.set_key(link.to_owned());
            } else {
                end_scene_event.write(EndSceneEvent);
            }
        // continue to next section
        } else if let Some(ref section) = dialogue.continue_to {
            self.set_key(section.to_owned())
        }
        // end scene
        else {
            end_scene_event.write(EndSceneEvent);
        }
    }

    pub fn get_current<'a>(
        &'a self,
        scene_manager: &'a SceneManager,
        scene_commands_events: &mut EventWriter<StaticCommandsEvent>,
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
        scene_commands_events: &mut EventWriter<StaticCommandsEvent>,
    ) {
        let dialogue = self.get_dialogue(scene_manager, scene_commands_events);
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
            ScenePlayerInput::MoveTo(i) => self.highlighted_response = i,
            ScenePlayerInput::Select(i) => {
                self.highlighted_response = i;
                self.select(dialogue, end_scene_event, scene_commands_events);
            }
            ScenePlayerInput::Select(_) | ScenePlayerInput::SelectCurrent => {
                self.select(dialogue, end_scene_event, scene_commands_events);
            }
        }
    }

    pub fn highlighted_response(&self) -> usize {
        self.highlighted_response
    }

    pub fn execute(
        &mut self,
        bookmark: SceneBookmark,
        commands: StaticCommands,
        scene_manager: &mut SceneManager,
        start_battle_event: &mut EventWriter<StartBattleEvent>,
        update_npc_event: &mut EventWriter<UpdateNpcEvent>,
    ) {
        if self.executed_commands.insert(bookmark) {
            commands.execute(scene_manager, start_battle_event, update_npc_event);
        }
    }
}

#[derive(Clone, Copy, Debug, Hash, PartialEq, Eq)]
pub enum ScenePlayerInput {
    MoveUp,
    MoveDown,
    MoveTo(usize),
    Select(usize),
    SelectCurrent,
}

pub struct UiScenePart<'a> {
    pub line: &'a Line,
    pub responses: Option<Vec<&'a Response>>,
}

#[derive(Resource, Debug, Clone, Default)]
pub struct SceneManager {
    pub(crate) scenes: HashMap<SceneId, Scene>,
    pub(crate) variables: HashMap<String, String>,
    pub(crate) entries: HashMap<SceneId, SceneSectionId>,
}

impl SceneManager {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn load_scene<P: AsRef<Path>>(&mut self, path: P) -> anyhow::Result<()> {
        info!("loading scene: {:?}", path.as_ref());
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

    pub fn play_scene(
        &self,
        scene_id: SceneId,
        spawn_npc_event: &mut EventWriter<SpawnNpcEvent>,
    ) -> Option<ScenePlayer> {
        self.scenes.contains_key(&scene_id).then(|| {
            self.scenes[&scene_id].definitions.create(spawn_npc_event);
            let scene_entry = self.entries.get(&scene_id);
            ScenePlayer::new(scene_id, scene_entry.cloned())
        })
    }

    pub(crate) fn update_variables<U>(&mut self, variables: U)
    where
        U: IntoIterator<Item = (String, String)>,
    {
        self.variables.extend(variables)
    }

    fn get_variable(&self, variable: &str) -> Option<&String> {
        self.variables.get(variable)
    }

    pub(crate) fn update_scene_entry(
        &mut self,
        scene: SceneId,
        key: SceneSectionId,
    ) -> Option<SceneSectionId> {
        self.entries.insert(scene, key)
    }
}
