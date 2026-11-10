//! Exact reachability for the admitted pure mana sources; no game-state clones,
//! heuristics that discard legal plans, or mutation during action generation.
use super::*;
use crate::cards::{ReplacementFilter, StaticAbilityFragment};
use ability_world::CharacteristicWorld;
use std::cell::RefCell;
use std::collections::HashSet;
use std::rc::Rc;
#[doc(hidden)]
#[path = "engine_cast_cost_planning.rs"]
pub mod cast_cost_planning;
use cast_cost_planning::CostContext;
#[doc(hidden)]
#[path = "engine_funding_entries.rs"]
pub mod funding_entries;
use funding_entries::{EntryProjection, PaymentContinuation};
#[doc(hidden)]
#[path = "engine_funding_entry_devour.rs"]
mod entry_devour_projection;
#[doc(hidden)]
#[path = "engine_funding_entry_exiles.rs"]
pub mod funding_entry_exiles;
#[doc(hidden)]
#[path = "engine_funding_linked_copies.rs"]
pub mod funding_linked_copies;

#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
#[derive(Clone, Debug)]
pub(super) struct FundingGoal {
    pub actor: Player,
    pub required: Mana,
    pub generic: u32,
    pub reserve: Option<(ObjectRef, AdditionalCost)>,
    pub source: Option<(ObjectRef, u16, Counters, Card, Option<u64>)>,
}
#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
impl FundingGoal {
    /// CR601.2g opens a window for the locked total, not the printed cost.
    /// Paying {0} still needs acknowledgment, but requires no mana.
    fn requires_mana(&self) -> bool {
        self.generic != 0 || self.required.iter().any(|&amount| amount != 0)
    }

    fn reserves_source_tap(&self, object: ObjectRef) -> bool {
        self.source
            .as_ref()
            .is_some_and(|(source, ability, _, definition, _)| {
                *source == object && definition.activation(*ability).cost.tap
            })
    }
}
#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
#[derive(Clone)]
struct Resource {
    id: CardId,
    attacking: bool,
    object: Object,
}

#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
fn activation_counter_placement_payable<W: CharacteristicWorld + ?Sized>(
    world: &W,
    goal: &FundingGoal,
) -> bool {
    goal.source
        .as_ref()
        .is_none_or(|(source, index, _, definition, _)| {
            !activations::mana_loyalty_placement_prohibited(
                world,
                *source,
                definition.activation(*index),
            ) && !activations::printed_counter_placement_prohibited(
                world,
                *source,
                *definition,
                *index,
            )
        })
}
#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
#[derive(Clone)]
pub(super) struct FundingState<'g> {
    game: &'g Game,
    actor: Player,
    life_gained: [bool; 2],
    life_lost: [bool; 2],
    enduring_story: [bool; 2],
    exhaust_activated: [bool; 2],
    permanent_left_battlefield: [bool; 2],
    void_active_this_turn: bool,
    active: Player,
    mana: [u128; 6],
    lots: Vec<mana_lots::Lot>,
    // Strict spell-only mana cannot pay any intermediate activated ability.
    // This fixed query bound includes all frozen ancestor payments and limits
    // only equivalent search-key amounts, never owned pools or allocations.
    spell_lot_demand: Option<u128>,
    purpose: mana_lots::Purpose,
    payment_source: Option<ObjectRef>,
    exhaust_source_reuse_authorized: bool,
    source_use_independent: bool,
    required_face_up: Option<ObjectRef>,
    pub(super) contributors: Option<Box<crew::ContributorSelection>>,
    // Sparse last-known facts for exact enclosing sources that leave a speculative world.
    payment_departures: Vec<(ObjectRef, mana_lots::Source, i128)>,
    watched_sources: Vec<ObjectRef>,
    life: i128,
    other_life: i128,
    life_replacement_possible: bool,
    resources: Vec<Resource>,
    named_counters: live_counters::Rows,
    copy_recipes: copies::Rows,
    counter_placement_history: counter_placement_history::Rows,
    convoke: bool,
    waterbend: u16,
    power_sensitive: bool,
    creatures_died: u64,
    controlled_creatures_died: [u64; 2],
    graveyard_left: [bool; 2],
    sacrifice_group: Vec<ObjectRef>,
    cost_group: Option<(AdditionalCost, Vec<ObjectRef>)>,
    cast_costs: Option<Box<CostContext>>,
    entry_projection: Option<Box<EntryProjection>>,
    continuation: Option<Box<PaymentContinuation<'g>>>,
    /// One boolean query owns this cache; action enumeration worlds never do.
    continuation_memo: Option<Rc<ContinuationMemo>>,
    /// A projected completion removes only its resolved child from CR605.3c's
    /// active-instance chain; the real game's immutable active frames stay intact.
    active_mana_keys: Option<Vec<mana_announcements::ManaActivationKey>>,
    borrowed_uses: Vec<(ObjectRef, u64)>,
    self_grant_uses: Vec<(ObjectRef, u64)>,
    graveyard_borrowing: graveyard_borrowing::Rows,
    borrowed_slots: borrowed_activations::SlotRows,
}
#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
impl ability_world::CharacteristicWorld for FundingState<'_> {
    fn void_active_this_turn(&self) -> bool {
        self.void_active_this_turn
    }
    fn departed_source_power(&self, source: ObjectRef) -> Option<i128> {
        self.payment_departures
            .iter()
            .find(|(r, _, _)| *r == source)
            .map(|(_, _, p)| *p)
            .or_else(|| self.game.contributor_source_facts(source).map(|f| f.power))
    }
    fn departed_source_subtypes(&self, source: ObjectRef) -> Option<(Card, GrantedSubtypes)> {
        self.payment_departures
            .iter()
            .find(|(object, _, _)| *object == source)
            .map(|(_, facts, _)| (facts.card, facts.subtypes))
            .or_else(|| {
                self.game
                    .contributor_source_facts(source)
                    .map(|facts| (facts.card, facts.granted_subtypes))
            })
    }
    fn projected_self_grant_used(&self, source: ObjectRef, timestamp: u64) -> bool {
        self.self_grant_uses.contains(&(source, timestamp))
    }
    fn borrowed_slots_state(&self) -> &borrowed_activations::SlotRows {
        &self.borrowed_slots
    }
    fn borrowed_slots(&self) -> &[borrowed_activations::SlotRow] {
        self.borrowed_slots
            .as_ref()
            .map_or(&[], |rows| rows.as_slice())
    }
    fn graveyard_borrowing_state(&self) -> &graveyard_borrowing::Rows {
        &self.graveyard_borrowing
    }
    fn graveyard_borrowing(&self) -> &[graveyard_borrowing::Row] {
        self.graveyard_borrowing
            .as_ref()
            .map_or(&[], |rows| rows.as_slice())
    }
    fn exhaust_activated_this_turn(&self, player: Player) -> bool {
        self.exhaust_activated[player.index()]
    }
    fn has_enduring_story(&self, player: Player) -> bool {
        self.enduring_story[player.index()]
    }
    fn ability_origin(&self, id: CardId) -> Option<u64> {
        if self.object(id).zone != Zone::Battlefield {
            return None;
        }
        let object = self.reference(id);
        self.entry_projection
            .as_ref()
            .and_then(|p| p.copy_origins.iter().find(|(source, _)| *source == object))
            .map_or_else(
                || self.game.copy_ability_origin(object),
                |(_, origin)| *origin,
            )
    }
    fn provided_origin(&self, object: ObjectRef) -> Option<u64> {
        self.entry_projection
            .as_ref()
            .and_then(|p| {
                p.provided_origins
                    .iter()
                    .find(|(source, _)| *source == object)
            })
            .map_or_else(
                || self.game.copied_grant_origin(object),
                |(_, origin)| *origin,
            )
    }
    fn face_down(&self, id: CardId) -> bool {
        self.projected_face_down_physical(self.reference(id))
            .is_some()
    }
    fn temporary_copy_rows_state(&self) -> &temporary_copy::Rows {
        self.entry_projection
            .as_ref()
            .map_or(&self.game.state.temporary_copies, |p| &p.temporary_copies)
    }
    fn temporary_copy_view(&self, object: ObjectRef) -> Option<TemporaryCopyView> {
        let mut view = self
            .temporary_copy_rows_state()
            .as_ref()?
            .iter()
            .find(|row| row.object == object)?
            .view();
        view.layers.retain(|layer| {
            layer.deadline.source().is_none_or(|source| {
                self.object(source.card).zone == Zone::Battlefield
                    && self.reference(source.card) == source
            })
        });
        view.active_origin = view.layers.last()?.origin;
        Some(view)
    }
    fn stored_recipe(&self, id: CardId) -> Option<CopyRecipe> {
        let reference = self.reference(id);
        let rows = self.copy_recipes.as_ref()?;
        let index = rows.binary_search_by_key(&id, |row| row.object.card).ok()?;
        (rows[index].object == reference).then_some(rows[index].recipe)
    }
    fn static_sources(&self) -> &[CardId] {
        self.entry_projection
            .as_ref()
            .map_or(&self.game.state.static_sources, |p| &p.static_sources)
    }
    fn trigger_multiplier_sources(&self) -> &[CardId] {
        self.entry_projection.as_ref().map_or_else(
            || self.game.trigger_multiplier_sources(),
            |projection| projection.static_sources.as_slice(),
        )
    }
    fn attachment_sources(&self) -> &[CardId] {
        self.entry_projection
            .as_ref()
            .map_or(&self.game.state.attachment_sources, |p| {
                &p.attachment_sources
            })
    }
    fn roles(&self) -> Option<&roles::RoleState> {
        self.entry_projection
            .as_ref()
            .map(|p| &p.roles)
            .or(self.game.state.roles.as_deref())
    }
    fn entry_values(&self) -> &[entry_choices::ChoiceRow] {
        self.entry_projection.as_ref().map_or_else(
            || {
                self.game
                    .state
                    .entry_values
                    .as_deref()
                    .map_or(&[] as &[entry_choices::ChoiceRow], |rows| rows.as_slice())
            },
            |p| p.entry_values.as_slice(),
        )
    }
    fn controls(&self) -> Option<&continuous_control::ControlState> {
        self.entry_projection
            .as_ref()
            .map(|p| &p.controls)
            .or(self.game.state.controls.as_deref())
    }
    fn creatures_died(&self) -> u64 {
        self.creatures_died
    }
    fn card_left_graveyard(&self, player: Player) -> bool {
        self.graveyard_left[player.index()]
    }
    fn borrowed_activation_used(&self, source: ObjectRef, slot: u64) -> bool {
        self.borrowed_uses.contains(&(source, slot))
    }
    fn entered_this_turn(&self, source: ObjectRef) -> bool {
        self.game.entered_this_turn(source)
            || self
                .entry_projection
                .as_ref()
                .is_some_and(|p| p.entered.contains(&source) || p.order.contains(&source))
    }
    fn battle_protector(&self, object: ObjectRef) -> Option<Player> {
        self.entry_projection
            .as_ref()
            .and_then(|p| {
                p.battle_protectors
                    .iter()
                    .find(|(reference, _)| *reference == object)
                    .map(|(_, player)| *player)
            })
            .or_else(|| self.game.battle_protector(object))
    }
    fn game(&self) -> &Game {
        self.game
    }
    fn object(&self, id: CardId) -> &Object {
        self.resources
            .iter()
            .find(|resource| resource.id == id)
            .map_or(&self.game.state.objects[id as usize], |resource| {
                &resource.object
            })
    }
    fn plus_one_placed_this_turn(&self, object: ObjectRef, player: Player) -> bool {
        counter_placement_history::plus_one_placed(&self.counter_placement_history, object, player)
    }
    fn named_counter_rows(&self) -> &[live_counters::NamedCounterRow] {
        self.named_counters
            .as_ref()
            .map_or(&[], |rows| rows.as_slice())
    }
    fn counter_amounts(&self, id: CardId) -> CounterAmounts<'_> {
        live_counters::amounts(
            &self.named_counters,
            self.reference(id),
            self.object(id).counters,
        )
    }
    fn life(&self, player: Player) -> i128 {
        if player == self.actor {
            self.life
        } else {
            self.other_life
        }
    }
    fn gained_life(&self, player: Player) -> bool {
        self.life_gained[player.index()]
    }
    fn lost_life(&self, player: Player) -> bool {
        self.life_lost[player.index()]
    }
    fn attacking(&self, id: CardId) -> bool {
        self.resources
            .iter()
            .find(|resource| resource.id == id)
            .map_or_else(
                || self.game.state.attackers.iter().any(|row| row.card == id),
                |resource| resource.attacking && resource.object.zone == Zone::Battlefield,
            )
    }
}

/// Exact permanent/card selections for a projected mana activation cost.
#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
#[derive(Clone)]
struct ExternalManaCost {
    selection: additional_costs::ManaCostObjects,
    kind: EffectChoiceKind,
}

#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
#[derive(Clone)]
struct FundingMove {
    resource: usize,
    ability: Option<u16>,
    grantor: Option<ObjectRef>,
    borrowed: Option<BorrowedDescriptor>,
    self_grant: Option<(Card, u64)>,
    payment: [u64; 6],
    counters: Counters,
    production: Mana,
    quality: Option<mana_lots::Quality>,
    allocation: Vec<[u64; 6]>,
    external: Option<ExternalManaCost>,
    /// A previously announced activation keeps its immutable ability definition.
    frozen: Option<Box<(Card, Option<u64>)>>,
    output: Option<Box<mana_output::ManaOutput>>,
}
#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
#[derive(Hash, PartialEq, Eq)]
struct ResourceKey {
    definition: usize,
    token: bool,
    card: bool,
    kinds: u16,
    colors: u8,
    protected: bool,
    reserved: bool,
    zone: u8,
    tapped: bool,
    sick: bool,
    entered: bool,
    counters: InitialCounters,
    used: u64,
    loyalty_activated_this_turn: bool,
    dealt_damage_this_incarnation: bool,
    dealt_combat_damage_this_incarnation: bool,
    gained: Keywords,
    owned: bool,
    controlled: bool,
    attacking: bool,
    identity: Option<(CardId, u64)>,
    attached: Option<(CardId, u64)>,
}
#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
#[derive(Hash, PartialEq, Eq)]
struct EntryFundingKey {
    copy_origins: Vec<(ObjectRef, Option<u64>)>,
    manifested: Vec<(ObjectRef, Card)>,
    order: Vec<ObjectRef>,
    entered: Vec<ObjectRef>,
    lands_entered: [u64; 2],
    battle_protectors: Vec<(ObjectRef, usize)>,
    values: Vec<(ObjectRef, EntryChoiceValue)>,
    riot_haste: Vec<(ObjectRef, u64)>,
    consumed: Vec<ObjectRef>,
}
#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
#[derive(Hash, PartialEq, Eq)]
struct FundingKey {
    mana_window_open: bool,
    life_lost: [bool; 2],
    enduring_story: [bool; 2],
    exhaust_activated: [bool; 2],
    permanent_left_battlefield: [bool; 2],
    void_active_this_turn: bool,
    payment_departures: Vec<(ObjectRef, mana_lots::Source, i128)>,
    source_linked_effects: Vec<SourceLinkedEffectView>,
    resolved_ability_modifications: Vec<ResolvedAbilityModificationView>,
    entries: Option<EntryFundingKey>,
    histories: Option<(u64, [bool; 2], i128, [bool; 2])>,
    controlled_creature_deaths: [u64; 2],
    borrowed_uses: Vec<(ObjectRef, u64)>,
    self_grant_uses: Vec<(ObjectRef, u64)>,
    graveyard_borrowing: Vec<graveyard_borrowing::Row>,
    borrowed_slots: Vec<borrowed_activations::SlotRow>,
    basic_land_types: Vec<subtypes::BasicLandTypesEffectView>,
    mana: [u128; 6],
    lots: Vec<mana_lots::Lot>,
    life: i128,
    payment_libraries: Option<[Vec<ObjectRef>; 2]>,
    // Equivalent sources are interchangeable for feasibility. Protect the
    // enclosing ability's source separately, even if its printed card matches.
    resources: Vec<ResourceKey>,
    named_counters: live_counters::Rows,
    counter_placement_history: counter_placement_history::Rows,
    keyword_counter_timestamps: Vec<(ObjectRef, CounterKind, u64)>,
}

#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
#[derive(Default)]
struct ContinuationMemo {
    visited: RefCell<HashSet<(usize, FundingKey)>>,
    exact: HashSet<CardId>,
    contributors: bool,
}

#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
fn payable<N: Copy + Into<u128>>(pool: [N; 6], required: Mana, generic: u32) -> bool {
    let pool = pool.map(Into::into);
    if (0..6).any(|i| pool[i] < u128::from(required[i])) {
        return false;
    }
    let mut left = u128::from(generic);
    for i in 0..6 {
        let spare = pool[i] - u128::from(required[i]);
        if spare >= left {
            return true;
        }
        left -= spare;
    }
    left == 0
}
// Funding is counterfactual. Its pools can exceed the concrete mana storage
// bound without overflowing an observation or excluding a mathematically legal plan.
#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
fn funding_payments(pool: [u128; 6], colored: Mana, generic: u32, mut emit: impl FnMut([u64; 6])) {
    visit_funding_payments(pool, colored, generic, |payment| {
        emit(payment);
        false
    });
}

/// Enumerate in the same order as `funding_payments`, stopping when accepted.
#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
fn visit_funding_payments(
    pool: [u128; 6],
    colored: Mana,
    generic: u32,
    mut emit: impl FnMut([u64; 6]) -> bool,
) -> bool {
    if (0..6).any(|i| pool[i] < u128::from(colored[i])) {
        return false;
    }
    fn visit(
        i: usize,
        left: u32,
        pool: [u128; 6],
        mut paid: [u64; 6],
        emit: &mut impl FnMut([u64; 6]) -> bool,
    ) -> bool {
        if i == 5 {
            if pool[i] >= u128::from(left) {
                paid[i] += u64::from(left);
                return emit(paid);
            }
            return false;
        }
        for n in 0..=pool[i].min(u128::from(left)) as u32 {
            paid[i] += u64::from(n);
            if visit(i + 1, left - n, pool, paid, emit) {
                return true;
            }
            paid[i] -= u64::from(n);
        }
        false
    }
    visit(
        0,
        generic,
        std::array::from_fn(|i| pool[i] - u128::from(colored[i])),
        colored.map(u64::from),
        &mut emit,
    )
}
/// Exact equivalents of one mana ability, with duplicate physical allocations removed.
#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
fn funding_cost_payments(
    pool: [u128; 6],
    cost: ActivationCost,
    discounts: power_up::ActivationDiscounts,
    increase: u32,
    reductions: Option<&power_up::ActivationCostReductions>,
    payment_cost: impl Fn(ManaCost) -> ManaCost,
    mut emit: impl FnMut([u64; 6], Option<ManaCost>),
) {
    let mut seen = std::collections::HashSet::new();
    for base in cost.mana_options() {
        for &reduction in power_up::ActivationCostReductions::options(reductions) {
            for equivalent in
                power_up::reduced_totals(base, reduction, discounts, u64::from(increase))
                    .into_iter()
                    .flatten()
            {
                let adjusted = payment_cost(equivalent);
                if !payable(pool, adjusted.colored, adjusted.generic) {
                    // A deferred activation locks its symbols, but spending
                    // permissions are evaluated again in the world that pays.
                    emit([0; 6], Some(equivalent));
                    continue;
                }
                funding_payments(pool, adjusted.colored, adjusted.generic, |payment| {
                    if (cost.alternatives.is_empty() && reductions.is_none())
                        || seen.insert(payment)
                    {
                        emit(payment, None);
                    }
                });
            }
        }
    }
}
#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
fn funding_allocations(
    pool: [u128; 6],
    lots: &[mana_lots::Lot],
    purpose: mana_lots::Purpose,
    payment: [u64; 6],
    deferred: Option<ManaCost>,
    mut emit: impl FnMut(Vec<[u64; 6]>),
) {
    if deferred.is_some() {
        emit(Vec::new());
    } else {
        mana_lots::visit_allocations(pool, lots, purpose, payment, emit);
    }
}
#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
fn concrete_payment(payment: [u64; 6]) -> Mana {
    payment.map(|n| u32::try_from(n).capacity("mana payment exhausted"))
}
#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
fn reserve_obligations<W: CharacteristicWorld + ?Sized>(
    world: &W,
    goal: &FundingGoal,
    object: Option<&Object>,
    keywords: impl FnOnce(CardId) -> Keywords,
) -> bool {
    goal.reserve.is_none_or(|(reference, spec)| {
        if spec.kind == EffectChoiceKind::ChooseControlledOrWarpedCreature {
            return object.is_some()
                && additional_costs::chosen_cost_creature_matches(world, goal.actor, reference);
        }
        object.is_some_and(|o| {
            if spec.kind == EffectChoiceKind::ExileGrantingAttachment {
                return o.generation == reference.generation && o.zone == Zone::Battlefield;
            }
            o.generation == reference.generation
                && (!spec.kind.is_attached_sacrifice()
                    || goal
                        .source
                        .as_ref()
                        .is_some_and(|(source, _, _, _, _)| o.attached_to == Some(*source)))
                && Game::additional_object_matches(
                    spec,
                    goal.actor,
                    o,
                    world.kinds(reference.card),
                    world.mana_value(reference.card),
                    keywords(reference.card),
                    world.subtypes(reference.card),
                    world.name(reference.card),
                    || world.legendary(reference.card),
                )
        })
    })
}
/// A source-exile cost must be legal in the final payment world. Do not reject
/// the initial state: paying mana may first remove its prohibition provider.
#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
fn source_exile_payable<W: ability_world::CharacteristicWorld + ?Sized>(
    world: &W,
    goal: &FundingGoal,
) -> bool {
    goal.source
        .as_ref()
        .is_none_or(|(source, index, _, definition, _)| {
            !definition.activation(*index).cost.exile_self
                || !world.graveyard_exile_prohibited(source.card)
        })
}
#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
#[allow(
    clippy::too_many_arguments,
    reason = "Actual and projected payment worlds supply independent readiness facts"
)]
fn source_obligations(
    goal: &FundingGoal,
    object: Option<&Object>,
    amounts: CounterAmounts<'_>,
    kinds: u16,
    keywords: impl FnOnce() -> Keywords,
    life: i128,
    exhaust_reuse_authorized: bool,
    independent_use: bool,
) -> bool {
    let Some((source, index, counters, definition, slot)) = goal.source.as_ref() else {
        return true;
    };
    let mut readiness = *definition.activation(*index);
    if exhaust_reuse_authorized {
        readiness.exhaust = false;
    }
    if independent_use {
        readiness.exhaust = false;
        readiness.once_per_turn = false;
    }
    if definition.activation(*index).exhaust && definition.grants_activation(*index) {
        readiness.exhaust = false;
        readiness.once_per_turn = false;
    }
    let ability = &readiness;
    if ability.cost.life != 0 && i128::from(ability.cost.life) > life {
        return false;
    }
    let current = object.filter(|object| {
        object.zone == ability.source_zone() && object.generation == source.generation
    });
    let requires_source = ability.exerts()
        || ability.cost.tap
        || ability.cost.sacrifice
        || ability.cost.discard
        || ability.cost.exile_self
        || ability.cost.remove.is_some()
        || ability.source_counter_cost().is_some()
        || ability.cost.loyalty.is_some();
    if let Some(object) = current {
        let mut ready_object = object.clone();
        if slot.is_some() {
            ready_object.used_abilities = 0;
        }
        Game::object_activation_available_with_kinds(
            &ready_object,
            amounts,
            kinds,
            keywords,
            life,
            ability,
            if slot.is_some() { 0 } else { *index as usize },
            goal.actor,
            true,
        ) && amounts.contains(counters)
            && amounts
                .get(CounterKind::Loyalty)
                .saturating_sub(counters.get(CounterKind::Loyalty))
                >= ability
                    .cost
                    .loyalty
                    .filter(|change| *change < 0)
                    .map_or(0, |change| u64::from(change.unsigned_abs()))
    } else {
        // The activated ability already exists independently of its source.
        // Only unpaid costs that use that incarnation require it to remain.
        !requires_source
    }
}

/// Upper bound on total mana gained by all remaining uses of one pure mana
/// ability. Converters may improve colors but cannot create spendable quantity.
/// Overlapping resource/life/counter costs across abilities are deliberately
/// counted independently: this can only overestimate what a legal path gains.
#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
fn activation_gain_upper<'a>(
    ability: &ActivatedAbility,
    production: ManaProduction,
    counters: impl Into<CounterAmounts<'a>>,
    life: i128,
    discount: u32,
) -> u128 {
    let counters = counters.into();
    let mut produced = 0;
    production.visit(|mana| {
        produced = produced.max(mana.into_iter().map(u128::from).sum::<u128>());
        false
    });
    let cost = ability.cost;
    let paid = cost
        .mana_options()
        .map(|option| {
            u128::from(option.generic.saturating_sub(discount))
                + option.colored.into_iter().map(u128::from).sum::<u128>()
        })
        .min()
        .unwrap(); // mana_options always includes the primary cost.
    let gain = produced.saturating_sub(paid);
    if gain == 0 {
        return 0;
    }
    // A true repeatable positive engine keeps an unbounded upper bound. Exact
    // reachability still determines whether its seed cost/colors are attainable.
    let mut uses = if cost.tap || ability.once_per_turn || ability.exhaust {
        1
    } else {
        u128::MAX
    };
    if cost.life > 0 {
        uses = uses.min(life.max(0) as u128 / u128::from(cost.life));
    }
    if let Some((kind, amount)) = cost.remove
        && amount > 0
    {
        uses = uses.min(counters.available(kind) / u128::from(amount));
    }
    uses.saturating_mul(gain)
}

