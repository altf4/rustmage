//! Non-target source choice and source-independent next-damage prevention.
//! CR609.7a follows an exact permanent spell through its stack-to-battlefield move.
use super::*;

/// A physical incarnation or a nonmana ability object on the stack.  Keeping
/// these realms distinct never substitutes an ability's source permanent.
#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize)]
#[serde(untagged)]
pub enum DamageSourceRef {
    Object(ObjectRef),
    StackObject { stack_object: u64 },
}
#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct DamageSourceChoiceView {
    pub player: Player,
    pub candidates: Vec<ObjectRef>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub stack_objects: Vec<u64>,
}
#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct ChosenSourceShieldView {
    pub id: u64,
    pub source: ObjectRef,
    pub controller: Player,
    pub chosen: DamageSourceRef,
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
                    + choice.stack_objects.len() * std::mem::size_of::<u64>()
            })
            + self.shields.len() * std::mem::size_of::<Shield>()
            + self
                .shields
                .iter()
                .map(|shield| shield.item.owned_bytes() - std::mem::size_of::<SpellView>())
                .sum::<usize>()
    }
}

pub(super) fn condition_refers_to_attachment(condition: TriggerCondition) -> bool {
    match condition {
        TriggerCondition::When { guard, event }
        | TriggerCondition::InterveningIf { guard, event } => {
            guard == StepGuard::AttachedControllerTurn || condition_refers_to_attachment(*event)
        }
        TriggerCondition::AtStep { guard, .. } => guard == StepGuard::AttachedControllerTurn,
        TriggerCondition::InZone { event, .. }
        | TriggerCondition::Granted(event)
        | TriggerCondition::OncePerTurn(event)
        | TriggerCondition::DuringOpponentsTurn(event)
        | TriggerCondition::CastFromHand(event)
        | TriggerCondition::CastFromExile(event)
        | TriggerCondition::TreasureManaSpent(event)
        | TriggerCondition::ManaSpent { event, .. } => condition_refers_to_attachment(*event),
        _ => false,
    }
}
fn attached_instruction(instruction: &Instruction) -> bool {
    match instruction {
        Instruction::CreateNonlegendaryCopyTokens { subject, .. }
        | Instruction::CreateCopyTokenForPlayer { subject, .. }
        | Instruction::Prepare { subject }
        | Instruction::Unprepare { subject }
        | Instruction::ChooseGrantKeywords { subject, .. }
        | Instruction::CreateCopyToken { subject, .. }
        | Instruction::Suspect { subject }
        | Instruction::CreateRole { subject, .. }
        | Instruction::CreateCopyTokenWithExceptions { subject, .. }
        | Instruction::CreateCounteredCopyToken { subject, .. }
        | Instruction::IfSubject { subject, .. }
        | Instruction::IfSubjectSubtype { subject, .. }
        | Instruction::GainControl { subject, .. }
        | Instruction::Connive { subject, .. }
        | Instruction::PutCountersScaled { subject, .. }
        | Instruction::AnimateSubject { subject, .. }
        | Instruction::SetBaseStats { subject, .. }
        | Instruction::SetBaseStatsFromSource { subject }
        | Instruction::SetResolvedBaseStats { subject, .. }
        | Instruction::Grow { subject, .. }
        | Instruction::GrowScaled { subject, .. }
        | Instruction::GrowResolved { subject, .. }
        | Instruction::AddCreatureTypes { subject, .. }
        | Instruction::AddBasicLandType { subject, .. }
        | Instruction::SetCreatureTypes { subject, .. }
        | Instruction::GrantKeywords { subject, .. }
        | Instruction::DoubleCounterKind { subject, .. }
        | Instruction::PutCounters { subject, .. }
        | Instruction::PutCountersMixed { subject, .. }
        | Instruction::MarkSaddled(subject)
        | Instruction::Explore(subject)
        | Instruction::GrantAllCreatureTypes(subject)
        | Instruction::RemoveAllCountersFrom(subject) => *subject == Subject::Attached,
        Instruction::FightAttached
        | Instruction::TapAttached
        | Instruction::UntapAttached
        | Instruction::AttachEquipmentToAttached { .. }
        | Instruction::SacrificeSourceIfAttachedCounters { .. }
        | Instruction::ExileAndCaptureAttachedCopy
        | Instruction::ApplyCapturedAttachedCopy => true,
        _ => false,
    }
}
fn attached_target(rule: TargetRule) -> bool {
    match rule {
        TargetRule::Filtered { base, predicate } => {
            *predicate == crate::cards::TargetPredicate::OtherThanSourceAttachment
                || attached_target(*base)
        }
        TargetRule::ManaValueExactly { base, .. }
        | TargetRule::ManaValueAtMost { base, .. }
        | TargetRule::Grouped { base, .. } => attached_target(*base),
        TargetRule::Union(rules) => rules.iter().copied().any(attached_target),
        _ => false,
    }
}

