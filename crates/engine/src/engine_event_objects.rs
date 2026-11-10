//! Owned untargeted event objects, distinct from the source of their ability.
use super::*;
use crate::cards::EndureRecipient;

#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
pub struct CreatureComparisonFacts {
    pub creature: bool,
    pub power: i128,
    pub toughness: i128,
    pub(super) controller: Player,
}
#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
impl CreatureComparisonFacts {
    pub(super) fn from_view(view: &ObjectView) -> Self {
        Self {
            creature: view.is_creature(),
            power: view.power,
            toughness: view.toughness,
            controller: view.controller,
        }
    }
    pub(super) fn greater_than(self, other: Self) -> bool {
        self.creature
            && other.creature
            && (self.power > other.power || self.toughness > other.toughness)
    }
}

#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub enum EventObjectFacts {
    /// Immutable pre-death multiset; never refreshed from a live/new incarnation.
    DeathCounters(Counters),
    /// Refresh at departure for a program that reads the exact permanent at resolution.
    CounterTotal(u128),
    /// Identity only; no private or unused event characteristics are captured.
    Identity,
    DamageActor(DamageSource),
    /// Frozen amount this exact event object dealt in one simultaneous damage event.
    DamageDealt {
        amount: u128,
    },
    /// Frozen amount of one counter kind placed in the triggering event.
    CountersPlaced {
        amount: u64,
    },
    DepartureManaValue(u64),
    /// Current or last-known controller of an exact untargeted object.
    Controller(Player),
    /// Exact untargeted entrant's current or latest battlefield copiable values.
    Copiable(CopyRecipe),
    CopiableGroup(Vec<EntryBatchCopyView>),
    CreatureComparison(CreatureComparisonFacts),
    /// Current or last-known signed power of a creature chosen for a cost.
    ChosenCostCreaturePower(i128),
}
#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct EventObjectView {
    pub object: ObjectRef,
    pub facts: EventObjectFacts,
    /// Exact objects semantically named by the event/condition/effect.  Sparse
    /// and cold; never inferred from incidental frozen implementation fields.
    #[serde(skip_serializing_if = "Option::is_none")]
    #[allow(
        clippy::box_collection,
        reason = "Cold semantic groups retain one pointer in the existing event packet"
    )]
    pub semantic_objects: Option<Box<Vec<DamageSourceRef>>>,
    /// Only saved on the actual departure of an untargeted conniver.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub conniver_creature: Option<bool>,
    /// Opt-in last-known characteristics for a later subtype condition.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub subtype_lki: Option<DamageFacts>,
}

