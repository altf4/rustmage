//! Non-target source choice and source-independent next-damage prevention.
//! CR609.7a follows an exact permanent spell through its stack-to-battlefield move.
use super::*;

#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct DamageSourceChoiceView {
    pub player: Player,
    pub candidates: Vec<ObjectRef>,
}
#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct ChosenSourceShieldView {
    pub id: u64,
    pub source: ObjectRef,
    pub controller: Player,
    pub chosen: ObjectRef,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub permanent: Option<ObjectRef>,
    pub turn: u32,
}
#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
#[derive(Clone, Debug, PartialEq, Eq)]
struct Shield {
    view: ChosenSourceShieldView,
    item: SpellView,
    ability: u16,
    permanent_spell: bool,
}
#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub(super) struct SourcePreventionState {
    choice: Option<(DamageSourceChoiceView, u16)>,
    shields: Vec<Shield>,
}
#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
impl SourcePreventionState {
    pub(super) fn bytes(&self) -> usize {
        std::mem::size_of::<Self>()
            + self.choice.as_ref().map_or(0, |(choice, _)| {
                choice.candidates.len() * std::mem::size_of::<ObjectRef>()
            })
            + self.shields.len() * std::mem::size_of::<Shield>()
            + self
                .shields
                .iter()
                .map(|shield| shield.item.owned_bytes() - std::mem::size_of::<SpellView>())
                .sum::<usize>()
    }
}

/// Explicit typed references only; card definitions and hidden zone contents
/// are never scanned for prospective damage capability or private card names.
fn packet_objects(item: &SpellView, objects: &mut Vec<ObjectRef>) {
    let mut add = |object| {
        if !objects.contains(&object) {
            objects.push(object);
        }
    };
    if let Some(source) = item.source {
        add(source.object);
    }
    for target in &item.targets {
        if let Some(object) = target.object {
            add(object);
        }
    }
    for object in [
        item.entering_object,
        item.source_attachment,
        item.damage_event.and_then(|e| e.dealer),
    ]
    .into_iter()
    .flatten()
    {
        add(object);
    }
    if let Some(subject) = &item.reflexive_subject {
        add(subject.object);
        for &object in &subject.additional_objects {
            add(object);
        }
    }
    if let Some(movement) = &item.source_move {
        add(movement.before);
        add(movement.after);
    }
    if let Some(crew) = &item.crew {
        for &object in &crew.contributors {
            add(object);
        }
    }
    if let Some(event) = &item.manifest_event {
        for &object in &event.graveyard_cards {
            add(object);
        }
        if let Some(object) = event.manifested {
            add(object);
        }
    }
    if let Some(event) = &item.event_object {
        add(event.object);
    }
    if let Some(event) = &item.cast_event {
        add(event.object);
    }
    if let Some(event) = &item.creature_combat_event {
        add(event.dealer);
    }
    if let Some(event) = &item.damage_received {
        add(event.recipient);
    }
    if let Some(event) = &item.attachment_trigger {
        add(event.grantor);
        add(event.recipient);
    }
    if let Some(subjects) = &item.cast_creature_subjects {
        for &object in &subjects.objects {
            add(object);
        }
    }
    if let Some(returning) = &item.return_link {
        add(returning.object);
        if let Some(rows) = &returning.additional_returning {
            for row in rows.iter() {
                add(row.object);
            }
        }
    }
    if let Some(row) = &item.delayed_sacrifice {
        add(row.source);
        for &object in &row.tokens {
            add(object);
        }
    }
    if let Some(row) = &item.delayed_warp_exile {
        add(row.source);
    }
    for rows in [
        &item.exiled_symbol_cards,
        &item.craft_materials,
        &item.exiled_trigger_cards,
    ]
    .into_iter()
    .flatten()
    {
        for &object in rows.iter() {
            add(object);
        }
    }
    if let Some(event) = &item.copy_event {
        packet_objects(&event.spell, objects);
    }
}

