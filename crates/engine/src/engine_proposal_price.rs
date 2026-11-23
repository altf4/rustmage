//! Optimistic mana prices for unopened priority proposals.
//!
//! Only current public resources and immutable rules are read. When a
//! discount cannot be bounded cheaply, credit its entire affected component.
//! Prices are necessary conditions, never permission to execute a proposal.
use super::*;
use crate::cards::{SpellReductionAmount, StaticAbilityFragment};

#[derive(Clone, Copy, Default)]
struct PriceReduction {
    generic: u64,
    colored: [u64; 6],
}

impl PriceReduction {
    fn add(&mut self, reduction: SpellReductionAmount, amount: u64) {
        let affected = match reduction {
            SpellReductionAmount::ColoredStat { color, .. } => &mut self.colored[color.index()],
            _ => &mut self.generic,
        };
        *affected = affected.saturating_add(amount);
    }

    fn apply(self, mut price: ManaCost) -> ManaCost {
        let mut generic = self.generic;
        for (required, discount) in price.colored.iter_mut().zip(self.colored) {
            let used = discount.min(u64::from(*required));
            *required -= used as u32;
            generic = generic.saturating_add(discount - used);
        }
        price.generic = price
            .generic
            .saturating_sub(u32::try_from(generic).unwrap_or(u32::MAX));
        price
    }
}

#[derive(Clone, Copy, Default)]
struct PriceCounts {
    total: u64,
    maximum_mana_value: u64,
    instant_or_sorcery: u64,
    instant_sorcery_or_adventure: u64,
}

fn count_zone(zone: CountZone) -> usize {
    match zone {
        CountZone::Battlefield => 0,
        CountZone::Graveyard => 1,
        CountZone::Exile => 2,
    }
}

pub(super) struct ProposalPriceFacts {
    actor: Player,
    counts: [[PriceCounts; 2]; 3],
    battlefield_untapped: u64,
    attackers: u64,
    copied_characteristics: bool,
    loose_board: bool,
    affinity_grants: u64,
    static_spell_discounts: PriceReduction,
    convoke_creatures: bool,
    improvise: bool,
    delve: bool,
    spell_any_mana: bool,
    spell_any_color: bool,
    activation_any_color: bool,
    equip_target_discount: u64,
    free_equip: bool,
    provisional_activation_reduction: bool,
}

impl ProposalPriceFacts {
    /// Aggregate once at priority, before testing individual actions. The rules
    /// attached to a definition are fixed-size for the current catalogue.
    pub(super) fn new(game: &Game, actor: Player) -> Self {
        let melded = game
            .state
            .roles
            .as_ref()
            .is_some_and(|roles| !roles.melded_permanents.is_empty());
        let mut facts = Self {
            actor,
            counts: [[PriceCounts::default(); 2]; 3],
            battlefield_untapped: 0,
            attackers: game.state.attacked_this_turn.len() as u64,
            copied_characteristics: game.state.copy_recipes.is_some()
                || game.state.temporary_copies.is_some(),
            loose_board: game.state.manifest.is_some() || !game.state.exile_links.is_empty(),
            affinity_grants: game
                .player_cast_grants_view(actor.index())
                .map_or(0, |grants| grants.next_noncreature_artifact_affinity),
            static_spell_discounts: PriceReduction::default(),
            convoke_creatures: false,
            improvise: false,
            delve: false,
            spell_any_mana: false,
            spell_any_color: false,
            activation_any_color: false,
            equip_target_discount: 0,
            free_equip: false,
            provisional_activation_reduction: false,
        };
        for object in &game.state.objects {
            let (zone, related) = match object.zone {
                Zone::Battlefield => (CountZone::Battlefield, object.controller),
                Zone::Graveyard => (CountZone::Graveyard, object.owner),
                Zone::Exile => (CountZone::Exile, object.owner),
                _ => continue,
            };
            let count = &mut facts.counts[count_zone(zone)][related.index()];
            count.total += 1;
            // Face-down exile contents never influence the bound. A concealed
            // permanent or linked return makes board-dependent pricing unknown.
            if object.zone == Zone::Exile {
                count.maximum_mana_value = u64::MAX;
                continue;
            }
            if facts.loose_board && object.zone == Zone::Battlefield {
                count.maximum_mana_value = u64::MAX;
                continue;
            }
            let card = object.card;
            let mana_value = if card.room().is_some() || melded && object.zone == Zone::Battlefield
            {
                u64::MAX
            } else {
                card.fixed_mana_value()
                    .max(card.face_parent().map_or(0, Card::fixed_mana_value))
            };
            count.maximum_mana_value = count.maximum_mana_value.max(mana_value);
            if object.zone == Zone::Graveyard && object.is_card() {
                let spell = card.kind_mask() & (4 | 8) != 0;
                count.instant_or_sorcery += u64::from(spell);
                count.instant_sorcery_or_adventure +=
                    u64::from(spell || card.adventure_spell().is_some());
            }
            if object.zone == Zone::Battlefield && related == actor && !object.tapped {
                facts.battlefield_untapped += 1;
            }
        }
        if facts.loose_board {
            return facts;
        }

        for object in game.state.objects.iter().filter(|object| {
            matches!(object.zone, Zone::Battlefield | Zone::Command) && object.controller == actor
        }) {
            let card = object.card;
            // Ignore all guards and layer removal. Over-crediting a printed
            // discount includes reducers a candidate may expose before lock.
            for reduction in card
                .cost_reductions()
                .iter()
                .filter(|rule| !rule.source_only)
            {
                let amount = facts.amount(reduction.amount);
                facts.static_spell_discounts.add(reduction.amount, amount);
            }
            let mut equip_discount: u64 = 0;
            for fragment in card
                .static_ability_groups()
                .iter()
                .flat_map(|group| group.fragments)
            {
                match *fragment {
                    StaticAbilityFragment::ConvokeCreatureSpells => facts.convoke_creatures = true,
                    StaticAbilityFragment::SpellImprovise(_) => facts.improvise = true,
                    StaticAbilityFragment::SpellDelve(_) => facts.delve = true,
                    StaticAbilityFragment::SpendAnyManaForCreatures => facts.spell_any_mana = true,
                    StaticAbilityFragment::SpendAnyColorForSpellSubtype(_) => {
                        facts.spell_any_color = true;
                    }
                    StaticAbilityFragment::SpendAnyColorForCreatureActivations => {
                        facts.activation_any_color = true;
                    }
                    StaticAbilityFragment::EquipTargetReduction(amount) => {
                        equip_discount = equip_discount.saturating_add(u64::from(amount));
                    }
                    StaticAbilityFragment::FirstEquipZeroCost { .. } => facts.free_equip = true,
                    StaticAbilityFragment::ArtifactTokenActivationReduction(_)
                    | StaticAbilityFragment::ControlledActivationReduction { .. }
                    | StaticAbilityFragment::OtherControlledExhaustGenericReduction(_) => {
                        facts.provisional_activation_reduction = true;
                    }
                    _ => {}
                }
            }
            facts.equip_target_discount = facts.equip_target_discount.max(equip_discount);
        }
        facts
    }

