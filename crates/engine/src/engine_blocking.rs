//! Pairwise evasion and exact completion of incremental blocker declarations.
use super::*;
use crate::cards::StaticAbilityFragment;

#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
impl Game {
    fn source_forbids_block(&self, attacker: CardId, blocker: CardId) -> bool {
        self.state.objects[attacker as usize]
            .card
            .static_ability_groups()
            .iter()
            .flat_map(|group| group.fragments)
            .any(|fragment| {
                let active = || {
                    self.continuous_fragment_available(attacker, |candidate| {
                        std::ptr::eq(candidate, fragment)
                    })
                };
                match *fragment {
                    StaticAbilityFragment::CannotBeBlockedIfSourcePowerAtMost(maximum) => {
                        active() && self.stats(attacker).power <= maximum
                    }
                    StaticAbilityFragment::CannotBeBlockedByNamelessCreatures => {
                        active()
                            && ability_world::CharacteristicWorld::name(self, blocker).is_empty()
                    }
                    _ => false,
                }
            })
    }

    fn controlled_creature_block_forbidden(&self, attacker: CardId, blocker: CardId) -> bool {
        let controller = self.state.objects[attacker as usize].controller;
        let attacker_stats = std::cell::OnceCell::new();
        let blocker_power = std::cell::OnceCell::new();
        self.state.static_sources.iter().copied().any(|source| {
            self.state.objects[source as usize].controller == controller
                && self.active_rule_fragment(source, |fragment| match *fragment {
                    StaticAbilityFragment::ControlledCreaturesCannotBeBlockedIfPowerOrToughnessAtMost(maximum) => {
                        let stats = attacker_stats.get_or_init(|| self.stats(attacker));
                        stats.power <= maximum || stats.toughness <= maximum
                    }
                    StaticAbilityFragment::ControlledCreaturesCannotBeBlockedByPower {
                        attacker_maximum,
                        blocker_minimum,
                    } => {
                        attacker_stats.get_or_init(|| self.stats(attacker)).power <= attacker_maximum
                            && *blocker_power.get_or_init(|| self.stats(blocker).power)
                                >= blocker_minimum
                    }
                    StaticAbilityFragment::ControlledSubtypeCannotBeBlockedByKeyword { subtype, blocker_keyword } => {
                        self.has_subtype(attacker, subtype) && self.keywords(blocker).contains(blocker_keyword)
                    }
                    _ => false,
                })
        })
    }

    pub(super) fn can_be_blocker(&self, id: CardId) -> bool {
        self.state.objects.get(id as usize).is_some_and(|b| {
            b.zone == Zone::Battlefield
                && b.controller == self.state.active.other()
                && self.is_creature(id)
                && self.current_kinds(id) & crate::cards::BATTLE_KIND == 0
                && !b.tapped
                && !self
                    .keywords(id)
                    .intersects(Keywords::CANT_BLOCK.union(Keywords::DECAYED))
                && !self.combat_restricted(id, false)
        })
    }

