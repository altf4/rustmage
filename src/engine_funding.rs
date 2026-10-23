//! Exact reachability for the admitted pure mana sources; no game-state clones,
//! heuristics that discard legal plans, or mutation during action generation.
use super::*;
use std::collections::HashSet;

#[derive(Clone, Debug)]
pub(super) struct FundingGoal {
    pub actor: Player,
    pub required: Mana,
    pub generic: u32,
    pub reserve: Option<(ObjectRef, AdditionalCost)>,
    pub source: Option<(ObjectRef, u16, Counters, Card)>,
}
#[derive(Clone)]
struct Resource {
    id: CardId,
    attacking: bool,
    object: Object,
}
#[derive(Clone)]
struct FundingState<'g> {
    game: &'g Game,
    actor: Player,
    life_gained: [bool; 2],
    active: Player,
    mana: [u128; 6],
    lots: Vec<mana_lots::Lot>,
    purpose: mana_lots::Purpose,
    life: i128,
    resources: Vec<Resource>,
    named_counters: live_counters::Rows,
    convoke: bool,
}
impl ability_world::CharacteristicWorld for FundingState<'_> {
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
            self.game.state.players[player.index()].life
        }
    }
    fn gained_life(&self, player: Player) -> bool {
        self.life_gained[player.index()]
    }
    fn attacking(&self, id: CardId) -> bool {
        self.resources
            .iter()
            .find(|resource| resource.id == id)
            .is_some_and(|resource| resource.attacking)
    }
}