#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
impl<'g> FundingState<'g> {
    // Paying costs can remove a negative modifier or change a scaling count.
    // Current power is not an upper bound on later production. Real moves still
    // evaluate exact borrowed-world power; only pruning uses this loose ceiling.
    fn potential_static_tap_mana(&self) -> impl Iterator<Item = ManaProduction> + '_ {
        self.static_sources()
            .iter()
            .filter_map(|&id| {
                let object = ability_world::CharacteristicWorld::object(self, id);
                (object.zone == Zone::Battlefield).then_some(object.card)
            })
            .flat_map(|card| card.static_ability_groups())
            .flat_map(|group| group.fragments)
            .filter_map(|fragment| match *fragment {
                crate::cards::StaticAbilityFragment::LandMana { production, .. }
                | crate::cards::StaticAbilityFragment::ControlledPermanentTapMana {
                    production,
                    ..
                }
                | crate::cards::StaticAbilityFragment::ControlledSubtypeTapMana {
                    production,
                    ..
                }
                | crate::cards::StaticAbilityFragment::OtherControlledPermanentTapMana(
                    production,
                ) => Some(production),
                _ => None,
            })
    }
    fn production_upper(&self, production: ManaProduction) -> ManaProduction {
        match production.base() {
            ManaProduction::TapSourcePowerChoice { mask } => ManaProduction::Choice {
                mask,
                amount: u32::MAX,
            },
            ManaProduction::SourcePowerSplit { first, second } => {
                let mut mana = [0; 6];
                mana[first.index()] = u32::MAX;
                mana[second.index()] = u32::MAX;
                ManaProduction::Fixed(mana)
            }
            ManaProduction::Split(split) => {
                let mut mana = [0; 6];
                let upper = split.total.min(u64::from(u32::MAX)) as u32;
                mana[split.first.index()] = upper;
                mana[split.second.index()] = upper;
                ManaProduction::Fixed(mana)
            }
            ManaProduction::DistinctCreaturePowers => ManaProduction::any_color(u32::MAX),
            ManaProduction::ChoiceStat { colors, .. } => ManaProduction::Choice {
                mask: colors,
                amount: u32::MAX,
            },
            ManaProduction::AnyColorChangingSource(amount) => ManaProduction::any_color(amount),
            ManaProduction::SourcePower { color } => {
                let mut mana = [0; 6];
                mana[usize::from(color)] = u32::MAX;
                ManaProduction::Fixed(mana)
            }
            // Each type can appear on every represented object after costs.
            // This is only a pruning bound; exact moves still count the final world.
            ManaProduction::CountSubtype { color, .. } => {
                let mut mana = [0; 6];
                mana[usize::from(color)] =
                    u32::try_from(self.game.state.objects.len()).unwrap_or(u32::MAX);
                ManaProduction::Fixed(mana)
            }
            ManaProduction::MonocoloredPermanentColors
            | ManaProduction::ColorsAmongControlledPermanents
            | ManaProduction::ChosenColor => ManaProduction::Fixed([1, 1, 1, 1, 1, 0]),
            value => value,
        }
    }

    fn set_purpose(&mut self, purpose: mana_lots::Purpose) {
        self.purpose = purpose;
        let needs_convoke_resources = matches!(purpose, mana_lots::Purpose::Spell(spell) if convoke::potential(self, spell.card));
        self.convoke |= needs_convoke_resources;
        let needs_improvise_resources = matches!(purpose, mana_lots::Purpose::Spell(spell) if improvise::potential(self, spell.card));
        let needs_delve_resources =
            matches!(purpose, mana_lots::Purpose::Spell(_)) && delve::potential(self);
        if self.waterbend == 0
            && !needs_convoke_resources
            && !needs_improvise_resources
            && !needs_delve_resources
        {
            return;
        }
        for (id, _) in self.game.state.objects.iter().enumerate() {
            let id = id as CardId;
            if self.resources.iter().any(|r| r.id == id) {
                continue;
            }
            let object = ability_world::CharacteristicWorld::object(self, id);
            if object.zone == Zone::Battlefield
                || needs_delve_resources && object.zone == Zone::Graveyard
            {
                self.resources.push(Resource {
                    id,
                    object: object.clone(),
                    attacking: self.game.state.attackers.iter().any(|a| a.card == id),
                });
            }
        }
    }
    fn improvise_available(&self) -> bool {
        matches!(self.purpose, mana_lots::Purpose::Spell(spell) if improvise::available(self, self.actor, spell.card))
    }
    fn convoke_available(&self) -> bool {
        match self.purpose {
            mana_lots::Purpose::Spell(spell) => convoke::available(self, self.actor, spell.card),
            _ => self.convoke,
        }
    }
    fn has_contributions(&self) -> bool {
        self.convoke_available()
            || self.improvise_available()
            || self.delve_available()
            || self.waterbend != 0
    }
    pub(super) fn delve_available(&self) -> bool {
        matches!(self.purpose, mana_lots::Purpose::Spell(spell) if delve::available(self, self.actor, spell.card))
    }
    fn from_game(game: &'g Game, goal: &FundingGoal) -> Self {
        Self::from_game_with_convoke(game, goal, false)
    }
    fn active_melds(&self) -> impl Iterator<Item = &meld::MeldedPermanent> {
        self.roles()
            .into_iter()
            .flat_map(|roles| &roles.melded_permanents)
            .filter(move |row| {
                self.object(row.object.card).zone == Zone::Battlefield
                    && self.reference(row.object.card) == row.object
            })
    }
    fn projected_face_down_physical(&self, object: ObjectRef) -> Option<Card> {
        self.entry_projection.as_ref().map_or_else(
            || self.game.face_down_physical(object),
            |projection| {
                projection
                    .manifested
                    .iter()
                    .find(|(reference, _)| *reference == object)
                    .map(|(_, card)| *card)
            },
        )
    }
    fn project_copy_departure(&mut self, before: ObjectRef, after: ObjectRef) {
        let index = self.copy_recipes.as_ref().and_then(|rows| {
            rows.binary_search_by_key(&before.card, |row| row.object.card)
                .ok()
                .filter(|&index| rows[index].object == before)
        });
        let recipe = self
            .temporary_copy_rows_state()
            .as_ref()
            .and_then(|rows| rows.iter().find(|row| row.object == before))
            .map_or_else(
                || index.map(|index| self.copy_recipes.as_ref().unwrap()[index].recipe),
                |row| row.original_recipe,
            );
        if let Some(index) = index {
            self.copy_recipes.as_mut().unwrap().remove(index);
        }
        if let Some(recipe) = recipe {
            let rows = self.copy_recipes.get_or_insert_default();
            let index = rows.partition_point(|row| row.object.card < after.card);
            rows.insert(
                index,
                copies::CopyRow {
                    object: after,
                    recipe,
                },
            );
        }
        if self
            .copy_recipes
            .as_ref()
            .is_some_and(|rows| rows.is_empty())
        {
            self.copy_recipes = None;
        }
        if let Some(p) = self.entry_projection.as_mut() {
            p.copy_origins.retain(|(object, _)| *object != before);
            p.provided_origins.retain(|(object, _)| *object != before);
            if let Some(rows) = p.temporary_copies.as_mut() {
                rows.retain(|row| row.object != before);
            }
            if p.temporary_copies
                .as_ref()
                .is_some_and(|rows| rows.is_empty())
            {
                p.temporary_copies = None;
            }
        }
    }
    fn from_game_with_convoke(game: &'g Game, goal: &FundingGoal, convoke: bool) -> Self {
        let waterbend = goal
            .source
            .as_ref()
            .map_or(0, |(_, index, _, definition, _)| {
                definition.activation(*index).waterbend_cost()
            });
        // A prerequisite need not itself produce mana (e.g. a changeling for
        // Mox Jasper). Keep only relevant owned witnesses, including removable
        // mana sources, so virtual sacrifices invalidate later guarded moves.
        let retain_permanent_counter_bearers =
            goal.source.as_ref().is_some_and(|(_, index, _, card, _)| {
                card.activation(*index).cost.additional.is_some_and(|cost| {
                    matches!(cost.kind, EffectChoiceKind::RemovePermanentCounters { .. })
                })
            });
        let mut subtypes = Vec::new();
        let mut graveyard_mana_cost = false;
        let mut sacrifice_mana_cost = false;
        let mut tap_mana_cost = false;
        let mut power_sensitive = false;
        let mut color_count_sensitive = false;
        // Loyalty thresholds must retain all battlefield objects: a projected
        // payment can change control or type before a later guarded mana action.
        let mut loyalty_threshold_sensitive =
            goal.source
                .as_ref()
                .is_some_and(|(_, index, _, definition, _)| {
                    matches!(
                        definition.activation(*index).guard,
                        ActivationGuard::ControlledPlaneswalkerSubtypeLoyaltyAtLeast { .. }
                    )
                });
        for (id, object) in game.state.objects.iter().enumerate() {
            if (object.zone == Zone::Battlefield && object.controller == goal.actor)
                || (matches!(object.zone, Zone::Hand | Zone::Graveyard)
                    && object.owner == goal.actor)
            {
                let inherited = borrowed_activations::abilities(game, id as CardId);
                for ability in object
                    .card
                    .activations()
                    .iter()
                    .chain(inherited.into_iter().map(|a| a.rules()))
                    .chain(
                        game.state
                            .self_granted_abilities
                            .as_deref()
                            .into_iter()
                            .flatten()
                            .filter(|row| row.recipient == game.object_ref(id as CardId))
                            .filter_map(|row| {
                                row.kind
                                    .activation()
                                    .map(|ability| row.granting_card.activation(ability))
                            }),
                    )
                {
                    if let Some(production) = ability.mana_production() {
                        color_count_sensitive |=
                            matches!(
                                production.base(),
                                ManaProduction::ColorsAmongControlledPermanents
                                    | ManaProduction::MonocoloredPermanentColors
                                    | ManaProduction::ChoiceStat { .. }
                                    | ManaProduction::AnyColorChangingSource(_)
                            ) || matches!(ability.guard, ActivationGuard::ControlColorsAtLeast(_));
                        loyalty_threshold_sensitive |= matches!(
                            ability.guard,
                            ActivationGuard::ControlledPlaneswalkerSubtypeLoyaltyAtLeast { .. }
                        );
                        power_sensitive |= matches!(
                            production.base(),
                            ManaProduction::SourcePower { .. }
                                | ManaProduction::SourcePowerSplit { .. }
                                | ManaProduction::TapSourcePowerChoice { .. }
                                | ManaProduction::DistinctCreaturePowers
                        );
                        sacrifice_mana_cost |= ability.cost.additional.is_some_and(|cost| {
                            cost.kind.payment_kind() == EffectChoiceKind::Sacrifice
                        });
                        tap_mana_cost |= ability.cost.additional.is_some_and(|cost| {
                            matches!(cost.kind.payment_kind(), EffectChoiceKind::TapPermanents(_))
                        });
                        graveyard_mana_cost |= ability
                            .cost
                            .additional
                            .is_some_and(|cost| cost.kind == EffectChoiceKind::ExileGraveyard);
                        if let ManaProduction::CountSubtype { subtype, .. } = production.base()
                            && !subtypes.contains(&subtype)
                        {
                            subtypes.push(subtype);
                        }
                        for subtype in ability.guard.subtypes() {
                            if !subtypes.contains(subtype) {
                                subtypes.push(*subtype);
                            }
                        }
                    }
                }
            }
        }
        let life_replacement_possible = game.state.objects.iter().any(|object| {
            object.card.static_ability_groups().iter().any(|group| {
                group.fragments.iter().any(|fragment| {
                    matches!(
                        fragment,
                        StaticAbilityFragment::ReplaceLifePaymentWithLibraryExile
                    )
                })
            })
        });
        // An ordinary permanent can gain a mana action from another permanent.
        // Retain controlled recipients even if their current type or ability
        // layer would hide the grant. The projected world tests it at use time.
        let has_controlled_tap_mana_grant = game.state.static_sources.iter().any(|&id| {
            let provider = &game.state.objects[id as usize];
            provider.zone == Zone::Battlefield
                && provider.controller == goal.actor
                && provider.card.static_ability_groups().iter().any(|group| {
                    group.fragments.iter().any(|fragment| {
                        matches!(
                            fragment,
                            StaticAbilityFragment::ControlledPermanentTapMana { .. }
                                | StaticAbilityFragment::ControlledSubtypeTapMana { .. }
                                | StaticAbilityFragment::OtherControlledPermanentTapMana(_)
                        )
                    })
                })
        });
        let shared_linked_activations = borrowed_activations::has_shared_links(game);
        let resources = game
            .state
            .objects
            .iter()
            .enumerate()
            .filter(|(id, object)| {
                (matches!(object.zone, Zone::Hand | Zone::Graveyard) && object.owner == goal.actor
                    && object.card.activations().iter().any(|a| a.source_zone() == object.zone && a.mana_production().is_some()))
                    || (life_replacement_possible && object.zone == Zone::Library && object.is_card())
                    || shared_linked_activations && object.zone == Zone::Exile
                        && game.state.borrowed_links.as_ref().unwrap().iter().any(|link| {
                            link.shared && link.exiled.card == *id as CardId
                        })
                    || (graveyard_mana_cost && object.zone == Zone::Graveyard && object.owner == goal.actor && object.is_card())
                    || game.plot_payment().is_some_and(|request| request.object.card == *id as CardId)
                    || (object.zone == Zone::Command && object.card.is_emblem()) || goal.reserve
                    .is_some_and(|(reference, _)| reference.card == *id as CardId)
                    || goal
                        .source.as_ref()
                        .is_some_and(|(source, _, _, _, _)| source.card == *id as CardId)
                    || object.zone == Zone::Battlefield
                        && game.state.linked_choices.as_ref().is_some_and(|data|
                            data.null_cast_links.iter().any(|link| link.source.card == *id as CardId))
                    || object.zone == Zone::Battlefield
                        && (shared_linked_activations || retain_permanent_counter_bearers || color_count_sensitive || life_replacement_possible || loyalty_threshold_sensitive || sacrifice_mana_cost || tap_mana_cost || game.sacrifice_prohibition_source(*id as CardId) || zone_replacements::card_provides_prevention(object.card) || {
                                #[cfg(feature = "test-support")] { zone_replacements::TEST_PREVENT_PROVIDER.with(|p| p.get() == Some(game.object_ref(*id as CardId))) }
                                #[cfg(not(feature = "test-support"))] { false }
                            } || game.state.controls.is_some() || object.card.has_cast_origin_abilities() || object.card.borrows_activations() || object.card.has_activation_mana_flexibility()
                            || object.card.is_attachment_source()
                            || characteristic_layers::has_equipment_form(game, *id as CardId)
                            || game.state.attachment_sources.iter().any(|&provider| game.state.objects[provider as usize].attached_to == Some(game.object_ref(*id as CardId)))
                            || !object.card.keyword_grants().is_empty()
                            || !object.card.counter_keyword_grants().is_empty()
                            || object.card.static_ability_groups().iter().any(|group| {
                                group.fragments.iter().any(|fragment| matches!(
                                    fragment,
                                    StaticAbilityFragment::ControlledOtherTappedLegendaryCreatureKeywords { .. }
                                        | StaticAbilityFragment::ControlledActivationReduction { .. }
                                        | StaticAbilityFragment::ControlledCreatureKindBonus { .. }
                                        | StaticAbilityFragment::ControlledBasicLandManaMultiplier(_)
                                        | StaticAbilityFragment::ControlledArtifactActivation(_)
                                        | StaticAbilityFragment::GrantRiotToOtherControlledSubtype(_)
                                ))
                            })
                            || object.card.static_ability_groups().iter().any(|group| group.fragments.iter().any(|fragment| matches!(fragment, crate::cards::StaticAbilityFragment::NamedActivationGenericIncrease(_) | crate::cards::StaticAbilityFragment::ActivatedPermanentKindsProhibited(_) | crate::cards::StaticAbilityFragment::AttachedActivationsProhibited | crate::cards::StaticAbilityFragment::ActivateControlledCreaturesAsHaste | crate::cards::StaticAbilityFragment::ReuseExhaustBeforeFirstActivationOwnTurn | crate::cards::StaticAbilityFragment::OtherControlledExhaustGenericReduction(_) | crate::cards::StaticAbilityFragment::ArtifactTokenActivationReduction(_) | crate::cards::StaticAbilityFragment::ControlledLandTapManaBonus(_) | crate::cards::StaticAbilityFragment::ControlledLandColorlessTapManaBonus | crate::cards::StaticAbilityFragment::BasicLandTapManaBonus(_) | crate::cards::StaticAbilityFragment::AttachmentManaBonus(_) | crate::cards::StaticAbilityFragment::ControlledPermanentKeywords { .. } | crate::cards::StaticAbilityFragment::OwnTurnSourceAndKindCreatureWard { .. } | crate::cards::StaticAbilityFragment::ControlledEquippedCreatureKeywords(_) | crate::cards::StaticAbilityFragment::ControlledTappedKeywords { .. } | crate::cards::StaticAbilityFragment::ControlledCreatureScaling { .. } | crate::cards::StaticAbilityFragment::AttachedAllCreatureTypes | crate::cards::StaticAbilityFragment::OpponentPlayerAttackerKeywords(..) | crate::cards::StaticAbilityFragment::ControlledSubtypeKeywords { .. } | crate::cards::StaticAbilityFragment::ControlledUntappedSubtypeKeywords { .. } | crate::cards::StaticAbilityFragment::LandMana { .. } | crate::cards::StaticAbilityFragment::OtherControlledPermanentTapMana(_) | crate::cards::StaticAbilityFragment::ControlledPermanentTapMana { .. } | crate::cards::StaticAbilityFragment::ControlledSubtypeTapMana { .. } | crate::cards::StaticAbilityFragment::ControlledArtifactFood(_) | crate::cards::StaticAbilityFragment::ChosenCreatureType | crate::cards::StaticAbilityFragment::ControlledCreatureSubtype { .. } | crate::cards::StaticAbilityFragment::OtherControlledEntryCountersByManaValue { .. } | crate::cards::StaticAbilityFragment::ControlledCreatureEntryCountersForLandsEntered | crate::cards::StaticAbilityFragment::SourceOffTurnCreature | crate::cards::StaticAbilityFragment::CreatureGroupBonus { .. } | crate::cards::StaticAbilityFragment::CreatureGroupScaling { .. } | crate::cards::StaticAbilityFragment::ControlledLegendaryCreatureBonus(_) | crate::cards::StaticAbilityFragment::ChosenBasicLandType | crate::cards::StaticAbilityFragment::ChosenModifier(..) | crate::cards::StaticAbilityFragment::DeathBaronLord | crate::cards::StaticAbilityFragment::SourceCounterSubtype { .. } | crate::cards::StaticAbilityFragment::SourceCounterCreature { .. } | crate::cards::StaticAbilityFragment::SourceCelebrationCreatureForm { .. } | crate::cards::StaticAbilityFragment::SourceGraveyardAllTypes { .. } | crate::cards::StaticAbilityFragment::SourceConditionalKeywords { .. } | crate::cards::StaticAbilityFragment::SourceCounterKeywords { .. } | crate::cards::StaticAbilityFragment::SourcePlusOnePlacedKeywords(..) | crate::cards::StaticAbilityFragment::PlusOnePlacedKeywords { .. } | crate::cards::StaticAbilityFragment::CounterKeywordOwnTurn(..) | crate::cards::StaticAbilityFragment::CounterKeywordOther(..))))
                            || object.controller == goal.actor
                                && subtypes
                                    .iter()
                                    .any(|subtype| game.has_subtype(*id as CardId, subtype))
                            || (object.controller == goal.actor
                                && (has_controlled_tap_mana_grant
                                    || (convoke
                                    && game.object_kinds(*id as CardId, object) & 1 != 0)
                                    || game.state.self_granted_abilities.as_deref().into_iter().flatten().any(|row|
                                        row.recipient == game.object_ref(*id as CardId) && row.kind.activation().is_some_and(|ability| row.granting_card.activation(ability).mana_production().is_some()))
                                    || object.card.mana_color().is_some()
                                    || game.object_kinds(*id as CardId, object) & 2 != 0
                                    || object
                                        .card
                                        .activations()
                                        .iter()
                                        .any(|ability| ability.mana_production().is_some()))))
            })
            .map(|(id, object)| Resource {
                id: id as CardId,
                attacking: game.state.attackers.iter().any(|a| a.card == id as CardId),
                object: object.clone(),
            })
            .collect();
        let mut result = Self {
            enduring_story: game.state.players.each_ref().map(|p| p.enduring_story),
            exhaust_activated: game
                .state
                .players
                .each_ref()
                .map(|p| p.turn_facts.exhaust_activated_this_turn),
            permanent_left_battlefield: game
                .state
                .players
                .each_ref()
                .map(|p| p.turn_facts.permanent_left_battlefield),
            void_active_this_turn: game.state.void_active_this_turn,
            exhaust_source_reuse_authorized: game.exhaust_reuse_authorized_for_goal(goal),
            source_use_independent: game.self_granted_goal(goal),
            copy_recipes: game.state.copy_recipes.clone(),
            cast_costs: None,
            entry_projection: None,
            continuation: None,
            continuation_memo: None,
            active_mana_keys: None,
            cost_group: game
                .state
                .announcing
                .as_ref()
                .filter(|a| !a.additional_paid)
                .and_then(|a| {
                    let spec = a.additional_cost()?;
                    if !matches!(
                        spec.kind.payment_kind(),
                        EffectChoiceKind::TapPermanents(_)
                            | EffectChoiceKind::SacrificeRange { .. }
                            | EffectChoiceKind::SacrificePermanents(_)
                            | EffectChoiceKind::ExileGraveyardRange { .. }
                            | EffectChoiceKind::ExileGraveyardCards(_)
                    ) {
                        return None;
                    }
                    Some((spec, a.additional_group.as_ref()?.to_vec()))
                }),
            sacrifice_group: game
                .state
                .announcing
                .as_ref()
                .filter(|a| {
                    !a.additional_paid
                        && a.additional_cost().is_some_and(|s| {
                            matches!(s.kind, EffectChoiceKind::SacrificeDifferentNames(_))
                        })
                })
                .and_then(|a| a.additional_group.as_ref())
                .map_or_else(Vec::new, |g| g.to_vec()),
            graveyard_borrowing: game.state.graveyard_borrowing.clone(),
            borrowed_slots: game.state.borrowed_slots.clone(),
            self_grant_uses: Vec::new(),
            borrowed_uses: game
                .state
                .borrowed_uses
                .as_ref()
                .map_or_else(Vec::new, |rows| {
                    rows.iter()
                        .filter(|row| row.exhaust || row.turn == game.state.turn)
                        .map(|row| (row.source, row.slot))
                        .collect()
                }),
            game,
            convoke,
            waterbend,
            power_sensitive: power_sensitive || color_count_sensitive,
            creatures_died: game.state.creatures_died_this_turn,
            controlled_creatures_died: game
                .state
                .players
                .each_ref()
                .map(|p| p.controlled_creatures_died_this_turn),
            graveyard_left: game
                .state
                .players
                .each_ref()
                .map(|p| p.card_left_graveyard_this_turn),
            actor: goal.actor,
            life_lost: game
                .state
                .players
                .each_ref()
                .map(|p| p.life_lost_this_turn > 0),
            life_gained: game
                .state
                .players
                .each_ref()
                .map(|p| p.life_gained_this_turn > 0),
            active: game.state.active,
            mana: game.state.players[goal.actor.index()].mana.map(u128::from),
            lots: game.mana_lots(goal.actor).to_vec(),
            spell_lot_demand: Some(
                goal.required.into_iter().map(u128::from).sum::<u128>() + u128::from(goal.generic),
            ),
            purpose: game.funding_purpose(goal),
            payment_source: goal.source.as_ref().map(|(source, ..)| *source),
            required_face_up: game.face_up_payment().map(|request| request.object),
            contributors: game.contributor_requirement(goal),
            payment_departures: Vec::new(),
            watched_sources: Vec::new(),
            life: game.state.players[goal.actor.index()].life,
            other_life: game.state.players[goal.actor.other().index()].life,
            life_replacement_possible,
            resources,
            named_counters: game.state.named_counters.clone(),
            counter_placement_history: game.state.counter_placement_history.clone(),
        };
        if game
            .state
            .roles
            .as_ref()
            .is_some_and(|r| !r.source_linked_effects.is_empty())
        {
            result.prepare_entry_projection();
        }
        result.set_purpose(result.purpose);
        result.set_cost_context(game.state.announcing.as_ref());
        result.contributors = game.contributor_requirement(goal);
        result.power_sensitive |= result.contributors.is_some();
        if let Some(a) = game.state.announcing.as_ref()
            && !a.additional_paid
            && a.additional_cost().is_some_and(|s| {
                evidence::threshold(s).is_some()
                    || matches!(s.kind, EffectChoiceKind::ExileGraveyardSymbols { .. })
                    || s.kind.counter_requirement().is_some()
            })
        {
            // Preserve exact additional-cost witnesses while nested mana
            // abilities consume fuel, including selected creature counters.
            result.set_payment_context(a);
        }
        if let Some(a) = game.state.announcing.as_ref()
            && goal
                .source
                .as_ref()
                .is_some_and(|(r, index, _, definition, slot)| {
                    a.item
                        .source
                        .expect("cost announcement has object source")
                        .object
                        == *r
                        && a.item.kind == StackKind::ActivatedAbility { ability: *index }
                        && a.item
                            .source
                            .expect("cost announcement has object source")
                            .card
                            == *definition
                        && a.item.borrowed_slot == *slot
                })
            && (game.state.mana_announcement_stack.is_some()
                || a.mana_output
                    .as_ref()
                    .and_then(|o| o.resume.as_deref())
                    .is_some_and(|pending| game.guided_mana_goal(pending).is_some()))
        {
            result.set_funding_continuation(
                a,
                game.state
                    .mana_announcement_stack
                    .as_deref()
                    .map_or(&[][..], |frames| frames.as_slice()),
                game.state
                    .mana_lots
                    .as_deref()
                    .and_then(|m| m.announcement_quality.as_ref())
                    .filter(|(source, _)| {
                        *source
                            == a.item
                                .source
                                .expect("cost announcement has object source")
                                .object
                    })
                    .map(|(_, q)| q),
            );
        }
        result
    }
    fn cost_group_ready(&self) -> bool {
        self.cost_group.as_ref().is_none_or(|(spec, group)| {
            !matches!(
                spec.kind.payment_kind(),
                EffectChoiceKind::ExileGraveyardRange { .. }
            ) || group.iter().all(|r| {
                !ability_world::CharacteristicWorld::graveyard_exile_prohibited(self, r.card)
            })
        })
    }
    fn obligations(&self, goal: &FundingGoal) -> bool {
        let source = goal
            .source
            .as_ref()
            .and_then(|(r, _, _, _, _)| self.resources.iter().find(|s| s.id == r.card));
        let reserve = goal
            .reserve
            .and_then(|(r, _)| self.resources.iter().find(|s| s.id == r.card));
        (self.cast_costs.is_some()
            || self.cost_group.as_ref().is_none_or(|(spec, group)| {
                group.iter().all(|r| {
                    let o = ability_world::CharacteristicWorld::object(self, r.card);
                    o.generation == r.generation
                        && (!matches!(
                            spec.kind.payment_kind(),
                            EffectChoiceKind::Sacrifice
                                | EffectChoiceKind::SacrificeRange { .. }
                                | EffectChoiceKind::SacrificeDifferentNames(_)
                                | EffectChoiceKind::SacrificePermanents(_)
                                | EffectChoiceKind::SacrificeAttachedEquipment
                                | EffectChoiceKind::SacrificeAttachedPermanent
                        ) || !self.sacrifice_prohibited(r.card))
                        && (spec.kind.payment_kind() != EffectChoiceKind::DiscardLegendary
                            || ability_world::CharacteristicWorld::legendary(self, r.card))
                        && (spec.kind.payment_kind() != EffectChoiceKind::DiscardMatchingLegendary
                            || additional_costs::legendary_name_match(self, goal.actor, r.card))
                        && Game::additional_object_matches(
                            *spec,
                            goal.actor,
                            o,
                            ability_world::CharacteristicWorld::kinds(self, r.card),
                            ability_world::CharacteristicWorld::mana_value(self, r.card),
                            self.keywords(&Resource {
                                id: r.card,
                                attacking: self
                                    .game
                                    .state
                                    .attackers
                                    .iter()
                                    .any(|a| a.card == r.card),
                                object: o.clone(),
                            }),
                            ability_world::CharacteristicWorld::subtypes(self, r.card),
                            ability_world::CharacteristicWorld::name(self, r.card),
                            || ability_world::CharacteristicWorld::legendary(self, r.card),
                        )
                })
            }))
            && self.sacrifice_group.iter().all(|r| {
                let o = ability_world::CharacteristicWorld::object(self, r.card);
                o.zone == Zone::Battlefield
                    && o.generation == r.generation
                    && o.controller == goal.actor
                    && !self.sacrifice_prohibited(r.card)
                    && o.is_token()
                    && ability_world::CharacteristicWorld::kinds(self, r.card) & 16 != 0
            })
            && self.game.state.announcing.as_ref().is_none_or(|a| {
                a.additional_group
                    .as_ref()
                    .filter(|_| {
                        a.additional_cost().is_some_and(|s| {
                            matches!(s.kind, EffectChoiceKind::SacrificeDifferentNames(_))
                        })
                    })
                    .is_none_or(|group| {
                        group.iter().all(|r| {
                            let o = ability_world::CharacteristicWorld::object(self, r.card);
                            o.zone == Zone::Battlefield
                                && o.generation == r.generation
                                && (a.additional_cost().is_some_and(|s| {
                                    matches!(s.kind, EffectChoiceKind::SacrificeDifferentNames(_))
                                }) || !o.tapped)
                                && o.controller == goal.actor
                        })
                    })
            })
            && goal.reserve.is_none_or(|(r, spec)| {
                (!matches!(
                    spec.kind.payment_kind(),
                    EffectChoiceKind::Sacrifice
                        | EffectChoiceKind::SacrificeRange { .. }
                        | EffectChoiceKind::SacrificeDifferentNames(_)
                        | EffectChoiceKind::SacrificePermanents(_)
                        | EffectChoiceKind::SacrificeAttachedEquipment
                        | EffectChoiceKind::SacrificeAttachedPermanent
                ) || !self.sacrifice_prohibited(r.card))
                    && (spec.kind.payment_kind() != EffectChoiceKind::ReturnUnblockedAttacker
                        || self.game.ninjutsu_attacker(goal.actor, r)
                            && ability_world::CharacteristicWorld::attacking(self, r.card))
                    && (spec.kind.payment_kind() != EffectChoiceKind::DiscardLegendary
                        || ability_world::CharacteristicWorld::legendary(self, r.card))
                    && (spec.kind.payment_kind() != EffectChoiceKind::DiscardMatchingLegendary
                        || additional_costs::legendary_name_match(self, goal.actor, r.card))
            })
            && reserve_obligations(self, goal, reserve.map(|r| &r.object), |_| {
                self.keywords(reserve.expect("reserved resource"))
            })
            && goal
                .source
                .as_ref()
                .is_none_or(|(r, index, _, definition, _)| {
                    !definition.activation(*index).cost.sacrifice
                        || !self.sacrifice_prohibited(r.card)
                })
            && source_obligations(
                goal,
                source.map(|r| &r.object),
                source.map_or(CounterAmounts::default(), |r| {
                    ability_world::CharacteristicWorld::counter_amounts(self, r.id)
                }),
                source.map_or(0, |r| self.kinds(r)),
                || {
                    source.map_or(Keywords::NONE, |r| {
                        ability_world::CharacteristicWorld::activation_keywords(self, r.id)
                    })
                },
                self.life,
                self.exhaust_source_reuse_authorized,
                self.source_use_independent,
            )
    }
    fn granted_subtypes(&self, id: CardId) -> GrantedSubtypes {
        ability_world::CharacteristicWorld::subtypes(self, id)
    }
    fn stat_value(&self, value: StatValue, source: &Resource) -> i128 {
        ability_world::CharacteristicWorld::value(self, value, source.id)
    }
    fn kinds(&self, r: &Resource) -> u16 {
        ability_world::CharacteristicWorld::kinds(self, r.id)
    }
    fn keywords(&self, target: &Resource) -> Keywords {
        if let Some(evaluation) = ability_evaluation::evaluate_if_needed(self) {
            return evaluation.result.objects[target.id as usize].keywords;
        }
        let mut keywords = ability_world::CharacteristicWorld::intrinsic_keywords(self, target.id)
            .union(target.object.gained_keywords)
            .union(ability_world::CharacteristicWorld::copied_keywords(
                self, target.id,
            ))
            .union(ability_world::CharacteristicWorld::counter_subtype_keywords(self, target.id))
            .union(ability_world::CharacteristicWorld::source_conditional_keywords(self, target.id))
            .union(ability_world::CharacteristicWorld::placement_history_keywords(self, target.id));
        if target.object.zone == Zone::Battlefield && self.kinds(target) & 1 != 0 {
            let reference = ObjectRef {
                card: target.id,
                generation: target.object.generation,
            };
            for source in &self.resources {
                if source.object.zone == Zone::Battlefield
                    && self.kinds(source) & 1 == 0
                    && source.object.attached_to == Some(reference)
                    && let Some(spec) = source.object.card.attachment_bonus()
                {
                    keywords = keywords.union(spec.keywords);
                    if source.object.controller == self.game.state.active {
                        keywords = keywords.union(source.object.card.attached_turn_keywords());
                    }
                }
            }
        }
        if target.object.zone == Zone::Battlefield {
            for source in &self.resources {
                if source.object.zone != Zone::Battlefield
                    && !(source.object.zone == Zone::Command && source.object.card.is_emblem())
                {
                    continue;
                }
                if self.kinds(target) & 1 != 0
                    && target.object.controller == source.object.controller
                {
                    for &(kind, grant) in source.object.card.counter_keyword_grants() {
                        if ability_world::CharacteristicWorld::counter_count(self, target.id, kind)
                            > 0
                        {
                            keywords = keywords.union(grant);
                        }
                    }
                }
                for group in source.object.card.static_ability_groups() {
                    for fragment in group.fragments {
                        match *fragment {
                            crate::cards::StaticAbilityFragment::ControlledPermanentKeywords {
                                kinds,
                                keywords: grant,
                            } if target.object.controller == source.object.controller
                                && self.kinds(target) & kinds != 0 =>
                            {
                                keywords = keywords.union(grant)
                            }
                            crate::cards::StaticAbilityFragment::ControlledSubtypeKeywords {
                                subtype,
                                keywords: grant,
                            } if target.object.controller == source.object.controller
                                && ability_world::CharacteristicWorld::subtypes(
                                    self, target.id,
                                )
                                .has(
                                    ability_world::CharacteristicWorld::definition(self, target.id),
                                    subtype,
                                ) =>
                            {
                                keywords = keywords.union(grant)
                            }
                            crate::cards::StaticAbilityFragment::GrantRiotToOtherControlledSubtype(subtype)
                                if target.object.controller == source.object.controller
                                    && target.id != source.id
                                    && ability_world::CharacteristicWorld::subtypes(self, target.id)
                                        .has(ability_world::CharacteristicWorld::definition(self, target.id), subtype) =>
                            {
                                keywords = keywords.union(Keywords::RIOT)
                            }
                            crate::cards::StaticAbilityFragment::ControlledEquippedCreatureKeywords(grant)
                                if target.object.controller == source.object.controller
                                    && self.kinds(target) & 1 != 0
                                    && self.equipped(target.id) =>
                            {
                                keywords = keywords.union(grant)
                            }
                            crate::cards::StaticAbilityFragment::ControlledTappedKeywords {
                                other,
                                keywords: grant,
                            } if target.object.controller == source.object.controller
                                && target.object.tapped
                                && self.kinds(target) & 1 != 0
                                && (!other || target.id != source.id) =>
                            {
                                keywords = keywords.union(grant)
                            }
                            StaticAbilityFragment::ControlledOtherTappedLegendaryCreatureKeywords {
                                condition,
                                keywords: grant,
                            } if ability_world::CharacteristicWorld::other_tapped_legendary_creature_applies(
                                self,
                                self.reference(source.id),
                                target.id,
                                condition,
                            ) => {
                                keywords = keywords.union(grant);
                            }
                            crate::cards::StaticAbilityFragment::ControlledUntappedSubtypeKeywords {
                                subtype,
                                keywords: grant,
                            } if !target.object.tapped
                                && target.object.controller == source.object.controller
                                && ability_world::CharacteristicWorld::subtypes(self, target.id)
                                    .has(ability_world::CharacteristicWorld::definition(self, target.id), subtype) =>
                            {
                                keywords = keywords.union(grant)
                            }
                            crate::cards::StaticAbilityFragment::PlusOnePlacedKeywords {
                                filter,
                                keywords: grant,
                            } if self.placement_history_recipient(
                                self.reference(source.id),
                                filter,
                                target.id,
                            ) =>
                            {
                                keywords = keywords.union(grant)
                            }
                            crate::cards::StaticAbilityFragment::DeathBaronLord
                                if ability_world::CharacteristicWorld::death_baron_applies(
                                    self,
                                    ObjectRef {
                                        card: source.id,
                                        generation: source.object.generation,
                                    },
                                    target.id,
                                ) =>
                            {
                                keywords = keywords.union(Keywords::DEATHTOUCH)
                            }
                            crate::cards::StaticAbilityFragment::CounterKeywordOwnTurn(
                                kind,
                                grant,
                            ) if source.object.controller == self.active
                                && target.object.controller == source.object.controller
                                && self.kinds(target) & 1 != 0
                                && ability_world::CharacteristicWorld::counter_count(
                                    self, target.id, kind,
                                ) > 0 =>
                            {
                                keywords = keywords.union(grant)
                            }
                            _ => (),
                        }
                    }
                }
                for grant in source.object.card.keyword_grants() {
                    if grant
                        .condition
                        .is_none_or(|value| self.stat_value(value, source) != 0)
                        && (!grant.requires_life_gain
                            || self.life_gained[source.object.controller.index()])
                        && grant.matches_characteristics(
                            target.object.card,
                            target.object.is_token(),
                            self.kinds(target),
                            self.granted_subtypes(target.id),
                            target.object.controller == source.object.controller,
                            target.id == source.id
                                && target.object.generation == source.object.generation,
                            target.attacking,
                            source.object.controller == self.active,
                        )
                    {
                        keywords = keywords.union(grant.keywords);
                    }
                }
            }
        }
        keywords
    }
    fn guard_available(&self, guard: ActivationGuard, source: CardId, actor: Player) -> bool {
        match guard {
            ActivationGuard::SourceCounterAtLeast { kind, minimum } => {
                ability_world::CharacteristicWorld::counter_count(self, source, kind) >= minimum
            }
            ActivationGuard::ClassLevelExactly(level) => {
                ability_world::CharacteristicWorld::object(self, source).class_level == level
            }
            ActivationGuard::CastInstantSorceryThisTurn => {
                self.game.state.players[actor.index()].instant_sorcery_cast_this_turn > 0
            }
            ActivationGuard::Always => true,
            ActivationGuard::ScriedOrSurveilledThisTurn => {
                self.game.state.players[actor.index()].scried_or_surveilled_this_turn
            }
            ActivationGuard::OpponentNoncombatDamageThisTurn => {
                self.game.state.players[actor.other().index()].noncombat_damage_this_turn
            }
            ActivationGuard::OpponentWasDealtLegendaryCombatDamageThisTurn => self
                .game
                .player_was_dealt_legendary_combat_damage(actor.other()),
            ActivationGuard::SourceSolved => self.game.case_solved(self.reference(source)),
            ActivationGuard::SourceAttackedThisTurn => self
                .game
                .state
                .attacked_this_turn
                .contains(&self.reference(source)),
            ActivationGuard::ControlCreatureToughnessAtLeast(minimum) => {
                activations::controls_tough_creature(self, actor, minimum)
            }
            ActivationGuard::GraveyardCardsAtLeast { kind, minimum } => {
                activations::graveyard_cards_at_least(self, actor, kind, minimum)
            }
            ActivationGuard::SourceEnteredOrControlsBasicLand => {
                ability_world::CharacteristicWorld::entered_this_turn(self, self.reference(source))
                    || ability_world::CharacteristicWorld::controls_basic_land(self, actor)
            }
            ActivationGuard::HandSizeAtLeast(minimum) => {
                self.game
                    .state
                    .objects
                    .iter()
                    .enumerate()
                    .filter(|(id, _)| {
                        let object =
                            ability_world::CharacteristicWorld::object(self, *id as CardId);
                        object.zone == Zone::Hand && object.owner == actor && object.is_card()
                    })
                    .count()
                    >= usize::from(minimum)
            }
            ActivationGuard::LifeAboveStarting(extra) => {
                (if actor == self.actor {
                    self.life
                } else {
                    self.game.state.players[actor.index()].life
                }) >= STARTING_LIFE + i128::from(extra)
            }
            ActivationGuard::OwnTurn => self.active == actor,
            ActivationGuard::OwnTurnAfterGraveyardDeparture => {
                self.active == actor
                    && ability_world::CharacteristicWorld::card_left_graveyard(self, actor)
            }
            ActivationGuard::ControlledPlaneswalkerSubtypeLoyaltyAtLeast { subtype, minimum } => {
                self.resources
                    .iter()
                    .filter(|r| {
                        r.object.zone == Zone::Battlefield
                            && r.object.controller == actor
                            && self.kinds(r) & 64 != 0
                            && self.granted_subtypes(r.id).has(r.object.card, subtype)
                    })
                    .fold(0u64, |sum, r| {
                        sum.saturating_add(ability_world::CharacteristicWorld::counter_count(
                            self,
                            r.id,
                            CounterKind::Loyalty,
                        ))
                    })
                    >= minimum
            }
            ActivationGuard::ControlColorsAtLeast(minimum) => {
                ability_world::controlled_colors(self, actor).count_ones() >= u32::from(minimum)
            }
            ActivationGuard::GraveyardSubtype(subtype) => {
                ability_world::graveyard_subtype(self, actor, subtype)
            }
            ActivationGuard::ControlLandsAtLeast(minimum) => {
                self.resources
                    .iter()
                    .filter(|r| {
                        r.object.zone == Zone::Battlefield
                            && r.object.controller == actor
                            && self.kinds(r) & 2 != 0
                    })
                    .count()
                    >= usize::from(minimum)
            }
            ActivationGuard::ControlSubtypeAny(_) => self.resources.iter().any(|r| {
                r.object.zone == Zone::Battlefield
                    && r.object.controller == actor
                    && guard
                        .subtypes()
                        .iter()
                        .any(|s| self.granted_subtypes(r.id).has(r.object.card, s))
            }),
        }
    }
    fn mana_activations(
        &self,
        id: CardId,
    ) -> impl Iterator<
        Item = (
            u16,
            &'static ActivatedAbility,
            Option<BorrowedDescriptor>,
            Option<(Card, u64)>,
        ),
    > {
        let object = ability_world::CharacteristicWorld::object(self, id);
        object
            .card
            .activations()
            .iter()
            .enumerate()
            .filter(move |(index, ability)| {
                !object.card.self_granted_activation_template(*index as u16)
                    && !object.card.legacy_subtype_mana_activation(*index as u16)
                    && ability.mana_production().is_some()
                    && ability.source_zone() == object.zone
                    && if object.card.grants_activation(*index as u16) {
                        ability_world::CharacteristicWorld::source_activation_template_available(
                            self,
                            id,
                            *index as u16,
                        )
                    } else {
                        ability_world::CharacteristicWorld::printed_abilities_available(self, id)
                    }
            })
            .map(|(index, ability)| (index as u16, ability, None, None))
            .chain(
                borrowed_activations::abilities(self, id)
                    .into_iter()
                    .filter(|ability| ability.rules().mana_production().is_some())
                    .map(|ability| (ability.ability, ability.rules(), Some(ability), None)),
            )
            .chain(
                self.game
                    .state
                    .self_granted_abilities
                    .as_deref()
                    .into_iter()
                    .flatten()
                    .filter(move |row| {
                        row.recipient == self.reference(id)
                            && row
                                .expires_turn
                                .is_none_or(|turn| turn == self.game.state.turn)
                            && characteristic_query::CharacteristicQuery::in_world(self)
                                .self_grant_available(row.recipient, row.timestamp)
                    })
                    .filter_map(|row| {
                        let ability = row.kind.activation()?;
                        let rules = row.granting_card.activation(ability);
                        rules.mana_production().map(|_| {
                            (
                                ability,
                                rules,
                                None,
                                Some((row.granting_card, row.timestamp)),
                            )
                        })
                    }),
            )
    }
    fn moves(&self, goal: &FundingGoal) -> Vec<FundingMove> {
        self.mana_moves(goal, false, Some(false))
    }
    fn deferred_moves(&self, goal: &FundingGoal) -> Vec<FundingMove> {
        self.mana_moves(goal, false, Some(true))
    }
    fn action_moves(&self, goal: &FundingGoal) -> Vec<FundingMove> {
        self.mana_moves(goal, true, None)
    }
    /// External witnesses belong to the later additional-cost choice. Search
    /// every exact projection until one succeeds, then expose its public mana
    /// announcement once. Ordinary moves retain their allocation-free visitor.
    fn visit_distinct_actions(
        &self,
        goal: &FundingGoal,
        mut viable: impl FnMut(&FundingMove) -> bool,
        emit: &mut impl FnMut(Action) -> bool,
    ) -> bool {
        let mut announced = Vec::new();
        for step in self.action_moves(goal) {
            let external_action = step.external.as_ref().map(|_| self.action(&step));
            if external_action
                .as_ref()
                .is_some_and(|action| announced.contains(action))
                || !viable(&step)
            {
                continue;
            }
            let action = if let Some(action) = external_action {
                announced.push(action.clone());
                action
            } else {
                self.action(&step)
            };
            if emit(action) {
                return true;
            }
        }
        false
    }
    fn visit_tap_mana_cost_groups(
        &self,
        spec: AdditionalCost,
        source: ObjectRef,
        prefix: &[ObjectRef],
        excluded: impl Fn(ObjectRef) -> bool,
        emit: &mut dyn FnMut(&[ObjectRef]) -> bool,
    ) -> bool {
        let EffectChoiceKind::TapPermanents(amount) = spec.kind.payment_kind() else {
            unreachable!("mana tap group requires a tap cost")
        };
        let amount = amount as usize;
        let eligible = |object| !excluded(object) && self.cost_object(spec, object, source);
        if amount == 0
            || prefix.len() > amount
            || prefix
                .iter()
                .enumerate()
                .any(|(index, &object)| prefix[..index].contains(&object) || !eligible(object))
        {
            return false;
        }
        if prefix.len() == amount {
            return emit(prefix);
        }
        let candidates = self
            .resources
            .iter()
            .map(|resource| self.reference(resource.id))
            .filter(|object| !prefix.contains(object) && eligible(*object))
            .collect::<Vec<_>>();
        fn visit_groups(
            group: &mut Vec<ObjectRef>,
            candidates: &[ObjectRef],
            left: usize,
            emit: &mut dyn FnMut(&[ObjectRef]) -> bool,
        ) -> bool {
            if left == 0 {
                return emit(group);
            }
            let Some(last) = candidates.len().checked_sub(left) else {
                return false;
            };
            for (index, &object) in candidates.iter().enumerate().take(last + 1) {
                group.push(object);
                let done = visit_groups(group, &candidates[index + 1..], left - 1, emit);
                group.pop();
                if done {
                    return true;
                }
            }
            false
        }
        visit_groups(
            &mut prefix.to_vec(),
            &candidates,
            amount - prefix.len(),
            emit,
        )
    }
    fn mana_moves(
        &self,
        goal: &FundingGoal,
        late_fixed: bool,
        deferred_only: Option<bool>,
    ) -> Vec<FundingMove> {
        // Neither an actual payment nor a proposal may invent mana activations
        // when its final total is zero. Positive totals retain the window even
        // when later Convoke/Delve choices can pay them without spending mana.
        if !self.mana_window_open(goal) {
            return Vec::new();
        }
        let mut moves = Vec::new();
        let actual_active;
        let active_mana = if let Some(keys) = self.active_mana_keys.as_deref() {
            keys
        } else {
            actual_active = self.game.active_mana_activation_keys();
            &actual_active
        };
        for (resource, entry) in self.resources.iter().enumerate() {
            let object = &entry.object;
            if !matches!(
                object.zone,
                Zone::Battlefield | Zone::Hand | Zone::Graveyard
            ) || (if object.zone == Zone::Battlefield {
                object.controller
            } else {
                object.owner
            }) != goal.actor
            {
                continue;
            }
            if ability_world::CharacteristicWorld::activation_prohibited(self, goal.actor, entry.id)
            {
                continue;
            }
            let reserved_tap = goal.reserves_source_tap(self.reference(entry.id));
            if !reserved_tap
                && object.zone == Zone::Battlefield
                && !object.tapped
                && (self.kinds(entry) & 1 == 0
                    || !object.summoning_sick
                    || ability_world::CharacteristicWorld::activation_keywords(self, entry.id)
                        .contains(Keywords::HASTE))
            {
                for grant in ability_world::CharacteristicWorld::granted_mana(self, entry.id) {
                    if grant.sacrifice && self.sacrifice_prohibited(entry.id) {
                        continue;
                    }
                    let increase = self.named_activation_generic_increase(entry.id);
                    let index = if grant.sacrifice {
                        Card::GRANTED_SACRIFICE_MANA
                    } else {
                        Card::GRANTED_TAP_MANA
                    };
                    let key = mana_announcements::ManaActivationKey {
                        source: self.reference(entry.id),
                        definition: object.card,
                        ability: index,
                        borrowed_slot: None,
                        self_grant: None,
                        source_origin: self.ability_origin(entry.id),
                        grant: Some((
                            grant.source,
                            self.definition(grant.source.card),
                            self.ability_origin(grant.source.card),
                        )),
                    };
                    if active_mana.contains(&key) {
                        continue;
                    }
                    let purpose =
                        mana_lots::Purpose::Activation(mana_lots::Source::in_world(self, entry.id));
                    funding_cost_payments(
                        mana_lots::available(self.mana, &self.lots, purpose),
                        ActivationCost::ZERO,
                        if increase == 0 {
                            power_up::ActivationDiscounts::default()
                        } else {
                            self.controlled_activation_discounts(entry.id)
                        },
                        increase,
                        None,
                        |cost| cost,
                        |payment, deferred| {
                            if deferred_only.is_some_and(|only| only != deferred.is_some()) {
                                return;
                            }
                            funding_allocations(
                                self.mana,
                                &self.lots,
                                purpose,
                                payment,
                                deferred,
                                |allocation| {
                                    grant.production.visit_at_start_with_fixed_choice(
                                        self,
                                        entry.id,
                                        late_fixed && (grant.sacrifice || increase != 0),
                                        |production| {
                                            moves.push(FundingMove {
                                                resource,
                                                ability: (increase != 0).then_some(index),
                                                grantor: Some(grant.source),
                                                borrowed: None,
                                                self_grant: None,
                                                payment,
                                                counters: Counters::default(),
                                                production,
                                                quality: mana_lots::quality_from_world(
                                                    self,
                                                    grant.production,
                                                    entry.id,
                                                ),
                                                allocation: allocation.clone(),
                                                external: None,
                                                frozen: None,
                                                output: Some(Box::new({
                                                    let mut o = mana_output::ManaOutput::new(
                                                        self,
                                                        grant.production,
                                                        production,
                                                        key,
                                                        goal.actor,
                                                        mana_lots::quality_from_world(
                                                            self,
                                                            grant.production,
                                                            entry.id,
                                                        ),
                                                    );
                                                    o.deferred_cost = deferred;
                                                    o
                                                })),
                                            });
                                            false
                                        },
                                    );
                                },
                            );
                        },
                    );
                }
            }
            if !reserved_tap
                && object.zone == Zone::Battlefield
                && ability_world::CharacteristicWorld::printed_abilities_available(self, entry.id)
                && !object.tapped
                && (self.kinds(entry) & 1 == 0
                    || !object.summoning_sick
                    || self.activation_keywords(entry.id).contains(Keywords::HASTE))
                && let Some(color) = self.intrinsic_mana_color(entry.id)
            {
                let mut production = [0; 6];
                production[color] = 1;
                let index = Card::intrinsic_mana_activation_index(color);
                let increase = self.named_activation_generic_increase(entry.id);
                let key = mana_announcements::ManaActivationKey {
                    source: self.reference(entry.id),
                    definition: object.card,
                    ability: index,
                    borrowed_slot: None,
                    self_grant: None,
                    source_origin: self.ability_origin(entry.id),
                    grant: None,
                };
                if !active_mana.contains(&key) {
                    let purpose =
                        mana_lots::Purpose::Activation(mana_lots::Source::in_world(self, entry.id));
                    funding_cost_payments(
                        mana_lots::available(self.mana, &self.lots, purpose),
                        ActivationCost::ZERO,
                        if increase == 0 {
                            power_up::ActivationDiscounts::default()
                        } else {
                            self.controlled_activation_discounts(entry.id)
                        },
                        increase,
                        None,
                        |cost| cost,
                        |payment, deferred| {
                            if deferred_only.is_some_and(|only| only != deferred.is_some()) {
                                return;
                            }
                            funding_allocations(
                                self.mana,
                                &self.lots,
                                purpose,
                                payment,
                                deferred,
                                |allocation| {
                                    moves.push(FundingMove {
                                        resource,
                                        ability: (increase != 0).then_some(index),
                                        grantor: None,
                                        borrowed: None,
                                        self_grant: None,
                                        payment,
                                        counters: Counters::default(),
                                        production,
                                        quality: mana_lots::quality_from_world(
                                            self,
                                            ManaProduction::Fixed(production),
                                            entry.id,
                                        ),
                                        allocation,
                                        external: None,
                                        frozen: None,
                                        output: deferred.map(|cost| {
                                            Box::new({
                                                let mut o = mana_output::ManaOutput::new(
                                                    self,
                                                    ManaProduction::Fixed(production),
                                                    production,
                                                    key,
                                                    goal.actor,
                                                    None,
                                                );
                                                o.deferred_cost = Some(cost);
                                                o
                                            })
                                        }),
                                    });
                                },
                            );
                        },
                    );
                }
            }
            for (index, ability, borrowed, self_grant) in self.mana_activations(entry.id) {
                let Some(production) = ability.mana_production() else {
                    continue;
                };
                {
                    let origin = ability_world::CharacteristicWorld::ability_origin(self, entry.id);
                    let source = self.reference(entry.id);
                    let key = mana_announcements::ManaActivationKey {
                        source,
                        definition: self_grant.map_or_else(
                            || borrowed.map_or(object.card, |row| row.definition),
                            |g| g.0,
                        ),
                        ability: index,
                        borrowed_slot: borrowed.map(|row| row.slot),
                        self_grant: self_grant.map(|g| g.1),
                        source_origin: self_grant.map(|g| g.1).or(origin),
                        grant: (borrowed.is_none()
                            && self_grant.is_none()
                            && self.source_activation_template_available(entry.id, index))
                        .then_some((source, object.card, origin)),
                    };
                    if active_mana.contains(&key) {
                        continue;
                    }
                }

                if (reserved_tap && ability.cost.tap)
                    || (ability.cost.sacrifice && self.sacrifice_prohibited(entry.id))
                    || (ability.cost.exile_self && self.graveyard_exile_prohibited(entry.id))
                {
                    continue;
                }
                // CR605.3c forbids recursively activating the same mana ability
                // before the enclosing activation has resolved.
                if active_mana.is_empty()
                    && goal
                        .source
                        .as_ref()
                        .is_some_and(|(source, pending, _, definition, slot)| {
                            *definition
                                == self_grant.map_or_else(
                                    || borrowed.map_or(object.card, |a| a.definition),
                                    |g| g.0,
                                )
                                && *slot == borrowed.map(|a| a.slot)
                                && source.card == entry.id
                                && object.generation == source.generation
                                && *pending == index
                        })
                {
                    continue;
                }
                let mut readiness = *ability;
                if readiness.exhaust && exhaust::reuse_available(self, goal.actor) {
                    readiness.exhaust = false;
                }
                let mut ready_object = object.clone();
                if borrowed.is_some() {
                    ready_object.used_abilities = 0;
                }
                if let Some((definition, timestamp)) = self_grant {
                    if self.self_grant_activation_used(
                        self.reference(entry.id),
                        timestamp,
                        definition,
                        index,
                        goal.actor,
                    ) {
                        continue;
                    }
                    ready_object.used_abilities = 0;
                }
                if borrowed.is_some_and(|a| {
                    a.used && (a.once_per_turn || readiness.once_per_turn || readiness.exhaust)
                }) || !self.guard_available(ability.guard, entry.id, goal.actor)
                    || !Game::object_activation_available_with_kinds(
                        &ready_object,
                        ability_world::CharacteristicWorld::counter_amounts(self, entry.id),
                        self.kinds(entry),
                        || ability_world::CharacteristicWorld::activation_keywords(self, entry.id),
                        self.life,
                        &readiness,
                        if borrowed.is_some() {
                            0
                        } else {
                            index as usize
                        },
                        goal.actor,
                        false,
                    )
                {
                    continue;
                }
                let external = if let Some(spec) = ability.cost.additional {
                    use additional_costs::ManaCostObjects;
                    let source = self.reference(entry.id);
                    let departs = ability.cost.sacrifice || ability.cost.exile_self;
                    let eligible = match spec.kind.payment_kind() {
                        EffectChoiceKind::TapPermanents(_) => {
                            let mut eligible = Vec::new();
                            self.visit_tap_mana_cost_groups(
                                spec,
                                source,
                                &[],
                                |object| {
                                    goal.reserves_source_tap(object)
                                        || (ability.cost.tap || departs) && source == object
                                },
                                &mut |objects| {
                                    eligible.push(ExternalManaCost {
                                        selection: ManaCostObjects::from_slice(objects),
                                        kind: spec.kind,
                                    });
                                    false
                                },
                            );
                            eligible
                        }
                        EffectChoiceKind::ExileGraveyard | EffectChoiceKind::Sacrifice => {
                            self.resources
                                .iter()
                                .filter_map(|resource| {
                                    let object = self.reference(resource.id);
                                    // A permanent cannot pay two separate departure costs.
                                    ((!departs || object != source)
                                        && self.cost_object(spec, object, source))
                                    .then_some(ExternalManaCost {
                                        selection: ManaCostObjects::One(object),
                                        kind: spec.kind,
                                    })
                                })
                                .collect()
                        }
                        _ => continue,
                    };
                    if eligible.is_empty() {
                        continue;
                    }
                    Some(eligible)
                } else {
                    None
                };
                activations::counter_payments(
                    ability.cost,
                    ability_world::CharacteristicWorld::counter_amounts(self, entry.id),
                    |counters| {
                        let purpose = mana_lots::Purpose::Activation(mana_lots::Source {
                            card: object.card,
                            mana_value: ability_world::CharacteristicWorld::mana_value(
                                self, entry.id,
                            ),
                            foreign_stash: false,
                            cast_from_hand: false,
                            cast_from_exile: false,
                            spend_any_mana: false,
                            linked_mana_source: None,
                            activation_object: Some(self.reference(entry.id)),
                            equip: ability.is_equip(),
                            kinds: self.kinds(entry),
                            subtypes: ability_world::CharacteristicWorld::subtypes(self, entry.id),
                        });
                        let pool = mana_lots::available(self.mana, &self.lots, purpose);
                        let quality = mana_lots::quality_from_world(self, production, entry.id)
                            .map(|q| {
                                q.with_definition(self_grant.map_or_else(
                                    || borrowed.map_or(object.card, |a| a.definition),
                                    |g| g.0,
                                ))
                            });
                        funding_cost_payments(
                            pool,
                            ability.cost,
                            self.activation_generic_discount(
                                entry.id,
                                self_grant.map_or_else(
                                    || borrowed.map_or(object.card, |a| a.definition),
                                    |g| g.0,
                                ),
                                index,
                            ),
                            self.named_activation_generic_increase(entry.id),
                            self.activation_power_up_costs(
                                self.reference(entry.id),
                                self_grant.map_or_else(
                                    || borrowed.map_or(object.card, |a| a.definition),
                                    |g| g.0,
                                ),
                                index,
                            )
                            .as_deref(),
                            |cost| {
                                cast_origins::activation_payment_cost(
                                    self,
                                    goal.actor,
                                    self.reference(entry.id),
                                    cost,
                                )
                            },
                            |payment, deferred| {
                                if deferred_only.is_some_and(|only| only != deferred.is_some()) {
                                    return;
                                }
                                funding_allocations(
                                    self.mana,
                                    &self.lots,
                                    purpose,
                                    payment,
                                    deferred,
                                    |allocation| {
                                        production.visit_at_start_with_fixed_choice(self, entry.id,
                                            late_fixed && matches!(production.base(), ManaProduction::Choice { .. } | ManaProduction::Combination { .. })
                                                && self.fixed_mana_choice_can_be_late(entry.id,
                                                    self_grant.map_or_else(|| borrowed.map_or(object.card, |a| a.definition), |g| g.0), index),
                                            |production| {
                                                let step = FundingMove {
                                                    resource,
                                                    ability: Some(index),
                                                    grantor: None,
                                                    borrowed,
                                                    self_grant,
                                                    payment,
                                                    counters: counters.clone(),
                                                    production,
                                                    quality,
                                                    allocation: allocation.clone(),
                                                    external: None,
                                                    frozen: None,
                                                    output: {
                                                        let definition = self_grant.map_or_else(|| borrowed.map_or(object.card, |a| a.definition), |g| g.0);
                                                        let origin = self.ability_origin(entry.id);
                                                        let key = mana_announcements::ManaActivationKey { source: self.reference(entry.id), definition,
                                                            ability: index, borrowed_slot: borrowed.map(|b| b.slot), self_grant: self_grant.map(|g| g.1),
                                                            source_origin: self_grant.map(|g| g.1).or(origin),
                                                            grant: (borrowed.is_none() && self_grant.is_none() && self.source_activation_template_available(entry.id,index))
                                                                .then_some((self.reference(entry.id), object.card, origin)) };
                                                        (deferred.is_some() || !matches!(ability.mana_production().unwrap().base(), ManaProduction::Fixed(_))).then(|| Box::new({
                                                            let mut o = mana_output::ManaOutput::new(self, ability.mana_production().unwrap(), production, key, goal.actor, quality);
                                                            o.deferred_cost = deferred; o
                                                        }))
                                                    },
                                                };
                                                if let Some(eligible) = &external {
                                                    for external in eligible {
                                                        moves.push(FundingMove {
                                                            external: Some(external.clone()),
                                                            ..step.clone()
                                                        });
                                                    }
                                                } else {
                                                    moves.push(step);
                                                }
                                                false
                                            },
                                        );
                                    },
                                );
                            },
                        );
                        false
                    },
                );
            }
        }
        // Try producing missing required types first; this changes only traversal
        // order, never the set of accepted paths.
        moves.sort_by_key(|step| {
            std::cmp::Reverse(
                (0..6)
                    .map(|i| {
                        u128::from(goal.required[i])
                            .saturating_sub(self.mana[i])
                            .min(u128::from(step.production[i])) as u64
                    })
                    .sum::<u64>(),
            )
        });
        moves
    }
    fn refresh_enduring_projection(&mut self) {
        let acquired = storied::acquisitions(self);
        for (current, acquired) in self.enduring_story.iter_mut().zip(acquired) {
            *current |= acquired;
        }
    }
    fn refresh_control_projection(&mut self) {
        self.refresh_source_bound_copies();
        for row in continuous_control::controllers(self) {
            if let Some(resource) = self
                .resources
                .iter_mut()
                .find(|resource| resource.id == row.object.card)
                && resource.object.controller != row.controller
            {
                resource.object.controller = row.controller;
                resource.object.summoning_sick = true;
                resource.attacking = false;
            }
        }
        self.graveyard_borrowing = graveyard_borrowing::refreshed(self);
        self.borrowed_slots = borrowed_activations::refreshed_slots(self);
    }
    pub(super) fn prune_source_linked_projection(&mut self) {
        let retained: Vec<_> = self
            .roles()
            .into_iter()
            .flat_map(|r| &r.source_linked_effects)
            .filter(|row| source_linked_effects::active(self, row))
            .copied()
            .collect();
        if let Some(p) = &mut self.entry_projection {
            p.roles.source_linked_effects = retained;
        }
    }
    fn expire_projected_blighted_land(&mut self, recipient: ObjectRef) {
        if source_linked_effects::blighted_land_expired(self, recipient) {
            // Own the expired role before any later counter payment can add
            // a blight counter back to the same exact land.
            self.prepare_entry_projection();
            self.entry_projection
                .as_mut()
                .unwrap()
                .roles
                .forget_blighted_land(recipient);
            self.refresh_control_projection();
        }
    }
    fn remember_payment_departure(&mut self, id: CardId) {
        let reference = self.reference(id);
        if let Some(p) = &mut self.entry_projection {
            p.hand_size.forget(reference);
        }
        let is_announcement_source = |item: &SpellView| {
            matches!(item.kind, StackKind::ActivatedAbility { .. })
                && item.source.is_some_and(|source| source.object == reference)
        };
        if self.payment_source != Some(reference)
            && !self.watched_sources.contains(&reference)
            && !self
                .game
                .state
                .announcing
                .as_ref()
                .is_some_and(|a| is_announcement_source(&a.item))
            && !self
                .game
                .state
                .mana_announcement_stack
                .as_deref()
                .is_some_and(|frames| {
                    frames
                        .iter()
                        .any(|frame| is_announcement_source(&frame.announcement.item))
                })
        {
            return;
        }
        if self
            .payment_departures
            .iter()
            .any(|(object, _, _)| *object == reference)
        {
            return;
        }
        let facts = mana_lots::Source::in_world(self, id);
        let power = self.power(id);
        self.payment_departures.push((reference, facts, power));
        self.payment_departures
            .sort_by_key(|(object, _, _)| (object.card, object.generation));
    }
    fn apply(
        &mut self,
        step: &FundingMove,
        preserve_departure_counters: bool,
    ) -> (ManaTapBonus, Option<ObjectRef>) {
        for (lot, allocation) in self.lots.iter_mut().zip(&step.allocation) {
            for (held, paid) in lot.amounts.iter_mut().zip(allocation) {
                *held -= u128::from(*paid);
            }
        }
        self.lots.retain(|lot| lot.amounts != [0; 6]);
        for (pool, paid) in self.mana.iter_mut().zip(step.payment) {
            *pool -= u128::from(paid);
        }
        let id = self.resources[step.resource].id;
        let sacrifice_grant = step.grantor.is_some_and(|source| {
            ability_world::CharacteristicWorld::granted_mana(self, id)
                .iter()
                .any(|g| g.source == source && g.sacrifice)
        });
        let activated_exhaust = step.ability.is_some_and(|index| {
            step.frozen
                .as_deref()
                .map_or_else(
                    || {
                        step.self_grant.map_or_else(
                            || {
                                step.borrowed.map_or_else(
                                    || self.resources[step.resource].object.card.activation(index),
                                    |a| a.rules(),
                                )
                            },
                            |g| g.0.activation(index),
                        )
                    },
                    |(definition, _)| definition.activation(index),
                )
                .exhaust
        });
        if activated_exhaust {
            self.exhaust_activated[self.actor.index()] = true;
        }
        let cost = step.ability.map(|index| {
            step.frozen
                .as_deref()
                .map_or_else(
                    || {
                        step.self_grant.map_or_else(
                            || {
                                step.borrowed.map_or_else(
                                    || self.resources[step.resource].object.card.activation(index),
                                    |a| a.rules(),
                                )
                            },
                            |g| g.0.activation(index),
                        )
                    },
                    |(definition, _)| definition.activation(index),
                )
                .cost
        });
        if let Some(cost) = cost {
            self.pay_projected_life(self.actor, u32::from(cost.life), &[]);
        }
        if step.output.as_ref().is_some_and(|o| {
            self.reference(id) != o.instance.source
                || self.object(id).zone
                    != o.instance
                        .definition
                        .activation(o.instance.ability)
                        .source_zone()
        }) {
            return (ManaTapBonus::default(), None);
        }
        if let Some((definition, timestamp)) = step.self_grant
            && step.ability.is_some_and(|i| {
                let rules = definition.activation(i);
                rules.exhaust || rules.once_per_turn
            })
        {
            self.self_grant_uses.push((self.reference(id), timestamp));
        }
        let object = &mut self.resources[step.resource].object;
        let mut departure = None;
        if let Some(index) = step.ability {
            let cost = cost.expect("activation cost");
            let borrowed = step.borrowed;
            if let Some(slot) = step.frozen.as_deref().and_then(|(_, slot)| *slot) {
                self.borrowed_uses.push((
                    ObjectRef {
                        card: id,
                        generation: object.generation,
                    },
                    slot,
                ));
            } else if let Some(ability) = borrowed {
                self.borrowed_uses.push((ability.source, ability.slot));
            } else if step.self_grant.is_none() {
                object.used_abilities |= 1u64.checked_shl(u32::from(index)).unwrap_or(0);
            }
            let reference = ObjectRef {
                card: id,
                generation: object.generation,
            };
            let mut counters =
                live_counters::amounts(&self.named_counters, reference, object.counters).to_owned();
            counters.pay(&step.counters);
            object.counters = counters.hot();
            live_counters::set(&mut self.named_counters, reference, counters.named);
            if step.counters.get(CounterKind::Blight) != 0 {
                self.expire_projected_blighted_land(reference);
            }
            if cost.sacrifice || cost.exile_self || cost.discard {
                departure = Some(if cost.exile_self {
                    Zone::Exile
                } else {
                    Zone::Graveyard
                });
            }
        } else if sacrifice_grant {
            departure = Some(Zone::Graveyard);
        }
        // Match the owned executor: counters and life are paid before the tap,
        // while the source is still in its current zone and incarnation.
        let pays_tap = cost.is_none_or(|c| c.tap);
        let tap_source = pays_tap.then(|| self.reference(id));
        let mut tap_bonus = ManaTapBonus::default();
        if pays_tap {
            tap_bonus = mana_tap_bonus::capture(self, self.actor, id);
            if CharacteristicWorld::object(self, id).zone == Zone::Battlefield
                && CharacteristicWorld::object(self, id).controller == self.actor
                && CharacteristicWorld::subtypes(self, id)
                    .has(CharacteristicWorld::object(self, id).card, "Mountain")
            {
                tap_bonus.fixed[3] = tap_bonus.fixed[3]
                    .checked_add(self.game.state.players[self.actor.index()].mountain_mana_bonus)
                    .capacity("mana representation exhausted");
            }
        }
        if pays_tap {
            self.resources[step.resource].object.tapped = true;
        }
        // Tap/counter costs are real prior states, even when the departure below
        // still needs replacement planning. Do not award from a tentative raw
        // destination in the preserve-departure path.
        self.refresh_enduring_projection();
        if let Some(destination) = departure {
            let from_battlefield = self.resources[step.resource].object.zone == Zone::Battlefield;
            // Counter and tap payments precede the actual move, as in the real
            // payment path. Capture characteristics before zone effects cease.
            if !preserve_departure_counters {
                if from_battlefield {
                    let controller = self.resources[step.resource].object.controller;
                    self.permanent_left_battlefield[controller.index()] = true;
                    self.void_active_this_turn |= CharacteristicWorld::kinds(self, id) & 2 == 0;
                }
                self.remember_payment_departure(id);
            }
            let reference = self.reference(id);
            // The fast path has already ruled out replacement effects. The
            // slow path commits its replacement-adjusted destination through
            // cost_departures, which accounts for death there instead.
            if !preserve_departure_counters
                && destination == Zone::Graveyard
                && from_battlefield
                && CharacteristicWorld::kinds(self, id) & 1 != 0
            {
                let controller = self.resources[step.resource].object.controller;
                self.creatures_died = self
                    .creatures_died
                    .checked_add(1)
                    .capacity("virtual death history exhausted");
                let deaths = &mut self.controlled_creatures_died[controller.index()];
                *deaths = deaths
                    .checked_add(1)
                    .capacity("virtual controlled death history exhausted");
            }
            let face_down_physical = self.projected_face_down_physical(reference);
            let physical_copy = self
                .temporary_copy_rows_state()
                .as_ref()
                .and_then(|rows| rows.iter().find(|row| row.object == reference))
                .map(|row| (row.physical_card, row.physical_kind));
            let object = &mut self.resources[step.resource].object;
            object.zone = destination;
            if !preserve_departure_counters {
                // A committed zone change creates a new incarnation in both the
                // owned game and the projected world.
                object.generation = object
                    .generation
                    .checked_add(1)
                    .capacity("virtual generation exhausted");
                // The slow path still needs the old face for replacement planning.
                // Normalize only when this step commits the virtual departure.
                if let Some((physical_card, physical_kind)) = physical_copy {
                    object.card = physical_card;
                    object.copy_kind = physical_kind;
                } else {
                    self.game.restore_projected_copy_identity(reference, object);
                }
                if let Some(physical) = face_down_physical {
                    object.card = physical.front_face();
                } else if !object.is_token() && !object.is_spell_copy() {
                    object.card = object.card.front_face();
                }
                object.controller = object.owner;
                object.flashback = false;
                object.cast_for_warp = false;
                object.tapped = false;
                object.summoning_sick = false;
                object.damage = 0;
                object.damaged_this_turn = false;
                object.dealt_damage_this_turn = false;
                object.dealt_damage_this_incarnation = false;
                object.dealt_combat_damage_this_incarnation = false;
                object.deathtouch_since_sba = false;
                object.growth = Stats::default();
                object.gained_keywords = Keywords::NONE;
                object.counters = InitialCounters::default();
                object.used_abilities = 0;
                object.loyalty_activated_this_turn = false;
                object.class_level = 1;
                object.attached_to = None;
                live_counters::set(&mut self.named_counters, reference, None);
            }
            if !preserve_departure_counters {
                self.resources[step.resource].attacking = false;
                if let Some(projection) = &mut self.entry_projection {
                    projection
                        .manifested
                        .retain(|(object, _)| *object != reference);
                }
                let after = self.reference(id);
                self.project_copy_departure(reference, after);
            }
        }
        // visit_applied commits source and selected additional cost components
        // in executor order, refreshing eligibility and replacement facts between them.
        if !preserve_departure_counters {
            self.refresh_control_projection();
            self.prune_source_linked_projection();
            self.refresh_enduring_projection();
        }
        (tap_bonus, tap_source)
    }
    fn project_mana_color_change(&mut self, output: &mana_output::ManaOutput, production: Mana) {
        let source = output.instance.source;
        if matches!(
            output.recipe.base(),
            ManaProduction::AnyColorChangingSource(_)
        ) && self.reference(source.card) == source
            && self.object(source.card).zone == Zone::Battlefield
        {
            let colors = production
                .iter()
                .enumerate()
                .take(5)
                .fold(0, |mask, (i, n)| mask | if *n > 0 { 1 << i } else { 0 });
            if colors != 0 {
                self.prepare_entry_projection();
                let roles = &mut self.entry_projection.as_mut().unwrap().roles;
                let timestamp = roles.next_timestamp();
                roles.animations.push(AnimationView::color(
                    source,
                    timestamp,
                    colors,
                    Some(self.game.state.turn),
                ));
            }
        }
    }
    fn add_funding_output(&mut self, step: &FundingMove, tap_bonus: Mana, production: Mana) {
        self.add_completed_mana(
            production,
            step.output.as_ref().map_or(step.quality, |o| o.quality),
            tap_bonus,
        );
        if let Some(output) = &step.output {
            self.project_mana_color_change(output, production);
        }
    }
    fn add_completed_mana(
        &mut self,
        production: Mana,
        quality: Option<mana_lots::Quality>,
        tap_bonus: Mana,
    ) {
        for (pool, produced) in self.mana.iter_mut().zip(production) {
            *pool = pool
                .checked_add(u128::from(produced))
                .capacity("mana representation exhausted");
        }
        if let Some(quality) = quality {
            mana_lots::add_lot(&mut self.lots, quality, production.map(u128::from));
        }
        for (pool, amount) in self.mana.iter_mut().zip(tap_bonus) {
            *pool = pool
                .checked_add(u128::from(amount))
                .capacity("mana representation exhausted");
        }
    }
    fn completed_mana_fits(&self, production: Mana, tap_bonus: Mana) -> bool {
        self.mana
            .into_iter()
            .zip(production)
            .zip(tap_bonus)
            .all(|((held, base), bonus)| {
                held.checked_add(u128::from(base) + u128::from(bonus))
                    .is_some_and(|total| total <= u128::from(u32::MAX))
            })
    }
    /// The ordinary fixed path adds to the already owned cost world directly.
    /// Dynamic choices keep a separate branch for every distinct concrete result.
    fn singleton_output(&self, step: &FundingMove, receipt: ManaTapBonus) -> Option<Mana> {
        let Some(output) = &step.output else {
            return Some(step.production);
        };
        if let Some(split) = output.unselected_split(self, output.source_power) {
            let (min, max) = split.bounds(self.projected_mana_split_capacity(receipt))?;
            return (min == max).then(|| split.production(min));
        }
        let mut first = None;
        let mut multiple = false;
        output.visit(self, output.source_power, |production| {
            if first.is_some_and(|one| one != production) {
                multiple = true;
                return true;
            }
            first = Some(production);
            false
        });
        if multiple { None } else { first }
    }
    fn visit_output(
        &self,
        step: &FundingMove,
        tap_bonus: ManaTapBonus,
        goal: Option<&FundingGoal>,
        emit: &mut dyn FnMut(&Self) -> bool,
    ) -> bool {
        let mut finish = |production| {
            let Some(replaced) = tap_bonus.replace_production(production) else {
                return false;
            };
            tap_bonus.visit(production, |bonus| {
                if !self.completed_mana_fits(replaced, bonus) {
                    return false;
                }
                let mut next = self.clone();
                next.add_funding_output(step, bonus, replaced);
                emit(&next)
            })
        };
        if let Some(output) = &step.output {
            if let Some(split) = output.unselected_split(self, output.source_power) {
                return self.visit_projected_mana_split(
                    split,
                    goal,
                    output.quality,
                    tap_bonus,
                    finish,
                );
            }
            output.visit(self, output.source_power, finish)
        } else {
            finish(step.production)
        }
    }
    fn projected_mana_split_capacity(&self, receipt: ManaTapBonus) -> Mana {
        let available = self
            .mana
            .map(|pool| u32::MAX - pool.min(u128::from(u32::MAX)) as u32);
        receipt.unreplaced_capacity(available)
    }

    fn visit_projected_mana_split(
        &self,
        split: crate::cards::ManaSplit,
        goal: Option<&FundingGoal>,
        quality: Option<mana_lots::Quality>,
        tap_bonus: ManaTapBonus,
        mut finish: impl FnMut(Mana) -> bool,
    ) -> bool {
        let Some((min, max)) = split.bounds(self.projected_mana_split_capacity(tap_bonus)) else {
            return false;
        };
        if max - min < 64 {
            return (min..=max).any(|amount| finish(split.production(amount)));
        }
        let preferred = goal.map_or(min, |goal| {
            let first = split.first.index();
            let available = mana_lots::available(self.mana, &self.lots, self.purpose);
            tap_bonus
                .unreplaced_required(
                    u128::from(goal.required[first]).saturating_sub(available[first]),
                )
                .clamp(u128::from(min), u128::from(max)) as u32
        });
        if finish(split.production(preferred)) {
            return true;
        }
        // The interval upper bound can add both component maxima for pruning.
        // Only exact allocations reach the actual feasibility continuation.
        let mut possible = |upper| {
            goal.is_none_or(|goal| {
                if tap_bonus.has_trigger_bonuses() {
                    return true;
                }
                let Some(replaced) = tap_bonus.replace_production(upper) else {
                    return true;
                };
                let mut optimistic = self.clone();
                optimistic.add_completed_mana(replaced, quality, [0; 6]);
                optimistic.optimistic(goal)
            })
        };
        mana_output::visit_split_range(split, min, max, &mut possible, &mut |production| {
            production[split.first.index()] != preferred && finish(production)
        })
    }
    fn key(&self, goal: &FundingGoal) -> FundingKey {
        // Attachment edges distinguish otherwise interchangeable mana sources.
        let linked = self
            .roles()
            .is_some_and(|r| !r.source_linked_effects.is_empty())
            || self.game.state.borrowed_links.is_some()
            || self
                .resources
                .iter()
                .any(|r| r.object.card.graveyard_activation_subtype().is_some())
            || self.named_counters.is_some()
            || self.counter_placement_history.is_some()
            || self
                .resources
                .iter()
                .any(|r| r.object.attached_to.is_some());
        let mut resources: Vec<_> = self
            .resources
            .iter()
            .map(|r| ResourceKey {
                definition: r.object.card.index(),
                token: r.object.is_token(),
                card: r.object.is_card(),
                kinds: self.kinds(r),
                colors: ability_world::CharacteristicWorld::colors(self, r.id),
                protected: goal
                    .source
                    .as_ref()
                    .is_some_and(|(source, _, _, _, _)| source.card == r.id),
                reserved: goal
                    .reserve
                    .is_some_and(|(reference, _)| reference.card == r.id)
                    || self
                        .sacrifice_group
                        .iter()
                        .any(|reference| reference.card == r.id)
                    || self.cost_group.as_ref().is_some_and(|(_, group)| {
                        group.iter().any(|reference| reference.card == r.id)
                    }),
                zone: r.object.zone as u8,
                tapped: r.object.tapped,
                sick: r.object.summoning_sick,
                // Ordinary basic lands remain interchangeable. Only immutable
                // guarded mana definitions need their entry history in this key;
                // borrowed/virtual-entry worlds already retain exact identities.
                entered: r.object.card.activations().iter().any(|ability| {
                    ability.guard == ActivationGuard::SourceEnteredOrControlsBasicLand
                        && ability.mana_production().is_some()
                }) && self.entered_this_turn(self.reference(r.id)),
                counters: r.object.counters,
                used: r.object.used_abilities,
                loyalty_activated_this_turn: r.object.loyalty_activated_this_turn,
                dealt_damage_this_incarnation: r.object.dealt_damage_this_incarnation,
                dealt_combat_damage_this_incarnation: r.object.dealt_combat_damage_this_incarnation,
                gained: r.object.gained_keywords,
                // Ownership still determines whose graveyard receives a sacrificed
                // resource; control determines who can use it before that move.
                owned: r.object.owner == goal.actor,
                controlled: r.object.controller == goal.actor,
                attacking: r.attacking,
                // Unattached, unmodified basic lands remain interchangeable even
                // when an unrelated Aura is present. Keep exact identities for
                // attachment participants, nonbasics and named-counter worlds.
                identity: (self.continuation_memo.as_ref().is_some_and(|memo| {
                    memo.contributors
                        || memo.exact.contains(&r.id)
                        || !r.object.card.is_basic_land()
                        || !r.object.card.activations().is_empty()
                        || self.kinds(r) != r.object.card.kind_mask()
                        || r.object.growth != Stats::default()
                        || r.object.gained_keywords != Keywords::NONE
                        || self
                            .borrowed_slots
                            .as_ref()
                            .is_some_and(|rows| rows.iter().any(|row| row.source.card == r.id))
                        || self.graveyard_borrowing.as_ref().is_some_and(|rows| {
                            rows.iter()
                                .any(|row| row.source.card == r.id || row.donor.card == r.id)
                        })
                }) || self.sacrifice_prohibition_source(r.id)
                    || zone_replacements::card_provides_prevention(r.object.card)
                    || {
                        #[cfg(feature = "test-support")]
                        {
                            zone_replacements::TEST_PREVENT_PROVIDER
                                .with(|p| p.get() == Some(self.reference(r.id)))
                        }
                        #[cfg(not(feature = "test-support"))]
                        {
                            false
                        }
                    }
                    || self.roles().is_some_and(|roles| {
                        roles
                            .source_linked_effects
                            .iter()
                            .any(|row| row.source.card == r.id || row.recipient.card == r.id)
                            || roles
                                .resolved_ability_modifications
                                .iter()
                                .any(|row| row.recipient.card == r.id)
                            || roles
                                .basic_land_types
                                .iter()
                                .any(|row| row.recipient.card == r.id)
                            // The row remains a funding identity witness after
                            // both cards take on their new incarnations.
                            || roles
                                .melded_permanents
                                .iter()
                                .any(|row| row.object.card == r.id || row.entry.companion.card == r.id)
                    })
                    || self.continuation.as_ref().is_some_and(|c| c.selects(r.id))
                    || self
                        .game
                        .plot_payment()
                        .is_some_and(|request| request.object.card == r.id)
                    || self.power_sensitive
                    || self
                        .game
                        .temporary_copy_row(self.game.object_ref(r.id))
                        .is_some()
                    || self
                        .entry_value(ObjectRef {
                            card: r.id,
                            generation: r.object.generation,
                        })
                        .is_some()
                    || (linked
                        && (self.named_counters.is_some()
                            || !r.object.card.is_basic_land()
                            || self.kinds(r) != r.object.card.kind_mask()
                            || r.object.attached_to.is_some()
                            || self.attachment_sources().iter().any(|&source| {
                                let provider =
                                    ability_world::CharacteristicWorld::object(self, source);
                                provider.zone == Zone::Battlefield
                                    && provider.attached_to
                                        == Some(ObjectRef {
                                            card: r.id,
                                            generation: r.object.generation,
                                        })
                            }))))
                .then_some((r.id, r.object.generation)),
                attached: r.object.attached_to.map(|r| (r.card, r.generation)),
            })
            .collect();
        resources.sort_by_key(|r| {
            (
                (r.definition, r.token, r.card),
                (r.kinds, r.colors),
                (r.protected, r.reserved),
                r.zone,
                (r.tapped, r.entered),
                r.sick,
                [
                    r.counters.plus_one,
                    r.counters.minus_one,
                    r.counters.stun,
                    r.counters.loyalty,
                    r.counters.lore,
                ],
                r.used,
                r.gained,
                (
                    r.attacking,
                    r.dealt_damage_this_incarnation,
                    r.dealt_combat_damage_this_incarnation,
                ),
                (r.owned, r.controlled, r.identity, r.attached),
            )
        });
        // This deliberately narrow equivalence applies only to a standalone
        // payment with no nested or projected continuation. Spell-only lots
        // ineligible for this fixed purpose cannot fund the goal or any mana
        // activation. Normalize only the search key; owned pools, payment
        // allocations and observable mana remain exact.
        let inert_spell_lots = self.continuation.is_none()
            && self.cast_costs.is_none()
            && self.entry_projection.is_none()
            && self.game.state.exile_links.is_empty()
            && goal.source.is_none()
            && goal.reserve.is_none()
            && self.cost_group.is_none()
            && self.sacrifice_group.is_empty()
            && self.required_face_up.is_none()
            && !self.convoke;
        // Spell-only lots cannot pay any mana activation. In this standalone
        // search their only consumer is the fixed final payment, which spends
        // at most the total printed goal. Extra units of one quality and color
        // cannot change reachability. This is a demand-derived equivalence for
        // the key only; all actual pools and payment witnesses remain exact.
        let final_demand =
            goal.required.into_iter().map(u128::from).sum::<u128>() + u128::from(goal.generic);
        let mut key_mana = self.mana;
        let key_lots = self
            .lots
            .iter()
            .filter_map(|lot| {
                let spell_only = matches!(
                    lot.quality.restriction,
                    mana_lots::ManaRestrictionView::CastFromExile
                        | mana_lots::ManaRestrictionView::CastFromNonhand
                        | mana_lots::ManaRestrictionView::ArtifactSpell
                        | mana_lots::ManaRestrictionView::InstantSorcery
                        | mana_lots::ManaRestrictionView::NoncreatureSpell
                        | mana_lots::ManaRestrictionView::PlaneswalkerSpell
                        | mana_lots::ManaRestrictionView::SpellSubtype(_)
                        | mana_lots::ManaRestrictionView::CreatureSpell(_)
                );
                if !spell_only {
                    return Some(lot.clone());
                }
                if inert_spell_lots && !lot.quality.eligible(self.purpose) {
                    for (pool, amount) in key_mana.iter_mut().zip(lot.amounts) {
                        *pool -= amount;
                    }
                    return None;
                }
                let Some(demand) = (if inert_spell_lots {
                    Some(final_demand)
                } else {
                    self.spell_lot_demand
                }) else {
                    return Some(lot.clone());
                };
                let mut capped = lot.clone();
                for (pool, amount) in key_mana.iter_mut().zip(&mut capped.amounts) {
                    let excess = amount.saturating_sub(demand);
                    *amount -= excess;
                    *pool -= excess;
                }
                capped
                    .amounts
                    .iter()
                    .any(|&amount| amount != 0)
                    .then_some(capped)
            })
            .collect();
        FundingKey {
            mana_window_open: self.mana_window_open(goal),
            enduring_story: self.enduring_story,
            exhaust_activated: self.exhaust_activated,
            permanent_left_battlefield: self.permanent_left_battlefield,
            void_active_this_turn: self.void_active_this_turn,
            payment_departures: self.payment_departures.clone(),
            resolved_ability_modifications: self
                .roles()
                .map_or_else(Vec::new, |r| r.resolved_ability_modifications.clone()),
            source_linked_effects: self
                .roles()
                .map_or_else(Vec::new, |r| r.source_linked_effects.clone()),
            histories: self.power_sensitive.then_some((
                self.creatures_died,
                self.graveyard_left,
                self.other_life,
                self.life_gained,
            )),
            controlled_creature_deaths: self.controlled_creatures_died,
            entries: self.entry_projection.as_ref().map(|p| EntryFundingKey {
                copy_origins: p.copy_origins.clone(),
                manifested: p.manifested.clone(),
                order: p.order.clone(),
                entered: p.entered.clone(),
                lands_entered: p.lands_entered,
                battle_protectors: p
                    .battle_protectors
                    .iter()
                    .map(|(object, player)| (*object, player.index()))
                    .collect(),
                values: p.entry_values.iter().map(|r| (r.object, r.value)).collect(),
                riot_haste: p
                    .roles
                    .keyword_effects
                    .iter()
                    .filter_map(|row| {
                        (row.origin == KeywordEffectOrigin::Riot
                            && self.reference(row.recipient.card) == row.recipient)
                            .then_some((row.recipient, row.timestamp))
                    })
                    .collect(),
                consumed: p.consumed.clone(),
            }),
            self_grant_uses: self.self_grant_uses.clone(),
            borrowed_uses: {
                let mut uses = self.borrowed_uses.clone();
                uses.sort_by_key(|(source, slot)| (source.card, source.generation, *slot));
                uses.dedup();
                uses
            },
            basic_land_types: self
                .roles()
                .map_or_else(Vec::new, |roles| roles.basic_land_types.clone()),
            borrowed_slots: self
                .borrowed_slots
                .as_ref()
                .map_or_else(Vec::new, |rows| rows.to_vec()),
            graveyard_borrowing: self
                .graveyard_borrowing
                .as_ref()
                .map_or_else(Vec::new, |rows| rows.to_vec()),
            mana: key_mana,
            lots: key_lots,
            life: self.life,
            payment_libraries: self.life_replacement_possible.then(|| {
                [Player::P0, Player::P1].map(|player| {
                    self.game.state.players[player.index()]
                        .library
                        .iter()
                        .copied()
                        .filter(|&id| self.object(id).zone == Zone::Library)
                        .map(|id| self.reference(id))
                        .collect()
                })
            }),
            life_lost: self.life_lost,
            resources,
            named_counters: self.named_counters.clone(),
            counter_placement_history: self.counter_placement_history.clone(),
            keyword_counter_timestamps: {
                let mut rows = self.roles().map_or_else(Vec::new, |state| {
                    state
                        .keyword_effects
                        .iter()
                        .filter_map(|row| {
                            let KeywordEffectOrigin::Counter { kind } = row.origin else {
                                return None;
                            };
                            (self.reference(row.recipient.card) == row.recipient
                                && self.counter_count(row.recipient.card, kind) > 0)
                                .then_some((row.recipient, kind, row.timestamp))
                        })
                        .collect::<Vec<_>>()
                });
                rows.sort_by_key(|(object, kind, _)| (object.card, object.generation, *kind));
                rows
            },
        }
    }
    fn attached_to_opposing_permanent(&self, id: CardId) -> bool {
        self.object(id).attached_to.is_some_and(|recipient| {
            self.reference(recipient.card) == recipient
                && self.object(recipient.card).zone == Zone::Battlefield
                && self.object(recipient.card).controller != self.actor
        })
    }
    fn land_tap_effects_upper(&self, fixed_control: bool) -> ([u128; 6], u128) {
        use crate::cards::StaticAbilityFragment::*;
        let mut bonus = [0u128; 6];
        let mut multiplicity = 1u128;
        let mut multiplier = 1u128;
        for &id in self.static_sources() {
            if self.object(id).zone != Zone::Battlefield {
                continue;
            }
            for fragment in self
                .definition(id)
                .static_ability_groups()
                .iter()
                .flat_map(|g| g.fragments)
            {
                match *fragment {
                    ControlledBasicLandManaMultiplier(factor)
                        if !fixed_control || self.object(id).controller == self.actor =>
                    {
                        multiplier = multiplier.saturating_mul(u128::from(factor.max(1)));
                    }
                    ControlledLandTapManaBonus(mana) => {
                        for (total, amount) in bonus.iter_mut().zip(mana) {
                            *total = total.saturating_add(u128::from(amount));
                        }
                    }
                    ControlledLandColorlessTapManaBonus => {
                        // Conservatively allow any tap to make the required {C}.
                        bonus[5] = bonus[5].saturating_add(1);
                    }
                    AttachmentManaBonus(production) => {
                        // A fixed-control proof cannot tap an opposing land,
                        // even if the Aura itself belongs to the actor.
                        if fixed_control && self.attached_to_opposing_permanent(id) {
                            continue;
                        }
                        self.production_upper(production).visit(|mana| {
                            for (total, amount) in bonus.iter_mut().zip(mana) {
                                *total = total.saturating_add(u128::from(amount));
                            }
                            false
                        });
                    }
                    BasicLandTapManaBonus(amount) => {
                        for total in &mut bonus {
                            *total = total.saturating_add(u128::from(amount));
                        }
                    }
                    AdditionalTriggers { .. } => {
                        multiplicity = multiplicity.saturating_add(1);
                    }
                    _ => {}
                }
            }
        }
        bonus = bonus.map(|amount| amount.saturating_mul(multiplicity));
        bonus[3] = bonus[3].saturating_add(u128::from(
            self.game.state.players[self.actor.index()].mountain_mana_bonus,
        ));
        (bonus, multiplier)
    }

    /// For closed funding paths, bound each source's alternative tap outputs once.
    /// Keep non-tap net gains and conservative grants additive before colors.
    /// Ignore guards, sickness, reservations and shared costs here, since those
    /// restrictions can only lower the amount attainable. Convoke is similarly
    /// overcounted alongside any mana the same creature might produce.
    fn total_mana_upper(&self) -> u128 {
        let mut total = self.mana.into_iter().fold(0u128, u128::saturating_add);
        // Bound each tap as though it could acquire basic-land characteristics
        // during funding. The exact event still checks the land and providers.
        let (tap_bonus, tap_factor) = self.land_tap_effects_upper(false);
        let tap_bonus = tap_bonus.into_iter().fold(0u128, u128::saturating_add);
        let scale_tap = |amount: u128| amount.saturating_mul(tap_factor);
        for resource in &self.resources {
            let object = &resource.object;
            if object.zone != Zone::Battlefield || object.controller != self.actor {
                continue;
            }
            // Intrinsic and printed tap abilities compete for this exact
            // source's single tap. Other grants below remain overestimates.
            let mut tap_upper = 0;
            // Source-linked grants are independent of the exiled source's
            // current abilities. Count every retained row, even if presently
            // blanked, so this bound cannot prune a later restored grant.
            if !object.tapped {
                let grants = self
                    .roles()
                    .into_iter()
                    .flat_map(|roles| &roles.source_linked_effects)
                    .filter(|row| {
                        row.kind == SourceLinkedEffectKind::GrantTwoColorlessUntilSourceCast
                            && row.recipient == self.reference(resource.id)
                    })
                    .count();
                total = total.saturating_add(scale_tap((grants as u128).saturating_mul(2)));
                let blue = self
                    .roles()
                    .into_iter()
                    .flat_map(|roles| &roles.source_linked_effects)
                    .filter(|row| {
                        row.kind == SourceLinkedEffectKind::LandUntilControllerNextTurn
                            && row.recipient == self.reference(resource.id)
                    })
                    .count();
                total = total.saturating_add(scale_tap(blue as u128));
            }
            if !object.tapped {
                // Overcount all possible tap sources; subtype may change during funding.
                total = total.saturating_add(tap_bonus);
                tap_upper =
                    u128::from(object.card.mana_color().is_some() || self.kinds(resource) & 2 != 0);
                // A chosen basic type can supply one mana even if ability removal
                // currently hides it; later funding moves may restore the grant.
                if object.card.static_ability_groups().iter().any(|group| {
                    group.fragments.iter().any(|fragment| {
                        matches!(
                            fragment,
                            crate::cards::StaticAbilityFragment::ChosenBasicLandType
                        )
                    })
                }) {
                    total = total.saturating_add(tap_factor);
                }
                if (self.waterbend != 0 && self.kinds(resource) & (1 | 16) != 0)
                    || (self.convoke && self.kinds(resource) & 1 != 0)
                    || matches!(self.purpose, mana_lots::Purpose::Spell(spell) if improvise::potential(self, spell.card))
                {
                    total = total.saturating_add(1);
                }
            }
            if !object.tapped {
                for &source in self.attachment_sources() {
                    let provider = ability_world::CharacteristicWorld::object(self, source);
                    if provider.zone == Zone::Battlefield
                        && provider.attached_to
                            == Some(ObjectRef {
                                card: resource.id,
                                generation: object.generation,
                            })
                    {
                        for group in provider.card.static_ability_groups() {
                            for fragment in group.fragments {
                                if let crate::cards::StaticAbilityFragment::AttachmentMana(
                                    production,
                                ) | crate::cards::StaticAbilityFragment::AttachmentSacrificeMana(production) = fragment
                                {
                                    let mut maximum = 0;
                                    self.production_upper(*production).visit(|mana| {
                                        maximum =
                                            maximum.max(mana.into_iter().map(u128::from).sum());
                                        false
                                    });
                                    total = total.saturating_add(scale_tap(maximum));
                                }
                            }
                        }
                    }
                }
            }
            if !object.tapped {
                for production in self.potential_static_tap_mana() {
                    let mut maximum = 0;
                    self.production_upper(production).visit(|mana| {
                        maximum = maximum.max(mana.into_iter().map(u128::from).sum());
                        false
                    });
                    total = total.saturating_add(scale_tap(maximum));
                }
            }
            for (index, ability, borrowed, self_grant) in self.mana_activations(resource.id) {
                // Tapped objects cannot pay tap costs in this closed search.
                if (!object.tapped || !ability.cost.tap) && ability.mana_production().is_some() {
                    let production = self.production_upper(ability.mana_production().unwrap());
                    if ability.cost.tap {
                        // Ignore input mana costs: another payment may restore
                        // a discount before this ability is activated.
                        production.visit(|mana| {
                            tap_upper = tap_upper.max(mana.into_iter().map(u128::from).sum());
                            false
                        });
                        continue;
                    }
                    let definition = self_grant.map_or_else(
                        || borrowed.map_or(object.card, |a| a.definition),
                        |grant| grant.0,
                    );
                    // Earlier mana payments can create or restore a live reduction.
                    // The upper bound must allow its largest possible generic discount.
                    let discount = if definition
                        .activation_generic_reductions(index)
                        .next()
                        .is_some()
                    {
                        u32::MAX
                    } else {
                        exhaust::generic_discount(self, resource.id, definition, index)
                    };
                    total = total.saturating_add(activation_gain_upper(
                        ability,
                        production,
                        ability_world::CharacteristicWorld::counter_amounts(self, resource.id),
                        if self.life_replacement_possible {
                            i128::MAX
                        } else {
                            self.life
                        },
                        discount,
                    ));
                }
            }
            total = total.saturating_add(scale_tap(tap_upper));
        }
        total
    }
    // A removal provider may depart during funding and expose hidden printed
    // abilities. Count all possible production, ignoring guards and sickness;
    // each restriction we omit only raises this upper bound. No funding move
    // untaps a source, creates a permanent, restores counters or gains life.
    fn possible_color_upper(&self) -> [u128; 6] {
        let mut upper = self.mana;
        let (tap_bonus, tap_factor) = self.land_tap_effects_upper(false);
        for resource in &self.resources {
            let object = &resource.object;
            if object.zone != Zone::Battlefield || object.controller != self.actor {
                continue;
            }
            if !object.tapped {
                let grants = self
                    .roles()
                    .into_iter()
                    .flat_map(|roles| &roles.source_linked_effects)
                    .filter(|row| {
                        row.kind == SourceLinkedEffectKind::GrantTwoColorlessUntilSourceCast
                            && row.recipient == self.reference(resource.id)
                    })
                    .count();
                upper[5] = upper[5].saturating_add(
                    (grants as u128)
                        .saturating_mul(2)
                        .saturating_mul(tap_factor),
                );
                let blue = self
                    .roles()
                    .into_iter()
                    .flat_map(|roles| &roles.source_linked_effects)
                    .filter(|row| {
                        row.kind == SourceLinkedEffectKind::LandUntilControllerNextTurn
                            && row.recipient == self.reference(resource.id)
                    })
                    .count();
                upper[1] = upper[1].saturating_add((blue as u128).saturating_mul(tap_factor));
            }
            if !object.tapped {
                for (total, amount) in upper.iter_mut().zip(tap_bonus) {
                    *total = total.saturating_add(amount);
                }
            }
            let mut add = |production: ManaProduction, uses: u128| {
                let production = self.production_upper(production);
                production.visit(|mana| {
                    for i in 0..6 {
                        upper[i] =
                            upper[i].saturating_add(uses.saturating_mul(u128::from(mana[i])));
                    }
                    false
                });
            };
            if !object.tapped {
                if let Some(color) = object.card.mana_color() {
                    let mut mana = [0; 6];
                    mana[color] = 1;
                    add(ManaProduction::Fixed(mana), tap_factor);
                }
                // A land's subtype can change during funding. Overestimate all
                // five colors rather than pruning a reachable future grant.
                if self.kinds(resource) & 2 != 0 {
                    add(
                        ManaProduction::Choice {
                            mask: 31,
                            amount: 1,
                        },
                        tap_factor,
                    );
                }
                if object.card.static_ability_groups().iter().any(|group| {
                    group.fragments.iter().any(|fragment| {
                        matches!(
                            fragment,
                            crate::cards::StaticAbilityFragment::ChosenBasicLandType
                        )
                    })
                }) {
                    // Conservative upper bound: no legal branch can produce
                    // more than one, but its actual chosen type may be projected.
                    add(
                        ManaProduction::Choice {
                            mask: 31,
                            amount: 1,
                        },
                        tap_factor,
                    );
                }
                if self.convoke {
                    // A currently noncreature resource may regain creature type.
                    add(
                        ManaProduction::Choice {
                            mask: 31,
                            amount: 1,
                        },
                        1,
                    );
                }
                for &source in self.attachment_sources() {
                    let provider = ability_world::CharacteristicWorld::object(self, source);
                    if provider.zone != Zone::Battlefield
                        || provider.attached_to
                            != Some(ObjectRef {
                                card: resource.id,
                                generation: object.generation,
                            })
                    {
                        continue;
                    }
                    for group in provider.card.static_ability_groups() {
                        for fragment in group.fragments {
                            if let crate::cards::StaticAbilityFragment::AttachmentMana(production)
                            | crate::cards::StaticAbilityFragment::AttachmentSacrificeMana(
                                production,
                            ) = *fragment
                            {
                                add(production, tap_factor);
                            }
                        }
                    }
                }
            }
            if !object.tapped {
                for production in self.potential_static_tap_mana() {
                    add(production, tap_factor);
                }
            }
            for (_, ability, _, _) in self.mana_activations(resource.id) {
                let Some(production) = ability.mana_production() else {
                    continue;
                };
                let cost = ability.cost;
                if object.tapped && cost.tap {
                    continue;
                }
                let mut uses = if cost.tap || ability.once_per_turn || ability.exhaust {
                    1
                } else {
                    u128::MAX
                };
                if cost.life > 0 && !self.life_replacement_possible {
                    uses = uses.min(self.life.max(0) as u128 / u128::from(cost.life));
                }
                if let Some((kind, amount)) = cost.remove
                    && amount > 0
                {
                    uses = uses.min(
                        ability_world::CharacteristicWorld::counter_amounts(self, resource.id)
                            .available(kind)
                            / u128::from(amount),
                    );
                }
                add(
                    production,
                    if cost.tap {
                        uses.saturating_mul(tap_factor)
                    } else {
                        uses
                    },
                );
            }
        }
        upper
    }
    /// Plain materialized copies may remain fixed during a tap-only search.
    /// Any temporary layer must be bound to its own exact live incarnation.
    fn fixed_copy_recipes(&self) -> bool {
        let plain_live_copy = |reference: ObjectRef, recipe: CopyRecipe| {
            self.reference(reference.card) == reference
                && self.object(reference.card).zone == Zone::Battlefield
                && recipe == CopyRecipe::new(self.object(reference.card).card)
        };
        self.copy_recipes.as_ref().is_none_or(|rows| {
            rows.iter()
                .all(|row| plain_live_copy(row.object, row.recipe))
        }) && self
            .temporary_copy_rows_state()
            .as_ref()
            .is_none_or(|rows| {
                rows.iter().all(|row| {
                    row.layers.iter().all(|layer| {
                        layer.value.deadline.source() == Some(row.object)
                            && plain_live_copy(row.object, layer.value.recipe)
                    })
                })
            })
    }
    /// In a fixed world without a mana converter, only final-purpose-eligible
    /// production can fund this payment. Bound each source's alternative tap
    /// outputs once for quantity, and independently for each required color.
    /// Dynamic sources, grants, entries and characteristic changes fall back to
    /// exact search. These bounds never remove mana lots or payment choices.
    fn fixed_mana_cannot_pay(&self, goal: &FundingGoal) -> bool {
        use crate::cards::{ManaRestriction, StaticAbilityFragment};
        let purpose = self.payment_purpose(goal);
        let convoke = self.convoke_available();
        let needs_web_return = self
            .cast_costs
            .as_deref()
            .is_some_and(CostContext::needs_web_return);
        if !matches!(
            purpose,
            mana_lots::Purpose::Activation(_) | mana_lots::Purpose::Spell(_)
        ) || goal.reserve.is_some()
            || self.continuation.is_some()
            || !self.only_mana_reveals_and_web_return(goal)
            || self.entry_projection.is_some()
            || self.cost_group.is_some()
            || !self.sacrifice_group.is_empty()
            || self.required_face_up.is_some()
            || self.improvise_available()
            || self.delve_available()
            || self.waterbend != 0
            || (convoke && self.cast_costs.is_some())
            || !self.game.state.exile_links.is_empty()
            || self.game.state.borrowed_links.is_some()
            || self.game.state.graveyard_borrowing.is_some()
            || self.game.state.borrowed_slots.is_some()
            || self.game.state.activation_observers.is_some()
            || self.game.state.temporary_abilities.is_some()
            || self.game.state.self_granted_abilities.is_some()
            || self.game.state.temporary_controls.is_some()
            || self.game.state.controls.is_some()
            || !self.fixed_copy_recipes()
            || self.game.state.crew.is_some()
            || self.resources.iter().any(|resource| {
                resource
                    .object
                    .card
                    .graveyard_activation_subtype()
                    .is_some()
            })
            || !self.game.state.tap_sources.is_empty()
            || self.land_tap_effects_upper(true) != ([0; 6], 1)
            || matches!(purpose, mana_lots::Purpose::Spell(spell)
                if spell.spend_any_mana || spell.linked_mana_source.is_some())
            // Resolved ability removal follows a fixed incarnation. The
            // tap-only routes below cannot end it or create a new recipient.
            || self.roles().is_some_and(|r| {
                !r.source_linked_effects.is_empty()
                    || !r.ability_removal_sources.is_empty()
                    || !r.basic_land_types.is_empty()
                    || !r.creature_types.is_empty()
                    || !r.animations.is_empty()
                    || !r.live_static.is_empty()
            })
        {
            return false;
        }
        let mut upper = mana_lots::available(self.mana, &self.lots, purpose);
        let mut quantity = upper.into_iter().fold(0u128, u128::saturating_add);
        // Plain creatures need not be mana resources. Count every exact owned
        // untapped body, including sick creatures and the source itself: other
        // filters and reservations can only shrink this generous witness pool.
        let count_untapped_creatures = || {
            self.game
                .state
                .objects
                .iter()
                .enumerate()
                .filter(|(id, _)| {
                    let id = *id as CardId;
                    let object = self.object(id);
                    object.zone == Zone::Battlefield
                        && object.controller == goal.actor
                        && !object.tapped
                        && ability_world::CharacteristicWorld::kinds(self, id) & 1 != 0
                })
                .count()
        };
        let mut creature_tap_upper = None;
        let mut group_minimum = usize::MAX;
        let mut group_colors = [0u128; 6];
        let mut group_quantity = 0u128;
        for (id, _) in self.game.state.objects.iter().enumerate() {
            let id = id as CardId;
            let object = self.object(id);
            if object.zone == Zone::Command {
                return false;
            }
            if object.zone != Zone::Battlefield {
                continue;
            }
            let web_creature = needs_web_return
                && object.controller == goal.actor
                && ability_world::CharacteristicWorld::kinds(self, id) & 1 != 0;
            if web_creature && object.tapped {
                return false;
            }
            if object.card.borrows_activations()
                || object.card.static_ability_groups().iter().any(|group| {
                    group.fragments.iter().any(|fragment| {
                        match fragment {
                            // These groups cannot add mana routes. The
                            // admitted production recipes do not read stats,
                            // and no land play or phase change can occur here.
                            StaticAbilityFragment::IntrinsicKeywords(_)
                            | StaticAbilityFragment::Changeling
                            | StaticAbilityFragment::AttachmentStats(_)
                            | StaticAbilityFragment::AttachmentTrigger(_)
                            | StaticAbilityFragment::OpponentEntryTapped(..)
                            | StaticAbilityFragment::ColoredLegendaryWebSlinging(_)
                            | StaticAbilityFragment::AttachedChosenColor
                            | StaticAbilityFragment::AttachedAllCreatureTypes
                            | StaticAbilityFragment::AdditionalLandPlays(_)
                            | StaticAbilityFragment::StaticModifier(..)
                            | StaticAbilityFragment::ScalingModifier(..)
                            | StaticAbilityFragment::ControlledCreatureScaling { .. }
                            | StaticAbilityFragment::ControlledSubtypeCannotBeBlockedByKeyword {
                                ..
                            }
                            | StaticAbilityFragment::ChosenBasicLandType => false,
                            StaticAbilityFragment::AttachmentManaBonus(_) => {
                                !self.attached_to_opposing_permanent(id)
                            }
                            StaticAbilityFragment::LandMana {
                                production: ManaProduction::Tagged { quality, .. },
                                ..
                            }
                            | StaticAbilityFragment::ControlledPermanentTapMana {
                                production: ManaProduction::Tagged { quality, .. },
                                ..
                            }
                            | StaticAbilityFragment::ControlledSubtypeTapMana {
                                production: ManaProduction::Tagged { quality, .. },
                                ..
                            }
                            | StaticAbilityFragment::OtherControlledPermanentTapMana(
                                ManaProduction::Tagged { quality, .. },
                            ) => {
                                needs_web_return
                                    || !matches!(
                                        quality.restriction,
                                        ManaRestriction::CastFromExile
                                            | ManaRestriction::CastFromNonhand
                                            | ManaRestriction::ArtifactSpell
                                            | ManaRestriction::InstantSorcery
                                            | ManaRestriction::NoncreatureSpell
                                            | ManaRestriction::PlaneswalkerSpell
                                            | ManaRestriction::SpellSubtype(_)
                                            | ManaRestriction::ChosenCreatureSpell
                                    )
                                    || mana_lots::Quality::capture(self, id, **quality)
                                        .eligible(purpose)
                            }
                            _ => true,
                        }
                    })
                })
            {
                return false;
            }
            let mut tap_colors = [0u128; 6];
            let mut tap_quantity = 0;
            if object.controller == goal.actor && !object.tapped {
                if let Some(color) =
                    ability_world::CharacteristicWorld::intrinsic_mana_color(self, id)
                {
                    tap_colors[color] = 1;
                    tap_quantity = 1;
                }
                for grant in ability_world::CharacteristicWorld::granted_mana(self, id) {
                    if grant.subtype_derived {
                        grant.production.visit(|mana| {
                            for (amount, produced) in tap_colors.iter_mut().zip(mana) {
                                *amount = (*amount).max(u128::from(produced));
                            }
                            tap_quantity = tap_quantity.max(mana.into_iter().map(u128::from).sum());
                            false
                        });
                    }
                }
            }
            if web_creature && tap_quantity > 0 {
                return false;
            }
            for (index, ability) in object.card.activations().iter().enumerate() {
                let Some(production) = ability.mana_production() else {
                    continue;
                };
                let cost = ability.cost;
                let group_taps = cost.additional.and_then(|spec| {
                    let EffectChoiceKind::TapPermanents(amount) = spec.kind.payment_kind() else {
                        return None;
                    };
                    (amount != 0 && spec.filter.kinds == 1).then_some(amount as usize)
                });
                if !matches!(ability.program, [Instruction::AddMana(_)])
                    || object.card.activation_put_counter(index as u16).is_some()
                    || ability.source_zone() != Zone::Battlefield
                    || (!cost.tap && group_taps.is_none())
                    || cost.sacrifice
                    || cost.exile_self
                    || cost.discard
                    || cost.life != 0
                    || cost.remove.is_some()
                    || cost.loyalty.is_some()
                    || cost.additional.is_some_and(|spec| {
                        !matches!(spec.kind.payment_kind(), EffectChoiceKind::TapPermanents(_))
                    })
                    || cost.mana_options().any(|mana| {
                        mana.generic != 0 || mana.colored.into_iter().any(|amount| amount != 0)
                    })
                {
                    return false;
                }
                // Neither a tap nor a reveal changes the colors or
                // membership of controlled permanents in this fixed world.
                let recipe = production.base();
                let fixed = if recipe == ManaProduction::ColorsAmongControlledPermanents {
                    recipe.in_world(self, id)
                } else {
                    recipe
                };
                // Compute maxima directly; do not enumerate a combinatorial
                // production domain just to obtain a conservative upper bound.
                let (colors, amount) = match fixed {
                    ManaProduction::Fixed(mana) => (
                        mana.map(u128::from),
                        mana.into_iter().map(u128::from).sum::<u128>(),
                    ),
                    ManaProduction::Choice { mask, amount }
                    | ManaProduction::Combination { mask, amount } => (
                        std::array::from_fn(|color| {
                            if mask & (1 << color) != 0 {
                                u128::from(amount)
                            } else {
                                0
                            }
                        }),
                        if mask & 63 != 0 {
                            u128::from(amount)
                        } else {
                            0
                        },
                    ),
                    _ => return false,
                };
                // The route audit above uses all printed templates. In
                // this fixed world a removed printed ability cannot return.
                if object.controller != goal.actor
                    || (cost.tap && object.tapped)
                    || !ability_world::CharacteristicWorld::printed_abilities_available(self, id)
                {
                    continue;
                }
                if needs_web_return && (web_creature || cost.additional.is_some()) {
                    // Even mana ineligible for this spell can tap a creature.
                    return false;
                }
                if let ManaProduction::Tagged { quality, .. } = production
                    && !mana_lots::Quality::capture(self, id, *quality).eligible(purpose)
                {
                    continue;
                }
                if let Some(group_taps) = group_taps {
                    if *creature_tap_upper.get_or_insert_with(count_untapped_creatures) < group_taps
                    {
                        continue;
                    }
                    if !cost.tap {
                        // Copies and different abilities draw from one tap pool.
                        group_minimum = group_minimum.min(group_taps);
                        group_quantity = group_quantity.max(amount);
                        for (bound, amount) in group_colors.iter_mut().zip(colors) {
                            *bound = (*bound).max(amount);
                        }
                        continue;
                    }
                }
                // Every admitted mana cost taps without a departure or
                // conversion. It can neither re-enter nor untap a source.
                for (bound, amount) in tap_colors.iter_mut().zip(colors) {
                    *bound = (*bound).max(amount);
                }
                tap_quantity = tap_quantity.max(amount);
            }
            for (bound, amount) in upper.iter_mut().zip(tap_colors) {
                *bound = bound.saturating_add(amount);
            }
            quantity = quantity.saturating_add(tap_quantity);
        }
        // These fixed routes can tap only noncreatures, and the pending
        // return cannot itself make its missing creature tapped.
        if needs_web_return {
            return true;
        }
        let mut group_gain = group_quantity;
        let creatures = if convoke || group_minimum != usize::MAX {
            *creature_tap_upper.get_or_insert_with(count_untapped_creatures)
        } else {
            0
        };
        let group_uses = (creatures / group_minimum) as u128;
        let convoke_quantity = if convoke {
            // Each grouped activation consumes creatures that could otherwise
            // contribute one each. Only a gain above that can increase quantity.
            group_gain = group_gain.saturating_sub(group_minimum as u128);
            creatures as u128
        } else {
            0
        };
        quantity = quantity
            .saturating_add(convoke_quantity)
            .saturating_add(group_uses.saturating_mul(group_gain));
        for (bound, amount) in upper.iter_mut().zip(group_colors) {
            *bound = bound
                .saturating_add(convoke_quantity)
                .saturating_add(group_uses.saturating_mul(amount));
        }
        let adjusted = cast_origins::payment_goal(self, goal, purpose);
        let required = u128::from(adjusted.generic)
            + adjusted.required.into_iter().map(u128::from).sum::<u128>();
        quantity < required || !payable(upper, adjusted.required, adjusted.generic)
    }
    /// A flexible producer can contribute to several individual color bounds,
    /// but it cannot produce its full amount for all those colors at once.
    /// Bound every subset of requested colors in a finite printed-mana world.
    fn joint_colors_possible(&self, goal: &FundingGoal) -> bool {
        let required_mask = goal
            .required
            .iter()
            .enumerate()
            .fold(0u8, |mask, (color, &amount)| {
                mask | (u8::from(amount != 0) << color)
            });
        if required_mask.count_ones() < 2 {
            return true;
        }
        let state = &self.game.state;
        if self.has_contributions()
            || self.cast_costs.is_some()
            || self.required_face_up.is_some()
            || self.entry_projection.is_some()
            || self.continuation.is_some()
            || self.controls().is_some()
            || state.temporary_controls.is_some()
            || self.temporary_copy_rows_state().is_some()
            || self.copy_recipes.is_some()
            || state.borrowed_links.is_some()
            || state.borrowed_slots.is_some()
            || state.graveyard_borrowing.is_some()
            || state.self_granted_abilities.is_some()
            || !state.tap_sources.is_empty()
            || self.land_tap_effects_upper(false) != ([0; 6], 1)
            || matches!(self.purpose, mana_lots::Purpose::Spell(spell) if spell.spend_any_mana || spell.linked_mana_source.is_some())
            || self.roles().is_some_and(|roles| {
                !roles.source_linked_effects.is_empty() || !roles.basic_land_types.is_empty()
            })
            || self
                .static_sources()
                .iter()
                .chain(self.attachment_sources())
                .any(|&id| {
                    let card = self.object(id).card;
                    (matches!(self.purpose, mana_lots::Purpose::Spell(_))
                        && card.has_cast_origin_abilities())
                        || card
                            .static_ability_groups()
                            .iter()
                            .flat_map(|g| g.fragments)
                            .any(|fragment| {
                                matches!(
                                    fragment,
                                    StaticAbilityFragment::ChosenBasicLandType
                                        | StaticAbilityFragment::AttachmentMana(_)
                                        | StaticAbilityFragment::AttachmentSacrificeMana(_)
                                        | StaticAbilityFragment::LandMana { .. }
                                        | StaticAbilityFragment::ControlledPermanentTapMana { .. }
                                        | StaticAbilityFragment::ControlledSubtypeTapMana { .. }
                                        | StaticAbilityFragment::OtherControlledPermanentTapMana(_)
                                        | StaticAbilityFragment::SpendAnyColorForCreatureActivations
                                )
                            })
                })
            || state.exile_links.iter().any(|link| {
                let card = self.object(link.exiled.card).card;
                // An immediate linked return can put a new producer, copy, or
                // mana provider onto the battlefield during the search.
                card.is_land()
                    || card.mana_color().is_some()
                    || card.entry_copy().is_some()
                    || !card.static_ability_groups().is_empty()
                    || card
                        .activations()
                        .iter()
                        .any(|a| a.mana_production().is_some())
            })
        {
            return true;
        }

        let mut fixed = self.mana;
        let mut flexible = [0u128; 64];
        for resource in &self.resources {
            let object = &resource.object;
            if object.zone != Zone::Battlefield || object.controller != self.actor {
                continue;
            }
            if !object.tapped {
                for color in object
                    .card
                    .printed_intrinsic_mana_colors()
                    .chain(object.card.mana_color())
                {
                    fixed[color] = fixed[color].saturating_add(1);
                }
            }
            for ability in object.card.activations() {
                let Some(production) = ability.mana_production() else {
                    continue;
                };
                let cost = ability.cost;
                if ability.source_zone() != Zone::Battlefield {
                    return true;
                }
                if cost.tap && object.tapped {
                    continue;
                }
                if !(cost.tap || cost.sacrifice || cost.exile_self) {
                    return true;
                }
                // Count every printed option separately and ignore costs, guards,
                // and mana restrictions. Each of these only increases the bound.
                match production.base() {
                    ManaProduction::Fixed(mana) => {
                        for (total, amount) in fixed.iter_mut().zip(mana) {
                            *total = total.saturating_add(u128::from(amount));
                        }
                    }
                    ManaProduction::Choice { mask, amount }
                    | ManaProduction::Combination { mask, amount } => {
                        let total = &mut flexible[usize::from(mask & required_mask)];
                        *total = total.saturating_add(u128::from(amount));
                    }
                    _ => return true,
                }
            }
        }
        let mut subset = required_mask;
        while subset != 0 {
            let mut demand = 0u128;
            let mut capacity = 0u128;
            for (color, &amount) in fixed.iter().enumerate() {
                if subset & (1 << color) != 0 {
                    demand += u128::from(goal.required[color]);
                    capacity = capacity.saturating_add(amount);
                }
            }
            for (mask, &amount) in flexible.iter().enumerate() {
                if usize::from(subset) & mask != 0 {
                    capacity = capacity.saturating_add(amount);
                }
            }
            if capacity < demand {
                return false;
            }
            subset = (subset - 1) & required_mask;
        }
        true
    }

    fn optimistic(&self, goal: &FundingGoal) -> bool {
        // A later mana sacrifice may both add graveyard payment resources and
        // change whether delve is granted. Battlefield mana alone is no bound.
        if matches!(self.purpose, mana_lots::Purpose::Spell(_)) && delve::potential(self) {
            return true;
        }
        if self.resources.iter().any(|r| {
            r.object.zone != Zone::Battlefield && self.mana_activations(r.id).next().is_some()
        }) {
            // The legacy battlefield-only bound cannot rule out source-zone
            // mana. Exact moves/visited states still determine reachability.
            return true;
        }
        if self.fixed_mana_cannot_pay(goal) {
            return false;
        }
        // A departure may reveal off-battlefield abilities on either front or
        // meet a zone-count guard. The current inventory cannot bound them.
        if self.active_melds().next().is_some() {
            return true;
        }
        // A virtual sacrifice can supply a previously absent graveyard mana
        // ability. The ordinary current-inventory upper bound is not conservative
        // for this opt-in rule; exact owned reachability still decides legality.
        if self
            .resources
            .iter()
            .any(|r| r.object.card.graveyard_activation_subtype().is_some())
        {
            return true;
        }
        // A funding departure may end a source-bound or attached control
        // effect and make previously opposing mana sources available. The
        // current-controller inventory cannot bound that future world.
        // ControlBase and source-independent ControlEffect rows are stable
        // for their object's incarnation throughout a mana payment. They do
        // not by themselves cause an opposing source to change controllers.
        if self.roles().is_some_and(|roles| {
            roles.source_linked_effects.iter().any(|row| {
                matches!(
                    row.kind,
                    SourceLinkedEffectKind::Control
                        | SourceLinkedEffectKind::ControlAndCombatLockWhileControlled
                )
            })
        }) || self
            .attachment_sources()
            .iter()
            .any(|&id| self.object(id).card.controls_attached())
        {
            return true;
        }
        // Sacrificing a removal provider or a source-bound copy's source can
        // restore a producer or a discount absent from the current inventory.
        // Inspect projected rows, including hidden source-bound copy layers.
        if ability_world::CharacteristicWorld::has_ability_removal(self)
            || self
                .temporary_copy_rows_state()
                .as_ref()
                .is_some_and(|rows| {
                    rows.iter().any(|row| {
                        self.object(row.object.card).zone == Zone::Battlefield
                            && self.reference(row.object.card) == row.object
                            && row
                                .layers
                                .iter()
                                .any(|layer| layer.value.deadline.source().is_some())
                    })
                })
        {
            return true;
        }
        if !self.joint_colors_possible(goal) {
            return false;
        }
        // Linked returns and external activation grants can change the mana
        // sources reachable in this payment. Use the exact projected moves.
        if !self.game.state.exile_links.is_empty()
            || borrowed_activations::has_shared_links(self.game)
        {
            return true;
        }
        let required =
            u128::from(goal.generic) + goal.required.into_iter().map(u128::from).sum::<u128>();
        if self.total_mana_upper() < required {
            return false;
        }
        // Substitution may be lost or restored by virtual sacrifices. Keep the
        // color bound conservative; the exact leaf uses the overlay.
        if let mana_lots::Purpose::Spell(spell) = self.purpose
            && (spell.spend_any_mana
                || spell.linked_mana_source.is_some()
                || self.static_sources().iter().any(|&id| {
                    self.game.state.objects[id as usize]
                        .card
                        .has_cast_origin_abilities()
                }))
        {
            return true;
        }
        if matches!(self.purpose, mana_lots::Purpose::Activation(_))
            && goal.required[..5] != [0; 5]
            && self
                .static_sources()
                .iter()
                .any(|&id| self.object(id).card.has_activation_mana_flexibility())
        {
            return true;
        }
        // A virtual source departure can restore an ability under layer6.
        // The total upper bound remains conservative; color/readiness pruning
        // must not assume the current blanking is permanent for this search.
        if matches!(self.purpose, mana_lots::Purpose::Spell(spell) if improvise::potential(self, spell.card))
            || self.game.state.borrowed_links.is_some()
            || ability_world::CharacteristicWorld::has_ability_removal(self)
        {
            let upper = self.possible_color_upper();
            return upper
                .into_iter()
                .zip(goal.required)
                .all(|(available, required)| available >= u128::from(required));
        }
        let mut upper = self.mana;
        let (tap_bonus, tap_factor) = self.land_tap_effects_upper(false);
        for resource in &self.resources {
            let object = &resource.object;
            if object.zone != Zone::Battlefield || object.controller != goal.actor {
                continue;
            }
            let tap_ready = !object.tapped
                && (self.kinds(resource) & 1 == 0
                    || !object.summoning_sick
                    || ability_world::CharacteristicWorld::activation_keywords(self, resource.id)
                        .contains(Keywords::HASTE));
            if tap_ready {
                // Overestimate: a later payment may change this source's subtype.
                for (total, amount) in upper.iter_mut().zip(tap_bonus) {
                    *total = total.saturating_add(amount);
                }
                for grant in ability_world::CharacteristicWorld::granted_mana(self, resource.id) {
                    self.production_upper(grant.production).visit(|mana| {
                        for i in 0..6 {
                            upper[i] = upper[i]
                                .saturating_add(u128::from(mana[i]).saturating_mul(tap_factor));
                        }
                        false
                    });
                }
            }
            if tap_ready
                && let Some(color) =
                    ability_world::CharacteristicWorld::intrinsic_mana_color(self, resource.id)
            {
                upper[color] = upper[color].saturating_add(tap_factor);
            }
            for (index, ability, borrowed, self_grant) in self.mana_activations(resource.id) {
                let Some(production) = ability.mana_production() else {
                    continue;
                };
                let mut readiness = *ability;
                if readiness.exhaust && exhaust::reuse_available(self, goal.actor) {
                    readiness.exhaust = false;
                }
                if goal
                    .source
                    .as_ref()
                    .is_some_and(|(source, pending, _, definition, slot)| {
                        *definition
                            == self_grant.map_or_else(
                                || borrowed.map_or(object.card, |a| a.definition),
                                |g| g.0,
                            )
                            && *slot == borrowed.map(|a| a.slot)
                            && source.card == resource.id
                            && object.generation == source.generation
                            && *pending == index
                    })
                    || !self.guard_available(ability.guard, resource.id, goal.actor)
                    || !Game::object_activation_available_with_kinds(
                        object,
                        ability_world::CharacteristicWorld::counter_amounts(self, resource.id),
                        self.kinds(resource),
                        || {
                            ability_world::CharacteristicWorld::activation_keywords(
                                self,
                                resource.id,
                            )
                        },
                        self.life,
                        &readiness,
                        if borrowed.is_some() {
                            0
                        } else {
                            index as usize
                        },
                        goal.actor,
                        false,
                    )
                {
                    continue;
                }
                let cost = ability.cost;
                let mut uses = if cost.tap || ability.once_per_turn || ability.exhaust {
                    1
                } else {
                    u64::MAX
                };
                if cost.life > 0 && !self.life_replacement_possible {
                    uses = uses.min(
                        u64::try_from(self.life.max(0) / i128::from(cost.life)).unwrap_or(u64::MAX),
                    );
                }
                if let Some((kind, amount)) = cost.remove {
                    let count =
                        ability_world::CharacteristicWorld::counter_amounts(self, resource.id)
                            .available(kind);
                    uses = uses.min(u64::try_from(count / u128::from(amount)).unwrap_or(u64::MAX));
                }
                let mut maximum = [0u32; 6];
                self.production_upper(production).visit(|mana| {
                    for index in 0..6 {
                        maximum[index] = maximum[index].max(mana[index]);
                    }
                    false
                });
                let uses = u128::from(uses);
                let uses = if cost.tap {
                    uses.saturating_mul(tap_factor)
                } else {
                    uses
                };
                for index in 0..6 {
                    upper[index] = upper[index]
                        .saturating_add(uses.saturating_mul(u128::from(maximum[index])));
                }
            }
        }
        if self.has_contributions() {
            return convoke::payable(upper, goal.required, goal.generic, &self.convoke_masks());
        }
        if (0..6).any(|index| upper[index] < u128::from(goal.required[index])) {
            return false;
        }
        upper
            .iter()
            .zip(goal.required)
            .fold(0u128, |total, (&bound, required)| {
                total.saturating_add(bound - u128::from(required))
            })
            >= u128::from(goal.generic)
    }
    fn convoke_masks(&self) -> [u64; 32] {
        if self.waterbend != 0 {
            return waterbend::masks(self, self.actor, self.waterbend, &[]);
        }
        let mut masks = improvise::masks(
            self,
            self.actor,
            self.convoke_available(),
            self.improvise_available(),
            &[],
        );
        if self.delve_available() {
            masks[0] += delve::count(self, self.actor, self.payment_source, &[]);
        }
        masks
    }
    fn evidence_value(&self) -> u128 {
        // A mana payment may exile an existing graveyard card or create new
        // evidence by sacrificing a permanent. Read each exact virtual object
        // once rather than retaining the actual world's departed graveyard cards.
        self.game
            .state
            .objects
            .iter()
            .enumerate()
            .map(|(id, _)| self.object(id as CardId))
            .filter(|o| o.owner == self.actor && o.zone == Zone::Graveyard && o.is_card())
            .map(|o| u128::from(o.card.fixed_mana_value()))
            .sum()
    }
    /// Final-purpose facts follow the speculative world, not its original game.
    /// Ordinary pools keep the existing constant-time path.
    fn payment_purpose(&self, goal: &FundingGoal) -> mana_lots::Purpose {
        if self.lots.is_empty() || !matches!(self.purpose, mana_lots::Purpose::Activation(_)) {
            return self.purpose;
        }
        if let Some((object, ability, _, definition, _)) = goal.source.as_ref() {
            if let Some((_, facts, _)) = self
                .payment_departures
                .iter()
                .find(|(source, _, _)| source == object)
            {
                return mana_lots::Purpose::Activation(
                    (*facts).with_activation(definition.activation(*ability)),
                );
            }
            if self.reference(object.card) == *object
                && self.object(object.card).zone == definition.activation(*ability).source_zone()
            {
                return mana_lots::Purpose::Activation(
                    mana_lots::Source::in_world(self, object.card)
                        .with_activation(definition.activation(*ability)),
                );
            }
        }
        self.purpose
    }
    fn reachable(&self, goal: &FundingGoal, visited: &mut HashSet<FundingKey>) -> bool {
        self.reachable_with_evidence(goal, None, visited)
    }
    fn reachable_with_evidence(
        &self,
        goal: &FundingGoal,
        evidence: Option<u16>,
        visited: &mut HashSet<FundingKey>,
    ) -> bool {
        if let Some(query) = self.scoped_continuation_query() {
            return query.reachable_with_evidence(goal, evidence, visited);
        }
        self.reachable_with_completion(goal, evidence, visited, &mut |world| {
            world.complete_funding_continuation(goal)
        })
    }
    fn reachable_with_completion(
        &self,
        goal: &FundingGoal,
        evidence: Option<u16>,
        visited: &mut HashSet<FundingKey>,
        completion: &mut dyn FnMut(&Self) -> bool,
    ) -> bool {
        if self.required_face_up.is_some_and(|object| {
            self.reference(object.card) != object
                || self.object(object.card).zone != Zone::Battlefield
                || self.object(object.card).controller != goal.actor
        }) {
            return false;
        }
        if !self.obligations(goal) {
            return false;
        }
        if self.additional_cost_deficit() || self.fixed_origin_counter_deficit() {
            return false;
        }
        if self.pure_tap_teamwork_deficit(goal) {
            return false;
        }
        let memoized = self.continuation_memo.is_some() && self.continuation_memo_safe();
        if memoized {
            let depth = self.continuation.as_ref().map_or(0, |c| c.outer.len());
            let key = self.key(goal);
            // Release the interior borrow before evaluating any continuation.
            let fresh = self
                .continuation_memo
                .as_ref()
                .unwrap()
                .visited
                .borrow_mut()
                .insert((depth, key));
            if !fresh {
                return false;
            }
        }
        let purpose = self.payment_purpose(goal);
        let adjusted = cast_origins::payment_goal(self, goal, purpose);
        if (if self.has_ordered_convoke() {
            true
        } else if self.has_contributions() {
            convoke::payable_in(
                self,
                goal.actor,
                purpose,
                mana_lots::available(self.mana, &self.lots, purpose),
                goal.required,
                goal.generic,
                &self.convoke_masks(),
            )
        } else {
            payable(
                mana_lots::available(self.mana, &self.lots, purpose),
                adjusted.required,
                adjusted.generic,
            )
        }) && evidence.is_none_or(|n| self.evidence_value() >= u128::from(n))
            && activation_counter_placement_payable(self, goal)
            && source_exile_payable(self, goal)
            && goal.reserve.is_none_or(|(reference, spec)| {
                spec.kind.payment_kind() != EffectChoiceKind::ExileControlledOrGraveyard
                    || !ability_world::CharacteristicWorld::graveyard_exile_prohibited(
                        self,
                        reference.card,
                    )
            })
            && self.cost_group_ready()
            && self.contributors.as_ref().is_none_or(|c| c.payable(self))
            && completion(self)
        {
            return true;
        }
        if evidence.is_some_and(|n| {
            let upper = self.evidence_value()
                + self
                    .resources
                    .iter()
                    .filter(|r| {
                        r.object.controller == self.actor
                            && r.object.zone == Zone::Battlefield
                            && !r.object.is_token()
                            && self.mana_activations(r.id).any(|(_, a, _, _)| {
                                a.cost.sacrifice
                                    && (!a.cost.tap || !r.object.tapped)
                                    && a.mana_production().is_some()
                            })
                    })
                    .map(|r| u128::from(r.object.card.fixed_mana_value()))
                    .sum::<u128>()
                + self
                    .active_melds()
                    .flat_map(|row| row.entry.front_cards)
                    .map(|card| u128::from(card.fixed_mana_value()))
                    .sum::<u128>();
            upper < u128::from(n)
        }) || !self.optimistic(goal)
        {
            return false;
        }
        if !memoized && !visited.insert(self.key(goal)) {
            return false;
        }
        // First try completing mana abilities that are payable now. Only a
        // failed ordinary branch needs to freeze another ability before funding
        // its cost. The second pass retains those exact nested plans without
        // starting their child searches ahead of available ordinary steps.
        for deferred in [false, true] {
            let mut moves = if deferred {
                self.deferred_moves(goal)
            } else {
                self.moves(goal)
            };
            // Prefer mana eligible for the final payment and colors that close
            // its requirements. Retain every move: an ineligible lot can still
            // fund an intermediate ability or change the projected world. Stable
            // sorting preserves the order of equally useful moves.
            let adjusted = cast_origins::payment_goal(self, goal, purpose);
            let available = mana_lots::available(self.mana, &self.lots, purpose);
            moves.sort_by_key(|step| {
                let ineligible = step
                    .quality
                    .is_some_and(|quality| !quality.eligible(purpose));
                let mut after = available;
                if !ineligible {
                    for (amount, produced) in after.iter_mut().zip(step.production) {
                        *amount = amount.saturating_add(u128::from(produced));
                    }
                }
                let missing =
                    after
                        .iter()
                        .zip(adjusted.required)
                        .fold(0u128, |sum, (&amount, required)| {
                            sum.saturating_add(u128::from(required).saturating_sub(amount))
                        });
                (
                    ineligible,
                    !payable(after, adjusted.required, adjusted.generic),
                    missing,
                )
            });
            for step in moves {
                if self.visit_applied_for(&step, Some(goal), &mut |next| {
                    next.reachable_with_completion(goal, evidence, visited, completion)
                }) {
                    return true;
                }
            }
        }
        false
    }
    fn restore_payment_context(&mut self, parent: &Self) {
        self.purpose = parent.purpose;
        self.spell_lot_demand = parent.spell_lot_demand;
        self.payment_source = parent.payment_source;
        self.exhaust_source_reuse_authorized = parent.exhaust_source_reuse_authorized;
        self.source_use_independent = parent.source_use_independent;
        self.required_face_up = parent.required_face_up;
        self.contributors = parent.contributors.clone();
        self.convoke = parent.convoke;
        self.waterbend = parent.waterbend;
        self.cost_group = parent.cost_group.clone();
        self.cast_costs = parent.cast_costs.clone();
        self.continuation = parent.continuation.clone();
        self.continuation_memo = parent.continuation_memo.clone();
        self.active_mana_keys = parent.active_mana_keys.clone();
        self.watched_sources = parent.watched_sources.clone();
    }
    fn visit_deferred(&self, step: &FundingMove, emit: &mut dyn FnMut(&Self) -> bool) -> bool {
        let output = step.output.as_ref().expect("deferred output");
        let cost = output.deferred_cost.expect("deferred cost");
        let key = output.instance;
        let rules = key.definition.activation(key.ability);
        let external = step.external.as_ref().map(|cost| {
            (
                rules.cost.additional.expect("external cost"),
                cost.selection.objects(),
            )
        });
        let goal = FundingGoal {
            actor: output.controller,
            required: cost.colored,
            generic: cost.generic,
            reserve: external.and_then(|(spec, objects)| match objects {
                [object] => Some((*object, spec)),
                _ => None,
            }),
            source: Some((
                key.source,
                key.ability,
                step.counters.clone(),
                key.definition,
                key.borrowed_slot,
            )),
        };
        let mut child = self.clone();
        if let Some(source) = self.payment_source {
            child.watched_sources.push(source);
        }
        let keys = child
            .active_mana_keys
            .get_or_insert_with(|| self.game.active_mana_activation_keys());
        if keys.contains(&key) {
            return false;
        }
        keys.push(key);
        child.purpose = mana_lots::Purpose::Activation(
            mana_lots::Source::in_world(self, key.source.card).with_activation(rules),
        );
        child.payment_source = Some(key.source);
        child.exhaust_source_reuse_authorized = exhaust::reuse_available(self, output.controller);
        child.source_use_independent = key.self_grant.is_some();
        child.required_face_up = None;
        child.contributors = None;
        child.convoke = false;
        child.waterbend = 0;
        child.cost_group = external
            .and_then(|(spec, objects)| (objects.len() > 1).then(|| (spec, objects.to_vec())));
        child.cast_costs = None;
        child.continuation = None;
        child.continuation_memo = None;
        child.reachable_with_completion(&goal, None, &mut HashSet::new(), &mut |world| {
            let purpose = world.payment_purpose(&goal);
            let adjusted = cast_origins::payment_goal(world, &goal, purpose);
            let pool = mana_lots::available(world.mana, &world.lots, purpose);
            let mut found = false;
            funding_payments(pool, adjusted.required, adjusted.generic, |payment| {
                if found {
                    return;
                }
                mana_lots::visit_allocations(
                    world.mana,
                    &world.lots,
                    purpose,
                    payment,
                    |allocation| {
                        if found {
                            return;
                        }
                        let mut paid = step.clone();
                        paid.payment = payment;
                        paid.allocation = allocation;
                        paid.output.as_mut().unwrap().deferred_cost = None;
                        paid.frozen = Some(Box::new((key.definition, key.borrowed_slot)));
                        found = world.visit_applied(&paid, &mut |completed| {
                            let mut next = completed.clone();
                            next.restore_payment_context(self);
                            emit(&next)
                        });
                    },
                );
            });
            found
        })
    }
    fn action(&self, step: &FundingMove) -> Action {
        let card = self.resources[step.resource].id;
        if let Some((_, grant_timestamp)) = step.self_grant {
            return Action::AnnounceSelfGrantedMana {
                source: self.reference(card),
                grant_timestamp,
                counter_payment: step.counters.clone(),
                production: step.production,
            };
        }
        if let Some(grantor) = step.grantor {
            if step.ability.is_some() {
                return Action::AnnounceGrantedMana {
                    card,
                    grantor,
                    production: step.production,
                };
            }
            return Action::TapGrantedMana {
                card,
                grantor,
                production: step.production,
            };
        }
        if step.external.is_some() || step.output.as_ref().and_then(|o| o.deferred_cost).is_some() {
            if let Some(borrowed) = step.borrowed {
                return if borrowed.zone == Zone::Graveyard {
                    Action::AnnounceGraveyardBorrowedMana {
                        card,
                        graveyard: borrowed.donor,
                        ability: borrowed.ability,
                        counter_payment: step.counters.clone(),
                        production: step.production,
                    }
                } else {
                    Action::AnnounceBorrowedMana {
                        card,
                        exiled: borrowed.donor,
                        ability: borrowed.ability,
                        counter_payment: step.counters.clone(),
                        production: step.production,
                    }
                };
            }
            return Action::AnnounceMana {
                card,
                ability: step.ability.expect("external mana ability"),
                counter_payment: step.counters.clone(),
                production: step.production,
            };
        }
        if let Some(borrowed) = step.borrowed {
            return if borrowed.zone == Zone::Graveyard {
                Action::ActivateGraveyardBorrowedMana {
                    card,
                    graveyard: borrowed.donor,
                    ability: borrowed.ability,
                    payment: concrete_payment(step.payment),
                    counter_payment: step.counters.clone(),
                    production: step.production,
                }
            } else {
                Action::ActivateBorrowedMana {
                    card,
                    exiled: borrowed.donor,
                    ability: borrowed.ability,
                    payment: concrete_payment(step.payment),
                    counter_payment: step.counters.clone(),
                    production: step.production,
                }
            };
        }
        match step.ability {
            Some(ability) => Action::ActivateMana {
                card,
                ability,
                payment: concrete_payment(step.payment),
                counter_payment: step.counters.clone(),
                production: step.production,
            },
            None => Action::TapMana { card },
        }
    }
}
#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
impl Game {
    pub(super) fn preview_battle_entry(&self, movement: &entries::EntryMove) -> Option<ObjectView> {
        self.ghost_entry_preview(movement)
    }
    pub(super) fn entry_replacement_has_subtype(
        &self,
        movement: &entries::EntryMove,
        subtype: &str,
    ) -> bool {
        let goal = FundingGoal {
            actor: movement.controller,
            required: [0; 6],
            generic: 0,
            reserve: None,
            source: None,
        };
        FundingState::from_game(self, &goal).entry_replacement_has_subtype(movement, subtype)
    }

