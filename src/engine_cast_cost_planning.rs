//! Completion search for declared nonmana clauses in the borrowed funding world.
use super::*;
use ability_world::CharacteristicWorld;

#[derive(Clone)]
pub(super) struct CostContext {
    actor: Player,
    source: ObjectRef,
    source_counter_reservation: Counters,
    clauses: Vec<CastCostRow>,
    active: Option<u16>,
    casualty: Vec<CasualtyDeclaration>,
    additional: Option<AdditionalCost>,
    additional_life: bool,
    forage: Option<ForageRoute>,
    selected: Vec<ObjectRef>,
    exact: bool,
    counters: Vec<(ObjectRef, CounterKind)>,
    mana: Option<ManaCost>,
    teammates: Vec<ObjectRef>,
    teamwork_exact: bool,
}
impl CostContext {
    fn from_announcement(a: &Announcement) -> Option<Box<Self>> {
        let c = a.cast_costs.as_ref()?;
        let mut selected = a.additional_group.as_deref().cloned().unwrap_or_default();
        if selected.is_empty() {
            selected.extend(a.additional);
        }
        let mut exact = a.additional.is_some();
        if let Some(f) = &a.forage {
            selected = f.selected.clone();
            exact = f.complete();
        }
        if let Some(e) = &a.evidence {
            selected = e.selected.clone();
            exact = e.complete;
        }
        let mut source_counter_reservation = Counters::default();
        if let StackKind::ActivatedAbility { ability } = a.item.kind
            && a.activation_cost_route != ActivationCostRoute::FirstEquipZero
        {
            source_counter_reservation = a.counter_payment.clone();
            if let Some(change) = a
                .item
                .source.expect("activated ability has a source")
                .card
                .activation(ability)
                .cost
                .loyalty
                .filter(|change| *change < 0)
            {
                source_counter_reservation
                    .add(CounterKind::Loyalty, u64::from(change.unsigned_abs()));
            }
        }
        Some(Box::new(Self {
            source_counter_reservation,
            teammates: c.teammates.clone(),
            teamwork_exact: false,
            actor: a.item.controller,
            mana: a
                .convoke
                .as_ref()
                .map(|c| c.remaining)
                .or(a.total_cost)
                .or(a.cost),
            source: a.item.source.expect("cost announcement has object source").object,
            clauses: c.clauses.clone(),
            active: c.active,
            casualty: a.casualty.as_deref().cloned().unwrap_or_default(),
            additional: a.additional_cost(),
            additional_life: a.additional_life,
            forage: a.forage.as_ref().and_then(|v| v.route),
            selected,
            exact,
            counters: a
                .creature_counter_cost
                .as_deref()
                .cloned()
                .unwrap_or_default(),
        }))
    }
}
impl FundingState<'_> {
    /// Reject an unpaid additional cost only if even an overestimate of the
    /// resources available after funding cannot satisfy any of its routes.
    /// Unknown costs and worlds with linked or nested state changes retain the
    /// complete search. Counts deliberately ignore filters, control, names,
    /// counters and tapping, all of which can change during funding.
    pub(super) fn additional_cost_deficit(&self) -> bool {
        let Some(c) = &self.cast_costs else {
            return false;
        };
        let Some(spec) = c.additional else {
            return false;
        };
        if !c
            .clauses
            .iter()
            .any(|r| !r.paid && r.clause == CastCostClause::Additional)
            || !self.game.state.exile_links.is_empty()
            || self.continuation.is_some()
            || self.entry_projection.is_some()
        {
            return false;
        }
        // These clauses can consume resources, tap, add counters, or move a
        // battlefield object to a graveyard. Without linked returns they cannot
        // increase life or introduce a card to hand or a new battlefield object.
        // Keep this exhaustive so new ordered clauses require a bound audit.
        for row in &c.clauses {
            match row.clause {
                CastCostClause::Additional
                | CastCostClause::Mana
                | CastCostClause::Life(_)
                | CastCostClause::AlternativeLife(_)
                | CastCostClause::Teamwork(_)
                | CastCostClause::Blight { .. }
                | CastCostClause::Casualty(_)
                | CastCostClause::Harmonize(_) => {}
            }
        }
        let mut hand = 0usize;
        let mut battlefield = 0usize;
        let mut graveyard = 0usize;
        for r in &self.resources {
            match r.object.zone {
                Zone::Hand if r.object.owner == c.actor && self.reference(r.id) != c.source => {
                    hand += 1;
                }
                Zone::Battlefield => battlefield += 1,
                Zone::Graveyard => graveyard += 1,
                _ => {}
            }
        }
        // Mana payments may sacrifice battlefield objects into graveyards. We
        // count every object there even if it is an opponent's, a token, or not
        // actually sacrificable, and ignore all card-characteristic filters.
        let potential_graveyard = graveyard.saturating_add(battlefield);
        let possible = match spec.kind {
            EffectChoiceKind::Discard
            | EffectChoiceKind::DiscardNamed(_)
            | EffectChoiceKind::DiscardMatchingLegendary
            | EffectChoiceKind::DiscardLegendary => hand > 0,
            EffectChoiceKind::DiscardOrLife(life) => hand > 0 || self.life >= i128::from(life),
            EffectChoiceKind::Behold | EffectChoiceKind::SacrificeOrDiscard { .. } => {
                battlefield > 0 || hand > 0
            }
            EffectChoiceKind::Sacrifice
            | EffectChoiceKind::SacrificeAttachedEquipment
            | EffectChoiceKind::ReturnUnblockedAttacker
            | EffectChoiceKind::ReturnToHand => battlefield > 0,
            EffectChoiceKind::SacrificeDifferentNames(n) | EffectChoiceKind::TapPermanents(n) => {
                battlefield >= usize::from(n)
            }
            EffectChoiceKind::ExileGraveyard => potential_graveyard > 0,
            EffectChoiceKind::ExileGraveyardCards(n) => potential_graveyard >= usize::from(n),
            // Any one potential card is treated as sufficient for a positive
            // weight. Its future face, colors and value need not be stable.
            EffectChoiceKind::CollectEvidence(n)
            | EffectChoiceKind::ExileGraveyardSymbols { minimum: n, .. } => {
                n == 0 || potential_graveyard > 0
            }
            EffectChoiceKind::RemoveCreatureCounters(n) => n == 0 || battlefield > 0,
            EffectChoiceKind::Forage | EffectChoiceKind::ForageOrBlack => match c.forage {
                Some(ForageRoute::Food) => battlefield > 0,
                Some(ForageRoute::ExileThree) => potential_graveyard >= 3,
                Some(ForageRoute::Black) | None => true,
            },
            _ => true,
        };
        !possible
    }
    pub(super) fn set_cost_context(&mut self, a: Option<&Announcement>) {
        self.install_cost_context(a.and_then(CostContext::from_announcement));
        if let Some(a) = a.filter(|a| a.cast_costs.is_some())
            && let Some(frame) = &a.convoke
        {
            // A contributor is reserved during selection and becomes tapped in
            // the speculative world. The live group commits only with payment.
            for selected in frame.objects() {
                if let Some(r) = self
                    .resources
                    .iter_mut()
                    .find(|r| r.id == selected.card && r.object.generation == selected.generation)
                {
                    r.object.tapped = true;
                }
            }
        }
    }
    pub(super) fn set_payment_context(&mut self, a: &Announcement) {
        if a.cast_costs.is_none() && a.additional_cost().is_some() {
            let mut shadow = a.clone();
            shadow.cast_costs = Some(Box::default());
            shadow.initialize_cast_costs(matches!(self.purpose, mana_lots::Purpose::Spell(spell) if improvise::potential(self, spell.card)));
            self.set_cost_context(Some(&shadow));
        } else {
            self.set_cost_context(Some(a));
        }
    }
    fn install_cost_context(&mut self, context: Option<Box<CostContext>>) {
        self.cast_costs = context;
        if self.cast_costs.is_some() {
            self.power_sensitive = true;
            // All live candidates and witnesses must follow virtual departures.
            // Hidden hands remain internal to the acting player's cost search.
            for (id, o) in self.game.state.objects.iter().enumerate() {
                if !self.resources.iter().any(|r| r.id == id as CardId) {
                    self.resources.push(Resource {
                        id: id as CardId,
                        object: o.clone(),
                        attacking: self
                            .game
                            .state
                            .attackers
                            .iter()
                            .any(|r| r.card == id as CardId),
                    });
                }
            }
        }
    }
    pub(super) fn cost_object(
        &self,
        spec: AdditionalCost,
        r: ObjectRef,
        source: ObjectRef,
    ) -> bool {
        let o = self.object(r.card);
        if spec.kind == EffectChoiceKind::SacrificeAttachedEquipment
            && o.attached_to != Some(source)
        {
            return false;
        }
        let spec = spec.in_zone(o.zone);
        (spec.kind != EffectChoiceKind::ReturnUnblockedAttacker
            || (self.game.sneak_attacker(self.actor, r) && self.attacking(r.card)))
            && self.reference(r.card) == r
            && (r != source
                || !matches!(
                    spec.kind,
                    EffectChoiceKind::Discard
                        | EffectChoiceKind::DiscardNamed(_)
                        | EffectChoiceKind::DiscardMatchingLegendary
                        | EffectChoiceKind::DiscardLegendary
                ) && !spec.filter.other)
            && (!spec.filter.source_only || r == source)
            && (spec.kind != EffectChoiceKind::DiscardLegendary || self.legendary(r.card))
            && (spec.kind != EffectChoiceKind::DiscardMatchingLegendary
                || additional_costs::legendary_name_match(self, self.actor, r.card))
            && Game::additional_object_matches(
                spec,
                self.actor,
                o,
                CharacteristicWorld::kinds(self, r.card),
                self.keywords_for_cost(r.card),
                self.subtypes(r.card),
                self.name(r.card),
            )
    }
    fn keywords_for_cost(&self, id: CardId) -> Keywords {
        self.resources
            .iter()
            .find(|r| r.id == id)
            .map_or(Keywords::NONE, |r| self.keywords(r))
    }
    pub(super) fn cost_departures(
        &self,
        objects: &[ObjectRef],
        requested: Zone,
        emit: &mut dyn FnMut(&Self) -> bool,
    ) -> bool {
        let destinations = objects
            .iter()
            .map(|r| {
                let o = self.object(r.card);
                if requested != Zone::Graveyard {
                    return vec![requested];
                }
                let own = o.card.zone_replacement();
                let shuffle = own == Some(crate::cards::ZoneReplacementRule::RevealShuffleSelf)
                    && (o.zone != Zone::Battlefield || self.printed_abilities_available(r.card));
                let exile = self.static_sources().iter().any(|&id| {
                    let p = self.object(id);
                    p.zone == Zone::Battlefield
                        && self.printed_abilities_available(id)
                        && match p.card.zone_replacement() {
                            Some(crate::cards::ZoneReplacementRule::ExileAllCardsAndTokens) => true,
                            Some(
                                crate::cards::ZoneReplacementRule::ExileOpposingCreatureDeath {
                                    ..
                                } | crate::cards::ZoneReplacementRule::ExileOpposingCreatureDeathPlain,
                            ) => {
                                o.zone == Zone::Battlefield
                                    && CharacteristicWorld::kinds(self, r.card) & 1 != 0
                                    && p.controller != o.controller
                            }
                            Some(crate::cards::ZoneReplacementRule::ExileInstantSorceryCards) => {
                                CharacteristicWorld::kinds(self, r.card) & 12 != 0 && !o.is_token()
                            }
                            _ => false,
                        }
                }) || (o.zone == Zone::Battlefield
                    && (self.counter_count(r.card, CounterKind::Finality) > 0
                        || self
                            .game
                            .state
                            .death_exile_replacements
                            .as_ref()
                            .is_some_and(|rules| {
                                rules.iter().any(|rule| {
                                    rule.object == *r && rule.expires_turn == self.game.state.turn
                                })
                            })));
                match (shuffle, exile) {
                    (true, true) => vec![Zone::Library, Zone::Exile],
                    (true, false) => vec![Zone::Library],
                    (false, true) => vec![Zone::Exile],
                    _ => vec![Zone::Graveyard],
                }
            })
            .collect::<Vec<_>>();
        let mut initial = self.clone();
        // These costs move one simultaneous group. Freeze all source facts
        // before any member or a layer provider leaves that world.
        for object in objects {
            initial.remember_payment_departure(object.card);
        }
        let mut worlds = vec![initial];
        for (r, choices) in objects.iter().zip(destinations) {
            let mut next = Vec::new();
            for w in worlds {
                for &zone in &choices {
                    let mut n = w.clone();
                    let i = n
                        .resources
                        .iter()
                        .position(|v| v.id == r.card)
                        .expect("cost resource retained");
                    let before = &n.resources[i].object;
                    if before.zone == Zone::Battlefield
                        && zone == Zone::Graveyard
                        && CharacteristicWorld::kinds(&n, r.card) & 1 != 0
                    {
                        let controller = before.controller;
                        n.controlled_creatures_died[controller.index()] = n
                            .controlled_creatures_died[controller.index()]
                        .checked_add(1)
                        .capacity("virtual controlled death history exhausted");
                        n.creatures_died = n
                            .creatures_died
                            .checked_add(1)
                            .capacity("virtual death history exhausted");
                    }
                    if before.zone == Zone::Graveyard {
                        n.graveyard_left[before.owner.index()] = true;
                    }
                    let face_down_physical = n.projected_face_down_physical(*r);
                    n.resources[i].object.zone = zone;
                    n.resources[i].object.generation = n.resources[i]
                        .object
                        .generation
                        .checked_add(1)
                        .capacity("virtual generation exhausted");
                    // Departure facts above retain the old face; the new object
                    // outside stack/battlefield has only its physical front.
                    let object = &mut n.resources[i].object;
                    n.game.restore_projected_copy_identity(*r, object);
                    if let Some(physical) = face_down_physical {
                        object.card = physical.front_face();
                    } else if !object.is_token() && !object.is_spell_copy() {
                        object.card = object.card.front_face();
                    }
                    if let Some(projection) = &mut n.entry_projection {
                        projection.manifested.retain(|(source, _)| *source != *r);
                    }
                    live_counters::set(&mut n.named_counters, *r, None);
                    n.resources[i].object.counters = InitialCounters::default();
                    n.resources[i].object.growth = Stats::default();
                    n.resources[i].object.attached_to = None;
                    n.resources[i].object.controller = n.resources[i].object.owner;
                    n.resources[i].object.summoning_sick = false;
                    n.resources[i].object.tapped = false;
                    n.resources[i].object.damage = 0;
                    n.resources[i].object.damaged_this_turn = false;
                    n.resources[i].object.dealt_damage_this_turn = false;
                    n.resources[i].object.dealt_combat_damage_this_incarnation = false;
                    n.resources[i].object.deathtouch_since_sba = false;
                    n.resources[i].object.gained_keywords = Keywords::NONE;
                    n.resources[i].object.used_abilities = 0;
                    n.resources[i].object.loyalty_activated_this_turn = false;
                    let after = n.reference(r.card);
                    n.project_copy_departure(*r, after);
                    n.resources[i].object.flashback = false;
                    n.resources[i].object.cast_for_warp = false;
                    n.resources[i].object.class_level = 1;
                    n.resources[i].attacking = false;
                    next.push(n);
                }
            }
            worlds = next;
        }
        worlds.iter_mut().any(|world| {
            // All departures in the cost happen simultaneously. Recompute layer
            // two once afterward, before linked returns or the next cost can use
            // a creature whose former control effect has just left.
            world.refresh_control_projection();
            world.refresh_enduring_projection();
            world.visit_linked_returns(emit)
        })
    }
    pub(super) fn set_cost_counter_prefix(&mut self, selected: &[(ObjectRef, CounterKind)]) {
        if let Some(c) = &mut self.cast_costs {
            c.counters = selected.to_vec();
        }
    }
    pub(in crate::engine) fn cost_completion(&self) -> bool {
        let Some(context) = &self.cast_costs else {
            return true;
        };
        let unpaid = context
            .clauses
            .iter()
            .enumerate()
            .filter(|(_, r)| !r.paid)
            .map(|(i, _)| i)
            .collect::<Vec<_>>();
        self.complete_cost_rows(context, &unpaid)
    }
    fn complete_cost_rows(&self, c: &CostContext, remaining: &[usize]) -> bool {
        if remaining.is_empty() {
            return true;
        }
        // With only Teamwork and Mana left, these inner costs merely tap
        // contributors. They cannot add mana or restore/change a type, color,
        // provider or mana-type permission. The current Hall test is therefore
        // an upper bound before any teammates are spent. A successful bound
        // retains the complete ordered search and every live power evaluation.
        // Other costs may remove suppression or return providers, so they must
        // keep the general path even when mana is not currently payable.
        if remaining
            .iter()
            .any(|&i| matches!(c.clauses[i].clause, CastCostClause::Teamwork(_)))
            && remaining
                .iter()
                .any(|&i| c.clauses[i].clause == CastCostClause::Mana)
            && remaining.iter().all(|&i| {
                matches!(
                    c.clauses[i].clause,
                    CastCostClause::Teamwork(_) | CastCostClause::Mana
                )
            })
            && !self.mana_clause_payable(c)
        {
            return false;
        }
        for (position, &index) in remaining.iter().enumerate() {
            if c.active.is_some_and(|i| usize::from(i) != index) {
                continue;
            }
            let mut rest = remaining.to_vec();
            rest.remove(position);
            let mut later = c.clone();
            later.active = None;
            if c.clauses[index].clause == CastCostClause::Additional {
                later.selected.clear();
                later.exact = false;
                later.counters.clear();
            }
            match c.clauses[index].clause {
                CastCostClause::Teamwork(minimum) => {
                    if self.complete_teamwork_clause(c, &later, &rest, minimum) {
                        return true;
                    }
                }
                CastCostClause::Blight { amount, selected } => {
                    for resource in &self.resources {
                        let reference = self.reference(resource.id);
                        if selected.is_some_and(|wanted| wanted != reference) {
                            continue;
                        }
                        if self.visit_blight_cost_projection(reference, amount, &mut |next| {
                            next.complete_cost_rows(&later, &rest)
                        }) {
                            return true;
                        }
                    }
                }
                CastCostClause::Mana => {
                    if self.complete_convoke_clause(c, &later, &rest) {
                        return true;
                    }
                }
                CastCostClause::Life(amount) | CastCostClause::AlternativeLife(amount) => {
                    if amount == 0 || self.life >= i128::from(amount) {
                        let mut next = self.clone();
                        next.life -= i128::from(amount);
                        next.life_lost[next.actor.index()] |= amount > 0;
                        if next.complete_cost_rows(&later, &rest) {
                            return true;
                        }
                    }
                }
                CastCostClause::Casualty(instance) => {
                    let row = &c.casualty[usize::from(instance)];
                    for resource in &self.resources {
                        let r = self.reference(resource.id);
                        let o = &resource.object;
                        if row.sacrifice.is_some_and(|wanted| wanted != r)
                            || o.zone != Zone::Battlefield
                            || o.controller != c.actor
                            || CharacteristicWorld::kinds(self, resource.id) & 1 == 0
                            || self.power(resource.id) < i128::from(row.grant.minimum)
                        {
                            continue;
                        }
                        if self.cost_departures(&[r], Zone::Graveyard, &mut |w| {
                            w.complete_cost_rows(&later, &rest)
                        }) {
                            return true;
                        }
                    }
                }
                CastCostClause::Harmonize(object) => {
                    if harmonize::contribution(self, c.actor, object).is_some()
                        && let Some(index) = self.resources.iter().position(|r| {
                            r.id == object.card && r.object.generation == object.generation
                        })
                    {
                        let mut next = self.clone();
                        next.resources[index].object.tapped = true;
                        if next.complete_cost_rows(&later, &rest) {
                            return true;
                        }
                    }
                }
                CastCostClause::Additional => {
                    if self.complete_additional(c, &later, &rest) {
                        return true;
                    }
                }
            }
        }
        false
    }
    pub(in crate::engine) fn set_teamwork_prefix(&mut self, selected: &[ObjectRef], exact: bool) {
        let c = self.cast_costs.as_mut().expect("teamwork cost context");
        c.teammates = selected.to_vec();
        c.teamwork_exact = exact;
    }
    fn complete_teamwork_clause(
        &self,
        c: &CostContext,
        later: &CostContext,
        rest: &[usize],
        minimum: u16,
    ) -> bool {
        let eligible = |r: ObjectRef| {
            let o = self.object(r.card);
            self.reference(r.card) == r
                && o.zone == Zone::Battlefield
                && o.controller == c.actor
                && !o.tapped
                && CharacteristicWorld::kinds(self, r.card) & 1 != 0
        };
        if !c.teammates.iter().all(|&r| eligible(r)) {
            return false;
        }
        let candidates: Vec<_> = self
            .resources
            .iter()
            .map(|r| self.reference(r.id))
            .filter(|&r| eligible(r))
            .collect();
        let mut selected = c.teammates.clone();
        self.complete_teamwork_choices(
            later,
            rest,
            minimum,
            &candidates,
            0,
            &mut selected,
            c.teamwork_exact,
        )
    }
    #[allow(clippy::too_many_arguments)]
    fn complete_teamwork_choices(
        &self,
        later: &CostContext,
        rest: &[usize],
        minimum: u16,
        candidates: &[ObjectRef],
        at: usize,
        selected: &mut Vec<ObjectRef>,
        exact: bool,
    ) -> bool {
        let mut sum = saddle::PowerSum::default();
        for r in selected.iter() {
            sum.add(self.power(r.card));
        }
        if sum.enough(minimum) {
            let mut next = self.clone();
            for r in selected.iter() {
                let i = next.resources.iter().position(|v| v.id == r.card).unwrap();
                next.resources[i].object.tapped = true;
            }
            if next.complete_cost_rows(later, rest) {
                return true;
            }
        }
        if exact {
            return false;
        }
        let mut maximum = sum;
        for r in &candidates[at..] {
            if !selected.contains(r) {
                maximum.add(self.power(r.card).max(0));
            }
        }
        if !maximum.enough(minimum) {
            return false;
        }
        for i in at..candidates.len() {
            let r = candidates[i];
            if selected.contains(&r) {
                continue;
            }
            selected.push(r);
            if self.complete_teamwork_choices(
                later,
                rest,
                minimum,
                candidates,
                i + 1,
                selected,
                false,
            ) {
                return true;
            }
            selected.pop();
        }
        false
    }
    pub(super) fn has_ordered_convoke(&self) -> bool {
        self.cast_costs.as_ref().is_some_and(|c| {
            c.clauses
                .iter()
                .any(|r| !r.paid && r.clause == CastCostClause::Mana)
        })
    }
    /// A deliberately narrow negative bound across future mana-source moves.
    /// Unknown providers and non-tap mutations retain the complete search.
    pub(super) fn pure_tap_teamwork_deficit(&self, goal: &FundingGoal) -> bool {
        let Some(c) = &self.cast_costs else {
            return false;
        };
        let mut minimum = None;
        for row in c.clauses.iter().filter(|row| !row.paid) {
            match row.clause {
                CastCostClause::Teamwork(n) => minimum = Some(minimum.unwrap_or(0).max(n)),
                CastCostClause::Mana => (),
                _ => return false,
            }
        }
        let Some(minimum) = minimum else {
            return false;
        };
        let mut upper = saddle::PowerSum::default();
        for resource in &self.resources {
            let o = &resource.object;
            if o.zone == Zone::Battlefield
                && o.controller == c.actor
                && !o.tapped
                && CharacteristicWorld::kinds(self, resource.id) & 1 != 0
            {
                let power = self.power(resource.id);
                if power > 0 {
                    upper.add(power);
                }
            }
        }
        if upper.enough(minimum) {
            return false;
        }
        if goal.source.is_some()
            || goal.reserve.is_some()
            || self.continuation.is_some()
            || self.entry_projection.is_some()
            || self.controls().is_some()
            || self.payment_source.is_some()
            || self.cost_group.is_some()
            || !self.sacrifice_group.is_empty()
            || self.game.state.borrowed_links.is_some()
            || !self.game.state.exile_links.is_empty()
        {
            return false;
        }
        if self
            .roles()
            .is_some_and(|r| !r.base_stats.is_empty() || !r.live_static.is_empty())
        {
            return false;
        }
        // Inspect every resource and every immutable mana activation, including
        // hidden/nonbattlefield/unavailable routes. Pure taps cannot introduce
        // a new route, recipient, incarnation or counter configuration.
        for resource in &self.resources {
            let card = resource.object.card;
            if self
                .mana_activations(resource.id)
                .any(|(_, a, borrowed)| borrowed.is_some() || !Self::only_tap_mana(a))
                || CharacteristicWorld::granted_mana(self, resource.id)
                    .iter()
                    .any(|g| g.sacrifice)
            {
                return false;
            }
            if resource.object.zone != Zone::Battlefield {
                continue;
            }
            if card.dynamic_stats().is_some()
                || !card.scaling_modifiers().is_empty()
                || !card.static_modifiers().is_empty()
                || !card.enchanted_modifiers().is_empty()
                || card.base_stat_setter().is_some()
                || card
                    .static_ability_groups()
                    .iter()
                    .flat_map(|g| g.fragments)
                    .any(|f| {
                        !matches!(
                            f,
                            crate::cards::StaticAbilityFragment::IntrinsicKeywords(_)
                                | crate::cards::StaticAbilityFragment::Changeling
                                | crate::cards::StaticAbilityFragment::AttachmentStats(_)
                                | crate::cards::StaticAbilityFragment::AttachmentKeywords(_)
                                | crate::cards::StaticAbilityFragment::AttachmentTrigger(_)
                                | crate::cards::StaticAbilityFragment::SpellImprovise(_)
                                | crate::cards::StaticAbilityFragment::LandMana { .. }
                                | crate::cards::StaticAbilityFragment::NoMaximumHandSize
                                | crate::cards::StaticAbilityFragment::MaximumHandSize(_)
                                | crate::cards::StaticAbilityFragment::SpendAnyManaForCreatures
                        )
                    })
            {
                return false;
            }
        }
        // Ordinary Teamwork keeps its mana in FundingGoal rather than an ordered
        // Mana row. Fixed attachment bonuses/keywords/trigger grants do not
        // change from pure tap/AddMana moves; no attachment can move here.
        // In this flat class base exceptions, fixed animation/base values,
        // counters, growth, choices and copy layers remain immutable. No mana
        // step changes turn/combat/attachment state. Further taps can only
        // remove a candidate from this exact positive-power upper sum.
        true
    }
    fn only_tap_mana(a: &ActivatedAbility) -> bool {
        let c = a.cost;
        a.origin == ActivationOrigin::Battlefield
            && c.tap
            && !c.sacrifice
            && !c.exile_self
            && !c.discard
            && c.life == 0
            && c.remove.is_none()
            && c.loyalty.is_none()
            && c.additional.is_none()
            && !a.exhaust
            && !a.once_per_turn
            && matches!(a.program, [Instruction::AddMana(_)])
    }
    fn mana_clause_payable(&self, c: &CostContext) -> bool {
        let Some(cost) = c.mana else {
            return false;
        };
        convoke::payable_in(
            self,
            c.actor,
            self.purpose,
            mana_lots::available(self.mana, &self.lots, self.purpose),
            cost.colored,
            cost.generic,
            &self.convoke_masks(),
        )
    }
    fn complete_convoke_clause(
        &self,
        c: &CostContext,
        later: &CostContext,
        rest: &[usize],
    ) -> bool {
        let Some(cost) = c.mana else {
            return false;
        };
        let pool = mana_lots::available(self.mana, &self.lots, self.purpose);
        // Fail before assigning individual helpers when the current mana clause
        // cannot be paid at all. If it is last, exact Hall matching is the whole
        // obligation; later nonmana costs still use the original assignment walk.
        if !self.mana_clause_payable(c) {
            return false;
        }
        if rest.is_empty() {
            return true;
        }
        let (required, generic) = cast_origins::payment_requirements(
            self,
            c.actor,
            self.purpose,
            cost.colored,
            cost.generic,
        );
        if payable(pool, required, generic) && self.complete_cost_rows(later, rest) {
            return true;
        }
        for resource in &self.resources {
            if resource.object.zone != Zone::Battlefield
                || resource.object.controller != c.actor
                || resource.object.tapped
            {
                continue;
            }
            let colors = CharacteristicWorld::colors(self, resource.id);
            let kinds = CharacteristicWorld::kinds(self, resource.id);
            let choices: &[ConvokeContribution] = if self.convoke && kinds & 1 != 0 {
                &convoke::contributions()
            } else if self.improvise_available() && kinds & 16 != 0 {
                &[ConvokeContribution::Generic]
            } else {
                continue;
            };
            for &contribution in choices {
                let mut remaining = cost;
                if !contribution.apply(colors, &mut remaining) {
                    continue;
                }
                let mut next = self.clone();
                let i = next
                    .resources
                    .iter()
                    .position(|r| r.id == resource.id)
                    .unwrap();
                next.resources[i].object.tapped = true;
                let mut context = c.clone();
                context.mana = Some(remaining);
                if next.complete_convoke_clause(&context, later, rest) {
                    return true;
                }
            }
        }
        false
    }
    pub(super) fn mark_mana_clause_paid(&mut self) {
        if let Some(c) = &mut self.cast_costs {
            for row in &mut c.clauses {
                if row.clause == CastCostClause::Mana {
                    row.paid = true;
                }
            }
            c.active = None;
        }
    }
    pub(in crate::engine) fn project_convoker(
        &mut self,
        object: ObjectRef,
        contribution: ConvokeContribution,
    ) -> bool {
        let Some(index) = self
            .resources
            .iter()
            .position(|r| r.id == object.card && r.object.generation == object.generation)
        else {
            return false;
        };
        let r = &self.resources[index];
        if r.object.zone != Zone::Battlefield
            || r.object.controller != self.actor
            || r.object.tapped
            || CharacteristicWorld::kinds(self, object.card) & 1 == 0
        {
            return false;
        }
        let colors = CharacteristicWorld::colors(self, object.card);
        let Some(c) = &mut self.cast_costs else {
            return false;
        };
        let Some(cost) = &mut c.mana else {
            return false;
        };
        if !contribution.apply(colors, cost) {
            return false;
        }
        self.resources[index].object.tapped = true;
        true
    }
    pub(in crate::engine) fn project_improviser(&mut self, object: ObjectRef) -> bool {
        if !self.improvise_available() || !improvise::eligible(self, self.actor, object) {
            return false;
        }
        let Some(index) = self
            .resources
            .iter()
            .position(|r| r.id == object.card && r.object.generation == object.generation)
        else {
            return false;
        };
        let Some(cost) = self.cast_costs.as_mut().and_then(|c| c.mana.as_mut()) else {
            return false;
        };
        let Some(generic) = cost.generic.checked_sub(1) else {
            return false;
        };
        cost.generic = generic;
        self.resources[index].object.tapped = true;
        true
    }
    fn complete_additional(
        &self,
        c: &CostContext,
        later: &CostContext,
        remaining: &[usize],
    ) -> bool {
        let Some(mut spec) = c.additional else {
            return true;
        };
        if let EffectChoiceKind::DiscardOrLife(life) = spec.kind {
            if !c.exact && self.life >= i128::from(life) {
                let mut next = self.clone();
                next.life -= i128::from(life);
                next.life_lost[next.actor.index()] |= life > 0;
                if next.complete_cost_rows(later, remaining) {
                    return true;
                }
            }
            if c.additional_life {
                return false;
            }
            spec.kind = EffectChoiceKind::Discard;
        }
        let mut amount = 1usize;
        let mut destination = Zone::Graveyard;
        let mut evidence_min = None;
        let mut counters = None;
        let mut any_graveyard = false;
        match spec.kind {
            EffectChoiceKind::Forage | EffectChoiceKind::ForageOrBlack => match c.forage {
                Some(ForageRoute::Black) => return self.complete_cost_rows(later, remaining),
                Some(ForageRoute::Food) => spec = forage::food_cost(),
                Some(ForageRoute::ExileThree) => {
                    amount = 3;
                    destination = Zone::Exile;
                    spec.kind = EffectChoiceKind::ExileGraveyard;
                    any_graveyard = true;
                }
                None => return false,
            },
            EffectChoiceKind::CollectEvidence(n) => {
                evidence_min = Some((u128::from(n), None));
                destination = Zone::Exile;
                spec.kind = EffectChoiceKind::ExileGraveyard;
                any_graveyard = true;
            }
            EffectChoiceKind::ExileGraveyardSymbols { color, minimum } => {
                evidence_min = Some((u128::from(minimum), Some(color)));
                destination = Zone::Exile;
            }
            EffectChoiceKind::RemoveCreatureCounters(n) => counters = Some(n),
            EffectChoiceKind::TapPermanents(n) | EffectChoiceKind::SacrificeDifferentNames(n) => {
                amount = usize::from(n)
            }
            EffectChoiceKind::ExileGraveyard => destination = Zone::Exile,
            EffectChoiceKind::ExileGraveyardCards(n) => {
                destination = Zone::Exile;
                amount = usize::from(n);
                spec.kind = EffectChoiceKind::ExileGraveyard;
            }
            EffectChoiceKind::ReturnUnblockedAttacker | EffectChoiceKind::ReturnToHand => {
                destination = Zone::Hand
            }
            EffectChoiceKind::Behold
            | EffectChoiceKind::Sacrifice
            | EffectChoiceKind::SacrificeAttachedEquipment
            | EffectChoiceKind::SacrificeOrDiscard { .. }
            | EffectChoiceKind::Discard
            | EffectChoiceKind::DiscardNamed(_)
            | EffectChoiceKind::DiscardMatchingLegendary
            | EffectChoiceKind::DiscardLegendary => {}
            _ => return false,
        }
        if let Some(n) = counters {
            return self.complete_counter_cost(c, later, remaining, n);
        }
        if any_graveyard {
            spec.filter = SacrificeFilter {
                keywords: Keywords::NONE,
                source_only: false,
                kinds: 0,
                nonland: false,
                other: false,
                tokens: None,
                artifact_enchantment_or_token: false,
                subtype: None,
            };
        }
        let candidates = self
            .resources
            .iter()
            .filter_map(|v| {
                let r = self.reference(v.id);
                self.cost_object(spec, r, c.source).then_some(r)
            })
            .collect::<Vec<_>>();
        if evidence_min.is_some_and(|(n, color)| {
            candidates
                .iter()
                .map(|r| cost_weight(self.object(r.card).card, color))
                .sum::<u128>()
                < n
        }) {
            return false;
        }
        if !c.selected.iter().all(|r| candidates.contains(r)) {
            return false;
        }
        let mut selected = c.selected.clone();
        self.complete_object_choices(
            c,
            later,
            remaining,
            spec,
            destination,
            amount,
            evidence_min,
            &candidates,
            0,
            &mut selected,
        )
    }
    #[allow(clippy::too_many_arguments)]
    fn complete_object_choices(
        &self,
        c: &CostContext,
        later: &CostContext,
        remaining: &[usize],
        spec: AdditionalCost,
        destination: Zone,
        amount: usize,
        evidence_min: Option<(u128, Option<crate::cards::ChoiceColor>)>,
        candidates: &[ObjectRef],
        at: usize,
        selected: &mut Vec<ObjectRef>,
    ) -> bool {
        let enough = evidence_min.map_or(selected.len() == amount, |(minimum, color)| {
            selected
                .iter()
                .map(|r| cost_weight(self.object(r.card).card, color))
                .sum::<u128>()
                >= minimum
        });
        if enough {
            if remaining.is_empty() {
                return true;
            }
            let success = if spec.kind == EffectChoiceKind::Behold {
                self.complete_cost_rows(later, remaining)
            } else if matches!(spec.kind, EffectChoiceKind::TapPermanents(_)) {
                let mut next = self.clone();
                for r in selected.iter() {
                    let i = next.resources.iter().position(|v| v.id == r.card).unwrap();
                    next.resources[i].object.tapped = true;
                }
                next.complete_cost_rows(later, remaining)
            } else {
                self.cost_departures(selected, destination, &mut |w| {
                    w.complete_cost_rows(later, remaining)
                })
            };
            if success {
                return true;
            }
        }
        if c.exact || evidence_min.is_none() && selected.len() >= amount {
            return false;
        }
        for i in at..candidates.len() {
            let r = candidates[i];
            if selected.contains(&r) {
                continue;
            }
            if matches!(spec.kind, EffectChoiceKind::SacrificeDifferentNames(_))
                && selected
                    .iter()
                    .any(|s| crate::card_names::same_name(self.name(s.card), self.name(r.card)))
            {
                continue;
            }
            selected.push(r);
            if self.complete_object_choices(
                c,
                later,
                remaining,
                spec,
                destination,
                amount,
                evidence_min,
                candidates,
                i + 1,
                selected,
            ) {
                return true;
            }
            selected.pop();
        }
        false
    }
    fn complete_counter_cost(
        &self,
        c: &CostContext,
        later: &CostContext,
        remaining: &[usize],
        amount: u16,
    ) -> bool {
        let mut next = self.clone();
        for &(r, kind) in &c.counters {
            if !next.remove_creature_counter(r, kind) {
                return false;
            }
        }
        if c.counters.len() > usize::from(amount)
            || (!c.source_counter_reservation.is_empty()
                && !CharacteristicWorld::counter_amounts(&next, c.source.card)
                    .contains(&c.source_counter_reservation))
        {
            return false;
        }
        fn fill(
            w: &FundingState<'_>,
            left: usize,
            later: &CostContext,
            remaining: &[usize],
            first: usize,
            source: ObjectRef,
            reservation: &Counters,
        ) -> bool {
            if left == 0 {
                return w.complete_cost_rows(later, remaining);
            }
            for (index, resource) in w.resources.iter().enumerate().skip(first) {
                if resource.object.zone != Zone::Battlefield
                    || resource.object.controller != w.actor
                    || CharacteristicWorld::kinds(w, resource.id) & 1 == 0
                {
                    continue;
                }
                for kind in CounterKind::ALL {
                    let mut next = w.clone();
                    if !next.remove_creature_counter(w.reference(resource.id), kind) {
                        continue;
                    }
                    if !reservation.is_empty()
                        && !CharacteristicWorld::counter_amounts(&next, source.card)
                            .contains(reservation)
                    {
                        continue;
                    }
                    if fill(
                        &next,
                        left - 1,
                        later,
                        remaining,
                        index,
                        source,
                        reservation,
                    ) {
                        return true;
                    }
                }
            }
            false
        }
        fill(
            &next,
            usize::from(amount) - c.counters.len(),
            later,
            remaining,
            0,
            c.source,
            &c.source_counter_reservation,
        )
    }
}

