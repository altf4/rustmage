//! Cold combat history, resolved requirements and exact damage accounting.
use super::*;

#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
pub struct CombatDamagePlayerView {
    pub source: ObjectRef,
    pub player: Player,
}
/// Actual player combat damage by recipient seat during the current turn.
#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize)]
pub struct PlayerCombatDamageView {
    pub amounts: [u128; 2],
    pub by_legendary_creature: [bool; 2],
}
#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub(super) struct CombatRulesState {
    pub additional_combats: u64,
    pub after_main: Option<MainPhaseCombatView>,
    pub inserted_phases: Vec<Step>,
    pub end_steps: AdditionalEndStepsView,
    pub turn_callbacks: Vec<TurnCallbackView>,
    pub attack_without_defender: Vec<ObjectRef>,
    #[allow(
        clippy::box_collection,
        reason = "Cold delayed packets use one pointer when absent"
    )]
    pub begin_combat_this_turn: Option<Box<Vec<SpellView>>>,
    pub must_be_blocked: Vec<ObjectRef>,
    pub sneak_entries: Option<SharedVec<ObjectRef>>,
    pub must_attack_this_combat: Vec<ObjectRef>,
    pub require_all_attack_until_next_turn: [usize; 2],
    pub cannot_block_this_turn: Vec<ObjectRef>,
    pub damaged_players: Vec<CombatDamagePlayerView>,
    pub player_damage_this_turn: PlayerCombatDamageView,
    excess: Option<ExcessDamage>,
}
#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
#[derive(Clone, Debug, PartialEq, Eq)]
struct ExcessDamage {
    source: ObjectRef,
    target: ObjectRef,
    lethal: i128,
    dealt: i128,
    destroy_subtype: Option<&'static str>,
    qualified_for_destruction: bool,
}
#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
impl CombatRulesState {
    pub(super) fn owned_bytes(&self) -> usize {
        std::mem::size_of::<Self>()
            + self.inserted_phases.len() * std::mem::size_of::<Step>()
            + self.attack_without_defender.len() * std::mem::size_of::<ObjectRef>()
            + self
                .turn_callbacks
                .iter()
                .map(|row| std::mem::size_of::<TurnCallbackView>() + row.item.owned_bytes())
                .sum::<usize>()
            + self.begin_combat_this_turn.as_ref().map_or(0, |rows| {
                std::mem::size_of::<Vec<SpellView>>()
                    + rows
                        .iter()
                        .map(|row| std::mem::size_of::<SpellView>() + row.owned_bytes())
                        .sum::<usize>()
            })
            + (self.must_be_blocked.len()
                + self.must_attack_this_combat.len()
                + self.cannot_block_this_turn.len())
                * std::mem::size_of::<ObjectRef>()
            + self.damaged_players.len() * std::mem::size_of::<CombatDamagePlayerView>()
            + self.sneak_entries.as_ref().map_or(0, |rows| {
                std::mem::size_of::<Vec<ObjectRef>>()
                    + rows.len() * std::mem::size_of::<ObjectRef>()
            })
    }
}
#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
impl Game {
    pub(super) fn distribute_counters(&mut self, item: &SpellView, kind: CounterKind) {
        let changes = item
            .target_allocations
            .as_ref()
            .into_iter()
            .flat_map(|rows| rows.iter())
            .filter(|row| item.targets.contains(&row.target) && self.same_target_object(row.target))
            .filter_map(|row| {
                row.target
                    .object
                    .map(|object| (object, Counters::of(kind, row.amount)))
            })
            .collect();
        self.effect_counters(changes, replacements::ResumeMode::Program);
    }
    pub(super) fn damage_by_group(&mut self, item: &SpellView, first: i32, other: i32) {
        let source = self.program_damage_source(item);
        let events: Vec<_> = item
            .targets
            .iter()
            .filter(|b| self.same_target_object(**b))
            .map(|bound| DamageEvent {
                source,
                target: bound.target,
                amount: i128::from(if bound.group == 0 { first } else { other }),
                combat: false,
                preventable: true,
            })
            .collect();
        self.apply_damage_batch(&events);
    }
    pub(super) fn damage_for_excess(&mut self, item: &SpellView, amount: AmountValue) {
        self.damage_for_tracked_target(item, amount, None);
    }
    pub(super) fn damage_for_subtype_destruction(
        &mut self,
        item: &SpellView,
        amount: u16,
        subtype: &'static str,
    ) {
        self.damage_for_tracked_target(item, AmountValue::Fixed(amount), Some(subtype));
    }
    fn damage_for_tracked_target(
        &mut self,
        item: &SpellView,
        amount: AmountValue,
        destroy_subtype: Option<&'static str>,
    ) {
        let source = self.program_damage_source(item);
        let Some(bound) = item
            .targets
            .first()
            .filter(|b| self.same_target_object(**b))
        else {
            return;
        };
        if let Some(state) = self.state.combat_rules.as_mut() {
            state.excess = None;
        }
        if let Some(target) = bound.object {
            // Excess is the greatest result for every applicable permanent type
            // (CR 120.10), equivalently the least applicable threshold.
            let lethal = self.excess_damage_threshold(source, target.card);
            self.state
                .combat_rules
                .get_or_insert_with(Default::default)
                .excess = Some(ExcessDamage {
                source: source.reference,
                target,
                lethal,
                dealt: 0,
                destroy_subtype,
                qualified_for_destruction: false,
            });
        }
        let amount =
            i128::try_from(self.item_amount(amount, item)).capacity("damage amount exhausted");
        self.apply_damage_batch(&[DamageEvent {
            source,
            target: bound.target,
            amount,
            combat: false,
            preventable: true,
        }]);
    }
    fn excess_damage_threshold(&self, source: DamageSource, target: CardId) -> i128 {
        let kinds = self.current_kinds(target);
        let creature = (kinds & 1 != 0).then(|| self.lethal_assignment(source, target));
        let loyalty =
            (kinds & 64 != 0).then(|| i128::from(self.counter_count(target, CounterKind::Loyalty)));
        let defense = (kinds & crate::cards::BATTLE_KIND != 0)
            .then(|| i128::from(self.counter_count(target, CounterKind::Defense)));
        [creature, loyalty, defense]
            .into_iter()
            .flatten()
            .min()
            .unwrap_or(0)
    }

