//! Exact reachability for the admitted pure mana sources; no game-state clones,
//! heuristics that discard legal plans, or mutation during action generation.
use super::*;
use crate::cards::StaticAbilityFragment;
use ability_world::CharacteristicWorld;
use std::collections::HashSet;
#[doc(hidden)]
#[path = "engine_cast_cost_planning.rs"]
pub mod cast_cost_planning;
use cast_cost_planning::CostContext;
#[doc(hidden)]
#[path = "engine_funding_entries.rs"]
pub mod funding_entries;
use funding_entries::{EntryProjection, PaymentContinuation};
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
#[derive(Clone)]
struct Resource {
    id: CardId,
    attacking: bool,
    object: Object,
}

#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
fn mana_loyalty_placement_payable<W: CharacteristicWorld + ?Sized>(
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
    active: Player,
    mana: [u128; 6],
    lots: Vec<mana_lots::Lot>,
    purpose: mana_lots::Purpose,
    payment_source: Option<ObjectRef>,
    exhaust_source_reuse_authorized: bool,
    required_face_up: Option<ObjectRef>,
    // Sparse last-known facts for exact enclosing sources that leave a speculative world.
    payment_departures: Vec<(ObjectRef, mana_lots::Source)>,
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
    borrowed_uses: Vec<(ObjectRef, u64)>,
    graveyard_borrowing: graveyard_borrowing::Rows,
    borrowed_slots: borrowed_activations::SlotRows,
}
#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
impl ability_world::CharacteristicWorld for FundingState<'_> {
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
    fn temporary_copy_view(&self, object: ObjectRef) -> Option<TemporaryCopyView> {
        let mut view = self.game.temporary_copy_view(object)?;
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
            .is_some_and(|resource| resource.attacking && resource.object.zone == Zone::Battlefield)
    }
}

/// An exact, still-live permanent/card selected for a mana activation cost.
#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
#[derive(Clone, Copy)]
struct ExternalManaCost {
    resource: usize,
    object: ObjectRef,
    kind: EffectChoiceKind,
}
#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
impl ExternalManaCost {
    fn destination(self) -> Zone {
        match self.kind.payment_kind() {
            EffectChoiceKind::Sacrifice => Zone::Graveyard,
            EffectChoiceKind::ExileGraveyard => Zone::Exile,
            _ => unreachable!("unsupported selected mana cost"),
        }
    }
}

