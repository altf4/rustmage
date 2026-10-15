//! Optional JSON control plane shared by ctypes and the diagnostic CLI. There is
//! no networking. Native Rust rollouts call Game directly, bypassing serialization.
use crate::{Action, Config, Game, Observation, Player, Snapshot};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

#[derive(Deserialize)]
#[serde(tag = "op", rename_all = "snake_case", deny_unknown_fields)]
pub enum Request {
    Observe {
        viewer: Player,
    },
    Step {
        actor: Player,
        revision: u64,
        index: usize,
        viewer: Player,
    },
    Act {
        actor: Player,
        revision: u64,
        action: Action,
        viewer: Player,
    },
    Snapshot,
    Restore {
        snapshot: u64,
    },
    DropSnapshot {
        snapshot: u64,
    },
}

#[derive(Serialize)]
pub struct Response {
    pub schema: u32,
    pub revision: u64,
    pub observation: Option<Observation>,
    pub snapshot: Option<u64>,
    pub error: Option<String>,
}

#[derive(Clone)]
pub struct Session {
    game: Game,
    snapshots: BTreeMap<u64, Snapshot>,
    next_snapshot: u64,
}

impl Session {
    pub fn new(config: Config) -> Result<Self, crate::Error> {
        Ok(Self {
            game: Game::new(config)?,
            snapshots: BTreeMap::new(),
            next_snapshot: 1,
        })
    }

    /// Fork current game only; stored checkpoints are deliberately not duplicated.
    pub fn fork(&self) -> Self {
        Self {
            game: self.game.clone(),
            snapshots: BTreeMap::new(),
            next_snapshot: 1,
        }
    }

    pub fn request(&mut self, request: Request) -> Response {
        let mut response = Response {
            schema: 1,
            revision: self.game.revision(),
            observation: None,
            snapshot: None,
            error: None,
        };
        let result = match request {
            Request::Observe { viewer } => {
                response.observation = Some(self.game.observe(viewer));
                Ok(())
            }
            Request::Step {
                actor,
                revision,
                index,
                viewer,
            } => {
                let result = self.game.apply_index(actor, revision, index);
                if result.is_ok() {
                    response.observation = Some(self.game.observe(viewer));
                }
                result.map_err(|e| e.to_string())
            }
            Request::Act {
                actor,
                revision,
                action,
                viewer,
            } => {
                let result = if revision != self.game.revision() {
                    Err(crate::Error::StaleRevision)
                } else {
                    self.game.apply(actor, action)
                };
                if result.is_ok() {
                    response.observation = Some(self.game.observe(viewer));
                }
                result.map_err(|e| e.to_string())
            }
            Request::Snapshot => {
                let id = self.next_snapshot;
                match id.checked_add(1) {
                    Some(next) => {
                        self.next_snapshot = next;
                        self.snapshots.insert(id, self.game.snapshot());
                        response.snapshot = Some(id);
                        Ok(())
                    }
                    None => Err("snapshot IDs exhausted".into()),
                }
            }
            Request::Restore { snapshot } => match self.snapshots.get(&snapshot) {
                Some(saved) => {
                    self.game.restore(saved);
                    Ok(())
                }
                None => Err("unknown snapshot".into()),
            },
            Request::DropSnapshot { snapshot } => match self.snapshots.remove(&snapshot) {
                Some(_) => Ok(()),
                None => Err("unknown snapshot".into()),
            },
        };
        response.revision = self.game.revision();
        response.error = result.err();
        response
    }

    pub fn json(&mut self, input: &str) -> String {
        match serde_json::from_str(input) {
            Ok(request) => serde_json::to_string(&self.request(request)).expect("response serialization"),
            Err(error) => serde_json::json!({ "schema": 1, "revision": self.game.revision(), "error": error.to_string() }).to_string(),
        }
    }
}