    /// Freeze the threshold after all replacement side effects, before the
    /// simultaneous damage batch changes marked damage or loyalty counters.
    pub(super) fn refresh_excess_threshold(&mut self, events: &[DamageEvent]) {
        let tracked = self
            .state
            .combat_rules
            .as_ref()
            .and_then(|rules| rules.excess.as_ref());
        let Some(tracked) = tracked else { return };
        // Preserve the threshold once this tracked damage has begun; a later
        // split batch must not replace it after marking damage.
        if tracked.dealt != 0 {
            return;
        }
        let Some(event) = events.iter().find(|event| {
            event.source.reference == tracked.source
                && matches!(event.target, Target::Card(id) if self.object_ref(id) == tracked.target)
        }) else {
            return;
        };
        let lethal = self.excess_damage_threshold(event.source, tracked.target.card);
        self.state
            .combat_rules
            .as_mut()
            .unwrap()
            .excess
            .as_mut()
            .unwrap()
            .lethal = lethal;
    }

    /// Called only for actual positive damage, after replacement/prevention and
    /// before that event changes life or marked damage. Later type changes do
    /// not change whether the recipient was a Dragon when it was damaged.
    pub(super) fn qualify_damage_destruction(&mut self, event: &DamageEvent) {
        let Target::Card(id) = event.target else {
            return;
        };
        let target = self.object_ref(id);
        let subtype = self
            .state
            .combat_rules
            .as_ref()
            .and_then(|r| r.excess.as_ref())
            .filter(|r| r.source == event.source.reference && r.target == target)
            .and_then(|r| r.destroy_subtype);
        if let Some(subtype) = subtype {
            let qualifies = self.has_subtype(id, subtype);
            self.state
                .combat_rules
                .as_mut()
                .unwrap()
                .excess
                .as_mut()
                .unwrap()
                .qualified_for_destruction |= qualifies;
        }
    }
    pub(super) fn destroy_damaged_subtype(&mut self, item: &SpellView) {
        let tracked = self
            .state
            .combat_rules
            .as_mut()
            .and_then(|r| r.excess.take());
        let Some(tracked) = tracked else {
            return;
        };
        if tracked.source != self.program_damage_source(item).reference
            || tracked.dealt <= 0
            || !tracked.qualified_for_destruction
        {
            return;
        }
        let targets: Vec<_> = item
            .targets
            .iter()
            .copied()
            .filter(|bound| {
                bound.object == Some(tracked.target)
                    && self.same_target_object(*bound)
                    && self.on_battlefield(tracked.target.card)
                    && !self.effect_destruction_prevented(tracked.target.card)
            })
            .collect();
        self.move_target_batch(&targets, Zone::Graveyard);
    }
    pub(super) fn record_combat_and_excess_damage(&mut self, events: &[DamageEvent]) {
        for event in events.iter().filter(|e| e.amount > 0) {
            if event.combat
                && let Target::Player(player) = event.target
            {
                let row = CombatDamagePlayerView {
                    source: event.source.reference,
                    player,
                };
                let rules = self.state.combat_rules.get_or_insert_with(Default::default);
                if !rules.damaged_players.contains(&row) {
                    rules.damaged_players.push(row);
                }
                let history = &mut rules.player_damage_this_turn;
                let recipient = player.index();
                history.amounts[recipient] = history.amounts[recipient]
                    .checked_add(event.amount.unsigned_abs())
                    .capacity("player combat damage history exhausted");
                history.by_legendary_creature[recipient] |= event.source.facts.legendary_creature;
            }
            let target = match event.target {
                Target::Card(id) => Some(self.object_ref(id)),
                _ => None,
            };
            if let Some(excess) = self
                .state
                .combat_rules
                .as_mut()
                .and_then(|r| r.excess.as_mut())
                && event.source.reference == excess.source
                && target == Some(excess.target)
            {
                excess.dealt = excess
                    .dealt
                    .checked_add(event.amount)
                    .capacity("excess damage exhausted");
            }
        }
    }
    pub(super) fn player_combat_damage_this_turn(&self) -> Option<&PlayerCombatDamageView> {
        self.state
            .combat_rules
            .as_ref()
            .map(|rules| &rules.player_damage_this_turn)
            .filter(|history| history.amounts != [0; 2])
    }
    pub(super) fn any_player_combat_damage_at_least(&self, minimum: u128) -> bool {
        self.player_combat_damage_this_turn()
            .is_some_and(|history| history.amounts.into_iter().any(|amount| amount >= minimum))
    }
    pub(super) fn player_was_dealt_legendary_combat_damage(&self, player: Player) -> bool {
        self.player_combat_damage_this_turn()
            .is_some_and(|history| history.by_legendary_creature[player.index()])
    }
    pub(super) fn tracked_excess_damage(&self) -> u64 {
        let amount = self
            .state
            .combat_rules
            .as_ref()
            .and_then(|r| r.excess.as_ref())
            .map_or(0, |excess| {
                excess.dealt.saturating_sub(excess.lethal).max(0)
            });
        u64::try_from(amount).capacity("excess amass counter amount exhausted")
    }
    pub(super) fn clear_tracked_excess_damage(&mut self) {
        if let Some(state) = self.state.combat_rules.as_mut() {
            state.excess = None;
        }
    }
    pub(super) fn take_tracked_excess_damage(&mut self) -> u64 {
        let amount = self.tracked_excess_damage();
        self.clear_tracked_excess_damage();
        amount
    }
    pub(super) fn create_excess_tokens(&mut self, item: &SpellView, token: Card) {
        let excess = self
            .state
            .combat_rules
            .as_mut()
            .and_then(|r| r.excess.take());
        if let Some(excess) = excess {
            let amount = excess.dealt.saturating_sub(excess.lethal).max(0) as u128;
            if amount > 0 {
                self.create_tokens(item.controller, token, amount, false);
            }
        }
    }
    pub(super) fn exile_excess_play(&mut self, item: &SpellView, next_turn: bool) {
        let excess = self
            .state
            .combat_rules
            .as_mut()
            .and_then(|r| r.excess.take());
        let Some(excess) = excess else { return };
        let amount = usize::try_from(excess.dealt.saturating_sub(excess.lethal).max(0))
            .unwrap_or(usize::MAX);
        let ids: Vec<_> = self.state.players[item.controller.index()]
            .library
            .iter()
            .rev()
            .filter(|&&id| self.state.objects[id as usize].is_card())
            .take(amount)
            .copied()
            .collect();
        let events = self.move_objects_raw(&ids, Zone::Exile);
        for event in &events {
            if event.moved && event.destination == Zone::Exile {
                self.grant_exile_play_permission(item.controller, event.after, next_turn);
            }
        }
        self.finish_zone_events(&events, &[]);
    }
    pub(super) fn add_combat(&mut self) {
        let count = &mut self
            .state
            .combat_rules
            .get_or_insert_with(Default::default)
            .additional_combats;
        *count = count
            .checked_add(1)
            .capacity("additional combat count exhausted");
    }
    pub(super) fn take_additional_combat(&mut self) -> bool {
        if let Some(rules) = &mut self.state.combat_rules
            && rules.additional_combats > 0
        {
            rules.additional_combats -= 1;
            true
        } else {
            false
        }
    }
    pub(super) fn require_blocks(&mut self, item: &SpellView, filter: CreatureFilter) {
        let recipients: Vec<_> = self
            .state
            .objects
            .iter()
            .enumerate()
            .filter(|(id, object)| {
                object.zone == Zone::Battlefield
                    && item.source.is_some_and(|source| {
                        self.creature_matches(filter, item.controller, source.object, *id as CardId)
                    })
            })
            .map(|(id, _)| self.object_ref(id as CardId))
            .collect();
        self.add_block_requirements(recipients);
    }
    pub(super) fn require_targets_blocked(&mut self, item: &SpellView, group: u16) {
        let recipients: Vec<_> = item
            .targets
            .iter()
            .filter(|bound| bound.group == group && self.same_target_object(**bound))
            .filter_map(|bound| bound.object)
            .filter(|object| self.on_battlefield(object.card))
            .collect();
        self.add_block_requirements(recipients);
    }
    fn add_block_requirements(&mut self, recipients: Vec<ObjectRef>) {
        let rules = self.state.combat_rules.get_or_insert_with(Default::default);
        // Repeated spells create independent requirements for the same object.
        rules.must_be_blocked.extend(recipients);
    }
    pub(super) fn block_requirement_weight(&self, id: CardId) -> usize {
        let matches_requirement = |fragment: &crate::cards::StaticAbilityFragment| {
            matches!(fragment, crate::cards::StaticAbilityFragment::MustBeBlocked)
        };
        let static_requirement = self.state.objects[id as usize]
            .card
            .static_ability_groups()
            .iter()
            .any(|group| group.fragments.iter().any(matches_requirement))
            && self
                .characteristic_query()
                .continuous_fragment_available(id, matches_requirement);
        let attached_requirements = if self
            .state
            .attackers
            .iter()
            .any(|attacker| attacker.card == id)
        {
            use ability_world::CharacteristicWorld;
            let query = self.characteristic_query();
            let recipient = self.object_ref(id);
            self.state
                .static_sources
                .iter()
                .filter(|&&source| {
                    let object = &self.state.objects[source as usize];
                    let applies = |fragment: &crate::cards::StaticAbilityFragment| {
                        matches!(
                            fragment,
                            crate::cards::StaticAbilityFragment::AttachedAttackerMustBeBlocked
                        )
                    };
                    object.attached_to == Some(recipient)
                        && query.is_equipment(source)
                        && query.attached(self.object_ref(source), id)
                        && object
                            .card
                            .static_ability_groups()
                            .iter()
                            .any(|group| group.fragments.iter().any(applies))
                        && query.continuous_fragment_available(source, applies)
                })
                .count()
        } else {
            0
        };
        usize::from(static_requirement)
            + attached_requirements
            + self.state.combat_rules.as_ref().map_or(0, |rules| {
                rules
                    .must_be_blocked
                    .iter()
                    .filter(|&&r| r == self.object_ref(id))
                    .count()
            })
    }
    pub(super) fn destroy_combat_damaged_players_permanents(&mut self, item: &SpellView) {
        let mut damaged = [false; 2];
        if let Some(rules) = &self.state.combat_rules {
            for row in &rules.damaged_players {
                if item
                    .source
                    .is_some_and(|source| row.source == source.object)
                {
                    damaged[row.player.index()] = true;
                }
            }
        }
        let x = u64::from(item.x.expect("announced X"));
        let mut ids: Vec<_> = self
            .state
            .objects
            .iter()
            .enumerate()
            .filter_map(|(id, object)| {
                let id = id as CardId;
                (object.zone == Zone::Battlefield
                    && damaged[object.controller.index()]
                    && self.current_kinds(id) & 2 == 0
                    && self.object_mana_value(id) == x
                    && !self.keywords(id).contains(Keywords::INDESTRUCTIBLE))
                .then_some(id)
            })
            .collect();
        ids.retain(|&id| !self.effect_destruction_prevented(id));
        let sources = self.death_trigger_sources();
        let events = self.move_objects_raw(&ids, Zone::Graveyard);
        self.finish_zone_events(&events, &sources);
    }
}
#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
impl ActivatedAbility {
    pub(crate) fn x_symbols(&self) -> u32 {
        if let Some(Instruction::ActivationXWithMinimum { symbols, .. }) = self.program.first() {
            return u32::from(*symbols);
        }
        u32::try_from(
            self.program
                .iter()
                .take_while(|i| **i == Instruction::ActivationX)
                .count(),
        )
        .expect("activation X header count")
    }
    pub(super) fn minimum_x(&self) -> u32 {
        match self.program.first() {
            Some(Instruction::ActivationXWithMinimum { minimum, .. }) => *minimum,
            _ => 0,
        }
    }
    pub(super) fn has_x_cost(&self) -> bool {
        self.x_symbols() != 0 || self.life_x_cost() || self.nonmana_x()
    }
    pub(super) fn nonmana_x(&self) -> bool {
        self.tap_x_cost() || self.program.first() == Some(&Instruction::ActivationNonmanaX)
    }
    pub(super) fn tap_x_cost(&self) -> bool {
        self.cost
            .additional
            .is_some_and(|spec| spec.kind == EffectChoiceKind::TapXPermanents)
    }
}
#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
impl Announcement {
    pub(super) fn has_x_cost(&self) -> bool {
        match self.item.kind {
            StackKind::Spell => {
                self.blight_x_cost()
                    || self.payment_x_symbols() > 0
                    || self
                        .item
                        .source
                        .expect("cost announcement has object source")
                        .card
                        .life_x_cost()
            }
            StackKind::ActivatedAbility { ability } => {
                self.item
                    .source
                    .expect("cost announcement has object source")
                    .card
                    .activation_x_symbols(ability)
                    > 0
                    || self
                        .item
                        .source
                        .expect("activation source")
                        .card
                        .activation(ability)
                        .life_x_cost()
                    || self
                        .item
                        .source
                        .expect("activation source")
                        .card
                        .activation(ability)
                        .nonmana_x()
            }
            _ => false,
        }
    }
}

