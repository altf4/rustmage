use crate::cards::{Effect, payments};
use crate::rng::Rng;
use crate::{Card, Mana};
use serde::{Deserialize, Serialize};

pub type CardId = u32;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Player {
    P0,
    P1,
}

impl Player {
    pub fn other(self) -> Self {
        match self {
            Self::P0 => Self::P1,
            Self::P1 => Self::P0,
        }
    }
    pub fn index(self) -> usize {
        self as usize
    }
}

#[cfg(test)]
#[path = "engine_tests.rs"]
mod tests;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
pub enum Zone {
    Library,
    Hand,
    Battlefield,
    Stack,
    Graveyard,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
pub enum Step {
    Pregame,
    Upkeep,
    Draw,
    Main1,
    BeginCombat,
    Attackers,
    Blockers,
    CombatDamage,
    EndCombat,
    Main2,
    End,
    Cleanup,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Target {
    Player(Player),
    Card(CardId),
    Spell(CardId),
}

/// Choices are incremental for declarations and multi-blocker damage; no combinatorial
/// enumeration of attacker subsets, blocking configurations, or damage partitions.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind")]
pub enum Action {
    Keep,
    Mulligan,
    Bottom {
        card: CardId,
    },
    Pass,
    PlayLand {
        card: CardId,
    },
    TapMana {
        card: CardId,
    },
    Cast {
        card: CardId,
        target: Option<Target>,
        payment: Mana,
    },
    Attack {
        card: CardId,
    },
    FinishAttackers,
    Block {
        blocker: CardId,
        attacker: CardId,
    },
    FinishBlockers,
    AssignDamage {
        blocker: CardId,
        amount: i32,
    },
    Discard {
        card: CardId,
    },
    Concede,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
pub enum Outcome {
    Winner(Player),
    Draw,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Error {
    IllegalAction,
    WrongActor,
    GameOver,
    InvalidDeck(String),
    StaleRevision,
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{self:?}")
    }
}
impl std::error::Error for Error {}

/// Freeform is deliberately explicit: useful for training, not tournament deck legality.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum DeckRules {
    #[default]
    Constructed,
    Freeform,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Config {
    pub decks: [Vec<Card>; 2],
    pub seed: u64,
    #[serde(default = "first_player")]
    pub starting_player: Player,
    #[serde(default)]
    pub deck_rules: DeckRules,
}
fn first_player() -> Player {
    Player::P0
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct Object {
    card: Card,
    owner: Player,
    zone: Zone,
    tapped: bool,
    // True until this creature's controller begins their next turn.
    summoning_sick: bool,
    damage: i32,
    growth: i32,
}

impl Object {
    fn power(&self) -> i32 {
        match self.card.effect() {
            Effect::Creature(p, _) => p + self.growth,
            _ => 0,
        }
    }
    fn toughness(&self) -> i32 {
        match self.card.effect() {
            Effect::Creature(_, t) => t + self.growth,
            _ => 0,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct PlayerState {
    life: i32,
    mana: Mana,
    library: Vec<CardId>, // top is last, bottom is first
    mulligans: u8,
    kept: bool,
    attempted_empty_draw: bool,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct SpellView {
    pub card: CardId,
    pub controller: Player,
    pub target: Option<Target>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct Attacker {
    card: CardId,
    blocked: bool,
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct DamageChoice {
    blockers: Vec<CardId>,
    remaining: i32,
}

#[derive(Clone, Debug, PartialEq, Eq)]
enum Pending {
    Mulligan(Player),
    Bottom { player: Player, remaining: u8 },
    Priority,
    Attackers,
    Blockers,
    Damage,
    Discard,
    Terminal,
}

/// All mutable rules state, including every suspended choice and future randomness.
/// Card definitions are enum/static data, never cloned object graphs or trait objects.
#[derive(Clone, Debug, PartialEq, Eq)]
struct State {
    objects: Vec<Object>,
    players: [PlayerState; 2],
    rng: Rng,
    active: Player,
    priority: Player,
    starting: Player,
    turn: u32,
    step: Step,
    pending: Pending,
    mulligan_round: [bool; 2],
    stack: Vec<SpellView>,
    passes: u8,
    land_played: bool,
    attackers: Vec<Attacker>,
    blocks: Vec<(CardId, CardId)>, // blocker, attacker
    selected_attackers: Vec<CardId>,
    selected_blocks: Vec<(CardId, CardId)>,
    hits: Vec<(Target, i32)>,
    damage_choices: Vec<DamageChoice>,
    outcome: Option<Outcome>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Snapshot(State);

/// Cloning produces an independent search branch. Restore accepts a typed complete
/// snapshot and advances the external revision, so old action indices stay invalid.
#[derive(Clone, Debug)]
pub struct Game {
    state: State,
    revision: u64,
}

#[derive(Clone, Debug, Serialize)]
pub struct ObjectView {
    pub id: CardId,
    pub card: Card,
    pub owner: Player,
    pub zone: Zone,
    pub tapped: bool,
    pub summoning_sick: bool,
    pub damage: i32,
    pub power: i32,
    pub toughness: i32,
}

#[derive(Clone, Debug, Serialize)]
pub struct PlayerView {
    pub life: i32,
    pub mana: Mana,
    pub library_count: usize,
    pub hand_count: usize,
}

#[derive(Clone, Debug, Serialize)]
pub struct Observation {
    pub schema: u32,
    pub viewer: Player,
    pub revision: u64,
    pub actor: Option<Player>,
    pub active: Player,
    pub turn: u32,
    pub step: Step,
    pub decision: &'static str,
    pub players: [PlayerView; 2],
    pub objects: Vec<ObjectView>,
    pub stack: Vec<SpellView>,
    pub attackers: Vec<CardId>,
    pub blocks: Vec<(CardId, CardId)>,
    pub damage_assigned: Vec<(Target, i32)>,
    pub outcome: Option<Outcome>,
    pub actions: Vec<Action>,
}

impl Game {
    pub fn new(config: Config) -> Result<Self, Error> {
        for deck in &config.decks {
            // Explicit representation bound, never silent truncation.
            if deck.len() < 7 || deck.len() > 65_535 {
                return Err(Error::InvalidDeck(
                    "supported deck size is 7..=65535".into(),
                ));
            }
            if config.deck_rules == DeckRules::Constructed {
                if deck.len() < 60 {
                    return Err(Error::InvalidDeck("constructed needs 60 cards".into()));
                }
                for card in Card::ALL {
                    if !card.is_land() && deck.iter().filter(|&&c| c == card).count() > 4 {
                        return Err(Error::InvalidDeck(format!(
                            "more than four {}",
                            card.name()
                        )));
                    }
                }
            }
        }
        let mut objects = Vec::with_capacity(config.decks.iter().map(Vec::len).sum());
        let mut players = std::array::from_fn(|_| PlayerState {
            life: 20,
            mana: [0; 5],
            library: Vec::new(),
            mulligans: 0,
            kept: false,
            attempted_empty_draw: false,
        });
        let mut rng = Rng(config.seed);
        for (p, deck) in [Player::P0, Player::P1].into_iter().zip(config.decks) {
            for card in deck {
                let id = objects.len() as CardId;
                objects.push(Object {
                    card,
                    owner: p,
                    zone: Zone::Library,
                    tapped: false,
                    summoning_sick: false,
                    damage: 0,
                    growth: 0,
                });
                players[p.index()].library.push(id);
            }
            rng.shuffle(&mut players[p.index()].library);
        }
        let mut game = Self {
            revision: 0,
            state: State {
                objects,
                players,
                rng,
                active: config.starting_player,
                priority: config.starting_player,
                starting: config.starting_player,
                turn: 1,
                step: Step::Pregame,
                pending: Pending::Mulligan(config.starting_player),
                stack: Vec::new(),
                passes: 0,
                mulligan_round: [false; 2],
                land_played: false,
                attackers: Vec::new(),
                blocks: Vec::new(),
                selected_attackers: Vec::new(),
                selected_blocks: Vec::new(),
                hits: Vec::new(),
                damage_choices: Vec::new(),
                outcome: None,
            },
        };
        for p in [Player::P0, Player::P1] {
            for _ in 0..7 {
                game.draw(p);
            }
        }
        Ok(game)
    }

    pub fn revision(&self) -> u64 {
        self.revision
    }
    pub fn outcome(&self) -> Option<Outcome> {
        self.state.outcome
    }
    pub fn snapshot(&self) -> Snapshot {
        Snapshot(self.state.clone())
    }
    pub fn restore(&mut self, snapshot: &Snapshot) {
        self.state.clone_from(&snapshot.0);
        self.revision = self.revision.checked_add(1).expect("revision exhausted");
    }

    pub fn actor(&self) -> Option<Player> {
        match self.state.pending {
            Pending::Mulligan(p) | Pending::Bottom { player: p, .. } => Some(p),
            Pending::Priority => Some(self.state.priority),
            Pending::Attackers | Pending::Damage | Pending::Discard => Some(self.state.active),
            Pending::Blockers => Some(self.state.active.other()),
            Pending::Terminal => None,
        }
    }

    /// Reuse a caller-owned action buffer during native rollouts.
    pub fn legal_actions_into(&self, out: &mut Vec<Action>) {
        out.clear();
        if self.actor().is_none() {
            return;
        }
        self.visit_actions(|a| {
            out.push(a);
            false
        });
    }

    pub fn legal_actions(&self) -> Vec<Action> {
        let mut actions = Vec::new();
        self.legal_actions_into(&mut actions);
        actions
    }

    /// Mutation happens only after complete validation. Rejected actions leave state
    /// and revision unchanged. No user-supplied IDs are used to index before validation.
    pub fn apply(&mut self, actor: Player, action: Action) -> Result<(), Error> {
        if self.state.outcome.is_some() {
            return Err(Error::GameOver);
        }
        if action == Action::Concede {
            self.state.outcome = Some(Outcome::Winner(actor.other()));
            self.state.pending = Pending::Terminal;
        } else {
            if self.actor() != Some(actor) {
                return Err(Error::WrongActor);
            }
            if !self.visit_actions(|a| a == action) {
                return Err(Error::IllegalAction);
            }
            self.apply_legal(actor, action);
            self.state_based_actions();
        }
        self.revision = self.revision.checked_add(1).expect("revision exhausted");
        Ok(())
    }

    pub fn apply_index(&mut self, actor: Player, revision: u64, index: usize) -> Result<(), Error> {
        if revision != self.revision {
            return Err(Error::StaleRevision);
        }
        if self.actor() != Some(actor) {
            return Err(Error::WrongActor);
        }
        let mut n = 0;
        let mut found = None;
        self.visit_actions(|a| {
            if n == index {
                found = Some(a);
                true
            } else {
                n += 1;
                false
            }
        });
        self.apply(actor, found.ok_or(Error::IllegalAction)?)
    }

    fn visit_actions(&self, mut emit: impl FnMut(Action) -> bool) -> bool {
        let s = &self.state;
        let Some(actor) = self.actor() else {
            return false;
        };
        if emit(Action::Concede) {
            return true;
        }
        match s.pending {
            Pending::Mulligan(p) => {
                if emit(Action::Keep) {
                    return true;
                }
                if s.players[p.index()].mulligans < 7 && emit(Action::Mulligan) {
                    return true;
                }
            }
            Pending::Bottom { .. } | Pending::Discard => {
                for (i, obj) in s.objects.iter().enumerate() {
                    if obj.owner == actor && obj.zone == Zone::Hand {
                        let a = if matches!(s.pending, Pending::Bottom { .. }) {
                            Action::Bottom { card: i as CardId }
                        } else {
                            Action::Discard { card: i as CardId }
                        };
                        if emit(a) {
                            return true;
                        }
                    }
                }
            }
            Pending::Priority => {
                if emit(Action::Pass) {
                    return true;
                }
                let sorcery = actor == s.active
                    && matches!(s.step, Step::Main1 | Step::Main2)
                    && s.stack.is_empty();
                for (i, obj) in s.objects.iter().enumerate() {
                    if obj.owner != actor {
                        continue;
                    }
                    let card = i as CardId;
                    if obj.zone == Zone::Battlefield
                        && !obj.tapped
                        && obj.card.mana_color().is_some()
                        && (!obj.card.is_creature() || !obj.summoning_sick)
                        && emit(Action::TapMana { card })
                    {
                        return true;
                    }
                    if obj.zone != Zone::Hand {
                        continue;
                    }
                    if obj.card.is_land() {
                        if sorcery && !s.land_played && emit(Action::PlayLand { card }) {
                            return true;
                        }
                        continue;
                    }
                    if !sorcery && !obj.card.is_instant() {
                        continue;
                    }
                    let (colored, generic) = obj.card.cost();
                    let mut done = false;
                    payments(s.players[actor.index()].mana, colored, generic, |payment| {
                        if done {
                            return;
                        }
                        match obj.card.effect() {
                            Effect::Damage(_) => {
                                for p in [Player::P0, Player::P1] {
                                    if emit(Action::Cast {
                                        card,
                                        target: Some(Target::Player(p)),
                                        payment,
                                    }) {
                                        done = true;
                                        return;
                                    }
                                }
                                for (j, target) in s.objects.iter().enumerate() {
                                    if target.zone == Zone::Battlefield
                                        && target.card.is_creature()
                                        && emit(Action::Cast {
                                            card,
                                            target: Some(Target::Card(j as CardId)),
                                            payment,
                                        })
                                    {
                                        done = true;
                                        return;
                                    }
                                }
                            }
                            Effect::Grow => {
                                for (j, target) in s.objects.iter().enumerate() {
                                    if target.zone == Zone::Battlefield
                                        && target.card.is_creature()
                                        && emit(Action::Cast {
                                            card,
                                            target: Some(Target::Card(j as CardId)),
                                            payment,
                                        })
                                    {
                                        done = true;
                                        return;
                                    }
                                }
                            }
                            Effect::Counter => {
                                for target in &s.stack {
                                    if emit(Action::Cast {
                                        card,
                                        target: Some(Target::Spell(target.card)),
                                        payment,
                                    }) {
                                        done = true;
                                        return;
                                    }
                                }
                            }
                            _ => {
                                done = emit(Action::Cast {
                                    card,
                                    target: None,
                                    payment,
                                });
                            }
                        }
                    });
                    if done {
                        return true;
                    }
                }
            }
            Pending::Attackers => {
                if emit(Action::FinishAttackers) {
                    return true;
                }
                for (i, obj) in s.objects.iter().enumerate() {
                    let card = i as CardId;
                    if obj.owner == actor
                        && obj.zone == Zone::Battlefield
                        && obj.card.is_creature()
                        && !obj.tapped
                        && !obj.summoning_sick
                        && !s.selected_attackers.contains(&card)
                        && emit(Action::Attack { card })
                    {
                        return true;
                    }
                }
            }
            Pending::Blockers => {
                if emit(Action::FinishBlockers) {
                    return true;
                }
                for (i, obj) in s.objects.iter().enumerate() {
                    let blocker = i as CardId;
                    if obj.owner != actor
                        || obj.zone != Zone::Battlefield
                        || !obj.card.is_creature()
                        || obj.tapped
                        || s.selected_blocks.iter().any(|&(b, _)| b == blocker)
                    {
                        continue;
                    }
                    for attacker in &s.attackers {
                        if self.on_battlefield(attacker.card)
                            && emit(Action::Block {
                                blocker,
                                attacker: attacker.card,
                            })
                        {
                            return true;
                        }
                    }
                }
            }
            Pending::Damage => {
                let choice = s.damage_choices.last().expect("damage choice");
                let blocker = choice.blockers[0];
                for amount in 0..=choice.remaining {
                    if emit(Action::AssignDamage { blocker, amount }) {
                        return true;
                    }
                }
            }
            Pending::Terminal => {}
        }
        false
    }

    fn apply_legal(&mut self, actor: Player, action: Action) {
        match action {
            Action::Keep => {
                self.state.players[actor.index()].kept = true;
                self.next_mulligan(actor);
            }
            Action::Mulligan => {
                // Mulligan decisions for both players precede the actual redraw.
                // Flag the choice using kept=false; next_mulligan executes the round.
                self.state.players[actor.index()].mulligans += 1;
                self.state.mulligan_round[actor.index()] = true;
                self.next_mulligan(actor);
            }
            Action::Bottom { card } => {
                self.state.objects[card as usize].zone = Zone::Library;
                // Chosen first is put lowest. Subsequent choices go above prior ones.
                let already_bottomed = match self.state.pending {
                    Pending::Bottom { remaining, .. } => {
                        self.state.players[actor.index()].mulligans - remaining
                    }
                    _ => unreachable!(),
                } as usize;
                self.state.players[actor.index()]
                    .library
                    .insert(already_bottomed, card);
                if let Pending::Bottom { remaining, .. } = &mut self.state.pending {
                    *remaining -= 1;
                    if *remaining == 0 {
                        self.next_bottom(actor);
                    }
                }
            }
            Action::Pass => {
                self.state.passes += 1;
                if self.state.passes == 1 {
                    self.state.priority = actor.other();
                } else {
                    self.state.passes = 0;
                    if self.state.stack.is_empty() {
                        self.advance_step();
                    } else {
                        self.resolve_top();
                        self.give_priority();
                    }
                }
            }
            Action::PlayLand { card } => {
                self.state.objects[card as usize].zone = Zone::Battlefield;
                self.state.land_played = true;
                self.state.passes = 0;
            }
            Action::TapMana { card } => {
                let obj = &mut self.state.objects[card as usize];
                obj.tapped = true;
                self.state.players[actor.index()].mana[obj.card.mana_color().unwrap()] += 1;
                self.state.passes = 0;
            }
            Action::Cast {
                card,
                target,
                payment,
            } => {
                for (pool, cost) in self.state.players[actor.index()]
                    .mana
                    .iter_mut()
                    .zip(payment)
                {
                    *pool -= cost;
                }
                self.state.objects[card as usize].zone = Zone::Stack;
                self.state.stack.push(SpellView {
                    card,
                    controller: actor,
                    target,
                });
                self.state.passes = 0;
                // Casting player retains priority (117.3c).
            }
            Action::Attack { card } => self.state.selected_attackers.push(card),
            Action::FinishAttackers => {
                for card in self.state.selected_attackers.drain(..) {
                    self.state.objects[card as usize].tapped = true;
                    self.state.attackers.push(Attacker {
                        card,
                        blocked: false,
                    });
                }
                self.give_priority();
            }
            Action::Block { blocker, attacker } => {
                self.state.selected_blocks.push((blocker, attacker))
            }
            Action::FinishBlockers => {
                self.state.blocks.append(&mut self.state.selected_blocks);
                for attacker in &mut self.state.attackers {
                    attacker.blocked = self.state.blocks.iter().any(|&(_, a)| a == attacker.card);
                }
                self.give_priority();
            }
            Action::AssignDamage { blocker, amount } => {
                self.state.hits.push((Target::Card(blocker), amount));
                let choice = self.state.damage_choices.last_mut().unwrap();
                choice.remaining -= amount;
                choice.blockers.remove(0);
                if choice.blockers.len() == 1 {
                    self.state
                        .hits
                        .push((Target::Card(choice.blockers[0]), choice.remaining));
                    self.state.damage_choices.pop();
                }
                if self.state.damage_choices.is_empty() {
                    self.deal_combat_damage();
                }
            }
            Action::Discard { card } => {
                self.state.objects[card as usize].zone = Zone::Graveyard;
                if self.hand_count(actor) <= 7 {
                    self.finish_cleanup();
                }
            }
            Action::Concede => unreachable!(),
        }
    }

    fn next_mulligan(&mut self, actor: Player) {
        let start = self.state.starting;
        if actor == start && !self.state.players[start.other().index()].kept {
            self.state.pending = Pending::Mulligan(start.other());
            return;
        }
        // Everyone who chose to mulligan shuffles and draws simultaneously.
        for p in [start, start.other()] {
            if self.state.mulligan_round[p.index()] {
                for (i, obj) in self.state.objects.iter_mut().enumerate() {
                    if obj.owner == p && obj.zone == Zone::Hand {
                        obj.zone = Zone::Library;
                        self.state.players[p.index()].library.push(i as CardId);
                    }
                }
                self.state
                    .rng
                    .shuffle(&mut self.state.players[p.index()].library);
                for _ in 0..7 {
                    self.draw(p);
                }
            }
        }
        for p in [start, start.other()] {
            if self.state.mulligan_round[p.index()] {
                self.state.pending = Pending::Bottom {
                    player: p,
                    remaining: self.state.players[p.index()].mulligans,
                };
                return;
            }
        }
        self.begin_mulligan_round();
    }

    fn next_bottom(&mut self, actor: Player) {
        self.state.mulligan_round[actor.index()] = false;
        let other = self.state.starting.other();
        if actor == self.state.starting && self.state.mulligan_round[other.index()] {
            self.state.pending = Pending::Bottom {
                player: other,
                remaining: self.state.players[other.index()].mulligans,
            };
        } else {
            self.begin_mulligan_round();
        }
    }

    fn begin_mulligan_round(&mut self) {
        for p in [self.state.starting, self.state.starting.other()] {
            if self.state.players[p.index()].mulligans == 7 {
                self.state.players[p.index()].kept = true;
            }
            if !self.state.players[p.index()].kept {
                self.state.pending = Pending::Mulligan(p);
                return;
            }
        }
        self.begin_turn();
    }

    fn hand_count(&self, p: Player) -> usize {
        self.state
            .objects
            .iter()
            .filter(|o| o.owner == p && o.zone == Zone::Hand)
            .count()
    }

    fn draw(&mut self, p: Player) {
        match self.state.players[p.index()].library.pop() {
            Some(id) => self.state.objects[id as usize].zone = Zone::Hand,
            None => self.state.players[p.index()].attempted_empty_draw = true,
        }
    }

    fn on_battlefield(&self, id: CardId) -> bool {
        self.state
            .objects
            .get(id as usize)
            .is_some_and(|o| o.zone == Zone::Battlefield)
    }

    fn give_priority(&mut self) {
        self.state.pending = Pending::Priority;
        self.state.priority = self.state.active;
        self.state.passes = 0;
    }

    fn begin_turn(&mut self) {
        self.state.land_played = false;
        self.state.attackers.clear();
        self.state.blocks.clear();
        for obj in &mut self.state.objects {
            if obj.owner == self.state.active && obj.zone == Zone::Battlefield {
                obj.tapped = false;
                obj.summoning_sick = false;
            }
        }
        self.state.step = Step::Upkeep;
        self.give_priority();
    }

    fn advance_step(&mut self) {
        for player in &mut self.state.players {
            player.mana = [0; 5];
        }
        match self.state.step {
            Step::Upkeep => {
                if self.state.turn == 1 {
                    self.state.step = Step::Main1;
                } else {
                    self.state.step = Step::Draw;
                    self.draw(self.state.active);
                }
                self.give_priority();
            }
            Step::Draw => {
                self.state.step = Step::Main1;
                self.give_priority();
            }
            Step::Main1 => {
                self.state.step = Step::BeginCombat;
                self.give_priority();
            }
            Step::BeginCombat => {
                self.state.step = Step::Attackers;
                self.state.pending = Pending::Attackers;
            }
            Step::Attackers => {
                if self.state.attackers.is_empty() {
                    self.state.step = Step::EndCombat;
                    self.give_priority();
                } else {
                    self.state.step = Step::Blockers;
                    self.state.pending = Pending::Blockers;
                }
            }
            Step::Blockers => {
                self.state.step = Step::CombatDamage;
                self.prepare_damage();
            }
            Step::CombatDamage => {
                self.state.step = Step::EndCombat;
                self.give_priority();
            }
            Step::EndCombat => {
                self.state.attackers.clear();
                self.state.blocks.clear();
                self.state.step = Step::Main2;
                self.give_priority();
            }
            Step::Main2 => {
                self.state.step = Step::End;
                self.give_priority();
            }
            Step::End => {
                self.state.step = Step::Cleanup;
                if self.hand_count(self.state.active) > 7 {
                    self.state.pending = Pending::Discard;
                } else {
                    self.finish_cleanup();
                }
            }
            Step::Cleanup => self.finish_cleanup(),
            Step::Pregame => unreachable!(),
        }
    }

    fn finish_cleanup(&mut self) {
        // Damage removal and expiration of until-end-of-turn effects are simultaneous.
        for obj in &mut self.state.objects {
            obj.damage = 0;
            obj.growth = 0;
        }
        // Supported creatures all have positive printed toughness and there are no
        // cleanup triggers; consequently no extra cleanup-priority cycle is needed.
        self.state.active = self.state.active.other();
        self.state.turn += 1;
        self.begin_turn();
    }

    fn resolve_top(&mut self) {
        let spell = self.state.stack.pop().unwrap();
        let effect = self.state.objects[spell.card as usize].card.effect();
        let valid = match spell.target {
            Some(Target::Card(id)) => {
                self.on_battlefield(id) && self.state.objects[id as usize].card.is_creature()
            }
            Some(Target::Spell(id)) => self.state.stack.iter().any(|s| s.card == id),
            _ => true,
        };
        if valid {
            match effect {
                Effect::Creature(..) => {
                    let obj = &mut self.state.objects[spell.card as usize];
                    obj.zone = Zone::Battlefield;
                    obj.summoning_sick = true;
                    return;
                }
                Effect::Damage(amount) => self.damage(spell.target.unwrap(), amount),
                Effect::Grow => {
                    if let Some(Target::Card(id)) = spell.target {
                        self.state.objects[id as usize].growth += 3;
                    }
                }
                Effect::Draw => {
                    self.draw(spell.controller);
                    self.draw(spell.controller);
                }
                Effect::Counter => {
                    if let Some(Target::Spell(id)) = spell.target {
                        self.state.stack.retain(|s| s.card != id);
                        self.state.objects[id as usize].zone = Zone::Graveyard;
                    }
                }
                Effect::Land => unreachable!(),
            }
        }
        self.state.objects[spell.card as usize].zone = Zone::Graveyard;
    }

    fn damage(&mut self, target: Target, amount: i32) {
        match target {
            Target::Player(p) => self.state.players[p.index()].life -= amount,
            Target::Card(id) => self.state.objects[id as usize].damage += amount,
            Target::Spell(_) => unreachable!(),
        }
    }

    fn prepare_damage(&mut self) {
        self.state.hits.clear();
        self.state.damage_choices.clear();
        for attacker in &self.state.attackers {
            if !self.on_battlefield(attacker.card) {
                continue;
            }
            let power = self.state.objects[attacker.card as usize].power().max(0);
            let blockers: Vec<_> = self
                .state
                .blocks
                .iter()
                .filter(|&&(b, a)| a == attacker.card && self.on_battlefield(b))
                .map(|&(b, _)| b)
                .collect();
            for &b in &blockers {
                self.state.hits.push((
                    Target::Card(attacker.card),
                    self.state.objects[b as usize].power().max(0),
                ));
            }
            if !attacker.blocked {
                self.state
                    .hits
                    .push((Target::Player(self.state.active.other()), power));
            } else if blockers.len() == 1 {
                self.state.hits.push((Target::Card(blockers[0]), power));
            } else if blockers.len() > 1 {
                self.state.damage_choices.push(DamageChoice {
                    blockers,
                    remaining: power,
                });
            }
        }
        if self.state.damage_choices.is_empty() {
            self.deal_combat_damage();
        } else {
            self.state.pending = Pending::Damage;
        }
    }

    fn deal_combat_damage(&mut self) {
        let hits = std::mem::take(&mut self.state.hits);
        for (target, amount) in hits {
            self.damage(target, amount);
        }
        self.give_priority();
    }

    fn state_based_actions(&mut self) {
        for obj in &mut self.state.objects {
            if obj.zone == Zone::Battlefield
                && obj.card.is_creature()
                && (obj.toughness() <= 0 || obj.damage >= obj.toughness())
            {
                obj.zone = Zone::Graveyard;
                obj.damage = 0;
                obj.growth = 0;
                obj.tapped = false;
                obj.summoning_sick = false;
            }
        }
        let lost = self
            .state
            .players
            .each_ref()
            .map(|p| p.life <= 0 || p.attempted_empty_draw);
        self.state.outcome = match lost {
            [true, true] => Some(Outcome::Draw),
            [true, false] => Some(Outcome::Winner(Player::P1)),
            [false, true] => Some(Outcome::Winner(Player::P0)),
            _ => None,
        };
        if self.state.outcome.is_some() {
            self.state.pending = Pending::Terminal;
        }
    }

    /// Player-facing information only. Opponent hands and all library identities
    /// and RNG state are absent; partial declarations are visible only to their actor.
    pub fn observe(&self, viewer: Player) -> Observation {
        let s = &self.state;
        let owns_choice = self.actor() == Some(viewer);
        let objects = s
            .objects
            .iter()
            .enumerate()
            .filter(|(_, o)| o.zone != Zone::Library && (o.zone != Zone::Hand || o.owner == viewer))
            .map(|(i, o)| ObjectView {
                id: i as CardId,
                card: o.card,
                owner: o.owner,
                zone: o.zone,
                tapped: o.tapped,
                summoning_sick: o.summoning_sick,
                damage: o.damage,
                power: o.power(),
                toughness: o.toughness(),
            })
            .collect();
        Observation {
            schema: 1,
            viewer,
            revision: self.revision,
            actor: self.actor(),
            active: s.active,
            turn: s.turn,
            step: s.step,
            decision: match s.pending {
                Pending::Mulligan(_) => "mulligan",
                Pending::Bottom { .. } => "bottom",
                Pending::Priority => "priority",
                Pending::Attackers => "attackers",
                Pending::Blockers => "blockers",
                Pending::Damage => "damage",
                Pending::Discard => "discard",
                Pending::Terminal => "terminal",
            },
            players: std::array::from_fn(|i| PlayerView {
                life: s.players[i].life,
                mana: s.players[i].mana,
                library_count: s.players[i].library.len(),
                hand_count: self.hand_count(if i == 0 { Player::P0 } else { Player::P1 }),
            }),
            objects,
            stack: s.stack.clone(),
            attackers: if owns_choice && s.pending == Pending::Attackers {
                s.selected_attackers.clone()
            } else {
                s.attackers.iter().map(|a| a.card).collect()
            },
            blocks: if owns_choice && s.pending == Pending::Blockers {
                s.selected_blocks.clone()
            } else {
                s.blocks.clone()
            },
            damage_assigned: s.hits.clone(),
            outcome: s.outcome,
            actions: if owns_choice {
                self.legal_actions()
            } else {
                Vec::new()
            },
        }
    }

    /// Bytes occupied by state structs plus initialized vector elements, excluding
    /// allocator overhead, spare capacity, snapshots, and static card definitions.
    pub fn state_bytes(&self) -> usize {
        let s = &self.state;
        std::mem::size_of::<State>()
            + s.objects.len() * std::mem::size_of::<Object>()
            + s.players
                .iter()
                .map(|p| p.library.len() * std::mem::size_of::<CardId>())
                .sum::<usize>()
            + s.stack.len() * std::mem::size_of::<SpellView>()
            + s.attackers.len() * std::mem::size_of::<Attacker>()
            + s.blocks.len() * std::mem::size_of::<(CardId, CardId)>()
            + s.selected_attackers.len() * std::mem::size_of::<CardId>()
            + s.selected_blocks.len() * std::mem::size_of::<(CardId, CardId)>()
            + s.hits.len() * std::mem::size_of::<(Target, i32)>()
            + s.damage_choices.len() * std::mem::size_of::<DamageChoice>()
            + s.damage_choices
                .iter()
                .map(|d| d.blockers.len() * std::mem::size_of::<CardId>())
                .sum::<usize>()
    }
}