#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
#[derive(Clone)]
struct FundingMove {
    resource: usize,
    ability: Option<u16>,
    grantor: Option<ObjectRef>,
    borrowed: Option<BorrowedDescriptor>,
    payment: [u64; 6],
    counters: Counters,
    production: Mana,
    quality: Option<mana_lots::Quality>,
    allocation: Vec<[u64; 6]>,
    external: Option<ExternalManaCost>,
    /// A previously announced activation keeps its immutable ability definition.
    frozen: Option<Box<(Card, Option<u64>)>>,
}
#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
#[derive(Hash, PartialEq, Eq)]
struct ResourceKey {
    definition: usize,
    token: bool,
    card: bool,
    kinds: u16,
    protected: bool,
    reserved: bool,
    zone: u8,
    tapped: bool,
    sick: bool,
    entered: bool,
    counters: InitialCounters,
    used: u64,
    loyalty_activated_this_turn: bool,
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
    battle_protectors: Vec<(ObjectRef, usize)>,
    values: Vec<(ObjectRef, EntryChoiceValue)>,
    consumed: Vec<ObjectRef>,
}
#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
#[derive(Hash, PartialEq, Eq)]
struct FundingKey {
    life_lost: [bool; 2],
    enduring_story: [bool; 2],
    exhaust_activated: [bool; 2],
    payment_departures: Vec<(ObjectRef, mana_lots::Source)>,
    source_linked_effects: Vec<SourceLinkedEffectView>,
    resolved_ability_modifications: Vec<ResolvedAbilityModificationView>,
    entries: Option<EntryFundingKey>,
    histories: Option<(u64, [bool; 2], i128)>,
    controlled_creature_deaths: [u64; 2],
    borrowed_uses: Vec<(ObjectRef, u64)>,
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
    discount: u32,
    mut emit: impl FnMut([u64; 6]),
) {
    let mut seen = std::collections::HashSet::new();
    for equivalent in cost.mana_options() {
        funding_payments(
            pool,
            equivalent.colored,
            equivalent.generic.saturating_sub(discount),
            |payment| {
                if cost.alternatives.is_empty() || seen.insert(payment) {
                    emit(payment);
                }
            },
        );
    }
}
#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
fn concrete_payment(payment: [u64; 6]) -> Mana {
    payment.map(|n| u32::try_from(n).capacity("mana payment exhausted"))
}
#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
fn reserve_obligations(
    goal: &FundingGoal,
    object: Option<&Object>,
    kinds: u16,
    keywords: Keywords,
    subtypes: GrantedSubtypes,
    name: &str,
) -> bool {
    goal.reserve.is_none_or(|(reference, spec)| {
        object.is_some_and(|o| {
            o.generation == reference.generation
                && (spec.kind.payment_kind() != EffectChoiceKind::SacrificeAttachedEquipment
                    || goal
                        .source
                        .as_ref()
                        .is_some_and(|(source, _, _, _, _)| o.attached_to == Some(*source)))
                && Game::additional_object_matches(
                    spec, goal.actor, o, kinds, keywords, subtypes, name,
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
fn source_obligations(
    goal: &FundingGoal,
    object: Option<&Object>,
    amounts: CounterAmounts<'_>,
    kinds: u16,
    keywords: impl FnOnce() -> Keywords,
    life: i128,
    exhaust_reuse_authorized: bool,
) -> bool {
    let Some((source, index, counters, definition, slot)) = goal.source.as_ref() else {
        return true;
    };
    let mut readiness = *definition.activation(*index);
    if exhaust_reuse_authorized {
        readiness.exhaust = false;
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
    fn potential_land_mana(&self) -> impl Iterator<Item = ManaProduction> + '_ {
        self.static_sources()
            .iter()
            .filter_map(|&id| {
                let object = ability_world::CharacteristicWorld::object(self, id);
                (object.zone == Zone::Battlefield).then_some(object.card)
            })
            .flat_map(|card| card.static_ability_groups())
            .flat_map(|group| group.fragments)
            .filter_map(|fragment| match *fragment {
                crate::cards::StaticAbilityFragment::LandMana { production, .. } => {
                    Some(production)
                }
                _ => None,
            })
    }
    fn power_upper_production(&self, production: ManaProduction, source: CardId) -> ManaProduction {
        match production.base() {
            ManaProduction::TapSourcePowerChoice { mask } => ManaProduction::Choice {
                mask,
                amount: u32::MAX,
            },
            ManaProduction::DistinctCreaturePowers => ManaProduction::any_color(u32::MAX),
            ManaProduction::SourcePower { color } => {
                let mut mana = [0; 6];
                mana[usize::from(color)] = u32::MAX;
                ManaProduction::Fixed(mana)
            }
            _ => production.in_world(self, source),
        }
    }

    fn set_purpose(&mut self, purpose: mana_lots::Purpose) {
        self.purpose = purpose;
        let needs_improvise_resources = matches!(purpose, mana_lots::Purpose::Spell(spell) if improvise::potential(self, spell.card));
        if self.waterbend == 0 && !needs_improvise_resources {
            return;
        }
        for (id, _) in self.game.state.objects.iter().enumerate() {
            let id = id as CardId;
            if self.resources.iter().any(|r| r.id == id) {
                continue;
            }
            let object = ability_world::CharacteristicWorld::object(self, id);
            if object.zone == Zone::Battlefield {
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
    fn has_contributions(&self) -> bool {
        self.convoke || self.improvise_available() || self.waterbend != 0
    }
    fn from_game(game: &'g Game, goal: &FundingGoal) -> Self {
        Self::from_game_with_convoke(game, goal, false)
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
        let recipe = self.game.temporary_copy_row(before).map_or_else(
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
        let mut subtypes = Vec::new();
        let mut graveyard_mana_cost = false;
        let mut sacrifice_mana_cost = false;
        let mut power_sensitive = false;
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
            if object.zone == Zone::Battlefield && object.controller == goal.actor {
                let inherited = borrowed_activations::abilities(game, id as CardId);
                for ability in object
                    .card
                    .activations()
                    .iter()
                    .chain(inherited.into_iter().map(|a| a.rules()))
                {
                    if let Some(production) = ability.mana_production() {
                        loyalty_threshold_sensitive |= matches!(
                            ability.guard,
                            ActivationGuard::ControlledPlaneswalkerSubtypeLoyaltyAtLeast { .. }
                        );
                        power_sensitive |= matches!(
                            production.base(),
                            ManaProduction::SourcePower { .. }
                                | ManaProduction::TapSourcePowerChoice { .. }
                                | ManaProduction::DistinctCreaturePowers
                        );
                        sacrifice_mana_cost |= ability.cost.additional.is_some_and(|cost| {
                            cost.kind.payment_kind() == EffectChoiceKind::Sacrifice
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
        let resources = game
            .state
            .objects
            .iter()
            .enumerate()
            .filter(|(id, object)| {
                (life_replacement_possible && object.zone == Zone::Library && object.is_card())
                    || (graveyard_mana_cost && object.zone == Zone::Graveyard && object.owner == goal.actor && object.is_card())
                    || (object.zone == Zone::Command && object.card.is_emblem()) || goal.reserve
                    .is_some_and(|(reference, _)| reference.card == *id as CardId)
                    || goal
                        .source.as_ref()
                        .is_some_and(|(source, _, _, _, _)| source.card == *id as CardId)
                    || object.zone == Zone::Battlefield
                        && game.state.linked_choices.as_ref().is_some_and(|data|
                            data.null_cast_links.iter().any(|link| link.source.card == *id as CardId))
                    || object.zone == Zone::Battlefield
                        && (life_replacement_possible || loyalty_threshold_sensitive || sacrifice_mana_cost || game.sacrifice_prohibition_source(*id as CardId) || zone_replacements::card_provides_prevention(object.card) || {
                                #[cfg(feature = "test-support")] { zone_replacements::TEST_PREVENT_PROVIDER.with(|p| p.get() == Some(game.object_ref(*id as CardId))) }
                                #[cfg(not(feature = "test-support"))] { false }
                            } || game.state.controls.is_some() || object.card.has_cast_origin_abilities() || object.card.borrows_activations()
                            || object.card.attachment_bonus().is_some()
                            || game.state.attachment_sources.iter().any(|&provider| game.state.objects[provider as usize].attached_to == Some(game.object_ref(*id as CardId)))
                            || !object.card.keyword_grants().is_empty()
                            || !object.card.counter_keyword_grants().is_empty()
                            || object.card.static_ability_groups().iter().any(|group| group.fragments.iter().any(|fragment| matches!(fragment, crate::cards::StaticAbilityFragment::ActivateControlledCreaturesAsHaste | crate::cards::StaticAbilityFragment::ReuseExhaustBeforeFirstActivationOwnTurn | crate::cards::StaticAbilityFragment::OtherControlledExhaustGenericReduction(_) | crate::cards::StaticAbilityFragment::ControlledPermanentKeywords { .. } | crate::cards::StaticAbilityFragment::ControlledSubtypeKeywords { .. } | crate::cards::StaticAbilityFragment::LandMana { .. } | crate::cards::StaticAbilityFragment::ChosenCreatureType | crate::cards::StaticAbilityFragment::ControlledCreatureSubtype { .. } | crate::cards::StaticAbilityFragment::SourceOffTurnCreature | crate::cards::StaticAbilityFragment::CreatureGroupBonus { .. } | crate::cards::StaticAbilityFragment::ChosenBasicLandType | crate::cards::StaticAbilityFragment::ChosenModifier(..) | crate::cards::StaticAbilityFragment::DeathBaronLord | crate::cards::StaticAbilityFragment::SourceCounterSubtype { .. } | crate::cards::StaticAbilityFragment::SourceCounterCreature { .. } | crate::cards::StaticAbilityFragment::SourceGraveyardAllTypes { .. } | crate::cards::StaticAbilityFragment::SourceConditionalKeywords { .. } | crate::cards::StaticAbilityFragment::SourceCounterKeywords { .. } | crate::cards::StaticAbilityFragment::SourcePlusOnePlacedKeywords(..) | crate::cards::StaticAbilityFragment::PlusOnePlacedKeywords { .. } | crate::cards::StaticAbilityFragment::CounterKeywordOwnTurn(..) | crate::cards::StaticAbilityFragment::CounterKeywordOther(..))))
                            || object.controller == goal.actor
                                && subtypes
                                    .iter()
                                    .any(|subtype| game.has_subtype(*id as CardId, subtype))
                            || (object.controller == goal.actor
                                && ((convoke
                                    && game.object_kinds(*id as CardId, object) & 1 != 0)
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
            exhaust_source_reuse_authorized: game.exhaust_reuse_authorized_for_goal(goal),
            copy_recipes: game.state.copy_recipes.clone(),
            cast_costs: None,
            entry_projection: None,
            continuation: None,
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
            power_sensitive,
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
            purpose: game.funding_purpose(goal),
            payment_source: goal.source.as_ref().map(|(source, ..)| *source),
            required_face_up: game.face_up_payment().map(|request| request.object),
            payment_departures: Vec::new(),
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
        if let Some(a) = game.state.announcing.as_ref()
            && !a.additional_paid
            && a.additional_cost().is_some_and(|s| {
                evidence::threshold(s).is_some()
                    || matches!(
                        s.kind,
                        EffectChoiceKind::ExileGraveyardSymbols { .. }
                            | EffectChoiceKind::RemoveCreatureCounters(_)
                    )
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
            && let Some(frames) = game.state.mana_announcement_stack.as_deref()
        {
            result.set_funding_continuation(
                a,
                frames.as_slice(),
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
                                | EffectChoiceKind::SacrificeAttachedEquipment
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
                        | EffectChoiceKind::SacrificeAttachedEquipment
                ) || !self.sacrifice_prohibited(r.card))
                    && (spec.kind.payment_kind() != EffectChoiceKind::ReturnUnblockedAttacker
                        || self.game.ninjutsu_attacker(goal.actor, r)
                            && ability_world::CharacteristicWorld::attacking(self, r.card))
                    && (spec.kind.payment_kind() != EffectChoiceKind::DiscardLegendary
                        || ability_world::CharacteristicWorld::legendary(self, r.card))
                    && (spec.kind.payment_kind() != EffectChoiceKind::DiscardMatchingLegendary
                        || additional_costs::legendary_name_match(self, goal.actor, r.card))
            })
            && reserve_obligations(
                goal,
                reserve.map(|r| &r.object),
                reserve.map_or(0, |r| self.kinds(r)),
                reserve.map_or(Keywords::NONE, |r| self.keywords(r)),
                reserve.map_or(GrantedSubtypes::default(), |r| self.granted_subtypes(r.id)),
                reserve.map_or("", |r| ability_world::CharacteristicWorld::name(self, r.id)),
            )
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
    ) -> impl Iterator<Item = (u16, &'static ActivatedAbility, Option<BorrowedDescriptor>)> {
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
            .map(|(index, ability)| (index as u16, ability, None))
            .chain(
                borrowed_activations::abilities(self, id)
                    .into_iter()
                    .filter(|ability| ability.rules().mana_production().is_some())
                    .map(|ability| (ability.ability, ability.rules(), Some(ability))),
            )
    }
    fn moves(&self, goal: &FundingGoal) -> Vec<FundingMove> {
        let mut moves = Vec::new();
        for (resource, entry) in self.resources.iter().enumerate() {
            let object = &entry.object;
            if object.zone != Zone::Battlefield || object.controller != goal.actor {
                continue;
            }
            if ability_world::CharacteristicWorld::activation_prohibited(self, goal.actor, entry.id)
            {
                continue;
            }
            if !object.tapped
                && (self.kinds(entry) & 1 == 0
                    || !object.summoning_sick
                    || ability_world::CharacteristicWorld::activation_keywords(self, entry.id)
                        .contains(Keywords::HASTE))
            {
                for grant in ability_world::CharacteristicWorld::granted_mana(self, entry.id) {
                    if grant.sacrifice && self.sacrifice_prohibited(entry.id) {
                        continue;
                    }
                    let quality = mana_lots::quality_from_world(self, grant.production, entry.id);
                    grant
                        .production
                        .in_world(self, entry.id)
                        .visit(|production| {
                            moves.push(FundingMove {
                                resource,
                                ability: None,
                                grantor: Some(grant.source),
                                borrowed: None,
                                payment: [0; 6],
                                counters: Counters::default(),
                                production,
                                quality,
                                allocation: Vec::new(),
                                external: None,
                                frozen: None,
                            });
                            false
                        });
                }
            }
            if ability_world::CharacteristicWorld::printed_abilities_available(self, entry.id)
                && !object.tapped
                && (self.kinds(entry) & 1 == 0
                    || !object.summoning_sick
                    || ability_world::CharacteristicWorld::activation_keywords(self, entry.id)
                        .contains(Keywords::HASTE))
                && let Some(color) =
                    ability_world::CharacteristicWorld::intrinsic_mana_color(self, entry.id)
            {
                let mut production = [0; 6];
                production[color] = 1;
                moves.push(FundingMove {
                    resource,
                    ability: None,
                    grantor: None,
                    borrowed: None,
                    payment: [0; 6],
                    counters: Counters::default(),
                    production,
                    quality: mana_lots::quality_from_world(
                        self,
                        ManaProduction::Fixed(production),
                        entry.id,
                    ),
                    allocation: Vec::new(),
                    external: None,
                    frozen: None,
                });
            }
            for (index, ability, borrowed) in self.mana_activations(entry.id) {
                let Some(production) = ability.mana_production() else {
                    continue;
                };
                if ability.cost.sacrifice && self.sacrifice_prohibited(entry.id) {
                    continue;
                }
                // CR605.3c forbids recursively activating the same mana ability
                // before the enclosing activation has resolved.
                if goal
                    .source
                    .as_ref()
                    .is_some_and(|(source, pending, _, definition, slot)| {
                        *definition == borrowed.map_or(object.card, |a| a.definition)
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
                    if !matches!(
                        spec.kind.payment_kind(),
                        EffectChoiceKind::ExileGraveyard | EffectChoiceKind::Sacrifice
                    ) {
                        continue;
                    }
                    let eligible: Vec<_> = self.resources.iter().enumerate()
                        .filter(|(_, r)| {
                            let object = self.reference(r.id);
                            self.cost_object(spec, object, self.reference(entry.id))
                                // A permanent cannot pay two distinct departure costs.
                                && (!(ability.cost.sacrifice || ability.cost.exile_self) || r.id != entry.id)
                        })
                        .map(|(resource, r)| ExternalManaCost {
                            resource,
                            object: self.reference(r.id),
                            kind: spec.kind,
                        }).collect();
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
                            foreign_stash: false,
                            cast_from_hand: false,
                            cast_from_exile: false,
                            spend_any_mana: false,
                            linked_mana_source: None,
                            equip: ability.is_equip(),
                            kinds: self.kinds(entry),
                            subtypes: ability_world::CharacteristicWorld::subtypes(self, entry.id),
                        });
                        let pool = mana_lots::available(self.mana, &self.lots, purpose);
                        let quality = mana_lots::quality_from_world(self, production, entry.id)
                            .map(|q| {
                                q.with_definition(borrowed.map_or(object.card, |a| a.definition))
                            });
                        funding_cost_payments(
                            pool,
                            ability.cost,
                            exhaust::generic_discount(
                                self,
                                entry.id,
                                borrowed.map_or(object.card, |a| a.definition),
                                index,
                            ),
                            |payment| {
                                mana_lots::visit_allocations(
                                    self.mana,
                                    &self.lots,
                                    purpose,
                                    payment,
                                    |allocation| {
                                        production.base().in_world(self, entry.id).visit(
                                            |production| {
                                                let step = FundingMove {
                                                    resource,
                                                    ability: Some(index),
                                                    grantor: None,
                                                    borrowed,
                                                    payment,
                                                    counters: counters.clone(),
                                                    production,
                                                    quality,
                                                    allocation: allocation.clone(),
                                                    external: None,
                                                    frozen: None,
                                                };
                                                if let Some(eligible) = &external {
                                                    for &external in eligible {
                                                        moves.push(FundingMove {
                                                            external: Some(external),
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
            .any(|(object, _)| *object == reference)
        {
            return;
        }
        let facts = mana_lots::Source::in_world(self, id);
        self.payment_departures.push((reference, facts));
        self.payment_departures
            .sort_by_key(|(object, _)| (object.card, object.generation));
    }
    fn apply(&mut self, step: &FundingMove, preserve_departure_counters: bool) {
        for (lot, allocation) in self.lots.iter_mut().zip(&step.allocation) {
            for (held, paid) in lot.amounts.iter_mut().zip(allocation) {
                *held -= u128::from(*paid);
            }
        }
        self.lots.retain(|lot| lot.amounts != [0; 6]);
        if let Some(quality) = step.quality {
            mana_lots::add_lot(&mut self.lots, quality, step.production.map(u128::from));
        }
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
                        step.borrowed.map_or_else(
                            || self.resources[step.resource].object.card.activation(index),
                            |a| a.rules(),
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
                        step.borrowed.map_or_else(
                            || self.resources[step.resource].object.card.activation(index),
                            |a| a.rules(),
                        )
                    },
                    |(definition, _)| definition.activation(index),
                )
                .cost
        });
        if let Some(cost) = cost {
            self.pay_projected_life(self.actor, u32::from(cost.life), &[]);
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
            } else {
                object.used_abilities |= 1u64 << index;
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
            if cost.sacrifice || cost.exile_self {
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
        let tap_bonus = if pays_tap
            && CharacteristicWorld::object(self, id).zone == Zone::Battlefield
            && CharacteristicWorld::object(self, id).controller == self.actor
            && CharacteristicWorld::subtypes(self, id)
                .has(CharacteristicWorld::object(self, id).card, "Mountain")
        {
            self.game.state.players[self.actor.index()].mountain_mana_bonus
        } else {
            0
        };
        if pays_tap {
            self.resources[step.resource].object.tapped = true;
        }
        // Tap/counter costs are real prior states, even when the departure below
        // still needs replacement planning. Do not award from a tentative raw
        // destination in the preserve-departure path.
        self.refresh_enduring_projection();
        if let Some(destination) = departure {
            // Counter and tap payments precede the actual move, as in the real
            // payment path. Capture characteristics before zone effects cease.
            if !preserve_departure_counters {
                self.remember_payment_departure(id);
            }
            let reference = self.reference(id);
            // The fast path has already ruled out replacement effects. The
            // slow path commits its replacement-adjusted destination through
            // cost_departures, which accounts for death there instead.
            if !preserve_departure_counters
                && destination == Zone::Graveyard
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
                self.game.restore_projected_copy_identity(reference, object);
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
        for (pool, produced) in self.mana.iter_mut().zip(step.production) {
            *pool = pool
                .checked_add(u128::from(produced))
                .capacity("mana representation exhausted");
        }
        // Triggered mana is ordinary even when the activated ability's own
        // production has a spending restriction or other quality.
        self.mana[3] = self.mana[3]
            .checked_add(u128::from(tap_bonus))
            .capacity("mana representation exhausted");
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
                identity: (self.sacrifice_prohibition_source(r.id)
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
                    })
                    || self.continuation.as_ref().is_some_and(|c| c.selects(r.id))
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
                r.kinds,
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
                (r.attacking, r.dealt_combat_damage_this_incarnation),
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
                let spell_only = inert_spell_lots
                    && matches!(
                        lot.quality.restriction,
                        mana_lots::ManaRestrictionView::CastFromExile
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
                if !lot.quality.eligible(self.purpose) {
                    for (pool, amount) in key_mana.iter_mut().zip(lot.amounts) {
                        *pool -= amount;
                    }
                    return None;
                }
                let mut capped = lot.clone();
                for (pool, amount) in key_mana.iter_mut().zip(&mut capped.amounts) {
                    let excess = amount.saturating_sub(final_demand);
                    *amount -= excess;
                    *pool -= excess;
                }
                Some(capped)
            })
            .collect();
        FundingKey {
            enduring_story: self.enduring_story,
            exhaust_activated: self.exhaust_activated,
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
            )),
            controlled_creature_deaths: self.controlled_creatures_died,
            entries: self.entry_projection.as_ref().map(|p| EntryFundingKey {
                copy_origins: p.copy_origins.clone(),
                manifested: p.manifested.clone(),
                order: p.order.clone(),
                entered: p.entered.clone(),
                battle_protectors: p
                    .battle_protectors
                    .iter()
                    .map(|(object, player)| (*object, player.index()))
                    .collect(),
                values: p.entry_values.iter().map(|r| (r.object, r.value)).collect(),
                consumed: p.consumed.clone(),
            }),
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
    /// Pure funding moves never create new resources, untap them, restore
    /// counters, or gain life. Sum maximum *net* gains before considering colors.
    /// Ignore guards, sickness, reservations and shared costs here, since those
    /// restrictions can only lower the amount attainable. Convoke is similarly
    /// overcounted alongside any mana the same creature might produce.
    fn total_mana_upper(&self) -> u128 {
        let mut total = self.mana.into_iter().fold(0u128, u128::saturating_add);
        for resource in &self.resources {
            let object = &resource.object;
            if object.zone != Zone::Battlefield || object.controller != self.actor {
                continue;
            }
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
                total = total.saturating_add((grants as u128).saturating_mul(2));
                let blue = self
                    .roles()
                    .into_iter()
                    .flat_map(|roles| &roles.source_linked_effects)
                    .filter(|row| {
                        row.kind == SourceLinkedEffectKind::LandUntilControllerNextTurn
                            && row.recipient == self.reference(resource.id)
                    })
                    .count();
                total = total.saturating_add(blue as u128);
            }
            if !object.tapped {
                // Overcount all possible tap sources; subtype may change during funding.
                total = total.saturating_add(u128::from(
                    self.game.state.players[self.actor.index()].mountain_mana_bonus,
                ));
                total = total.saturating_add(u128::from(
                    object.card.mana_color().is_some() || self.kinds(resource) & 2 != 0,
                ));
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
                    total = total.saturating_add(1);
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
                                    self.power_upper_production(*production, resource.id).visit(|mana| {
                                        maximum =
                                            maximum.max(mana.into_iter().map(u128::from).sum());
                                        false
                                    });
                                    total = total.saturating_add(maximum);
                                }
                            }
                        }
                    }
                }
            }
            if !object.tapped {
                for production in self.potential_land_mana() {
                    let mut maximum = 0;
                    self.power_upper_production(production, resource.id)
                        .visit(|mana| {
                            maximum = maximum.max(mana.into_iter().map(u128::from).sum());
                            false
                        });
                    total = total.saturating_add(maximum);
                }
            }
            for (index, ability, borrowed) in self.mana_activations(resource.id) {
                // Tapped objects cannot pay tap costs in this closed search.
                if (!object.tapped || !ability.cost.tap) && ability.mana_production().is_some() {
                    total = total.saturating_add(activation_gain_upper(
                        ability,
                        self.power_upper_production(
                            ability.mana_production().unwrap(),
                            resource.id,
                        ),
                        ability_world::CharacteristicWorld::counter_amounts(self, resource.id),
                        if self.life_replacement_possible {
                            i128::MAX
                        } else {
                            self.life
                        },
                        exhaust::generic_discount(
                            self,
                            resource.id,
                            borrowed.map_or(object.card, |a| a.definition),
                            index,
                        ),
                    ));
                }
            }
        }
        total
    }
    // A removal provider may depart during funding and expose hidden printed
    // abilities. Count all possible production, ignoring guards and sickness;
    // each restriction we omit only raises this upper bound. No funding move
    // untaps a source, creates a permanent, restores counters or gains life.
    fn possible_color_upper(&self) -> [u128; 6] {
        let mut upper = self.mana;
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
                upper[5] = upper[5].saturating_add((grants as u128).saturating_mul(2));
                let blue = self
                    .roles()
                    .into_iter()
                    .flat_map(|roles| &roles.source_linked_effects)
                    .filter(|row| {
                        row.kind == SourceLinkedEffectKind::LandUntilControllerNextTurn
                            && row.recipient == self.reference(resource.id)
                    })
                    .count();
                upper[1] = upper[1].saturating_add(blue as u128);
            }
            if !object.tapped {
                upper[3] = upper[3].saturating_add(u128::from(
                    self.game.state.players[self.actor.index()].mountain_mana_bonus,
                ));
            }
            let mut add = |production: ManaProduction, uses: u128| {
                let production = match production.base() {
                    // Types may be restored after an Aura leaves. Counting every
                    // object (even in other zones) deliberately overestimates.
                    ManaProduction::MonocoloredPermanentColors => {
                        ManaProduction::Fixed([1, 1, 1, 1, 1, 0])
                    }
                    ManaProduction::TapSourcePowerChoice { mask } => ManaProduction::Choice {
                        mask,
                        amount: u32::MAX,
                    },
                    ManaProduction::DistinctCreaturePowers => ManaProduction::any_color(u32::MAX),
                    ManaProduction::SourcePower { color } => {
                        let mut mana = [0; 6];
                        mana[usize::from(color)] = u32::MAX;
                        ManaProduction::Fixed(mana)
                    }
                    ManaProduction::CountSubtype { color, .. } => {
                        let mut mana = [0; 6];
                        mana[usize::from(color)] =
                            u32::try_from(self.game.state.objects.len()).unwrap_or(u32::MAX);
                        ManaProduction::Fixed(mana)
                    }
                    _ => production.in_world(self, resource.id),
                };
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
                    add(ManaProduction::Fixed(mana), 1);
                }
                // A land's subtype can change during funding. Overestimate all
                // five colors rather than pruning a reachable future grant.
                if self.kinds(resource) & 2 != 0 {
                    add(
                        ManaProduction::Choice {
                            mask: 31,
                            amount: 1,
                        },
                        1,
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
                        1,
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
                                add(production, 1);
                            }
                        }
                    }
                }
            }
            if !object.tapped {
                for production in self.potential_land_mana() {
                    add(production, 1);
                }
            }
            for (_, ability, _) in self.mana_activations(resource.id) {
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
                add(production, uses);
            }
        }
        upper
    }
    /// A fixed battlefield whose only potential mana grants are spell-only
    /// and ineligible for this payment cannot improve it by tapping those sources.
    /// This deliberately conservative shortcut falls back to the normal search
    /// whenever an ability, type, controller, copy, or tap-triggered mana may
    /// change the available sources during payment.
    fn fixed_ineligible_spell_mana_cannot_pay(&self, goal: &FundingGoal) -> bool {
        use crate::cards::{ManaRestriction, StaticAbilityFragment};
        let purpose = self.payment_purpose(goal);
        if !matches!(
            purpose,
            mana_lots::Purpose::Activation(_) | mana_lots::Purpose::Spell(_)
        ) || goal.reserve.is_some()
            || self.continuation.is_some()
            || self.cast_costs.is_some()
            || self.entry_projection.is_some()
            || self.cost_group.is_some()
            || !self.sacrifice_group.is_empty()
            || self.required_face_up.is_some()
            || self.has_contributions()
            || !self.game.state.exile_links.is_empty()
            || self.game.state.borrowed_links.is_some()
            || self.game.state.graveyard_borrowing.is_some()
            || self.game.state.borrowed_slots.is_some()
            || self.game.state.activation_observers.is_some()
            || self.game.state.temporary_abilities.is_some()
            || self.game.state.self_granted_abilities.is_some()
            || self.game.state.temporary_controls.is_some()
            || self.game.state.controls.is_some()
            || self.game.state.temporary_copies.is_some()
            || self.copy_recipes.is_some()
            || self.game.state.crew.is_some()
            || !self.game.state.attachment_sources.is_empty()
            || !self.game.state.tap_sources.is_empty()
            || self.game.state.players[self.actor.index()].mountain_mana_bonus != 0
            || self.roles().is_some_and(|r| {
                !r.source_linked_effects.is_empty()
                    || !r.resolved_ability_modifications.is_empty()
                    || !r.ability_removal_sources.is_empty()
                    || !r.characteristic_sources.is_empty()
                    || !r.basic_land_types.is_empty()
                    || !r.animations.is_empty()
                    || !r.live_static.is_empty()
            })
        {
            return false;
        }
        let mut upper = mana_lots::available(self.mana, &self.lots, purpose);
        for (id, object) in self.game.state.objects.iter().enumerate() {
            if object.zone != Zone::Battlefield {
                continue;
            }
            let object = self.object(id as CardId);
            if object.card.borrows_activations()
                || object
                    .card
                    .activations()
                    .iter()
                    .any(|a| a.mana_production().is_some())
            {
                return false;
            }
            if object.card.static_ability_groups().iter().any(|group| {
                group.fragments.iter().any(|fragment| {
                    !matches!(fragment, StaticAbilityFragment::LandMana {
                        production: ManaProduction::Tagged { quality, .. }, ..
                    } if matches!(quality.restriction,
                        ManaRestriction::CastFromExile | ManaRestriction::ArtifactSpell | ManaRestriction::InstantSorcery
                        | ManaRestriction::NoncreatureSpell | ManaRestriction::PlaneswalkerSpell
                        | ManaRestriction::SpellSubtype(_) | ManaRestriction::ChosenCreatureSpell)
                        && !mana_lots::Quality::capture(self, id as CardId, **quality).eligible(purpose))
                })
            }) {
                return false;
            }
            if object.controller == goal.actor && !object.tapped {
                // Count every intrinsic color independently. This may count a
                // single land several times, so it only raises the bound.
                if let Some(color) =
                    ability_world::CharacteristicWorld::intrinsic_mana_color(self, id as CardId)
                {
                    upper[color] = upper[color].saturating_add(1);
                }
                for grant in ability_world::CharacteristicWorld::granted_mana(self, id as CardId) {
                    if grant.subtype_derived {
                        grant.production.visit(|mana| {
                            for (amount, produced) in upper.iter_mut().zip(mana) {
                                *amount = amount.saturating_add(u128::from(produced));
                            }
                            false
                        });
                    }
                }
            }
        }
        let adjusted = cast_origins::payment_goal(self, goal, purpose);
        !payable(upper, adjusted.required, adjusted.generic)
    }
    fn optimistic(&self, goal: &FundingGoal) -> bool {
        if self.fixed_ineligible_spell_mana_cannot_pay(goal) {
            return false;
        }
        // Immediate linked returns may introduce new sources, untap them, or
        // restore an ability provider. Current-resource bounds are inapplicable.
        if !self.game.state.exile_links.is_empty() {
            return true;
        }
        let required =
            u128::from(goal.generic) + goal.required.into_iter().map(u128::from).sum::<u128>();
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
            roles
                .source_linked_effects
                .iter()
                .any(|row| row.kind == SourceLinkedEffectKind::Control)
        }) || self
            .attachment_sources()
            .iter()
            .any(|&id| self.object(id).card.controls_attached())
        {
            return true;
        }
        if self.total_mana_upper() < required {
            return false;
        }
        // Substitution may be lost or restored by virtual sacrifices. Keep the
        // color bound conservative; the exact leaf uses the overlay.
        if matches!(self.purpose, mana_lots::Purpose::Spell(_))
            && self.static_sources().iter().any(|&id| {
                self.game.state.objects[id as usize]
                    .card
                    .has_cast_origin_abilities()
            })
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
                upper[3] = upper[3].saturating_add(u128::from(
                    self.game.state.players[self.actor.index()].mountain_mana_bonus,
                ));
                for grant in ability_world::CharacteristicWorld::granted_mana(self, resource.id) {
                    grant.production.visit(|mana| {
                        for i in 0..6 {
                            upper[i] = upper[i].saturating_add(u128::from(mana[i]));
                        }
                        false
                    });
                }
            }
            if tap_ready
                && let Some(color) =
                    ability_world::CharacteristicWorld::intrinsic_mana_color(self, resource.id)
            {
                upper[color] = upper[color].saturating_add(1);
            }
            for (index, ability, borrowed) in self.mana_activations(resource.id) {
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
                        *definition == borrowed.map_or(object.card, |a| a.definition)
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
                production.in_world(self, resource.id).visit(|mana| {
                    for index in 0..6 {
                        maximum[index] = maximum[index].max(mana[index]);
                    }
                    false
                });
                for index in 0..6 {
                    upper[index] = upper[index].saturating_add(
                        u128::from(uses).saturating_mul(u128::from(maximum[index])),
                    );
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
        improvise::masks(
            self,
            self.actor,
            self.convoke,
            self.improvise_available(),
            &[],
        )
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
            if let Some((_, facts)) = self
                .payment_departures
                .iter()
                .find(|(source, _)| source == object)
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
        if self.additional_cost_deficit() {
            return false;
        }
        if self.pure_tap_teamwork_deficit(goal) {
            return false;
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
            && mana_loyalty_placement_payable(self, goal)
            && source_exile_payable(self, goal)
            && goal.reserve.is_none_or(|(reference, spec)| {
                spec.kind.payment_kind() != EffectChoiceKind::ExileControlledOrGraveyard
                    || !ability_world::CharacteristicWorld::graveyard_exile_prohibited(
                        self,
                        reference.card,
                    )
            })
            && self.cost_group_ready()
            && self.complete_funding_continuation(goal)
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
                            && self.mana_activations(r.id).any(|(_, a, _)| {
                                a.cost.sacrifice
                                    && (!a.cost.tap || !r.object.tapped)
                                    && a.mana_production().is_some()
                            })
                    })
                    .map(|r| u128::from(r.object.card.fixed_mana_value()))
                    .sum::<u128>();
            upper < u128::from(n)
        }) || !self.optimistic(goal)
        {
            return false;
        }
        if !visited.insert(self.key(goal)) {
            return false;
        }
        let mut moves = self.moves(goal);
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
            if self.visit_applied(&step, &mut |next| {
                next.reachable_with_evidence(goal, evidence, visited)
            }) {
                return true;
            }
        }
        false
    }
    fn action(&self, step: &FundingMove) -> Action {
        let card = self.resources[step.resource].id;
        if let Some(grantor) = step.grantor {
            return Action::TapGrantedMana {
                card,
                grantor,
                production: step.production,
            };
        }
        if step.external.is_some() {
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
    pub(super) fn can_fund_for(&self, goal: FundingGoal, purpose: mana_lots::Purpose) -> bool {
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
            && a.creature_counter_cost_amount().is_some()
        {
            return self.creature_counter_fundable(
                goal,
                purpose,
                a.creature_counter_cost_amount().unwrap(),
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
        if !reserve_obligations(
            &goal,
            reserve,
            reserve.map_or(0, |o| self.object_kinds(goal.reserve.unwrap().0.card, o)),
            reserve.map_or(Keywords::NONE, |_| {
                self.keywords(goal.reserve.unwrap().0.card)
            }),
            reserve.map_or(GrantedSubtypes::default(), |_| {
                self.granted_subtypes(goal.reserve.unwrap().0.card)
            }),
            reserve.map_or("", |_| self.object_name(goal.reserve.unwrap().0.card)),
        ) || !source_obligations(
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
        ) {
            return false;
        }
        let adjusted = cast_origins::payment_goal(self, &goal, purpose);
        // The common prefunded path needs no resource graph or heap allocations.
        if payable(
            self.available_mana(goal.actor, purpose),
            adjusted.required,
            adjusted.generic,
        ) && mana_loyalty_placement_payable(self, &goal)
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
            &improvise::masks(self, goal.actor, convoke, artifact, &[]),
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
        if let Some(reference) = object {
            proposal.add_cast_cost_clause(CastCostClause::Harmonize(reference));
        }
        proposal.initialize_cast_costs(improvise::potential(
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
            a.item
                .source
                .expect("cost announcement has object source")
                .card
                .has_convoke(),
        );
        world.set_purpose(self.purpose_for_item(&a.item));
        world.set_cost_context(Some(&proposal));
        world.reachable(&goal, &mut HashSet::new())
    }
    pub(super) fn announcement_cost_fundable(&self, a: &Announcement, cost: ManaCost) -> bool {
        if a.item
            .cast
            .is_some_and(|c| c.method == CastMethod::Harmonize)
        {
            if self.harmonize_choice_fundable(a, None) {
                return true;
            }
            return (0..self.state.objects.len())
                .any(|id| self.harmonize_choice_fundable(a, Some(self.object_ref(id as CardId))));
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
            return self.creature_counter_fundable(
                FundingGoal {
                    actor: a.item.controller,
                    required: cost.colored,
                    generic: cost.generic,
                    reserve: None,
                    source: None,
                },
                self.purpose_for_item(&a.item),
                a.graveyard_origin_counter_cost().unwrap(),
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
        if a.item
            .cast
            .is_some_and(|c| matches!(c.method, CastMethod::Sneak | CastMethod::ExileLife))
        {
            let mut proposal = a.clone();
            proposal.cost = Some(cost);
            proposal.cast_costs = Some(Box::default());
            proposal.initialize_cast_costs(improvise::potential(
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
                a.item
                    .source
                    .expect("cost announcement has object source")
                    .card
                    .has_convoke(),
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
                    self.spell_can_fund_total(
                        a.item.controller,
                        a.item
                            .source
                            .expect("cost announcement has object source")
                            .id,
                        total,
                        None,
                    )
                })
            });
        }
        if a.item
            .source
            .expect("cost announcement has object source")
            .card
            .target_cost_reduction()
            .is_some()
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
            | StackKind::CopyEndStepSacrifice
            | StackKind::CopyEndStepConditionalSacrifice
            | StackKind::CasualtyCopy
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
                        if announcement
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
                    .item
                    .source
                    .expect("cost announcement has object source")
                    .card
                    .activation(ability)
                    .cost;
                let power_up = announcement.total_cost.or_else(|| {
                    announcement.power_up_costs.as_ref().map(|_| {
                        announcement.cost.unwrap_or_else(|| {
                            announcement.cost_options().next().expect("activation cost")
                        })
                    })
                });
                FundingGoal {
                    reserve: self.announcement_reserve(announcement),
                    actor: announcement.item.controller,
                    required: power_up
                        .or(announcement.cost)
                        .map_or(cost.colored, |c| c.colored),
                    generic: power_up
                        .or(announcement.cost)
                        .map_or(
                            cost.generic
                                .saturating_sub(announcement.activation_discount),
                            |c| c.generic,
                        )
                        .checked_add(if announcement.has_x_cost() && power_up.is_none() {
                            announcement
                                .item
                                .x
                                .unwrap_or(0)
                                .checked_mul(
                                    announcement
                                        .item
                                        .source
                                        .expect("object-backed announcement")
                                        .card
                                        .activation(ability)
                                        .x_symbols(),
                                )
                                .capacity("activation X cost exhausted")
                        } else {
                            0
                        })
                        .capacity("activation cost exhausted"),
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
            | StackKind::CopyEndStepSacrifice
            | StackKind::CopyEndStepConditionalSacrifice
            | StackKind::CasualtyCopy
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
            Pending::Convoke { .. } | Pending::Improvise { .. } | Pending::Waterbend { .. }
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
                || !state.cost_completion()
                || !mana_loyalty_placement_payable(&state, &goal)
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
            proposal
                .item
                .source
                .expect("cost announcement has object source")
                .card
                .has_convoke(),
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
        let Some(frames) = self.state.mana_announcement_stack.as_deref() else {
            return true;
        };
        let (frame, outer) = frames.split_last().expect("nonempty mana frame chain");
        let child = self
            .state
            .announcing
            .as_ref()
            .expect("child mana announcement");
        let goal = self.funding_goal(&frame.announcement);
        let mut state = FundingState::from_game_with_convoke(
            self,
            &goal,
            frame
                .announcement
                .item
                .source
                .expect("cost announcement has object source")
                .card
                .has_convoke(),
        );
        state.set_payment_context(&frame.announcement);
        state.set_purpose(self.purpose_for_item(&frame.announcement.item));
        state.set_funding_continuation(
            &frame.announcement,
            outer,
            frame.quality.as_ref().map(|(_, q)| q),
        );
        state.moves(&goal).into_iter().any(|step| {
            if child
                .item
                .source
                .is_none_or(|source| state.resources[step.resource].id != source.id)
                || step.ability
                    != match child.item.kind {
                        StackKind::ActivatedAbility { ability } => Some(ability),
                        _ => None,
                    }
                || step.counters != child.counter_payment
                || Some(step.production) != child.production
                || step.borrowed.map(|v| v.slot) != child.item.borrowed_slot
                || payment.is_some_and(|p| concrete_payment(step.payment) != p)
                || external
                    .or(child.additional)
                    .is_some_and(|wanted| step.external.is_none_or(|cost| cost.object != wanted))
            {
                return false;
            }
            state.visit_applied(&step, &mut |next| {
                next.reachable_with_evidence(
                    &goal,
                    frame.announcement.evidence.as_ref().map(|e| e.threshold),
                    &mut HashSet::new(),
                )
            })
        })
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
        let convoke = announcement.item.kind == StackKind::Spell
            && announcement
                .item
                .source
                .expect("cost announcement has object source")
                .card
                .has_convoke();
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
        if (announcement
            .item
            .cast
            .is_some_and(|c| c.method == CastMethod::Harmonize || c.blight == Some(true))
            || announcement.ordered_craft()
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
        if (convoke || artifact || waterbend)
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
                || (waterbend && emit(Action::BeginWaterbend)))
        {
            return true;
        }
        // An explicit {0} still has a mana component; a pure tap/sacrifice
        // activation cost does not open a mana-ability window.
        if let Some((_, index, _, definition, _)) = goal.source.as_ref()
            && !definition.activation(*index).cost.mana_component
        {
            return false;
        }
        for step in state.moves(&goal) {
            if state.visit_applied(&step, &mut |next| {
                next.reachable_with_evidence(
                    &goal,
                    announcement.evidence.as_ref().map(|e| e.threshold),
                    &mut HashSet::new(),
                )
            }) && emit(state.action(&step))
            {
                return true;
            }
        }
        false
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
        for step in state.moves(&goal) {
            if state.visit_applied(&step, &mut |next| {
                next.reachable(&goal, &mut HashSet::new())
            }) && emit(state.action(&step))
            {
                return true;
            }
        }
        false
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
        for step in state.moves(&goal) {
            if state.visit_applied(&step, &mut |next| {
                next.reachable(&goal, &mut HashSet::new())
            }) && emit(state.action(&step))
            {
                return true;
            }
        }
        false
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
        if request
            .life
            .is_some_and(|life| self.state.players[request.payer.index()].life < i128::from(life))
        {
            return false;
        }
        for cost in std::iter::once(request.cost).chain(request.alternatives.iter().copied()) {
            let goal = FundingGoal {
                reserve: None,
                actor: request.payer,
                required: cost.colored,
                generic: cost.generic,
                source: None,
            };
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
        // A payment request grants a mana-ability window even if it is declined.
        // Emit the shared window once regardless of the number of hybrid choices.
        let goal = FundingGoal {
            reserve: None,
            actor: request.payer,
            required: request.cost.colored,
            generic: request.cost.generic,
            source: None,
        };
        let state = FundingState::from_game(self, &goal);
        for step in state.moves(&goal) {
            if emit(state.action(&step)) {
                return true;
            }
        }
        false
    }
}

#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
impl Game {
    /// A mana ability in an enclosing guided payment window must leave some
    /// completion of that window. Reuse its exact resource search, retaining
    /// complete plans so later scalar choices cannot mix incompatible prefixes.
    pub(super) fn prepare_mana_allocations(&mut self, action: &Action) {
        let Action::ActivateMana { payment, .. } = action else {
            return;
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
        let (goal, convoke, evidence) = match self.state.pending {
            Pending::PayCosts { .. }
            | Pending::Convoke { .. }
            | Pending::Improvise { .. }
            | Pending::Waterbend { .. } => {
                let a = self.state.announcing.as_ref().unwrap();
                (
                    self.funding_goal(a),
                    a.item.kind == StackKind::Spell
                        && a.item
                            .source
                            .expect("cost announcement has object source")
                            .card
                            .has_convoke(),
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
        for step in world.moves(&goal) {
            if world.action(&step) != *action {
                continue;
            }
            if world.visit_applied(&step, &mut |next| {
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
    fn remove_creature_counter(&mut self, r: ObjectRef, kind: CounterKind) -> bool {
        let Some(index) = self.resources.iter().position(|v| v.id == r.card) else {
            return false;
        };
        let o = &self.resources[index].object;
        if o.zone != Zone::Battlefield
            || o.generation != r.generation
            || o.controller != self.actor
            || self.kinds(&self.resources[index]) & 1 == 0
        {
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
        true
    }
    fn creature_counter_payment_possible(
        &self,
        goal: &FundingGoal,
        amount: u16,
        selected: &[(ObjectRef, CounterKind)],
    ) -> bool {
        if selected.len() > usize::from(amount) {
            return false;
        }
        let mut after = self.clone();
        for &(r, kind) in selected {
            if !after.remove_creature_counter(r, kind) {
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
                    let object = after.object(source.card);
                    if object.zone != Zone::Battlefield
                        || object.controller != after.actor
                        || after.kinds(&Resource {
                            id: source.card,
                            object: object.clone(),
                            attacking: false,
                        }) & 1
                            == 0
                    {
                        return 0;
                    }
                    let loyalty = definition
                        .activation(*index)
                        .cost
                        .loyalty
                        .filter(|change| *change < 0)
                        .map_or(0, |change| u64::from(change.unsigned_abs()));
                    selected.total() + u128::from(loyalty)
                });
        let available = after
            .resources
            .iter()
            .filter(|r| {
                r.object.zone == Zone::Battlefield
                    && r.object.controller == self.actor
                    && after.kinds(r) & 1 != 0
            })
            .map(|r| ability_world::CharacteristicWorld::counter_amounts(&after, r.id).total())
            .sum::<u128>();
        available.saturating_sub(reserved) >= u128::from(amount) - selected.len() as u128
    }
    fn fund_creature_counter_cost(
        &self,
        goal: &FundingGoal,
        amount: u16,
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
        ) && self.creature_counter_payment_possible(goal, amount, selected)
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
                    ability_world::CharacteristicWorld::reference(self, r.card) == r
                        && ability_world::CharacteristicWorld::object(self, r.card).zone
                            == Zone::Battlefield
                        && ability_world::CharacteristicWorld::object(self, r.card).controller
                            == self.actor
                        && ability_world::CharacteristicWorld::kinds(self, r.card) & 1 != 0,
                )
            })
            .collect();
        if !visited.insert((self.key(goal), facts)) {
            return false;
        }
        for step in self.moves(goal) {
            if self.visit_applied(&step, &mut |next| {
                next.fund_creature_counter_cost(goal, amount, selected, visited)
            }) {
                return true;
            }
        }
        false
    }
}
#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
impl Game {
    pub(super) fn creature_counter_fundable(
        &self,
        goal: FundingGoal,
        purpose: mana_lots::Purpose,
        amount: u16,
        selected: &[(ObjectRef, CounterKind)],
    ) -> bool {
        if self.cast_costs_mana_paid() {
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
                Some(Self::creature_counter_spec(amount)),
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
        let adds = self.state.objects.iter().any(|o| {
            o.card.activations().iter().enumerate().any(|(index, a)| {
                a.mana_production().is_some()
                    && o.card.activation_put_counter(index as u16).is_some()
            })
        });
        if !adds {
            let upper = self
                .state
                .objects
                .iter()
                .enumerate()
                .filter(|(_, o)| o.zone == Zone::Battlefield)
                .map(|(id, _)| self.counter_amounts(id as CardId).total())
                .sum::<u128>();
            if upper < u128::from(amount) {
                return false;
            }
        }
        world.fund_creature_counter_cost(&goal, amount, selected, &mut HashSet::new())
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
        for step in state.moves(&goal) {
            if state.visit_applied(&step, &mut |next| {
                next.reachable(&goal, &mut HashSet::new())
            }) && emit(state.action(&step))
            {
                return true;
            }
        }
        false
    }
}