impl Game {
    pub(in crate::engine) fn teamwork_proposal_fundable(&self, a: &Announcement) -> bool {
        a.cost_options().any(|cost| {
            let mut choice = a.clone();
            choice.cost = Some(cost);
            let goal = self.funding_goal(&choice);
            let mut world = FundingState::from_game_with_convoke(
                self,
                &goal,
                choice.item.source.expect("cost announcement has object source").card.has_convoke(),
            );
            world.set_purpose(self.purpose_for_item(&choice.item));
            world.set_cost_context(Some(&choice));
            world.reachable(&goal, &mut HashSet::new())
        })
    }
    pub(in crate::engine) fn teamwork_prefix_fundable(
        &self,
        selected: &[ObjectRef],
        exact: bool,
    ) -> bool {
        let a = self.state.announcing.as_ref().unwrap();
        let mut world = self.cast_cost_world(a);
        world.set_teamwork_prefix(selected, exact);
        world.cost_completion()
    }
    pub(in crate::engine) fn blight_subject_fundable(&self, object: ObjectRef) -> bool {
        let a = self.state.announcing.as_ref().unwrap();
        let mut world = self.cast_cost_world(a);
        let costs = world.cast_costs.as_mut().unwrap();
        let active = costs.active.unwrap();
        let row = &mut costs.clauses[usize::from(active)];
        let CastCostClause::Blight { amount, .. } = row.clause else {
            return false;
        };
        row.clause = CastCostClause::Blight {
            amount,
            selected: Some(object),
        };
        world.cost_completion()
    }
    pub(in crate::engine) fn cast_cost_world(&self, a: &Announcement) -> FundingState<'_> {
        let goal = FundingGoal {
            actor: a.item.controller,
            required: [0; 6],
            generic: 0,
            reserve: None,
            source: None,
        };
        let mut world =
            FundingState::from_game_with_convoke(self, &goal, a.item.source.expect("cost announcement has object source").card.has_convoke());
        world.set_purpose(self.purpose_for_item(&a.item));
        world.set_cost_context(Some(a));
        world
    }
    pub(in crate::engine) fn visit_cast_cost_clauses(
        &self,
        emit: &mut impl FnMut(Action) -> bool,
    ) -> bool {
        let a = self.state.announcing.as_ref().unwrap();
        let world = self.cast_cost_world(a);
        for (index, row) in a.cast_costs.as_ref().unwrap().clauses.iter().enumerate() {
            if row.paid {
                continue;
            }
            let mut choice = world.clone();
            choice.cast_costs.as_mut().unwrap().active = Some(index as u16);
            if choice.cost_completion()
                && emit(Action::ChooseCastCost {
                    clause: index as u16,
                })
            {
                return true;
            }
        }
        false
    }
    pub(in crate::engine) fn visit_casualty_payments(
        &self,
        emit: &mut impl FnMut(Action) -> bool,
    ) -> bool {
        let instance = match self.state.pending {
            Pending::CasualtyPayment { instance, .. } => instance,
            _ => return false,
        };
        let a = self.state.announcing.as_ref().unwrap();
        let world = self.cast_cost_world(a);
        for id in 0..self.state.objects.len() {
            let object = self.object_ref(id as CardId);
            let mut choice = world.clone();
            choice.cast_costs.as_mut().unwrap().casualty[usize::from(instance)].sacrifice =
                Some(object);
            if choice.cost_completion() && emit(Action::PayCasualty { object }) {
                return true;
            }
        }
        false
    }
    pub(in crate::engine) fn cast_cost_object_fundable(&self, object: ObjectRef) -> bool {
        let Some(a) = self
            .state
            .announcing
            .as_ref()
            .filter(|a| a.cast_costs.as_ref().is_some_and(|c| c.mana_closed))
        else {
            return true;
        };
        let mut world = self.cast_cost_world(a);
        let c = world.cast_costs.as_mut().unwrap();
        c.selected = vec![object];
        c.exact = true;
        world.cost_completion()
    }
    pub(in crate::engine) fn cast_cost_group_fundable(
        &self,
        selected: &[ObjectRef],
        exact: bool,
    ) -> bool {
        let a = self.state.announcing.as_ref().unwrap();
        let mut world = self.cast_cost_world(a);
        let c = world.cast_costs.as_mut().unwrap();
        c.selected = selected.to_vec();
        c.exact = exact;
        world.cost_completion()
    }
    pub(in crate::engine) fn cast_cost_counters_fundable(
        &self,
        selected: &[(ObjectRef, CounterKind)],
    ) -> bool {
        let a = self.state.announcing.as_ref().unwrap();
        let mut world = self.cast_cost_world(a);
        world.cast_costs.as_mut().unwrap().counters = selected.to_vec();
        world.cost_completion()
    }
    pub(in crate::engine) fn casualty_intent_fundable(&self, pay: bool) -> bool {
        let mut a = self.state.announcing.as_ref().unwrap().clone();
        let row = a
            .casualty
            .as_mut()
            .unwrap()
            .iter_mut()
            .find(|r| !r.declared)
            .unwrap();
        row.pay = pay;
        row.declared = true;
        a.initialize_cast_costs(improvise::potential(self, a.item.source.expect("cost announcement has object source").card));
        if a.cast_costs.is_none() {
            a.cast_costs = Some(Box::default());
            a.initialize_cast_costs(improvise::potential(self, a.item.source.expect("cost announcement has object source").card));
        }
        if a.item
            .cast
            .is_some_and(|c| c.method == CastMethod::Harmonize)
        {
            return self.harmonize_choice_fundable(&a, None)
                || (0..self.state.objects.len()).any(|id| {
                    self.harmonize_choice_fundable(&a, Some(self.object_ref(id as CardId)))
                });
        }
        a.cost_options().any(|cost| {
            let mut choice = a.clone();
            choice.cost = Some(cost);
            let goal = self.funding_goal(&choice);
            let mut world = FundingState::from_game_with_convoke(
                self,
                &goal,
                choice.item.source.expect("cost announcement has object source").card.has_convoke(),
            );
            world.set_purpose(self.purpose_for_item(&choice.item));
            world.set_cost_context(Some(&choice));
            world.reachable(&goal, &mut HashSet::new())
        })
    }
    /// One resource solver handles mana and ordered additional life costs, including previews before announcement.
    pub(in crate::engine) fn can_fund_life_cost(
        &self,
        goal: FundingGoal,
        purpose: mana_lots::Purpose,
        life: u32,
    ) -> bool {
        let mut world = FundingState::from_game(self, &goal);
        world.set_purpose(purpose);
        if world.cast_costs.is_none() {
            world.install_cost_context(Some(Box::new(CostContext {
                teammates: Vec::new(),
                teamwork_exact: false,
                actor: goal.actor,
                mana: None,
                source_counter_reservation: Counters::default(),
                source: ObjectRef {
                    card: 0,
                    generation: 0,
                },
                clauses: Vec::new(),
                active: None,
                casualty: Vec::new(),
                additional: None,
                additional_life: false,
                forage: None,
                selected: Vec::new(),
                exact: false,
                counters: Vec::new(),
            })));
        }
        let c = world.cast_costs.as_mut().unwrap();
        if !c
            .clauses
            .iter()
            .any(|r| matches!(r.clause, CastCostClause::Life(_)))
        {
            c.clauses.push(CastCostRow {
                clause: CastCostClause::Life(life),
                paid: false,
            });
        }
        world.reachable(&goal, &mut HashSet::new())
    }
}