/// Explicit typed references only; card definitions and hidden zone contents
/// are never scanned for prospective damage capability or private card names.
fn packet_objects(item: &SpellView, objects: &mut Vec<ObjectRef>, stack_objects: &mut Vec<u64>) {
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
        } else if let Target::Stack(stack_object) = target.target {
            stack_objects.push(stack_object);
        }
    }
    for object in [item.entering_object].into_iter().flatten() {
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
    // A bare "you manifest dread" action does not refer to every card moved
    // while performing it. This diagnostic payload is not a semantic roster.
    if let Some(event) = &item.event_object {
        add(event.object);
        for reference in event.semantic_objects.iter().flat_map(|rows| rows.iter()) {
            match *reference {
                DamageSourceRef::Object(object) => add(object),
                DamageSourceRef::StackObject { stack_object } => stack_objects.push(stack_object),
            }
        }
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
        // Copying refers to that spell/ability object, not every frozen field
        // used as its template (for example an old Equip bearer).
        match event.spell.semantic_stack_reference() {
            Some(DamageSourceRef::Object(object)) => add(object),
            Some(DamageSourceRef::StackObject { stack_object }) => stack_objects.push(stack_object),
            None => {}
        }
    }
}

#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
impl Game {
    fn packet_attached_sources(&self, item: &SpellView, objects: &mut Vec<ObjectRef>) {
        let program = item.program();
        let uses_attachment = program.is_some_and(|(plan, body)| {
            body.iter().any(attached_instruction)
                || match plan {
                    TargetPlan::Single(spec) => attached_target(spec.rule),
                    TargetPlan::Groups(groups) => {
                        groups.iter().any(|spec| attached_target(spec.rule))
                    }
                }
        });
        let condition = match item.kind {
            StackKind::TriggeredAbility { ability } => Some(
                item.trigger_definition().triggers()[usize::from(ability)]
                    .condition
                    .event(),
            ),
            _ => None,
        };
        let attachment_event = matches!(
            condition,
            Some(
                TriggerCondition::EquippedAttacks
                    | TriggerCondition::EquippedAttacksAlone
                    | TriggerCondition::AttachedAttacksOrBlocks
            )
        );
        if uses_attachment || attachment_event {
            let object = if let Some(event) = &item.attachment_trigger {
                Some(event.recipient)
            } else {
                item.source.and_then(|source| {
                    if self.on_battlefield(source.id) && self.object_ref(source.id) == source.object
                    {
                        self.state.objects[source.id as usize].attached_to
                    } else {
                        item.source_attachment
                    }
                })
            };
            objects.extend(object);
        }
        if program.is_some_and(|(_, body)| {
            body.iter()
                .any(|i| matches!(i, Instruction::PutGrantorCounter(_)))
        }) {
            objects.extend(item.attachment_trigger.as_ref().map(|event| event.grantor));
        }
    }
    fn chosen_source_objects(&self) -> (Vec<ObjectRef>, Vec<u64>) {
        let mut stack_objects = Vec::new();
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
            self.packet_attached_sources(item, &mut objects);
            packet_objects(item, &mut objects, &mut stack_objects);
        }
        // The resolving object remains on the stack until it finishes resolving.
        if let Some(resolving) = &self.state.resolving {
            self.packet_attached_sources(&resolving.item, &mut objects);
            packet_objects(&resolving.item, &mut objects, &mut stack_objects);
        }
        for item in self.delayed_spell_copy_view() {
            self.packet_attached_sources(&item, &mut objects);
            packet_objects(&item, &mut objects, &mut stack_objects);
        }
        if let Some(paradigm) = &self.state.paradigm {
            for item in paradigm.packets() {
                packet_objects(item, &mut objects, &mut stack_objects);
            }
        }
        objects.extend(self.waiting_mana_copy_sources());
        if let Some(rows) = &self.state.death_exile_replacements {
            for row in rows
                .iter()
                .filter(|row| row.expires_turn == self.state.turn)
            {
                objects.push(row.object);
                if row.source_referred {
                    objects.push(row.source);
                }
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
            self.packet_attached_sources(item, &mut objects);
            packet_objects(item, &mut objects, &mut stack_objects);
        }
        if let Some(rows) = &self.state.delayed_target_events {
            for row in rows
                .iter()
                .filter(|row| row.expires_turn == self.state.turn)
            {
                objects.push(row.subject);
                packet_objects(&row.packet, &mut objects, &mut stack_objects);
            }
        }
        if let Some(rows) = &self.state.delayed_returns {
            for row in rows.iter() {
                objects.push(row.view.returning.object);
                objects.extend(
                    row.view
                        .returning
                        .additional_returning
                        .iter()
                        .flat_map(|rows| rows.iter())
                        .map(|r| r.object),
                );
                packet_objects(&row.item, &mut objects, &mut stack_objects);
            }
        }
        if let Some(rows) = &self.state.earthbend_returns {
            for row in rows.iter() {
                objects.push(row.subject);
                packet_objects(&row.item, &mut objects, &mut stack_objects);
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
                    self.packet_attached_sources(item, &mut objects);
                    packet_objects(item, &mut objects, &mut stack_objects);
                }
            }
            for row in &rules.turn_callbacks {
                packet_objects(&row.item, &mut objects, &mut stack_objects);
            }
        }
        if let Some(roles) = &self.state.roles {
            objects.extend(
                roles
                    .keyword_effects
                    .iter()
                    .filter(|row| {
                        row.rules.contains(Keywords::PREVENT_COMBAT_TO)
                            && row.expires_turn.is_none_or(|turn| turn >= self.state.turn)
                    })
                    .map(|row| row.recipient),
            );
            for row in &roles.source_linked_effects {
                if row.kind == SourceLinkedEffectKind::PreventDamageBy
                    && source_linked_effects::active(self, row)
                {
                    objects.extend([row.source, row.recipient]);
                }
            }
            if let Some(state) = &roles.source_prevention {
                for shield in &state.shields {
                    objects.push(shield.view.source);
                    match shield.view.chosen {
                        DamageSourceRef::Object(object) => objects.push(object),
                        DamageSourceRef::StackObject { stack_object } => {
                            stack_objects.push(stack_object)
                        }
                    }
                    objects.extend(shield.view.permanent);
                }
            }
        }
        objects.sort_unstable();
        objects.dedup();
        stack_objects.sort_unstable();
        stack_objects.dedup();
        (objects, stack_objects)
    }
    pub(super) fn choose_source_prevention(&mut self, ability: u16) {
        let player = self
            .state
            .resolving
            .as_ref()
            .expect("resolving prevention")
            .item
            .controller;
        let (candidates, stack_objects) = self.chosen_source_objects();
        assert!(!candidates.is_empty(), "resolving object is a legal source");
        self.state
            .roles
            .get_or_insert_with(Default::default)
            .source_prevention
            .get_or_insert_with(Default::default)
            .choice = Some((
            DamageSourceChoiceView {
                player,
                candidates,
                stack_objects,
            },
            ability,
        ));
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
    pub(super) fn damage_stack_source_choices(&self) -> &[u64] {
        self.state
            .roles
            .as_ref()
            .and_then(|r| r.source_prevention.as_ref())
            .and_then(|s| s.choice.as_ref())
            .map_or(&[], |(choice, _)| &choice.stack_objects)
    }
    pub(super) fn finish_source_prevention_choice(&mut self, chosen: DamageSourceRef) {
        let item = self
            .state
            .resolving
            .as_ref()
            .expect("resolving prevention")
            .item
            .clone();
        let permanent_spell = match chosen {
            DamageSourceRef::Object(object) => {
                self.object_ref(object.card) == object
                    && self.state.objects[object.card as usize].zone == Zone::Stack
                    && self.current_kinds(object.card) & (4 | 8) == 0
            }
            DamageSourceRef::StackObject { .. } => false,
        };
        let roles = self.state.roles.as_mut().unwrap();
        let id = roles.next_acquisition();
        let state = roles.source_prevention.as_mut().unwrap();
        let (choice, ability) = state.choice.take().expect("source choice");
        assert!(match chosen {
            DamageSourceRef::Object(object) => choice.candidates.contains(&object),
            DamageSourceRef::StackObject { stack_object } =>
                choice.stack_objects.contains(&stack_object),
        });
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
                            && DamageSourceRef::Object(event.before.reference) == shield.view.chosen
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
                    && (shield.view.chosen == DamageSourceRef::Object(event.source.reference)
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
        // The reflexive clause names the exact source, whose controller is
        // read when it resolves. The damage amount remains an event-time fact.
        let pending = self.state.pending_triggers.len();
        self.queue_reflexive(&item, shield.ability, Some(event.source.reference));
        if let Some(packet) = self.state.pending_triggers.get_mut(pending)
            && packet.event_object.is_none()
        {
            packet.set_event_object(EventObjectView {
                object: event.source.reference,
                facts: EventObjectFacts::Controller(event.source.controller),
                semantic_objects: None,
                conniver_creature: None,
                subtype_lki: None,
            });
        }
    }
    pub(super) fn expire_source_prevention(&mut self) {
        if let Some(roles) = &mut self.state.roles {
            roles.source_prevention = None;
        }
    }
}
