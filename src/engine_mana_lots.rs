//! Cold, owned mana provenance. Ordinary games retain their six-counter fast path.
//! A lot records rules, never a producer pointer: source departure cannot change
//! its restriction, and equivalent production merges instead of growing history.
use super::*;
const CREATURE_KIND: u16 = 1;
use crate::cards::{CreatureType, ManaQuality, ManaRestriction, ManaRetention, ManaSpellBonus};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize)]
pub enum ManaRestrictionView {
    ArtifactSpell,
    ExceptNonArtifactSpells,
    SubtypeSpellOrActivation(&'static str),
    InstantSorcery,
    Unrestricted,
    Unavailable,
    SpellSubtype(&'static str),
    CreatureSpellOrActivation(CreatureType),
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize)]
pub enum ManaSpellBonusView {
    None,
    DragonCreatureHaste,
    CopyRedInstantSorcery,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
pub struct ManaCopySource {
    pub object: ObjectRef,
    pub card: Card,
    pub definition: Card,
    pub controller: Player,
    pub facts: DamageFacts,
    pub keywords: Keywords,
    pub tapped: bool,
}
impl std::hash::Hash for ManaCopySource {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        std::hash::Hash::hash(
            &(
                self.object,
                self.card,
                self.definition,
                self.controller.index(),
            ),
            state,
        );
    }
}
impl ManaCopySource {
    fn capture<W: ability_world::CharacteristicWorld + ?Sized>(world: &W, source: CardId) -> Self {
        let object = world.object(source);
        let mut facts = world.game().damage_facts(source);
        facts.card = object.card;
        facts.controller = object.controller;
        facts.kinds = world.kinds(source);
        facts.colors = world.colors(source);
        facts.granted_subtypes = world.subtypes(source);
        facts.token = world.is_token(source);
        facts.counter = world.counter_amounts(source).has_any();
        facts.plus_one = world.counter_count(source, CounterKind::PlusOne) > 0;
        facts.attacking = world.attacking(source);
        Self {
            object: world.reference(source),
            card: object.card,
            definition: object.card,
            controller: object.controller,
            facts,
            keywords: world.game().keywords(source),
            tapped: object.tapped,
        }
    }
}
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub(super) struct SpendOutcome {
    pub haste: bool,
    #[allow(clippy::box_collection)]
    pub copies: Option<Box<Vec<(ManaCopySource, u64)>>>,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize)]
