//! Exact reachability for the admitted pure mana sources; no game-state clones,
//! heuristics that discard legal plans, or mutation during action generation.
use super::*;
use std::collections::HashSet;

#[derive(Clone, Copy, Debug)]
pub(super) struct FundingGoal {
    pub actor: Player,
    pub required: Mana,
    pub generic: u32,
    pub source: Option<(ObjectRef, u16, Counters, Card)>,
}
#[derive(Clone)]
struct Resource {
    id: CardId,
    object: Object,
}
#[derive(Clone)]
struct FundingState {
    mana: Mana,
    life: i64,
    resources: Vec<Resource>,
}
#[derive(Clone)]
struct FundingMove {
    resource: usize,
    ability: Option<u16>,
    payment: Mana,
    counters: Counters,
    production: Mana,
}
#[derive(Hash, PartialEq, Eq)]
struct ResourceKey {
    definition: usize,
    protected: bool,
    present: bool,
    tapped: bool,
    sick: bool,
    counters: Counters,
    used: u64,
    gained: Keywords,
    owned: bool,
    identity: Option<CardId>,
    attached: Option<(CardId, u64)>,
}
#[derive(Hash, PartialEq, Eq)]
struct FundingKey {
    mana: Mana,
    life: i64,
    // Equivalent sources are interchangeable for feasibility. Protect the
    // enclosing ability's source separately, even if its printed card matches.
    resources: Vec<ResourceKey>,
}

fn payable(pool: Mana, required: Mana, generic: u32) -> bool {
    if (0..6).any(|index| pool[index] < required[index]) {
        return false;
    }
    pool.iter()
        .zip(required)
        .map(|(&pool, need)| u64::from(pool - need))
        .sum::<u64>()
        >= u64::from(generic)
}
fn source_obligations(
    goal: FundingGoal,
    object: Option<&Object>,
    keywords: Keywords,
    life: i64,
) -> bool {
    let Some((source, index, counters, definition)) = goal.source else {
        return true;
    };
    let ability = &definition.activations()[index as usize];
    if i64::from(ability.cost.life) > life {
        return false;
    }
    let current = object.filter(|object| {
        object.zone == Zone::Battlefield && object.generation == source.generation
    });
    let requires_source =
        ability.cost.tap || ability.cost.sacrifice || ability.cost.remove.is_some();
    if let Some(object) = current {
        Game::object_activation_available(
            object,
            keywords,
            life,
            ability,
            index as usize,
            goal.actor,
            true,
        ) && object.counters.plus_one >= counters.plus_one
            && object.counters.minus_one >= counters.minus_one
            && object.counters.stun >= counters.stun
    } else {
        // The activated ability already exists independently of its source.
        // Only unpaid costs that use that incarnation require it to remain.
        !requires_source
    }
}

