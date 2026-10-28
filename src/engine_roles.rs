//! Snapshot-owned Role timestamps and predefined attachment creation (CR303.7/704.5).
use super::*;

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct RoleTimestampView {
    pub aura: ObjectRef,
    pub recipient: ObjectRef,
    pub timestamp: u64,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct RoleGroupView {
    pub controller: Player,
    pub recipient: ObjectRef,
    pub objects: Vec<ObjectRef>,
    pub keep: Option<ObjectRef>,
}
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub(super) struct RoleState {
    clock: u64,
    /// Only true while an entry transaction assigns one simultaneous timestamp group.
    pub deferred_timestamps: bool,
    /// Rule effect on players, independent of a fixed creature set or its source.
    pub toughness_combat_damage: [bool; 2],
    pub attached: Vec<RoleTimestampView>,
    pub base_stats: Vec<BaseStatsEffectView>,
    pub live_static: Vec<LiveStaticSourceView>,
    pub stat_subjects: Vec<ResolvedStatSubjectView>,
    pub ability_timestamps: Vec<AbilitySourceTimestampView>,
    pub creature_types: Vec<subtypes::CreatureTypesEffectView>,
    pub animations: Vec<AnimationView>,
    pub keyword_effects: Vec<KeywordEffectView>,
    pub source_linked_effects: Vec<SourceLinkedEffectView>,
    pub resolved_ability_modifications: Vec<ResolvedAbilityModificationView>,
    pub ability_removal_sources: Vec<ObjectRef>,
    pub characteristic_sources: Vec<ObjectRef>,
    pub harnessed: Vec<ObjectRef>,
}
impl RoleState {
    pub(super) fn next_timestamp(&mut self) -> u64 {
        if self.deferred_timestamps {
            return 0;
        }
        self.clock = self
            .clock
            .checked_add(1)
            .capacity("Role timestamps exhausted");
        self.clock
    }
    pub fn bytes(&self) -> usize {
        std::mem::size_of::<Self>()
            + self.attached.len() * std::mem::size_of::<RoleTimestampView>()
            + self.base_stats.len() * std::mem::size_of::<BaseStatsEffectView>()
            + self.live_static.len() * std::mem::size_of::<LiveStaticSourceView>()
            + self.stat_subjects.len() * std::mem::size_of::<ResolvedStatSubjectView>()
            + self.ability_timestamps.len() * std::mem::size_of::<AbilitySourceTimestampView>()
            + self.creature_types.len() * std::mem::size_of::<subtypes::CreatureTypesEffectView>()
            + self.animations.len() * std::mem::size_of::<AnimationView>()
            + self.keyword_effects.len() * std::mem::size_of::<KeywordEffectView>()
            + self.source_linked_effects.len() * std::mem::size_of::<SourceLinkedEffectView>()
            + self.resolved_ability_modifications.len()
                * std::mem::size_of::<ResolvedAbilityModificationView>()
            + self.ability_removal_sources.len() * std::mem::size_of::<ObjectRef>()
            + self.characteristic_sources.len() * std::mem::size_of::<ObjectRef>()
            + self.harnessed.len() * std::mem::size_of::<ObjectRef>()
    }
}
impl Game {
    pub(super) fn queue_reflexive(
        &mut self,
        item: &SpellView,
        ability: u16,
        subject: Option<ObjectRef>,
    ) {
        let condition = item.trigger_definition().triggers()[usize::from(ability)].condition;
        if matches!(condition, TriggerCondition::InterveningIf { .. })
            && !self.trigger_guards_hold(
                condition,
                item.controller,
                item.object,
                item.source_tapped,
            )
        {
            return;
        }
        let mut reflexive = item.clone();
        reflexive.id = self.next_stack_id();
        reflexive.kind = StackKind::TriggeredAbility { ability };
        reflexive.is_copy = false;
        reflexive.targets.clear();
        reflexive.mode = None;
        if let Some((_, program)) = reflexive.program()
            && let Some(facts) = card_quantities::counter_quantity_facts(program, |kind| {
                self.source_counter_quantity_live(reflexive.object, kind)
                    .unwrap_or_else(|| {
                        item.source_counter_lki
                            .as_ref()
                            .and_then(|rows| rows.iter().find(|row| row.kind == kind))
                            .map_or(0, |row| row.count)
                    })
            })
        {
            reflexive.source_counter_lki = Some(facts);
        }
        if let TriggerCondition::InterveningIf {
            guard:
                StepGuard::Expression {
                    value: StatValue::SourceCounters(kind),
                    ..
                },
            ..
        } = condition
        {
            let count = self
                .source_counter_quantity_live(reflexive.object, kind)
                .unwrap_or(0);
            let facts = reflexive
                .source_counter_lki
                .get_or_insert_with(Default::default);
            if let Some(fact) = facts.iter_mut().find(|fact| fact.kind == kind) {
                fact.count = count;
            } else {
                facts.push(card_quantities::SourceCounterQuantityView { kind, count });
            }
        }
        reflexive.reflexive_subject = subject.map(|object| {
            Box::new(ReflexiveSubjectView {
                object,
                additional_objects: Vec::new(),
            })
        });
        if reflexive
            .program()
            .is_some_and(|(_, program)| program.contains(&Instruction::EventObjectPowerDamage))
            && let Some(object) = subject
        {
            reflexive.event_object = Some(Box::new(EventObjectView {
                object,
                facts: EventObjectFacts::DamageActor(self.damage_source(object.card)),
                conniver_creature: None,
                subtype_lki: None,
            }));
        }
        self.state.pending_triggers.push(reflexive);
    }
    pub(super) fn exact_source_can_tap(&self, item: &SpellView) -> bool {
        self.on_battlefield(item.card)
            && self.object_ref(item.card) == item.object
            && !self.state.objects[item.card as usize].tapped
    }
    pub(super) fn complete_optional_tap_reflexive(&mut self) {
        let resolution = self.state.resolving.as_ref().unwrap();
        let Some(Instruction::MayTapSourceReflexive { ability }) =
            resolution.program.get(resolution.pc.saturating_sub(1))
        else {
            return;
        };
        let (item, ability) = (resolution.item.clone(), *ability);
        if self.exact_source_can_tap(&item) {
            self.tap_objects(&[item.card]);
            if self.on_battlefield(item.card)
                && self.object_ref(item.card) == item.object
                && self.state.objects[item.card as usize].tapped
            {
                self.queue_reflexive(&item, ability, None);
            }
        }
    }
    pub(super) fn resolve_role_reflexive(&mut self, item: &SpellView, token: Card, ability: u16) {
        assert!(
            token.role().is_some(),
            "Role-success instruction requires a Role factory"
        );
        for bound in &item.targets {
            if self.same_target_object(*bound)
                && let Target::Card(id) = bound.target
            {
                let recipient = self.object_ref(id);
                if self.aura_can_enchant(token, item.controller, recipient) {
                    self.stage_token_creation(
                        item.controller,
                        token,
                        1,
                        false,
                        replacements::TokenCompletion::RoleReflexive(Box::new(
                            replacements::RoleReflexiveCompletion {
                                recipient,
                                token,
                                item: item.clone(),
                                ability,
                            },
                        )),
                    );
                }
            }
        }
    }
    pub(super) fn resolve_roles_matching(
        &mut self,
        item: &SpellView,
        token: Card,
        filter: CreatureFilter,
    ) {
        assert!(token.role().is_some(), "predefined Role factory required");
        let recipients = self
            .state
            .objects
            .iter()
            .enumerate()
            .filter_map(|(id, _)| {
                let id = id as CardId;
                let recipient = self.object_ref(id);
                (self.creature_matches(filter, item.controller, item.object, id)
                    && self.aura_can_enchant(token, item.controller, recipient))
                .then_some(recipient)
            })
            .collect();
        self.stage_role_creation(item.controller, token, recipients);
    }

    pub(super) fn next_role_timestamp(&mut self) -> u64 {
        let r = self.state.roles.get_or_insert_with(Default::default);
        r.next_timestamp()
    }
    pub(super) fn static_source_timestamp(&mut self, source: ObjectRef) -> u64 {
        // All continuous abilities of a permanent share its most recent timestamp.
        if let Some(r) = &self.state.roles {
            if let Some(v) = r.ability_timestamps.iter().find(|v| v.source == source) {
                return v.timestamp;
            }
            if let Some(v) = r
                .base_stats
                .iter()
                .find(|v| v.static_source && v.source == source)
            {
                return v.timestamp;
            }
            if let Some(v) = r.live_static.iter().find(|v| v.source == source) {
                return v.timestamp;
            }
        }
        self.next_role_timestamp()
    }
    pub(super) fn assign_static_source_timestamp(&mut self, source: ObjectRef) {
        if self.retimestamp_temporary_copy(source) {
            return;
        }
        let timestamp = self.next_role_timestamp();
        if let Some(r) = &mut self.state.roles {
            for row in &mut r.keyword_effects {
                if row.recipient == source
                    && row.timestamp == 0
                    && matches!(row.origin, KeywordEffectOrigin::Counter { .. })
                {
                    row.timestamp = timestamp;
                }
            }
            // Return modifications are resolution-created effects on the
            // entering incarnation, not abilities of their historical source.
            // Only deferred rows acquire the entry timestamp: later provider
            // retimestamps must preserve an ordinary resolution grant's time.
            for row in &mut r.creature_types {
                if row.recipient == source && row.timestamp == 0 {
                    row.timestamp = timestamp;
                }
            }
            for v in &mut r.ability_timestamps {
                if v.source == source {
                    v.timestamp = timestamp;
                }
            }
            for v in &mut r.base_stats {
                if v.static_source && v.source == source {
                    v.timestamp = timestamp;
                }
            }
            for v in &mut r.live_static {
                if v.source == source {
                    v.timestamp = timestamp;
                }
            }
            for v in &mut r.attached {
                if v.aura == source {
                    v.timestamp = timestamp;
                }
            }
        }
        self.retimestamp_self_granted_statics(source);
    }
    pub(super) fn record_role(&mut self, aura: ObjectRef, recipient: ObjectRef, timestamp: u64) {
        let r = self.state.roles.get_or_insert_with(Default::default);
        for v in &mut r.ability_timestamps {
            if v.source == aura {
                v.timestamp = timestamp;
            }
        }
        r.attached.retain(|v| v.aura != aura);
        r.attached.push(RoleTimestampView {
            aura,
            recipient,
            timestamp,
        });
    }
    pub(super) fn forget_role(&mut self, aura: ObjectRef) {
        if let Some(r) = &mut self.state.roles {
            r.attached.retain(|v| v.aura != aura);
        }
    }

    pub(super) fn resolve_role(&mut self, item: &SpellView, token: Card, subject: Subject) {
        assert!(
            token.role().is_some(),
            "Role instruction requires a predefined Role factory"
        );
        let recipients: Vec<_> = match subject {
            Subject::Source => (self.on_battlefield(item.card)
                && self.object_ref(item.card) == item.object)
                .then_some(item.object)
                .into_iter()
                .collect(),
            Subject::Attached => self
                .attached_subject(item)
                .map(|id| self.object_ref(id))
                .into_iter()
                .collect(),
            Subject::Entering => self
                .entering_subject(item)
                .map(|id| self.object_ref(id))
                .into_iter()
                .collect(),
            Subject::Targets => item
                .targets
                .iter()
                .filter(|&&b| self.same_target_object(b))
                .filter_map(|b| {
                    if let Target::Card(id) = b.target {
                        Some(self.object_ref(id))
                    } else {
                        None
                    }
                })
                .collect(),
        };
        for recipient in recipients {
            if self.aura_can_enchant(token, item.controller, recipient) {
                self.stage_token_creation(
                    item.controller,
                    token,
                    1,
                    false,
                    replacements::TokenCompletion::RoleAttach(recipient),
                );
            }
        }
    }
    pub(super) fn create_attached_role_groups(
        &mut self,
        controller: Player,
        groups: &[TokenGroupView],
        recipient: ObjectRef,
    ) -> Vec<ObjectRef> {
        self.create_token_groups_attached(controller, groups, None, false, Some(recipient))
    }
    /// All roles participate even if another SBA would simultaneously remove one.
    /// Only malformed/illegal attachments are excluded by the enchantment rule.
    pub(super) fn collect_role_choices(&self, deaths: &mut Vec<CardId>) -> Vec<RoleGroupView> {
        let Some(state) = &self.state.roles else {
            return Vec::new();
        };
        let mut groups: Vec<(Player, ObjectRef, Vec<&RoleTimestampView>)> = Vec::new();
        for v in &state.attached {
            if !self.on_battlefield(v.aura.card) || self.object_ref(v.aura.card) != v.aura {
                continue;
            }
            let aura = &self.state.objects[v.aura.card as usize];
            if aura.attached_to != Some(v.recipient)
                || !self.aura_can_enchant(aura.card, aura.controller, v.recipient)
            {
                continue;
            }
            if let Some(g) = groups
                .iter_mut()
                .find(|g| g.0 == aura.controller && g.1 == v.recipient)
            {
                g.2.push(v);
            } else {
                groups.push((aura.controller, v.recipient, vec![v]));
            }
        }
        let mut choices = Vec::new();
        for (controller, recipient, roles) in groups {
            if roles.len() < 2 {
                continue;
            }
            let newest = roles.iter().map(|v| v.timestamp).max().unwrap();
            deaths.extend(
                roles
                    .iter()
                    .filter(|v| v.timestamp < newest)
                    .map(|v| v.aura.card),
            );
            let objects: Vec<_> = roles
                .iter()
                .filter(|v| v.timestamp == newest)
                .map(|v| v.aura)
                .collect();
            if objects.len() > 1 {
                choices.push(RoleGroupView {
                    controller,
                    recipient,
                    objects,
                    keep: None,
                });
            }
        }
        choices
    }
    pub(super) fn keep_role(&mut self, object: ObjectRef) {
        let batch = self.state.sba_batch.as_mut().unwrap();
        batch
            .roles
            .iter_mut()
            .find(|g| g.keep.is_none() && g.objects.contains(&object))
            .unwrap()
            .keep = Some(object);
        self.advance_sba_choices();
    }
}