    fn aggregate(&self, filter: CountFilter, read: impl Fn(PriceCounts) -> u64, max: bool) -> u64 {
        let scopes = self.counts[count_zone(filter.zone)];
        let you = read(scopes[self.actor.index()]);
        let opponent = read(scopes[self.actor.other().index()]);
        match filter.relation {
            0 => you,
            1 => opponent,
            _ if max => you.max(opponent),
            _ => you.saturating_add(opponent),
        }
    }

    fn stat_upper(&self, value: StatValue) -> u64 {
        match value {
            StatValue::Fixed(n) => u64::from(n),
            StatValue::Count(filter)
            | StatValue::CountAllKinds(filter)
            | StatValue::TappedCount(filter)
            | StatValue::UntappedCount(filter) => {
                if filter.zone == CountZone::Graveyard
                    && filter.cards_only
                    && filter.kind != 0
                    && filter.kind & !(4 | 8) == 0
                    && !self.copied_characteristics
                {
                    return self.aggregate(filter, |scope| scope.instant_or_sorcery, false);
                }
                self.aggregate(filter, |scope| scope.total, false)
            }
            StatValue::GreatestManaValue(filter) => {
                self.aggregate(filter, |scope| scope.maximum_mana_value, true)
            }
            StatValue::GraveyardCards => self.counts[1][self.actor.index()].total,
            StatValue::GraveyardInstantSorceryAdventure => {
                self.counts[1][self.actor.index()].instant_sorcery_or_adventure
            }
            // Unknown current or declaration-derived stats get an unrestricted
            // discount of the affected kind. This can only keep more actions.
            _ => u64::MAX,
        }
    }

    fn amount(&self, amount: SpellReductionAmount) -> u64 {
        match amount {
            SpellReductionAmount::Fixed(n) => u64::from(n),
            SpellReductionAmount::Attackers { factor } => {
                self.attackers.saturating_mul(u64::from(factor))
            }
            SpellReductionAmount::GraveyardInstantSorceryAdventure => {
                self.counts[1][self.actor.index()].instant_sorcery_or_adventure
            }
            SpellReductionAmount::CreaturePowerCount { .. } => {
                self.counts[0][self.actor.index()].total
            }
            SpellReductionAmount::Stat { value, factor }
            | SpellReductionAmount::ColoredStat { value, factor, .. } => {
                self.stat_upper(value).saturating_mul(u64::from(factor))
            }
            _ => u64::MAX,
        }
    }
}

fn spend_as_generic(mut price: ManaCost, count: usize) -> ManaCost {
    for required in &mut price.colored[..count] {
        price.generic = price.generic.saturating_add(std::mem::take(required));
    }
    price
}

