//! Cold, owned mana provenance. Ordinary games retain their six-counter fast path.
//! A lot records rules, never a producer pointer: source departure cannot change
//! its restriction, and equivalent production merges instead of growing history.
use super::*;
const CREATURE_KIND: u16 = 1;
use crate::cards::{CreatureType, ManaQuality, ManaRestriction, ManaRetention, ManaSpellBonus};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize)]
pub enum ManaRestrictionView {
    Unrestricted,
    SpellSubtype(&'static str),
    CreatureSpellOrActivation(CreatureType),
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize)]
pub enum ManaSpellBonusView {
    None,
    DragonCreatureHaste,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize)]
pub(super) struct Quality {
    pub restriction: ManaRestrictionView,
    pub spell_bonus: ManaSpellBonusView,
    pub retention: ManaRetention,
}
impl Quality {
    const ORDINARY: Self = Self {
        restriction: ManaRestrictionView::Unrestricted,
        spell_bonus: ManaSpellBonusView::None,
        retention: ManaRetention::Step,
    };
    pub(super) fn capture<W: ability_world::CharacteristicWorld + ?Sized>(
        world: &W,
        source: CardId,
        quality: ManaQuality,
    ) -> Self {
        Self {
            restriction: match quality.restriction {
                ManaRestriction::Unrestricted => ManaRestrictionView::Unrestricted,
                ManaRestriction::SpellSubtype(subtype) => {
                    ManaRestrictionView::SpellSubtype(subtype)
                }
                ManaRestriction::ChosenCreatureSpellOrActivation => {
                    let Some(EntryChoiceValue::CreatureType(kind)) =
                        world.game().entry_value(world.reference(source))
                    else {
                        panic!("chosen mana source must have made its entry choice")
                    };
                    ManaRestrictionView::CreatureSpellOrActivation(kind)
                }
            },
            spell_bonus: match quality.spell_bonus {
                ManaSpellBonus::None => ManaSpellBonusView::None,
                ManaSpellBonus::DragonCreatureHaste => ManaSpellBonusView::DragonCreatureHaste,
            },
            retention: quality.retention,
        }
    }
    pub(super) fn eligible(self, purpose: Purpose) -> bool {
        match self.restriction {
            ManaRestrictionView::Unrestricted => true,
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
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) struct Source {
    pub card: Card,
    pub kinds: u16,
    pub subtypes: GrantedSubtypes,
}
impl Source {
    fn has_subtype(self, subtype: &str) -> bool {
        self.subtypes.has(self.card, subtype)
    }
    fn printed(card: Card) -> Self {
        Self {
            card,
            kinds: card.kind_mask(),
            subtypes: GrantedSubtypes::default(),
        }
    }
}
impl From<DamageFacts> for Source {
    fn from(facts: DamageFacts) -> Self {
        Self {
            card: facts.card,
            kinds: facts.kinds,
            subtypes: facts.granted_subtypes,
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
    pub restriction: ManaRestrictionView,
    pub spell_bonus: ManaSpellBonusView,
    pub retention: ManaRetention,
    pub amounts: Mana,
}
/// One scalar allocation domain; large pools never enumerate a Cartesian product.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct ManaLotSpendRange {
    pub lot: usize,
    pub color: usize,
    pub minimum: u32,
    pub maximum: u32,
}
impl ManaLotSpendRange {
    pub fn contains(&self, amount: u32) -> bool {
        (self.minimum..=self.maximum).contains(&amount)
    }
}
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub(super) struct ManaState {
    pub pools: [Vec<Lot>; 2],
    pub trace: Option<SpendRequest>,
    pub replay: Option<Vec<Mana>>,
    pub choice: Option<Box<SpendChoice>>,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) struct SpendRequest {
    actor: Player,
    purpose: Purpose,
    payment: Mana,
    lots: Vec<Lot>,
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
    fn range(&mut self) -> Option<ManaLotSpendRange> {
        while self.cursor < self.request.lots.len() * 6 {
            let lot = self.cursor / 6;
            let color = self.cursor % 6;
            let (minimum, maximum) = allocation_range(
                &self.request.lots,
                self.request.ordinary,
                self.request.purpose,
                self.remaining,
                lot,
                color,
            );
            let minimum = u32::try_from(minimum).expect("payment is concrete");
            let maximum = u32::try_from(maximum).expect("payment is concrete");
            if minimum != maximum {
                return Some(ManaLotSpendRange {
                    lot,
                    color,
                    minimum,
                    maximum,
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
    if let Some(lot) = lots.iter_mut().find(|lot| lot.quality == quality) {
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
            ManaProduction::Tagged { quality, .. } => Some(Quality::capture(self, source, quality)),
            _ => None,
        }
    }
    pub(super) fn add_quality_mana(&mut self, actor: Player, mana: Mana, quality: Option<Quality>) {
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
            state.pools.iter().all(Vec::is_empty)
                && state.choice.is_none()
                && state.replay.is_none()
                && state.trace.is_none()
        }) {
            self.state.mana_lots = None;
        }
    }
}

impl Game {
    pub(super) fn mana_lot_spend_range(&self) -> Option<ManaLotSpendRange> {
        let choice = self.state.mana_lots.as_ref()?.choice.as_ref()?;
        let lot = choice.cursor / 6;
        let color = choice.cursor % 6;
        let (minimum, maximum) = allocation_range(
            &choice.request.lots,
            choice.request.ordinary,
            choice.request.purpose,
            choice.remaining,
            lot,
            color,
        );
        Some(ManaLotSpendRange {
            lot,
            color,
            minimum: u32::try_from(minimum).expect("concrete payment"),
            maximum: u32::try_from(maximum).expect("concrete payment"),
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
                    .map(move |lot| ManaLotView {
                        player,
                        restriction: lot.quality.restriction,
                        spell_bonus: lot.quality.spell_bonus,
                        retention: lot.quality.retention,
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
        let Some(request) = request else { return };
        let mut choice = SpendChoice {
            allocations: vec![[0; 6]; request.lots.len()],
            remaining: request.payment.map(u128::from),
            cursor: 0,
            action,
            pending: checkpoint.pending.clone(),
            request,
        };
        if choice.range().is_none() {
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
        if self.actor() != Some(actor) {
            return Err(Error::WrongActor);
        }
        if action == Action::Concede {
            return self.apply_transaction(actor, action);
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
            self.spell_mana_purpose(item.source_card)
        } else {
            let facts = if self
                .state
                .objects
                .get(item.object.card as usize)
                .is_some_and(|o| {
                    o.zone != Zone::Outside && self.object_ref(item.object.card) == item.object
                }) {
                self.damage_facts(item.object.card)
            } else {
                item.source_facts
            };
            Purpose::Activation(facts.into())
        }
    }
    pub(super) fn spell_mana_purpose(&self, definition: Card) -> Purpose {
        Purpose::Spell(Source::printed(definition))
    }
    pub(super) fn spend_mana_for(
        &mut self,
        actor: Player,
        payment: Mana,
        purpose: Purpose,
    ) -> bool {
        if payment == [0; 6] {
            return false;
        }
        if self.mana_lots(actor).is_empty() {
            self.spend_ordinary_mana(actor, payment);
            return false;
        }
        let pool = self.state.players[actor.index()].mana.map(u128::from);
        let lots = self.mana_lots(actor).to_vec();
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
            state.trace = Some(SpendRequest {
                actor,
                purpose,
                payment,
                lots: lots.clone(),
                ordinary,
            });
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
        };
        let mut grants_haste = false;
        let state = self.state.mana_lots.as_mut().unwrap();
        let mut ordinary_payment = payment;
        for ((lot, original), allocation) in state.pools[actor.index()]
            .iter_mut()
            .zip(&lots)
            .zip(selected)
        {
            if allocation != [0; 6] && original.quality.grants_haste(purpose) {
                grants_haste = true
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
        grants_haste
    }
}

impl Game {
    /// Concrete color payments still use the established public actions. Their
    /// eligibility depends on what is being paid for, including raw shortcuts.
    pub(super) fn mana_action_payable(&self, action: &Action) -> bool {
        let Some(actor) = self.actor() else {
            return false;
        };
        let (payment, purpose) = match action {
            Action::Cast { card, payment, .. } | Action::BeginCast { card, payment } => (
                *payment,
                self.spell_mana_purpose(self.effective_spell_card(*card)),
            ),
            Action::Activate { card, payment, .. }
            | Action::BeginActivate { card, payment, .. }
            | Action::ActivateMana { card, payment, .. } => (
                *payment,
                Purpose::Activation(self.damage_facts(*card).into()),
            ),
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
        if let Some((source, _, _, _)) = &goal.source {
            return Purpose::Activation(self.damage_facts(source.card).into());
        }
        self.state
            .announcing
            .as_ref()
            .filter(|a| a.item.controller == goal.actor)
            .map_or(Purpose::Other, |a| self.purpose_for_item(&a.item))
    }
}

#[cfg(test)]
mod tests {
    use super::super::ability_timeline_tests::{cast, checked, settle, setup};
    use super::super::tests::spawn;
    use super::*;

    fn named(g: &mut Game, p: Player, name: &str, zone: Zone) -> CardId {
        spawn(g, p, Card::from_name(name).unwrap(), zone)
    }
    fn activate(g: &mut Game, card: CardId, ability: u16, production: Mana) {
        checked(
            g,
            Action::ActivateMana {
                card,
                ability,
                production,
                payment: [0; 6],
                counter_payment: Counters::default(),
            },
        );
    }
    fn reject(g: &mut Game, action: Action) {
        let before = g.snapshot();
        let revision = g.revision();
        assert_eq!(
            g.apply(g.actor().unwrap(), action),
            Err(Error::IllegalAction)
        );
        assert_eq!(before, g.snapshot());
        assert_eq!(revision, g.revision());
    }
    #[test]
    fn tagged_mana_orb_paid_cast_chooses_real_haste_and_owned_transaction() {
        assert_eq!(std::mem::size_of::<Object>(), 144);
        for p in [Player::P0, Player::P1] {
            let mut g = setup(p);
            g.state.players[p.index()].mana = [0, 0, 0, 7, 0, 0];
            let orb = named(&mut g, p, "Carnelian Orb of Dragonkind", Zone::Battlefield);
            activate(&mut g, orb, 0, [0, 0, 0, 1, 0, 0]);
            let dragon = named(&mut g, p, "Shivan Dragon", Zone::Hand);
            checked(
                &mut g,
                Action::Cast {
                    card: dragon,
                    target: None,
                    payment: [0, 0, 0, 6, 0, 0],
                },
            );
            assert_eq!(g.object_view(dragon).zone, Zone::Hand);
            assert_eq!(g.state.players[p.index()].mana[3], 8);
            let range = g.mana_lot_spend_range().unwrap();
            assert_eq!((range.minimum, range.maximum), (0, 1));
            assert!(g.observe(p).mana_lot_spend.is_some());
            assert!(g.observe(p.other()).mana_lot_spend.is_none());
            assert!(!g.observe(p.other()).objects.iter().any(|o| o.id == dragon));
            reject(&mut g, Action::ChooseManaLotSpend { amount: 2 });
            let choice = g.snapshot();
            checked(&mut g, Action::ChooseManaLotSpend { amount: 0 });
            settle(&mut g);
            assert!(!g.keywords(dragon).contains(Keywords::HASTE));
            g.restore(&choice).unwrap();
            checked(&mut g, Action::ChooseManaLotSpend { amount: 1 });
            settle(&mut g);
            assert!(g.keywords(dragon).contains(Keywords::HASTE));
            assert_eq!(g.state.players[p.index()].mana[3], 2);
        }
    }
    #[test]
    fn tagged_mana_giada_restricts_raw_announced_and_funding_spells() {
        for p in [Player::P0, Player::P1] {
            let mut g = setup(p);
            g.state.players[p.index()].mana = [0, 0, 0, 0, 0, 1];
            let giada = named(&mut g, p, "Giada, Font of Hope", Zone::Battlefield);
            activate(&mut g, giada, 0, [1, 0, 0, 0, 0, 0]);
            let chaplain = named(&mut g, p, "Daybreak Chaplain", Zone::Hand);
            assert!(
                !g.legal_actions()
                    .contains(&Action::AnnounceCast { card: chaplain })
            );
            reject(
                &mut g,
                Action::Cast {
                    card: chaplain,
                    target: None,
                    payment: [1, 0, 0, 0, 0, 1],
                },
            );
            assert_eq!(g.available_mana(p, Purpose::Other), [0, 0, 0, 0, 0, 1]);
            let angel = named(&mut g, p, "Serra Angel", Zone::Hand);
            g.add_mana(p, [1, 0, 0, 0, 0, 2]);
            cast(&mut g, angel, None, &[]);
            settle(&mut g);
            assert_eq!(g.object_view(angel).zone, Zone::Battlefield);
            assert_eq!(g.counter_amounts(angel).get(CounterKind::PlusOne), 1);
        }
    }
    #[test]
    fn tagged_mana_courtyard_captures_type_after_producer_departure() {
        for p in [Player::P0, Player::P1] {
            let mut g = setup(p);
            g.state.players[p.index()].mana = [0, 0, 0, 0, 0, 0];
            let courtyard = named(&mut g, p, "Secluded Courtyard", Zone::Hand);
            checked(&mut g, Action::PlayLand { card: courtyard });
            let angel = CreatureType::from_name("Angel").unwrap();
            checked(
                &mut g,
                Action::ChooseEntryValue {
                    value: EntryChoiceValue::CreatureType(angel),
                },
            );
            activate(&mut g, courtyard, 1, [1, 0, 0, 0, 0, 0]);
            g.sacrifice_batch(&[(p, courtyard)]);
            assert_eq!(g.object_view(courtyard).zone, Zone::Graveyard);
            assert_eq!(
                g.mana_lots(p)[0].quality.restriction,
                ManaRestrictionView::CreatureSpellOrActivation(angel)
            );
            let serra = Card::from_name("Serra Angel").unwrap();
            assert_eq!(
                g.available_mana(p, g.spell_mana_purpose(serra)),
                [1, 0, 0, 0, 0, 0]
            );
            assert_eq!(g.available_mana(p, Purpose::Other), [0; 6]);
            let creature = Source::printed(serra);
            assert_eq!(
                g.available_mana(p, Purpose::Activation(creature)),
                [1, 0, 0, 0, 0, 0]
            );
            g.add_mana(p, [1, 0, 0, 0, 0, 3]);
            let spell = named(&mut g, p, "Serra Angel", Zone::Hand);
            cast(&mut g, spell, None, &[]);
            settle(&mut g);
            assert_eq!(g.object_view(spell).zone, Zone::Battlefield);
        }
    }
    #[test]
    fn tagged_mana_giada_simultaneous_entries_count_pre_event_angels_and_double() {
        for p in [Player::P0, Player::P1] {
            let mut g = setup(p);
            named(&mut g, p, "Giada, Font of Hope", Zone::Battlefield);
            named(&mut g, p, "Serra Angel", Zone::Battlefield);
            named(&mut g, p, "Doubling Season", Zone::Battlefield);
            let a = named(&mut g, p, "Inspiring Overseer", Zone::Graveyard);
            let b = named(&mut g, p, "Serra Angel", Zone::Graveyard);
            let moves = [a, b]
                .into_iter()
                .map(|id| g.entry_move(id, Zone::Battlefield, false))
                .collect();
            g.stage_entries(moves, None);
            g.advance_entries();
            while matches!(g.state.pending, Pending::Replacement { .. }) {
                let action = g
                    .legal_actions()
                    .into_iter()
                    .find(|a| *a != Action::Concede)
                    .unwrap();
                checked(&mut g, action);
            }
            assert_eq!(g.counter_amounts(a).get(CounterKind::PlusOne), 4);
            assert_eq!(g.counter_amounts(b).get(CounterKind::PlusOne), 4);
        }
    }
    #[test]
    fn tagged_mana_ventmaw_attack_retains_both_colors_until_actual_turn_end() {
        for p in [Player::P0, Player::P1] {
            let mut g = setup(p);
            g.state.players[p.index()].mana = [0; 6];
            let ventmaw = named(&mut g, p, "Savage Ventmaw", Zone::Battlefield);
            g.state.step = Step::Attackers;
            g.state.pending = Pending::Attackers;
            checked(&mut g, Action::Attack { card: ventmaw });
            checked(&mut g, Action::FinishAttackers);
            settle(&mut g);
            assert_eq!(g.state.players[p.index()].mana, [0, 0, 0, 3, 3, 0]);
            g.clear_step_mana(false, true);
            assert_eq!(g.state.players[p.index()].mana, [0, 0, 0, 3, 3, 0]);
            g.clear_step_mana(false, false);
            assert_eq!(g.state.players[p.index()].mana, [0; 6]);
            assert!(g.state.mana_lots.is_none());
        }
    }
}
