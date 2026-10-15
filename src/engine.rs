use crate::cards::{Effect, payments};
use crate::rng::Rng;
use crate::{Card, Keywords, Mana};
use serde::{Deserialize, Serialize};

pub type CardId = u32;

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
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
#[path = "engine_damage_tests.rs"]
mod damage_tests;
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
    FirstStrikeDamage,
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
        target: Target,
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
    /// Standard pool frozen at the compiled catalogue version, not host time.
    Standard,
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
    deathtouch_since_sba: bool,
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
    life: i64,
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
    source: DamageSource,
    blockers: Vec<CardId>,
    remaining: i32,
    trample: bool,
    prior_blockers_lethal: bool,
}

/// Source characteristics captured when a damage event is constructed. Damage
/// need not originate from a permanent that survives the simultaneous batch.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize)]
pub struct DamageSource {
    pub object: CardId,
    pub controller: Player,
    pub keywords: Keywords,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
pub struct DamageEvent {
    pub source: DamageSource,
    pub target: Target,
    pub amount: i32,
    pub combat: bool,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct LifeGainEvent {
    source: DamageSource,
    amount: i32,
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
    hits: Vec<DamageEvent>,
    damage_choices: Vec<DamageChoice>,
    first_strikers: Vec<CardId>,
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
    pub keywords: Keywords,
}

#[derive(Clone, Debug, Serialize)]
pub struct PlayerView {
    pub life: i64,
    pub mana: Mana,
    pub library_count: usize,
    pub hand_count: usize,
}

#[derive(Clone, Debug, Serialize)]
pub struct Observation {
    pub schema: u32,
    pub catalogue: &'static str,
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
    pub damage_assigned: Vec<DamageEvent>,
    pub damage_source: Option<CardId>,
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
            if config.deck_rules != DeckRules::Freeform {
                if deck.len() < 60 {
                    return Err(Error::InvalidDeck("constructed needs 60 cards".into()));
                }
                let mut counts = vec![0u8; Card::catalogue_len()];
                for &card in deck {
                    if let Some(limit) = card.info().deck_limit {
                        counts[card.index()] = counts[card.index()].saturating_add(1);
                        if counts[card.index()] > limit {
                            return Err(Error::InvalidDeck(format!(
                                "more than {limit} {}",
                                card.name()
                            )));
                        }
                    }
                }
            }
            for &card in deck {
                if config.deck_rules == DeckRules::Standard && !card.is_standard_legal() {
                    return Err(Error::InvalidDeck(format!(
                        "not Standard legal in this catalogue: {}",
                        card.name()
                    )));
                }
                if !card.is_supported() {
                    return Err(Error::InvalidDeck(format!(
                        "rules not implemented: {}",
                        card.name()
                    )));
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
                    deathtouch_since_sba: false,
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
                first_strikers: Vec::new(),
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
                        && (!obj.card.is_creature()
                            || !obj.summoning_sick
                            || obj.card.keywords().contains(Keywords::HASTE))
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
                    if !sorcery
                        && !obj.card.is_instant()
                        && !obj.card.keywords().contains(Keywords::FLASH)
                    {
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
                        && (!obj.summoning_sick || obj.card.keywords().contains(Keywords::HASTE))
                        && !obj.card.keywords().contains(Keywords::DEFENDER)
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
                            && (!s.objects[attacker.card as usize]
                                .card
                                .keywords()
                                .contains(Keywords::FLYING)
                                || obj.card.keywords().contains(Keywords::FLYING)
                                || obj.card.keywords().contains(Keywords::REACH))
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
                let target = Target::Card(choice.blockers[0]);
                let minimum = if choice.blockers.len() == 1 && choice.trample {
                    if choice.prior_blockers_lethal {
                        self.lethal_assignment(choice.source, choice.blockers[0])
                            .min(choice.remaining)
                    } else {
                        choice.remaining
                    }
                } else {
                    0
                };
                for amount in minimum..=choice.remaining {
                    if emit(Action::AssignDamage { target, amount }) {
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
                    let obj = &mut self.state.objects[card as usize];
                    if !obj.card.keywords().contains(Keywords::VIGILANCE) {
                        obj.tapped = true;
                    }
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
            Action::AssignDamage { target, amount } => {
                let Target::Card(blocker) = target else {
                    unreachable!()
                };
                let source = self.state.damage_choices.last().unwrap().source;
                let lethal = amount >= self.lethal_assignment(source, blocker);
                self.queue_hit(source, target, amount, true);
                let choice = self.state.damage_choices.last_mut().unwrap();
                choice.remaining -= amount;
                choice.blockers.remove(0);
                choice.prior_blockers_lethal &= lethal;
                if choice.blockers.is_empty() {
                    let choice = self.state.damage_choices.pop().unwrap();
                    self.queue_hit(
                        source,
                        Target::Player(self.state.active.other()),
                        choice.remaining,
                        true,
                    );
                }
                self.advance_damage_choices();
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
        self.state.first_strikers.clear();
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
                self.state.first_strikers = self
                    .combatants()
                    .into_iter()
                    .filter(|&id| {
                        let keywords = self.state.objects[id as usize].card.keywords();
                        keywords.contains(Keywords::FIRST_STRIKE)
                            || keywords.contains(Keywords::DOUBLE_STRIKE)
                    })
                    .collect();
                self.state.step = if self.state.first_strikers.is_empty() {
                    Step::CombatDamage
                } else {
                    Step::FirstStrikeDamage
                };
                self.prepare_damage();
            }
            Step::FirstStrikeDamage => {
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
                self.state.first_strikers.clear();
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
            obj.deathtouch_since_sba = false;
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
                Effect::Damage(amount) => {
                    let event = DamageEvent {
                        source: self.damage_source(spell.card),
                        target: spell.target.unwrap(),
                        amount,
                        combat: false,
                    };
                    self.apply_damage_batch(&[event]);
                }
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
                Effect::Land | Effect::Unsupported => {
                    unreachable!("unsupported card passed admission")
                }
            }
        }
        self.state.objects[spell.card as usize].zone = Zone::Graveyard;
    }

    fn damage_source(&self, object: CardId) -> DamageSource {
        let obj = &self.state.objects[object as usize];
        // Control-changing cards are not admitted yet. This is the single place
        // where current controller must be used once control layers are added.
        DamageSource {
            object,
            controller: obj.owner,
            keywords: obj.card.keywords(),
        }
    }

    fn queue_hit(&mut self, source: DamageSource, target: Target, amount: i32, combat: bool) {
        if amount > 0 {
            self.state.hits.push(DamageEvent {
                source,
                target,
                amount,
                combat,
            });
        }
    }

    fn apply_damage_batch(&mut self, events: &[DamageEvent]) -> Vec<LifeGainEvent> {
        let mut gains: Vec<LifeGainEvent> = Vec::new();
        let mut gain_indices: std::collections::BTreeMap<DamageSource, usize> =
            std::collections::BTreeMap::new();
        for event in events {
            if event.amount <= 0 {
                continue;
            }
            match event.target {
                Target::Player(p) => self.state.players[p.index()].life -= i64::from(event.amount),
                Target::Card(id) => {
                    let obj = &mut self.state.objects[id as usize];
                    obj.damage += event.amount;
                    obj.deathtouch_since_sba |=
                        event.source.keywords.contains(Keywords::DEATHTOUCH);
                }
                Target::Spell(_) => unreachable!("damage cannot target stack objects"),
            }
            if event.source.keywords.contains(Keywords::LIFELINK) {
                // One source damaging multiple objects simultaneously causes a
                // single life-gain event; distinct sources remain distinct.
                if let Some(&index) = gain_indices.get(&event.source) {
                    gains[index].amount += event.amount;
                } else {
                    gain_indices.insert(event.source, gains.len());
                    gains.push(LifeGainEvent {
                        source: event.source,
                        amount: event.amount,
                    });
                }
            }
        }
        for gain in &gains {
            self.state.players[gain.source.controller.index()].life += i64::from(gain.amount);
        }
        // Future trigger dispatch consumes damage + gain events here, before
        // their storage is discarded. SBAs run only after the whole batch.
        gains
    }

    fn combatants(&self) -> Vec<CardId> {
        self.state
            .attackers
            .iter()
            .map(|attacker| attacker.card)
            .chain(self.state.blocks.iter().map(|&(blocker, _)| blocker))
            .filter(|&id| self.on_battlefield(id))
            .collect()
    }

    fn assigns_this_step(&self, id: CardId) -> bool {
        if self.state.step == Step::FirstStrikeDamage {
            self.state.first_strikers.contains(&id)
        } else {
            !self.state.first_strikers.contains(&id)
                || self.state.objects[id as usize]
                    .card
                    .keywords()
                    .contains(Keywords::DOUBLE_STRIKE)
        }
    }

    fn lethal_assignment(&self, source: DamageSource, blocker: CardId) -> i32 {
        let obj = &self.state.objects[blocker as usize];
        let remaining = (obj.toughness() - obj.damage).max(0);
        if source.keywords.contains(Keywords::DEATHTOUCH) {
            remaining.min(1)
        } else {
            remaining
        }
    }

    fn prepare_damage(&mut self) {
        self.state.hits.clear();
        self.state.damage_choices.clear();
        // Clone only the declaration list so construction may append events.
        for attacker in self.state.attackers.clone() {
            if !self.on_battlefield(attacker.card) {
                continue;
            }
            let blockers: Vec<_> = self
                .state
                .blocks
                .iter()
                .filter(|&&(b, a)| a == attacker.card && self.on_battlefield(b))
                .map(|&(b, _)| b)
                .collect();
            for &blocker in &blockers {
                if self.assigns_this_step(blocker) {
                    let source = self.damage_source(blocker);
                    let amount = self.state.objects[blocker as usize].power().max(0);
                    self.queue_hit(source, Target::Card(attacker.card), amount, true);
                }
            }
            if !self.assigns_this_step(attacker.card) {
                continue;
            }
            let source = self.damage_source(attacker.card);
            let power = self.state.objects[attacker.card as usize].power().max(0);
            let trample = source.keywords.contains(Keywords::TRAMPLE);
            if !attacker.blocked || (trample && blockers.is_empty()) {
                self.queue_hit(
                    source,
                    Target::Player(self.state.active.other()),
                    power,
                    true,
                );
            } else if !blockers.is_empty() && power > 0 {
                self.state.damage_choices.push(DamageChoice {
                    source,
                    blockers,
                    remaining: power,
                    trample,
                    prior_blockers_lethal: true,
                });
            }
        }
        self.advance_damage_choices();
    }

    fn advance_damage_choices(&mut self) {
        while let Some(choice) = self.state.damage_choices.last() {
            let forced = choice.remaining == 0
                || (choice.blockers.len() == 1
                    && (!choice.trample
                        || !choice.prior_blockers_lethal
                        || choice.remaining
                            <= self.lethal_assignment(choice.source, choice.blockers[0])));
            if !forced {
                self.state.pending = Pending::Damage;
                return;
            }
            let choice = self.state.damage_choices.pop().unwrap();
            self.queue_hit(
                choice.source,
                Target::Card(choice.blockers[0]),
                choice.remaining,
                true,
            );
        }
        self.deal_combat_damage();
    }

    fn deal_combat_damage(&mut self) {
        let hits = std::mem::take(&mut self.state.hits);
        self.apply_damage_batch(&hits);
        self.give_priority();
    }

    fn state_based_actions(&mut self) {
        for obj in &mut self.state.objects {
            if obj.zone == Zone::Battlefield
                && obj.card.is_creature()
                && (obj.toughness() <= 0
                    || obj.damage >= obj.toughness()
                    || obj.deathtouch_since_sba)
            {
                obj.zone = Zone::Graveyard;
                obj.damage = 0;
                obj.growth = 0;
                obj.tapped = false;
                obj.summoning_sick = false;
            }
        }
        // Deathtouch applies only since the previous SBA check, not forever.
        for obj in &mut self.state.objects {
            obj.deathtouch_since_sba = false;
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
                keywords: o.card.keywords(),
            })
            .collect();
        Observation {
            schema: 3,
            catalogue: crate::CATALOGUE_VERSION,
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
            damage_assigned: if owns_choice {
                s.hits.clone()
            } else {
                Vec::new()
            },
            damage_source: if owns_choice {
                s.damage_choices.last().map(|choice| choice.source.object)
            } else {
                None
            },
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
            + s.hits.len() * std::mem::size_of::<DamageEvent>()
            + s.first_strikers.len() * std::mem::size_of::<CardId>()
            + s.damage_choices.len() * std::mem::size_of::<DamageChoice>()
            + s.damage_choices
                .iter()
                .map(|d| d.blockers.len() * std::mem::size_of::<CardId>())
                .sum::<usize>()
    }
}