    pub(super) fn entry_counter_bonus_by_mana_value(
        &self,
        movement: &entries::EntryMove,
        filter: ReplacementFilter,
        maximum: u64,
        at_most: u16,
        above: u16,
    ) -> Option<u64> {
        let goal = FundingGoal {
            actor: movement.controller,
            required: [0; 6],
            generic: 0,
            reserve: None,
            source: None,
        };
        FundingState::from_game(self, &goal)
            .entry_counter_bonus_by_mana_value(movement, filter, maximum, at_most, above)
    }
    pub(super) fn preview_entry_riot(
        &self,
        movement: &entries::EntryMove,
    ) -> Option<riot::Preparation> {
        let goal = FundingGoal {
            actor: movement.controller,
            required: [0; 6],
            generic: 0,
            reserve: None,
            source: None,
        };
        FundingState::from_game(self, &goal).preview_entry_riot(movement)
    }
    pub(super) fn ghost_entry_preview(&self, movement: &entries::EntryMove) -> Option<ObjectView> {
        let goal = FundingGoal {
            actor: movement.controller,
            required: [0; 6],
            generic: 0,
            reserve: None,
            source: None,
        };
        FundingState::from_game(self, &goal).preview_entry_view(movement)
    }
    pub(super) fn can_fund_cost_group(
        &self,
        goal: FundingGoal,
        purpose: mana_lots::Purpose,
        spec: AdditionalCost,
        group: &[ObjectRef],
    ) -> bool {
        if let Some(result) = self.preview_optional_cast_costs(
            goal.clone(),
            purpose,
            ObjectRef {
                card: CardId::MAX,
                generation: 0,
            },
            Some(spec),
            None,
            group,
            false,
            &[],
            &[],
        ) {
            return result;
        }
        let mut state = FundingState::from_game(self, &goal);
        state.set_purpose(purpose);
        state.cost_group = Some((spec, group.to_vec()));
        state.reachable(&goal, &mut HashSet::new())
    }
    pub(super) fn can_fund_sacrifice_group(
        &self,
        goal: FundingGoal,
        _spec: AdditionalCost,
        group: &[ObjectRef],
    ) -> bool {
        let mut state = FundingState::from_game(self, &goal);
        state.sacrifice_group = group.to_vec();
        state.reachable(&goal, &mut HashSet::new())
    }
    pub(super) fn can_fund(&self, goal: FundingGoal) -> bool {
        let purpose = self.funding_purpose(&goal);
        self.can_fund_for(goal, purpose)
    }
    pub(super) fn can_fund_face_up(&self, goal: FundingGoal, object: ObjectRef) -> bool {
        let mut state = FundingState::from_game(self, &goal);
        state.set_purpose(mana_lots::Purpose::Other);
        state.required_face_up = Some(object);
        state.reachable(&goal, &mut HashSet::new())
    }
    pub(super) fn contributor_selection_fundable(
        &self,
        selected: &[ObjectRef],
        complete: bool,
    ) -> bool {
        let a = self
            .state
            .announcing
            .as_ref()
            .expect("contributor announcement");
        let goal = self.funding_goal(a);
        let mut world = FundingState::from_game(self, &goal);
        let selection = world
            .contributors
            .as_mut()
            .expect("Crew or Saddle requirement");
        selection.selected = selected.to_vec();
        selection.complete = complete;
        world.set_purpose(self.purpose_for_item(&a.item));
        if goal.required == [0; 6]
            && goal.generic == 0
            && a.activation_generic_increase == 0
            && goal
                .source
                .as_ref()
                .is_some_and(|(_, ability, _, definition, _)| {
                    !definition.activation(*ability).cost.mana_component
                })
        {
            return world.obligations(&goal)
                && world
                    .contributors
                    .as_ref()
                    .is_none_or(|c| c.payable(&world));
        }
        world.reachable(&goal, &mut HashSet::new())
    }