impl FundingState {
    fn from_game(game: &Game, goal: FundingGoal) -> Self {
        let resources = game
            .state
            .objects
            .iter()
            .enumerate()
            .filter(|(id, object)| {
                object.zone == Zone::Battlefield
                    && (object.card.attachment_bonus().is_some()
                        || (object.owner == goal.actor
                            && (object.card.mana_color().is_some()
                                || object
                                    .card
                                    .activations()
                                    .iter()
                                    .any(|ability| ability.mana_production().is_some())
                                || goal.source.is_some_and(|(source, _, _, _)| {
                                    source.card == *id as CardId
                                }))))
            })
            .map(|(id, object)| Resource {
                id: id as CardId,
                object: object.clone(),
            })
            .collect();
        Self {
            mana: game.state.players[goal.actor.index()].mana,
            life: game.state.players[goal.actor.index()].life,
            resources,
        }
    }
    fn obligations(&self, goal: FundingGoal) -> bool {
        let object = goal.source.and_then(|(source, _, _, _)| {
            self.resources
                .iter()
                .find(|resource| resource.id == source.card)
                .map(|resource| &resource.object)
        });
        let keywords = goal
            .source
            .and_then(|(source, _, _, _)| self.resources.iter().find(|r| r.id == source.card))
            .map_or(Keywords::NONE, |r| self.keywords(r));
        source_obligations(goal, object, keywords, self.life)
    }
    fn keywords(&self, target: &Resource) -> Keywords {
        let mut keywords = target.object.keywords();
        if target.object.zone == Zone::Battlefield && target.object.card.is_creature() {
            let reference = ObjectRef {
                card: target.id,
                generation: target.object.generation,
            };
            for source in &self.resources {
                if source.object.zone == Zone::Battlefield
                    && !source.object.card.is_creature()
                    && source.object.attached_to == Some(reference)
                    && let Some(spec) = source.object.card.attachment_bonus()
                {
                    keywords = keywords.union(spec.keywords);
                }
            }
        }
        keywords
    }
    fn moves(&self, goal: FundingGoal) -> Vec<FundingMove> {
        let mut moves = Vec::new();
        for (resource, entry) in self.resources.iter().enumerate() {
            let object = &entry.object;
            if object.zone != Zone::Battlefield || object.owner != goal.actor {
                continue;
            }
            if !object.tapped
                && (!object.card.is_creature()
                    || !object.summoning_sick
                    || self.keywords(entry).contains(Keywords::HASTE))
                && let Some(color) = object.card.mana_color()
            {
                let mut production = [0; 6];
                production[color] = 1;
                moves.push(FundingMove {
                    resource,
                    ability: None,
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
                if goal.source.is_some_and(|(source, pending, _, definition)| {
                    definition == object.card
                        && source.card == entry.id
                        && object.generation == source.generation
                        && pending as usize == index
                }) {
                    continue;
                }
                if !Game::object_activation_available(
                    object,
                    self.keywords(entry),
                    self.life,
                    ability,
                    index,
                    goal.actor,
                    false,
                ) {
                    continue;
                }
                activations::counter_payments(ability.cost, object.counters, |counters| {
                    payments(
                        self.mana,
                        ability.cost.colored,
                        ability.cost.generic,
                        |payment| {
                            production.visit(|production| {
                                moves.push(FundingMove {
                                    resource,
                                    ability: Some(index as u16),
                                    payment,
                                    counters,
                                    production,
                                });
                                false
                            });
                        },
                    );
                    false
                });
            }
        }
        // Try producing missing required types first; this changes only traversal
        // order, never the set of accepted paths.
        moves.sort_by_key(|step| {
            std::cmp::Reverse(
                (0..6)
                    .map(|i| {
                        goal.required[i]
                            .saturating_sub(self.mana[i])
                            .min(step.production[i])
                    })
                    .sum::<u32>(),
            )
        });
        moves
    }
    fn apply(&mut self, step: &FundingMove) {
        for (pool, paid) in self.mana.iter_mut().zip(step.payment) {
            *pool -= paid;
        }
        let object = &mut self.resources[step.resource].object;
        if let Some(index) = step.ability {
            let cost = object.card.activations()[index as usize].cost;
            self.life -= i64::from(cost.life);
            object.used_abilities |= 1u64 << index;
            if cost.tap {
                object.tapped = true;
            }
            object.counters.plus_one -= step.counters.plus_one;
            object.counters.minus_one -= step.counters.minus_one;
            object.counters.stun -= step.counters.stun;
            if cost.sacrifice {
                object.zone = Zone::Graveyard;
            }
        } else {
            object.tapped = true;
        }
        for (pool, produced) in self.mana.iter_mut().zip(step.production) {
            *pool = pool
                .checked_add(produced)
                .expect("mana representation exhausted");
        }
    }
    fn key(&self, goal: FundingGoal) -> FundingKey {
        // Attachment edges distinguish otherwise interchangeable mana sources.
        let linked = self
            .resources
            .iter()
            .any(|r| r.object.attached_to.is_some());
        let mut resources: Vec<_> = self
            .resources
            .iter()
            .map(|r| ResourceKey {
                definition: r.object.card.index(),
                protected: goal
                    .source
                    .is_some_and(|(source, _, _, _)| source.card == r.id),
                present: r.object.zone == Zone::Battlefield,
                tapped: r.object.tapped,
                sick: r.object.summoning_sick,
                counters: r.object.counters,
                used: r.object.used_abilities,
                gained: r.object.gained_keywords,
                owned: r.object.owner == goal.actor,
                identity: linked.then_some(r.id),
                attached: r.object.attached_to.map(|r| (r.card, r.generation)),
            })
            .collect();
        resources.sort_by_key(|r| {
            (
                r.definition,
                r.protected,
                r.present,
                r.tapped,
                r.sick,
                r.counters.plus_one,
                r.counters.minus_one,
                r.counters.stun,
                r.used,
                r.gained,
                (r.owned, r.identity, r.attached),
            )
        });
        FundingKey {
            mana: self.mana,
            life: self.life,
            resources,
        }
    }
    fn optimistic(&self, goal: FundingGoal) -> bool {
        let mut upper = self.mana.map(u64::from);
        for resource in &self.resources {
            let object = &resource.object;
            if object.zone != Zone::Battlefield || object.owner != goal.actor {
                continue;
            }
            let tap_ready = !object.tapped
                && (!object.card.is_creature()
                    || !object.summoning_sick
                    || self.keywords(resource).contains(Keywords::HASTE));
            if tap_ready && let Some(color) = object.card.mana_color() {
                upper[color] = upper[color].saturating_add(1);
            }
            for (index, ability) in object.card.activations().iter().enumerate() {
                let Some(production) = ability.mana_production() else {
                    continue;
                };
                if goal.source.is_some_and(|(source, pending, _, definition)| {
                    definition == object.card
                        && source.card == resource.id
                        && object.generation == source.generation
                        && pending as usize == index
                }) || !Game::object_activation_available(
                    object,
                    self.keywords(resource),
                    self.life,
                    ability,
                    index,
                    goal.actor,
                    false,
                ) {
                    continue;
                }
                let cost = ability.cost;
                let mut uses = if cost.tap || cost.sacrifice || ability.once_per_turn {
                    1
                } else {
                    u64::MAX
                };
                if cost.life > 0 {
                    uses = uses.min(self.life.max(0) as u64 / u64::from(cost.life));
                }
                if let Some((kind, amount)) = cost.remove {
                    let count = match kind {
                        Some(CounterKind::PlusOne) => u64::from(object.counters.plus_one),
                        Some(CounterKind::MinusOne) => u64::from(object.counters.minus_one),
                        Some(CounterKind::Stun) => u64::from(object.counters.stun),
                        None => {
                            u64::from(object.counters.plus_one)
                                + u64::from(object.counters.minus_one)
                                + u64::from(object.counters.stun)
                        }
                    };
                    uses = uses.min(count / u64::from(amount));
                }
                let mut maximum = [0u32; 6];
                production.visit(|mana| {
                    for index in 0..6 {
                        maximum[index] = maximum[index].max(mana[index]);
                    }
                    false
                });
                for index in 0..6 {
                    upper[index] =
                        upper[index].saturating_add(uses.saturating_mul(u64::from(maximum[index])));
                }
            }
        }
        if (0..6).any(|index| upper[index] < u64::from(goal.required[index])) {
            return false;
        }
        upper
            .iter()
            .zip(goal.required)
            .fold(0u64, |total, (&bound, required)| {
                total.saturating_add(bound - u64::from(required))
            })
            >= u64::from(goal.generic)
    }
    fn reachable(&self, goal: FundingGoal, visited: &mut HashSet<FundingKey>) -> bool {
        if !self.obligations(goal) {
            return false;
        }
        if payable(self.mana, goal.required, goal.generic) {
            return true;
        }
        if !self.optimistic(goal) {
            return false;
        }
        if !visited.insert(self.key(goal)) {
            return false;
        }
        for step in self.moves(goal) {
            let mut next = self.clone();
            next.apply(&step);
            if next.reachable(goal, visited) {
                return true;
            }
        }
        false
    }
    fn action(&self, step: &FundingMove) -> Action {
        let card = self.resources[step.resource].id;
        match step.ability {
            Some(ability) => Action::ActivateMana {
                card,
                ability,
                payment: step.payment,
                counter_payment: step.counters,
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
            .and_then(|(source, _, _, _)| self.state.objects.get(source.card as usize));
        let keywords = object.map_or(Keywords::NONE, |_| {
            self.keywords(goal.source.unwrap().0.card)
        });
        if !source_obligations(
            goal,
            object,
            keywords,
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
        FundingState::from_game(self, goal).reachable(goal, &mut HashSet::new())
    }
    pub(super) fn funding_goal(&self, announcement: &Announcement) -> FundingGoal {
        match announcement.item.kind {
            StackKind::Spell => {
                let (required, generic) = announcement.item.source_card.cost();
                FundingGoal {
                    actor: announcement.item.controller,
                    required,
                    generic,
                    source: None,
                }
            }
            StackKind::ActivatedAbility { ability } => {
                let cost = announcement.item.source_card.activations()[ability as usize].cost;
                FundingGoal {
                    actor: announcement.item.controller,
                    required: cost.colored,
                    generic: cost.generic,
                    source: Some((
                        announcement.item.object,
                        ability,
                        announcement.counter_payment,
                        announcement.item.source_card,
                    )),
                }
            }
            StackKind::TriggeredAbility { .. } => unreachable!(),
        }
    }
    pub(super) fn visit_cost_payment(&self, emit: &mut impl FnMut(Action) -> bool) -> bool {
        let announcement = self.state.announcing.as_ref().unwrap();
        let goal = self.funding_goal(announcement);
        let state = FundingState::from_game(self, goal);
        let mut done = false;
        if state.obligations(goal) {
            payments(state.mana, goal.required, goal.generic, |payment| {
                if !done {
                    done = emit(Action::PayCosts { payment });
                }
            });
        }
        if done {
            return true;
        }
        // An explicit {0} still has a mana component; a pure tap/sacrifice
        // activation cost does not open a mana-ability window.
        if let Some((_, index, _, definition)) = goal.source
            && !definition.activations()[index as usize].cost.mana_component
        {
            return false;
        }
        for step in state.moves(goal) {
            let mut next = state.clone();
            next.apply(&step);
            if next.reachable(goal, &mut HashSet::new()) && emit(state.action(&step)) {
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
            actor: Player::P0,
            required: [0, 0, 0, 0, 1, 0],
            generic: 0,
            source: None,
        };
        let mut state = FundingState::from_game(&g, goal);
        assert!(state.reachable(goal, &mut HashSet::new()));
        let initial_key = state.key(goal);
        state
            .resources
            .iter_mut()
            .find(|r| r.id == boots)
            .unwrap()
            .object
            .zone = Zone::Graveyard;
        assert!(state.key(goal) != initial_key);
        assert!(!state.reachable(goal, &mut HashSet::new()));
        assert!(
            !state
                .moves(goal)
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
            actor: Player::P0,
            required: [0, 0, 0, 0, 1, 0],
            generic: 0,
            source: None,
        };
        let state = FundingState::from_game(&g, goal);
        let mut changed = state.clone();
        changed
            .resources
            .iter_mut()
            .find(|r| r.id == boots)
            .unwrap()
            .object
            .attached_to = Some(g.object_ref(b));
        assert!(state.key(goal) != changed.key(goal));
    }
}
