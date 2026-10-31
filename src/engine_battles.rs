//! Battle defeat packets retain exact incarnation and use normal stack ordering.
use super::*;

impl Game {
    pub(super) fn queue_siege_defeat(&mut self, reference: ObjectRef) {
        if !self.on_battlefield(reference.card)
            || self.object_ref(reference.card) != reference
            || self.current_kinds(reference.card) & crate::cards::BATTLE_KIND == 0
            || !self.has_subtype(reference.card, "Siege")
            || !self.printed_abilities_available(reference.card)
        {
            return;
        }
        let source = self.object_view(reference.card);
        let id = self.next_stack_id();
        self.state.pending_triggers.push(SpellView {
            source: Some(StackSourceView {
                id: source.id,
                object: reference,
                card: source.card,
                facts: DamageFacts::from_view(&source),
            }),
            source_less_rule: None,
            additional_triggers: source.additional_triggers,
            source_move: None,
            borrowed_slot: None,
            self_granted_activation: None,
            target_controller_lki: None,
            crime_qualifying_target: false,
            target_kinds_lki: None,
            target_type_slots: None,
            gift: None,
            reflexive_subject: None,
            delayed_warp_exile: None,
            return_link: None,
            crew: None,
            cast_event: None,
            cast_creature_subjects: None,
            copy_event: None,
            captured_artifact_copy: None,
            source_copy_recipe: None,
            source_ability_origin: None,
            target_allocations: None,
            mana_copy_sources: None,
            is_copy: false,
            ninjutsu_defender: None,
            delayed_sacrifice: None,
            x: None,
            loyalty_x: None,
            reflexive_target_limit: None,
            attachment_trigger: None,
            cast: None,
            entering_object: None,
            evidence_collected: false,
            id,
            kind: StackKind::SiegeDefeat,

            split_second: false,
            source_keywords: source.keywords,
            source_tapped: source.tapped,
            mana_tap_bonus: None,
            source_attachment: source.attached_to,
            source_transform: None,

            related_stack: None,
            damage_event: None,
            damage_received: None,
            creature_combat_event: None,
            discard_event: None,
            draw_event: None,
            life_event: None,
            source_counter_payment: None,
            loyalty_counters_paid: 0,
            exiled_symbol_cards: None,
            source_counter_lki: None,
            source_quantity_lki: None,
            event_player: None,
            event_object: None,
            manifest_event: None,
            sacrificed_toughness: None,
            sacrificed_power: None,

            targets: Vec::new(),
            mode: None,
            controller: source.controller,
        });
    }
    pub(super) fn battle_zero_defense_sba(&self, id: CardId) -> bool {
        if self.current_kinds(id) & crate::cards::BATTLE_KIND == 0
            || self.counter_count(id, CounterKind::Defense) != 0
        {
            return false;
        }
        if !self.has_subtype(id, "Siege") {
            return true;
        }
        let source = self.object_ref(id);
        let pending = |item: &SpellView| {
            item.source
                .is_some_and(|identity| identity.object == source)
                && !matches!(
                    item.kind,
                    StackKind::Spell | StackKind::ActivatedAbility { .. }
                )
        };
        !self
            .state
            .stack
            .iter()
            .chain(&self.state.pending_triggers)
            .chain(&self.state.ordering_triggers)
            .any(pending)
            && !self
                .state
                .announcing
                .as_ref()
                .is_some_and(|a| pending(&a.item))
            && !self
                .state
                .resolving
                .as_ref()
                .is_some_and(|r| pending(&r.item))
    }
    pub(super) fn resolve_siege_defeat(&mut self, item: &SpellView) -> bool {
        let Some(source) = item.source else {
            return false;
        };
        if !self.on_battlefield(source.id) || self.object_ref(source.id) != source.object {
            return false;
        }
        let token = self.state.objects[source.id as usize].is_token();
        let event = self.move_object(source.id, Zone::Exile);
        if token
            || !event.moved
            || event.destination != Zone::Exile
            || self.object_ref(source.id) != event.after
            || self.state.objects[source.id as usize]
                .card
                .back_face()
                .is_none()
        {
            return false;
        }
        self.state.resolution_free_casts = Some(Box::new(ResolutionFreeCasts {
            siege_transformed: true,
            paid: None,
            casts_remaining: Some(1),
            pending_cast: None,
            remaining_mana_value: None,
            alternative_cost: None,
            maximum_mana_value: None,
            origin: Zone::Exile,
            exile_on_graveyard: false,
            allow_lands: false,
            player: item.controller,
            remaining: vec![event.after],
        }));
        self.state.pending = Pending::ResolutionCasts {
            player: item.controller,
        };
        true
    }
}
