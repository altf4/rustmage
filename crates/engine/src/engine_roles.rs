//! Snapshot-owned Role timestamps and predefined attachment creation (CR303.7/704.5).
use super::*;

/// The closed exile-loop proof may alpha-rename acquisition timestamps only
/// when they have no layer/order consumers and refer to live loop objects.
pub(super) fn same_inert_loop_timestamps(before: &State, after: &State, ids: &[CardId]) -> bool {
    fn key(state: &State, ids: &[CardId]) -> Option<Vec<CardId>> {
        let mut value = state.roles.as_deref().cloned().unwrap_or_default();
        let mut sources: Vec<_> = value
            .ability_timestamps
            .iter()
            .map(|r| r.source.card)
            .collect();
        if value.ability_timestamps.iter().any(|r| {
            !ids.contains(&r.source.card)
                || r.source.generation != state.objects[r.source.card as usize].generation
        }) {
            return None;
        }
        sources.sort_unstable();
        value.clock = 0;
        value.ability_timestamps.clear();
        (value == RoleState::default()).then_some(sources)
    }
    match (key(before, ids), key(after, ids)) {
        (Some(a), Some(b)) => a == b,
        _ => false,
    }
}

#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct RoleTimestampView {
    pub aura: ObjectRef,
    pub recipient: ObjectRef,
    pub timestamp: u64,
}
#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct RoleGroupView {
    pub controller: Player,
    pub recipient: ObjectRef,
    pub objects: Vec<ObjectRef>,
    pub keep: Option<ObjectRef>,
}
#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub(super) struct RoleState {
    clock: u64,
    /// Only true while an entry transaction assigns one simultaneous timestamp group.
    pub deferred_timestamps: bool,
    /// Rule effect on players, independent of a fixed creature set or its source.
    pub toughness_combat_damage: [bool; 2],
    /// Each controller's rule lasts until that controller's next turn begins.
    pub prohibit_opponent_nonhand_casts: [bool; 2],
    pub attached: Vec<RoleTimestampView>,
    pub base_stats: Vec<BaseStatsEffectView>,
    pub until_next_turn_stats: Vec<UntilNextTurnStatsView>,
    pub live_static: Vec<LiveStaticSourceView>,
    pub stat_subjects: Vec<ResolvedStatSubjectView>,
    pub ability_timestamps: Vec<AbilitySourceTimestampView>,
    pub creature_types: Vec<subtypes::CreatureTypesEffectView>,
    pub basic_land_types: Vec<subtypes::BasicLandTypesEffectView>,
    pub animations: Vec<AnimationView>,
    pub keyword_effects: Vec<KeywordEffectView>,
    pub source_linked_effects: Vec<SourceLinkedEffectView>,
    /// Exact paid exile objects cast while their grant activation is still pending.
    pub exile_grant_cast_history: Vec<ObjectRef>,
    pub resolved_ability_modifications: Vec<ResolvedAbilityModificationView>,
    pub ability_removal_sources: Vec<ObjectRef>,
    pub characteristic_sources: Vec<ObjectRef>,
    pub control_change_sources: Option<SharedVec<CardId>>,
    pub melded_permanents: Vec<meld::MeldedPermanent>,
    pub meld_departure_orders: Vec<MeldDepartureOrder>,
    pub harnessed: Vec<ObjectRef>,
    pub source_prevention: Option<Box<source_prevention::SourcePreventionState>>,
    pub player_damage_shields: Vec<return_links::PlayerDamageShieldView>,
    pub creature_damage_shields: Vec<CreatureDamageShieldView>,
    pub damage_doublers: Vec<DamageDoublerView>,
    pub chosen_creature_links: Vec<ChosenCreatureLinkView>,
}
#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
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
    /// Ability acquisitions need distinct origins even during deferred entry timestamps.
    pub(super) fn next_acquisition(&mut self) -> u64 {
        self.clock = self
            .clock
            .checked_add(1)
            .capacity("Ability origins exhausted");
        self.clock
    }
    pub fn bytes(&self) -> usize {
        std::mem::size_of::<Self>()
            + self.attached.len() * std::mem::size_of::<RoleTimestampView>()
            + self.base_stats.len() * std::mem::size_of::<BaseStatsEffectView>()
            + self.until_next_turn_stats.len() * std::mem::size_of::<UntilNextTurnStatsView>()
            + self.live_static.len() * std::mem::size_of::<LiveStaticSourceView>()
            + self.stat_subjects.len() * std::mem::size_of::<ResolvedStatSubjectView>()
            + self.ability_timestamps.len() * std::mem::size_of::<AbilitySourceTimestampView>()
            + self.creature_types.len() * std::mem::size_of::<subtypes::CreatureTypesEffectView>()
            + self.basic_land_types.len()
                * std::mem::size_of::<subtypes::BasicLandTypesEffectView>()
            + self.animations.len() * std::mem::size_of::<AnimationView>()
            + self.keyword_effects.len() * std::mem::size_of::<KeywordEffectView>()
            + self.source_linked_effects.len() * std::mem::size_of::<SourceLinkedEffectView>()
            + self.player_damage_shields.len()
                * std::mem::size_of::<return_links::PlayerDamageShieldView>()
            + self.creature_damage_shields.len() * std::mem::size_of::<CreatureDamageShieldView>()
            + self.damage_doublers.len() * std::mem::size_of::<DamageDoublerView>()
            + self.chosen_creature_links.len() * std::mem::size_of::<ChosenCreatureLinkView>()
            + self.exile_grant_cast_history.len() * std::mem::size_of::<ObjectRef>()
            + self.resolved_ability_modifications.len()
                * std::mem::size_of::<ResolvedAbilityModificationView>()
            + self.ability_removal_sources.len() * std::mem::size_of::<ObjectRef>()
            + self.characteristic_sources.len() * std::mem::size_of::<ObjectRef>()
            + self.control_change_sources.as_ref().map_or(0, |rows| {
                std::mem::size_of::<Vec<CardId>>() + rows.len() * std::mem::size_of::<CardId>()
            })
            + self.melded_permanents.len() * std::mem::size_of::<meld::MeldedPermanent>()
            + self.meld_departure_orders.len() * std::mem::size_of::<MeldDepartureOrder>()
            + self.harnessed.len() * std::mem::size_of::<ObjectRef>()
            + self
                .source_prevention
                .as_ref()
                .map_or(0, |state| state.bytes())
    }
}
#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum ReflexiveAction {
    Sacrifice,
    Discard,
    Mill,
}