impl Game {
    /// Preview declarations that are still optional; fixed declarations stay fixed.
    #[allow(clippy::too_many_arguments)]
    pub(in crate::engine) fn preview_optional_cast_costs(
        &self,
        goal: FundingGoal,
        purpose: mana_lots::Purpose,
        source: ObjectRef,
        additional: Option<AdditionalCost>,
        route: Option<ForageRoute>,
        selected: &[ObjectRef],
        exact: bool,
        counters: &[(ObjectRef, CounterKind)],
        extra: &[CastCostClause],
    ) -> Option<bool> {
        let mana_lots::Purpose::Spell(spell) = purpose else {
            return None;
        };
        let current = self.state.announcing.as_ref().filter(|a| {
            a.item.kind == StackKind::Spell
                && a.item.controller == goal.actor
                && a.item.source.expect("cost announcement has object source").card == spell.card
        });
        let ordered_additional = additional
            .is_some_and(|a| matches!(a.kind, EffectChoiceKind::DiscardOrLife(_)))
            || additional.is_some()
                && (spell.card.has_convoke() || improvise::potential(self, spell.card));
        if !ordered_additional
            && extra.is_empty()
            && current.is_none_or(|a| a.casualty.is_none() && a.cast_costs.is_none())
            && !self.has_casualty(goal.actor, spell.card)
        {
            return None;
        }
        let grants = current
            .and_then(|a| a.casualty.as_deref())
            .cloned()
            .unwrap_or_else(|| self.casualty_grants(goal.actor, spell.card));
        if !ordered_additional
            && grants.is_empty()
            && extra.is_empty()
            && current.is_none_or(|a| a.cast_costs.is_none())
        {
            return None;
        }
        fn declarations(
            rows: &mut [CasualtyDeclaration],
            index: usize,
            visit: &mut impl FnMut(&[CasualtyDeclaration]) -> bool,
        ) -> bool {
            if index == rows.len() {
                return visit(rows);
            }
            if rows[index].declared {
                return declarations(rows, index + 1, visit);
            }
            rows[index].declared = true;
            for pay in [false, true] {
                rows[index].pay = pay;
                if declarations(rows, index + 1, visit) {
                    return true;
                }
            }
            rows[index].declared = false;
            false
        }
        let mut grants = grants;
        let routes = if additional.is_some_and(forage::is_forage_cost) && route.is_none() {
            let mut v = vec![Some(ForageRoute::ExileThree), Some(ForageRoute::Food)];
            if additional.is_some_and(|s| s.kind == EffectChoiceKind::ForageOrBlack) {
                v.push(Some(ForageRoute::Black));
            }
            v
        } else {
            vec![route]
        };
        Some(declarations(&mut grants, 0, &mut |casualty| {
            routes.iter().any(|&chosen_route| {
                let mut attempt = goal.clone();
                if chosen_route == Some(ForageRoute::Black) && route.is_none() {
                    attempt.required[2] = attempt.required[2]
                        .checked_add(1)
                        .capacity("forage mana exhausted");
                }
                let mut clauses = casualty
                    .iter()
                    .enumerate()
                    .filter(|(_, r)| r.pay && !r.paid)
                    .map(|(i, _)| CastCostRow {
                        clause: CastCostClause::Casualty(i as u16),
                        paid: false,
                    })
                    .collect::<Vec<_>>();
                if let Some(a) = current
                    && a.item.cast.is_some_and(|c| c.teamwork == Some(true))
                {
                    clauses.push(CastCostRow {
                        clause: CastCostClause::Teamwork(spell.card.teamwork().unwrap().minimum),
                        paid: false,
                    });
                }
                if additional.is_some() {
                    clauses.push(CastCostRow {
                        clause: CastCostClause::Additional,
                        paid: false,
                    });
                }
                if spell.card.has_convoke() || improvise::potential(self, spell.card) {
                    clauses.push(CastCostRow {
                        clause: CastCostClause::Mana,
                        paid: false,
                    });
                }
                for &clause in extra {
                    if !clauses.iter().any(|r| r.clause == clause) {
                        clauses.push(CastCostRow {
                            clause,
                            paid: false,
                        });
                    }
                }
                let context = CostContext {
                    teammates: current
                        .and_then(|a| a.cast_costs.as_ref())
                        .map_or_else(Vec::new, |c| c.teammates.clone()),
                    teamwork_exact: false,
                    actor: goal.actor,
                    mana: (spell.card.has_convoke() || improvise::potential(self, spell.card))
                        .then_some(ManaCost {
                            colored: attempt.required,
                            generic: attempt.generic,
                        }),
                    source,
                    source_counter_reservation: Counters::default(),
                    clauses,
                    active: None,
                    casualty: casualty.to_vec(),
                    additional,
                    additional_life: current.is_some_and(|a| a.additional_life),
                    forage: chosen_route,
                    selected: selected.to_vec(),
                    exact,
                    counters: counters.to_vec(),
                };
                let mut world =
                    FundingState::from_game_with_convoke(self, &attempt, spell.card.has_convoke());
                world.set_purpose(purpose);
                world.install_cost_context(Some(Box::new(context)));
                world.reachable(&attempt, &mut HashSet::new())
            })
        }))
    }
}