    fn self_granted_goal(&self, goal: &FundingGoal) -> bool {
        self.state.announcing.as_ref().is_some_and(|a| {
            a.item.self_granted_activation.is_some()
                && goal
                    .source
                    .as_ref()
                    .is_some_and(|(source, index, _, definition, _)| {
                        a.item
                            .source
                            .is_some_and(|s| s.object == *source && s.card == *definition)
                            && a.item.kind == StackKind::ActivatedAbility { ability: *index }
                    })
        })
    }
    pub(super) fn can_fund_for(&self, goal: FundingGoal, purpose: mana_lots::Purpose) -> bool {
        if self.contributor_requirement(&goal).is_some() {
            let mut world = FundingState::from_game(self, &goal);
            world.set_purpose(purpose);
            if goal.required == [0; 6]
                && goal.generic == 0
                && goal
                    .source
                    .as_ref()
                    .is_some_and(|(source, ability, _, definition, _)| {
                        !definition.activation(*ability).cost.mana_component
                            && self.named_activation_generic_increase(source.card) == 0
                    })
            {
                return world.obligations(&goal)
                    && world
                        .contributors
                        .as_ref()
                        .is_none_or(|c| c.payable(&world));
            }
            return world.reachable(&goal, &mut HashSet::new());
        }
        if self.state.announcing.as_ref().is_some_and(|a| {
            a.cast_costs.is_some()
                || a.additional_cost()
                    .is_some_and(|s| s.kind.variable_cost_minimum().is_some())
        }) {
            let mut world = FundingState::from_game(self, &goal);
            world.set_purpose(purpose);
            return world.reachable(&goal, &mut HashSet::new());
        }
        if let Some(result) = self.preview_optional_cast_costs(
            goal.clone(),
            purpose,
            ObjectRef {
                card: CardId::MAX,
                generation: 0,
            },
            goal.reserve.map(|(_, spec)| spec),
            None,
            &goal.reserve.map(|(r, _)| r).into_iter().collect::<Vec<_>>(),
            goal.reserve.is_some(),
            &[],
            &[],
        ) {
            return result;
        }
        if let Some(a) = &self.state.announcing
            && !a.additional_paid
            && let Some(spec) = a.counter_cost_spec()
        {
            return self.counter_cost_fundable(
                goal,
                purpose,
                spec,
                a.creature_counter_cost
                    .as_ref()
                    .map_or(&[], |v| v.as_slice()),
            );
        }
        if let Some(a) = &self.state.announcing
            && !a.additional_paid
            && let Some(group) = &a.additional_group
        {
            let spec = a.additional_cost().expect("selected tap cost");
            if !group.iter().all(|&r| {
                self.additional_candidate(
                    spec,
                    a.item.controller,
                    a.item
                        .source
                        .expect("cost announcement has object source")
                        .object,
                    r,
                )
            }) {
                return false;
            }
        }
        let object = goal
            .source
            .as_ref()
            .and_then(|(r, _, _, _, _)| self.state.objects.get(r.card as usize));
        let reserve = goal
            .reserve
            .and_then(|(r, _)| self.state.objects.get(r.card as usize));
        if !reserve_obligations(self, &goal, reserve, |card| self.keywords(card))
            || !source_obligations(
                &goal,
                object,
                goal.source
                    .as_ref()
                    .map_or(CounterAmounts::default(), |(r, _, _, _, _)| {
                        self.counter_amounts(r.card)
                    }),
                object.map_or(0, |o| {
                    self.object_kinds(goal.source.as_ref().unwrap().0.card, o)
                }),
                || {
                    object.map_or(Keywords::NONE, |_| {
                        ability_world::CharacteristicWorld::activation_keywords(
                            self,
                            goal.source.as_ref().unwrap().0.card,
                        )
                    })
                },
                self.state.players[goal.actor.index()].life,
                self.exhaust_reuse_authorized_for_goal(&goal),
                self.self_granted_goal(&goal),
            )
        {
            return false;
        }
        let adjusted = cast_origins::payment_goal(self, &goal, purpose);
        // The common prefunded path needs no resource graph or heap allocations.
        if payable(
            self.available_mana(goal.actor, purpose),
            adjusted.required,
            adjusted.generic,
        ) && activation_counter_placement_payable(self, &goal)
            && source_exile_payable(self, &goal)
            && goal.reserve.is_none_or(|(reference, spec)| {
                spec.kind.payment_kind() != EffectChoiceKind::ExileControlledOrGraveyard
                    || !ability_world::CharacteristicWorld::graveyard_exile_prohibited(
                        self,
                        reference.card,
                    )
            })
        {
            return true;
        }
        let mut state = FundingState::from_game(self, &goal);
        state.set_purpose(purpose);
        state.reachable(&goal, &mut HashSet::new())
    }
    // Numeric funding fixtures use the default-purpose convenience wrapper;
    // runtime spell callers always provide their actual face purpose.
    pub(super) fn can_fund_convoke_for(
        &self,
        goal: FundingGoal,
        purpose: mana_lots::Purpose,
    ) -> bool {
        self.can_fund_contributions_for(goal, purpose, true)
    }
    pub(super) fn can_fund_contributions_for(
        &self,
        goal: FundingGoal,
        purpose: mana_lots::Purpose,
        convoke: bool,
    ) -> bool {
        let artifact = matches!(purpose, mana_lots::Purpose::Spell(spell) if improvise::available(self, goal.actor, spell.card));
        if convoke::payable_in(
            self,
            goal.actor,
            purpose,
            self.available_mana(goal.actor, purpose).map(u128::from),
            goal.required,
            goal.generic,
            &improvise::masks(
                self,
                goal.actor,
                match purpose {
                    mana_lots::Purpose::Spell(spell) => {
                        convoke::available(self, goal.actor, spell.card)
                    }
                    _ => convoke,
                },
                artifact,
                &[],
            ),
        ) {
            return true;
        }
        let mut state = FundingState::from_game_with_convoke(self, &goal, convoke);
        state.set_purpose(purpose);
        state.reachable(&goal, &mut HashSet::new())
    }
    pub(super) fn can_fund_evidence_for(
        &self,
        goal: FundingGoal,
        threshold: u16,
        purpose: mana_lots::Purpose,
    ) -> bool {
        let current = self.state.announcing.as_ref().filter(|a| {
            a.item.controller == goal.actor
                && matches!(purpose, mana_lots::Purpose::Spell(spell) if a.item.source.expect("cost announcement has object source").card == spell.card)
        });
        let selection = current.and_then(|a| a.evidence.as_deref());
        if let Some(result) = self.preview_optional_cast_costs(
            goal.clone(),
            purpose,
            current.map_or(
                ObjectRef {
                    card: CardId::MAX,
                    generation: 0,
                },
                |a| {
                    a.item
                        .source
                        .expect("cost announcement has object source")
                        .object
                },
            ),
            Some(evidence::cost(threshold)),
            None,
            selection.map_or(&[], |e| e.selected.as_slice()),
            selection.is_some_and(|e| e.complete),
            &[],
            &[CastCostClause::Additional],
        ) {
            return result;
        }
        let mut state = FundingState::from_game(self, &goal);
        state.set_purpose(purpose);
        state.reachable_with_evidence(&goal, Some(threshold), &mut HashSet::new())
    }
    pub(super) fn announcement_payment_request(
        &self,
        viewer: Player,
    ) -> Option<PaymentRequestView> {
        let viewer = self.choice_information_player(viewer);
        if self.semantic_actor() != Some(viewer)
            || matches!(
                self.state.pending,
                Pending::Saddle { .. } | Pending::Crew { .. }
            )
        {
            return None;
        }
        let a = self.state.announcing.as_ref()?;
        // A hybrid equivalent may still be pending during mode selection too.
        // Project a fixed payment only after that cost has been committed.
        if viewer != a.item.controller
            || a.additional_paid
            || !a.pay_window
            || (a.has_x_cost() && a.item.x.is_none())
            || a.needs_offspring()
            || a.needs_teamwork()
            || a.needs_blight()
            || a.needs_kicker()
            || a.needs_gift()
            || a.needs_bargain()
            || a.needs_evidence()
            || a.additional_cost().is_some_and(forage::is_forage_cost) && a.forage.is_none()
            || a.needs_mana_cost()
            || a.activation_order_costs().is_some()
            || (a
                .item
                .source
                .expect("cost announcement has object source")
                .card
                .tiered_costs()
                .is_some()
                && a.item.mode.is_none())
            || (a
                .additional_cost()
                .is_some_and(|spec| !forage::is_forage_cost(spec))
                && !a
                    .additional_cost()
                    .is_some_and(|s| evidence::threshold(s).is_some())
                && a.additional.is_none()
                && !a.additional_group_complete)
            || a.evidence.as_ref().is_some_and(|e| !e.complete)
            || a.forage.as_ref().is_some_and(|f| !f.complete())
        {
            return None;
        }
        let goal = self.funding_goal(a);
        let goal = cast_origins::payment_goal(self, &goal, self.purpose_for_item(&a.item));
        Some(PaymentRequestView {
            required: goal.required,
            generic: goal.generic,
            counters: a.counter_payment.clone(),
            production: a.production,
        })
    }
    pub(super) fn harmonize_proposal_fundable(
        &self,
        a: &Announcement,
        object: Option<ObjectRef>,
        total: ManaCost,
    ) -> bool {
        let mut proposal = a.clone();
        proposal.harmonizer = Some(object);
        proposal.total_cost = Some(total);
        if delve::potential(self) {
            proposal.add_cast_cost_clause(CastCostClause::Mana);
        }
        if let Some(reference) = object {
            proposal.add_cast_cost_clause(CastCostClause::Harmonize(reference));
        }
        proposal.initialize_cast_costs(convoke::cost_potential(
            self,
            a.item
                .source
                .expect("cost announcement has object source")
                .card,
        ));
        let goal = self.funding_goal(&proposal);
        let mut world = FundingState::from_game_with_convoke(
            self,
            &goal,
            convoke::potential(
                self,
                a.item
                    .source
                    .expect("cost announcement has object source")
                    .card,
            ),
        );
        world.set_purpose(self.purpose_for_item(&a.item));
        world.set_cost_context(Some(&proposal));
        world.reachable(&goal, &mut HashSet::new())
    }
    pub(super) fn blight_x_cost_fundable(&self, a: &Announcement, cost: ManaCost, x: u32) -> bool {
        let mut proposal = a.clone();
        proposal.item.x = Some(x);
        proposal.cost = Some(cost);
        proposal.total_cost = None;
        let definition = proposal.item.source.unwrap().card;
        proposal.initialize_cast_costs(improvise::potential(self, definition));
        let goal = self.funding_goal(&proposal);
        let mut world = FundingState::from_game_with_convoke(self, &goal, definition.has_convoke());
        world.set_purpose(self.purpose_for_item(&proposal.item));
        world.set_cost_context(Some(&proposal));
        world.reachable(&goal, &mut HashSet::new())
    }
    pub(super) fn announcement_cost_fundable(&self, a: &Announcement, cost: ManaCost) -> bool {
        if a.item.kind == StackKind::Spell
            && a.additional_cost().is_some_and(|s| {
                matches!(
                    s.kind,
                    EffectChoiceKind::BeholdMany(_) | EffectChoiceKind::BeholdAndExile
                )
            })
        {
            return self.behold_announcement_fundable(a, cost);
        }
        if a.needs_behold() {
            return self.behold_intent_fundable(a, false) || self.behold_intent_fundable(a, true);
        }
        if !self.exact_x_targets_fundable(a, None, false) {
            return false;
        }
        if a.blight_x_cost() {
            return self.blight_x_cost_fundable(a, cost, a.item.x.unwrap_or(0));
        }
        if a.nonmana_x() {
            let mut proposal = a.clone();
            proposal.cost = Some(cost);
            proposal.plan = self.target_spec(&proposal.item);
            if proposal.item.x.is_none()
                || !self.item_targets_available(&proposal.item, proposal.plan)
            {
                return false;
            }
            proposal.cast_costs.get_or_insert_with(Default::default);
            proposal.initialize_cast_costs(false);
            return self.activation_packet_fundable(&proposal);
        }
        if let StackKind::ActivatedAbility { ability } = a.item.kind
            && a.item
                .source
                .expect("activation source")
                .card
                .activation(ability)
                .life_x_cost()
        {
            let mut proposal = a.clone();
            proposal.cost = Some(cost);
            proposal.add_cast_cost_clause(CastCostClause::Life(a.item.x.unwrap_or(0)));
            proposal.initialize_cast_costs(false);
            return self.activation_packet_fundable(&proposal);
        }
        if a.item
            .cast
            .is_some_and(|cast| cast.method == CastMethod::Free)
            || a.linked_origin_counter_cost().is_some()
        {
            // Source-linked counter payments supplement all other costs. The
            // complete ordered context reserves them during initial preflight.
            return self.behold_announcement_fundable(a, cost);
        }
        if a.item.cast.is_some_and(|cast| cast.kicker == Some(true))
            && a.item
                .source
                .is_some_and(|source| source.card.kicker_additional_cost().is_some())
        {
            return self.behold_announcement_fundable(a, cost);
        }
        if a.self_graveyard_cast_cost().is_some() {
            return self.self_graveyard_announcement_fundable(a, cost);
        }
        if a.exile_play_surcharge() > 0 {
            let source = a.item.source.expect("surcharged spell source");
            if a.item
                .cast
                .is_some_and(|cast| cast.method == CastMethod::Normal)
                && !self.spell_cost_option_available(
                    a.item.controller,
                    source.id,
                    a.cost_before_exile_play_surcharge(cost),
                )
            {
                return false;
            }
            if source.card.has_target_cost_modifier() {
                return self.announcement_target_fundable(a, None, Some(cost), None, None, false);
            }
        }
        if a.exile_play_surcharge() > 0
            || a.item
                .cast
                .is_some_and(|cast| cast.method == CastMethod::Warp)
                && (a.payment_x_symbols() > 0
                    || a.item
                        .source
                        .is_some_and(|source| source.card.mana_additional_alternative().is_some()))
        {
            let mut proposal = a.clone();
            proposal.cost = Some(cost);
            if let Some(x) = proposal.item.x {
                let source = proposal.item.source.expect("warp spell source");
                let Some(total) = self.scalar_x_total_with_symbols(
                    proposal.item.controller,
                    source.id,
                    Self::announcement_base_cost(&proposal),
                    x,
                    source.card,
                    proposal.payment_x_symbols(),
                    &proposal.item.targets,
                    Self::announcement_optional_reduction(&proposal),
                ) else {
                    return false;
                };
                proposal.total_cost = Some(total);
            }
            return self.behold_announcement_fundable(&proposal, cost);
        }
        if a.has_ordered_life_cost() || a.item.cast.is_some_and(|cast| cast.method.is_mayhem()) {
            // These routes must preflight the actual alternative and its
            // additional costs instead of inferring them from the printed cost.
            return self.behold_announcement_fundable(a, cost);
        }
        if a.item.cast.is_some_and(|c| c.method.is_harmonize()) {
            let mut selected = a.clone();
            selected.cost = Some(cost);
            if let Some(object) = selected.harmonizer {
                return self.harmonize_choice_fundable(&selected, object);
            }
            if self.harmonize_choice_fundable(&selected, None) {
                return true;
            }
            return (0..self.state.objects.len()).any(|id| {
                self.harmonize_choice_fundable(&selected, Some(self.object_ref(id as CardId)))
            });
        }
        if a.item.kind == StackKind::Spell
            && a.item
                .source
                .expect("spell source")
                .card
                .behold_cast()
                .is_some()
        {
            return self.behold_announcement_fundable(a, cost);
        }
        if let StackKind::ActivatedAbility { ability } = a.item.kind
            && Self::activation_target_sensitive(
                a.item
                    .source
                    .expect("cost announcement has object source")
                    .card,
                ability,
            )
        {
            let mut proposal = a.clone();
            proposal.cost = Some(cost);
            return self.announcement_activation_target_fundable(&proposal, None, false);
        }
        if a.item.kind == StackKind::Spell
            && a.item
                .source
                .expect("spell source")
                .card
                .has_target_cost_modifier()
        {
            return self.announcement_target_fundable(a, None, Some(cost), None, None, false);
        }
        if a.item
            .cast
            .is_some_and(|c| c.method == CastMethod::Flashback)
            && let Some(spec) = a
                .item
                .source
                .expect("cost announcement has object source")
                .card
                .flashback_additional()
        {
            let Some(cost) = self.reduced_spell_cost(
                a.item.controller,
                a.item
                    .source
                    .expect("cost announcement has object source")
                    .id,
                cost,
            ) else {
                return false;
            };
            let goal = FundingGoal {
                actor: a.item.controller,
                required: cost.colored,
                generic: cost.generic,
                reserve: None,
                source: None,
            };
            let purpose = self.purpose_for_item(&a.item);
            if matches!(
                spec.kind.payment_kind(),
                EffectChoiceKind::TapPermanents(_)
                    | EffectChoiceKind::ExileGraveyardCards(_)
                    | EffectChoiceKind::ExileGraveyardSymbols { .. }
            ) {
                return self.group_cost_fundable(
                    goal,
                    purpose,
                    spec,
                    a.item
                        .source
                        .expect("cost announcement has object source")
                        .object,
                    &[],
                );
            }
            return self.state.objects.iter().enumerate().any(|(id, _)| {
                let candidate = self.object_ref(id as CardId);
                self.additional_candidate(
                    spec,
                    a.item.controller,
                    a.item
                        .source
                        .expect("cost announcement has object source")
                        .object,
                    candidate,
                ) && self.can_fund_for(
                    FundingGoal {
                        reserve: Some((candidate, spec)),
                        ..goal.clone()
                    },
                    purpose,
                )
            });
        }
        if a.graveyard_origin_counter_cost().is_some() {
            let Some(cost) = self.reduced_spell_cost(
                a.item.controller,
                a.item
                    .source
                    .expect("cost announcement has object source")
                    .id,
                cost,
            ) else {
                return false;
            };
            return self.counter_cost_fundable(
                FundingGoal {
                    actor: a.item.controller,
                    required: cost.colored,
                    generic: cost.generic,
                    reserve: None,
                    source: None,
                },
                self.purpose_for_item(&a.item),
                Self::creature_counter_spec(a.graveyard_origin_counter_cost().unwrap()),
                a.creature_counter_cost
                    .as_ref()
                    .map_or(&[], |v| v.as_slice()),
            );
        }
        if a.item.kind == StackKind::Spell
            && a.item.cast.is_some_and(|c| {
                matches!(
                    c.method,
                    CastMethod::Free | CastMethod::Plot | CastMethod::Miracle | CastMethod::Airbend
                )
            })
        {
            let definition = a
                .item
                .source
                .expect("cost announcement has object source")
                .card;
            let base = Self::with_kicker_cost(
                Self::with_offspring_cost(
                    cost,
                    definition,
                    a.item.cast.is_some_and(|c| c.offspring == Some(true)),
                ),
                definition,
                a.item.cast.is_some_and(|c| c.kicker == Some(true)),
            );
            if definition.has_bargain() {
                return match a.item.cast.and_then(|c| c.bargain) {
                    Some(true) => self.bargain_cost_fundable(
                        a.item.controller,
                        a.item
                            .source
                            .expect("cost announcement has object source")
                            .id,
                        base,
                        a.additional,
                    ),
                    Some(false) => self.bargain_cost_fundable(
                        a.item.controller,
                        a.item
                            .source
                            .expect("cost announcement has object source")
                            .id,
                        base,
                        None,
                    ),
                    None => self.initial_bargain_fundable(
                        a.item.controller,
                        a.item
                            .source
                            .expect("cost announcement has object source")
                            .id,
                        base,
                    ),
                };
            }
            if definition.collect_evidence_cost().is_some() {
                return match a.item.cast.and_then(|c| c.evidence) {
                    Some(collect) => self.evidence_cast_fundable(
                        a.item.controller,
                        a.item
                            .source
                            .expect("cost announcement has object source")
                            .id,
                        base,
                        collect,
                    ),
                    None => {
                        self.evidence_cast_fundable(
                            a.item.controller,
                            a.item
                                .source
                                .expect("cost announcement has object source")
                                .id,
                            base,
                            false,
                        ) || self.evidence_cast_fundable(
                            a.item.controller,
                            a.item
                                .source
                                .expect("cost announcement has object source")
                                .id,
                            base,
                            true,
                        )
                    }
                };
            }
            let Some(total) = self.spell_total_for_definition(
                a.item.controller,
                a.item
                    .source
                    .expect("cost announcement has object source")
                    .id,
                base,
                definition,
            ) else {
                return false;
            };
            if definition.mana_additional_alternative().is_some()
                && Some(cost) != a.fixed_alternative_cost()
            {
                return self.can_fund_for(
                    FundingGoal {
                        actor: a.item.controller,
                        required: total.colored,
                        generic: total.generic,
                        reserve: None,
                        source: None,
                    },
                    self.purpose_for_item(&a.item),
                );
            }
            return self.spell_can_fund_total_for_definition(
                a.item.controller,
                a.item
                    .source
                    .expect("cost announcement has object source")
                    .id,
                total,
                None,
                definition,
            );
        }
        if a.item.cast.is_some_and(|c| {
            matches!(c.method, CastMethod::Sneak | CastMethod::WebSlinging)
                || c.method.pays_life_for_mana_value()
        }) {
            let mut proposal = a.clone();
            proposal.cost = Some(cost);
            proposal.cast_costs = Some(Box::default());
            proposal.initialize_cast_costs(convoke::cost_potential(
                self,
                a.item
                    .source
                    .expect("cost announcement has object source")
                    .card,
            ));
            let goal = self.funding_goal(&proposal);
            let mut world = FundingState::from_game_with_convoke(
                self,
                &goal,
                convoke::potential(
                    self,
                    a.item
                        .source
                        .expect("cost announcement has object source")
                        .card,
                ),
            );
            world.set_purpose(self.purpose_for_item(&a.item));
            world.set_payment_context(&proposal);
            return world.reachable(&goal, &mut HashSet::new());
        }
        if a.item.kind == StackKind::Spell
            && !self.spell_cost_option_available(
                a.item.controller,
                a.item
                    .source
                    .expect("cost announcement has object source")
                    .id,
                cost,
            )
        {
            return false;
        }
        if a.item.kind == StackKind::Spell
            && a.item
                .source
                .expect("cost announcement has object source")
                .card
                .x_symbols()
                > 0
        {
            return a.item.x.is_some_and(|x| {
                self.scalar_x_total(
                    a.item.controller,
                    a.item
                        .source
                        .expect("cost announcement has object source")
                        .id,
                    cost,
                    x,
                )
                .is_some_and(|total| {
                    self.spell_can_fund_total_at_value(
                        a.item.controller,
                        a.item
                            .source
                            .expect("cost announcement has object source")
                            .id,
                        total,
                        None,
                        a.item.source.unwrap().card,
                        Some(a.item.mana_value()),
                    )
                })
            });
        }
        if a.item
            .source
            .expect("cost announcement has object source")
            .card
            .has_target_cost_modifier()
            && a.item.kind == StackKind::Spell
        {
            return self.announcement_target_fundable(a, None, Some(cost), None, None, false);
        }
        if a.item.kind == StackKind::Spell
            && a.item
                .source
                .expect("cost announcement has object source")
                .card
                .collect_evidence_cost()
                .is_some()
        {
            return self.evidence_cast_fundable(
                a.item.controller,
                a.item
                    .source
                    .expect("cost announcement has object source")
                    .id,
                cost,
                a.item.cast.is_some_and(|c| c.evidence == Some(true)),
            );
        }
        if a.item.kind == StackKind::Spell
            && a.item
                .source
                .expect("cost announcement has object source")
                .card
                .has_bargain()
        {
            return self.bargain_cost_fundable(
                a.item.controller,
                a.item
                    .source
                    .expect("cost announcement has object source")
                    .id,
                cost,
                a.additional,
            );
        }
        match a.item.kind {
            StackKind::Spell => self.spell_can_fund_for_definition(
                a.item.controller,
                a.item
                    .source
                    .expect("cost announcement has object source")
                    .id,
                Self::with_kicker_cost(
                    Self::with_offspring_cost(
                        cost,
                        a.item
                            .source
                            .expect("cost announcement has object source")
                            .card,
                        a.item.cast.is_some_and(|c| c.offspring == Some(true)),
                    ),
                    a.item
                        .source
                        .expect("cost announcement has object source")
                        .card,
                    a.item.cast.is_some_and(|c| c.kicker == Some(true)),
                ),
                a.item
                    .source
                    .expect("cost announcement has object source")
                    .card,
            ),
            StackKind::ActivatedAbility { ability } => self.activation_can_fund(
                FundingGoal {
                    reserve: None,
                    actor: a.item.controller,
                    required: cost.colored,
                    generic: cost.generic,
                    source: Some((
                        a.item
                            .source
                            .expect("cost announcement has object source")
                            .object,
                        ability,
                        a.counter_payment.clone(),
                        a.item
                            .source
                            .expect("cost announcement has object source")
                            .card,
                        a.item.borrowed_slot,
                    )),
                },
                a.additional_cost(),
            ),
            StackKind::TriggeredAbility { .. }
            | StackKind::DecayedAttack
            | StackKind::SuspendTick
            | StackKind::SuspendCast
            | StackKind::CopyEndStepSacrifice
            | StackKind::CopyEndStepConditionalSacrifice
            | StackKind::CasualtyCopy
            | StackKind::ConspireCopy
            | StackKind::DelayedSacrifice
            | StackKind::DelayedReturn
            | StackKind::DelayedTokenCreation { .. }
            | StackKind::DelayedManifestDread
            | StackKind::DelayedWarpExile
            | StackKind::InherentRule { .. }
            | StackKind::SiegeDefeat => false,
        }
    }
    pub(super) fn funding_goal(&self, announcement: &Announcement) -> FundingGoal {
        if announcement.activation_cost_route == ActivationCostRoute::FirstEquipZero {
            // Only the printed equip cost is replaced. Preserve the computed
            // total after modifiers; the waived printed nonmana costs are not
            // reintroduced as source obligations during funding.
            let cost = announcement
                .total_cost
                .or(announcement.cost)
                .expect("selected equip route");
            return FundingGoal {
                actor: announcement.item.controller,
                required: cost.colored,
                generic: cost.generic,
                reserve: None,
                source: None,
            };
        }
        match announcement.item.kind {
            StackKind::Spell => {
                let cost = announcement
                    .convoke
                    .as_ref()
                    .map(|f| f.remaining)
                    .or(announcement.total_cost)
                    .unwrap_or_else(|| {
                        if announcement.payment_x_symbols() > 0
                            || announcement
                                .item
                                .source
                                .expect("cost announcement has object source")
                                .card
                                .x_symbols()
                                > 0
                        {
                            return self
                                .scalar_x_total(
                                    announcement.item.controller,
                                    announcement
                                        .item
                                        .source
                                        .expect("cost announcement has object source")
                                        .id,
                                    Self::announcement_base_cost(announcement),
                                    announcement.item.x.unwrap_or(0),
                                )
                                .expect("validated scalar cost");
                        }
                        self.provisional_target_cost(
                            announcement,
                            Self::announcement_base_cost(announcement),
                        )
                    });
                let required = cost.colored;
                let generic = cost.generic;
                FundingGoal {
                    reserve: self.announcement_reserve(announcement),
                    actor: announcement.item.controller,
                    required,
                    generic,
                    source: None,
                }
            }
            StackKind::ActivatedAbility { ability } => {
                let cost = announcement
                    .total_cost
                    .or(announcement.cost)
                    .unwrap_or_else(|| {
                        announcement.cost_options().next().expect("activation cost")
                    });
                FundingGoal {
                    reserve: self.announcement_reserve(announcement),
                    actor: announcement.item.controller,
                    required: cost.colored,
                    generic: cost.generic,
                    source: (!announcement.ordered_craft() || !announcement.craft_source_paid())
                        .then_some((
                            announcement
                                .item
                                .source
                                .expect("cost announcement has object source")
                                .object,
                            ability,
                            announcement.counter_payment.clone(),
                            announcement
                                .item
                                .source
                                .expect("cost announcement has object source")
                                .card,
                            announcement.item.borrowed_slot,
                        )),
                }
            }
            StackKind::TriggeredAbility { .. }
            | StackKind::DecayedAttack
            | StackKind::SuspendTick
            | StackKind::SuspendCast
            | StackKind::CopyEndStepSacrifice
            | StackKind::CopyEndStepConditionalSacrifice
            | StackKind::CasualtyCopy
            | StackKind::ConspireCopy
            | StackKind::DelayedSacrifice
            | StackKind::DelayedReturn
            | StackKind::DelayedTokenCreation { .. }
            | StackKind::DelayedManifestDread
            | StackKind::DelayedWarpExile
            | StackKind::InherentRule { .. }
            | StackKind::SiegeDefeat => unreachable!(),
        }
    }
    /// Complete coupled allocation constraints after every nonmana selection.
    pub fn mana_payment_domain(&self) -> Option<ManaPaymentDomain> {
        if matches!(self.state.pending, Pending::AttackPayment { .. }) {
            let request = self.state.attack_payment.as_ref()?;
            let available = self.available_mana(request.player, mana_lots::Purpose::Other);
            return payable(available.map(u128::from), [0; 6], request.generic).then_some(
                ManaPaymentDomain {
                    available,
                    required: [0; 6],
                    generic: request.generic,
                },
            );
        }
        if !matches!(
            self.state.pending,
            Pending::PayCosts { .. }
                | Pending::Convoke { .. }
                | Pending::Improvise { .. }
                | Pending::Delve { .. }
                | Pending::Waterbend { .. }
        ) {
            return None;
        }
        let a = self.state.announcing.as_ref()?;
        if a.cast_costs.as_ref().is_some_and(|c| {
            c.mana_closed
                && c.active
                    .is_none_or(|i| c.clauses[usize::from(i)].clause != CastCostClause::Mana)
        }) && !matches!(
            self.state.pending,
            Pending::Convoke { .. }
                | Pending::Improvise { .. }
                | Pending::Delve { .. }
                | Pending::Waterbend { .. }
        ) {
            return None;
        }
        let pool = self.available_mana(a.item.controller, self.purpose_for_item(&a.item));
        let (required, generic) = if let Some(frame) = &a.convoke {
            if !self.tap_contributors_valid(a) {
                return None;
            }
            if a.cast_costs.is_some() {
                let mut world = self.cast_cost_world(a);
                world.mark_mana_clause_paid();
                if !world.cost_completion() {
                    return None;
                }
            }
            cast_origins::payment_requirements(
                self,
                a.item.controller,
                self.purpose_for_item(&a.item),
                frame.remaining.colored,
                frame.remaining.generic,
            )
        } else {
            let goal = self.funding_goal(a);
            let mut state = FundingState::from_game(self, &goal);
            state.mark_mana_clause_paid();
            if !state.obligations(&goal)
                || !state.cost_group_ready()
                || !state
                    .contributors
                    .as_ref()
                    .is_none_or(|c| c.payable(&state))
                || !state.cost_completion()
                || !activation_counter_placement_payable(&state, &goal)
                || !source_exile_payable(&state, &goal)
                || a.cast_costs.is_none() && a.evidence.as_ref().is_some_and(|e| !e.complete)
            {
                return None;
            }
            cast_origins::payment_requirements(
                self,
                a.item.controller,
                self.purpose_for_item(&a.item),
                goal.required,
                goal.generic,
            )
        };
        payable(pool.map(u128::from), required, generic).then_some(ManaPaymentDomain {
            available: pool,
            required,
            generic,
        })
    }
    pub(super) fn discard_or_life_route_fundable(&self, announcement: &Announcement) -> bool {
        let mut proposal = announcement.clone();
        proposal.additional_life = true;
        if proposal.cast_costs.as_ref().is_some_and(|c| c.mana_closed) {
            return self.cast_cost_world(&proposal).cost_completion();
        }
        let goal = self.funding_goal(&proposal);
        let mut world = FundingState::from_game_with_convoke(
            self,
            &goal,
            convoke::potential(
                self,
                proposal
                    .item
                    .source
                    .expect("cost announcement has object source")
                    .card,
            ),
        );
        world.set_purpose(self.purpose_for_item(&proposal.item));
        world.set_payment_context(&proposal);
        world.reachable(&goal, &mut HashSet::new())
    }
    pub(super) fn mana_parent_fundable(
        &self,
        external: Option<ObjectRef>,
        payment: Option<Mana>,
    ) -> bool {
        let Some(announcement) = self.state.announcing.as_ref() else {
            return true;
        };
        if !self.has_mana_parent(announcement) {
            return true;
        }
        let Some(object) = external.filter(|_| payment.is_none()) else {
            return self.mana_total_parent_fundable(announcement, payment);
        };
        let mut child = announcement.clone();
        if let Some(spec) = child.additional_cost()
            && let EffectChoiceKind::TapPermanents(amount) = spec.kind.payment_kind()
        {
            let group = child.additional_group.get_or_insert_default();
            if group.len() >= amount as usize || group.contains(&object) {
                return false;
            }
            group.push(object);
            child.additional = (group.len() == amount as usize).then_some(object);
        } else {
            child.additional = Some(object);
        }
        self.mana_total_parent_fundable(&child, payment)
    }
    pub(super) fn activation_total_fundable(&self, candidate: &Announcement) -> bool {
        self.activation_packet_fundable(candidate)
            && (candidate.production.is_none() || self.mana_total_parent_fundable(candidate, None))
    }
    fn has_mana_parent(&self, child: &Announcement) -> bool {
        self.state.mana_announcement_stack.is_some()
            || child
                .mana_output
                .as_ref()
                .and_then(|o| o.resume.as_deref())
                .is_some_and(|pending| self.guided_mana_goal(pending).is_some())
    }
    fn mana_total_parent_fundable(&self, child: &Announcement, payment: Option<Mana>) -> bool {
        if !self.has_mana_parent(child) {
            return true;
        }
        let frames = self
            .state
            .mana_announcement_stack
            .as_deref()
            .map_or(&[][..], |frames| frames.as_slice());
        let goal = self.funding_goal(child);
        let quality = self
            .state
            .mana_lots
            .as_ref()
            .and_then(|m| m.announcement_quality.as_ref())
            .map(|(_, q)| q);
        let mut state = FundingState::from_game(self, &goal);
        state.set_purpose(self.purpose_for_item(&child.item));
        state.set_payment_context(child);
        state.set_funding_continuation(child, frames, quality);
        if let Some(payment) = payment {
            // This is completion of an existing CR602 announcement. It must not
            // pass through moves(), which enumerates only new legal activations
            // and correctly excludes every active mana-ability instance.
            state.obligations(&goal)
                && source_exile_payable(&state, &goal)
                && activation_counter_placement_payable(&state, &goal)
                && state.complete_funding_continuation_for(&goal, Some(payment))
        } else {
            state.reachable_with_evidence(
                &goal,
                child.evidence.as_ref().map(|e| e.threshold),
                &mut HashSet::new(),
            )
        }
    }
    /// Initial acquired nonmana activations need the immutable packet's origin
    /// before any live announcement exists. Reuse the full cost context so
    /// source use, target totals and additional witnesses stay in one world.
    pub(super) fn activation_packet_fundable(&self, candidate: &Announcement) -> bool {
        let goal = self.funding_goal(candidate);
        let mut world = FundingState::from_game(self, &goal);
        world.set_purpose(self.purpose_for_item(&candidate.item));
        world.set_payment_context(candidate);
        if goal.generic == 0
            && goal.required == [0; 6]
            && candidate.activation_generic_increase == 0
            && goal
                .source
                .as_ref()
                .is_some_and(|(_, index, _, definition, _)| {
                    !definition.activation(*index).cost.mana_component
                })
        {
            // A full proposal still has no mana-ability window when its total
            // contains only nonmana costs. Complete those costs in this world.
            return world.obligations(&goal) && world.cost_completion();
        }
        world.reachable_with_evidence(
            &goal,
            candidate.evidence.as_ref().map(|row| row.threshold),
            &mut HashSet::new(),
        )
    }
    pub(super) fn mana_packet_fundable(
        &self,
        packet: &SpellView,
        counters: Counters,
        production: Mana,
    ) -> bool {
        let quality = self.announced_mana_quality(packet, Some(production));
        let candidate = self.payment_announcement(packet.clone(), counters, Some(production), None);
        let parents = self.deferred_mana_parents();
        if !parents.is_empty()
            && candidate.additional_cost().is_some_and(|spec| {
                matches!(spec.kind.payment_kind(), EffectChoiceKind::TapPermanents(_))
            })
        {
            // During a parent payment, this family is enumerated by exact
            // FundingMoves, including deferred mana components. An unselected
            // packet has no external witness to project into its continuation;
            // it must not invent output or spend an ancestor's reserved tap.
            return false;
        }
        self.mana_cost_routes_fundable(candidate, &parents, quality.as_ref().map(|(_, q)| q))
    }
    fn mana_cost_routes_fundable(
        &self,
        mut candidate: Announcement,
        parents: &[mana_announcements::ManaAnnouncement],
        quality: Option<&mana_lots::Quality>,
    ) -> bool {
        (0..candidate.cost_options().count()).any(|slot| {
            candidate.select_cost(slot);
            candidate
                .untargeted_activation_totals()
                .into_iter()
                .flatten()
                .any(|total| {
                    candidate.total_cost = Some(total);
                    self.deferred_mana_fundable(&candidate, parents, quality)
                })
        })
    }
    fn deferred_mana_fundable(
        &self,
        candidate: &Announcement,
        parents: &[mana_announcements::ManaAnnouncement],
        quality: Option<&mana_lots::Quality>,
    ) -> bool {
        if candidate
            .item
            .mana_activation_key()
            .is_some_and(|key| self.active_mana_activation_keys().contains(&key))
        {
            return false;
        }
        let goal = self.funding_goal(candidate);
        // A payment cannot untap an exact source for an ancestor's pending
        // tap cost. Reject this child before searching ways to pay for it.
        if let Some((source, ..)) = goal.source.as_ref()
            && goal.reserves_source_tap(*source)
            && parents.iter().any(|frame| {
                self.funding_goal(&frame.announcement)
                    .reserves_source_tap(*source)
            })
        {
            return false;
        }
        let mut world = FundingState::from_game(self, &goal);
        world.set_purpose(self.purpose_for_item(&candidate.item));
        world.set_payment_context(candidate);
        world.set_funding_continuation(candidate, parents, quality);
        world.reachable_with_evidence(
            &goal,
            candidate.evidence.as_ref().map(|row| row.threshold),
            &mut HashSet::new(),
        )
    }
    fn visit_deferred_indexed_mana(&self, emit: &mut impl FnMut(Action) -> bool) -> bool {
        let Some(actor) = self.semantic_actor() else {
            return false;
        };
        let parents = self.deferred_mana_parents();
        if parents.is_empty() {
            return false;
        }
        for (id, object) in self.state.objects.iter().enumerate() {
            let id = id as CardId;
            if self.named_activation_generic_increase(id) != 0
                && (self.visit_intrinsic_mana(id, actor, emit)
                    || self.visit_granted_mana(id, actor, emit))
            {
                return true;
            }
            if self.visit_borrowed_activations(id, actor, false, &mut |action| {
                matches!(
                    action,
                    Action::AnnounceBorrowedMana { .. }
                        | Action::AnnounceGraveyardBorrowedMana { .. }
                ) && emit(action)
            }) || self.visit_self_granted_activations(id, actor, false, &mut |action| {
                matches!(action, Action::AnnounceSelfGrantedMana { .. }) && emit(action)
            }) {
                return true;
            }
            for (index, ability) in object.card.activations().iter().enumerate() {
                let Some(production) = ability.mana_production() else {
                    continue;
                };
                // Selected external costs retain the existing incremental route;
                // this candidate path adds deferred payment of a mana component.
                if (!ability.cost.mana_component && self.named_activation_generic_increase(id) == 0)
                    || ability.cost.additional.is_some()
                    || object.card.legacy_subtype_mana_activation(index as u16)
                    || object.card.self_granted_activation_template(index as u16)
                    || !self.activation_cost_available(id, ability, index, actor, false)
                {
                    continue;
                }
                let packet = self.activation_packet(actor, id, index as u16);
                let quality = self.announced_mana_quality(&packet, None);
                if self.visit_activation_counter_payments(
                    actor,
                    ability,
                    self.counter_amounts(id),
                    |counters| {
                        self.visit_activation_mana_outputs(
                            production,
                            id,
                            object.card,
                            index as u16,
                            |produced| {
                                let candidate = self.payment_announcement(
                                    packet.clone(),
                                    counters.clone(),
                                    Some(produced),
                                    None,
                                );
                                self.mana_cost_routes_fundable(
                                    candidate,
                                    &parents,
                                    quality.as_ref().map(|(_, q)| q),
                                ) && emit(Action::AnnounceMana {
                                    card: id,
                                    ability: index as u16,
                                    counter_payment: counters.clone(),
                                    production: produced,
                                })
                            },
                        )
                    },
                ) {
                    return true;
                }
            }
        }
        false
    }
    pub(super) fn visit_cost_payment(&self, emit: &mut impl FnMut(Action) -> bool) -> bool {
        let announcement = self.state.announcing.as_ref().unwrap();
        if announcement.cast_costs.as_ref().is_some_and(|c| {
            c.mana_closed
                && c.active
                    .is_none_or(|i| c.clauses[usize::from(i)].clause != CastCostClause::Mana)
        }) {
            return false;
        }
        let goal = self.funding_goal(announcement);
        let convoke = self.item_convoke(&announcement.item);
        let state = FundingState::from_game_with_convoke(self, &goal, convoke);
        let mut done = false;
        let mut direct = state.clone();
        direct.mark_mana_clause_paid();
        if state.obligations(&goal)
            && direct.cost_completion()
            && (announcement.cast_costs.is_some()
                || announcement.evidence.as_ref().is_none_or(|e| e.complete))
            && let Some(domain) = self.mana_payment_domain()
        {
            domain.representatives(|payment| {
                if !done && self.mana_parent_fundable(announcement.additional, Some(payment)) {
                    done = emit(Action::PayCosts { payment });
                }
            });
        }
        if done {
            return true;
        }
        // A public suppressed/linked provider may grant convoke only after an
        // additional cost. Let that cost be paid first; selecting the mana clause
        // still requires a complete payment with the grant actually available.
        let deferred_convoke = !convoke
            && announcement.item.kind == StackKind::Spell
            && announcement
                .item
                .source
                .is_some_and(|source| convoke::potential(self, source.card));
        if (deferred_convoke
            || delve::potential(self)
            || announcement
                .conspire
                .as_deref()
                .into_iter()
                .flatten()
                .any(|r| r.pay)
            || announcement.additional_cost().is_some_and(|s| {
                matches!(
                    s.kind,
                    EffectChoiceKind::BeholdMany(_) | EffectChoiceKind::BeholdAndExile
                )
            })
            || announcement
                .item
                .cast
                .is_some_and(|cast| cast.beheld == Some(true))
            || announcement
                .item
                .cast
                .is_some_and(|c| c.method.is_harmonize() || c.blight == Some(true))
            || announcement.ordered_craft()
            || announcement.nonmana_x()
            || announcement
                .additional_cost()
                .is_some_and(|s| matches!(s.kind.payment_kind(), EffectChoiceKind::BlightCost(_)))
            || announcement.has_compound_sneak_cost()
            || announcement
                .item
                .cast
                .is_some_and(|cast| cast.method == CastMethod::WebSlinging)
            || announcement
                .additional_cost()
                .is_some_and(|cost| cost.kind == EffectChoiceKind::ReturnTappedCreature)
            || announcement.self_graveyard_cast_cost().is_some()
            || announcement.has_ordered_life_cost()
            || announcement.linked_origin_counter_cost().is_some()
            || announcement
                .additional_cost()
                .is_some_and(|s| matches!(s.kind, EffectChoiceKind::DiscardOrLife(_))))
            && announcement
                .cast_costs
                .as_ref()
                .is_some_and(|c| !c.mana_closed)
            && state.cost_completion()
            && emit(Action::BeginOrderedCosts)
        {
            return true;
        }
        // Closing the mana window can precede a nonmana cost that restores
        // the grant. Individual artifact selections still require it live.
        let delving = self.item_delve(&announcement.item);
        let artifact = self.item_improvise(&announcement.item)
            || (announcement.item.kind == StackKind::Spell
                && announcement.cast_costs.is_some()
                && improvise::potential(
                    self,
                    announcement
                        .item
                        .source
                        .expect("cost announcement has object source")
                        .card,
                ));
        let waterbend = self.item_waterbend(&announcement.item) != 0;
        if (convoke || artifact || delving || waterbend)
            && (if state.has_ordered_convoke() {
                state.cost_completion()
            } else {
                convoke::payable_in(
                    &state,
                    goal.actor,
                    state.purpose,
                    mana_lots::available(state.mana, &state.lots, state.purpose),
                    goal.required,
                    goal.generic,
                    &state.convoke_masks(),
                )
            })
            && ((convoke && emit(Action::BeginConvoke))
                || (artifact && emit(Action::BeginImprovise))
                || (delving && emit(Action::BeginDelve))
                || (waterbend && emit(Action::BeginWaterbend)))
        {
            return true;
        }
        // This also gates deferred indexed activations, which are enumerated
        // outside FundingState::mana_moves. Printed mana components and source
        // presence do not decide whether the locked total requires mana.
        if !state.mana_window_open(&goal) {
            return false;
        }
        // Acquired abilities can appear in both the exact funding moves and
        // the deferred packet fallback. Share one announcement set across the
        // two visitors, while retaining distinct actions and fallback-only
        // routes. Ordinary direct mana actions need no allocation here.
        let mut announced = Vec::new();
        let mut emit_once = |action: Action| {
            if matches!(
                action,
                Action::AnnounceMana { .. }
                    | Action::AnnounceGrantedMana { .. }
                    | Action::AnnounceBorrowedMana { .. }
                    | Action::AnnounceGraveyardBorrowedMana { .. }
                    | Action::AnnounceSelfGrantedMana { .. }
            ) {
                if announced.contains(&action) {
                    return false;
                }
                announced.push(action.clone());
            }
            emit(action)
        };
        if state.visit_distinct_actions(
            &goal,
            |step| {
                state.visit_applied_for(step, Some(&goal), &mut |next| {
                    next.reachable_with_evidence(
                        &goal,
                        announcement.evidence.as_ref().map(|e| e.threshold),
                        &mut HashSet::new(),
                    )
                })
            },
            &mut emit_once,
        ) {
            return true;
        }
        self.visit_deferred_indexed_mana(&mut emit_once)
    }
}

