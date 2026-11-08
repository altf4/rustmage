//! CR702.190: alternative spell cost, not a hand activated ability.
use super::*;
use crate::cards::StaticAbilityFragment;
#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
pub(super) const COST: AdditionalCost = AdditionalCost {
    kind: EffectChoiceKind::ReturnUnblockedAttacker,
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
#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
impl Game {
    pub(super) fn record_sneak_entry(&mut self, object: ObjectRef) {
        self.state
            .combat_rules
            .get_or_insert_with(Default::default)
            .sneak_entries
            .get_or_insert_with(Default::default)
            .push(object);
    }
    pub(super) fn source_sneaked_this_turn(&self, object: ObjectRef) -> bool {
        self.state
            .combat_rules
            .as_ref()
            .and_then(|rules| rules.sneak_entries.as_ref())
            .is_some_and(|rows| rows.contains(&object))
    }
    /// Ninjutsu's unblocked attacker can be returned after blockers, including
    /// in a combat-damage or end-of-combat priority window. Sneak's own
    /// announcement check remains restricted to the blockers step.
    pub(super) fn ninjutsu_attacker(&self, actor: Player, r: ObjectRef) -> bool {
        self.state.active == actor
            && matches!(
                self.state.step,
                Step::Blockers | Step::FirstStrikeDamage | Step::CombatDamage | Step::EndCombat
            )
            && self.object_ref(r.card) == r
            && self.state.objects[r.card as usize].zone == Zone::Battlefield
            && self.state.objects[r.card as usize].controller == actor
            && self
                .state
                .attackers
                .iter()
                .any(|a| a.card == r.card && !a.blocked)
    }
    fn graveyard_sneak_rule<T>(
        &self,
        actor: Player,
        qualify: impl Fn(&StaticAbilityFragment) -> Option<(u8, T)>,
    ) -> Option<T> {
        let query = self.characteristic_query();
        self.state.static_sources.iter().find_map(|&id| {
            let source = &self.state.objects[id as usize];
            if source.zone != Zone::Battlefield || source.controller != actor {
                return None;
            }
            source
                .card
                .static_ability_groups()
                .iter()
                .flat_map(|g| g.fragments)
                .find_map(|fragment| {
                    let (minimum, result) = qualify(fragment)?;
                    (source.class_level >= minimum
                        && query.continuous_fragment_available(id, |found| {
                            std::ptr::eq(found, fragment)
                        }))
                    .then_some(result)
                })
        })
    }
    fn graveyard_sneak_permission(&self, actor: Player) -> bool {
        self.graveyard_sneak_rule(actor, |fragment| match *fragment {
            StaticAbilityFragment::CastGraveyardCreaturesUsingSneak {
                minimum_class_level,
            } => Some((minimum_class_level, ())),
            _ => None,
        })
        .is_some()
    }
    pub(super) fn granted_sneak_cost(&self, actor: Player, card: Card) -> Option<ManaCost> {
        if !card.is_creature() {
            return None;
        }
        self.graveyard_sneak_rule(actor, |fragment| match *fragment {
            StaticAbilityFragment::GrantGraveyardCreatureSneak {
                minimum_class_level,
                cost,
            } => Some((minimum_class_level, cost)),
            _ => None,
        })
    }
    pub(super) fn pay_sneak_return(&mut self, object: ObjectRef) {
        let defender = self
            .state
            .attackers
            .iter()
            .find(|attack| attack.card == object.card)
            .and_then(|attack| attack.defender);
        let announcement = self.state.announcing.as_mut().unwrap();
        let costs = announcement.cast_costs.as_mut().unwrap();
        let index = usize::from(costs.active.take().unwrap());
        costs.clauses[index].clause = CastCostClause::SneakReturn(Some(object));
        costs.clauses[index].paid = true;
        announcement.item.cast.as_mut().unwrap().sneak_defender = defender;
        self.state.pending = Pending::CastCosts {
            player: announcement.item.controller,
        };
        self.move_object(object.card, Zone::Hand);
        if self.state.entry_batch.is_none() {
            self.finish_announcement();
        }
    }
    pub(super) fn configure_sneak(item: &mut SpellView) {
        item.cast.as_mut().unwrap().method = CastMethod::Sneak;
        if item.source.unwrap().card.x_symbols() > 0 {
            item.x = Some(0);
        }
    }
    pub(super) fn has_sneak_cost(&self, actor: Player, card: CardId) -> bool {
        let object = &self.state.objects[card as usize];
        object.card.sneak_cost_options().next().is_some()
            || object.zone == Zone::Graveyard
                && object.owner == actor
                && self.granted_sneak_cost(actor, object.card).is_some()
    }
    pub(super) fn visit_graveyard_finality_cast(
        &self,
        actor: Player,
        card: CardId,
        sorcery: bool,
        emit: &mut impl FnMut(Action) -> bool,
    ) -> bool {
        let object = &self.state.objects[card as usize];
        if self.state.active != actor
            || object.zone != Zone::Graveyard
            || object.owner != actor
            || !object.is_card()
            || !object.card.is_creature()
            || !cast_origins::has_fragment(self, actor, |fragment| {
                matches!(
                    fragment,
                    StaticAbilityFragment::GraveyardSmallCreaturesWithFinality
                )
            })
        {
            return false;
        }
        let stats = self.stats(card);
        if stats.power > 1 && stats.toughness > 1 {
            return false;
        }
        (self.permitted_cast_available(
            actor,
            card,
            CastMethod::Normal,
            CastFace::Normal,
            sorcery,
            false,
        ) && emit(Action::AnnounceGraveyardFinality { card, sneak: false }))
            || (self.state.step == Step::Blockers
                && self.has_sneak_cost(actor, card)
                && self.permitted_cast_available(
                    actor,
                    card,
                    CastMethod::Sneak,
                    CastFace::Normal,
                    false,
                    false,
                )
                && emit(Action::AnnounceGraveyardFinality { card, sneak: true }))
    }
    pub(super) fn sneak_cast_available(&self, actor: Player, card: CardId) -> bool {
        let o = &self.state.objects[card as usize];
        if self.state.active != actor
            || self.state.step != Step::Blockers
            || !o.is_card()
            || !self.has_sneak_cost(actor, card)
            || !(o.zone == Zone::Hand && o.owner == actor
                || self.can_play_exiled(actor, card)
                || self.can_cast_library_top(actor, card)
                || self.has_graveyard_cast(actor, card, GraveyardCastKind::Cast)
                || o.zone == Zone::Graveyard
                    && o.owner == actor
                    && o.card.is_creature()
                    && self.graveyard_sneak_permission(actor))
        {
            return false;
        }
        self.permitted_cast_available(
            actor,
            card,
            CastMethod::Sneak,
            CastFace::Normal,
            false,
            false,
        )
    }
}
