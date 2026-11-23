//! Optional JSON control plane shared by ctypes and the diagnostic CLI. There is
//! no networking. Native Rust rollouts call Game directly, bypassing serialization.
use crate::{Action, AutoTapOptions, AutoTapResult, Config, Game, Observation, Player, Snapshot};
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
    PlanManaPayment(AutoTapRequest),
    AutoPayMana(AutoTapRequest),
    Snapshot,
    Restore {
        snapshot: u64,
    },
    DropSnapshot {
        snapshot: u64,
    },
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AutoTapRequest {
    pub actor: Player,
    pub revision: u64,
    pub attempt_id: u64,
    #[serde(default)]
    pub options: AutoTapOptions,
}

#[derive(Serialize)]
pub struct Response {
    pub schema: u32,
    pub revision: u64,
    pub observation: Option<Observation>,
    pub snapshot: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub auto_tap: Option<AutoTapResult>,
    pub error: Option<String>,
    pub capacity: Option<crate::CapacityExceeded>,
}

impl Response {
    fn engine_error(&mut self, error: crate::Error) -> String {
        if let crate::Error::CapacityExceeded(capacity) = &error {
            self.capacity = Some(*capacity);
        }
        error.to_string()
    }
}

/// Session-only JSON settings. Native Rust callers keep using `Config`.
#[derive(Deserialize)]
pub struct SessionConfig {
    #[serde(flatten)]
    config: Config,
    #[serde(default)]
    proposal_affordability: bool,
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
    pub fn new_with_outside(
        config: Config,
        outside: crate::OutsideGameConfig,
    ) -> Result<Self, crate::Error> {
        Ok(Self::from_game(Game::new_with_outside(config, outside)?))
    }
    pub fn from_config(
        config: SessionConfig,
        outside: Option<crate::OutsideGameConfig>,
    ) -> Result<Self, crate::Error> {
        let SessionConfig {
            config,
            proposal_affordability,
        } = config;
        let game = match outside {
            Some(outside) => Game::new_with_outside(config, outside)?,
            None => Game::new(config)?,
        };
        Ok(Self::from_game(
            game.with_proposal_affordability(proposal_affordability),
        ))
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
            auto_tap: None,
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
                result.map_err(|e| response.engine_error(e))
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
                result.map_err(|e| response.engine_error(e))
            }
            Request::PlanManaPayment(request) => self
                .game
                .plan_mana_payment(
                    request.actor,
                    request.revision,
                    request.attempt_id,
                    &request.options,
                )
                .map(|plan| response.auto_tap = Some(plan))
                .map_err(|e| response.engine_error(e)),
            Request::AutoPayMana(request) => self
                .game
                .auto_pay_mana(
                    request.actor,
                    request.revision,
                    request.attempt_id,
                    &request.options,
                )
                .map(|plan| {
                    response.auto_tap = Some(plan);
                    response.observation = Some(self.game.observe(request.actor));
                })
                .map_err(|e| response.engine_error(e)),
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
                Some(saved) => self
                    .game
                    .restore(saved)
                    .map_err(|e| response.engine_error(e)),
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
