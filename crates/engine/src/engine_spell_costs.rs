//! Generic reductions and the total cost locked before mana abilities/payment.
use super::*;
use crate::cards::CastFilter;
use ability_world::CharacteristicWorld;
#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
impl Game {
    /// Compute the wide pre-reduction generic increase from active sources.
    pub(super) fn spell_generic_tax(&self, actor: Player, definition: Card) -> u64 {
        use crate::cards::StaticAbilityFragment::{
            ChosenCardTypeSpellTax, OpponentNoncreatureSpellTax,
        };

        let kinds = definition.kind_mask();
        // A creature spell can only be taxed here if another one of its
        // types was chosen. Preserve the common no-tax path for creatures.
        if kinds & 1 != 0
            && !self.state.entry_values.as_ref().is_some_and(|rows| {
                rows.iter().any(|row| {
                    matches!(row.value, EntryChoiceValue::CardType(kind) if kind.spell_mask() & kinds != 0)
                })
            })
        {
            return 0;
        }
        let mut total = 0u64;
        let mut query = None;
        for &id in &self.state.static_sources {
            if !self.on_battlefield(id) {
                continue;
            }
            let object = &self.state.objects[id as usize];
            let source = self.object_ref(id);
            for (index, group) in object.card.static_ability_groups().iter().enumerate() {
                let amount = group
                    .fragments
                    .iter()
                    .filter_map(|fragment| match *fragment {
                        OpponentNoncreatureSpellTax(amount)
                            if kinds & 1 == 0 && object.controller != actor =>
                        {
                            Some(amount)
                        }
                        ChosenCardTypeSpellTax(amount)
                            if matches!(
                                CharacteristicWorld::entry_value(self, source),
                                Some(EntryChoiceValue::CardType(kind)) if kind.spell_mask() & kinds != 0
                            ) =>
                        {
                            Some(amount)
                        }
                        _ => None,
                    })
                    .map(u64::from)
                    .fold(0, u64::saturating_add);
                if amount != 0
                    && query
                        .get_or_insert_with(|| self.characteristic_query())
                        .continuous_group_available(source, index, group)
                {
                    total = total.saturating_add(amount);
                }
            }
        }
        total
    }
    /// Keep increases and all reductions wide until the final payable total.
    #[allow(clippy::too_many_arguments)] // Keep the distinct cost components explicit.
    pub(super) fn spell_total_with_reductions(
        &self,
        actor: Player,
        id: CardId,
        base: ManaCost,
        definition: Card,
        mana_value: u64,
        extra_generic: u64,
        discount: ManaCost,
        extra_reduction: u64,
    ) -> Option<ManaCost> {
        let raw = u64::from(base.generic)
            .checked_add(extra_generic)?
            .checked_add(self.spell_generic_tax(actor, definition))?;
        let colored = std::array::from_fn(|i| base.colored[i].saturating_sub(discount.colored[i]));
        let excess = (0..6)
            .map(|i| u64::from(discount.colored[i].saturating_sub(base.colored[i])))
            .sum::<u64>();
        let reductions = self
            .spell_reduction_for_definition(actor, id, mana_value, definition)
            .saturating_add(u64::from(discount.generic))
            .saturating_add(extra_reduction)
            .saturating_add(excess);
        Some(ManaCost {
            colored,
            generic: u32::try_from(raw.saturating_sub(reductions)).ok()?,
        })
    }
    pub(super) fn spell_total_for_definition(
        &self,
        actor: Player,
        id: CardId,
        base: ManaCost,
        definition: Card,
    ) -> Option<ManaCost> {
        self.spell_total_with_reductions(
            actor,
            id,
            base,
            definition,
            definition.fixed_mana_value(),
            0,
            ManaCost {
                colored: [0; 6],
                generic: 0,
            },
            0,
        )
    }
    pub(super) fn spell_cost_option_available(
        &self,
        actor: Player,
        card: CardId,
        cost: ManaCost,
    ) -> bool {
        let definition = self.effective_spell_card(card);
        let (colored, generic) = definition.cost();
        cost == ManaCost { colored, generic }
            || definition.cost_alternative_guard().is_none_or(|guard| {
                self.step_guard(
                    guard,
                    actor,
                    self.object_ref(card),
                    self.state.objects[card as usize].tapped,
                )
            })
    }
    pub(super) fn announcement_optional_reduction(a: &Announcement) -> u64 {
        let evidence = a.item.cast.is_some_and(|c| c.evidence == Some(true));
        let bargain = a.item.cast.is_some_and(|c| c.bargain == Some(true));
        u64::from(if evidence {
            a.item
                .source
                .expect("cost announcement has object source")
                .card
                .evidence_reduction()
        } else {
            0
        }) + u64::from(if bargain {
            a.item
                .source
                .expect("cost announcement has object source")
                .card
                .bargain_reduction()
        } else {
            0
        })
    }
    pub(super) fn announcement_base_cost(a: &Announcement) -> ManaCost {
        let mut base = a.cost.unwrap_or_else(|| {
            if a.item
                .cast
                .is_some_and(|cast| cast.method == CastMethod::GrantedHarmonize)
            {
                return a
                    .fixed_alternative_cost()
                    .expect("printed harmonize mana cost");
            }
            if a.item.cast.is_some_and(|cast| cast.method.is_mayhem()) {
                return a.mayhem_cost_options().next().expect("Mayhem cost");
            }
            if a.item
                .cast
                .is_some_and(|c| matches!(c.method, CastMethod::Miracle | CastMethod::WebSlinging))
            {
                return a
                    .origin_alternative_cost
                    .expect("selected alternative cost");
            }
            if a.item.cast.is_some_and(|c| c.method == CastMethod::Sneak) {
                return a.sneak_cost_options().next().expect("Sneak cost");
            }
            if a.item
                .cast
                .is_some_and(|c| c.method == CastMethod::Harmonize)
            {
                return a
                    .item
                    .source
                    .expect("cost announcement has object source")
                    .card
                    .harmonize_cost()
                    .expect("Harmonize cost");
            }
            if a.item
                .cast
                .is_some_and(|c| c.method == CastMethod::Disguise)
            {
                return ManaCost {
                    colored: [0; 6],
                    generic: 3,
                };
            }
            if a.item.cast.is_some_and(|c| c.method == CastMethod::Warp) {
                return a.warp_spec().mana;
            }
            if a.item.cast.is_some_and(|c| c.method == CastMethod::Airbend) {
                return ManaCost {
                    colored: [0; 6],
                    generic: 2,
                };
            }
            if a.item.cast.is_some_and(|c| {
                matches!(
                    c.method,
                    CastMethod::ExileLife
                        | CastMethod::LibraryLife
                        | CastMethod::Plot
                        | CastMethod::Free
                )
            }) {
                return ManaCost {
                    colored: [0; 6],
                    generic: 0,
                };
            }
            if a.item
                .cast
                .is_some_and(|cast| cast.method == CastMethod::Flashback)
            {
                return a
                    .item
                    .source
                    .expect("cost announcement has object source")
                    .card
                    .flashback_cost()
                    .unwrap();
            }
            assert!(
                !a.item
                    .source
                    .expect("cost announcement has object source")
                    .card
                    .has_hybrid_cost(),
                "hybrid cost not chosen"
            );
            let (colored, generic) = a
                .item
                .source
                .expect("cost announcement has object source")
                .card
                .cost();
            ManaCost { colored, generic }
        });
        // A selected hybrid equivalent already includes the declared surcharge.
        if a.cost.is_none() {
            base = Self::with_flash_surcharge(
                base,
                a.item
                    .source
                    .expect("cost announcement has object source")
                    .card,
                a.item.cast.is_some_and(|c| c.flash_surcharge),
            );
        }
        if a.cost.is_none() {
            base = Self::with_tiered_mana(base, &a.item).expect("validated selected Tiered cost");
            base.generic = base
                .generic
                .checked_add(a.behold_decline_generic())
                .and_then(|generic| generic.checked_add(a.exile_play_surcharge()))
                .capacity("spell generic cost exhausted");
        }
        if a.forage
            .as_ref()
            .is_some_and(|f| f.route == Some(ForageRoute::Black))
        {
            base.colored[2] = base.colored[2]
                .checked_add(1)
                .capacity("forage alternative mana cost exhausted");
        }
        Self::with_kicker_cost(
            Self::with_offspring_cost(
                base,
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
        )
    }
    pub(super) fn with_flash_surcharge(mut base: ManaCost, card: Card, pay: bool) -> ManaCost {
        if pay {
            let extra = card.flash_surcharge().expect("flash surcharge recipe");
            base.generic = base
                .generic
                .checked_add(extra.generic)
                .capacity("flash surcharge mana cost exhausted");
            for (amount, extra) in base.colored.iter_mut().zip(extra.colored) {
                *amount = amount
                    .checked_add(extra)
                    .capacity("flash surcharge mana cost exhausted");
            }
        }
        base
    }
    pub(super) fn flash_surcharge_cast_fundable(
        &self,
        actor: Player,
        card: CardId,
        flashback: bool,
    ) -> bool {
        let definition = self.state.objects[card as usize].card;
        let available = match definition.effect() {
            Effect::Modal(modes) => modes.iter().any(|mode| {
                self.cast_targets_available(mode.targets, actor, self.object_ref(card))
            }),
            Effect::Spell(plan, _) | Effect::Aura(plan) => {
                self.cast_targets_available(plan, actor, self.object_ref(card))
            }
            _ => true,
        };
        if !available {
            return false;
        }
        let costs = if flashback {
            definition.flashback_cost_options()
        } else {
            definition.cost_options()
        };
        costs.into_iter().any(|cost| {
            let cost = Self::with_flash_surcharge(cost, definition, true);
            if definition.has_target_cost_modifier() {
                self.initial_target_cost_fundable(actor, card, cost)
            } else if definition.collect_evidence_cost().is_some() {
                self.evidence_cast_fundable(actor, card, cost, false)
                    || self.evidence_cast_fundable(actor, card, cost, true)
            } else if definition.has_bargain() {
                self.initial_bargain_fundable(actor, card, cost)
            } else {
                self.spell_can_fund(actor, card, cost)
            }
        })
    }
    pub(super) fn with_kicker_cost(mut base: ManaCost, card: Card, pay: bool) -> ManaCost {
        if pay {
            let extra = card.kicker_cost().expect("kicker recipe");
            base.generic = base
                .generic
                .checked_add(extra.generic)
                .capacity("kicker mana cost exhausted");
            for (amount, extra) in base.colored.iter_mut().zip(extra.colored) {
                *amount = amount
                    .checked_add(extra)
                    .capacity("kicker mana cost exhausted");
            }
        }
        base
    }
    pub(super) fn kicker_choice_fundable(&self, a: &Announcement, pay: bool) -> bool {
        let mut proposal = a.clone();
        proposal.item.cast.as_mut().unwrap().kicker = Some(pay);
        let source = proposal
            .item
            .source
            .expect("cost announcement has object source");
        if source.card.kicked_targets().is_some() {
            proposal.plan = self.target_spec(&proposal.item);
            if !self.cast_definition_targets_available(
                proposal.plan,
                proposal.item.controller,
                source.object,
                source.card,
            ) {
                return false;
            }
        }
        proposal
            .cost_options()
            .any(|cost| self.announcement_cost_fundable(&proposal, cost))
    }
    pub(super) fn with_offspring_cost(mut base: ManaCost, card: Card, pay: bool) -> ManaCost {
        if pay {
            let extra = card.offspring_cost().expect("offspring recipe");
            base.generic = base
                .generic
                .checked_add(extra.generic)
                .capacity("offspring mana cost exhausted");
            for (amount, extra) in base.colored.iter_mut().zip(extra.colored) {
                *amount = amount
                    .checked_add(extra)
                    .capacity("offspring mana cost exhausted");
            }
        }
        base
    }
    pub(super) fn offspring_choice_fundable(&self, a: &Announcement, pay: bool) -> bool {
        if a.item
            .source
            .expect("cost announcement has object source")
            .card
            .has_target_cost_modifier()
        {
            return self.announcement_target_fundable(a, None, None, Some(pay), None, false);
        }
        a.cost_options().any(|cost| {
            self.spell_can_fund(
                a.item.controller,
                a.item
                    .source
                    .expect("cost announcement has object source")
                    .id,
                Self::with_offspring_cost(
                    cost,
                    a.item
                        .source
                        .expect("cost announcement has object source")
                        .card,
                    pay,
                ),
            )
        })
    }
    /// Announced spells already occupy the stack; retain their actual cast origin.
    /// During nested mana abilities the enclosing announcement remains authoritative.
    fn spell_origin(&self, id: CardId) -> Zone {
        let packet = self
            .state
            .announcing
            .iter()
            .map(|a| &a.item)
            .chain(
                self.state
                    .mana_announcement_stack
                    .iter()
                    .flat_map(|frames| frames.iter().map(|f| &f.announcement.item)),
            )
            .chain(self.state.stack.iter())
            .find(|s| s.kind == StackKind::Spell && s.source.is_some_and(|source| source.id == id));
        packet
            .and_then(|s| s.cast.map(|c| c.origin))
            .unwrap_or(self.state.objects[id as usize].zone)
    }
    fn spell_origin_is_nonhand(&self, actor: Player, id: CardId) -> bool {
        self.spell_origin(id) != Zone::Hand || self.state.objects[id as usize].owner != actor
    }
    pub(super) fn spell_reduction(&self, actor: Player, id: CardId) -> u32 {
        u32::try_from(self.spell_reduction_at_mana_value(
            actor,
            id,
            self.effective_spell_card(id).fixed_mana_value(),
        ))
        .unwrap_or(u32::MAX)
    }
    pub(super) fn spell_reduction_at_mana_value(
        &self,
        actor: Player,
        id: CardId,
        mana_value: u64,
    ) -> u64 {
        self.spell_reduction_for_definition(actor, id, mana_value, self.effective_spell_card(id))
    }
    pub(super) fn spell_reduction_for_definition(
        &self,
        actor: Player,
        id: CardId,
        mana_value: u64,
        definition: Card,
    ) -> u64 {
        if definition.cost_reductions().is_empty()
            && self.state.static_sources.is_empty()
            && self.affinity_grants(actor, definition) == 0
        {
            return self.turn_spell_reduction(actor, definition);
        }
        if let Some(world) = self.cast_departure(self.object_ref(id), actor) {
            self.spell_reduction_in_world(&world, actor, id, mana_value, definition)
        } else {
            self.spell_reduction_in_world(self, actor, id, mana_value, definition)
        }
    }

    fn affinity_grants(&self, player: Player, definition: Card) -> u64 {
        if definition.is_creature() {
            return 0;
        }
        self.player_cast_grants_view(player.index())
            .map_or(0, |row| row.next_noncreature_artifact_affinity)
    }
    fn spell_reduction_in_world<W: CharacteristicWorld + ?Sized>(
        &self,
        world: &W,
        actor: Player,
        id: CardId,
        mana_value: u64,
        definition: Card,
    ) -> u64 {
        let recipient = world.object(id);
        let stats = |id| characteristic_query::CharacteristicQuery::in_world(world).stats(id);
        let applies = |rule: &crate::cards::SpellReduction, source: CardId| {
            let object = world.object(source);
            (match rule.filter {
                CastFilter::SourceChosenCreatureType => {
                    matches!(world.entry_value(world.reference(source)), Some(EntryChoiceValue::CreatureType(typ)) if definition.has_subtype(typ.name()))
                }
                CastFilter::FirstQualifiedCreature(qualifier) => {
                    qualifier.matches(definition)
                        && self.state.players[actor.index()]
                            .turn_facts
                            .qualified_creature_cast_count(qualifier)
                            == 0
                }
                CastFilter::CreatureToughnessGreaterThanPower => {
                    definition.is_creature() && {
                        let value = stats(id);
                        value.toughness > value.power
                    }
                }
                _ => rule.filter.matches_mana_value(definition, mana_value),
            }) && match rule.turn {
                CostTurn::Any => true,
                CostTurn::Graveyard => {
                    self.spell_origin(id) == Zone::Graveyard && recipient.owner == actor
                }
                CostTurn::GraveyardOrExile => match self.spell_origin(id) {
                    Zone::Graveyard => recipient.owner == actor,
                    Zone::Exile => true,
                    _ => false,
                },
                CostTurn::NonHand => self.spell_origin_is_nonhand(actor, id),
                CostTurn::Own => self.state.active == actor,
                CostTurn::Other => self.state.active != actor,
            } && self.step_guard_in_world(
                world,
                rule.guard,
                actor,
                self.object_ref(source),
                object.tapped,
            )
        };
        let amount = |rule: &crate::cards::SpellReduction, source: CardId| match rule.amount {
            crate::cards::SpellReductionAmount::GraveyardInstantSorceryAdventure => {
                adventure::graveyard_instant_sorcery_adventure_count(
                    actor,
                    (0..self.state.objects.len()).map(|id| world.object(id as CardId)),
                ) as u64
            }
            crate::cards::SpellReductionAmount::CreaturePowerCount { minimum } => {
                self.state
                    .objects
                    .iter()
                    .enumerate()
                    .filter(|(id, o)| {
                        o.zone == Zone::Battlefield
                            && o.controller == actor
                            && world.kinds(*id as CardId) & 1 != 0
                            && stats(*id as CardId).power >= i128::from(minimum)
                    })
                    .count() as u64
            }
            crate::cards::SpellReductionAmount::Fixed(n) => u64::from(n),
            crate::cards::SpellReductionAmount::Attackers { factor } => {
                u64::try_from(self.state.attacked_this_turn.len())
                    .unwrap_or(u64::MAX)
                    .saturating_mul(u64::from(factor))
            }
            crate::cards::SpellReductionAmount::EquippedPower => self.state.objects
                [source as usize]
                .attached_to
                .filter(|target| {
                    self.on_battlefield(target.card)
                        && self.object_ref(target.card) == *target
                        && world.kinds(target.card) & 1 != 0
                })
                .map_or(0, |target| {
                    u64::try_from(stats(target.card).power.max(0)).unwrap_or(u64::MAX)
                }),
            crate::cards::SpellReductionAmount::GreatestCreatureToughness => {
                let maximum = self
                    .state
                    .objects
                    .iter()
                    .enumerate()
                    .filter(|(id, object)| {
                        object.zone == Zone::Battlefield
                            && object.controller == actor
                            && world.kinds(*id as CardId) & 1 != 0
                    })
                    .map(|(id, _)| stats(id as CardId).toughness)
                    .max()
                    .unwrap_or(0)
                    .max(0);
                u64::try_from(maximum).unwrap_or(u64::MAX)
            }
            crate::cards::SpellReductionAmount::SourcePower => {
                u64::try_from(stats(source).power.max(0)).unwrap_or(u64::MAX)
            }
            crate::cards::SpellReductionAmount::CreaturePower { filter, greatest } => {
                let query = characteristic_query::CharacteristicQuery::in_world(world);
                let mut powers = self
                    .state
                    .objects
                    .iter()
                    .enumerate()
                    .filter(|&(id, _)| {
                        query.creature_matches(filter, actor, world.reference(source), id as CardId)
                    })
                    .map(|(id, _)| query.stats(id as CardId).power);
                // Sum signed values before clamping: negative-power creatures
                // reduce the total, but cannot create a negative discount.
                let value = if greatest {
                    powers.max().unwrap_or(0)
                } else {
                    powers
                        .try_fold(0i128, i128::checked_add)
                        .capacity("spell power aggregate exhausted")
                };
                u64::try_from(value.max(0)).unwrap_or(u64::MAX)
            }
            crate::cards::SpellReductionAmount::Stat { value, factor } => {
                // Widen before multiplying: raw X plus printed generic may exceed
                // u32 before a legal discount makes its payment representable.
                let n = self
                    .stat_value_in_world(world, value, world.reference(source), actor)
                    .max(0);
                u64::try_from(n)
                    .unwrap_or(u64::MAX)
                    .saturating_mul(u64::from(factor))
            }
        };
        // Raw scalar requirements fit u64; discounts above u64::MAX are
        // equivalent and impose no artificial reducer-count limit.
        let mut reduction = self.turn_spell_reduction(actor, definition);
        let affinity_grants = self.affinity_grants(actor, definition);
        if affinity_grants != 0 {
            let artifacts = (0..self.state.objects.len())
                .map(|id| id as CardId)
                .filter(|&id| {
                    let object = world.object(id);
                    object.zone == Zone::Battlefield
                        && object.controller == actor
                        && world.kinds(id) & 16 != 0
                })
                .count() as u64;
            reduction = reduction.saturating_add(affinity_grants.saturating_mul(artifacts));
        }
        for rule in definition.cost_reductions() {
            if rule.source_only && applies(rule, id) {
                reduction = reduction.saturating_add(amount(rule, id));
            }
        }
        for &source in &self.state.static_sources {
            let object = world.object(source);
            if object.controller != actor {
                continue;
            }
            for (index, rule) in object.card.cost_reductions().iter().enumerate() {
                if rule.source_only || !world.fragment_available(source, |fragment| matches!(fragment,
                    crate::cards::StaticAbilityFragment::CostReduction(slot, _) if usize::from(*slot) == index)) { continue; }
                if applies(rule, source) {
                    reduction = reduction.saturating_add(amount(rule, source));
                }
            }
        }
        reduction
    }
    pub(super) fn reduced_spell_cost(
        &self,
        actor: Player,
        id: CardId,
        base: ManaCost,
    ) -> Option<ManaCost> {
        self.spell_total_for_definition(actor, id, base, self.effective_spell_card(id))
    }
}