#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
impl Game {
    pub(super) fn nonhand_cast_prohibited(&self, player: Player) -> bool {
        self.state
            .roles
            .as_ref()
            .is_some_and(|roles| roles.prohibit_opponent_nonhand_casts[player.other().index()])
    }

    pub(super) fn capture_reflexive_zone_action(
        &mut self,
        events: &[ZoneChange],
        action: ReflexiveAction,
    ) {
        let Some(resolving) = self.state.resolving.as_ref() else {
            return;
        };
        let instruction = resolving.program.get(resolving.pc.saturating_sub(1));
        let follows = resolving.program[resolving.pc..].iter().any(|i| {
            matches!(
                i,
                Instruction::Reflexive { .. } | Instruction::ReflexiveSacrificedPower { .. }
            )
        });
        let matches_action = match (action, instruction) {
            (
                ReflexiveAction::Sacrifice,
                Some(
                    Instruction::SacrificeSourceReflexive { .. }
                    | Instruction::MaySacrificeSourceReflexive { .. },
                ),
            ) => true,
            (ReflexiveAction::Mill, Some(Instruction::MillThenReflexive { .. })) => true,
            (ReflexiveAction::Sacrifice, Some(Instruction::ChooseObjects { kind, .. })) => {
                follows
                    && matches!(
                        kind.payment_kind(),
                        EffectChoiceKind::Sacrifice
                            | EffectChoiceKind::SacrificeGreatestManaValue
                            | EffectChoiceKind::SacrificeOrDiscard { .. }
                    )
            }
            (ReflexiveAction::Discard, Some(Instruction::ChooseObjects { kind, .. })) => {
                follows
                    && matches!(
                        kind.payment_kind(),
                        EffectChoiceKind::Discard | EffectChoiceKind::SacrificeOrDiscard { .. }
                    )
            }
            _ => false,
        };
        if !matches_action {
            return;
        }
        let mut objects = Vec::new();
        for event in events {
            if action != ReflexiveAction::Mill {
                objects.push(event.before.reference);
            }
            if event.moved
                && matches!(
                    event.destination,
                    Zone::Battlefield | Zone::Stack | Zone::Graveyard | Zone::Exile | Zone::Command
                )
            {
                objects.push(event.after);
            }
        }
        let item = &mut self.state.resolving.as_mut().unwrap().item;
        item.set_reflexive_action_objects(objects);
    }
    pub(super) fn queue_reflexive(
        &mut self,
        item: &SpellView,
        ability: u16,
        subject: Option<ObjectRef>,
    ) {
        self.queue_reflexive_objects(item, ability, subject);
    }
    pub(super) fn queue_reflexive_objects(
        &mut self,
        item: &SpellView,
        ability: u16,
        subjects: impl IntoIterator<Item = ObjectRef>,
    ) {
        let mut subjects = subjects.into_iter();
        let subject = subjects.next();
        let Some(source) = item.source else {
            return;
        };
        let condition = item.trigger_definition().triggers()[usize::from(ability)].condition;
        if matches!(condition, TriggerCondition::InterveningIf { .. })
            && !self.trigger_guards_hold(
                condition,
                item.controller,
                source.object,
                item.source_tapped,
            )
        {
            return;
        }
        let mut reflexive = item.clone();
        // A reflexive clause is a new ability. The original event is not its
        // condition; retain scalar/fact inputs but rebind its own action objects.
        reflexive.clear_initiating_event_references();
        reflexive.damage_event = item.damage_event; // amount / damaged-player scalar
        reflexive.return_link = None;
        reflexive.exiled_trigger_cards = None;
        reflexive.exiled_symbol_cards = None;
        reflexive.craft_materials = None;
        reflexive.id = self.next_stack_id();
        reflexive.kind = StackKind::TriggeredAbility { ability };
        reflexive.is_copy = false;
        reflexive.targets.clear();
        reflexive.target_type_slots = None;
        reflexive.mode = None;
        if let Some((_, program)) = reflexive.program()
            && let Some(facts) = card_quantities::counter_quantity_facts(program, |kind| {
                self.source_counter_quantity_live(source.object, kind)
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
                .source_counter_quantity_live(source.object, kind)
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
                action: false,
                additional_objects: subjects.collect(),
            })
        });
        if reflexive
            .program()
            .is_some_and(|(_, program)| program.contains(&Instruction::EventObjectPowerDamage))
            && let Some(object) = subject
        {
            reflexive.set_event_object(EventObjectView {
                object,
                facts: EventObjectFacts::DamageActor(self.damage_source(object.card)),
                semantic_objects: None,
                conniver_creature: None,
                subtype_lki: None,
            });
        }
        self.state.pending_triggers.push(reflexive);
    }
    pub(super) fn exact_source_can_tap(&self, item: &SpellView) -> bool {
        item.source.is_some_and(|source| {
            self.on_battlefield(source.id)
                && self.object_ref(source.id) == source.object
                && !self.state.objects[source.id as usize].tapped
        })
    }
    pub(super) fn sacrifice_source_reflexive(&mut self, item: &SpellView, ability: u16) {
        if self.try_sacrifice_exact_source(item) {
            // Departure refreshes the saved packet's last-known source facts.
            let sacrificed = self.state.resolving.as_ref().unwrap().item.clone();
            self.queue_reflexive_objects(
                &sacrificed,
                ability,
                sacrificed.reflexive_action_objects(),
            );
        }
    }

    pub(super) fn complete_optional_source_reflexive(&mut self) {
        let resolution = self.state.resolving.as_ref().unwrap();
        match resolution
            .program
            .get(resolution.pc.saturating_sub(1))
            .copied()
        {
            Some(Instruction::MayTapSourceReflexive { ability })
                if self.exact_source_can_tap(&resolution.item) =>
            {
                let item = resolution.item.clone();
                let source = item.source.expect("tap source was checked");
                self.tap_objects_as(item.controller, &[source.id]);
                if self.on_battlefield(source.id)
                    && self.object_ref(source.id) == source.object
                    && self.state.objects[source.id as usize].tapped
                {
                    self.queue_reflexive(&item, ability, None);
                }
            }
            Some(Instruction::MaySacrificeSourceReflexive { ability }) => {
                let item = resolution.item.clone();
                self.sacrifice_source_reflexive(&item, ability);
            }
            _ => {}
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
        let Some(source) = item.source else {
            return;
        };
        assert!(token.role().is_some(), "predefined Role factory required");
        let recipients = self
            .state
            .objects
            .iter()
            .enumerate()
            .filter_map(|(id, _)| {
                let id = id as CardId;
                let recipient = self.object_ref(id);
                (self.creature_matches(filter, item.controller, source.object, id)
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
            for row in &mut r.basic_land_types {
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
        let Some(source) = item.source else {
            return;
        };
        assert!(
            token.role().is_some(),
            "Role instruction requires a predefined Role factory"
        );
        let recipients: Vec<_> = match subject {
            Subject::Source => (self.on_battlefield(source.id)
                && self.object_ref(source.id) == source.object)
                .then_some(source.object)
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
    /// CR 704.5z counts Roles that are still attached, even if a concurrent
    /// state-based action puts one of those Auras into its owner's graveyard.
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
                || !self.on_battlefield(v.recipient.card)
                || self.object_ref(v.recipient.card) != v.recipient
                || !self.has_subtype(v.aura.card, "Role")
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
