//! Frozen state-based events and snapshot-owned legend choices (CR704/101.4).
use super::*;

#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) struct LegendGroup {
    pub controller: Player,
    pub name: &'static str,
    pub objects: Vec<ObjectRef>,
    pub keep: Option<ObjectRef>,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) struct SbaBatch {
    resume: Pending,
    deaths: Vec<CardId>,
    ceased: Vec<CardId>,
    detached: Vec<ObjectRef>,
    pairs: Vec<(ObjectRef, u64)>,
    lost: [bool; 2],
    pub groups: Vec<LegendGroup>,
    pub roles: Vec<RoleGroupView>,
}
impl SbaBatch {
    pub fn bytes(&self) -> usize {
        std::mem::size_of::<Self>()
            + self.resume.owned_bytes()
            + (self.deaths.len() + self.ceased.len()) * std::mem::size_of::<CardId>()
            + self.pairs.len() * std::mem::size_of::<(ObjectRef, u64)>()
            + self.detached.len() * std::mem::size_of::<ObjectRef>()
            + self.roles.len() * std::mem::size_of::<RoleGroupView>()
            + self.roles.iter().map(|g| g.objects.len()).sum::<usize>()
                * std::mem::size_of::<ObjectRef>()
            + self.groups.len() * std::mem::size_of::<LegendGroup>()
            + self.groups.iter().map(|g| g.objects.len()).sum::<usize>()
                * std::mem::size_of::<ObjectRef>()
    }
}
impl Game {
    /// Track attacked permanent types at each continuous-effect boundary.
    fn refresh_combat_defender_types(&mut self) {
        let mut current = Vec::<(ObjectRef, u16)>::new();
        for attack in &self.state.attackers {
            let Some(CombatDefender::Battle(object) | CombatDefender::Planeswalker(object)) =
                attack.defender
            else {
                continue;
            };
            if current.iter().any(|(prior, _)| *prior == object) {
                continue;
            }
            let kinds =
                if self.on_battlefield(object.card) && self.object_ref(object.card) == object {
                    self.current_kinds(object.card) & (64 | crate::cards::BATTLE_KIND)
                } else {
                    0
                };
            current.push((object, kinds));
        }
        let removed: Vec<_> = current
            .iter()
            .filter_map(|&(object, kinds)| {
                let old = self
                    .state
                    .combat_defender_types
                    .as_ref()?
                    .iter()
                    .find(|(prior, _)| *prior == object)?
                    .1;
                let walker = old & 64 != 0;
                let battle = old & crate::cards::BATTLE_KIND != 0;
                let now_walker = kinds & 64 != 0;
                let now_battle = kinds & crate::cards::BATTLE_KIND != 0;
                let remove = if walker && battle {
                    (!now_walker && !now_battle)
                        || (now_walker
                            && !now_battle
                            && self.battle_protector(object)
                                != Some(self.state.objects[object.card as usize].controller))
                } else {
                    (walker && !now_walker) || (battle && !now_battle)
                };
                remove.then_some(object)
            })
            .collect();
        for attack in &mut self.state.attackers {
            if matches!(attack.defender, Some(CombatDefender::Battle(r) | CombatDefender::Planeswalker(r)) if removed.contains(&r))
            {
                attack.defender = None;
            }
        }
        current.retain(|(object, _)| !removed.contains(object));
        self.state.combat_defender_types = (!current.is_empty()).then(|| current.into());
    }
    pub(super) fn refresh_combat_battle_types(&mut self) {
        self.refresh_combat_defender_types();
        if self.state.attackers.is_empty() && self.state.blocking.is_empty() {
            self.state.battle_combat_types = None;
            return;
        }
        if self.state.battle_combat_types.as_ref().is_some_and(|old| {
            old.len() == self.state.attackers.len() + self.state.blocking.len()
                && old
                    .iter()
                    .zip(
                        self.state
                            .attackers
                            .iter()
                            .map(|a| a.card)
                            .chain(self.state.blocking.iter().copied()),
                    )
                    .all(|(&(prior, value), id)| {
                        prior == self.object_ref(id)
                            && value == (self.current_kinds(id) & crate::cards::BATTLE_KIND != 0)
                    })
        }) {
            return;
        }
        // A combat creature that becomes a Battle leaves combat, even if
        // it remains a creature. Track each exact combat incarnation.
        let combat_types: Vec<_> = self
            .state
            .attackers
            .iter()
            .map(|a| a.card)
            .chain(self.state.blocking.iter().copied())
            .map(|id| {
                (
                    self.object_ref(id),
                    self.current_kinds(id) & crate::cards::BATTLE_KIND != 0,
                )
            })
            .collect();
        let newly_battles: Vec<_> = combat_types
            .iter()
            .filter_map(|&(object, battle)| {
                (battle
                    && self.state.battle_combat_types.as_ref().is_some_and(|old| {
                        old.iter()
                            .any(|&(prior, was_battle)| prior == object && !was_battle)
                    }))
                .then_some(object)
            })
            .collect();
        for object in newly_battles {
            self.remove_from_combat(object.card, object);
        }
        let retained: Vec<_> = combat_types
            .into_iter()
            .filter(|(object, _)| {
                self.state.attackers.iter().any(|a| a.card == object.card)
                    || self.state.blocking.contains(&object.card)
            })
            .collect();
        self.state.battle_combat_types = (!retained.is_empty()).then(|| retained.into());
    }
    fn collect_sba_batch(&self) -> SbaBatch {
        let mut batch = SbaBatch {
            resume: self.state.pending.clone(),
            deaths: Vec::new(),
            ceased: Vec::new(),
            detached: Vec::new(),
            pairs: Vec::new(),
            groups: Vec::new(),
            roles: Vec::new(),
            lost: self.allowed_game_losses(
                self.state
                    .players
                    .each_ref()
                    .map(|p| p.life <= 0 || p.poison_counters >= 10 || p.attempted_empty_draw),
            ),
        };
        // A batch observes one immutable battlefield. Reuse its layer evaluation
        // across creatures, but rebuild it after any batch changes the world.
        let characteristics = std::cell::OnceCell::new();
        let mut legends: [std::collections::BTreeMap<&'static str, Vec<ObjectRef>>; 2] =
            std::array::from_fn(|_| std::collections::BTreeMap::new());
        for (index, object) in self.state.objects.iter().enumerate() {
            let id = index as CardId;
            if object.is_token() && !matches!(object.zone, Zone::Battlefield | Zone::Outside) {
                batch.ceased.push(id);
            }
            if object.copy_kind == CopyKind::Spell
                && !matches!(object.zone, Zone::Stack | Zone::Outside)
            {
                batch.ceased.push(id);
                continue;
            }
            if object.copy_kind == CopyKind::Card
                && !matches!(object.zone, Zone::Stack | Zone::Outside)
                && !(object.zone == Zone::Exile && self.keeps_card_copy(self.object_ref(id)))
            {
                batch.ceased.push(id);
                continue;
            }
            if object.zone != Zone::Battlefield {
                continue;
            }
            if self.battle_zero_defense_sba(id) {
                batch.deaths.push(id);
            }
            if self.saga_sacrifice_required(id) {
                batch.deaths.push(id);
            }
            if self.current_kinds(id) & 64 != 0 && object.counters.loyalty == 0
                && !self.state.objects.iter().enumerate().any(|(provider, source)| {
                    let provider = provider as CardId;
                    source.zone == Zone::Battlefield && source.controller == object.controller
                        && source.card.static_ability_groups().iter().any(|group| group.fragments.iter().any(|f| matches!(f, crate::cards::StaticAbilityFragment::ControlledPlaneswalkersSurviveZeroLoyalty)))
                        && self.continuous_fragment_available(provider, |f| matches!(f, crate::cards::StaticAbilityFragment::ControlledPlaneswalkersSurviveZeroLoyalty))
                }) {
                batch.deaths.push(id);
            }
            if self.current_kinds(id) & crate::cards::BATTLE_KIND != 0
                && object.attached_to.is_some()
            {
                batch.detached.push(self.object_ref(id));
            } else if ability_world::CharacteristicWorld::is_aura(self, id) {
                // CR303.4d/704.5p: a creature Aura first becomes unattached;
                // the next SBA sees the unattached Aura and puts it away.
                if self.is_creature(id) && object.attached_to.is_some() {
                    batch.detached.push(self.object_ref(id));
                } else if !self.printed_abilities_available(id)
                    || !object.attached_to.is_some_and(|target| {
                        self.aura_can_enchant(object.card, object.controller, target)
                    })
                {
                    batch.deaths.push(id);
                }
            } else if let Some(target) = object.attached_to
                && (!ability_world::CharacteristicWorld::is_equipment(self, id)
                    || self.is_creature(id)
                    || target == self.object_ref(id)
                    || !self.on_battlefield(target.card)
                    || self.object_ref(target.card) != target
                    || !self.is_creature(target.card)
                    || characteristics
                        .get_or_init(|| self.characteristic_query())
                        .keywords(target.card)
                        .contains(Keywords::PROTECTION_EVERYTHING))
            {
                batch.detached.push(self.object_ref(id));
            }
            if !self.state.players[object.controller.index()].legend_rule_ignored_this_turn
                && ability_world::CharacteristicWorld::legendary(self, id)
            {
                legends[object.controller.index()]
                    .entry(crate::card_names::comparison_key(self.object_name(id)))
                    .or_default()
                    .push(self.object_ref(id));
            }
            if self.is_creature(id) {
                let query = characteristics.get_or_init(|| self.characteristic_query());
                let toughness = query.stats(id).toughness;
                if toughness <= 0
                    || ((object.damage >= toughness || object.deathtouch_since_sba)
                        && !query.keywords(id).contains(Keywords::INDESTRUCTIBLE))
                {
                    batch.deaths.push(id);
                }
            }
            let pairs = object.counters.plus_one.min(object.counters.minus_one);
            if pairs > 0 {
                batch.pairs.push((self.object_ref(id), pairs));
            }
        }
        for controller in [self.state.active, self.state.active.other()] {
            let groups = std::mem::take(&mut legends[controller.index()]);
            for (name, objects) in groups {
                if objects.len() > 1 {
                    batch.groups.push(LegendGroup {
                        controller,
                        name,
                        objects,
                        keep: None,
                    });
                }
            }
        }
        batch.roles = self.collect_role_choices(&mut batch.deaths);
        batch
    }

    pub(super) fn sba_required(&self) -> bool {
        let b = self.collect_sba_batch();
        !b.deaths.is_empty()
            || !b.detached.is_empty()
            || !b.ceased.is_empty()
            || !b.pairs.is_empty()
            || !b.groups.is_empty()
            || !b.roles.is_empty()
            || b.lost.iter().any(|&lost| lost)
    }
    pub(super) fn state_based_actions(&mut self) {
        if self.state.step == Step::Pregame
            || self.state.sba_batch.is_some()
            || self.state.outcome.is_some()
            || self.state.entry_batch.is_some()
        {
            return;
        }
        loop {
            self.refresh_controllers();
            // An ineligible protector is replaced even during combat. A missing
            // protector is chosen only while no creature is attacking the Battle.
            let protectors: Vec<_> = self
                .state
                .objects
                .iter()
                .enumerate()
                .filter_map(|(id, o)| {
                    let id = id as CardId;
                    if o.zone != Zone::Battlefield
                        || self.current_kinds(id) & crate::cards::BATTLE_KIND == 0
                    {
                        return None;
                    }
                    let reference = self.object_ref(id);
                    let expected = if self.has_subtype(id, "Siege") {
                        o.controller.other()
                    } else {
                        o.controller
                    };
                    let prior = self.battle_protector(reference);
                    let attacking = self.state.attackers.iter().any(|a| {
                        matches!(a.defender, Some(CombatDefender::Battle(r) | CombatDefender::Planeswalker(r)) if r == reference)
                    });
                    ((prior.is_some() && prior != Some(expected)) || (prior.is_none() && !attacking))
                        .then_some((reference, expected))
                })
                .collect();
            for (reference, player) in protectors {
                self.set_battle_protector(reference, player);
            }
            self.refresh_combat_battle_types();
            // CR506.4: a permanent that stops being a creature leaves combat.
            let no_longer_creatures: Vec<_> = self
                .state
                .attackers
                .iter()
                .map(|a| a.card)
                .chain(self.state.blocking.iter().copied())
                .filter(|&id| !self.is_creature(id))
                .collect();
            for id in no_longer_creatures {
                self.remove_from_combat(id, self.object_ref(id));
            }
            // CR506.4c/d: losing planeswalker type ends only its defending role.
            // Regaining that type later cannot reconnect the existing attack.
            let invalid_defenders: Vec<_> = self
                .state
                .attackers
                .iter()
                .chain(&self.state.selected_attackers)
                .filter_map(|a| match a.defender {
                    Some(CombatDefender::Planeswalker(r))
                        if !self.on_battlefield(r.card)
                            || self.object_ref(r.card) != r
                            || self.current_kinds(r.card) & (64 | crate::cards::BATTLE_KIND)
                                == 0 =>
                    {
                        Some(r)
                    }
                    Some(CombatDefender::Battle(r))
                        if !self.on_battlefield(r.card)
                            || self.object_ref(r.card) != r
                            || self.current_kinds(r.card) & (64 | crate::cards::BATTLE_KIND)
                                == 0 =>
                    {
                        Some(r)
                    }
                    _ => None,
                })
                .collect();
            for attack in self
                .state
                .attackers
                .iter_mut()
                .chain(&mut self.state.selected_attackers)
            {
                if let Some(CombatDefender::Planeswalker(r) | CombatDefender::Battle(r)) =
                    attack.defender
                    && invalid_defenders.contains(&r)
                {
                    attack.defender = None;
                }
            }
            let batch = self.collect_sba_batch();
            // Consume prevented empty-library draws at this actual SBA check.
            for p in [Player::P0, Player::P1] {
                if self.state.players[p.index()].attempted_empty_draw && self.cant_lose_game(p) {
                    self.state.players[p.index()].attempted_empty_draw = false;
                }
            }
            let changed = !batch.deaths.is_empty()
                || !batch.detached.is_empty()
                || !batch.ceased.is_empty()
                || !batch.pairs.is_empty()
                || !batch.groups.is_empty()
                || !batch.roles.is_empty()
                || batch.lost.iter().any(|&lost| lost);
            // The check consumes this interval's deathtouch history, even when
            // indestructible prevents destruction. New entrants start clear.
            // Preserve shared hot rows when there is no history to consume.
            if self
                .state
                .objects
                .iter()
                .any(|object| object.deathtouch_since_sba)
            {
                for object in &mut self.state.objects {
                    object.deathtouch_since_sba = false;
                }
            }
            if !changed {
                return;
            }
            if !batch.groups.is_empty() || !batch.roles.is_empty() {
                self.state.sba_batch = Some(Box::new(batch));
                self.advance_sba_choices();
                return;
            }
            self.commit_sba_batch(batch);
            if self.state.outcome.is_some() || self.state.entry_batch.is_some() {
                return;
            }
        }
    }

    pub(super) fn keep_legend(&mut self, object: ObjectRef) {
        let batch = self.state.sba_batch.as_mut().unwrap();
        let group = batch
            .groups
            .iter_mut()
            .find(|group| group.keep.is_none() && group.objects.contains(&object))
            .unwrap();
        group.keep = Some(object);
        self.advance_sba_choices();
    }

    pub(super) fn advance_sba_choices(&mut self) {
        let batch = self.state.sba_batch.as_ref().unwrap();
        for player in [self.state.active, self.state.active.other()] {
            if batch
                .groups
                .iter()
                .any(|g| g.controller == player && g.keep.is_none())
            {
                self.state.pending = Pending::Legend { player };
                return;
            }
            if batch
                .roles
                .iter()
                .any(|g| g.controller == player && g.keep.is_none())
            {
                self.state.pending = Pending::Role { player };
                return;
            }
        }
        let batch = *self.state.sba_batch.take().unwrap();
        self.commit_sba_batch(batch);
    }

    fn commit_sba_batch(&mut self, mut batch: SbaBatch) {
        for group in &batch.groups {
            let keep = group.keep.expect("every legend choice precedes the event");
            batch.deaths.extend(
                group
                    .objects
                    .iter()
                    .filter(|&&object| object != keep)
                    .map(|object| object.card),
            );
        }
        for group in &batch.roles {
            let keep = group
                .keep
                .expect("every Role choice precedes the simultaneous event");
            batch
                .deaths
                .extend(group.objects.iter().filter(|&&r| r != keep).map(|r| r.card));
        }
        batch.deaths.sort_unstable();
        batch.deaths.dedup();
        let sources = if batch.deaths.is_empty() {
            Vec::new()
        } else {
            self.death_trigger_sources()
        };
        let sacrifices: Vec<_> = batch
            .deaths
            .iter()
            .filter(|&&id| self.saga_sacrifice_required(id))
            .map(|&id| (self.state.objects[id as usize].controller, id))
            .collect();
        self.food_sacrifice_triggers(&sacrifices, &sources);
        let mut events = self.move_objects_raw(&batch.deaths, Zone::Graveyard);
        for event in &mut events {
            if let Some(&(actor, _)) = sacrifices.iter().find(|&&(_, id)| id == event.before.id) {
                event.sacrificed_by = Some(actor);
            }
        }
        for source in batch.detached {
            if self.object_ref(source.card) == source {
                self.state.objects[source.card as usize].attached_to = None;
                self.renew_copy_grant_origin(source, false);
            }
        }
        // Cancellation belongs to the frozen event, never a newly returned object.
        for (object, pairs) in batch.pairs {
            if self.object_ref(object.card) == object {
                let counters = &mut self.state.objects[object.card as usize].counters;
                counters.plus_one -= pairs;
                counters.minus_one -= pairs;
            }
        }
        for id in batch.ceased {
            if matches!(
                self.state.objects[id as usize].copy_kind,
                CopyKind::Card | CopyKind::Spell
            ) {
                self.cease_card_copy(self.object_ref(id));
            } else {
                self.cease_token(id);
            }
        }
        self.finish_zone_events(&events, &sources);
        self.state.pending = batch.resume;
        self.commit_game_losses(batch.lost);
    }
}