#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
impl Game {
    fn chosen_source_objects(&self) -> Vec<ObjectRef> {
        let mut objects = self
            .state
            .objects
            .iter()
            .enumerate()
            .filter_map(|(id, object)| {
                let id = id as CardId;
                (matches!(object.zone, Zone::Battlefield | Zone::Stack)
                    || (object.zone == Zone::Command && !self.is_face_down(self.object_ref(id))))
                .then_some(self.object_ref(id))
            })
            .collect::<Vec<_>>();
        // Objects referred to by spells/abilities currently on the stack.
        for item in &self.state.stack {
            packet_objects(item, &mut objects);
        }
        // The resolving object remains on the stack until it finishes resolving.
        if let Some(resolving) = &self.state.resolving {
            packet_objects(&resolving.item, &mut objects);
        }
        for item in self.delayed_spell_copy_view() {
            packet_objects(&item, &mut objects);
        }
        if let Some(rows) = &self.state.death_exile_replacements {
            for row in rows.iter() {
                objects.extend([row.object, row.source]);
            }
        }
        // Waiting delayed triggered abilities, including departed exact sources.
        for item in self
            .state
            .delayed_artifact_copies
            .iter()
            .flat_map(|rows| rows.iter())
            .chain(
                self.state
                    .delayed_attack_this_turn
                    .iter()
                    .flat_map(|rows| rows.iter()),
            )
            .chain(
                self.state
                    .delayed_opponent_attack_until_next_turn
                    .iter()
                    .flat_map(|rows| rows.iter()),
            )
            .chain(
                self.state
                    .delayed_main_mana
                    .iter()
                    .flat_map(|rows| rows.iter()),
            )
            .chain(
                self.state
                    .delayed_upkeep_tokens
                    .iter()
                    .flat_map(|rows| rows.iter()),
            )
        {
            packet_objects(item, &mut objects);
        }
        if let Some(rows) = &self.state.delayed_target_events {
            for row in rows.iter() {
                objects.push(row.subject);
                packet_objects(&row.packet, &mut objects);
            }
        }
        if let Some(rows) = &self.state.delayed_returns {
            for row in rows.iter() {
                packet_objects(&row.item, &mut objects);
            }
        }
        if let Some(rows) = &self.state.earthbend_returns {
            for row in rows.iter() {
                packet_objects(&row.item, &mut objects);
            }
        }
        if let Some(rows) = &self.state.delayed_sacrifices {
            for row in rows.iter() {
                objects.push(row.view.source);
                objects.extend_from_slice(&row.view.tokens);
            }
        }
        if let Some(warp) = &self.state.warp {
            for row in &warp.delayed {
                objects.push(row.view.source);
            }
        }
        if let Some(rules) = &self.state.combat_rules {
            if let Some(rows) = &rules.begin_combat_this_turn {
                for item in rows.iter() {
                    packet_objects(item, &mut objects);
                }
            }
            for row in &rules.turn_callbacks {
                packet_objects(&row.item, &mut objects);
            }
        }
        if let Some(roles) = &self.state.roles {
            for row in &roles.source_linked_effects {
                if row.kind == SourceLinkedEffectKind::PreventDamageBy
                    && source_linked_effects::active(self, row)
                {
                    objects.extend([row.source, row.recipient]);
                }
            }
            if let Some(state) = &roles.source_prevention {
                for shield in &state.shields {
                    objects.extend([shield.view.source, shield.view.chosen]);
                    objects.extend(shield.view.permanent);
                }
            }
        }
        objects.sort_unstable();
        objects.dedup();
        objects
    }
    pub(super) fn choose_source_prevention(&mut self, ability: u16) {
        let player = self
            .state
            .resolving
            .as_ref()
            .expect("resolving prevention")
            .item
            .controller;
        let candidates = self.chosen_source_objects();
        assert!(!candidates.is_empty(), "resolving object is a legal source");
        self.state
            .roles
            .get_or_insert_with(Default::default)
            .source_prevention
            .get_or_insert_with(Default::default)
            .choice = Some((DamageSourceChoiceView { player, candidates }, ability));
        self.state.pending = Pending::DamageSource { player };
    }
    pub(super) fn damage_source_choice(&self, viewer: Player) -> Option<DamageSourceChoiceView> {
        let (choice, _) = self
            .state
            .roles
            .as_ref()?
            .source_prevention
            .as_ref()?
            .choice
            .as_ref()?;
        self.shares_information(viewer, choice.player)
            .then(|| choice.clone())
    }
    pub(super) fn chosen_source_shields(&self) -> Option<Vec<ChosenSourceShieldView>> {
        let rows = &self
            .state
            .roles
            .as_ref()?
            .source_prevention
            .as_ref()?
            .shields;
        (!rows.is_empty()).then(|| rows.iter().map(|row| row.view.clone()).collect())
    }
    pub(super) fn damage_source_choices(&self) -> &[ObjectRef] {
        self.state
            .roles
            .as_ref()
            .and_then(|r| r.source_prevention.as_ref())
            .and_then(|s| s.choice.as_ref())
            .map_or(&[], |(choice, _)| &choice.candidates)
    }
    pub(super) fn finish_source_prevention_choice(&mut self, chosen: ObjectRef) {
        let item = self
            .state
            .resolving
            .as_ref()
            .expect("resolving prevention")
            .item
            .clone();
        let permanent_spell = self.object_ref(chosen.card) == chosen
            && self.state.objects[chosen.card as usize].zone == Zone::Stack
            && self.current_kinds(chosen.card) & (4 | 8) == 0;
        let roles = self.state.roles.as_mut().unwrap();
        let id = roles.next_acquisition();
        let state = roles.source_prevention.as_mut().unwrap();
        let (choice, ability) = state.choice.take().expect("source choice");
        assert!(choice.candidates.contains(&chosen));
        state.shields.push(Shield {
            view: ChosenSourceShieldView {
                id,
                source: item.source.expect("prevention source").object,
                controller: choice.player,
                chosen,
                permanent: None,
                turn: self.state.turn,
            },
            item,
            ability,
            permanent_spell,
        });
        self.run_program();
    }
    pub(super) fn capture_source_prevention_departure(&mut self, before: &ObjectView) {
        if let Some(state) = self
            .state
            .roles
            .as_mut()
            .and_then(|r| r.source_prevention.as_mut())
        {
            for shield in &mut state.shields {
                if shield
                    .item
                    .source
                    .is_some_and(|source| source.object == before.reference)
                {
                    shield.item.source.as_mut().unwrap().facts = DamageFacts::from_view(before);
                    shield.item.source_keywords = before.keywords;
                    shield.item.source_tapped = before.tapped;
                    shield.item.source_attachment = before.attached_to;
                    shield
                        .item
                        .refresh_counter_quantities(|kind| before.counters.get(kind));
                }
            }
        }
    }
    pub(super) fn advance_source_prevention_permanents(&mut self, events: &[ZoneChange]) {
        if let Some(state) = self
            .state
            .roles
            .as_mut()
            .and_then(|r| r.source_prevention.as_mut())
        {
            for shield in &mut state.shields {
                if shield.permanent_spell
                    && shield.view.permanent.is_none()
                    && let Some(event) = events.iter().find(|event| {
                        event.moved
                            && event.before.reference == shield.view.chosen
                            && event.before.zone == Zone::Stack
                            && event.destination == Zone::Battlefield
                    })
                {
                    shield.view.permanent = Some(event.after);
                }
            }
        }
    }
    pub(super) fn source_prevention_candidates(
        &self,
        event: &DamageEvent,
    ) -> Vec<DamagePreventionView> {
        let Some(state) = self
            .state
            .roles
            .as_ref()
            .and_then(|r| r.source_prevention.as_ref())
        else {
            return Vec::new();
        };
        state
            .shields
            .iter()
            .filter(|shield| {
                shield.view.turn == self.state.turn
                    && event.target == Target::Player(shield.view.controller)
                    && (shield.view.chosen == event.source.reference
                        || shield.view.permanent == Some(event.source.reference))
            })
            .map(|shield| DamagePreventionView {
                source: shield.view.source,
                source_card: shield.item.source.expect("prevention source").card,
                kind: DamagePreventionKind::ChosenSource(shield.view.id),
            })
            .collect()
    }
    pub(super) fn consume_source_prevention(&mut self, id: u64, event: &DamageEvent) {
        let state = self
            .state
            .roles
            .as_mut()
            .unwrap()
            .source_prevention
            .as_mut()
            .unwrap();
        let index = state
            .shields
            .iter()
            .position(|shield| shield.view.id == id)
            .expect("chosen shield");
        let shield = state.shields.remove(index);
        if state.shields.is_empty() && state.choice.is_none() {
            self.state.roles.as_mut().unwrap().source_prevention = None;
        }
        assert!(event.amount > 0 && event.preventable);
        let mut item = shield.item;
        item.damage_event = Some(DamageTriggerView {
            dealer: Some(event.source.reference),
            player: shield.view.controller,
            amount: event.amount as u128,
            combat: event.combat,
        });
        item.event_player = Some(event.source.controller);
        self.queue_reflexive(&item, shield.ability, None);
    }
    pub(super) fn expire_source_prevention(&mut self) {
        if let Some(roles) = &mut self.state.roles {
            roles.source_prevention = None;
        }
    }
}