#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
impl Game {
    pub(super) fn visit_face_up_payment(&self, emit: &mut impl FnMut(Action) -> bool) -> bool {
        let request = self.face_up_payment().unwrap();
        if self.object_ref(request.object.card) != request.object
            || !self.on_battlefield(request.object.card)
            || self.state.objects[request.object.card as usize].controller != request.controller
            || !self.is_face_down(request.object)
        {
            return false;
        }
        let goal = FundingGoal {
            reserve: None,
            actor: request.controller,
            required: request.cost.colored,
            generic: request.cost.generic,
            source: None,
        };
        let mut state = FundingState::from_game(self, &goal);
        state.set_purpose(mana_lots::Purpose::Other);
        if visit_funding_payments(
            mana_lots::available(state.mana, &state.lots, state.purpose),
            goal.required,
            goal.generic,
            |payment| {
                emit(Action::PayFaceUp {
                    payment: concrete_payment(payment),
                })
            },
        ) {
            return true;
        }
        state.visit_distinct_actions(
            &goal,
            |step| {
                state.visit_applied_for(step, Some(&goal), &mut |next| {
                    next.reachable(&goal, &mut HashSet::new())
                })
            },
            emit,
        )
    }
    pub(super) fn visit_plot_payment(&self, emit: &mut impl FnMut(Action) -> bool) -> bool {
        let request = self.plot_payment().unwrap();
        if self.object_ref(request.object.card) != request.object
            || self.state.objects[request.object.card as usize].zone != Zone::Hand
            || self.state.objects[request.object.card as usize].owner != request.controller
        {
            return false;
        }
        let goal = FundingGoal {
            reserve: None,
            actor: request.controller,
            required: request.cost.colored,
            generic: request.cost.generic,
            source: None,
        };
        let state = FundingState::from_game(self, &goal);
        if visit_funding_payments(
            mana_lots::available(state.mana, &state.lots, state.purpose),
            goal.required,
            goal.generic,
            |payment| {
                emit(Action::PayPlot {
                    payment: concrete_payment(payment),
                })
            },
        ) {
            return true;
        }
        state.visit_distinct_actions(
            &goal,
            |step| {
                state.visit_applied_for(step, Some(&goal), &mut |next| {
                    next.guided_mana_resume_fundable(&Pending::PlotPayment {
                        player: request.controller,
                    })
                })
            },
            emit,
        )
    }
    pub(super) fn project_resolution_life(&self, payer: Player, amount: i128) -> FundingState<'_> {
        let goal = FundingGoal {
            actor: payer,
            required: [0; 6],
            generic: 0,
            reserve: None,
            source: None,
        };
        let mut world = FundingState::from_game(self, &goal);
        world.pay_projected_life(payer, amount, &[]);
        world
    }

    pub(super) fn visit_resolution_payment(&self, emit: &mut impl FnMut(Action) -> bool) -> bool {
        let request = self
            .state
            .resolving
            .as_ref()
            .unwrap()
            .mana_payment
            .as_ref()
            .unwrap();
        if request.discard {
            let choice = self
                .state
                .resolving
                .as_ref()
                .unwrap()
                .choice
                .as_ref()
                .unwrap();
            for (id, _) in self.state.objects.iter().enumerate() {
                if self.effect_choice_candidate(
                    choice,
                    request.payer,
                    self.object_ref(id as CardId),
                ) && emit(Action::Discard { card: id as CardId })
                {
                    return true;
                }
            }
        }
        if emit(Action::DeclineResolutionMana) {
            return true;
        }
        if request.discard_only {
            return false;
        }
        if self.visit_resolution_cost_selections(request, emit) {
            return true;
        }
        let goal_for = |cost: ManaCost| FundingGoal {
            reserve: None,
            actor: request.payer,
            required: cost.colored,
            generic: cost.generic,
            source: None,
        };
        let costs = || std::iter::once(request.cost).chain(request.alternatives.iter().copied());
        let can_pay_life = self.resolution_life_available(request.payer, request.life);
        if !can_pay_life && request.choices.is_none() {
            return false;
        }
        if request.route_selected() && can_pay_life && self.resolution_objects_ready(request) {
            for goal in costs().map(goal_for) {
                let state = FundingState::from_game(self, &goal);
                if visit_funding_payments(
                    mana_lots::available(state.mana, &state.lots, state.purpose),
                    goal.required,
                    goal.generic,
                    |payment| {
                        emit(Action::PayResolutionMana {
                            payment: concrete_payment(payment),
                        })
                    },
                ) {
                    return true;
                }
            }
        }
        // An offered mana cost opens the same window before the final payment,
        // including while its nonmana selections are still incomplete.
        let goal = if let Some(choices) = &request.choices {
            choices
                .options
                .iter()
                .map(|choice| goal_for(choice.mana))
                .find(FundingGoal::requires_mana)
        } else {
            costs().map(goal_for).find(FundingGoal::requires_mana)
        };
        let Some(goal) = goal else {
            return false;
        };
        let mut state = FundingState::from_game(self, &goal);
        // An explicit resolution request opens its own window independently of
        // any suspended announcement's cost-payment phase.
        state.set_cost_context(None);
        state.visit_distinct_actions(&goal, |_| true, emit)
    }
}