// Matching handles all ordinary one-blocker requirements in polynomial time.
// Only attackers requiring a bundle of two/three blockers need subset branching.
#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
fn cover_slots(game: &Game, slots: &[CardId], available: &[CardId]) -> Option<Vec<Option<usize>>> {
    fn augment(
        game: &Game,
        slots: &[CardId],
        available: &[CardId],
        slot: usize,
        assigned: &mut [Option<usize>],
        seen: &mut [bool],
    ) -> bool {
        for (b, &blocker) in available.iter().enumerate() {
            if seen[b] || !game.can_block_pair(blocker, slots[slot]) {
                continue;
            }
            seen[b] = true;
            if assigned[b]
                .is_none_or(|prior| augment(game, slots, available, prior, assigned, seen))
            {
                assigned[b] = Some(slot);
                return true;
            }
        }
        false
    }
    let mut assigned = vec![None; available.len()];
    for slot in 0..slots.len() {
        if !augment(
            game,
            slots,
            available,
            slot,
            &mut assigned,
            &mut vec![false; available.len()],
        ) {
            return None;
        }
    }
    Some(assigned)
}
#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
impl Game {
    fn maximum_block_requirements(&self, selected: &[(CardId, CardId)]) -> Option<usize> {
        let available: Vec<_> = (0..self.state.objects.len())
            .map(|id| id as CardId)
            .filter(|&id| self.can_be_blocker(id) && !selected.iter().any(|&(b, _)| b == id))
            .collect();
        let mut slots = Vec::new();
        let mut fixed = 0;
        let mut normal = Vec::new();
        let mut bundles = Vec::new();
        for a in &self.state.attackers {
            let count = selected.iter().filter(|&&(_, id)| id == a.card).count();
            let weight = self.block_requirement_weight(a.card);
            let minimum = self.minimum_blockers(a.card);
            let max_one = self.keywords(a.card).contains(Keywords::MAX_ONE_BLOCKER);
            if count > 0 {
                if max_one && (count > 1 || minimum > 1) {
                    return None;
                }
                slots.extend(std::iter::repeat_n(a.card, minimum.saturating_sub(count)));
                fixed += weight;
            } else if weight > 0 && !(max_one && minimum > 1) {
                if minimum == 1 {
                    normal.push((a.card, weight));
                } else {
                    bundles.push((a.card, minimum, weight));
                }
            }
        }
        cover_slots(self, &slots, &available)?;
        normal.sort_by_key(|&(_, weight)| std::cmp::Reverse(weight));
        fn search(
            game: &Game,
            available: &[CardId],
            slots: &mut Vec<CardId>,
            normal: &[(CardId, usize)],
            bundles: &[(CardId, usize, usize)],
            score: usize,
        ) -> usize {
            if let Some((&(id, minimum, weight), remaining)) = bundles.split_first() {
                let without = search(game, available, slots, normal, remaining, score);
                let length = slots.len();
                slots.extend(std::iter::repeat_n(id, minimum));
                let with = if cover_slots(game, slots, available).is_some() {
                    search(game, available, slots, normal, remaining, score + weight)
                } else {
                    0
                };
                slots.truncate(length);
                without.max(with)
            } else {
                let length = slots.len();
                let mut score = score;
                for &(id, weight) in normal {
                    slots.push(id);
                    if cover_slots(game, slots, available).is_some() {
                        score += weight;
                    } else {
                        slots.pop();
                    }
                }
                slots.truncate(length);
                score
            }
        }
        Some(search(
            self, &available, &mut slots, &normal, &bundles, fixed,
        ))
    }
    pub(super) fn block_requirements_complete(&self) -> bool {
        if !self
            .state
            .attackers
            .iter()
            .any(|a| self.block_requirement_weight(a.card) > 0)
        {
            return true;
        }
        let achieved: usize = self
            .state
            .attackers
            .iter()
            .filter(|a| {
                self.state
                    .selected_blocks
                    .iter()
                    .any(|&(_, id)| id == a.card)
            })
            .map(|a| self.block_requirement_weight(a.card))
            .sum();
        Some(achieved) == self.maximum_block_requirements(&[])
    }
    pub(super) fn block_requirements_preserved(&self, blocker: CardId, attacker: CardId) -> bool {
        if !self
            .state
            .attackers
            .iter()
            .any(|a| self.block_requirement_weight(a.card) > 0)
        {
            return true;
        }
        let mut selected = self.state.selected_blocks.clone();
        selected.push((blocker, attacker));
        self.maximum_block_requirements(&selected) == self.maximum_block_requirements(&[])
    }
}
