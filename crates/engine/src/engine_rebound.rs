//! Resolution-only Rebound replacement and exact one-shot upkeep offers.
use super::*;

#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
impl Game {
    pub(super) fn grant_cast_rebound(&mut self, item: &SpellView) {
        let Some(spell) = self.live_cast_spell(item) else {
            return;
        };
        if !self.keywords(spell.card).contains(Keywords::REBOUND) {
            self.grant_stack_keywords_expiring(spell, Keywords::REBOUND, None);
        }
        // Adventure and Omen can compete without another registered provider.
        self.state.zone_rules.get_or_insert_default();
    }

    pub(super) fn spell_has_rebound(&self, item: &SpellView) -> bool {
        let Some(source) = item.source.filter(|_| item.kind == StackKind::Spell) else {
            return false;
        };
        let Some(object) = self.state.objects.get(source.id as usize) else {
            return false;
        };
        object.zone == Zone::Stack
            && self.object_ref(source.id) == source.object
            && (source.card.keywords().contains(Keywords::REBOUND)
                || self.state.roles.as_ref().is_some_and(|roles| {
                    roles.keyword_effects.iter().any(|effect| {
                        effect.recipient == source.object
                            && effect.keywords.contains(Keywords::REBOUND)
                    })
                }))
            && (self.keywords(source.id).contains(Keywords::REBOUND)
                || source.card.keywords().contains(Keywords::REBOUND))
    }

    pub(super) fn finish_spell_with_resolution_exile(&mut self, item: &SpellView) -> bool {
        let Some(source) = item.source else {
            return false;
        };
        if self.object_ref(source.id) != source.object {
            return false;
        }
        let plot = self
            .state
            .zone_rules
            .as_ref()
            .is_some_and(|rules| rules.plot_spells.contains(&source.object));
        let rebound = (item
            .cast
            .is_some_and(|cast| cast.origin == Zone::Hand && cast.caster == item.controller)
            && self.state.objects[source.id as usize].owner == item.controller
            && !item.is_copy
            && self.spell_has_rebound(item))
        .then_some(item.controller);
        let dream = self.mark_dream_spell_resolving(item);
        if rebound.is_none() && !dream && !plot {
            return false;
        }
        let before = self.chosen_spell_before(item);
        let face = item.cast.map_or(CastFace::Normal, |cast| cast.face);
        let event = self.transition_object_configured(
            before,
            Zone::Graveyard,
            false,
            Counters::default(),
            None,
            |rules| {
                rules.rebound = rebound;
                rules.plot = plot;
                rules.resolution_face = Some((item.controller, face));
            },
        );
        self.finish_zone_events(std::slice::from_ref(&event), &[]);
        true
    }

    pub(super) fn schedule_rebound(
        &mut self,
        before: &ObjectView,
        object: ObjectRef,
        player: Player,
    ) {
        let mut item = self.activation_item(player, object.card, 0);
        item.kind = StackKind::ReboundCast;
        item.source = Some(StackSourceView {
            id: before.id,
            object: before.reference,
            card: before.card,
            facts: DamageFacts::from_view(before),
        });
        item.source_keywords = before.keywords;
        item.source_counter_lki = None;
        item.source_quantity_lki = None;
        item.refer_objects([object]);
        self.state
            .linked_choices
            .get_or_insert_default()
            .delayed_steps
            .push(delayed_steps::DelayedStep {
                steps: &[Step::Upkeep],
                active_player: Some(player),
                expires_turn: None,
                item,
            });
    }

    pub(super) fn resolve_rebound_cast(&mut self, item: &SpellView) -> bool {
        let Some(object) = item.event_object.as_ref().map(|event| event.object) else {
            return false;
        };
        if self.object_ref(object.card) != object
            || self.state.objects[object.card as usize].zone != Zone::Exile
            || !self.state.objects[object.card as usize].is_card()
        {
            return false;
        }
        let player = item.controller;
        self.state.resolution_free_casts = Some(Box::new(ResolutionFreeCasts {
            permanent_type: None,
            uncast_to_hand: false,
            siege_transformed: false,
            paid: None,
            casts_remaining: Some(1),
            pending_cast: None,
            remaining_mana_value: None,
            alternative_cost: None,
            maximum_mana_value: None,
            origin: Zone::Exile,
            exile_on_graveyard: false,
            allow_lands: false,
            player,
            remaining: vec![object],
        }));
        if !self.visit_resolution_casts(player, &mut |action| {
            matches!(action, Action::CastExiledDuringResolution { .. })
        }) {
            self.state.resolution_free_casts = None;
            return false;
        }
        self.state.pending = Pending::ResolutionCasts { player };
        true
    }
}
