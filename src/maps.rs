use std::path::{Path, PathBuf};

use bevy::{
    log::{error, info},
    platform::collections::HashMap,
    prelude::Resource,
};
use serde::Deserialize;

use crate::{Definitions, StaticCommands, TODO, types::Position};

#[derive(
    Deserialize, Debug, Hash, Clone, PartialEq, Eq, derive_more::From, derive_more::Display,
)]
pub struct MapId(pub String);

impl MapId {
    pub fn new(s: &str) -> Self {
        Self(s.to_string())
    }
}

#[derive(Deserialize, Debug, Clone, PartialEq, Eq)]
pub struct MapLayers {
    ground: PathBuf,
    base: PathBuf,
    sky: Option<PathBuf>,
}

#[derive(Deserialize, Debug, Clone, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub enum ActionPosition {
    #[serde(alias = "pos")]
    Position(Position),
    #[serde(alias = "pos-range")]
    Range { start: Position, end: Position },
}

#[derive(Deserialize, Debug, Clone, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub struct MapAction {
    #[serde(flatten)]
    position: ActionPosition,
    name: String,
    #[serde(alias = "cond")]
    condition: Option<TODO>,
    #[serde(flatten)]
    commands: StaticCommands,
}

#[derive(Deserialize, Debug, Clone, PartialEq, Eq)]
#[serde(rename_all = "kebab-case", deny_unknown_fields)]
pub struct Map {
    id: MapId,
    music: TODO,
    layers: MapLayers,
    #[serde(flatten)]
    definitions: Definitions,
    #[serde(alias = "player-pos")]
    player_position: Position,
    actions: Vec<MapAction>,
}

#[derive(Resource, Debug, Clone, Default)]
pub struct MapManager {
    pub(crate) maps: HashMap<MapId, Map>,
}

impl MapManager {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn load_map<P: AsRef<Path>>(&mut self, path: P) -> anyhow::Result<()> {
        info!("loading map: {:?}", path.as_ref());
        let map_json = std::fs::read_to_string(path)?;
        let map: Map = serde_json::from_str(&map_json)?;
        self.maps.insert(map.id.clone(), map);
        Ok(())
    }

    pub fn with_load_map<P: AsRef<Path>>(mut self, path: P) -> anyhow::Result<Self> {
        self.load_map(path)?;
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
                if let Err(e) = self.load_map(entry.path()) {
                    error!("failed to load map: {e}");
                }
            });
        Ok(())
    }

    pub fn with_load_folder<P: AsRef<Path>>(mut self, path: P) -> anyhow::Result<Self> {
        self.load_folder(path)?;
        Ok(self)
    }
}