    pub(super) fn can_block_pair(&self, blocker: CardId, attacker: CardId) -> bool {
        let Some(b) = self.state.objects.get(blocker as usize) else {
            return false;
        };
        let Some(a) = self.state.objects.get(attacker as usize) else {
            return false;
        };
        if !self.can_be_blocker(blocker)
            || a.zone != Zone::Battlefield
            || a.controller != self.state.active
            || !self.is_creature(attacker)
        {
            return false;
        }
        let keywords = self.keywords(attacker);
        if self.state.objects[attacker as usize]
            .card
            .static_ability_groups()
            .iter()
            .flat_map(|group| group.fragments)
            .any(|fragment| {
                matches!(
                    fragment,
                    crate::cards::StaticAbilityFragment::CannotBeBlockedByCreaturesWithLessPower
                ) && self.continuous_fragment_available(attacker, |candidate| {
                    std::ptr::eq(candidate, fragment)
                })
            })
            && self.stats(blocker).power < self.stats(attacker).power
        {
            return false;
        }
        if self.flying_block_forbidden(blocker, attacker)
            || self.source_forbids_block(attacker, blocker)
            || self.controlled_creature_block_forbidden(attacker, blocker)
            || self.blocker_subtype_excluded(attacker, blocker)
            || self.live_static_forbids_block(blocker, attacker)
            || self.blocker_power_forbidden(attacker, blocker)
            || keywords.contains(Keywords::CANT_BE_BLOCKED)
            || keywords.contains(Keywords::ONLY_HASTE_BLOCKERS)
                && !self.keywords(blocker).contains(Keywords::HASTE)
            || keywords.protects_from_controller(
                ability_world::CharacteristicWorld::colors(self, blocker),
                self.current_kinds(blocker),
                a.controller != b.controller,
            )
            || ability_world::CharacteristicWorld::protected_from_subtype(
                self,
                attacker,
                |subtype| self.has_subtype(blocker, subtype),
            )
            || keywords.contains(Keywords::NO_HUMAN_BLOCKERS) && self.has_subtype(blocker, "Human")
            || keywords.contains(Keywords::NO_TOKEN_BLOCKERS) && b.is_token()
            || keywords.contains(Keywords::FLYING)
                && !self
                    .keywords(blocker)
                    .intersects(Keywords::FLYING.union(Keywords::REACH))
        {
            return false;
        }
        if !keywords.contains(Keywords::FLYING) {
            let flying_only = |fragment: &StaticAbilityFragment| {
                matches!(fragment, StaticAbilityFragment::CanBlockOnlyFlying)
            };
            if b.card
                .static_ability_groups()
                .iter()
                .any(|group| group.fragments.iter().any(flying_only))
                && self.continuous_fragment_available(blocker, flying_only)
            {
                return false;
            }
        }
        let power_rules = Keywords::SKULK
            .union(Keywords::NO_GREATER_POWER_BLOCKERS)
            .union(Keywords::NO_POWER_LE_TWO_BLOCKERS)
            .union(Keywords::NO_POWER_GE_THREE_BLOCKERS);
        if keywords.intersects(power_rules) {
            let power = self.stats(blocker).power;
            if keywords.intersects(Keywords::SKULK.union(Keywords::NO_GREATER_POWER_BLOCKERS))
                && power > self.stats(attacker).power
                || keywords.contains(Keywords::NO_POWER_LE_TWO_BLOCKERS) && power <= 2
                || keywords.contains(Keywords::NO_POWER_GE_THREE_BLOCKERS) && power >= 3
            {
                return false;
            }
        }
        true
    }

    fn blocker_power_forbidden(&self, attacker: CardId, blocker: CardId) -> bool {
        self.state.combat_rules.as_ref().is_some_and(|rules| {
            let attacker = self.object_ref(attacker);
            rules.blocker_power_limits.iter().any(|limit| {
                limit.attacker == attacker && self.stats(blocker).power >= i128::from(limit.minimum)
            })
        }) || self.active_rule_fragment(attacker, |fragment| {
            matches!(
                fragment,
                StaticAbilityFragment::CannotBeBlockedByCreaturesWithPowerAtLeast(minimum)
                    if self.stats(blocker).power >= *minimum
            )
        })
    }

    pub(super) fn attacker_at_most_one_blocker(&self, attacker: CardId) -> bool {
        if has_attacker_keyword(self, attacker, Keywords::MAX_ONE_BLOCKER) {
            return true;
        }
        let controller = self.state.objects[attacker as usize].controller;
        let power = std::cell::OnceCell::new();
        self.state.static_sources.iter().copied().any(|source| {
            self.state.objects[source as usize].controller == controller
                && self.active_rule_fragment(source, |fragment| {
                    matches!(
                        *fragment,
                        StaticAbilityFragment::ControlledCreaturesWithPowerAtLeastMaxOneBlocker(minimum)
                            if *power.get_or_init(|| self.stats(attacker).power) >= minimum
                    )
                })
        })
    }

    pub(super) fn minimum_blockers(&self, attacker: CardId) -> usize {
        if has_attacker_keyword(self, attacker, Keywords::MIN_THREE_BLOCKERS) {
            3
        } else if has_attacker_keyword(self, attacker, Keywords::MENACE) {
            2
        } else {
            1
        }
    }

    pub(super) fn blocks_complete(&self, taxed: bool) -> bool {
        !matches!(self.state.selected_blocks.as_slice(), [(sole, _)]
            if self.combat_requires_company(*sole))
            && (taxed || self.block_requirements_complete())
            && self.state.attackers.iter().all(|a| {
                let count = self
                    .state
                    .selected_blocks
                    .iter()
                    .filter(|&&(_, id)| id == a.card)
                    .count();
                (count == 0 || count >= self.minimum_blockers(a.card))
                    && (!self.attacker_at_most_one_blocker(a.card) || count <= 1)
            })
    }