#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
impl Game {
    /// A mana ability in an enclosing guided payment window must leave some
    /// completion of that window. Reuse its exact resource search, retaining
    /// complete plans so later scalar choices cannot mix incompatible prefixes.
    pub(super) fn prepare_mana_allocations(&mut self, action: &Action) {
        let payment = match action {
            Action::ActivateMana { payment, .. }
            | Action::ActivateBorrowedMana { payment, .. }
            | Action::ActivateGraveyardBorrowedMana { payment, .. }
            | Action::PayCosts { payment } => payment,
            _ => return,
        };
        let Some(actor) = self.semantic_actor() else {
            return;
        };
        if *payment == [0; 6]
            || self.mana_lots(actor).is_empty()
            || self
                .state
                .mana_lots
                .as_ref()
                .is_some_and(|state| state.replay.is_some())
        {
            return;
        }
        if matches!(action, Action::PayCosts { .. }) {
            let Some(a) = self
                .state
                .announcing
                .as_ref()
                .filter(|a| a.production.is_some())
            else {
                return;
            };
            let frames = self
                .state
                .mana_announcement_stack
                .as_deref()
                .map_or(&[][..], |frames| frames.as_slice());
            let terminal = a
                .mana_output
                .as_ref()
                .and_then(|o| o.resume.as_deref())
                .is_some_and(|pending| self.guided_mana_goal(pending).is_some());
            if frames.is_empty() && !terminal {
                return;
            }
            let goal = self.funding_goal(a);
            let mut world = FundingState::from_game(self, &goal);
            world.set_purpose(self.purpose_for_item(&a.item));
            world.set_payment_context(a);
            let quality = self
                .state
                .mana_lots
                .as_ref()
                .and_then(|m| m.announcement_quality.as_ref())
                .map(|(_, q)| q);
            world.set_funding_continuation(a, frames, quality);
            let mut viable = Vec::new();
            world.visit_funding_continuation_allocations(&goal, Some(*payment), &mut |plan| {
                let allocation = plan
                    .iter()
                    .map(|row| row.map(|n| u32::try_from(n).expect("concrete action payment")))
                    .collect::<Vec<_>>();
                if !viable.contains(&allocation) {
                    viable.push(allocation);
                }
                false
            });
            assert!(
                !viable.is_empty(),
                "validated mana payment has a completion"
            );
            self.state.mana_lots.as_mut().unwrap().prepared = Some(viable);
            return;
        }
        let (goal, convoke, evidence) = match self.state.pending {
            Pending::PayCosts { .. }
            | Pending::Convoke { .. }
            | Pending::Improvise { .. }
            | Pending::Delve { .. }
            | Pending::Waterbend { .. } => {
                let a = self.state.announcing.as_ref().unwrap();
                (
                    self.funding_goal(a),
                    a.item.kind == StackKind::Spell
                        && convoke::potential(
                            self,
                            a.item
                                .source
                                .expect("cost announcement has object source")
                                .card,
                        ),
                    a.evidence.as_ref().map(|e| e.threshold),
                )
            }
            Pending::AttackPayment { .. } => {
                let request = self.state.attack_payment.as_ref().unwrap();
                (
                    FundingGoal {
                        actor,
                        required: [0; 6],
                        generic: request.generic,
                        reserve: None,
                        source: None,
                    },
                    false,
                    None,
                )
            }
            Pending::PlotPayment { .. } => {
                let request = self.plot_payment().unwrap();
                (
                    FundingGoal {
                        actor,
                        required: request.cost.colored,
                        generic: request.cost.generic,
                        reserve: None,
                        source: None,
                    },
                    false,
                    None,
                )
            }
            Pending::FaceUpPayment { .. } => {
                let request = self.face_up_payment().unwrap();
                (
                    FundingGoal {
                        actor,
                        required: request.cost.colored,
                        generic: request.cost.generic,
                        reserve: None,
                        source: None,
                    },
                    false,
                    None,
                )
            }
            _ => return,
        };
        let world = FundingState::from_game_with_convoke(self, &goal, convoke);
        let mut viable = Vec::new();
        for step in world.action_moves(&goal) {
            if world.action(&step) != *action {
                continue;
            }
            if world.visit_applied_for(&step, Some(&goal), &mut |next| {
                next.reachable_with_evidence(&goal, evidence, &mut HashSet::new())
            }) {
                let allocation = step
                    .allocation
                    .iter()
                    .map(|row| row.map(|n| u32::try_from(n).expect("concrete action payment")))
                    .collect::<Vec<_>>();
                if !viable.contains(&allocation) {
                    viable.push(allocation)
                }
            }
        }
        assert!(
            !viable.is_empty(),
            "validated guided mana action has a completion"
        );
        self.state.mana_lots.as_mut().unwrap().prepared = Some(viable);
    }
}

