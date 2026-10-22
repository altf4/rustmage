//! Immutable characteristic evaluation scoped to one borrowed game world.
//! None records a computed no-removal result. The borrow prevents mutation until
//! every dependent view has finished; no derived data enters snapshots or branches.
use super::ability_evaluation::AbilityEvaluation;
use super::ability_world::CharacteristicWorld;
use super::*;
use crate::cards::{LiveStatValue, LiveStaticEffect, RoleKind, StaticAbilityFragment};

pub(super) struct CharacteristicQuery<'a> {
    game: &'a Game,
    evaluation: Option<AbilityEvaluation>,
}
impl std::ops::Deref for CharacteristicQuery<'_> {
    type Target = Game;
    fn deref(&self) -> &Game {
        self.game
    }
}
impl Game {
    /// Central current-name query. Views freeze this value for last-known information.
    pub(super) fn object_name(&self, id: CardId) -> &'static str {
        CharacteristicWorld::name(self, id)
    }
    pub(super) fn characteristic_query(&self) -> CharacteristicQuery<'_> {
        CharacteristicQuery {
            game: self,
            evaluation: ability_evaluation::evaluate_if_needed(self),
        }
    }
}
impl CharacteristicQuery<'_> {
    pub(super) fn creature_matches(
        &self,
        filter: CreatureFilter,
        controller: Player,
        source: ObjectRef,
        id: CardId,
    ) -> bool {
        let object = &self.state.objects[id as usize];
        object.zone == Zone::Battlefield
            && self.is_creature(id)
            && filter.matches_characteristics(
                object.card,
                CharacteristicWorld::is_token(self, id),
                self.current_kinds(id),
                self.granted_subtypes(id),
                object.controller == controller,
                self.object_ref(id) == source,
                self.state.attackers.iter().any(|a| a.card == id),
            )
            && (filter.keyword == Keywords::NONE || self.keywords(id).contains(filter.keyword))
    }
    pub(super) fn stats(&self, id: CardId) -> Stats {
        self.stats_without_hand(id, None)
    }
    pub(super) fn base_stats_without_hand(&self, id: CardId, departing: Option<CardId>) -> Stats {
        let object = &self.state.objects[id as usize];
        let mut stats = if let Some((power, toughness)) = object.card.vehicle_stats() {
            if !self.is_creature(id) {
                return Stats::default();
            }
            Stats { power, toughness }
        } else {
            match object.card.effect() {
                Effect::Creature(power, toughness) => Stats {
                    power: i128::from(power),
                    toughness: i128::from(toughness),
                },
                _ => Stats::default(),
            }
        };
        // Layer 7a applies in every zone, before setters and additive modifiers.
        if let Some(base) = object.card.dynamic_stats()
            && self.continuous_fragment_available(id, |fragment| {
                matches!(fragment, StaticAbilityFragment::DynamicStats(..))
            })
        {
            let reference = self.object_ref(id);
            let power = base.power.map(|expr| {
                self.stat_expression_without_hand(expr, reference, object.controller, departing)
            });
            if let Some(power) = power {
                stats.power = power;
            }
            if let Some(expr) = base.toughness {
                stats.toughness = if let Some(p) = base.power
                    && p.value == expr.value
                    && p.multiplier == expr.multiplier
                {
                    power
                        .unwrap()
                        .checked_add(expr.offset)
                        .and_then(|v| v.checked_sub(p.offset))
                        .capacity("toughness exhausted")
                } else {
                    self.stat_expression_without_hand(expr, reference, object.controller, departing)
                };
            }
        }
        let battlefield_creature = object.zone == Zone::Battlefield && self.is_creature(id);
        if battlefield_creature {
            stats = self.apply_base_stat_layers(id, stats, departing);
        }
        stats
    }
    pub(super) fn stats_without_hand(&self, id: CardId, departing: Option<CardId>) -> Stats {
        self.layered_stats_without_hand(id, departing).1
    }
    pub(super) fn layered_stats_without_hand(
        &self,
        id: CardId,
        departing: Option<CardId>,
    ) -> (Stats, Stats) {
        let object = &self.state.objects[id as usize];
        let base = self.base_stats_without_hand(id, departing);
        // A Vehicle has no effective creature stats while it is not a creature.
        // Counters remain on it, ready for a later successful crew.
        if object.card.vehicle_stats().is_some() && !self.is_creature(id) {
            return (base, base);
        }
        let mut stats = base;
        let battlefield_creature = object.zone == Zone::Battlefield && self.is_creature(id);
        if battlefield_creature
            || matches!(object.card.effect(), Effect::Creature(..))
            || object.card.vehicle_stats().is_some()
            || object.card.dynamic_stats().is_some()
        {
            let dynamic = object.card.dynamic_stats();
            let power_resource = if dynamic.is_some_and(|base| base.power.is_some()) {
                "power exhausted"
            } else if object.card.vehicle_stats().is_some() {
                "Vehicle power exhausted"
            } else {
                "power representation exhausted"
            };
            let toughness_resource = if dynamic.is_some_and(|base| base.toughness.is_some()) {
                "toughness exhausted"
            } else if object.card.vehicle_stats().is_some() {
                "Vehicle toughness exhausted"
            } else {
                "toughness representation exhausted"
            };
            stats.power = stats
                .power
                .checked_add(object.growth.power)
                .and_then(|v| v.checked_add(object.counter_delta()))
                .capacity(power_resource);
            stats.toughness = stats
                .toughness
                .checked_add(object.growth.toughness)
                .and_then(|v| v.checked_add(object.counter_delta()))
                .capacity(toughness_resource);
        }
        if !battlefield_creature {
            return (base, stats);
        }
        if let Some(roles) = &self.state.roles {
            let recipient = self.object_ref(id);
            for row in &roles.stat_modifiers {
                if row.recipient == recipient {
                    stats += Stats {
                        power: row.power,
                        toughness: row.toughness,
                    };
                }
            }
        }
        stats += self.chosen_bonus(id);
        stats += self.live_static_bonus(id);
        stats += self.attachment_bonus(id).0;
        // Only the small live-source index is traversed. Central zone transitions
        // maintain it, and snapshots own it; no derived stat cache needs invalidation.
        for &source in &self.state.static_sources {
            let object = &self.state.objects[source as usize];
            if object.card.static_ability_groups().iter().any(|group| {
                group
                    .fragments
                    .iter()
                    .any(|fragment| matches!(fragment, StaticAbilityFragment::DeathBaronLord))
            }) && ability_world::CharacteristicWorld::death_baron_applies(
                self,
                self.object_ref(source),
                id,
            ) && self.continuous_fragment_available(source, |fragment| {
                matches!(fragment, StaticAbilityFragment::DeathBaronLord)
            }) {
                stats += Stats {
                    power: 1,
                    toughness: 1,
                };
            }
            for group in object.card.static_ability_groups() {
                for fragment in group.fragments {
                    let StaticAbilityFragment::ControlledCreatureScaling {
                        subtype,
                        value,
                        stats: scale,
                    } = *fragment
                    else {
                        continue;
                    };
                    if subtype.is_some_and(|subtype| !self.has_subtype(id, subtype))
                        || object.controller != self.state.objects[id as usize].controller
                        || Some(source) == departing
                        || !self.continuous_fragment_available(source, |fragment| matches!(fragment,
                            StaticAbilityFragment::ControlledCreatureScaling { subtype: t, value: v, stats: s } if *t == subtype && *v == value && *s == scale)) {
                        continue;
                    }
                    let amount = self.stat_value_without_hand(
                        value,
                        self.object_ref(source),
                        object.controller,
                        departing,
                    );
                    stats += Stats {
                        power: scale
                            .power
                            .checked_mul(amount)
                            .capacity("controlled scaling power exhausted"),
                        toughness: scale
                            .toughness
                            .checked_mul(amount)
                            .capacity("controlled scaling toughness exhausted"),
                    };
                }
            }
            for (index, modifier) in object.card.scaling_modifiers().iter().enumerate() {
                if !self.continuous_fragment_available(source, |fragment| {
                    matches!(fragment,
                    StaticAbilityFragment::ScalingModifier(slot, _) if usize::from(*slot) == index)
                }) {
                    continue;
                }
                let applies = match modifier.subject {
                    Subject::Source => source == id,
                    Subject::Attached => {
                        !self.is_creature(source) && object.attached_to == Some(self.object_ref(id))
                    }
                    Subject::Targets | Subject::Entering => {
                        unreachable!("event or targeted scaling static modifier")
                    }
                };
                if applies {
                    let value = self.stat_value_without_hand(
                        modifier.value,
                        self.object_ref(source),
                        object.controller,
                        departing,
                    );
                    stats += Stats {
                        power: modifier
                            .stats
                            .power
                            .checked_mul(value)
                            .capacity("power exhausted"),
                        toughness: modifier
                            .stats
                            .toughness
                            .checked_mul(value)
                            .capacity("toughness exhausted"),
                    };
                }
            }
            for (index, modifier) in object.card.static_modifiers().iter().enumerate() {
                if !self.continuous_fragment_available(source, |fragment| {
                    matches!(fragment,
                    StaticAbilityFragment::StaticModifier(slot, _) if usize::from(*slot) == index)
                }) {
                    continue;
                }
                if (!modifier.own_turn || self.state.active == object.controller)
                    && (!modifier.requires_life_gain
                        || self.state.players[object.controller.index()].life_gained_this_turn > 0)
                    && self.creature_matches(
                        modifier.filter,
                        object.controller,
                        self.object_ref(source),
                        id,
                    )
                {
                    stats += modifier.stats;
                }
            }
            if !object.card.enchanted_modifiers().is_empty() && self.is_enchanted(id) {
                for (index, modifier) in object.card.enchanted_modifiers().iter().enumerate() {
                    if !self.continuous_fragment_available(source, |fragment| matches!(fragment,
                        StaticAbilityFragment::EnchantedModifier(slot, _) if usize::from(*slot) == index)) { continue; }
                    if self.creature_matches(
                        modifier.filter,
                        object.controller,
                        self.object_ref(source),
                        id,
                    ) {
                        stats += modifier.stats;
                    }
                }
            }
        }
        (base, stats)
    }
    pub(super) fn attachment_bonus(&self, id: CardId) -> (Stats, Keywords) {
        let mut stats = Stats::default();
        let mut keywords = Keywords::NONE;
        if !self.on_battlefield(id) || !self.is_creature(id) {
            return (stats, keywords);
        }
        let target = self.object_ref(id);
        let evaluation = self.evaluation.as_ref();
        for &source in &self.state.attachment_sources {
            let object = &self.state.objects[source as usize];
            if object.attached_to == Some(target) && !self.is_creature(source) {
                if let Some(evaluation) = &evaluation {
                    for (index, group) in object.card.static_ability_groups().iter().enumerate() {
                        if !evaluation.group_available(self.object_ref(source), index) {
                            continue;
                        }
                        for fragment in group.fragments {
                            match *fragment {
                                StaticAbilityFragment::AttachmentStats(value) => stats += value,
                                StaticAbilityFragment::AttachmentTurnStats(value)
                                    if object.controller == self.state.active =>
                                {
                                    stats += value
                                }
                                StaticAbilityFragment::AttachmentKeywords(value) => {
                                    keywords = keywords.union(value)
                                }
                                StaticAbilityFragment::AttachedTurnKeywords(value)
                                    if object.controller == self.state.active =>
                                {
                                    keywords = keywords.union(value)
                                }
                                _ => (),
                            }
                        }
                    }
                    continue;
                }
                let spec = object.card.attachment_bonus().unwrap();
                stats += spec.stats;
                keywords = keywords.union(spec.keywords);
                if object.controller == self.state.active {
                    keywords = keywords.union(object.card.attached_turn_keywords());
                    for group in object.card.static_ability_groups() {
                        for fragment in group.fragments {
                            if let StaticAbilityFragment::AttachmentTurnStats(value) = *fragment {
                                stats += value;
                            }
                        }
                    }
                }
            }
        }
        (stats, keywords)
    }
    pub(super) fn keywords(&self, id: CardId) -> Keywords {
        if matches!(
            self.state.objects[id as usize].zone,
            Zone::Battlefield | Zone::Stack
        ) && let Some(evaluation) = self.evaluation.as_ref()
        {
            return evaluation.result.objects[id as usize].keywords;
        }
        if self.state.objects[id as usize].zone == Zone::Stack {
            let definition = self.effective_spell_card(id);
            if definition.inset_parent().is_some() {
                return definition
                    .info()
                    .keywords
                    .union(keyword_counters::keywords(self.counter_amounts(id)));
            }
        }
        let target = &self.state.objects[id as usize];
        let mut keywords = target
            .keywords()
            .union(self.copied_keywords(self.object_ref(id)))
            .union(self.attachment_bonus(id).1)
            .union(CharacteristicWorld::counter_subtype_keywords(self, id));
        if target.zone != Zone::Battlefield {
            return keywords.union(keyword_counters::keywords(self.counter_amounts(id)));
        }
        if self.is_suspected(id) {
            keywords = keywords.union(Keywords::MENACE).union(Keywords::CANT_BLOCK);
        }
        let reference = self.object_ref(id);
        let attacking = self.state.attackers.iter().any(|a| a.card == id);
        for &source in &self.state.static_sources {
            let object = &self.state.objects[source as usize];
            if self.is_creature(id) && target.controller == object.controller {
                for &(kind, grant) in object.card.counter_keyword_grants() {
                    if self.counter_count(id, kind) > 0 {
                        keywords = keywords.union(grant);
                    }
                }
            }
            for group in object.card.static_ability_groups() {
                for fragment in group.fragments {
                    match *fragment {
                        StaticAbilityFragment::ChosenCreatureKeywords(grant)
                            if ability_world::CharacteristicWorld::chosen_creature_type_applies(
                                self,
                                self.object_ref(source),
                                id,
                            ) =>
                        {
                            keywords = keywords.union(grant);
                        }
                        StaticAbilityFragment::ControlledTappedKeywords {
                            other,
                            keywords: grant,
                        } if object.controller == target.controller
                            && target.tapped
                            && self.is_creature(id)
                            && (!other || source != id) =>
                        {
                            keywords = keywords.union(grant)
                        }
                        StaticAbilityFragment::DeathBaronLord
                            if ability_world::CharacteristicWorld::death_baron_applies(
                                self,
                                self.object_ref(source),
                                id,
                            ) =>
                        {
                            keywords = keywords.union(Keywords::DEATHTOUCH)
                        }
                        StaticAbilityFragment::CounterKeywordOwnTurn(kind, grant)
                            if object.controller == self.state.active
                                && target.controller == object.controller
                                && self.is_creature(id)
                                && self.counter_count(id, kind) > 0 =>
                        {
                            keywords = keywords.union(grant)
                        }
                        _ => (),
                    }
                }
            }
            for grant in object.card.keyword_grants() {
                if grant.condition.is_none_or(|value| {
                    self.stat_value(value, self.object_ref(source), object.controller) != 0
                }) && (!grant.requires_life_gain
                    || self.state.players[object.controller.index()].life_gained_this_turn > 0)
                    && grant.matches_characteristics(
                        target.card,
                        target.is_token(),
                        self.current_kinds(id),
                        self.granted_subtypes(id),
                        target.controller == object.controller,
                        reference == self.object_ref(source),
                        attacking,
                        object.controller == self.state.active,
                    )
                {
                    keywords = keywords.union(grant.keywords);
                }
            }
        }
        keywords.union(self.live_static_keywords(id))
    }
    pub(super) fn apply_base_stat_layers(
        &self,
        id: CardId,
        mut stats: Stats,
        departing: Option<CardId>,
    ) -> Stats {
        let object = self.object_ref(id);
        let cursed = self.cursed_role_timestamp(object);
        let mut power_time = cursed;
        let mut toughness_time = cursed;
        if cursed.is_some() {
            stats = Stats {
                power: 1,
                toughness: 1,
            };
        }
        if let Some(state) = &self.state.roles {
            for effect in &state.base_stats {
                if effect.recipient != object {
                    continue;
                }
                if effect.static_source {
                    let source = &self.state.objects[effect.source.card as usize];
                    if source.zone != Zone::Battlefield
                        || self.object_ref(effect.source.card) != effect.source
                    {
                        continue;
                    }
                    let Some(spec) = source.card.base_stat_setter() else {
                        continue;
                    };
                    if !self.continuous_fragment_available(effect.source.card, |fragment| {
                        matches!(fragment, StaticAbilityFragment::BaseStatSetter(..))
                    }) {
                        continue;
                    }
                    if spec.own_turn && self.state.active != source.controller {
                        continue;
                    }
                    if spec.condition.is_some_and(|condition| {
                        self.stat_value_without_hand(
                            condition,
                            effect.source,
                            source.controller,
                            departing,
                        ) == 0
                    }) {
                        continue;
                    }
                }
                if let Some(power) = effect.power
                    && power_time.is_none_or(|t| effect.timestamp > t)
                {
                    stats.power = power;
                    power_time = Some(effect.timestamp);
                }
                if let Some(toughness) = effect.toughness
                    && toughness_time.is_none_or(|t| effect.timestamp > t)
                {
                    stats.toughness = toughness;
                    toughness_time = Some(effect.timestamp);
                }
            }
        }
        stats = self.apply_live_base_stat_layers(id, stats, &mut power_time, &mut toughness_time);
        for (time, (power, toughness)) in characteristic_layers::base_changes(self, id) {
            if power_time.is_none_or(|old| time > old) {
                stats.power = power;
                power_time = Some(time);
            }
            if toughness_time.is_none_or(|old| time > old) {
                stats.toughness = toughness;
                toughness_time = Some(time);
            }
        }
        stats
    }
    fn live_static_applies(
        &self,
        row: &LiveStaticSourceView,
        effect: &LiveStaticEffect,
        index: usize,
        id: CardId,
    ) -> bool {
        self.continuous_fragment_available(row.source.card, |fragment| matches!(fragment,
            StaticAbilityFragment::LiveBase(slot, _) | StaticAbilityFragment::LiveBonus(slot, _)
            | StaticAbilityFragment::LiveKeywords(slot, _) | StaticAbilityFragment::LiveBlockRestriction(slot, _)
            if usize::from(*slot) == index))
            && ability_world::CharacteristicWorld::live_applies(self, row.source, effect, id)
    }
    fn live_stat_value(&self, value: LiveStatValue, source: ObjectRef) -> i128 {
        match value {
            LiveStatValue::Fixed(value) => value,
            LiveStatValue::ControllerLife => {
                self.state.players[self.state.objects[source.card as usize].controller.index()].life
            }
        }
    }
    pub(super) fn apply_live_base_stat_layers(
        &self,
        id: CardId,
        mut stats: Stats,
        power_time: &mut Option<u64>,
        toughness_time: &mut Option<u64>,
    ) -> Stats {
        if let Some(r) = &self.state.roles {
            for row in &r.live_static {
                for (index, effect) in row.effects.iter().enumerate() {
                    if !self.live_static_applies(row, effect, index, id) {
                        continue;
                    }
                    if let Some(power) = effect.power
                        && power_time.is_none_or(|t| row.timestamp > t)
                    {
                        stats.power = self.live_stat_value(power, row.source);
                        *power_time = Some(row.timestamp);
                    }
                    if let Some(toughness) = effect.toughness
                        && toughness_time.is_none_or(|t| row.timestamp > t)
                    {
                        stats.toughness = self.live_stat_value(toughness, row.source);
                        *toughness_time = Some(row.timestamp);
                    }
                }
            }
        }
        stats
    }
    pub(super) fn live_static_bonus(&self, id: CardId) -> Stats {
        let mut stats = Stats::default();
        if let Some(r) = &self.state.roles {
            for row in &r.live_static {
                for (index, effect) in row.effects.iter().enumerate() {
                    if self.live_static_applies(row, effect, index, id) {
                        stats += Stats {
                            power: effect.bonus_power,
                            toughness: effect.bonus_toughness,
                        };
                    }
                }
            }
        }
        stats
    }
    pub(super) fn live_static_keywords(&self, id: CardId) -> Keywords {
        let mut keywords = Keywords::NONE;
        if let Some(r) = &self.state.roles {
            for row in &r.live_static {
                for (index, effect) in row.effects.iter().enumerate() {
                    if self.live_static_applies(row, effect, index, id) {
                        keywords = keywords.union(effect.keywords);
                    }
                }
            }
        }
        keywords
    }
    pub(super) fn live_static_forbids_block(&self, blocker: CardId, attacker: CardId) -> bool {
        self.state.roles.as_ref().is_some_and(|r| {
            r.live_static.iter().any(|row| {
                row.effects.iter().enumerate().any(|(index, effect)| {
                    self.live_static_applies(row, effect, index, blocker)
                        && effect.cannot_block_subtype.is_some_and(|subtype| {
                            self.granted_subtypes(attacker)
                                .has(self.state.objects[attacker as usize].card, subtype)
                        })
                })
            })
        })
    }
    pub(super) fn cursed_role_timestamp(&self, recipient: ObjectRef) -> Option<u64> {
        self.state.roles.as_ref().and_then(|r| {
            r.attached
                .iter()
                .filter(|v| {
                    v.recipient == recipient
                        && self.on_battlefield(v.aura.card)
                        && self.object_ref(v.aura.card) == v.aura
                        && self.state.objects[v.aura.card as usize].card.role()
                            == Some(RoleKind::Cursed)
                        && self.state.objects[v.aura.card as usize].attached_to == Some(recipient)
                        && self.continuous_fragment_available(v.aura.card, |fragment| {
                            matches!(
                                fragment,
                                crate::cards::StaticAbilityFragment::PredefinedRoleBase
                            )
                        })
                })
                .map(|v| v.timestamp)
                .max()
        })
    }
    pub(super) fn continuous_fragment_available(
        &self,
        source: CardId,
        matches: impl Fn(&StaticAbilityFragment) -> bool,
    ) -> bool {
        if !self.on_battlefield(source) {
            return true;
        }
        let Some(evaluation) = self.evaluation.as_ref() else {
            return true;
        };
        let reference = self.object_ref(source);
        let mut bound = false;
        let mut available = false;
        for (index, group) in self.state.objects[source as usize]
            .card
            .static_ability_groups()
            .iter()
            .enumerate()
        {
            if group.fragments.iter().any(&matches) {
                bound = true;
                available |= evaluation.group_available(reference, index);
            }
        }
        assert!(
            bound,
            "continuous helper has no immutable source-group identity"
        );
        available
    }
    pub(super) fn ability_availability(&self, id: CardId) -> Option<Box<AbilityAvailabilityView>> {
        if !self.on_battlefield(id) {
            return None;
        }
        let evaluation = self.evaluation.as_ref()?;
        let object = &evaluation.result.objects[id as usize];
        Some(Box::new(AbilityAvailabilityView {
            printed: object.printed,
            grants: object
                .grants
                .iter()
                .map(|grant| GrantedAbilityView {
                    grantor: grant.source,
                    granting_card: grant.granting_card,
                    ability: grant.ability,
                    grantor_controller: grant.grantor_controller,
                    timestamp: grant.timestamp,
                })
                .collect(),
        }))
    }
    pub(super) fn object_view(&self, id: CardId) -> ObjectView {
        let obj = &self.state.objects[id as usize];
        let definition = if obj.zone == Zone::Stack {
            self.effective_spell_card(id)
        } else {
            obj.card
        };
        let inset = definition.inset_parent().is_some();
        let (base, stats) = if inset {
            (Stats::default(), Stats::default())
        } else {
            self.layered_stats_without_hand(id, None)
        };
        ObjectView {
            abilities: self.ability_availability(id),
            kinds: self.current_kinds(id),
            colors: CharacteristicWorld::colors(self, id),
            granted_subtypes: self.granted_subtypes(id),
            entry_choice: self.entry_value(self.object_ref(id)),
            saddled: self.is_saddled(self.object_ref(id)),
            id,
            reference: self.object_ref(id),
            card: definition,
            name: CharacteristicWorld::name(self, id),
            owner: obj.owner,
            controller: obj.controller,
            zone: obj.zone,
            tapped: obj.tapped,
            summoning_sick: obj.summoning_sick,
            damage: obj.damage,
            damaged_this_turn: obj.damaged_this_turn,
            dealt_damage_this_turn: obj.dealt_damage_this_turn,
            counters: self.counter_amounts(id).to_owned(),
            used_abilities: obj.used_abilities,
            exhausted_abilities: obj.used_abilities & obj.card.exhaust_mask(),
            attached_to: obj.attached_to,
            modified: self.is_modified(id),
            attacking: self.state.attackers.iter().any(|a| a.card == id),
            power: stats.power,
            toughness: stats.toughness,
            power_above_base: obj.zone == Zone::Battlefield
                && self.is_creature(id)
                && stats.power > base.power,
            keywords: if inset {
                definition.info().keywords
            } else {
                self.keywords(id)
            },
            copy_recipe: if obj.is_copy_token() {
                self.copy_recipe(self.object_ref(id))
            } else {
                None
            },
            token: self.object_token_info(id),
            emblem: obj.card.emblem_info(),
        }
    }
}

impl CharacteristicWorld for CharacteristicQuery<'_> {
    fn game(&self) -> &Game {
        self.game
    }
    fn object(&self, id: CardId) -> &Object {
        &self.game.state.objects[id as usize]
    }
    fn life(&self, player: Player) -> i128 {
        CharacteristicWorld::life(self.game, player)
    }
    fn gained_life(&self, player: Player) -> bool {
        CharacteristicWorld::gained_life(self.game, player)
    }
    fn attacking(&self, id: CardId) -> bool {
        CharacteristicWorld::attacking(self.game, id)
    }
    fn printed_abilities_available(&self, id: CardId) -> bool {
        self.object(id).zone != Zone::Battlefield
            || self
                .evaluation
                .as_ref()
                .is_none_or(|v| v.result.objects[id as usize].printed)
    }
}