fn cost_weight(card: Card, color: Option<crate::cards::ChoiceColor>) -> u128 {
    u128::from(color.map_or_else(
        || card.fixed_mana_value(),
        |color| card.mana_symbols_of_color(color),
    ))
}

impl Game {
    pub(in crate::engine) fn can_fund_symbol_cost_group(
        &self,
        goal: FundingGoal,
        purpose: mana_lots::Purpose,
        spec: AdditionalCost,
        source: ObjectRef,
        selected: &[ObjectRef],
    ) -> bool {
        let mut world = FundingState::from_game(self, &goal);
        world.set_purpose(purpose);
        world.install_cost_context(Some(Box::new(CostContext {
            actor: goal.actor,
            source,
            source_counter_reservation: Counters::default(),
            clauses: vec![
                CastCostRow {
                    clause: CastCostClause::Additional,
                    paid: false,
                },
                CastCostRow {
                    clause: CastCostClause::Mana,
                    paid: false,
                },
            ],
            active: None,
            casualty: Vec::new(),
            additional: Some(spec),
            additional_life: false,
            forage: None,
            selected: selected.to_vec(),
            exact: false,
            counters: Vec::new(),
            mana: Some(ManaCost {
                colored: goal.required,
                generic: goal.generic,
            }),
            teammates: Vec::new(),
            teamwork_exact: false,
        })));
        world.reachable(&goal, &mut HashSet::new())
    }
}