#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
type CounterCostFundingKey = (FundingKey, Vec<(ObjectRef, CounterKind, u64, bool)>);

#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
impl FundingState<'_> {
    fn remove_cost_counter(&mut self, r: ObjectRef, kind: CounterKind, bearer_kinds: u16) -> bool {
        let Some(index) = self.resources.iter().position(|v| v.id == r.card) else {
            return false;
        };
        if !additional_costs::counter_cost_bearer(self, self.actor, r, bearer_kinds) {
            return false;
        }
        let mut counters =
            ability_world::CharacteristicWorld::counter_amounts(self, r.card).to_owned();
        if counters.get(kind) == 0 {
            return false;
        }
        counters.remove(kind, 1);
        self.resources[index].object.counters = counters.hot();
        live_counters::set(&mut self.named_counters, r, counters.named);
        if kind == CounterKind::Blight {
            self.expire_projected_blighted_land(r);
        }
        true
    }
    fn counter_payment_possible(
        &self,
        goal: &FundingGoal,
        requirement: CounterCostRequirement,
        selected: &[(ObjectRef, CounterKind)],
    ) -> bool {
        let CounterCostRequirement {
            amount,
            kind: required_kind,
            bearer_kinds,
        } = requirement;
        if selected.len() > usize::from(amount) {
            return false;
        }
        let mut after = self.clone();
        for &(r, kind) in selected {
            if !after.remove_cost_counter(r, kind, bearer_kinds) {
                return false;
            }
        }
        if !after.obligations(goal) {
            return false;
        }
        let reserved =
            goal.source
                .as_ref()
                .map_or(0u128, |(source, index, selected, definition, _)| {
                    if !additional_costs::counter_cost_bearer(
                        &after,
                        after.actor,
                        *source,
                        bearer_kinds,
                    ) {
                        return 0;
                    }
                    let loyalty = definition
                        .activation(*index)
                        .cost
                        .loyalty
                        .filter(|change| *change < 0)
                        .map_or(0, |change| u64::from(change.unsigned_abs()));
                    required_kind.map_or_else(
                        || selected.total() + u128::from(loyalty),
                        |kind| {
                            u128::from(selected.get(kind))
                                + if kind == CounterKind::Loyalty {
                                    u128::from(loyalty)
                                } else {
                                    0
                                }
                        },
                    )
                });
        let available = after
            .resources
            .iter()
            .filter(|r| {
                additional_costs::counter_cost_bearer(
                    &after,
                    self.actor,
                    after.reference(r.id),
                    bearer_kinds,
                )
            })
            .map(|r| {
                let counts = ability_world::CharacteristicWorld::counter_amounts(&after, r.id);
                required_kind.map_or_else(|| counts.total(), |kind| u128::from(counts.get(kind)))
            })
            .sum::<u128>();
        available.saturating_sub(reserved) >= u128::from(amount) - selected.len() as u128
    }
    /// A mana window consisting only of taps and mana production cannot
    /// increase the number of counters on permanents the payer controls.
    fn fixed_counter_cost_deficit(
        &self,
        requirement: CounterCostRequirement,
        selected: &[(ObjectRef, CounterKind)],
    ) -> bool {
        let CounterCostRequirement { amount, kind, .. } = requirement;
        let state = &self.game.state;
        let upper = state
            .objects
            .iter()
            .enumerate()
            .filter(|(_, object)| {
                object.zone == Zone::Battlefield && object.controller == self.actor
            })
            .map(|(id, _)| {
                // Include other types in case tapping changes a type layer.
                let counters = self.counter_amounts(id as CardId);
                kind.map_or_else(|| counters.total(), |kind| u128::from(counters.get(kind)))
            })
            .sum::<u128>();
        // A tap-only window cannot replenish an overdrawn exact bearer.
        let invalid_selection = selected
            .iter()
            .enumerate()
            .any(|(index, &(reference, kind))| {
                let object = self.object(reference.card);
                let previous = selected[..index]
                    .iter()
                    .filter(|&&choice| choice == (reference, kind))
                    .count();
                self.reference(reference.card) != reference
                    || object.zone != Zone::Battlefield
                    || object.controller != self.actor
                    || previous as u64 >= self.counter_amounts(reference.card).get(kind)
            });
        (upper < u128::from(amount) || invalid_selection) && self.fixed_counter_cost_inventory()
    }

    fn fixed_counter_cost_inventory(&self) -> bool {
        let state = &self.game.state;
        if self.continuation.is_some()
            || self.entry_projection.is_some()
            || self.controls().is_some()
            || state.temporary_controls.is_some()
            // A live linked source cannot leave through any route audited below.
            || state.exile_links.iter().any(|link| {
                self.reference(link.source.card) != link.source
                    || self.object(link.source.card).zone != Zone::Battlefield
            })
            || state.borrowed_links.is_some()
            || state.borrowed_slots.is_some()
            || state.graveyard_borrowing.is_some()
            || state.activation_observers.is_some()
            || state.temporary_abilities.is_some()
            || state.self_granted_abilities.is_some()
            || !self.fixed_copy_recipes()
            || self
                .roles()
                .is_some_and(|roles| !roles.source_linked_effects.is_empty())
        {
            return false;
        }
        for object in &state.objects {
            let card = object.card;
            // Include dormant definitions that could become available after
            // another source taps. Sacrifice grants and borrowed slots can
            // introduce effects outside the tap-only proof.
            if object.zone == Zone::Command
                || object.zone == Zone::Battlefield
                    && (card.borrows_activations()
                        || card.controls_attached()
                        || card.static_ability_groups().iter().any(|group| {
                            group.fragments.iter().any(|fragment| {
                                matches!(
                                    fragment,
                                    StaticAbilityFragment::AttachmentSacrificeMana(_)
                                )
                            })
                        }))
            {
                return false;
            }
            let payer_controls = match object.zone {
                Zone::Battlefield => object.controller == self.actor,
                Zone::Hand | Zone::Graveyard => object.owner == self.actor,
                _ => false,
            };
            if payer_controls
                && card
                    .activations()
                    .iter()
                    .enumerate()
                    .any(|(index, ability)| {
                        ability.source_zone() == object.zone
                            && ability.mana_production().is_some()
                            && (!Self::only_tap_mana(ability)
                                || card.activation_put_counter(index as u16).is_some())
                    })
            {
                return false;
            }
        }
        true
    }
    fn fund_counter_cost(
        &self,
        goal: &FundingGoal,
        requirement: CounterCostRequirement,
        selected: &[(ObjectRef, CounterKind)],
        visited: &mut HashSet<CounterCostFundingKey>,
    ) -> bool {
        if !self.obligations(goal) {
            return false;
        }
        // Mana abilities are activated before paying this nonmana cost. The
        // chosen counters remain reserved, rather than being removed before
        // counter-dependent mana production has been evaluated.
        let purpose = self.payment_purpose(goal);
        let adjusted = cast_origins::payment_goal(self, goal, purpose);
        if payable(
            mana_lots::available(self.mana, &self.lots, purpose),
            adjusted.required,
            adjusted.generic,
        ) && self.counter_payment_possible(goal, requirement, selected)
        {
            return true;
        }
        if !self.optimistic(goal) {
            return false;
        }
        let facts = selected
            .iter()
            .map(|&(r, kind)| {
                (
                    r,
                    kind,
                    ability_world::CharacteristicWorld::counter_count(self, r.card, kind),
                    additional_costs::counter_cost_bearer(
                        self,
                        self.actor,
                        r,
                        requirement.bearer_kinds,
                    ),
                )
            })
            .collect();
        if !visited.insert((self.key(goal), facts)) {
            return false;
        }
        for deferred in [false, true] {
            let moves = if deferred {
                self.deferred_moves(goal)
            } else {
                self.moves(goal)
            };
            for step in moves {
                if self.visit_applied_for(&step, Some(goal), &mut |next| {
                    next.fund_counter_cost(goal, requirement, selected, visited)
                }) {
                    return true;
                }
            }
        }
        false
    }
}
#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
impl Game {
    pub(super) fn current_counter_cost_payable(&self, announcement: &Announcement) -> bool {
        let goal = self.funding_goal(announcement);
        let requirement = announcement
            .counter_cost_spec()
            .and_then(|cost| cost.kind.counter_requirement())
            .expect("distributed counter cost");
        let selected = announcement.creature_counter_cost.as_deref();
        // A selectable prefix can depend on future mana abilities. Only
        // keep selecting now if the complete payment fits the live inventory.
        FundingState::from_game(self, &goal).counter_payment_possible(
            &goal,
            requirement,
            selected.map_or(&[], Vec::as_slice),
        )
    }