#[derive(Clone)]
struct FundingMove {
    resource: usize,
    ability: Option<u16>,
    grantor: Option<ObjectRef>,
    payment: [u64; 6],
    counters: Counters,
    production: Mana,
    quality: Option<mana_lots::Quality>,
    allocation: Vec<[u64; 6]>,
}
#[derive(Hash, PartialEq, Eq)]
struct ResourceKey {
    definition: usize,
    token: bool,
    kinds: u16,
    protected: bool,
    reserved: bool,
    present: bool,
    tapped: bool,
    sick: bool,
    counters: InitialCounters,
    used: u64,
    gained: Keywords,
    owned: bool,
    controlled: bool,
    attacking: bool,
    identity: Option<CardId>,
    attached: Option<(CardId, u64)>,
}
#[derive(Hash, PartialEq, Eq)]
struct FundingKey {
    mana: [u128; 6],
    lots: Vec<mana_lots::Lot>,
    life: i128,
    // Equivalent sources are interchangeable for feasibility. Protect the
    // enclosing ability's source separately, even if its printed card matches.
    resources: Vec<ResourceKey>,
    named_counters: live_counters::Rows,
}

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
fn funding_payments(pool: [u128; 6], colored: Mana, generic: u32, mut emit: impl FnMut([u64; 6])) {
    if (0..6).any(|i| pool[i] < u128::from(colored[i])) {
        return;
    }
    fn visit(
        i: usize,
        left: u32,
        pool: [u128; 6],
        mut paid: [u64; 6],
        emit: &mut impl FnMut([u64; 6]),
    ) {
        if i == 5 {
            if pool[i] >= u128::from(left) {
                paid[i] += u64::from(left);
                emit(paid);
            }
            return;
        }
        for n in 0..=pool[i].min(u128::from(left)) as u32 {
            let old = paid[i];
            paid[i] = old + u64::from(n);
            visit(i + 1, left - n, pool, paid, emit);
            paid[i] = old;
        }
    }
    visit(
        0,
        generic,
        std::array::from_fn(|i| pool[i] - u128::from(colored[i])),
        colored.map(u64::from),
        &mut emit,
    );
}
/// Exact equivalents of one mana ability, with duplicate physical allocations removed.
fn funding_cost_payments(pool: [u128; 6], cost: ActivationCost, mut emit: impl FnMut([u64; 6])) {
    let mut seen = std::collections::HashSet::new();
    for equivalent in cost.mana_options() {
        funding_payments(pool, equivalent.colored, equivalent.generic, |payment| {
            if cost.alternatives.is_empty() || seen.insert(payment) {
                emit(payment);
            }
        });
    }
}
fn concrete_payment(payment: [u64; 6]) -> Mana {
    payment.map(|n| u32::try_from(n).capacity("mana payment exhausted"))
}
fn reserve_obligations(
    goal: &FundingGoal,
    object: Option<&Object>,
    kinds: u16,
    keywords: Keywords,
    subtypes: GrantedSubtypes,
) -> bool {
    goal.reserve.is_none_or(|(reference, spec)| {
        object.is_some_and(|o| {
            o.generation == reference.generation
                && Game::additional_object_matches(spec, goal.actor, o, kinds, keywords, subtypes)
        })
    })
}
fn source_obligations(
    goal: &FundingGoal,
    object: Option<&Object>,
    amounts: CounterAmounts<'_>,
    kinds: u16,
    keywords: impl FnOnce() -> Keywords,
    life: i128,
) -> bool {
    let Some((source, index, counters, definition)) = goal.source.as_ref() else {
        return true;
    };
    let ability = &definition.activations()[*index as usize];
    if ability.cost.life != 0 && i128::from(ability.cost.life) > life {
        return false;
    }
    let current = object.filter(|object| {
        object.zone == ability.source_zone() && object.generation == source.generation
    });
    let requires_source = ability.cost.tap
        || ability.cost.sacrifice
        || ability.cost.discard
        || ability.cost.exile_self
        || ability.cost.remove.is_some()
        || ability.cost.loyalty.is_some();
    if let Some(object) = current {
        Game::object_activation_available_with_kinds(
            object,
            amounts,
            kinds,
            keywords,
            life,
            ability,
            *index as usize,
            goal.actor,
            true,
        ) && amounts.contains(counters)
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
fn activation_gain_upper<'a>(
    ability: &ActivatedAbility,
    production: ManaProduction,
    counters: impl Into<CounterAmounts<'a>>,
    life: i128,
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
            u128::from(option.generic) + option.colored.into_iter().map(u128::from).sum::<u128>()
        })
        .min()
        .unwrap(); // mana_options always includes the primary cost.
    let gain = produced.saturating_sub(paid);
    if gain == 0 {
        return 0;
    }
    // A true repeatable positive engine keeps an unbounded upper bound. Exact
    // reachability still determines whether its seed cost/colors are attainable.
    let mut uses = if cost.tap || cost.sacrifice || ability.once_per_turn || ability.exhaust {
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

impl<'g> FundingState<'g> {
    fn from_game(game: &'g Game, goal: &FundingGoal) -> Self {
        Self::from_game_with_convoke(game, goal, false)
    }
    fn from_game_with_convoke(game: &'g Game, goal: &FundingGoal, convoke: bool) -> Self {
        // A prerequisite need not itself produce mana (e.g. a changeling for
        // Mox Jasper). Keep only relevant owned witnesses, including removable
        // mana sources, so virtual sacrifices invalidate later guarded moves.
        let mut subtypes = Vec::new();
        for object in &game.state.objects {
            if object.zone == Zone::Battlefield && object.controller == goal.actor {
                for ability in object.card.activations() {
                    if let Some(production) = ability.mana_production() {
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
        let resources = game
            .state
            .objects
            .iter()
            .enumerate()
            .filter(|(id, object)| {
                (object.zone == Zone::Command && object.card.is_emblem()) || goal.reserve
                    .is_some_and(|(reference, _)| reference.card == *id as CardId)
                    || goal
                        .source.as_ref()
                        .is_some_and(|(source, _, _, _)| source.card == *id as CardId)
                    || object.zone == Zone::Battlefield
                        && (game.state.controls.is_some() || object.card.has_cast_origin_abilities()
                            || object.card.attachment_bonus().is_some()
                            || !object.card.keyword_grants().is_empty()
                            || !object.card.counter_keyword_grants().is_empty()
                            || object.card.static_ability_groups().iter().any(|group| group.fragments.iter().any(|fragment| matches!(fragment, crate::cards::StaticAbilityFragment::ChosenCreatureType | crate::cards::StaticAbilityFragment::ChosenModifier(..) | crate::cards::StaticAbilityFragment::DeathBaronLord | crate::cards::StaticAbilityFragment::SourceCounterSubtype { .. } | crate::cards::StaticAbilityFragment::SourceCounterKeywords { .. } | crate::cards::StaticAbilityFragment::CounterKeywordOwnTurn(..))))
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
        Self {
            game,
            convoke,
            actor: goal.actor,
            life_gained: game
                .state
                .players
                .each_ref()
                .map(|p| p.life_gained_this_turn > 0),
            active: game.state.active,
            mana: game.state.players[goal.actor.index()].mana.map(u128::from),
            lots: game.mana_lots(goal.actor).to_vec(),
            purpose: game.funding_purpose(goal),
            life: game.state.players[goal.actor.index()].life,
            resources,
            named_counters: game.state.named_counters.clone(),
        }
    }
    fn obligations(&self, goal: &FundingGoal) -> bool {
        let source = goal
            .source
            .as_ref()
            .and_then(|(r, _, _, _)| self.resources.iter().find(|s| s.id == r.card));
        let reserve = goal
            .reserve
            .and_then(|(r, _)| self.resources.iter().find(|s| s.id == r.card));
        self.game.state.announcing.as_ref().is_none_or(|a| {
            a.additional_group.as_ref().is_none_or(|group| {
                group.iter().all(|r| {
                    let o = ability_world::CharacteristicWorld::object(self, r.card);
                    o.zone == Zone::Battlefield
                        && o.generation == r.generation
                        && !o.tapped
                        && o.controller == goal.actor
                })
            })
        }) && reserve_obligations(
            goal,
            reserve.map(|r| &r.object),
            reserve.map_or(0, |r| self.kinds(r)),
            reserve.map_or(Keywords::NONE, |r| self.keywords(r)),
            reserve.map_or(GrantedSubtypes::default(), |r| self.granted_subtypes(r.id)),
        ) && source_obligations(
            goal,
            source.map(|r| &r.object),
            source.map_or(CounterAmounts::default(), |r| {
                ability_world::CharacteristicWorld::counter_amounts(self, r.id)
            }),
            source.map_or(0, |r| self.kinds(r)),
            || source.map_or(Keywords::NONE, |r| self.keywords(r)),
            self.life,
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
        let mut keywords = target
            .object
            .keywords()
            .union(
                self.game
                    .copied_keywords(ability_world::CharacteristicWorld::reference(
                        self, target.id,
                    )),
            )
            .union(ability_world::CharacteristicWorld::counter_subtype_keywords(self, target.id));
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
    fn guard_available(&self, guard: ActivationGuard, actor: Player) -> bool {
        match guard {
            ActivationGuard::Always => true,
            ActivationGuard::LifeAboveStarting(extra) => {
                (if actor == self.actor {
                    self.life
                } else {
                    self.game.state.players[actor.index()].life
                }) >= STARTING_LIFE + i128::from(extra)
            }
            ActivationGuard::OwnTurn => self.active == actor,
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
    fn moves(&self, goal: &FundingGoal) -> Vec<FundingMove> {
        let mut moves = Vec::new();
        for (resource, entry) in self.resources.iter().enumerate() {
            let object = &entry.object;
            if object.zone != Zone::Battlefield || object.controller != goal.actor {
                continue;
            }
            if !object.tapped
                && (self.kinds(entry) & 1 == 0
                    || !object.summoning_sick
                    || self.keywords(entry).contains(Keywords::HASTE))
            {
                for grant in ability_world::CharacteristicWorld::granted_mana(self, entry.id) {
                    grant.production.visit(|production| {
                        moves.push(FundingMove {
                            resource,
                            ability: None,
                            grantor: Some(grant.source),
                            payment: [0; 6],
                            counters: Counters::default(),
                            production,
                            quality: None,
                            allocation: Vec::new(),
                        });
                        false
                    });
                }
            }
            if !ability_world::CharacteristicWorld::printed_abilities_available(self, entry.id) {
                continue;
            }
            if !object.tapped
                && (self.kinds(entry) & 1 == 0
                    || !object.summoning_sick
                    || self.keywords(entry).contains(Keywords::HASTE))
                && let Some(color) = object.card.mana_color()
            {
                let mut production = [0; 6];
                production[color] = 1;
                moves.push(FundingMove {
                    resource,
                    ability: None,
                    grantor: None,
                    payment: [0; 6],
                    counters: Counters::default(),
                    production,
                    quality: None,
                    allocation: Vec::new(),
                });
            }
            for (index, ability) in object.card.activations().iter().enumerate() {
                let Some(production) = ability.mana_production() else {
                    continue;
                };
                // CR605.3c forbids recursively activating the same mana ability
                // before the enclosing activation has resolved.
                if goal
                    .source
                    .as_ref()
                    .is_some_and(|(source, pending, _, definition)| {
                        *definition == object.card
                            && source.card == entry.id
                            && object.generation == source.generation
                            && *pending as usize == index
                    })
                {
                    continue;
                }
                if !self.guard_available(ability.guard, goal.actor)
                    || !Game::object_activation_available_with_kinds(
                        object,
                        ability_world::CharacteristicWorld::counter_amounts(self, entry.id),
                        self.kinds(entry),
                        || self.keywords(entry),
                        self.life,
                        ability,
                        index,
                        goal.actor,
                        false,
                    )
                {
                    continue;
                }
                activations::counter_payments(
                    ability.cost,
                    ability_world::CharacteristicWorld::counter_amounts(self, entry.id),
                    |counters| {
                        let purpose = mana_lots::Purpose::Activation(mana_lots::Source {
                            card: object.card,
                            foreign_stash: false,
                            kinds: self.kinds(entry),
                            subtypes: ability_world::CharacteristicWorld::subtypes(self, entry.id),
                        });
                        let pool = mana_lots::available(self.mana, &self.lots, purpose);
                        let quality = match production {
                            ManaProduction::Tagged { quality, .. } => {
                                Some(mana_lots::Quality::capture(self, entry.id, *quality))
                            }
                            _ => None,
                        };
                        funding_cost_payments(pool, ability.cost, |payment| {
                            mana_lots::visit_allocations(
                                self.mana,
                                &self.lots,
                                purpose,
                                payment,
                                |allocation| {
                                    production.base().in_world(self, entry.id).visit(
                                        |production| {
                                            moves.push(FundingMove {
                                                resource,
                                                ability: Some(index as u16),
                                                grantor: None,
                                                payment,
                                                counters: counters.clone(),
                                                production,
                                                quality,
                                                allocation: allocation.clone(),
                                            });
                                            false
                                        },
                                    );
                                },
                            );
                        });
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
    fn refresh_control_projection(&mut self) {
        for row in continuous_control::controllers(self.game, |id| {
            ability_world::CharacteristicWorld::object(self, id).clone()
        }) {
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
    }
    fn apply(&mut self, step: &FundingMove) {
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
        let object = &mut self.resources[step.resource].object;
        if let Some(index) = step.ability {
            let cost = object.card.activations()[index as usize].cost;
            self.life -= i128::from(cost.life);
            object.used_abilities |= 1u64 << index;
            if cost.tap {
                object.tapped = true;
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
            if cost.sacrifice {
                object.zone = Zone::Graveyard;
                object.counters = InitialCounters::default();
                live_counters::set(&mut self.named_counters, reference, None);
            }
        } else {
            object.tapped = true;
        }
        self.refresh_control_projection();
        for (pool, produced) in self.mana.iter_mut().zip(step.production) {
            *pool = pool
                .checked_add(u128::from(produced))
                .capacity("mana representation exhausted");
        }
    }
    fn key(&self, goal: &FundingGoal) -> FundingKey {
        // Attachment edges distinguish otherwise interchangeable mana sources.
        let linked = self.named_counters.is_some()
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
                kinds: self.kinds(r),
                protected: goal
                    .source
                    .as_ref()
                    .is_some_and(|(source, _, _, _)| source.card == r.id),
                reserved: goal
                    .reserve
                    .is_some_and(|(reference, _)| reference.card == r.id),
                present: r.object.zone == Zone::Battlefield,
                tapped: r.object.tapped,
                sick: r.object.summoning_sick,
                counters: r.object.counters,
                used: r.object.used_abilities,
                gained: r.object.gained_keywords,
                // Ownership still determines whose graveyard receives a sacrificed
                // resource; control determines who can use it before that move.
                owned: r.object.owner == goal.actor,
                controlled: r.object.controller == goal.actor,
                attacking: r.attacking,
                // Unattached, unmodified basic lands remain interchangeable even
                // when an unrelated Aura is present. Keep exact identities for
                // attachment participants, nonbasics and named-counter worlds.
                identity: (self
                    .game
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
                            || self.game.state.attachment_sources.iter().any(|&source| {
                                let provider =
                                    ability_world::CharacteristicWorld::object(self, source);
                                provider.zone == Zone::Battlefield
                                    && provider.attached_to
                                        == Some(ObjectRef {
                                            card: r.id,
                                            generation: r.object.generation,
                                        })
                            }))))
                .then_some(r.id),
                attached: r.object.attached_to.map(|r| (r.card, r.generation)),
            })
            .collect();
        resources.sort_by_key(|r| {
            (
                (r.definition, r.token),
                r.kinds,
                (r.protected, r.reserved),
                r.present,
                r.tapped,
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
                r.attacking,
                (r.owned, r.controlled, r.identity, r.attached),
            )
        });
        FundingKey {
            mana: self.mana,
            lots: self.lots.clone(),
            life: self.life,
            resources,
            named_counters: self.named_counters.clone(),
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
            if !object.tapped {
                total = total.saturating_add(u128::from(object.card.mana_color().is_some()));
                if self.convoke && self.kinds(resource) & 1 != 0 {
                    total = total.saturating_add(1);
                }
            }
            if !object.tapped {
                for &source in &self.game.state.attachment_sources {
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
                                ) = fragment
                                {
                                    let mut maximum = 0;
                                    production.visit(|mana| {
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
            for ability in object.card.activations() {
                // Tapped objects cannot pay tap costs in this closed search.
                if (!object.tapped || !ability.cost.tap) && ability.mana_production().is_some() {
                    total = total.saturating_add(activation_gain_upper(
                        ability,
                        ability
                            .mana_production()
                            .unwrap()
                            .in_world(self, resource.id),
                        ability_world::CharacteristicWorld::counter_amounts(self, resource.id),
                        self.life,
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
            let mut add = |production: ManaProduction, uses: u128| {
                let production = match production.base() {
                    // Types may be restored after an Aura leaves. Counting every
                    // object (even in other zones) deliberately overestimates.
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
                for &source in &self.game.state.attachment_sources {
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
                            if let crate::cards::StaticAbilityFragment::AttachmentMana(production) =
                                *fragment
                            {
                                add(production, 1);
                            }
                        }
                    }
                }
            }
            for ability in object.card.activations() {
                let Some(production) = ability.mana_production() else {
                    continue;
                };
                let cost = ability.cost;
                if object.tapped && cost.tap {
                    continue;
                }
                let mut uses =
                    if cost.tap || cost.sacrifice || ability.once_per_turn || ability.exhaust {
                        1
                    } else {
                        u128::MAX
                    };
                if cost.life > 0 {
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
    fn optimistic(&self, goal: &FundingGoal) -> bool {
        let required =
            u128::from(goal.generic) + goal.required.into_iter().map(u128::from).sum::<u128>();
        if self.total_mana_upper() < required {
            return false;
        }
        // Substitution may be lost or restored by virtual sacrifices. Keep the
        // color bound conservative; the exact leaf uses the overlay.
        if matches!(self.purpose, mana_lots::Purpose::Spell(_))
            && self.game.state.static_sources.iter().any(|&id| {
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
        if ability_world::CharacteristicWorld::has_ability_removal(self) {
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
                    || self.keywords(resource).contains(Keywords::HASTE));
            if tap_ready {
                for grant in ability_world::CharacteristicWorld::granted_mana(self, resource.id) {
                    grant.production.visit(|mana| {
                        for i in 0..6 {
                            upper[i] = upper[i].saturating_add(u128::from(mana[i]));
                        }
                        false
                    });
                }
            }
            if tap_ready && let Some(color) = object.card.mana_color() {
                upper[color] = upper[color].saturating_add(1);
            }
            for (index, ability) in object.card.activations().iter().enumerate() {
                let Some(production) = ability.mana_production() else {
                    continue;
                };
                if goal
                    .source
                    .as_ref()
                    .is_some_and(|(source, pending, _, definition)| {
                        *definition == object.card
                            && source.card == resource.id
                            && object.generation == source.generation
                            && *pending as usize == index
                    })
                    || !self.guard_available(ability.guard, goal.actor)
                    || !Game::object_activation_available_with_kinds(
                        object,
                        ability_world::CharacteristicWorld::counter_amounts(self, resource.id),
                        self.kinds(resource),
                        || self.keywords(resource),
                        self.life,
                        ability,
                        index,
                        goal.actor,
                        false,
                    )
                {
                    continue;
                }
                let cost = ability.cost;
                let mut uses =
                    if cost.tap || cost.sacrifice || ability.once_per_turn || ability.exhaust {
                        1
                    } else {
                        u64::MAX
                    };
                if cost.life > 0 {
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
        if self.convoke {
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
        let mut masks = [0; 32];
        for r in &self.resources {
            let o = &r.object;
            if o.zone == Zone::Battlefield
                && o.controller == self.actor
                && self.kinds(r) & 1 != 0
                && !o.tapped
            {
                masks[ability_world::CharacteristicWorld::colors(self, r.id) as usize & 31] += 1;
            }
        }
        masks
    }
    fn evidence_value(&self) -> u128 {
        let actual: u128 = self
            .game
            .state
            .objects
            .iter()
            .filter(|o| o.owner == self.actor && o.zone == Zone::Graveyard && !o.is_token())
            .map(|o| u128::from(o.card.fixed_mana_value()))
            .sum();
        actual
            + self
                .resources
                .iter()
                .filter(|r| {
                    r.object.owner == self.actor
                        && r.object.zone == Zone::Graveyard
                        && self.game.state.objects[r.id as usize].zone != Zone::Graveyard
                        && !r.object.is_token()
                })
                .map(|r| u128::from(r.object.card.fixed_mana_value()))
                .sum::<u128>()
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
        if !self.obligations(goal) {
            return false;
        }
        let adjusted = cast_origins::payment_goal(self, goal, self.purpose);
        if (if self.convoke {
            convoke::payable_in(
                self,
                goal.actor,
                self.purpose,
                mana_lots::available(self.mana, &self.lots, self.purpose),
                goal.required,
                goal.generic,
                &self.convoke_masks(),
            )
        } else {
            payable(
                mana_lots::available(self.mana, &self.lots, self.purpose),
                adjusted.required,
                adjusted.generic,
            )
        }) && evidence.is_none_or(|n| self.evidence_value() >= u128::from(n))
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
                            && r.object.card.activations().iter().any(|a| {
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
        for step in self.moves(goal) {
            let mut next = self.clone();
            next.apply(&step);
            if next.reachable_with_evidence(goal, evidence, visited) {
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
impl Game {
    pub(super) fn can_fund(&self, goal: FundingGoal) -> bool {
        let purpose = self.funding_purpose(&goal);
        self.can_fund_for(goal, purpose)
    }
    pub(super) fn can_fund_for(&self, goal: FundingGoal, purpose: mana_lots::Purpose) -> bool {
        if let Some(a) = &self.state.announcing
            && !a.additional_paid
            && a.item
                .cast
                .is_some_and(|c| c.method == CastMethod::GraveyardCounterCost)
        {
            return self.creature_counter_fundable(
                goal,
                purpose,
                a.item.source_card.graveyard_counter_cost().unwrap(),
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
            if !group
                .iter()
                .all(|&r| self.additional_candidate(spec, a.item.controller, a.item.object, r))
            {
                return false;
            }
        }
        let object = goal
            .source
            .as_ref()
            .and_then(|(r, _, _, _)| self.state.objects.get(r.card as usize));
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
        ) || !source_obligations(
            &goal,
            object,
            goal.source
                .as_ref()
                .map_or(CounterAmounts::default(), |(r, _, _, _)| {
                    self.counter_amounts(r.card)
                }),
            object.map_or(0, |o| {
                self.object_kinds(goal.source.as_ref().unwrap().0.card, o)
            }),
            || {
                object.map_or(Keywords::NONE, |_| {
                    self.keywords(goal.source.as_ref().unwrap().0.card)
                })
            },
            self.state.players[goal.actor.index()].life,
        ) {
            return false;
        }
        let adjusted = cast_origins::payment_goal(self, &goal, purpose);
        // The common prefunded path needs no resource graph or heap allocations.
        if payable(
            self.available_mana(goal.actor, purpose),
            adjusted.required,
            adjusted.generic,
        ) {
            return true;
        }
        let mut state = FundingState::from_game(self, &goal);
        state.purpose = purpose;
        state.reachable(&goal, &mut HashSet::new())
    }
    // Numeric funding fixtures use the default-purpose convenience wrapper;
    // runtime spell callers always provide their actual face purpose.
    pub(super) fn can_fund_convoke_for(
        &self,
        goal: FundingGoal,
        purpose: mana_lots::Purpose,
    ) -> bool {
        if convoke::payable_in(
            self,
            goal.actor,
            purpose,
            self.available_mana(goal.actor, purpose).map(u128::from),
            goal.required,
            goal.generic,
            &self.convoke_masks(goal.actor, &[]),
        ) {
            return true;
        }
        let mut state = FundingState::from_game_with_convoke(self, &goal, true);
        state.purpose = purpose;
        state.reachable(&goal, &mut HashSet::new())
    }
    pub(super) fn can_fund_evidence_for(
        &self,
        goal: FundingGoal,
        threshold: u16,
        purpose: mana_lots::Purpose,
    ) -> bool {
        if self.can_collect_evidence(goal.actor, threshold) {
            return self.can_fund_for(goal.clone(), purpose);
        }
        let mut state = FundingState::from_game(self, &goal);
        state.purpose = purpose;
        state.reachable_with_evidence(&goal, Some(threshold), &mut HashSet::new())
    }
    pub(super) fn announcement_payment_request(
        &self,
        viewer: Player,
    ) -> Option<PaymentRequestView> {
        if self.actor() != Some(viewer)
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
            || a.needs_kicker()
            || a.needs_gift()
            || a.needs_bargain()
            || a.needs_evidence()
            || a.additional_cost().is_some_and(forage::is_forage_cost) && a.forage.is_none()
            || a.needs_mana_cost()
            || (a.item.source_card.tiered_costs().is_some() && a.item.mode.is_none())
            || (a
                .additional_cost()
                .is_some_and(|spec| !forage::is_forage_cost(spec))
                && !a
                    .additional_cost()
                    .is_some_and(|s| evidence::threshold(s).is_some())
                && a.additional.is_none())
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
    pub(super) fn announcement_cost_fundable(&self, a: &Announcement, cost: ManaCost) -> bool {
        if a.item
            .cast
            .is_some_and(|c| c.method == CastMethod::GraveyardCounterCost)
        {
            let cost = self.reduced_spell_cost(a.item.controller, a.item.card, cost);
            return self.creature_counter_fundable(
                FundingGoal {
                    actor: a.item.controller,
                    required: cost.colored,
                    generic: cost.generic,
                    reserve: None,
                    source: None,
                },
                self.purpose_for_item(&a.item),
                a.item.source_card.graveyard_counter_cost().unwrap(),
                a.creature_counter_cost
                    .as_ref()
                    .map_or(&[], |v| v.as_slice()),
            );
        }
        if a.item.kind == StackKind::Spell
            && a.item.cast.is_some_and(|c| c.method == CastMethod::Free)
        {
            let definition = a.item.source_card;
            let mut total = Self::with_kicker_cost(
                Self::with_offspring_cost(
                    cost,
                    definition,
                    a.item.cast.is_some_and(|c| c.offspring == Some(true)),
                ),
                definition,
                a.item.cast.is_some_and(|c| c.kicker == Some(true)),
            );
            total.generic = total.generic.saturating_sub(
                u32::try_from(self.spell_reduction_for_definition(
                    a.item.controller,
                    a.item.card,
                    definition.fixed_mana_value(),
                    definition,
                ))
                .unwrap_or(u32::MAX),
            );
            return self.spell_can_fund_total_for_definition(
                a.item.controller,
                a.item.card,
                total,
                None,
                definition,
            );
        }
        if a.item.kind == StackKind::Spell
            && !self.spell_cost_option_available(a.item.controller, a.item.card, cost)
        {
            return false;
        }
        if a.item.kind == StackKind::Spell && a.item.source_card.x_symbols() > 0 {
            return a.item.x.is_some_and(|x| {
                self.scalar_x_total(a.item.controller, a.item.card, cost, x)
                    .is_some_and(|total| {
                        self.spell_can_fund_total(a.item.controller, a.item.card, total, None)
                    })
            });
        }
        if a.item.source_card.target_cost_reduction().is_some() && a.item.kind == StackKind::Spell {
            return self.announcement_target_fundable(a, None, Some(cost), None, None, false);
        }
        if a.item.kind == StackKind::Spell && a.item.source_card.collect_evidence_cost().is_some() {
            return self.evidence_cast_fundable(
                a.item.controller,
                a.item.card,
                cost,
                a.item.cast.is_some_and(|c| c.evidence == Some(true)),
            );
        }
        if a.item.kind == StackKind::Spell && a.item.source_card.has_bargain() {
            return self.bargain_cost_fundable(a.item.controller, a.item.card, cost, a.additional);
        }
        match a.item.kind {
            StackKind::Spell => self.spell_can_fund(
                a.item.controller,
                a.item.card,
                Self::with_kicker_cost(
                    Self::with_offspring_cost(
                        cost,
                        a.item.source_card,
                        a.item.cast.is_some_and(|c| c.offspring == Some(true)),
                    ),
                    a.item.source_card,
                    a.item.cast.is_some_and(|c| c.kicker == Some(true)),
                ),
            ),
            StackKind::ActivatedAbility { ability } => self.activation_can_fund(
                FundingGoal {
                    reserve: None,
                    actor: a.item.controller,
                    required: cost.colored,
                    generic: cost.generic,
                    source: Some((
                        a.item.object,
                        ability,
                        a.counter_payment.clone(),
                        a.item.source_card,
                    )),
                },
                a.additional_cost(),
            ),
            StackKind::TriggeredAbility { .. }
            | StackKind::CopyEndStepSacrifice
            | StackKind::DelayedSacrifice
            | StackKind::DelayedReturn
            | StackKind::DelayedWarpExile => false,
        }
    }
    pub(super) fn funding_goal(&self, announcement: &Announcement) -> FundingGoal {
        match announcement.item.kind {
            StackKind::Spell => {
                let cost = announcement
                    .convoke
                    .as_ref()
                    .map(|f| f.remaining)
                    .or(announcement.total_cost)
                    .unwrap_or_else(|| {
                        if announcement.item.source_card.x_symbols() > 0 {
                            return self
                                .scalar_x_total(
                                    announcement.item.controller,
                                    announcement.item.card,
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
                let cost = announcement.item.source_card.activations()[ability as usize].cost;
                FundingGoal {
                    reserve: self.announcement_reserve(announcement),
                    actor: announcement.item.controller,
                    required: announcement.cost.map_or(cost.colored, |c| c.colored),
                    generic: announcement
                        .cost
                        .map_or(cost.generic, |c| c.generic)
                        .checked_add(if announcement.has_x_cost() {
                            announcement.item.x.unwrap_or(0)
                        } else {
                            0
                        })
                        .capacity("activation cost exhausted"),
                    source: Some((
                        announcement.item.object,
                        ability,
                        announcement.counter_payment.clone(),
                        announcement.item.source_card,
                    )),
                }
            }
            StackKind::TriggeredAbility { .. }
            | StackKind::CopyEndStepSacrifice
            | StackKind::DelayedSacrifice
            | StackKind::DelayedReturn
            | StackKind::DelayedWarpExile => unreachable!(),
        }
    }
    /// Complete coupled allocation constraints after every nonmana selection.
    pub fn mana_payment_domain(&self) -> Option<ManaPaymentDomain> {
        if !matches!(
            self.state.pending,
            Pending::PayCosts { .. } | Pending::Convoke { .. }
        ) {
            return None;
        }
        let a = self.state.announcing.as_ref()?;
        let pool = self.available_mana(a.item.controller, self.purpose_for_item(&a.item));
        let (required, generic) = if let Some(frame) = &a.convoke {
            if !frame
                .selected
                .iter()
                .all(|s| self.convoke_eligible(a.item.controller, s.object))
            {
                return None;
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
            let state = FundingState::from_game(self, &goal);
            if !state.obligations(&goal) || a.evidence.as_ref().is_some_and(|e| !e.complete) {
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
    pub(super) fn visit_cost_payment(&self, emit: &mut impl FnMut(Action) -> bool) -> bool {
        let announcement = self.state.announcing.as_ref().unwrap();
        let goal = self.funding_goal(announcement);
        let convoke = announcement.item.kind == StackKind::Spell
            && announcement.item.source_card.has_convoke();
        let state = FundingState::from_game_with_convoke(self, &goal, convoke);
        let mut done = false;
        if state.obligations(&goal)
            && announcement.evidence.as_ref().is_none_or(|e| e.complete)
            && let Some(domain) = self.mana_payment_domain()
        {
            domain.representatives(|payment| {
                if !done {
                    done = emit(Action::PayCosts { payment });
                }
            });
        }
        if done {
            return true;
        }
        if convoke
            && convoke::payable_in(
                &state,
                goal.actor,
                state.purpose,
                mana_lots::available(state.mana, &state.lots, state.purpose),
                goal.required,
                goal.generic,
                &state.convoke_masks(),
            )
            && emit(Action::BeginConvoke)
        {
            return true;
        }
        // An explicit {0} still has a mana component; a pure tap/sacrifice
        // activation cost does not open a mana-ability window.
        if let Some((_, index, _, definition)) = goal.source.as_ref()
            && !definition.activations()[*index as usize]
                .cost
                .mana_component
        {
            return false;
        }
        for step in state.moves(&goal) {
            let mut next = state.clone();
            next.apply(&step);
            if next.reachable_with_evidence(
                &goal,
                announcement.evidence.as_ref().map(|e| e.threshold),
                &mut HashSet::new(),
            ) && emit(state.action(&step))
            {
                return true;
            }
        }
        false
    }
}

impl Game {
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
        let mut done = false;
        funding_payments(
            mana_lots::available(state.mana, &state.lots, state.purpose),
            goal.required,
            goal.generic,
            |payment| {
                if !done {
                    done = emit(Action::PayPlot {
                        payment: concrete_payment(payment),
                    });
                }
            },
        );
        if done {
            return true;
        }
        for step in state.moves(&goal) {
            let mut next = state.clone();
            next.apply(&step);
            if next.reachable(&goal, &mut HashSet::new()) && emit(state.action(&step)) {
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
        if emit(Action::DeclineResolutionMana) {
            return true;
        }
        if request
            .life
            .is_some_and(|life| self.state.players[request.payer.index()].life < i128::from(life))
        {
            return false;
        }
        let goal = FundingGoal {
            reserve: None,
            actor: request.payer,
            required: request.cost.colored,
            generic: request.cost.generic,
            source: None,
        };
        let state = FundingState::from_game(self, &goal);
        let mut done = false;
        funding_payments(
            mana_lots::available(state.mana, &state.lots, state.purpose),
            goal.required,
            goal.generic,
            |payment| {
                if !done {
                    done = emit(Action::PayResolutionMana {
                        payment: concrete_payment(payment),
                    });
                }
            },
        );
        if done {
            return true;
        }
        // A payment request grants a mana-ability window even if the player
        // eventually declines or cannot assemble the whole payment.
        for step in state.moves(&goal) {
            if emit(state.action(&step)) {
                return true;
            }
        }
        false
    }
}

impl Game {
    /// A mana ability in an enclosing guided payment window must leave some
    /// completion of that window. Reuse its exact resource search, retaining
    /// complete plans so later scalar choices cannot mix incompatible prefixes.
    pub(super) fn prepare_mana_allocations(&mut self, action: &Action) {
        let Action::ActivateMana { payment, .. } = action else {
            return;
        };
        let Some(actor) = self.actor() else { return };
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
            Pending::PayCosts { .. } | Pending::Convoke { .. } => {
                let a = self.state.announcing.as_ref().unwrap();
                (
                    self.funding_goal(a),
                    a.item.kind == StackKind::Spell && a.item.source_card.has_convoke(),
                    a.evidence.as_ref().map(|e| e.threshold),
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
            _ => return,
        };
        let world = FundingState::from_game_with_convoke(self, &goal, convoke);
        let mut viable = Vec::new();
        for step in world.moves(&goal) {
            if world.action(&step) != *action {
                continue;
            }
            let mut next = world.clone();
            next.apply(&step);
            if next.reachable_with_evidence(&goal, evidence, &mut HashSet::new()) {
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

type CounterCostFundingKey = (FundingKey, Vec<(ObjectRef, CounterKind, u64, bool)>);

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
        available >= u128::from(amount) - selected.len() as u128
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
        let adjusted = cast_origins::payment_goal(self, goal, self.purpose);
        if payable(
            mana_lots::available(self.mana, &self.lots, self.purpose),
            adjusted.required,
            adjusted.generic,
        ) && self.creature_counter_payment_possible(amount, selected)
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
            let mut next = self.clone();
            next.apply(&step);
            if next.fund_creature_counter_cost(goal, amount, selected, visited) {
                return true;
            }
        }
        false
    }
}
impl Game {
    pub(super) fn creature_counter_fundable(
        &self,
        goal: FundingGoal,
        purpose: mana_lots::Purpose,
        amount: u16,
        selected: &[(ObjectRef, CounterKind)],
    ) -> bool {
        let mut world = FundingState::from_game_with_convoke(self, &goal, true);
        world.convoke = false;
        world.purpose = purpose;
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
