//! Pairwise evasion and exact completion of incremental blocker declarations.
use super::*;
use crate::cards::StaticAbilityFragment;

#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
impl Game {
    fn source_power_forbids_block(&self, attacker: CardId) -> bool {
        self.state.objects[attacker as usize]
            .card
            .static_ability_groups()
            .iter()
            .flat_map(|group| group.fragments)
            .any(|fragment| {
                let crate::cards::StaticAbilityFragment::CannotBeBlockedIfSourcePowerAtMost(
                    maximum,
                ) = *fragment
                else {
                    return false;
                };
                self.continuous_fragment_available(attacker, |candidate| {
                    std::ptr::eq(candidate, fragment)
                }) && self.stats(attacker).power <= maximum
            })
    }

    fn controlled_creature_block_forbidden(&self, attacker: CardId, blocker: CardId) -> bool {
        let controller = self.state.objects[attacker as usize].controller;
        self.state.static_sources.iter().copied().any(|source| {
            self.state.objects[source as usize].controller == controller
                && self.active_rule_fragment(source, |fragment| match *fragment {
                    StaticAbilityFragment::ControlledCreaturesCannotBeBlockedIfPowerOrToughnessAtMost(maximum) => {
                        let stats = self.stats(attacker);
                        stats.power <= maximum || stats.toughness <= maximum
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
            || self.source_power_forbids_block(attacker)
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

    pub(super) fn blocks_complete(&self) -> bool {
        self.block_requirements_complete()
            && self.state.attackers.iter().all(|a| {
                let count = self
                    .state
                    .selected_blocks
                    .iter()
                    .filter(|&&(_, id)| id == a.card)
                    .count();
                (count == 0 || count >= self.minimum_blockers(a.card))
                    && (!has_attacker_keyword(self, a.card, Keywords::MAX_ONE_BLOCKER)
                        || count <= 1)
            })
    }

    pub(super) fn block_preserves_completion(&self, blocker: CardId, attacker: CardId) -> bool {
        if has_attacker_keyword(self, attacker, Keywords::MAX_ONE_BLOCKER)
            && self
                .state
                .selected_blocks
                .iter()
                .any(|&(_, id)| id == attacker)
        {
            return false;
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
        if needs.is_empty() {
            return self.block_requirements_preserved(blocker, attacker);
        }
        if needs
            .iter()
            .any(|&id| has_attacker_keyword(self, id, Keywords::MAX_ONE_BLOCKER))
        {
            return false;
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
                    && needs.iter().any(|&a| self.can_block_pair(id, a)))
                .then_some(id)
            })
            .collect();
        if available.len() < needs.len() {
            return false;
        }
        // Every outstanding minimum-count obligation needs one distinct unused
        // blocker. Augmenting paths find an exact covering matching.
        let edges: Vec<Vec<_>> = needs
            .iter()
            .map(|&a| {
                available
                    .iter()
                    .enumerate()
                    .filter_map(|(i, &b)| self.can_block_pair(b, a).then_some(i))
                    .collect()
            })
            .collect();
        fn augment(
            need: usize,
            edges: &[Vec<usize>],
            assigned: &mut [Option<usize>],
            seen: &mut [bool],
        ) -> bool {
            for &blocker in &edges[need] {
                if seen[blocker] {
                    continue;
                }
                seen[blocker] = true;
                if assigned[blocker].is_none_or(|prior| augment(prior, edges, assigned, seen)) {
                    assigned[blocker] = Some(need);
                    return true;
                }
            }
            false
        }
        let mut assigned = vec![None; available.len()];
        for need in 0..needs.len() {
            if !augment(
                need,
                &edges,
                &mut assigned,
                &mut vec![false; available.len()],
            ) {
                return false;
            }
        }
        self.block_requirements_preserved(blocker, attacker)
    }
}
#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
fn has_attacker_keyword(game: &Game, id: CardId, keyword: Keywords) -> bool {
    game.keywords(id).contains(keyword)
}
