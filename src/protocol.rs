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
    pub capacity: Option<crate::CapacityExceeded>,
}

#[derive(Clone)]
pub struct Session {
    game: Game,
    snapshots: BTreeMap<u64, Snapshot>,
    next_snapshot: u64,
}

impl Session {
    pub fn new(config: Config) -> Result<Self, crate::Error> {
        Ok(Self::from_game(Game::new(config)?))
    }
    /// Wrap an existing owned native game in the optional control interface.
    pub fn from_game(game: Game) -> Self {
        Self {
            game,
            snapshots: BTreeMap::new(),
            next_snapshot: 1,
        }
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
            schema: 2,
            revision: self.game.revision(),
            observation: None,
            snapshot: None,
            error: None,
            capacity: None,
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
                result.map_err(|e| {
                    if let crate::Error::CapacityExceeded(capacity) = &e {
                        response.capacity = Some(*capacity);
                    }
                    e.to_string()
                })
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
                result.map_err(|e| {
                    if let crate::Error::CapacityExceeded(capacity) = &e {
                        response.capacity = Some(*capacity);
                    }
                    e.to_string()
                })
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
                    None => {
                        response.capacity = Some(crate::CapacityExceeded {
                            resource: "snapshot identity exhausted",
                        });
                        Err("CapacityExceeded: snapshot identity exhausted".into())
                    }
                }
            }
            Request::Restore { snapshot } => match self.snapshots.get(&snapshot) {
                Some(saved) => self.game.restore(saved).map_err(|e| {
                    if let crate::Error::CapacityExceeded(capacity) = &e {
                        response.capacity = Some(*capacity);
                    }
                    e.to_string()
                }),
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
            Err(error) => serde_json::json!({ "schema": 2, "revision": self.game.revision(), "error": error.to_string() }).to_string(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn snapshot_identity_exhaustion_is_typed_and_keeps_the_handle_usable() {
        let mut session = Session::new(Config {
            decks: [vec![crate::Card::Forest; 60], vec![crate::Card::Forest; 60]],
            seed: 1,
            starting_player: Player::P0,
            deck_rules: crate::DeckRules::Freeform,
        })
        .unwrap();
        let saved = session.request(Request::Snapshot).snapshot.unwrap();
        let before = session.json(r#"{"op":"observe","viewer":"P0"}"#);
        session.next_snapshot = u64::MAX;
        for _ in 0..2 {
            let error = session.request(Request::Snapshot);
            assert_eq!(
                error.capacity,
                Some(crate::CapacityExceeded {
                    resource: "snapshot identity exhausted"
                })
            );
            assert_eq!(error.schema, 2);
            assert_eq!(session.json(r#"{"op":"observe","viewer":"P0"}"#), before);
            assert!(session.snapshots.contains_key(&saved));
            assert_eq!(session.next_snapshot, u64::MAX);
        }
        assert!(
            session
                .request(Request::Restore { snapshot: saved })
                .error
                .is_none()
        );
        assert!(
            session
                .request(Request::DropSnapshot { snapshot: saved })
                .error
                .is_none()
        );
    }
}