impl FundingState<'_> {
    /// Enumerate the exact pre-SBA counter worlds produced by paying a blight cost.
    /// This remains unconnected to casting until blight intent and prohibitions exist.
    fn visit_blight_cost_projection(
        &self,
        subject: ObjectRef,
        amount: u16,
        emit: &mut dyn FnMut(&Self) -> bool,
    ) -> bool {
        let Some(row) = self.resources.iter().find(|r| r.id == subject.card) else {
            return false;
        };
        if amount == 0
            || row.object.zone != Zone::Battlefield
            || row.object.generation != subject.generation
            || row.object.controller != self.actor
            || CharacteristicWorld::kinds(self, subject.card) & 1 == 0
        {
            return false;
        }
        if replacements::counters_prohibited(self, subject) {
            return false;
        }
        let placement = replacements::CounterPlacement {
            placed_by: self.actor,
            object: subject,
            card: row.object.card,
            owner: row.object.owner,
            controller: row.object.controller,
            protector: CharacteristicWorld::battle_protector(self, subject),
            subtypes: self.subtypes(subject.card),
            kind: CharacteristicWorld::kinds(self, subject.card),
            power: characteristic_query::CharacteristicQuery::in_world(self)
                .stats(subject.card)
                .power,
            amount: Counters::of(CounterKind::MinusOne, u64::from(amount)),
            effect: false,
            entry_index: None,
            private: false,
        };
        replacements::counter_outcomes(
            self,
            &placement,
            &mut Vec::new(),
            &|placement| placement.clone(),
            &mut |placed| {
                let mut next = self.clone();
                let mut counters =
                    CharacteristicWorld::counter_amounts(&next, subject.card).to_owned();
                for kind in CounterKind::ALL {
                    counters.add(kind, placed.get(kind));
                }
                let resource = next
                    .resources
                    .iter_mut()
                    .find(|r| r.id == subject.card)
                    .unwrap();
                resource.object.counters = counters.hot();
                live_counters::set(&mut next.named_counters, subject, counters.named);
                emit(&next)
            },
        )
    }
}

#[cfg(test)]
#[path = "engine_additional_bound_tests.rs"]
mod additional_bound_tests;