#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
impl EventObjectView {
    pub(super) fn owned_bytes(&self) -> usize {
        std::mem::size_of::<Self>()
            + self.semantic_objects.as_ref().map_or(0, |rows| {
                std::mem::size_of::<Vec<DamageSourceRef>>()
                    + rows.len() * std::mem::size_of::<DamageSourceRef>()
            })
            + match &self.facts {
                EventObjectFacts::DeathCounters(counters) => counters.owned_bytes(),
                EventObjectFacts::CopiableGroup(rows) => {
                    rows.len() * std::mem::size_of::<EntryBatchCopyView>()
                }
                _ => 0,
            }
    }
}
#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
impl SpellView {
    pub(super) fn semantic_stack_reference(&self) -> Option<DamageSourceRef> {
        if self.kind == StackKind::Spell {
            self.source
                .map(|source| DamageSourceRef::Object(source.object))
        } else if let StackKind::ActivatedAbility { ability } = self.kind
            && self
                .source
                .is_some_and(|source| source.card.activation(ability).mana_production().is_some())
        {
            None
        } else {
            Some(DamageSourceRef::StackObject {
                stack_object: self.id,
            })
        }
    }
    /// A newly installed delayed clause is its own ability. Its source remains
    /// exact, but the initiating packet's unrelated event/target data is not a
    /// reference of that new clause. Callers rebind explicitly named subjects.
    pub(super) fn clear_initiating_event_references(&mut self) {
        self.event_object = None;
        self.entering_object = None;
        self.source_move = None;
        self.reflexive_subject = None;
        self.damage_event = None;
        self.damage_received = None;
        self.creature_combat_event = None;
        self.cast_event = None;
        self.cast_creature_subjects = None;
        self.copy_event = None;
        self.manifest_event = None;
        self.crew = None;
    }
    pub(super) fn set_reflexive_action_objects(
        &mut self,
        objects: impl IntoIterator<Item = ObjectRef>,
    ) {
        // The parent still refers to its original event until it finishes.
        if let Some(previous) = self.reflexive_subject.take() {
            self.refer_objects(std::iter::once(previous.object).chain(previous.additional_objects));
        }
        let mut objects = objects.into_iter();
        self.reflexive_subject = objects.next().map(|object| {
            Box::new(ReflexiveSubjectView {
                object,
                action: true,
                additional_objects: objects.collect(),
            })
        });
    }
    pub(super) fn reflexive_action_objects(&self) -> impl Iterator<Item = ObjectRef> + '_ {
        self.reflexive_subject
            .iter()
            .filter(|row| row.action)
            .flat_map(|row| {
                std::iter::once(row.object).chain(row.additional_objects.iter().copied())
            })
    }
    pub(super) fn refer_zone_event(&mut self, event: &ZoneChange) {
        // This helper captures a departure condition. Ordinary ETB dispatch
        // separately captures only its new entrant, not its previous object.
        if matches!(
            event.before.zone,
            Zone::Battlefield | Zone::Stack | Zone::Graveyard | Zone::Exile | Zone::Command
        ) {
            self.refer_objects([event.before.reference]);
        }
        if matches!(
            event.destination,
            Zone::Battlefield | Zone::Stack | Zone::Graveyard | Zone::Exile | Zone::Command
        ) {
            self.refer_objects(event.objects_after());
        }
    }
    pub(super) fn refer_objects(&mut self, objects: impl IntoIterator<Item = ObjectRef>) {
        self.refer_sources(objects.into_iter().map(DamageSourceRef::Object));
    }
    pub(super) fn refer_sources(&mut self, objects: impl IntoIterator<Item = DamageSourceRef>) {
        for object in objects {
            if self.event_object.is_none() {
                let primary = match object {
                    DamageSourceRef::Object(object) => object,
                    DamageSourceRef::StackObject { .. } => {
                        self.source.expect("object-backed event").object
                    }
                };
                self.event_object = Some(Box::new(EventObjectView {
                    object: primary,
                    facts: EventObjectFacts::Identity,
                    semantic_objects: None,
                    conniver_creature: None,
                    subtype_lki: None,
                }));
            }
            let event = self.event_object.as_mut().unwrap();
            if object == DamageSourceRef::Object(event.object) {
                continue;
            }
            let rows = event.semantic_objects.get_or_insert_with(Default::default);
            if !rows.contains(&object) {
                rows.push(object);
            }
        }
    }
    pub(super) fn set_event_object(&mut self, event: EventObjectView) {
        if let Some(previous) = self.event_object.take() {
            let primary = previous.object;
            let objects = previous.semantic_objects;
            self.event_object = Some(Box::new(event));
            self.refer_objects([primary]);
            self.refer_sources(objects.into_iter().flat_map(|rows| *rows));
        } else {
            self.event_object = Some(Box::new(event));
        }
    }
}

