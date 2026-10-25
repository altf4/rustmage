//! Owned untargeted event objects, distinct from the source of their ability.
use super::*;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
pub struct CreatureComparisonFacts {
    pub creature: bool,
    pub power: i128,
    pub toughness: i128,
}
impl CreatureComparisonFacts {
    pub(super) fn from_view(view: &ObjectView) -> Self {
        Self {
            creature: view.is_creature(),
            power: view.power,
            toughness: view.toughness,
        }
    }
    pub(super) fn greater_than(self, other: Self) -> bool {
        self.creature
            && other.creature
            && (self.power > other.power || self.toughness > other.toughness)
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub enum EventObjectFacts {
    DamageActor(DamageSource),
    /// Current or latest battlefield controller of an exact untargeted conniver.
    Controller(Player),
    CreatureComparison(CreatureComparisonFacts),
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct EventObjectView {
    pub object: ObjectRef,
    pub facts: EventObjectFacts,
    /// Only saved on the actual departure of an untargeted conniver.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub conniver_creature: Option<bool>,
}
fn actor(view: &ObjectView) -> DamageSource {
    DamageSource {
        facts: DamageFacts::from_view(view),
        object: view.id,
        reference: view.reference,
        controller: view.controller,
        keywords: view.keywords,
    }
}
impl Game {
    fn event_fact_packets(&self) -> impl Iterator<Item = &SpellView> {
        self.state
            .stack
            .iter()
            .chain(&self.state.pending_triggers)
            .chain(&self.state.ordering_triggers)
            .chain(self.state.resolving.iter().map(|r| &r.item))
            .chain(self.state.announcing.iter().map(|a| &a.item))
            .chain(
                self.state
                    .mana_announcement_stack
                    .iter()
                    .flat_map(|s| s.iter())
                    .map(|f| &f.announcement.item),
            )
    }
    /// All callers pass the intact pre-event battlefield, before ANY commit.
    pub(super) fn capture_event_object_facts<'a>(
        &mut self,
        before: impl IntoIterator<Item = &'a ObjectView>,
    ) {
        if !self.event_fact_packets().any(|p| {
            p.event_object.is_some()
                || p.source_quantity_lki
                    .as_ref()
                    .is_some_and(|q| q.creature_comparison.is_some())
        }) {
            return;
        }
        let facts: Vec<_> = before
            .into_iter()
            .filter(|v| v.zone == Zone::Battlefield)
            .filter(|v| {
                self.event_fact_packets().any(|p| {
                    p.event_object
                        .as_ref()
                        .is_some_and(|e| e.object == v.reference)
                        || (p.object == v.reference
                            && p.source_quantity_lki
                                .as_ref()
                                .is_some_and(|q| q.creature_comparison.is_some()))
                })
            })
            .map(|v| (v.reference, CreatureComparisonFacts::from_view(v), actor(v)))
            .collect();
        if facts.is_empty() {
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
                    .mana_announcement_stack
                    .iter_mut()
                    .flat_map(|s| s.iter_mut())
                    .map(|f| &mut f.announcement.item),
            )
        {
            if let Some(q) = &mut p.source_quantity_lki
                && q.creature_comparison.is_some()
                && let Some((_, stats, _)) = facts.iter().find(|(r, _, _)| *r == p.object)
            {
                q.creature_comparison = Some(*stats);
            }
            if let Some(e) = &mut p.event_object
                && let Some((_, stats, damage)) = facts.iter().find(|(r, _, _)| *r == e.object)
            {
                e.facts = match e.facts {
                    EventObjectFacts::DamageActor(_) => EventObjectFacts::DamageActor(*damage),
                    EventObjectFacts::Controller(_) => {
                        e.conniver_creature = Some(stats.creature);
                        EventObjectFacts::Controller(damage.controller)
                    }
                    EventObjectFacts::CreatureComparison(_) => {
                        EventObjectFacts::CreatureComparison(*stats)
                    }
                };
            }
        }
    }
    /// Only programs that connive the exact event object own controller facts.
    /// Other packets retain their existing payload and departure work.
    pub(super) fn bind_entering_conniver(
        &mut self,
        entering: &ObjectView,
        program: &[Instruction],
    ) {
        if program.iter().any(|instruction| {
            matches!(
                instruction,
                Instruction::Connive {
                    subject: Subject::Entering,
                    ..
                }
            )
        }) {
            self.state.pending_triggers.last_mut().unwrap().event_object =
                Some(Box::new(EventObjectView {
                    object: entering.reference,
                    facts: EventObjectFacts::Controller(entering.controller),
                    conniver_creature: None,
                }));
        }
    }
    pub(super) fn bind_creature_comparison(&mut self, source: &ObjectView, entering: &ObjectView) {
        let p = self.state.pending_triggers.last_mut().unwrap();
        p.source_quantity_lki = Some(Box::new(SourceQuantityView {
            toughness: None,
            equipment_count: None,
            creature_comparison: Some(CreatureComparisonFacts::from_view(source)),
        }));
        p.event_object = Some(Box::new(EventObjectView {
            object: entering.reference,
            facts: EventObjectFacts::CreatureComparison(CreatureComparisonFacts::from_view(
                entering,
            )),
            conniver_creature: None,
        }));
    }
    fn comparison_facts(
        &self,
        reference: ObjectRef,
        saved: CreatureComparisonFacts,
    ) -> CreatureComparisonFacts {
        if self.on_battlefield(reference.card) && self.object_ref(reference.card) == reference {
            CreatureComparisonFacts::from_view(&self.object_view(reference.card))
        } else {
            saved
        }
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
        self.comparison_facts(e.object, entrant)
            .greater_than(self.comparison_facts(p.object, source))
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
                    self.state.pending_triggers.last_mut().unwrap().event_object =
                        Some(Box::new(EventObjectView {
                            object: recipient,
                            facts,
                            conniver_creature: None,
                        }));
                }
            }
        }
    }
    pub(super) fn event_object_damage_players(
        &mut self,
        p: &SpellView,
        players: PlayerSet,
        amount: i32,
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
                amount: i128::from(amount),
                combat: false,
                preventable: true,
            })
            .collect();
        self.apply_damage_batch(&events);
    }
}