    /// Additional blockers needed to accompany one already legal block pair.
    pub(super) fn minimum_blocking_company(
        &self,
        blocker: CardId,
        attacker: CardId,
        available: &[CardId],
    ) -> Option<usize> {
        self.state
            .attackers
            .iter()
            .filter_map(|attack| {
                let max_one = self.attacker_at_most_one_blocker(attack.card);
                let needed = if attack.card == attacker {
                    if max_one {
                        return None;
                    }
                    1
                } else {
                    self.minimum_blockers(attack.card)
                };
                if max_one && needed > 1 {
                    return None;
                }
                (available
                    .iter()
                    .filter(|&&id| id != blocker && self.can_block_pair(id, attack.card))
                    .take(needed)
                    .count()
                    == needed)
                    .then_some(needed)
            })
            .min()
    }

    /// Minimum size of a structurally legal declaration extending this choice.
    pub(super) fn minimum_block_declaration_with(
        &self,
        blocker: CardId,
        attacker: CardId,
        taxed: bool,
    ) -> Option<usize> {
        let selected = self.state.selected_blocks.len() + 1;
        if self.attacker_at_most_one_blocker(attacker)
            && self
                .state
                .selected_blocks
                .iter()
                .any(|&(_, id)| id == attacker)
        {
            return None;
        }
        let needs: Vec<_> = self
            .state
            .attackers
            .iter()
            .filter_map(|a| {
                let minimum = self.minimum_blockers(a.card);
                let count = self
                    .state
                    .selected_blocks
                    .iter()
                    .filter(|&&(_, id)| id == a.card)
                    .count()
                    + usize::from(a.card == attacker);
                (count > 0 && count < minimum).then(|| (a.card, minimum - count))
            })
            .flat_map(|(id, remaining)| std::iter::repeat_n(id, remaining))
            .collect();
        let needs_company =
            selected == 1 && needs.is_empty() && self.combat_requires_company(blocker);
        if needs.is_empty() && !needs_company {
            return (taxed || self.block_requirements_preserved(blocker, attacker))
                .then_some(selected);
        }
        if needs
            .iter()
            .any(|&id| self.attacker_at_most_one_blocker(id))
        {
            return None;
        }
        let available: Vec<_> = self
            .state
            .objects
            .iter()
            .enumerate()
            .filter_map(|(id, _)| {
                let id = id as CardId;
                (id != blocker
                    && !self.state.selected_blocks.iter().any(|&(b, _)| b == id)
                    && (needs_company && self.can_be_blocker(id)
                        || needs.iter().any(|&a| self.can_block_pair(id, a))))
                .then_some(id)
            })
            .collect();
        let extra = if needs_company {
            self.minimum_blocking_company(blocker, attacker, &available)?
        } else {
            if !combat_allocations::cover_block_slots(self, &needs, &available) {
                return None;
            }
            needs.len()
        };
        (taxed || self.block_requirements_preserved(blocker, attacker)).then_some(selected + extra)
    }

    pub(super) fn finish_block_declaration(&mut self, actor: Player) {
        let declared: Vec<_> = self
            .state
            .selected_blocks
            .iter()
            .map(|&(blocker, _)| blocker)
            .collect();
        self.state.blocking.extend(declared.iter().copied());
        let first_new_block = self.state.blocks.len();
        self.state.blocks.append(&mut self.state.selected_blocks);
        self.attachment_block_triggers(actor);
        self.self_block_triggers(&declared, first_new_block);
        self.opponent_blocker_triggers(actor, &declared);
        let mut newly_blocked = Vec::new();
        for attacker in &mut self.state.attackers {
            let blocked = self.state.blocks.iter().any(|&(_, a)| a == attacker.card);
            if blocked && !attacker.blocked {
                newly_blocked.push(attacker.card);
            }
            attacker.blocked |= blocked;
        }
        self.block_relationship_triggers(first_new_block, &newly_blocked);
        self.give_priority();
    }
}
#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
fn has_attacker_keyword(game: &Game, id: CardId, keyword: Keywords) -> bool {
    game.keywords(id).contains(keyword)
}