#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
fn actor(view: &ObjectView) -> DamageSource {
    DamageSource {
        facts: DamageFacts::from_view(view),
        object: view.id,
        reference: view.reference,
        controller: view.controller,
        keywords: view.keywords,
    }
}
#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
impl Game {
    fn event_fact_packets(&self) -> impl Iterator<Item = &SpellView> {
        self.state
            .stack
            .iter()
            .chain(&self.state.pending_triggers)
            .chain(&self.state.ordering_triggers)
            .chain(self.state.resolving.iter().map(|r| &r.item))
            .chain(self.state.announcing.iter().map(|a| &a.item))
            .chain(self.state.activation_payment.iter().map(|p| &p.item))
            .chain(
                self.state
                    .mana_announcement_stack
                    .iter()
                    .flat_map(|s| s.iter())
                    .map(|f| &f.announcement.item),
            )
    }
    /// All callers pass intact pre-move views before committing any moves.
    pub(super) fn capture_event_object_facts<'a>(
        &mut self,
        before: impl IntoIterator<Item = &'a ObjectView>,
    ) {
        let ordinary = self.event_fact_packets().any(|p| {
            p.event_object.is_some()
                || p.source_quantity_lki
                    .as_ref()
                    .is_some_and(|q| q.creature_comparison.is_some())
        });
        let cold = self.state.stack_copy_data.is_some()
            || self.state.replacements.is_some()
            || self
                .state
                .entry_batch
                .as_ref()
                .is_some_and(|batch| batch.replacement_resume.is_some())
            || self.state.delayed_returns.is_some();
        if !ordinary && !cold {
            return;
        }
        let before = before.into_iter().collect::<Vec<_>>();
        self.capture_entry_batch_copy_departures(&before);
        if !ordinary {
            return;
        }
        // An Aura's ETB reads its live attachment or the attachment in its
        // departure LKI. Rebind when a new attachment or the Aura leaves so
        // subsequent moves can refresh the appropriate creature's LKI.
        let attachments: Vec<_> = self
            .event_fact_packets()
            .filter_map(|packet| {
                let source = packet.source?;
                let attached = self.state.objects[source.id as usize].attached_to;
                if attached == packet.source_attachment
                    || !self.on_battlefield(source.id)
                    || self.object_ref(source.id) != source.object
                    || !before.iter().any(|view| {
                        view.reference == source.object || Some(view.reference) == attached
                    })
                    || !packet.program().is_some_and(|(_, program)| {
                        program.contains(&Instruction::AttachedCreaturePowerDamage)
                    })
                {
                    return None;
                }
                let actor = attached
                    .filter(|r| self.on_battlefield(r.card) && self.object_ref(r.card) == *r)
                    .map(|r| self.damage_source(r.card));
                Some((packet.id, attached, actor))
            })
            .collect();
        let facts: Vec<_> = before
            .into_iter()
            .filter(|v| matches!(v.zone, Zone::Battlefield | Zone::Stack | Zone::Exile))
            .filter(|v| {
                self.event_fact_packets().any(|p| {
                    p.event_object.as_ref().is_some_and(|e| {
                        e.object == v.reference
                            && (v.zone != Zone::Exile
                                || matches!(e.facts, EventObjectFacts::ChosenCostCreaturePower(_)))
                    }) || (v.zone != Zone::Exile
                        && p.source.is_some_and(|source| source.object == v.reference)
                        && p.source_quantity_lki
                            .as_ref()
                            .is_some_and(|q| q.creature_comparison.is_some()))
                })
            })
            .map(|v| {
                (
                    v.reference,
                    CreatureComparisonFacts::from_view(v),
                    actor(v),
                    v.copy_recipe.unwrap_or_else(|| CopyRecipe::new(v.card)),
                    v.zone,
                    &v.counters,
                )
            })
            .collect();
        if facts.is_empty() && attachments.is_empty() {
            return;
        }
        for p in self
            .state
            .stack
            .iter_mut()
            .chain(&mut self.state.pending_triggers)
            .chain(&mut self.state.ordering_triggers)
            .chain(self.state.resolving.iter_mut().map(|r| &mut r.item))
            .chain(self.state.announcing.iter_mut().map(|a| &mut a.item))
            .chain(
                self.state
                    .activation_payment
                    .iter_mut()
                    .map(|p| &mut p.item),
            )
            .chain(
                self.state
                    .mana_announcement_stack
                    .iter_mut()
                    .flat_map(|s| s.iter_mut())
                    .map(|f| &mut f.announcement.item),
            )
        {
            if let Some(&(_, attached, damage)) = attachments.iter().find(|row| row.0 == p.id) {
                p.source_attachment = attached;
                if let (Some(recipient), Some(damage)) = (attached, damage) {
                    p.set_event_object(EventObjectView {
                        object: recipient,
                        facts: EventObjectFacts::DamageActor(damage),
                        semantic_objects: None,
                        conniver_creature: None,
                        subtype_lki: None,
                    });
                }
            }
            if let Some(q) = &mut p.source_quantity_lki
                && q.creature_comparison.is_some()
                && let Some((_, stats, _, _, _, _)) = facts.iter().find(|(r, _, _, _, zone, _)| {
                    *zone != Zone::Exile && p.source.is_some_and(|source| *r == source.object)
                })
            {
                q.creature_comparison = Some(*stats);
            }
            if let Some(e) = &mut p.event_object
                && let Some((_, stats, damage, recipe, zone, counters)) =
                    facts.iter().find(|(r, _, _, _, _, _)| *r == e.object)
            {
                if matches!(e.facts, EventObjectFacts::ChosenCostCreaturePower(_)) {
                    e.facts = EventObjectFacts::ChosenCostCreaturePower(if stats.creature {
                        stats.power
                    } else {
                        0
                    });
                    continue;
                }
                if *zone == Zone::Exile {
                    continue;
                }
                if e.subtype_lki.is_some() {
                    e.subtype_lki = Some(damage.facts);
                }
                e.facts = match e.facts {
                    EventObjectFacts::DeathCounters(_)
                    | EventObjectFacts::CopiableGroup(_)
                    | EventObjectFacts::DamageDealt { .. }
                    | EventObjectFacts::DepartureManaValue(_)
                    | EventObjectFacts::CountersPlaced { .. } => {
                        continue;
                    }
                    EventObjectFacts::Identity => EventObjectFacts::Identity,
                    EventObjectFacts::CounterTotal(_) => {
                        EventObjectFacts::CounterTotal(counters.total())
                    }
                    EventObjectFacts::DamageActor(_) => EventObjectFacts::DamageActor(*damage),
                    EventObjectFacts::Copiable(_) => EventObjectFacts::Copiable(*recipe),
                    EventObjectFacts::Controller(_) => {
                        e.conniver_creature = Some(stats.creature);
                        EventObjectFacts::Controller(damage.controller)
                    }
                    EventObjectFacts::CreatureComparison(_) => {
                        EventObjectFacts::CreatureComparison(*stats)
                    }
                    EventObjectFacts::ChosenCostCreaturePower(_) => unreachable!(),
                };
            }
        }
    }
    /// Programs that connive or endure the exact event object own controller facts.
    /// Other packets retain their existing payload and departure work.
    pub(super) fn bind_entering_conniver(
        &mut self,
        entering: &ObjectView,
        program: &[Instruction],
    ) {
        if program
            .iter()
            .any(|instruction| matches!(instruction, Instruction::EnteringPowerDamagePlayers(_)))
        {
            self.state
                .pending_triggers
                .last_mut()
                .unwrap()
                .set_event_object(EventObjectView {
                    object: entering.reference,
                    facts: EventObjectFacts::DamageActor(actor(entering)),
                    semantic_objects: None,
                    conniver_creature: None,
                    subtype_lki: None,
                });
            return;
        }
        if program.iter().any(|instruction| {
            matches!(
                instruction,
                Instruction::Connive {
                    subject: Subject::Entering,
                    ..
                } | Instruction::EndureScaled {
                    recipient: EndureRecipient::Entering,
                    ..
                }
            )
        }) {
            self.state
                .pending_triggers
                .last_mut()
                .unwrap()
                .set_event_object(EventObjectView {
                    object: entering.reference,
                    facts: EventObjectFacts::Controller(entering.controller),
                    semantic_objects: None,
                    conniver_creature: None,
                    subtype_lki: None,
                });
        }
    }
    /// Opt in to subtype LKI only for programs that examine their event object.
    pub(super) fn bind_entering_subtype_lki(
        &mut self,
        entering: &ObjectView,
        program: &[Instruction],
    ) {
        if !program
            .iter()
            .any(|instruction| matches!(instruction, Instruction::IfEventObjectSubtype { .. }))
        {
            return;
        }
        let packet = self.state.pending_triggers.last_mut().unwrap();
        let facts = actor(entering).facts;
        if let Some(event) = &mut packet.event_object {
            assert_eq!(
                event.object, entering.reference,
                "event object identity mismatch"
            );
            event.subtype_lki = Some(facts);
        } else {
            packet.set_event_object(EventObjectView {
                object: entering.reference,
                facts: EventObjectFacts::DamageActor(actor(entering)),
                semantic_objects: None,
                conniver_creature: None,
                subtype_lki: Some(facts),
            });
        }
    }
    pub(super) fn event_object_has_subtype(&self, item: &SpellView, subtype: &str) -> bool {
        let Some(event) = &item.event_object else {
            return false;
        };
        if self.on_battlefield(event.object.card)
            && self.object_ref(event.object.card) == event.object
        {
            return self.has_subtype(event.object.card, subtype);
        }
        event
            .subtype_lki
            .is_some_and(|facts| facts.has_subtype(subtype))
    }
    pub(super) fn bind_creature_comparison(&mut self, source: &ObjectView, entering: &ObjectView) {
        let p = self.state.pending_triggers.last_mut().unwrap();
        p.source_quantity_lki = Some(Box::new(SourceQuantityView {
            chosen_creature_type: None,
            power: None,
            toughness: None,
            equipment_count: None,
            attached_equipment: None,
            creature_comparison: Some(CreatureComparisonFacts::from_view(source)),
        }));
        p.set_event_object(EventObjectView {
            object: entering.reference,
            facts: EventObjectFacts::CreatureComparison(CreatureComparisonFacts::from_view(
                entering,
            )),
            semantic_objects: None,
            conniver_creature: None,
            subtype_lki: None,
        });
    }
    pub(super) fn comparison_facts(
        &self,
        reference: ObjectRef,
        saved: CreatureComparisonFacts,
    ) -> CreatureComparisonFacts {
        if self.on_battlefield(reference.card) && self.object_ref(reference.card) == reference {
            let id = reference.card;
            let world = self.characteristic_query();
            let stats = world.stats(id);
            CreatureComparisonFacts {
                creature: ability_world::CharacteristicWorld::kinds(&world, id) & 1 != 0,
                power: stats.power,
                toughness: stats.toughness,
                controller: self.state.objects[id as usize].controller,
            }
        } else {
            saved
        }
    }
    pub(super) fn chosen_cost_creature_power(&self, item: &SpellView) -> i128 {
        let Some(event) = item.event_object.as_ref() else {
            return 0;
        };
        let EventObjectFacts::ChosenCostCreaturePower(saved) = event.facts else {
            return 0;
        };
        if self.object_ref(event.object.card) != event.object {
            return saved;
        }
        let view = self.object_view(event.object.card);
        if view.is_creature() { view.power } else { 0 }
    }
    pub(super) fn event_object_guard(&self, p: &SpellView) -> bool {
        let StackKind::TriggeredAbility { ability } = p.kind else {
            return true;
        };
        if p.trigger_definition().triggers()[usize::from(ability)]
            .condition
            .event()
            != TriggerCondition::OtherCreatureEntersYouIfGreaterStats
        {
            return true;
        }
        let e = p.event_object.as_ref().expect("creature comparison event");
        let EventObjectFacts::CreatureComparison(entrant) = e.facts else {
            unreachable!("comparison facts");
        };
        let source = p
            .source_quantity_lki
            .as_ref()
            .and_then(|q| q.creature_comparison)
            .expect("source comparison facts");
        let Some(identity) = p.source else {
            return false;
        };
        self.comparison_facts(e.object, entrant)
            .greater_than(self.comparison_facts(identity.object, source))
    }
    pub(super) fn attached_tap_triggers(&mut self, changed: &[CardId]) {
        if changed.is_empty() || self.state.tap_sources.is_empty() {
            return;
        }
        let sources = self.state.tap_sources.clone();
        for id in sources {
            if !self.state.objects[id as usize]
                .card
                .triggers()
                .iter()
                .any(|a| a.condition.event() == TriggerCondition::AttachedCreatureBecomesTapped)
            {
                continue;
            }
            let Some(recipient) = self.attachment_recipient(id) else {
                continue;
            };
            if !changed.contains(&recipient.card) || !self.is_creature(recipient.card) {
                continue;
            }
            let source = self.object_view(id);
            for (index, ability) in source.card.triggers().iter().enumerate() {
                if ability.condition.event() == TriggerCondition::AttachedCreatureBecomesTapped
                    && self.queue_trigger(&source, index)
                {
                    let facts = EventObjectFacts::DamageActor(self.damage_source(recipient.card));
                    self.state
                        .pending_triggers
                        .last_mut()
                        .unwrap()
                        .set_event_object(EventObjectView {
                            object: recipient,
                            facts,
                            semantic_objects: None,
                            conniver_creature: None,
                            subtype_lki: None,
                        });
                }
            }
        }
    }
    pub(super) fn bind_trigger_damage_actor(&mut self, object: ObjectRef) {
        let damage = self.damage_source(object.card);
        self.state
            .pending_triggers
            .last_mut()
            .unwrap()
            .set_event_object(EventObjectView {
                object,
                facts: EventObjectFacts::DamageActor(damage),
                semantic_objects: None,
                conniver_creature: None,
                subtype_lki: None,
            });
    }

    pub(super) fn attached_creature_power_damage(&mut self, item: &SpellView) {
        let Some(aura) = item.source else {
            return;
        };
        let attached = if self.on_battlefield(aura.id) && self.object_ref(aura.id) == aura.object {
            self.state.objects[aura.id as usize].attached_to
        } else {
            item.source_attachment
        };
        let Some(attached) = attached else {
            return;
        };
        if self.on_battlefield(attached.card) && self.object_ref(attached.card) == attached {
            self.damage_targets_by_actor_power(item, self.damage_source(attached.card));
        } else if item
            .event_object
            .as_ref()
            .is_some_and(|event| event.object == attached)
        {
            self.event_object_power_damage(item);
        }
    }

    pub(super) fn event_object_power_damage(&mut self, item: &SpellView) {
        let Some(e) = &item.event_object else {
            return;
        };
        let EventObjectFacts::DamageActor(saved) = e.facts else {
            return;
        };
        let source =
            if self.on_battlefield(e.object.card) && self.object_ref(e.object.card) == e.object {
                self.damage_source(e.object.card)
            } else {
                saved
            };
        self.damage_targets_by_actor_power(item, source);
    }

    fn damage_targets_by_actor_power(&mut self, item: &SpellView, source: DamageSource) {
        // CR208.3: a current noncreature permanent has no power; do not read
        // printed Vehicle stats or reuse its previous creature characteristics.
        let amount = if source.facts.kinds & 1 != 0 {
            source.facts.power.max(0)
        } else {
            0
        };
        let events = item
            .targets
            .iter()
            .filter(|b| self.same_target_object(**b))
            .map(|b| DamageEvent {
                source,
                target: b.target,
                amount,
                combat: false,
                preventable: true,
            })
            .collect::<Vec<_>>();
        self.apply_damage_batch(&events);
    }
    pub(super) fn event_object_power_damage_players(
        &mut self,
        item: &SpellView,
        players: PlayerSet,
    ) {
        let Some(e) = &item.event_object else {
            return;
        };
        let EventObjectFacts::DamageActor(saved) = e.facts else {
            return;
        };
        let source =
            if self.on_battlefield(e.object.card) && self.object_ref(e.object.card) == e.object {
                self.damage_source(e.object.card)
            } else {
                saved
            };
        let amount = if source.facts.is_creature() {
            source.facts.power.max(0)
        } else {
            0
        };
        let events = self
            .program_players(players, item)
            .map(|player| DamageEvent {
                source,
                target: Target::Player(player),
                amount,
                combat: false,
                preventable: true,
            })
            .collect::<Vec<_>>();
        self.apply_damage_batch(&events);
    }
    pub(super) fn event_object_damage_players(
        &mut self,
        p: &SpellView,
        players: PlayerSet,
        amount: i128,
    ) {
        let e = p
            .event_object
            .as_ref()
            .expect("independent event damage actor");
        let EventObjectFacts::DamageActor(saved) = e.facts else {
            unreachable!("damage actor facts");
        };
        let source =
            if self.on_battlefield(e.object.card) && self.object_ref(e.object.card) == e.object {
                self.damage_source(e.object.card)
            } else {
                saved
            };
        let events: Vec<_> = self
            .program_players(players, p)
            .map(|player| DamageEvent {
                source,
                target: Target::Player(player),
                amount,
                combat: false,
                preventable: true,
            })
            .collect();
        self.apply_damage_batch(&events);
    }
    pub(super) fn event_object_controller(&self, item: &SpellView) -> Option<Player> {
        let event = item.event_object.as_ref()?;
        let saved = match event.facts {
            EventObjectFacts::Controller(controller) => controller,
            EventObjectFacts::DamageActor(source) => source.controller,
            _ => return None,
        };
        let current = self
            .state
            .objects
            .get(event.object.card as usize)
            .filter(|object| {
                matches!(object.zone, Zone::Battlefield | Zone::Stack)
                    && object.generation == event.object.generation
            });
        Some(current.map_or(saved, |object| object.controller))
    }
    pub(super) fn damage_event_object_controller(&mut self, item: &SpellView, amount: i32) {
        let controller = self
            .event_object_controller(item)
            .expect("blocker controller facts");
        self.apply_damage_batch(&[DamageEvent {
            source: self.program_damage_source(item),
            target: Target::Player(controller),
            amount: i128::from(amount),
            combat: false,
            preventable: true,
        }]);
    }
}
