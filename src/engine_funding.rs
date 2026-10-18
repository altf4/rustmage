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
    keywords: Keywords,
    life: i128,
) -> bool {
    let Some((source, index, counters, definition)) = goal.source.as_ref() else {
        return true;
    };
    let ability = &definition.activations()[*index as usize];
    if i128::from(ability.cost.life) > life {
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
                        if let ManaProduction::CountSubtype { subtype, .. } = production
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
                        && (object.card.attachment_bonus().is_some()
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
        reserve_obligations(
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
            source.map_or(Keywords::NONE, |r| self.keywords(r)),
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
                        self.keywords(entry),
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
                        funding_cost_payments(self.mana, ability.cost, |payment| {
                            production.in_world(self, entry.id).visit(|production| {
                                moves.push(FundingMove {
                                    resource,
                                    ability: Some(index as u16),
                                    grantor: None,
                                    payment,
                                    counters: counters.clone(),
                                    production,
                                });
                                false
                            });
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
    fn apply(&mut self, step: &FundingMove) {
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
                identity: linked.then_some(r.id),
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
    fn optimistic(&self, goal: &FundingGoal) -> bool {
        let required =
            u128::from(goal.generic) + goal.required.into_iter().map(u128::from).sum::<u128>();
        if self.total_mana_upper() < required {
            return false;
        }
        // A virtual source departure can restore an ability under layer6.
        // The total upper bound remains conservative; color/readiness pruning
        // must not assume the current blanking is permanent for this search.
        if ability_world::CharacteristicWorld::has_ability_removal(self) {
            return true;
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
                        self.keywords(resource),
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
        if (if self.convoke {
            convoke::payable(
                self.mana,
                goal.required,
                goal.generic,
                &self.convoke_masks(),
            )
        } else {
            payable(self.mana, goal.required, goal.generic)
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
            object.map_or(Keywords::NONE, |_| {
                self.keywords(goal.source.as_ref().unwrap().0.card)
            }),
            self.state.players[goal.actor.index()].life,
        ) {
            return false;
        }
        // The common prefunded path needs no resource graph or heap allocations.
        if payable(
            self.state.players[goal.actor.index()].mana,
            goal.required,
            goal.generic,
        ) {
            return true;
        }
        FundingState::from_game(self, &goal).reachable(&goal, &mut HashSet::new())
    }
    pub(super) fn can_fund_convoke(&self, goal: FundingGoal) -> bool {
        if convoke::payable(
            self.state.players[goal.actor.index()].mana.map(u128::from),
            goal.required,
            goal.generic,
            &self.convoke_masks(goal.actor, &[]),
        ) {
            return true;
        }
        FundingState::from_game_with_convoke(self, &goal, true)
            .reachable(&goal, &mut HashSet::new())
    }
    pub(super) fn can_fund_evidence(&self, goal: FundingGoal, threshold: u16) -> bool {
        if self.can_collect_evidence(goal.actor, threshold) {
            return self.can_fund(goal.clone());
        }
        FundingState::from_game(self, &goal).reachable_with_evidence(
            &goal,
            Some(threshold),
            &mut HashSet::new(),
        )
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
            || (a.item.source_card.x_symbols() > 0 && a.item.x.is_none())
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
        Some(PaymentRequestView {
            required: goal.required,
            generic: goal.generic,
            counters: a.counter_payment.clone(),
            production: a.production,
        })
    }
    pub(super) fn announcement_cost_fundable(&self, a: &Announcement, cost: ManaCost) -> bool {
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
            | StackKind::DelayedSacrifice
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
                    generic: announcement.cost.map_or(cost.generic, |c| c.generic),
                    source: Some((
                        announcement.item.object,
                        ability,
                        announcement.counter_payment.clone(),
                        announcement.item.source_card,
                    )),
                }
            }
            StackKind::TriggeredAbility { .. }
            | StackKind::DelayedSacrifice
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
        let pool = self.state.players[a.item.controller.index()].mana;
        let (required, generic) = if let Some(frame) = &a.convoke {
            if !frame
                .selected
                .iter()
                .all(|s| self.convoke_eligible(a.item.controller, s.object))
            {
                return None;
            }
            (frame.remaining.colored, frame.remaining.generic)
        } else {
            let goal = self.funding_goal(a);
            let state = FundingState::from_game(self, &goal);
            if !state.obligations(&goal) || a.evidence.as_ref().is_some_and(|e| !e.complete) {
                return None;
            }
            (goal.required, goal.generic)
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
            && convoke::payable(
                state.mana,
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
        funding_payments(state.mana, goal.required, goal.generic, |payment| {
            if !done {
                done = emit(Action::PayPlot {
                    payment: concrete_payment(payment),
                });
            }
        });
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
        let goal = FundingGoal {
            reserve: None,
            actor: request.payer,
            required: request.cost.colored,
            generic: request.cost.generic,
            source: None,
        };
        let state = FundingState::from_game(self, &goal);
        let mut done = false;
        funding_payments(state.mana, goal.required, goal.generic, |payment| {
            if !done {
                done = emit(Action::PayResolutionMana {
                    payment: concrete_payment(payment),
                });
            }
        });
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

#[cfg(test)]
mod equipment_tests {
    use super::super::tests::{scenario, spawn};
    use super::*;
    #[test]
    fn guarded_mana_retains_nonmana_witnesses_and_recomputes_after_virtual_departure() {
        let mut g = scenario();
        let mox = spawn(
            &mut g,
            Player::P0,
            Card::from_name("Mox Jasper").unwrap(),
            Zone::Battlefield,
        );
        let dragon = spawn(
            &mut g,
            Player::P0,
            Card::from_name("Rooftop Percher").unwrap(),
            Zone::Battlefield,
        );
        let goal = FundingGoal {
            reserve: None,
            actor: Player::P0,
            required: [1, 0, 0, 0, 0, 0],
            generic: 0,
            source: None,
        };
        let mut state = FundingState::from_game(&g, &goal);
        assert!(state.resources.iter().any(|r| r.id == dragon));
        assert!(state.reachable(&goal, &mut HashSet::new()));
        let key = state.key(&goal);
        state
            .resources
            .iter_mut()
            .find(|r| r.id == dragon)
            .unwrap()
            .object
            .zone = Zone::Graveyard;
        assert!(key != state.key(&goal));
        assert!(!state.reachable(&goal, &mut HashSet::new()));
        assert!(state.moves(&goal).is_empty());
        // Starting restrictions are not rechecked as source-cost obligations
        // after an ability has already legally been announced.
        let declared = FundingGoal {
            reserve: None,
            actor: Player::P0,
            required: [0; 6],
            generic: 0,
            source: Some((
                g.object_ref(mox),
                0,
                Counters::default(),
                g.state.objects[mox as usize].card,
            )),
        };
        assert!(state.obligations(&declared));
        assert!(!state.guard_available(
            g.state.objects[mox as usize].card.activations()[0].guard,
            Player::P0
        ));
    }
    #[test]
    fn unguarded_plans_do_not_retain_nonmana_subtype_witnesses() {
        let mut g = scenario();
        let dragon = spawn(
            &mut g,
            Player::P0,
            Card::from_name("Rooftop Percher").unwrap(),
            Zone::Battlefield,
        );
        let goal = FundingGoal {
            reserve: None,
            actor: Player::P0,
            required: [0; 6],
            generic: 0,
            source: None,
        };
        assert!(
            !FundingState::from_game(&g, &goal)
                .resources
                .iter()
                .any(|r| r.id == dragon)
        );
    }
    #[test]
    fn planning_recomputes_continuous_haste_when_source_leaves() {
        let mut g = scenario();
        let elf = spawn(&mut g, Player::P0, Card::LlanowarElves, Zone::Battlefield);
        g.state.objects[elf as usize].summoning_sick = true;
        let samut = spawn(
            &mut g,
            Player::P0,
            Card::from_name("Samut, Hazoret's Champion").unwrap(),
            Zone::Battlefield,
        );
        let goal = FundingGoal {
            reserve: None,
            actor: Player::P0,
            required: [0, 0, 0, 0, 1, 0],
            generic: 0,
            source: None,
        };
        let mut state = FundingState::from_game(&g, &goal);
        assert!(state.reachable(&goal, &mut HashSet::new()));
        for resource in &state.resources {
            assert_eq!(state.keywords(resource), g.keywords(resource.id));
        }
        let before = state.key(&goal);
        state
            .resources
            .iter_mut()
            .find(|r| r.id == samut)
            .unwrap()
            .object
            .zone = Zone::Graveyard;
        assert!(before != state.key(&goal));
        assert!(!state.reachable(&goal, &mut HashSet::new()));
        assert!(state.moves(&goal).is_empty());
    }
    #[test]
    fn funding_keywords_match_live_turn_attacker_artifact_and_tribal_scopes() {
        let mut g = scenario();
        for name in [
            "Bearer of Glory",
            "Feisty Spikeling",
            "Regal Caracal",
            "Boldwyr Aggressor",
            "Krang, Utrom Warlord",
            "Okoye, Dora Milaje Leader",
            "Samut, Hazoret's Champion",
            "Three Tree Mascot",
            "Llanowar Elves",
        ] {
            for owner in [Player::P0, Player::P1] {
                spawn(
                    &mut g,
                    owner,
                    Card::from_name(name).unwrap(),
                    Zone::Battlefield,
                );
            }
        }
        let token = Card::catalogue()
            .find(|c| c.token_info().is_some_and(|t| t.name == "Soldier Token"))
            .unwrap();
        let id = spawn(&mut g, Player::P0, token, Zone::Battlefield);
        g.state.attackers.push(Attacker {
            defender: Some(CombatDefender::Player(Player::P1)),
            card: id,
            blocked: false,
        });
        let goal = FundingGoal {
            reserve: None,
            actor: Player::P0,
            required: [0; 6],
            generic: 0,
            source: Some((g.object_ref(id), 0, Counters::default(), token)),
        };
        // The protected source is retained even without a mana ability. No
        // activation goal is evaluated here: compare pure derived keywords.
        for active in [Player::P0, Player::P1] {
            g.state.active = active;
            let state = FundingState::from_game(&g, &goal);
            for resource in &state.resources {
                assert_eq!(
                    state.keywords(resource),
                    g.keywords(resource.id),
                    "{} {active:?}",
                    resource.object.card.name()
                );
            }
        }
    }
    #[test]
    fn planning_recomputes_haste_after_supporting_equipment_leaves() {
        let mut g = scenario();
        let elf = spawn(&mut g, Player::P0, Card::LlanowarElves, Zone::Battlefield);
        g.state.objects[elf as usize].summoning_sick = true;
        let boots = spawn(
            &mut g,
            Player::P1,
            Card::from_name("Swiftfoot Boots").unwrap(),
            Zone::Battlefield,
        );
        assert!(g.attach_equipment(g.object_ref(boots), g.object_ref(elf)));
        let goal = FundingGoal {
            reserve: None,
            actor: Player::P0,
            required: [0, 0, 0, 0, 1, 0],
            generic: 0,
            source: None,
        };
        let mut state = FundingState::from_game(&g, &goal);
        assert!(state.reachable(&goal, &mut HashSet::new()));
        let initial_key = state.key(&goal);
        state
            .resources
            .iter_mut()
            .find(|r| r.id == boots)
            .unwrap()
            .object
            .zone = Zone::Graveyard;
        assert!(state.key(&goal) != initial_key);
        assert!(!state.reachable(&goal, &mut HashSet::new()));
        assert!(
            !state
                .moves(&goal)
                .iter()
                .any(|m| state.resources[m.resource].id == elf)
        );
    }
    #[test]
    fn attachment_edges_distinguish_otherwise_identical_resources_in_funding_keys() {
        let mut g = scenario();
        let a = spawn(&mut g, Player::P0, Card::LlanowarElves, Zone::Battlefield);
        let b = spawn(&mut g, Player::P0, Card::LlanowarElves, Zone::Battlefield);
        let boots = spawn(
            &mut g,
            Player::P0,
            Card::from_name("Swiftfoot Boots").unwrap(),
            Zone::Battlefield,
        );
        assert!(g.attach_equipment(g.object_ref(boots), g.object_ref(a)));
        let goal = FundingGoal {
            reserve: None,
            actor: Player::P0,
            required: [0, 0, 0, 0, 1, 0],
            generic: 0,
            source: None,
        };
        let state = FundingState::from_game(&g, &goal);
        let mut changed = state.clone();
        changed
            .resources
            .iter_mut()
            .find(|r| r.id == boots)
            .unwrap()
            .object
            .attached_to = Some(g.object_ref(b));
        assert!(state.key(&goal) != changed.key(&goal));
    }
}

#[cfg(test)]
mod hybrid_cost_tests {
    use super::*;
    #[test]
    fn counterfactual_mana_steps_accept_cheaper_two_brid_and_deduplicate_shared_hybrid_allocations()
    {
        // Explicit future-cost IR; no partial Standard card is admitted by this fixture.
        let mut cost = Card::from_name("Cindering Cutthroat")
            .unwrap()
            .activations()[0]
            .cost;
        cost.colored = [0; 6];
        cost.generic = 2;
        cost.alternatives = &[ManaCost {
            colored: [1, 0, 0, 0, 0, 0],
            generic: 0,
        }];
        let mut found = Vec::new();
        funding_cost_payments([1, 0, 0, 0, 0, 0], cost, |payment| found.push(payment));
        assert_eq!(found, vec![[1, 0, 0, 0, 0, 0]]);
        cost.colored = [0, 0, 1, 0, 0, 0];
        cost.generic = 1;
        cost.alternatives = &[ManaCost {
            colored: [0, 0, 0, 1, 0, 0],
            generic: 1,
        }];
        found.clear();
        funding_cost_payments([0, 0, 1, 1, 0, 0], cost, |payment| found.push(payment));
        assert_eq!(found, vec![[0, 0, 1, 1, 0, 0]]);
    }
}

#[cfg(test)]
mod own_turn_guard_tests {
    use super::*;
    #[test]
    fn virtual_guard_respects_active_player_without_a_board_witness() {
        // Future mana-cost IR uses the same guard helper; neither new printed
        // ability in this stage produces mana.
        for active in [Player::P0, Player::P1] {
            let game = super::super::tests::scenario();
            let state = FundingState {
                game: &game,
                actor: Player::P0,
                life_gained: [false; 2],
                active,
                mana: [0; 6],
                life: 20,
                resources: Vec::new(),
                named_counters: None,
                convoke: false,
            };
            assert!(state.guard_available(ActivationGuard::OwnTurn, active));
            assert!(!state.guard_available(ActivationGuard::OwnTurn, active.other()));
            assert!(state.guard_available(ActivationGuard::Always, active.other()));
            assert!(
                !state.guard_available(ActivationGuard::ControlSubtypeAny(&["Dragon"]), active)
            );
        }
    }
}

#[cfg(test)]
mod life_gain_grant_tests {
    use super::*;
    use crate::engine::tests::{scenario, spawn};
    #[test]
    fn virtual_keyword_queries_share_the_live_controller_history_condition() {
        for gained in [false, true] {
            let mut g = scenario();
            let elf = spawn(
                &mut g,
                Player::P0,
                Card::from_name("Llanowar Elves").unwrap(),
                Zone::Battlefield,
            );
            spawn(
                &mut g,
                Player::P0,
                Card::from_name("Thornfist Striker").unwrap(),
                Zone::Battlefield,
            );
            spawn(
                &mut g,
                Player::P1,
                Card::from_name("Thornfist Striker").unwrap(),
                Zone::Battlefield,
            );
            g.state.players[0].life_gained_this_turn = if gained { u128::MAX } else { 0 };
            g.state.players[1].life_gained_this_turn = 1;
            let state = FundingState::from_game(
                &g,
                &FundingGoal {
                    actor: Player::P0,
                    required: [0; 6],
                    generic: 1,
                    reserve: None,
                    source: None,
                },
            );
            let target = state.resources.iter().find(|r| r.id == elf).unwrap();
            assert_eq!(state.keywords(target), g.keywords(elf));
            assert_eq!(state.keywords(target).contains(Keywords::TRAMPLE), gained);
        }
    }
}

#[cfg(test)]
mod exhaust_funding_tests {
    use super::*;
    use crate::engine::tests::{act, scenario, spawn};
    #[test]
    fn exhausted_mana_cannot_be_reused_by_virtual_search_even_after_untap_or_later_turns() {
        let mut g = scenario();
        let loot = spawn(
            &mut g,
            Player::P0,
            Card::from_name("Loot, the Pathfinder").unwrap(),
            Zone::Battlefield,
        );
        let forest = spawn(&mut g, Player::P0, Card::Forest, Zone::Battlefield);
        let goal = FundingGoal {
            actor: Player::P0,
            required: [0, 3, 0, 0, 0, 0],
            generic: 0,
            reserve: None,
            source: None,
        };
        let saved = g.snapshot();
        assert!(g.can_fund(goal.clone()));
        assert_eq!(g.snapshot(), saved);
        assert!(!g.can_fund(FundingGoal {
            required: [0, 4, 0, 0, 0, 0],
            ..goal.clone()
        }));
        act(&mut g, Action::TapMana { card: forest });
        let a=g.legal_actions().into_iter().find(|a|matches!(a,Action::ActivateMana{card,ability:0,production:[0,3,0,0,0,0],..}if *card==loot)).unwrap();
        act(&mut g, a);
        assert_eq!(g.object_view(loot).exhausted_abilities, 1);
        g.finish_cleanup();
        g.finish_cleanup();
        g.state.players[0].mana = [0; 6]; // normal step advancement empties pools
        assert!(!g.object_view(loot).tapped);
        assert!(!g.can_fund(goal.clone()));
        g.move_object(loot, Zone::Hand);
        g.move_object(loot, Zone::Battlefield);
        assert_eq!(g.object_view(loot).exhausted_abilities, 0);
        assert!(g.can_fund(goal.clone()));
    }
}

#[cfg(test)]
mod conditional_keyword_tests {
    use super::super::tests::{act, scenario, spawn};
    use super::*;
    fn named(g: &mut Game, p: Player, n: &str, z: Zone) -> CardId {
        let c = Card::from_name(n).unwrap();
        assert!(c.is_supported());
        spawn(g, p, c, z)
    }
    fn goal(p: Player) -> FundingGoal {
        FundingGoal {
            actor: p,
            required: [0, 0, 0, 1, 0, 0],
            generic: 0,
            reserve: None,
            source: None,
        }
    }
    #[test]
    fn borrowed_overlay_matches_actual_conditional_keywords_after_real_nontoken_mana_sacrifice() {
        for p in [Player::P0, Player::P1] {
            let mut g = scenario();
            g.state.active = p;
            g.state.priority = p;
            g.state.players[p.index()].mana = [0, 0, 0, 0, 0, 1];
            let tomb = named(&mut g, p, "Goblin Tomb Raider", Zone::Battlefield);
            let spine = named(&mut g, p, "Spineseeker Centipede", Zone::Battlefield);
            let pizza = named(&mut g, p, "Omni-Cheese Pizza", Zone::Battlefield);
            for n in ["Forest", "Opt", "Bear Cub"] {
                named(&mut g, p, n, Zone::Graveyard);
            }
            let saved = g.snapshot();
            let mut state = FundingState::from_game(&g, &goal(p));
            let before = state.key(&goal(p));
            assert!(
                state
                    .keywords(state.resources.iter().find(|r| r.id == tomb).unwrap())
                    .contains(Keywords::HASTE)
            );
            assert!(
                !state
                    .keywords(state.resources.iter().find(|r| r.id == spine).unwrap())
                    .contains(Keywords::VIGILANCE)
            );
            let step = state
                .moves(&goal(p))
                .into_iter()
                .find(|m| state.resources[m.resource].id == pizza && m.production[3] == 1)
                .unwrap();
            let action = state.action(&step);
            state.apply(&step);
            assert!(state.key(&goal(p)) != before);
            let mut actual = g.clone();
            act(&mut actual, action);
            for id in [tomb, spine] {
                let r = state.resources.iter().find(|r| r.id == id).unwrap();
                assert_eq!(state.keywords(r), actual.keywords(id));
            }
            assert!(
                !state
                    .keywords(state.resources.iter().find(|r| r.id == tomb).unwrap())
                    .contains(Keywords::HASTE)
            );
            assert!(
                state
                    .keywords(state.resources.iter().find(|r| r.id == spine).unwrap())
                    .contains(Keywords::VIGILANCE)
            );
            assert_eq!(g.snapshot(), saved);
        }
    }
    #[test]
    fn token_sacrifice_removes_artifact_witness_without_adding_a_graveyard_card_type() {
        for p in [Player::P0, Player::P1] {
            let mut g = scenario();
            g.state.active = p;
            g.state.priority = p;
            let tomb = named(&mut g, p, "Goblin Tomb Raider", Zone::Battlefield);
            let spine = named(&mut g, p, "Spineseeker Centipede", Zone::Battlefield);
            for n in ["Forest", "Opt", "Bear Cub"] {
                named(&mut g, p, n, Zone::Graveyard);
            }
            let c = Card::catalogue()
                .find(|c| c.token_info().is_some_and(|t| t.subtypes == "Treasure"))
                .unwrap();
            let treasure = spawn(&mut g, p, c, Zone::Battlefield);
            let mut state = FundingState::from_game(&g, &goal(p));
            let step = state
                .moves(&goal(p))
                .into_iter()
                .find(|m| state.resources[m.resource].id == treasure && m.production[3] == 1)
                .unwrap();
            let action = state.action(&step);
            state.apply(&step);
            let mut actual = g.clone();
            act(&mut actual, action);
            assert!(
                !state
                    .keywords(state.resources.iter().find(|r| r.id == tomb).unwrap())
                    .contains(Keywords::HASTE)
            );
            assert!(
                !state
                    .keywords(state.resources.iter().find(|r| r.id == spine).unwrap())
                    .contains(Keywords::VIGILANCE)
            );
            for id in [tomb, spine] {
                assert_eq!(
                    state.keywords(state.resources.iter().find(|r| r.id == id).unwrap()),
                    actual.keywords(id)
                );
            }
        }
    }
    #[test]
    fn runtime_copy_funding_keys_distinguish_original_and_token_sacrifice_evidence() {
        for p in [Player::P0, Player::P1] {
            let mut g = scenario();
            g.state.active = p;
            g.state.priority = p;
            let definition = Card::from_name("Omni-Cheese Pizza").unwrap();
            let original = named(&mut g, p, definition.name(), Zone::Battlefield);
            g.stage_copy_creation(p, CopyRecipe { definition }, 1);
            super::super::ability_timeline_tests::settle(&mut g);
            let token = g
                .state
                .objects
                .iter()
                .enumerate()
                .find(|(_, o)| o.copy_token && o.card == definition && o.zone == Zone::Battlefield)
                .map(|(id, _)| id as CardId)
                .unwrap();
            for id in [original, token] {
                g.state.objects[id as usize].summoning_sick = false;
            }
            g.state.players[p.index()].mana = [0, 0, 0, 0, 0, 1];
            let before = g.snapshot();
            let goal = goal(p);
            let state = FundingState::from_game(&g, &goal);
            let sacrifice = |id| {
                let step = state
                    .moves(&goal)
                    .into_iter()
                    .find(|m| state.resources[m.resource].id == id && m.production[3] == 1)
                    .unwrap();
                let mut next = state.clone();
                next.apply(&step);
                next
            };
            let physical_sacrificed = sacrifice(original);
            let token_sacrificed = sacrifice(token);
            assert_eq!(physical_sacrificed.mana, token_sacrificed.mana);
            assert!(
                physical_sacrificed.key(&goal) != token_sacrificed.key(&goal),
                "physical definition equality does not make tokens interchangeable with cards"
            );
            let evidence = u128::from(definition.fixed_mana_value());
            assert!(evidence > 0);
            assert_eq!(physical_sacrificed.evidence_value(), evidence);
            assert_eq!(token_sacrificed.evidence_value(), 0);
            for fork in [&physical_sacrificed, &token_sacrificed] {
                assert!(fork.reachable_with_evidence(
                    &goal,
                    Some(evidence as u16),
                    &mut HashSet::new()
                ));
            }
            assert_eq!(g.snapshot(), before);
        }
    }
    #[test]
    fn one_shared_evaluator_covers_all_quantities_and_virtual_actor_life_without_mutation() {
        const BF: CountFilter = CountFilter {
            zone: CountZone::Battlefield,
            relation: 0,
            kind: 0,
            exclude: 0,
            subtype: None,
            basic: false,
            other: false,
            cards_only: false,
        };
        const GRAVE: CountFilter = CountFilter {
            zone: CountZone::Graveyard,
            cards_only: true,
            ..BF
        };
        let mut g = scenario();
        let source = named(&mut g, Player::P0, "Goblin Tomb Raider", Zone::Battlefield);
        named(&mut g, Player::P0, "Three Tree Mascot", Zone::Battlefield);
        named(&mut g, Player::P0, "Forest", Zone::Battlefield);
        for n in ["Opt", "Anthem of Champions", "Scorpion Sentinel", "Forest"] {
            named(&mut g, Player::P0, n, Zone::Graveyard);
        }
        let s = g.snapshot();
        let mut state = FundingState::from_game(&g, &goal(Player::P0));
        let resource = state.resources.iter().find(|r| r.id == source).unwrap();
        for value in [
            StatValue::ControllerLife,
            StatValue::SpellsCastThisTurn,
            StatValue::LifeGainOccurredThisTurn,
            StatValue::Count(BF),
            StatValue::Count(GRAVE),
            StatValue::DistinctNames(BF),
            StatValue::Colors(BF),
            StatValue::CardTypes(GRAVE),
            StatValue::GreatestManaValue(BF),
            StatValue::GraveyardCardTypes,
            StatValue::CountUnion(&[BF, GRAVE]),
            StatValue::Sum(&[StatValue::Count(BF), StatValue::Count(GRAVE)]),
            StatValue::AtLeast {
                value: &StatValue::Count(GRAVE),
                minimum: 4,
            },
        ] {
            assert_eq!(
                state.stat_value(value, resource),
                g.stat_value(value, g.object_ref(source), Player::P0)
            );
        }
        state.life = 7;
        let r = state.resources.iter().find(|r| r.id == source).unwrap();
        assert_eq!(state.stat_value(StatValue::ControllerLife, r), 7);
        assert_eq!(g.snapshot(), s);
    }

    #[test]
    fn source_predicate_overlay_tracks_virtual_counters_attachments_life_history_and_attack_commit()
    {
        for p in [Player::P0, Player::P1] {
            let mut g = scenario();
            g.state.active = p;
            g.state.priority = p;
            let spider = named(&mut g, p, "Skyward Spider", Zone::Battlefield);
            let paladin = named(&mut g, p, "Twinblade Paladin", Zone::Battlefield);
            let kite = named(&mut g, p, "Kitesail Corsair", Zone::Battlefield);
            let pick = named(&mut g, p.other(), "Goldvein Pick", Zone::Battlefield);
            g.state.objects[pick as usize].attached_to = Some(g.object_ref(spider));
            g.state.objects[spider as usize].counters.plus_one = 1;
            g.state.players[p.index()].life = 25;
            g.state.players[p.index()].cards_drawn_this_turn = 2;
            g.state.attackers.push(AttackDeclaration {
                card: kite,
                defender: Some(CombatDefender::Player(p.other())),
                blocked: false,
            });
            let mut state = FundingState::from_game(&g, &goal(p));
            for id in [spider, paladin, kite] {
                let r = state.resources.iter().find(|r| r.id == id).unwrap();
                for value in [
                    StatValue::SourceModified,
                    StatValue::SourceAttacking,
                    StatValue::CardsDrawnThisTurn,
                    StatValue::Not(&StatValue::SpellsCastThisTurn),
                ] {
                    assert_eq!(
                        state.stat_value(value, r),
                        g.stat_value(value, g.object_ref(id), p)
                    );
                }
            }
            let spider_r = state.resources.iter().position(|r| r.id == spider).unwrap();
            state.resources[spider_r].object.counters.plus_one = 0;
            assert!(
                state
                    .keywords(&state.resources[spider_r])
                    .contains(Keywords::FLYING)
            );
            state
                .resources
                .iter_mut()
                .find(|r| r.id == pick)
                .unwrap()
                .object
                .zone = Zone::Graveyard;
            assert!(
                !state
                    .keywords(&state.resources[spider_r])
                    .contains(Keywords::FLYING)
            );
            let mut actual = g.clone();
            actual.state.objects[spider as usize].counters.plus_one = 0;
            actual.move_object(pick, Zone::Graveyard);
            assert_eq!(
                state.keywords(&state.resources[spider_r]),
                actual.keywords(spider)
            );
            state.life = 24;
            let r = state.resources.iter().find(|r| r.id == paladin).unwrap();
            assert!(!state.keywords(r).contains(Keywords::DOUBLE_STRIKE));
            assert!(g.keywords(paladin).contains(Keywords::DOUBLE_STRIKE));
            assert!(
                !state
                    .keywords(state.resources.iter().find(|r| r.id == kite).unwrap())
                    .contains(Keywords::FLYING)
            );
            drop(state);
            g.state.attack_declared = true;
            let state = FundingState::from_game(&g, &goal(p));
            assert!(
                state
                    .keywords(state.resources.iter().find(|r| r.id == kite).unwrap())
                    .contains(Keywords::FLYING)
            );
        }
    }
    #[test]
    fn actual_and_virtual_treasure_sacrifice_disable_druid_token_bonus_without_mutating_original() {
        for p in [Player::P0, Player::P1] {
            let mut g = scenario();
            g.state.active = p;
            g.state.priority = p;
            let druid = named(&mut g, p, "Druid of the Spade", Zone::Battlefield);
            let token = Card::catalogue()
                .find(|c| c.token_info().is_some_and(|t| t.subtypes == "Treasure"))
                .unwrap();
            let treasure = spawn(&mut g, p, token, Zone::Battlefield);
            let saved = g.snapshot();
            let mut state = FundingState::from_game(&g, &goal(p));
            let step = state
                .moves(&goal(p))
                .into_iter()
                .find(|m| state.resources[m.resource].id == treasure)
                .unwrap();
            let action = state.action(&step);
            state.apply(&step);
            let mut actual = g.clone();
            act(&mut actual, action);
            let r = state.resources.iter().find(|r| r.id == druid).unwrap();
            assert_eq!(state.keywords(r), actual.keywords(druid));
            assert!(!state.keywords(r).contains(Keywords::TRAMPLE));
            assert!(g.keywords(druid).contains(Keywords::TRAMPLE));
            assert_eq!(g.snapshot(), saved);
        }
    }
}

#[cfg(test)]
mod celebration_funding_tests {
    use super::*;
    use crate::engine::tests::{scenario, spawn};
    #[test]
    fn virtual_keywords_share_exact_historical_celebration_predicate() {
        for p in [Player::P0, Player::P1] {
            for n in [0, 1, 2, u64::MAX] {
                let mut g = scenario();
                g.state.players[p.index()].nonland_permanents_entered_this_turn = n;
                for name in ["Gallant Pie-Wielder", "Grand Ball Guest", "Tuinvale Guide"] {
                    let id = spawn(&mut g, p, Card::from_name(name).unwrap(), Zone::Battlefield);
                    let state = FundingState::from_game(
                        &g,
                        &FundingGoal {
                            actor: p,
                            required: [0; 6],
                            generic: 1,
                            reserve: None,
                            source: None,
                        },
                    );
                    let resource = state.resources.iter().find(|r| r.id == id).unwrap();
                    assert_eq!(state.keywords(resource), g.keywords(id));
                    let keyword = match name {
                        "Gallant Pie-Wielder" => Keywords::DOUBLE_STRIKE,
                        "Grand Ball Guest" => Keywords::TRAMPLE,
                        _ => Keywords::LIFELINK,
                    };
                    assert_eq!(state.keywords(resource).contains(keyword), n >= 2)
                }
            }
        }
    }
}

#[cfg(test)]
mod job_subtype_tests {
    use super::super::tests::{scenario, spawn};
    use super::*;
    #[test]
    fn job_overlay_removes_exact_grant_before_reserved_subtype_payment() {
        let mut g = scenario();
        let bear = spawn(&mut g, Player::P0, Card::GrizzlyBears, Zone::Battlefield);
        let sword = spawn(
            &mut g,
            Player::P0,
            Card::from_name("Warrior's Sword").unwrap(),
            Zone::Battlefield,
        );
        assert!(g.attach_equipment(g.object_ref(sword), g.object_ref(bear)));
        let spec = AdditionalCost {
            kind: EffectChoiceKind::Sacrifice,
            filter: SacrificeFilter {
                keywords: Keywords::NONE,
                source_only: false,
                kinds: 1,
                nonland: false,
                other: false,
                tokens: None,
                artifact_enchantment_or_token: false,
                subtype: Some("Warrior"),
            },
        };
        let goal = FundingGoal {
            actor: Player::P0,
            required: [0; 6],
            generic: 0,
            source: None,
            reserve: Some((g.object_ref(bear), spec)),
        };
        assert!(g.can_fund(goal.clone()));
        let mut virtual_state = FundingState::from_game(&g, &goal);
        assert!(virtual_state.granted_subtypes(bear).contains("Warrior"));
        assert!(virtual_state.obligations(&goal));
        virtual_state
            .resources
            .iter_mut()
            .find(|r| r.id == sword)
            .unwrap()
            .object
            .zone = Zone::Graveyard;
        assert!(!virtual_state.granted_subtypes(bear).contains("Warrior"));
        assert!(!virtual_state.obligations(&goal));
        assert!(g.has_subtype(bear, "Warrior"));
    }
}

#[cfg(test)]
#[path = "engine_funding_total_tests.rs"]
mod total_tests;

#[cfg(test)]
mod ability_world_tests {
    use super::super::tests::{scenario, spawn};
    use super::*;
    use crate::engine::ability_world::CharacteristicWorld;

    #[test]
    fn funding_ability_world_uses_virtual_source_zone_controller_and_exact_incarnation() {
        for p in [Player::P0, Player::P1] {
            let mut g = scenario();
            let maha = spawn(
                &mut g,
                p,
                Card::from_name("Maha, Its Feathers Night").unwrap(),
                Zone::Battlefield,
            );
            let bear = spawn(&mut g, p.other(), Card::GrizzlyBears, Zone::Battlefield);
            let source = g.object_ref(maha);
            let effect = g.state.objects[maha as usize].card.live_static_effects()[0];
            let spec = AdditionalCost {
                kind: EffectChoiceKind::Sacrifice,
                filter: SacrificeFilter {
                    keywords: Keywords::NONE,
                    source_only: false,
                    kinds: 1,
                    nonland: false,
                    other: false,
                    tokens: None,
                    artifact_enchantment_or_token: false,
                    subtype: None,
                },
            };
            let goal = FundingGoal {
                actor: p,
                required: [0; 6],
                generic: 0,
                source: None,
                reserve: Some((source, spec)),
            };
            let mut world = FundingState::from_game(&g, &goal);
            assert!(world.live_applies(source, &effect, bear));
            let provider = world.resources.iter().position(|r| r.id == maha).unwrap();
            world.resources[provider].object.controller = p.other();
            assert!(!world.live_applies(source, &effect, bear));
            world.resources[provider].object.controller = p;
            world.resources[provider].object.zone = Zone::Graveyard;
            assert!(!world.live_applies(source, &effect, bear));
            world.resources[provider].object.zone = Zone::Battlefield;
            world.resources[provider].object.generation += 1;
            assert!(!world.live_applies(source, &effect, bear));
            assert!(g.live_applies(source, &effect, bear));
            assert_eq!(g.state.objects[maha as usize].controller, p);
        }
    }
    #[test]
    fn fdn_library_eight_command_emblem_keywords_match_owned_funding_world() {
        for p in [Player::P0, Player::P1] {
            let mut g = scenario();
            let elf = spawn(
                &mut g,
                p,
                Card::from_name("Llanowar Elves").unwrap(),
                Zone::Battlefield,
            );
            let emblem = Card::catalogue().nth(5338).unwrap();
            g.create_emblem(p, emblem);
            let state = FundingState::from_game(
                &g,
                &FundingGoal {
                    actor: p,
                    required: [0; 6],
                    generic: 0,
                    reserve: None,
                    source: None,
                },
            );
            let resource = state.resources.iter().find(|r| r.id == elf).unwrap();
            assert_eq!(state.keywords(resource), g.keywords(elf));
            assert!(state.keywords(resource).contains(Keywords::INDESTRUCTIBLE));
        }
    }
}

#[cfg(test)]
#[path = "engine_fdn_scaled_mana_tests.rs"]
mod fdn_six_scaled_tests;

#[cfg(test)]
#[path = "engine_subtype_layers_world_tests.rs"]
mod subtype_layers_world_tests;
#[cfg(test)]
mod named_counter_funding_tests {
    use super::super::ability_timeline_tests::{named, setup};
    use super::*;
    #[test]
    fn named_counter_reservations_use_speculative_counts_and_distinct_keys() {
        for player in [Player::P0, Player::P1] {
            let mut game = setup(player);
            let source = named(&mut game, player, "Gnarlbark Elm", Zone::Battlefield);
            game.set_counters(source, Counters::of(CounterKind::Soul, 2));
            let goal = FundingGoal {
                actor: player,
                required: [0; 6],
                generic: 0,
                reserve: None,
                source: Some((
                    game.object_ref(source),
                    0,
                    Counters::of(CounterKind::Soul, 2),
                    game.state.objects[source as usize].card,
                )),
            };
            assert!(game.can_fund(goal.clone()));
            let mut world = FundingState::from_game(&game, &goal);
            assert!(world.obligations(&goal));
            let before = world.key(&goal);
            let mut counters =
                ability_world::CharacteristicWorld::counter_amounts(&world, source).to_owned();
            counters.remove(CounterKind::Soul, 1);
            live_counters::set(
                &mut world.named_counters,
                game.object_ref(source),
                counters.named,
            );
            assert!(!world.obligations(&goal));
            assert!(before != world.key(&goal));
            assert_eq!(game.object_view(source).counters.get(CounterKind::Soul), 2);
            assert!(game.can_fund(goal));
        }
    }
}

#[cfg(test)]
#[path = "engine_entry_choices_world_tests.rs"]
mod entry_choices_world_tests;

#[cfg(test)]
#[path = "engine_qualified_hexproof_world_tests.rs"]
mod qualified_hexproof_world_tests;