impl Game {
    /// `printed` is one candidate payment route, at the smallest permitted X,
    /// with any optional surcharges already chosen by the caller.
    pub(super) fn spell_mana_lower_bound(
        &self,
        definition: Card,
        _source: CardId,
        origin: Zone,
        method: CastMethod,
        printed: ManaCost,
        facts: &ProposalPriceFacts,
    ) -> Option<ManaCost> {
        if facts.loose_board {
            return None;
        }
        let mut discount = facts.static_spell_discounts;
        for rule in definition
            .cost_reductions()
            .iter()
            .filter(|rule| rule.source_only)
        {
            discount.add(rule.amount, facts.amount(rule.amount));
        }
        let battlefield = facts.counts[0][facts.actor.index()].total;
        discount.generic = discount
            .generic
            .saturating_add(self.turn_spell_reduction(facts.actor, definition))
            .saturating_add(facts.affinity_grants.saturating_mul(battlefield))
            .saturating_add(u64::from(definition.bargain_reduction()))
            .saturating_add(u64::from(definition.evidence_reduction()));
        if let Some(rule) = definition.target_cost_reduction() {
            for (total, amount) in discount.colored.iter_mut().zip(rule.discount.colored) {
                *total = total.saturating_add(u64::from(amount));
            }
            discount.generic = discount
                .generic
                .saturating_add(u64::from(rule.discount.generic));
        }
        if definition
            .additional_cost()
            .is_some_and(|cost| cost.kind == EffectChoiceKind::SacrificeForGenericReduction)
        {
            discount.generic = discount.generic.saturating_add(battlefield);
        }
        if method.is_harmonize() {
            discount.generic = u64::MAX;
        }
        // Pay every potential contribution under an intentionally generous
        // resource bound. Exact contribution and source conflicts remain the
        // job of the staged payment path.
        if definition.has_convoke() || definition.is_creature() && facts.convoke_creatures {
            discount.generic = discount.generic.saturating_add(facts.battlefield_untapped);
            for color in &mut discount.colored[..5] {
                *color = color.saturating_add(facts.battlefield_untapped);
            }
        }
        if definition.has_improvise()
            || facts.improvise
            || definition.waterbend_additional().is_some()
            || method == CastMethod::LinkedExileWaterbend
        {
            discount.generic = discount.generic.saturating_add(facts.battlefield_untapped);
        }
        if facts.delve {
            discount.generic = discount
                .generic
                .saturating_add(facts.counts[1][facts.actor.index()].total);
        }
        let price = discount.apply(printed);
        if (definition.is_creature() && facts.spell_any_mana)
            || origin == Zone::Exile
                && !matches!(method, CastMethod::Airbend | CastMethod::Foretell)
        {
            Some(spend_as_generic(price, 6))
        } else if facts.spell_any_color {
            Some(spend_as_generic(price, 5))
        } else {
            Some(price)
        }
    }

    /// Ordinary activation discounts may be resampled after targeting. A
    /// caller can pass a cost already discounted by the engine; any further
    /// discount here deliberately errs on the side of keeping it selectable.
    pub(super) fn activation_mana_lower_bound(
        &self,
        definition: Card,
        source: CardId,
        ability: u16,
        mut price: ManaCost,
        facts: &ProposalPriceFacts,
    ) -> Option<ManaCost> {
        if facts.loose_board {
            return None;
        }
        let activation = definition.activation(ability);
        let equip = activation.is_equip();
        if equip && facts.free_equip {
            return Some(ManaCost::ZERO);
        }
        let power_up = definition.power_up_abilities().contains(&ability);
        if power_up && !definition.power_up_mana_cost_alternatives().is_empty() {
            return None;
        }
        if facts.provisional_activation_reduction
            || power_up
            || definition
                .activation_generic_reductions(ability)
                .next()
                .is_some()
            || definition.activation_subtype_discount(ability).is_some()
            || definition.activation_target_color_discount(ability) != 0
            || definition
                .activation_target_counter_discount(ability)
                .is_some()
            || definition.activation_target_reduction(ability).is_some()
            || definition.activation_counter_discount(ability).is_some()
        {
            price.generic = 0;
        } else if equip {
            price.generic = price
                .generic
                .saturating_sub(u32::try_from(facts.equip_target_discount).unwrap_or(u32::MAX));
        }
        if waterbend::TapPayment::for_activation(activation, self.object_ref(source), None)
            .is_some()
        {
            // Counting every untapped permanent also accommodates an
            // unchosen X while keeping the bound independent of object type,
            // shared taps and the eventual contribution window.
            price.generic = price
                .generic
                .saturating_sub(u32::try_from(facts.battlefield_untapped).unwrap_or(u32::MAX));
        }
        if facts.activation_any_color {
            price = spend_as_generic(price, 5);
        }
        Some(price)
    }
}