    pub(super) fn optional_counter_cost_mana(&self, announcement: &Announcement) -> bool {
        if announcement.cast_costs.is_some()
            || !matches!(announcement.item.kind, StackKind::ActivatedAbility { .. })
        {
            return false;
        }
        let goal = self.funding_goal(announcement);
        goal.requires_mana() && !FundingState::from_game(self, &goal).fixed_counter_cost_inventory()
    }

    pub(super) fn counter_cost_fundable(
        &self,
        goal: FundingGoal,
        purpose: mana_lots::Purpose,
        spec: AdditionalCost,
        selected: &[(ObjectRef, CounterKind)],
    ) -> bool {
        let requirement = spec.kind.counter_requirement().unwrap();
        if selected
            .iter()
            .any(|&(_, kind)| requirement.kind.is_some_and(|required| kind != required))
        {
            return false;
        }
        if self
            .state
            .announcing
            .as_ref()
            .is_some_and(|a| a.cast_costs.as_ref().is_some_and(|c| c.mana_closed))
        {
            return self.cast_cost_counters_fundable(selected);
        }
        if self
            .state
            .announcing
            .as_ref()
            .is_some_and(|a| a.cast_costs.is_some())
        {
            if let Some(result) = self.preview_optional_cast_costs(
                goal.clone(),
                purpose,
                ObjectRef {
                    card: CardId::MAX,
                    generation: 0,
                },
                Some(spec),
                None,
                &[],
                false,
                selected,
                &[],
            ) {
                return result;
            }
            let mut world = FundingState::from_game(self, &goal);
            world.set_purpose(purpose);
            world.set_cost_counter_prefix(selected);
            return world.reachable(&goal, &mut HashSet::new());
        }
        let mut world = FundingState::from_game_with_convoke(self, &goal, true);
        world.convoke = false;
        world.set_purpose(purpose);
        if world.fixed_counter_cost_deficit(requirement, selected) {
            return false;
        }
        world.fund_counter_cost(&goal, requirement, selected, &mut HashSet::new())
    }
}

#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
impl Game {
    pub(super) fn attack_tax_fundable(&self, player: Player, generic: u32) -> bool {
        let goal = FundingGoal {
            reserve: None,
            actor: player,
            required: [0; 6],
            generic,
            source: None,
        };
        FundingState::from_game(self, &goal).reachable(&goal, &mut HashSet::new())
    }
    pub(super) fn visit_attack_tax_payment(&self, emit: &mut impl FnMut(Action) -> bool) -> bool {
        let payment = self.state.attack_payment.as_ref().unwrap();
        let goal = FundingGoal {
            reserve: None,
            actor: payment.player,
            required: [0; 6],
            generic: payment.generic,
            source: None,
        };
        let state = FundingState::from_game(self, &goal);
        if visit_funding_payments(
            mana_lots::available(state.mana, &state.lots, state.purpose),
            goal.required,
            goal.generic,
            |payment| {
                emit(Action::PayAttackCost {
                    payment: concrete_payment(payment),
                })
            },
        ) {
            return true;
        }
        state.visit_distinct_actions(
            &goal,
            |step| {
                state.visit_applied_for(step, Some(&goal), &mut |next| {
                    next.reachable(&goal, &mut HashSet::new())
                })
            },
            emit,
        )
    }
}

#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
impl Game {
    pub(super) fn self_graveyard_announcement_fundable(
        &self,
        a: &Announcement,
        cost: ManaCost,
    ) -> bool {
        let source = a.item.source.expect("graveyard route spell source");
        let base = Self::with_kicker_cost(
            Self::with_offspring_cost(
                cost,
                source.card,
                a.item.cast.is_some_and(|c| c.offspring == Some(true)),
            ),
            source.card,
            a.item.cast.is_some_and(|c| c.kicker == Some(true)),
        );
        let Some(total) =
            self.spell_total_for_definition(a.item.controller, source.id, base, source.card)
        else {
            return false;
        };
        let mut proposed = a.clone();
        proposed.cost = Some(cost);
        proposed.total_cost = Some(total);
        proposed.initialize_cast_costs(convoke::cost_potential(self, source.card));
        let goal = self.funding_goal(&proposed);
        let mut world = FundingState::from_game_with_convoke(
            self,
            &goal,
            convoke::potential(self, source.card),
        );
        world.set_purpose(self.purpose_for_item(&proposed.item));
        world.set_payment_context(&proposed);
        world.reachable(&goal, &mut HashSet::new())
    }
}

#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
impl FundingState<'_> {
    /// This paid child will commit before its parent resumes. Carry only that
    /// activation's use receipt into the virtual completion world; its costs
    /// and output domain have already been evaluated in the post-cost world.
    fn project_paid_mana_use(&mut self, item: &SpellView) {
        if item.is_copy {
            return;
        }
        let Some(key) = item.mana_activation_key() else {
            return;
        };
        let rules = key.definition.activation(key.ability);
        if rules.exhaust {
            // The activation event survives departure of its exact source.
            self.exhaust_activated[item.controller.index()] = true;
        }
        let Some(index) = self.resources.iter().position(|r| {
            r.id == key.source.card
                && r.object.generation == key.source.generation
                && r.object.zone == rules.source_zone()
        }) else {
            return;
        };
        if let Some(timestamp) = key.self_grant {
            if rules.exhaust || rules.once_per_turn {
                self.self_grant_uses.push((key.source, timestamp));
            }
        } else if let Some(slot) = key.borrowed_slot {
            self.borrowed_uses.push((key.source, slot));
        } else if !item.is_retained_copy_activation()
            && self.definition(key.source.card) == key.definition
            && self.ability_origin(key.source.card) == key.source_origin
        {
            self.resources[index].object.used_abilities |=
                1u64.checked_shl(u32::from(key.ability)).unwrap_or(0);
        }
    }

    /// Costs and entry choices already belong to this world. Add only the
    /// actual post-cost receipt output and completed use; never replay payment.
    pub(super) fn visit_paid_mana_output(
        &self,
        a: &Announcement,
        selected: Option<Mana>,
        emit: &mut dyn FnMut(&Self) -> bool,
    ) -> bool {
        let bonus = a.item.tap_mana_bonus(self);
        let quality = a.mana_output.as_ref().map_or_else(
            || {
                self.game
                    .state
                    .mana_lots
                    .as_ref()
                    .and_then(|m| m.announcement_quality)
                    .map(|(_, q)| q)
            },
            |o| o.quality,
        );
        let mut finish = |production| {
            if selected.is_some_and(|chosen| chosen != production) {
                return false;
            }
            let Some(replaced) = bonus.replace_production(production) else {
                return false;
            };
            bonus.visit(production, |bonus| {
                if !self.completed_mana_fits(replaced, bonus) {
                    return false;
                }
                let mut next = self.clone();
                next.add_completed_mana(replaced, quality, bonus);
                if let Some(output) = &a.mana_output {
                    next.project_mana_color_change(output, production);
                }
                next.project_paid_mana_use(&a.item);
                emit(&next)
            })
        };
        if let Some(output) = &a.mana_output {
            let power = a.item.source.expect("mana source").facts.power;
            if let Some(split) = output.unselected_split(self, power) {
                if let Some(production) = selected {
                    return split.contains(production) && finish(production);
                }
                return self.visit_projected_mana_split(
                    split,
                    self.game.mana_output_parent_goal().as_ref(),
                    quality,
                    bonus,
                    finish,
                );
            }
            output.visit(self, power, finish)
        } else if let Some(production) = a.production {
            finish(production)
        } else {
            emit(self)
        }
    }
    pub(super) fn paid_mana_parent_fundable(&self, a: &Announcement) -> bool {
        if let Some(parent) = self
            .game
            .state
            .mana_announcement_stack
            .as_deref()
            .and_then(|frames| frames.last())
        {
            let goal = self.game.funding_goal(&parent.announcement);
            self.reachable_with_evidence(
                &goal,
                parent.announcement.evidence.as_ref().map(|e| e.threshold),
                &mut HashSet::new(),
            )
        } else {
            a.mana_output
                .as_ref()
                .and_then(|o| o.resume.as_deref())
                .is_none_or(|pending| self.guided_mana_resume_fundable(pending))
        }
    }
    pub(super) fn guided_mana_resume_fundable(&self, pending: &Pending) -> bool {
        let Some(goal) = self.game.guided_mana_goal(pending) else {
            return true;
        };
        let mut next = self.clone();
        next.set_purpose(mana_lots::Purpose::Other);
        next.set_cost_context(None);
        next.payment_source = None;
        next.cost_group = None;
        next.convoke = false;
        next.waterbend = 0;
        next.continuation = None;
        next.continuation_memo = None;
        // Reaching a non-announcement resume completed every mana frame. Only
        // the now-finished chain is retired; a repeatable converter may run anew.
        next.active_mana_keys = Some(Vec::new());
        if let Pending::FaceUpPayment { .. } = pending {
            next.required_face_up = self.game.face_up_payment().map(|r| r.object);
        }
        next.reachable_with_completion(&goal, None, &mut HashSet::new(), &mut |world| {
            let valid = match pending {
                Pending::PlotPayment { .. } => world.game.plot_payment().is_some_and(|r| {
                    world.reference(r.object.card) == r.object
                        && world.object(r.object.card).zone == Zone::Hand
                        && world.object(r.object.card).owner == r.controller
                }),
                _ => true,
            };
            valid && world.cost_completion()
        })
    }
}
#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
impl Game {
    pub(super) fn guided_mana_goal(&self, pending: &Pending) -> Option<FundingGoal> {
        let (actor, required, generic) = match pending {
            Pending::PlotPayment { .. } => {
                let r = self.plot_payment()?;
                (r.controller, r.cost.colored, r.cost.generic)
            }
            Pending::FaceUpPayment { .. } => {
                let r = self.face_up_payment()?;
                (r.controller, r.cost.colored, r.cost.generic)
            }
            Pending::AttackPayment { .. } => {
                let r = self.state.attack_payment.as_ref()?;
                (r.player, [0; 6], r.generic)
            }
            _ => return None,
        };
        Some(FundingGoal {
            actor,
            required,
            generic,
            reserve: None,
            source: None,
        })
    }
    pub(super) fn paid_mana_parent_world<'g>(&'g self, a: &'g Announcement) -> FundingState<'g> {
        if let Some((parent, outer)) = self
            .state
            .mana_announcement_stack
            .as_deref()
            .and_then(|frames| frames.split_last())
        {
            let goal = self.funding_goal(&parent.announcement);
            let mut world = FundingState::from_game_with_convoke(
                self,
                &goal,
                parent.announcement.item.kind == StackKind::Spell
                    && convoke::potential(
                        self,
                        parent.announcement.item.source.expect("parent source").card,
                    ),
            );
            world.set_purpose(self.purpose_for_item(&parent.announcement.item));
            world.set_payment_context(&parent.announcement);
            world.set_funding_continuation(
                &parent.announcement,
                outer,
                parent.quality.as_ref().map(|(_, q)| q),
            );
            world
        } else {
            let goal = a
                .mana_output
                .as_ref()
                .and_then(|o| o.resume.as_deref())
                .and_then(|pending| self.guided_mana_goal(pending))
                .unwrap_or(FundingGoal {
                    actor: a.item.controller,
                    required: [0; 6],
                    generic: 0,
                    reserve: None,
                    source: None,
                });
            let mut world = FundingState::from_game(self, &goal);
            world.set_purpose(mana_lots::Purpose::Other);
            world.set_cost_context(None);
            world.payment_source = None;
            world.cost_group = None;
            world.convoke = false;
            world.waterbend = 0;
            world.continuation = None;
            world.continuation_memo = None;
            world.active_mana_keys = Some(Vec::new());
            world
        }
    }
    pub(super) fn mana_output_parent_goal(&self) -> Option<FundingGoal> {
        let a = self.state.announcing.as_ref()?;
        let output = a.mana_output.as_ref()?;
        self.state
            .mana_announcement_stack
            .as_deref()
            .and_then(|frames| frames.last())
            .map(|frame| self.funding_goal(&frame.announcement))
            .or_else(|| {
                output
                    .resume
                    .as_deref()
                    .and_then(|pending| self.guided_mana_goal(pending))
            })
    }

    pub(super) fn mana_split_output_may_fund(&self, upper: Mana) -> bool {
        let Some(goal) = self.mana_output_parent_goal() else {
            return true;
        };
        let a = self.state.announcing.as_ref().expect("paid split mana");
        let output = a.mana_output.as_ref().expect("paid split output");
        let mut world = self.paid_mana_parent_world(a);
        let receipt = a.item.tap_mana_bonus(&world);
        if receipt.has_trigger_bonuses() {
            return true;
        }
        let Some(replaced) = receipt.replace_production(upper) else {
            return true;
        };
        world.add_completed_mana(replaced, output.quality, [0; 6]);
        world.project_paid_mana_use(&a.item);
        world.optimistic(&goal)
    }

    pub(super) fn mana_output_choice_fundable(&self, production: Mana) -> bool {
        let a = self
            .state
            .announcing
            .as_ref()
            .expect("owned mana announcement");
        // Free and optional resumes have no parent completion to constrain.
        if self.state.mana_announcement_stack.is_none()
            && a.mana_output
                .as_ref()
                .and_then(|o| o.resume.as_deref())
                .is_none_or(|pending| self.guided_mana_goal(pending).is_none())
        {
            return true;
        }
        self.paid_mana_parent_world(a)
            .visit_paid_mana_output(a, Some(production), &mut |next| {
                next.paid_mana_parent_fundable(a)
            })
    }
}

#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
impl Game {
    pub(super) fn tap_bonus_choice_fundable(&self, bonus: Mana) -> bool {
        let choice = self
            .state
            .mana_lots
            .as_ref()
            .unwrap()
            .tap_bonus
            .as_ref()
            .unwrap();
        if let Some(a) = &self.state.announcing {
            let goal = self.funding_goal(a);
            let mut world = FundingState::from_game_with_convoke(
                self,
                &goal,
                a.item.kind == StackKind::Spell
                    && convoke::potential(self, a.item.source.unwrap().card),
            );
            world.set_purpose(self.purpose_for_item(&a.item));
            world.set_payment_context(a);
            world.set_funding_continuation(
                a,
                self.state
                    .mana_announcement_stack
                    .as_deref()
                    .map_or(&[], Vec::as_slice),
                self.state
                    .mana_lots
                    .as_ref()
                    .and_then(|m| m.announcement_quality.as_ref().map(|(_, q)| q)),
            );
            world.add_completed_mana([0; 6], None, bonus);
            world.reachable_with_evidence(
                &goal,
                a.evidence.as_ref().map(|e| e.threshold),
                &mut HashSet::new(),
            )
        } else if let Some(goal) = choice
            .previous
            .as_ref()
            .and_then(|pending| self.guided_mana_goal(pending))
        {
            let mut world = FundingState::from_game(self, &goal);
            world.add_completed_mana([0; 6], None, bonus);
            world.reachable_with_evidence(&goal, None, &mut HashSet::new())
        } else {
            true
        }
    }
}