pub(super) struct Quality {
    pub restriction: ManaRestrictionView,
    pub spell_bonus: ManaSpellBonusView,
    pub retention: ManaRetention,
    pub copy_source: Option<ManaCopySource>,
}
impl Quality {
    const ORDINARY: Self = Self {
        restriction: ManaRestrictionView::Unrestricted,
        spell_bonus: ManaSpellBonusView::None,
        retention: ManaRetention::Step,
        copy_source: None,
    };
    pub(super) fn capture<W: ability_world::CharacteristicWorld + ?Sized>(
        world: &W,
        source: CardId,
        quality: ManaQuality,
    ) -> Self {
        Self {
            restriction: match quality.restriction {
                ManaRestriction::ArtifactSpell => ManaRestrictionView::ArtifactSpell,
                ManaRestriction::ExceptNonArtifactSpells => {
                    ManaRestrictionView::ExceptNonArtifactSpells
                }
                ManaRestriction::SubtypeSpellOrActivation(subtype) => {
                    ManaRestrictionView::SubtypeSpellOrActivation(subtype)
                }
                ManaRestriction::InstantSorcery => ManaRestrictionView::InstantSorcery,
                ManaRestriction::Unrestricted => ManaRestrictionView::Unrestricted,
                ManaRestriction::SpellSubtype(subtype) => {
                    ManaRestrictionView::SpellSubtype(subtype)
                }
                ManaRestriction::ChosenCreatureSpellOrActivation => {
                    match world.entry_value(world.reference(source)) {
                        Some(EntryChoiceValue::CreatureType(kind)) => {
                            ManaRestrictionView::CreatureSpellOrActivation(kind)
                        }
                        _ => ManaRestrictionView::Unavailable,
                    }
                }
            },
            spell_bonus: match quality.spell_bonus {
                ManaSpellBonus::None => ManaSpellBonusView::None,
                ManaSpellBonus::DragonCreatureHaste => ManaSpellBonusView::DragonCreatureHaste,
                ManaSpellBonus::CopyRedInstantSorcery => ManaSpellBonusView::CopyRedInstantSorcery,
            },
            retention: quality.retention,
            copy_source: (quality.spell_bonus == ManaSpellBonus::CopyRedInstantSorcery)
                .then(|| ManaCopySource::capture(world, source)),
        }
    }
    pub(super) fn with_definition(mut self, definition: Card) -> Self {
        if let Some(source) = &mut self.copy_source {
            source.definition = definition;
        }
        self
    }
    fn same_rules(self, other: Self) -> bool {
        let identity = |source: Option<ManaCopySource>| {
            source.map(|s| (s.object, s.card, s.definition, s.controller))
        };
        self.restriction == other.restriction
            && self.spell_bonus == other.spell_bonus
            && self.retention == other.retention
            && identity(self.copy_source) == identity(other.copy_source)
    }
    pub(super) fn eligible(self, purpose: Purpose) -> bool {
        match self.restriction {
            ManaRestrictionView::ArtifactSpell => {
                matches!(purpose, Purpose::Spell(facts) if facts.kinds & 16 != 0)
            }
            ManaRestrictionView::ExceptNonArtifactSpells => {
                !matches!(purpose, Purpose::Spell(facts) if facts.kinds & 16 == 0)
            }
            ManaRestrictionView::SubtypeSpellOrActivation(subtype) => match purpose {
                Purpose::Spell(facts) | Purpose::Activation(facts) => facts.has_subtype(subtype),
                Purpose::Other => false,
            },
            ManaRestrictionView::InstantSorcery => {
                matches!(purpose, Purpose::Spell(facts) if facts.kinds & (4 | 8) != 0)
            }
            ManaRestrictionView::Unrestricted => true,
            ManaRestrictionView::Unavailable => false,
            ManaRestrictionView::SpellSubtype(subtype) => {
                matches!(purpose, Purpose::Spell(facts) if facts.has_subtype(subtype))
            }
            ManaRestrictionView::CreatureSpellOrActivation(kind) => match purpose {
                Purpose::Spell(facts) | Purpose::Activation(facts) => {
                    facts.kinds & CREATURE_KIND != 0 && facts.has_subtype(kind.name())
                }
                Purpose::Other => false,
            },
        }
    }
    pub(super) fn grants_haste(self, purpose: Purpose) -> bool {
        self.spell_bonus == ManaSpellBonusView::DragonCreatureHaste
            && matches!(purpose, Purpose::Spell(facts)
                if facts.kinds & CREATURE_KIND != 0 && facts.has_subtype("Dragon"))
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum Purpose {
    Other,
    Spell(Source),
    Activation(Source),
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub(super) struct Source {
    pub card: Card,
    pub kinds: u16,
    pub subtypes: GrantedSubtypes,
    pub foreign_stash: bool,
    pub spend_any_mana: bool,
}
impl Source {
    pub(super) fn in_world<W: ability_world::CharacteristicWorld + ?Sized>(
        world: &W,
        id: CardId,
    ) -> Self {
        Self {
            card: world.object(id).card,
            kinds: world.kinds(id),
            subtypes: world.subtypes(id),
            foreign_stash: false,
            spend_any_mana: false,
        }
    }
    fn has_subtype(self, subtype: &str) -> bool {
        self.subtypes.has(self.card, subtype)
    }
    fn printed(card: Card) -> Self {
        Self {
            card,
            kinds: card.kind_mask(),
            subtypes: GrantedSubtypes::default(),
            foreign_stash: false,
            spend_any_mana: false,
        }
    }
}
impl From<DamageFacts> for Source {
    fn from(facts: DamageFacts) -> Self {
        Self {
            card: facts.card,
            kinds: facts.kinds,
            subtypes: facts.granted_subtypes,
            foreign_stash: false,
            spend_any_mana: false,
        }
    }
}
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub(super) struct Lot {
    pub quality: Quality,
    pub amounts: [u128; 6],
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct ManaLotView {
    pub player: Player,
    pub lot: usize,
    pub restriction: ManaRestrictionView,
    pub spell_bonus: ManaSpellBonusView,
    pub retention: ManaRetention,
    pub copy_source: Option<ManaCopySource>,
    pub amounts: Mana,
}
/// One scalar allocation domain; large pools never enumerate a Cartesian product.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct ManaLotSpendRange {
    /// Original action whose costs remain uncommitted during this choice.
    pub action: Action,
    pub lot: usize,
    pub color: usize,
    pub minimum: u32,
    pub maximum: u32,
    /// When present, this exact sorted set refines the interval (guided mana
    /// windows can have holes). None means every integer in the interval.
    pub values: Option<Vec<u32>>,
}
impl ManaLotSpendRange {
    pub fn contains(&self, amount: u32) -> bool {
        (self.minimum..=self.maximum).contains(&amount)
            && self
                .values
                .as_ref()
                .is_none_or(|values| values.binary_search(&amount).is_ok())
    }
}
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub(super) struct ManaState {
    pub pools: [SharedVec<Lot>; 2],
    pub prepared: Option<Vec<Vec<Mana>>>,
    pub effect_quality: Option<Quality>,
    pub announcement_quality: Option<(ObjectRef, Quality)>,
    pub trace: Option<SpendRequest>,
    pub replay: Option<Vec<Mana>>,
    pub choice: Option<Box<SpendChoice>>,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) struct SpendRequest {
    actor: Player,
    purpose: Purpose,
    payment: Mana,
    lots: SharedVec<Lot>,
    viable: Option<Vec<Vec<Mana>>>,
    ordinary: [u128; 6],
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) struct SpendChoice {
    request: SpendRequest,
    action: Action,
    pending: Pending,
    allocations: Vec<Mana>,
    remaining: [u128; 6],
    cursor: usize,
}
impl SpendChoice {
    fn bounds(&self, lot: usize, color: usize) -> (u32, u32, Option<Vec<u32>>) {
        if let Some(plans) = &self.request.viable {
            let mut values: Vec<_> = plans.iter().map(|plan| plan[lot][color]).collect();
            values.sort_unstable();
            values.dedup();
            let minimum = *values.first().expect("guided allocation remains feasible");
            let maximum = *values.last().unwrap();
            let holes = u64::from(maximum - minimum) + 1 != values.len() as u64;
            return (minimum, maximum, holes.then_some(values));
        }
        let (minimum, maximum) = allocation_range(
            &self.request.lots,
            self.request.ordinary,
            self.request.purpose,
            self.remaining,
            lot,
            color,
        );
        (
            u32::try_from(minimum).expect("concrete payment"),
            u32::try_from(maximum).expect("concrete payment"),
            None,
        )
    }
    fn range(&mut self) -> Option<ManaLotSpendRange> {
        while self.cursor < self.request.lots.len() * 6 {
            let lot = self.cursor / 6;
            let color = self.cursor % 6;
            let (minimum, maximum, values) = self.bounds(lot, color);
            if minimum != maximum {
                return Some(ManaLotSpendRange {
                    action: self.action.clone(),
                    lot,
                    color,
                    minimum,
                    maximum,
                    values,
                });
            }
            self.allocations[lot][color] = minimum;
            self.remaining[color] -= u128::from(minimum);
            self.cursor += 1;
        }
        None
    }
}

pub(super) fn add_lot(lots: &mut Vec<Lot>, quality: Quality, amounts: [u128; 6]) {
    if quality == Quality::ORDINARY || amounts == [0; 6] {
        return;
    }
    if let Some(lot) = lots.iter_mut().find(|lot| lot.quality.same_rules(quality)) {
        for (held, amount) in lot.amounts.iter_mut().zip(amounts) {
            *held = held
                .checked_add(amount)
                .capacity("mana lot representation exhausted");
        }
    } else {
        lots.push(Lot { quality, amounts });
    }
}
/// Restrictions are separable by color once the purpose is fixed. Each ordinary
/// unit is the residual of the total pool after subtracting all explicit lots.
pub(super) fn available(pool: [u128; 6], lots: &[Lot], purpose: Purpose) -> [u128; 6] {
    let mut result = pool;
    for lot in lots {
        if !lot.quality.eligible(purpose) {
            for (amount, excluded) in result.iter_mut().zip(lot.amounts) {
                *amount -= excluded;
            }
        }
    }
    result
}
/// Remaining feasible interval when the preceding lots are already fixed.
/// `ordinary` excludes every tagged lot, including ineligible lots.
pub(super) fn allocation_range(
    lots: &[Lot],
    ordinary: [u128; 6],
    purpose: Purpose,
    remaining: [u128; 6],
    lot_index: usize,
    color: usize,
) -> (u128, u128) {
    let lot = &lots[lot_index];
    if !lot.quality.eligible(purpose) {
        return (0, 0);
    }
    let future = lots[lot_index + 1..]
        .iter()
        .filter(|lot| lot.quality.eligible(purpose))
        .fold(ordinary[color], |total, lot| total + lot.amounts[color]);
    (
        remaining[color].saturating_sub(future),
        remaining[color].min(lot.amounts[color]),
    )
}
pub(super) fn ordinary(pool: [u128; 6], lots: &[Lot]) -> [u128; 6] {
    let mut result = pool;
    for lot in lots {
        for (amount, tagged) in result.iter_mut().zip(lot.amounts) {
            *amount -= tagged;
        }
    }
    result
}

impl Game {
    pub(super) fn capture_mana_copy_source_departure(&mut self, before: &ObjectView) {
        let Some(state) = &mut self.state.mana_lots else {
            return;
        };
        for lots in &mut state.pools {
            for lot in lots.iter_mut() {
                if let Some(source) = &mut lot.quality.copy_source
                    && source.object == before.reference
                {
                    source.facts = DamageFacts::from_view(before);
                    source.keywords = before.keywords;
                    source.tapped = before.tapped;
                }
            }
        }
    }
    pub(super) fn mana_lots(&self, actor: Player) -> &[Lot] {
        self.state
            .mana_lots
            .as_ref()
            .map_or(&[], |state| &state.pools[actor.index()])
    }
    pub(super) fn available_mana(&self, actor: Player, purpose: Purpose) -> Mana {
        let pool = self.state.players[actor.index()].mana;
        if self.state.mana_lots.is_none() {
            return pool;
        }
        available(pool.map(u128::from), self.mana_lots(actor), purpose)
            .map(|amount| u32::try_from(amount).expect("subset of concrete mana pool"))
    }
    pub(super) fn mana_quality(
        &self,
        production: ManaProduction,
        source: CardId,
    ) -> Option<Quality> {
        match production {
            ManaProduction::Tagged { quality, .. } => {
                Some(Quality::capture(self, source, *quality))
            }
            _ => None,
        }
    }
    // Capture before announcement costs can remove the producer or its choice.
    // The exact source reference identifies this announcement, not a live lookup.
    pub(super) fn capture_announced_mana_quality(&mut self, item: &SpellView) {
        let StackKind::ActivatedAbility { ability } = item.kind else {
            return;
        };
        let quality = item
            .source_card
            .activation(ability)
            .mana_production()
            .and_then(|production| self.mana_quality(production, item.object.card))
            .map(|quality| quality.with_definition(item.source_card));
        if let Some(quality) = quality {
            self.state
                .mana_lots
                .get_or_insert_default()
                .announcement_quality = Some((item.object, quality));
        }
    }
    pub(super) fn add_announced_mana(&mut self, item: &SpellView, mana: Mana) {
        let quality = self.state.mana_lots.as_mut().and_then(|state| {
            state.announcement_quality.take().map(|(source, quality)| {
                assert_eq!(source, item.object);
                quality
            })
        });
        self.add_quality_mana(item.controller, mana, quality);
    }
    pub(super) fn add_quality_mana(&mut self, actor: Player, mana: Mana, quality: Option<Quality>) {
        self.record_cast_mana_production(actor, mana, quality);
        self.add_mana(actor, mana);
        if let Some(quality) = quality
            && quality != Quality::ORDINARY
            && mana != [0; 6]
        {
            let state = self.state.mana_lots.get_or_insert_default();
            add_lot(
                &mut state.pools[actor.index()],
                quality,
                mana.map(u128::from),
            );
        }
    }
    pub(super) fn clear_step_mana(&mut self, keep_combat: bool, keep_turn: bool) {
        for actor in [Player::P0, Player::P1] {
            let mut keep = [0; 6];
            if let Some(state) = self.state.mana_lots.as_mut() {
                state.pools[actor.index()].retain(|lot| {
                    (lot.quality.retention == ManaRetention::Turn && keep_turn)
                        || (lot.quality.retention == ManaRetention::Combat && keep_combat)
                });
                for lot in &state.pools[actor.index()] {
                    for (total, amount) in keep.iter_mut().zip(lot.amounts) {
                        *total += u32::try_from(amount).expect("concrete lot bound");
                    }
                }
            }
            let player = &mut self.state.players[actor.index()];
            player.combat_red_mana = if keep_combat {
                player.combat_red_mana
            } else {
                0
            };
            // Legacy firebending remains implicit until it is migrated to lots.
            keep[3] += player.combat_red_mana;
            player.mana = keep;
        }
        if self.state.mana_lots.as_ref().is_some_and(|state| {
            state.pools.iter().all(|pool| pool.is_empty())
                && state.choice.is_none()
                && state.effect_quality.is_none()
                && state.announcement_quality.is_none()
                && state.replay.is_none()
                && state.trace.is_none()
        }) {
            self.state.mana_lots = None;
        }
    }
}

impl Game {
    pub(super) fn mana_lot_spend_range(&self) -> Option<ManaLotSpendRange> {
        if !matches!(self.state.pending, Pending::ManaLotSpend { .. }) {
            return None;
        }
        let choice = self.state.mana_lots.as_ref()?.choice.as_ref()?;
        let lot = choice.cursor / 6;
        let color = choice.cursor % 6;
        let (minimum, maximum, values) = choice.bounds(lot, color);
        Some(ManaLotSpendRange {
            action: choice.action.clone(),
            lot,
            color,
            minimum,
            maximum,
            values,
        })
    }
    pub(super) fn mana_lot_views(&self) -> Vec<ManaLotView> {
        let Some(state) = &self.state.mana_lots else {
            return Vec::new();
        };
        [Player::P0, Player::P1]
            .into_iter()
            .flat_map(|player| {
                state.pools[player.index()]
                    .iter()
                    .enumerate()
                    .map(move |(index, lot)| ManaLotView {
                        lot: index,
                        player,
                        restriction: lot.quality.restriction,
                        spell_bonus: lot.quality.spell_bonus,
                        retention: lot.quality.retention,
                        copy_source: lot.quality.copy_source,
                        amounts: lot
                            .amounts
                            .map(|n| u32::try_from(n).expect("concrete lot bound")),
                    })
            })
            .collect()
    }
    pub(super) fn defer_mana_allocation(
        &mut self,
        actor: Player,
        action: Action,
        checkpoint: State,
        revision: u64,
    ) {
        let request = self
            .state
            .mana_lots
            .as_mut()
            .and_then(|state| state.trace.take());
        let Some(request) = request else {
            self.compact_mana_lots();
            return;
        };
        let mut choice = SpendChoice {
            allocations: vec![[0; 6]; request.lots.len()],
            remaining: request.payment.map(u128::from),
            cursor: 0,
            action,
            pending: checkpoint.pending.clone(),
            request,
        };
        if choice.range().is_none() {
            self.compact_mana_lots();
            return;
        }
        // Discard the tentative pure-state transaction using apply's original
        // checkpoint. RNG, costs, trigger queues and revisions all roll back.
        self.state = checkpoint;
        self.state.mana_lots.get_or_insert_default().choice = Some(Box::new(choice));
        self.state.pending = Pending::ManaLotSpend { player: actor };
        self.revision = revision.checked_add(1).capacity("revision exhausted");
    }
    pub(super) fn apply_mana_lot_choice(
        &mut self,
        actor: Player,
        action: Action,
    ) -> Result<(), Error> {
        if action == Action::Concede {
            self.state.mana_lots.as_mut().unwrap().choice = None;
            return self.apply_transaction(actor, action);
        }
        if self.semantic_actor() != Some(actor) {
            return Err(Error::WrongActor);
        }
        let Action::ChooseManaLotSpend { amount } = action else {
            return Err(Error::IllegalAction);
        };
        let range = self.mana_lot_spend_range().ok_or(Error::IllegalAction)?;
        if !range.contains(amount) {
            return Err(Error::IllegalAction);
        }
        let mut choice = self
            .state
            .mana_lots
            .as_mut()
            .unwrap()
            .choice
            .take()
            .unwrap();
        choice.allocations[range.lot][range.color] = amount;
        choice.remaining[range.color] -= u128::from(amount);
        choice.cursor += 1;
        if let Some(plans) = &mut choice.request.viable {
            plans.retain(|plan| plan[range.lot][range.color] == amount);
            debug_assert!(!plans.is_empty());
        }
        if choice.range().is_some() {
            self.state.mana_lots.as_mut().unwrap().choice = Some(choice);
            self.revision = self.revision.checked_add(1).capacity("revision exhausted");
            return Ok(());
        }
        self.state.pending = choice.pending;
        self.state.mana_lots.as_mut().unwrap().replay = Some(choice.allocations);
        self.apply_transaction(actor, choice.action)
    }
    pub(super) fn purpose_for_item(&self, item: &SpellView) -> Purpose {
        if item.kind == StackKind::Spell {
            let mut source = Source::printed(item.source_card);
            source.foreign_stash = item.cast.is_some_and(|c| c.stash_origin);
            source.spend_any_mana = item.cast.is_some_and(|c| c.spend_any_mana);
            Purpose::Spell(source)
        } else if matches!(item.kind, StackKind::ActivatedAbility { .. }) {
            let source = if self
                .state
                .objects
                .get(item.object.card as usize)
                .is_some_and(|o| {
                    o.zone != Zone::Outside && self.object_ref(item.object.card) == item.object
                }) {
                Source::in_world(self, item.object.card)
            } else {
                item.source_facts.into()
            };
            Purpose::Activation(source)
        } else {
            // Costs of triggered abilities (including ward) are not activations.
            Purpose::Other
        }
    }
    pub(super) fn spell_mana_purpose_for_card(
        &self,
        actor: Player,
        card: CardId,
        definition: Card,
    ) -> Purpose {
        let mut source = Source::printed(definition);
        source.spend_any_mana = self.exile_cast_modifiers(actor, card).0
            || self.state.announcing.as_ref().is_some_and(|a| {
                a.item.card == card && a.item.cast.is_some_and(|c| c.spend_any_mana)
            });
        source.foreign_stash = self.foreign_stash(actor, card)
            || self.state.announcing.as_ref().is_some_and(|a| {
                a.item.card == card && a.item.cast.is_some_and(|c| c.stash_origin)
            });
        Purpose::Spell(source)
    }
    pub(super) fn spend_mana_for(
        &mut self,
        actor: Player,
        payment: Mana,
        purpose: Purpose,
    ) -> SpendOutcome {
        if payment == [0; 6] {
            return SpendOutcome::default();
        }
        if self.mana_lots(actor).is_empty() {
            self.spend_ordinary_mana(actor, payment);
            return SpendOutcome::default();
        }
        let pool = self.state.players[actor.index()].mana.map(u128::from);
        let lots = self.state.mana_lots.as_ref().unwrap().pools[actor.index()].clone();
        let ordinary = ordinary(pool, &lots);
        let available = available(pool, &lots, purpose);
        assert!(
            payment
                .iter()
                .enumerate()
                .all(|(i, n)| u128::from(*n) <= available[i]),
            "validated mana purpose"
        );
        let replay = self.state.mana_lots.as_mut().unwrap().replay.take();
        let selected = if let Some(selected) = replay {
            selected
        } else {
            let state = self.state.mana_lots.as_mut().unwrap();
            assert!(state.trace.is_none(), "one committed payment per action");
            let viable = state.prepared.take();
            state.trace = Some(SpendRequest {
                viable: viable.clone(),
                actor,
                purpose,
                payment,
                lots: lots.clone(),
                ordinary,
            });
            if let Some(viable) = viable {
                viable
                    .into_iter()
                    .next()
                    .expect("guided allocation is feasible")
            } else {
                let mut remaining = payment.map(u128::from);
                let mut selected = vec![[0; 6]; lots.len()];
                for (i, allocation) in selected.iter_mut().enumerate() {
                    for color in 0..6 {
                        let (minimum, _) =
                            allocation_range(&lots, ordinary, purpose, remaining, i, color);
                        allocation[color] = u32::try_from(minimum).expect("concrete payment bound");
                        remaining[color] -= minimum;
                    }
                }
                selected
            }
        };
        self.record_cast_mana_spent_lots(&lots, &selected);
        let mut outcome = SpendOutcome::default();
        let state = self.state.mana_lots.as_mut().unwrap();
        let mut ordinary_payment = payment;
        for ((lot, original), allocation) in state.pools[actor.index()]
            .iter_mut()
            .zip(&lots)
            .zip(selected)
        {
            if allocation != [0; 6] && original.quality.grants_haste(purpose) {
                outcome.haste = true
            }
            if allocation != [0; 6]
                && original.quality.spell_bonus == ManaSpellBonusView::CopyRedInstantSorcery
                && matches!(purpose, Purpose::Spell(facts) if facts.kinds & (4 | 8) != 0 && facts.card.colors() & 8 != 0)
            {
                let amount = allocation.into_iter().map(u64::from).sum();
                outcome.copies.get_or_insert_with(Default::default).push((
                    original.quality.copy_source.expect("copy mana attribution"),
                    amount,
                ));
            }
            for color in 0..6 {
                lot.amounts[color] -= u128::from(allocation[color]);
                ordinary_payment[color] -= allocation[color];
            }
        }
        state.pools[actor.index()].retain(|lot| lot.amounts != [0; 6]);
        // The legacy retained-red subset is contained in the ordinary residual.
        let player = &mut self.state.players[actor.index()];
        let ordinary_red =
            u32::try_from(ordinary[3]).expect("concrete ordinary bound") - player.combat_red_mana;
        player.combat_red_mana -= ordinary_payment[3].saturating_sub(ordinary_red);
        for (held, paid) in player.mana.iter_mut().zip(payment) {
            *held -= paid
        }
        outcome
    }
}

impl Game {
    /// Concrete color payments still use the established public actions. Their
    /// eligibility depends on what is being paid for, including raw shortcuts.
    pub(super) fn mana_action_payable(&self, action: &Action) -> bool {
        let Some(actor) = self.semantic_actor() else {
            return false;
        };
        let (payment, purpose) = match action {
            Action::Cast { card, payment, .. } | Action::BeginCast { card, payment } => (
                *payment,
                self.spell_mana_purpose_for_card(actor, *card, self.effective_spell_card(*card)),
            ),
            Action::Activate { card, payment, .. }
            | Action::BeginActivate { card, payment, .. }
            | Action::ActivateMana { card, payment, .. }
            | Action::ActivateBorrowedMana { card, payment, .. } => {
                (*payment, Purpose::Activation(Source::in_world(self, *card)))
            }
            Action::PayCosts { payment } => {
                let Some(a) = &self.state.announcing else {
                    return false;
                };
                (*payment, self.purpose_for_item(&a.item))
            }
            Action::PayPlot { payment } | Action::PayResolutionMana { payment } => {
                (*payment, Purpose::Other)
            }
            _ => return true,
        };
        let pool = self.available_mana(actor, purpose);
        payment.iter().zip(pool).all(|(paid, held)| *paid <= held)
    }
}

/// Enumerate actual provenance allocations for the funding search. The common
/// no-lot case emits once without recursion or a heap allocation.
pub(super) fn visit_allocations(
    pool: [u128; 6],
    lots: &[Lot],
    purpose: Purpose,
    payment: [u64; 6],
    mut emit: impl FnMut(Vec<[u64; 6]>),
) {
    if lots.is_empty() {
        emit(Vec::new());
        return;
    }
    if available(pool, lots, purpose)
        .iter()
        .zip(payment)
        .any(|(held, paid)| *held < u128::from(paid))
    {
        return;
    }
    fn walk(
        lots: &[Lot],
        ordinary: [u128; 6],
        purpose: Purpose,
        remaining: &mut [u128; 6],
        selected: &mut Vec<[u64; 6]>,
        cursor: usize,
        emit: &mut impl FnMut(Vec<[u64; 6]>),
    ) {
        if cursor == lots.len() * 6 {
            emit(selected.clone());
            return;
        }
        let lot = cursor / 6;
        let color = cursor % 6;
        let (minimum, maximum) = allocation_range(lots, ordinary, purpose, *remaining, lot, color);
        for amount in minimum..=maximum {
            selected[lot][color] = u64::try_from(amount).expect("funding payment bound");
            remaining[color] -= amount;
            walk(
                lots,
                ordinary,
                purpose,
                remaining,
                selected,
                cursor + 1,
                emit,
            );
            remaining[color] += amount;
        }
    }
    walk(
        lots,
        ordinary(pool, lots),
        purpose,
        &mut payment.map(u128::from),
        &mut vec![[0; 6]; lots.len()],
        0,
        &mut emit,
    );
}

impl Game {
    pub(super) fn funding_purpose(&self, goal: &FundingGoal) -> Purpose {
        if let Some((source, ability, _, definition, slot)) = &goal.source {
            if let Some(item) = self
                .state
                .announcing
                .as_ref()
                .map(|a| &a.item)
                .into_iter()
                .chain(
                    self.state
                        .mana_announcement_stack
                        .iter()
                        .flat_map(|frames| frames.iter().rev())
                        .map(|frame| &frame.announcement.item),
                )
                .find(|item| {
                    item.object == *source
                        && item.controller == goal.actor
                        && item.kind == StackKind::ActivatedAbility { ability: *ability }
                        && item.source_card == *definition
                        && item.borrowed_slot == *slot
                })
            {
                return self.purpose_for_item(item);
            }
            return if self.object_ref(source.card) == *source
                && self.state.objects[source.card as usize].zone
                    == definition.activation(*ability).source_zone()
            {
                Purpose::Activation(Source::in_world(self, source.card))
            } else {
                Purpose::Other
            };
        }
        self.state
            .announcing
            .as_ref()
            .filter(|a| a.item.controller == goal.actor)
            .map_or(Purpose::Other, |a| self.purpose_for_item(&a.item))
    }
}

impl Game {
    fn compact_mana_lots(&mut self) {
        if self.state.mana_lots.as_ref().is_some_and(|state| {
            state.pools.iter().all(|pool| pool.is_empty())
                && state.trace.is_none()
                && state.prepared.is_none()
                && state.replay.is_none()
                && state.choice.is_none()
                && state.effect_quality.is_none()
                && state.announcement_quality.is_none()
        }) {
            self.state.mana_lots = None
        }
    }
    pub(super) fn mana_lot_bytes(&self) -> usize {
        fn plans_bytes(plans: &Option<Vec<Vec<Mana>>>) -> usize {
            plans.as_ref().map_or(0, |plans| {
                plans.len() * std::mem::size_of::<Vec<Mana>>()
                    + plans
                        .iter()
                        .map(|plan| plan.len() * std::mem::size_of::<Mana>())
                        .sum::<usize>()
            })
        }
        let Some(state) = &self.state.mana_lots else {
            return 0;
        };
        std::mem::size_of::<ManaState>()
            + 2 * std::mem::size_of::<Vec<Lot>>()
            + state
                .pools
                .iter()
                .map(|pool| pool.len() * std::mem::size_of::<Lot>())
                .sum::<usize>()
            + plans_bytes(&state.prepared)
            + state
                .replay
                .as_ref()
                .map_or(0, |plan| plan.len() * std::mem::size_of::<Mana>())
            + state.choice.as_ref().map_or(0, |choice| {
                std::mem::size_of::<SpendChoice>()
                    + choice.allocations.len() * std::mem::size_of::<Mana>()
                    + choice.request.lots.len() * std::mem::size_of::<Lot>()
                    + plans_bytes(&choice.request.viable)
                    + match &choice.action {
                        Action::Activate {
                            counter_payment, ..
                        }
                        | Action::ActivateMana {
                            counter_payment, ..
                        }
                        | Action::ActivateBorrowedMana {
                            counter_payment, ..
                        }
                        | Action::AnnounceBorrowedMana {
                            counter_payment, ..
                        }
                        | Action::AnnounceBorrowedAbility {
                            counter_payment, ..
                        }
                        | Action::BeginActivate {
                            counter_payment, ..
                        }
                        | Action::AnnounceActivate {
                            counter_payment, ..
                        }
                        | Action::AnnounceMana {
                            counter_payment, ..
                        } => counter_payment.owned_bytes(),
                        _ => 0,
                    }
                    + choice.pending.owned_bytes()
            })
    }
}
