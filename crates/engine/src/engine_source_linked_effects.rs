//! Resolution-created continuous effects with exact, irreversible source durations.
use super::ability_world::CharacteristicWorld;
use super::*;

#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize)]
pub enum SourceLinkedEffectKind {
    RemoveAbilities,
    /// Type replacement and blue mana grant, ending at the captured controller's next turn.
    LandUntilControllerNextTurn,
    /// Source-independent removal lasting until cleanup.
    RemoveAbilitiesUntilEndOfTurn,
    GrantHexproof,
    /// Independent land grant, ending when the captured exile incarnation is cast.
    GrantTwoColorlessUntilSourceCast,
    PreventDamageBy,
    PreventUntap,
    /// Source-independent prohibition ending at the recipient’s first untap.
    PreventActivationsUntilUntapped,
    Control,
    /// One fixed resolving controller; losing source control ends both effects forever.
    ControlAndCombatLockWhileControlled,
}
#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
pub struct SourceLinkedEffectView {
    pub source: ObjectRef,
    pub recipient: ObjectRef,
    pub controller: Player,
    pub kind: SourceLinkedEffectKind,
    pub timestamp: u64,
}
#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
pub(super) fn active<W: CharacteristicWorld + ?Sized>(
    world: &W,
    row: &SourceLinkedEffectView,
) -> bool {
    (matches!(
        row.kind,
        SourceLinkedEffectKind::RemoveAbilitiesUntilEndOfTurn
            | SourceLinkedEffectKind::GrantTwoColorlessUntilSourceCast
            | SourceLinkedEffectKind::PreventActivationsUntilUntapped
    ) || (world.object(row.source.card).zone == Zone::Battlefield
        && world.reference(row.source.card) == row.source))
        && (row.kind == SourceLinkedEffectKind::PreventDamageBy
            || (world.object(row.recipient.card).zone == Zone::Battlefield
                && world.reference(row.recipient.card) == row.recipient))
        && (!matches!(
            row.kind,
            SourceLinkedEffectKind::PreventUntap
                | SourceLinkedEffectKind::ControlAndCombatLockWhileControlled
        ) || world.object(row.source.card).controller == row.controller)
        && (row.kind != SourceLinkedEffectKind::PreventActivationsUntilUntapped
            || world.object(row.recipient.card).tapped)
}
#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
impl Game {
    pub(super) fn resolve_source_linked_effect(
        &mut self,
        item: &SpellView,
        kind: SourceLinkedEffectKind,
    ) {
        let Some(source) = item.source else {
            return;
        };
        // CR611.2b: an already-ended duration never starts. The preceding tap
        // remains independent and has already happened even when this fails.
        if !self.on_battlefield(source.object.card)
            || self.object_ref(source.object.card) != source.object
            || (matches!(
                kind,
                SourceLinkedEffectKind::PreventUntap
                    | SourceLinkedEffectKind::ControlAndCombatLockWhileControlled
            ) && self.state.objects[source.object.card as usize].controller != item.controller)
        {
            return;
        }
        let recipients: Vec<_> = self
            .keyword_subjects(item, Subject::Targets)
            .into_iter()
            .filter(|&id| self.on_battlefield(id))
            .map(|id| self.object_ref(id))
            .collect();
        if recipients.is_empty() {
            return;
        }
        if matches!(
            kind,
            SourceLinkedEffectKind::Control
                | SourceLinkedEffectKind::ControlAndCombatLockWhileControlled
        ) {
            for &recipient in &recipients {
                self.ensure_control_base(recipient);
            }
        }
        let timestamp = self.next_role_timestamp();
        let rows = &mut self.state.roles.as_mut().unwrap().source_linked_effects;
        for recipient in recipients {
            rows.push(SourceLinkedEffectView {
                source: source.object,
                recipient,
                controller: item.controller,
                kind,
                timestamp,
            });
        }
        if matches!(
            kind,
            SourceLinkedEffectKind::Control
                | SourceLinkedEffectKind::ControlAndCombatLockWhileControlled
        ) {
            self.refresh_controllers();
        } else {
            self.refresh_copy_grant_availability();
        }
    }
    pub(super) fn grant_land_mana_until_paid_source_cast(&mut self, item: &SpellView) {
        let Some(source_move) = item.source_move.as_ref() else {
            return;
        };
        // A replacement can change or prevent the exile event while the cost
        // remains paid (CR 118.11). The exact resulting incarnation anchors
        // the duration; exile permission is separately conditional below.
        if source_move.origin != Zone::Hand {
            return;
        }
        let source = source_move.after;
        if self
            .state
            .roles
            .as_ref()
            .is_some_and(|roles| roles.exile_grant_cast_history.contains(&source))
        {
            return;
        }
        let recipients: Vec<_> = self
            .keyword_subjects(item, Subject::Targets)
            .into_iter()
            .filter(|&id| self.on_battlefield(id) && self.current_kinds(id) & 2 != 0)
            .map(|id| self.object_ref(id))
            .collect();
        if recipients.is_empty() {
            return;
        }
        let timestamp = self.next_role_timestamp();
        for recipient in recipients {
            self.state
                .roles
                .get_or_insert_default()
                .source_linked_effects
                .push(SourceLinkedEffectView {
                    source,
                    recipient,
                    controller: item.controller,
                    kind: SourceLinkedEffectKind::GrantTwoColorlessUntilSourceCast,
                    timestamp,
                });
        }
        if self.object_ref(source.card) == source
            && self.state.objects[source.card as usize].zone == Zone::Exile
        {
            let index = self
                .state
                .exile_permissions
                .partition_point(|row| row.object.card < source.card);
            self.state.exile_permissions.insert(
                index,
                ExilePlayPermission {
                    object: source,
                    player: item.controller,
                    expiry: ExilePlayExpiry::WhileExiled,
                    spend_any_mana: false,
                    cast_only: true,
                    exile_on_graveyard: false,
                    life_instead_of_mana: false,
                    without_paying_mana_cost: false,
                    airbend: false,
                    requires_subtype: None,
                },
            );
        }
    }
    pub(super) fn expire_land_mana_on_exile_cast(&mut self, item: &SpellView) {
        if !item.cast.is_some_and(|cast| cast.origin == Zone::Exile) {
            return;
        }
        let Some(source) = item.source else {
            return;
        };
        let Some(generation) = source.object.generation.checked_sub(1) else {
            return;
        };
        let before = ObjectRef {
            card: source.object.card,
            generation,
        };
        // Retain the exact pre-cast incarnation even when an ability copy is
        // nested inside a continuation rather than directly on the stack.
        let rows = &mut self
            .state
            .roles
            .get_or_insert_default()
            .exile_grant_cast_history;
        if !rows.contains(&before) {
            rows.push(before);
        }
        if let Some(roles) = &mut self.state.roles {
            roles.source_linked_effects.retain(|row| {
                row.kind != SourceLinkedEffectKind::GrantTwoColorlessUntilSourceCast
                    || row.source != before
            });
        }
    }
    /// Reclaim history only at a quiescent boundary where no retained spell
    /// packet or rollback can later recreate a pending hand ability.
    pub(super) fn prune_exile_grant_cast_history(&mut self) {
        if !self.state.stack.is_empty()
            || !self.state.pending_triggers.is_empty()
            || !self.state.ordering_triggers.is_empty()
            || self.state.resolving.is_some()
            || self.state.announcing.is_some()
            || self.state.mana_announcement_stack.is_some()
            || self.state.cast_rollback.is_some()
            || self.state.stack_copy_data.is_some()
            || self.state.delayed_target_events.is_some()
            || self.state.paradigm.is_some()
            || self.state.entry_batch.is_some()
            || self
                .state
                .zone_rules
                .as_ref()
                .is_some_and(|z| z.has_checkpoint())
            || self.state.sba_batch.is_some()
            || self.state.trigger_multiplicity.is_some()
            || self.state.resolution_free_casts.is_some()
            || self.state.miracle.is_some()
            || self.state.mobilize.is_some()
            || self.state.warp.is_some()
            || self
                .state
                .delayed_sacrifices
                .as_ref()
                .is_some_and(|rows| !rows.is_empty())
            || self.state.replacements.is_some()
            || self.state.linked_choices.is_some()
            || self
                .state
                .delayed_artifact_copies
                .as_ref()
                .is_some_and(|rows| !rows.is_empty())
            || self.state.combat_rules.as_ref().is_some_and(|rules| {
                rules
                    .begin_combat_this_turn
                    .as_ref()
                    .is_some_and(|rows| !rows.is_empty())
            })
            || self
                .state
                .delayed_attack_this_turn
                .as_ref()
                .is_some_and(|rows| !rows.is_empty())
            || self
                .state
                .delayed_opponent_attack_until_next_turn
                .as_ref()
                .is_some_and(|rows| !rows.is_empty())
            || self
                .state
                .delayed_main_mana
                .as_ref()
                .is_some_and(|rows| !rows.is_empty())
            || self
                .state
                .delayed_upkeep_tokens
                .as_ref()
                .is_some_and(|rows| !rows.is_empty())
            || self
                .state
                .delayed_returns
                .as_ref()
                .is_some_and(|rows| !rows.is_empty())
            || self
                .state
                .earthbend_returns
                .as_ref()
                .is_some_and(|rows| !rows.is_empty())
            || self
                .state
                .manifest
                .as_ref()
                .is_some_and(|m| !m.deaths.is_empty())
        {
            return;
        }
        if let Some(roles) = &mut self.state.roles {
            roles.exile_grant_cast_history.clear();
        }
    }
    pub(super) fn remove_target_abilities_until_end_of_turn(&mut self, item: &SpellView) {
        let Some(source) = item.source else {
            return;
        };
        let recipients: Vec<_> = self
            .keyword_subjects(item, Subject::Targets)
            .into_iter()
            .filter(|&id| self.on_battlefield(id))
            .map(|id| self.object_ref(id))
            .collect();
        if recipients.is_empty() {
            return;
        }
        let timestamp = self.next_role_timestamp();
        let rows = &mut self.state.roles.as_mut().unwrap().source_linked_effects;
        for recipient in recipients {
            rows.push(SourceLinkedEffectView {
                source: source.object,
                recipient,
                controller: item.controller,
                kind: SourceLinkedEffectKind::RemoveAbilitiesUntilEndOfTurn,
                timestamp,
            });
        }
        self.refresh_copy_grant_availability();
    }
    pub(super) fn expire_temporary_ability_removals(&mut self) {
        let changed = if let Some(roles) = &mut self.state.roles {
            let before = roles.source_linked_effects.len();
            roles
                .source_linked_effects
                .retain(|row| row.kind != SourceLinkedEffectKind::RemoveAbilitiesUntilEndOfTurn);
            before != roles.source_linked_effects.len()
        } else {
            false
        };
        if changed {
            self.refresh_copy_grant_availability();
        }
    }
    pub(super) fn prevent_target_activations_until_untapped(&mut self, item: &SpellView) {
        let Some(source) = item.source else { return };
        let recipients: Vec<_> = self
            .keyword_subjects(item, Subject::Targets)
            .into_iter()
            .filter(|&id| self.on_battlefield(id) && self.state.objects[id as usize].tapped)
            .map(|id| self.object_ref(id))
            .collect();
        if recipients.is_empty() {
            return;
        }
        let timestamp = self.next_role_timestamp();
        for recipient in recipients {
            self.state
                .roles
                .as_mut()
                .unwrap()
                .source_linked_effects
                .push(SourceLinkedEffectView {
                    source: source.object,
                    recipient,
                    controller: item.controller,
                    kind: SourceLinkedEffectKind::PreventActivationsUntilUntapped,
                    timestamp,
                });
        }
    }

    pub(super) fn forget_untapped_activation_locks(&mut self, untapped: &[ObjectRef]) {
        if let Some(state) = &mut self.state.roles {
            state.source_linked_effects.retain(|row| {
                row.kind != SourceLinkedEffectKind::PreventActivationsUntilUntapped
                    || !untapped.contains(&row.recipient)
            });
        }
    }
    pub(super) fn forget_source_linked_effects(&mut self, reference: ObjectRef) {
        if let Some(state) = &mut self.state.roles {
            state.source_linked_effects.retain(|row| {
                (row.source != reference
                    || matches!(
                        row.kind,
                        SourceLinkedEffectKind::RemoveAbilitiesUntilEndOfTurn
                            | SourceLinkedEffectKind::GrantTwoColorlessUntilSourceCast
                            | SourceLinkedEffectKind::PreventActivationsUntilUntapped
                    ))
                    && (row.recipient != reference
                        || row.kind == SourceLinkedEffectKind::PreventDamageBy)
            });
        }
    }
    pub(super) fn end_control_linked_effects(&mut self, reference: ObjectRef) {
        if let Some(state) = &mut self.state.roles {
            state.source_linked_effects.retain(|row| {
                row.source != reference
                    || !matches!(
                        row.kind,
                        SourceLinkedEffectKind::PreventUntap
                            | SourceLinkedEffectKind::ControlAndCombatLockWhileControlled
                    )
            });
        }
    }
    pub(super) fn source_linked_effect_views(&self) -> Option<Vec<SourceLinkedEffectView>> {
        let rows: Vec<_> = self
            .state
            .roles
            .as_ref()?
            .source_linked_effects
            .iter()
            .filter(|row| active(self, row))
            .copied()
            .collect();
        (!rows.is_empty()).then_some(rows)
    }
}

#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
impl std::hash::Hash for SourceLinkedEffectView {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.source.hash(state);
        self.recipient.hash(state);
        self.controller.index().hash(state);
        self.kind.hash(state);
        self.timestamp.hash(state);
    }
}

#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
impl Game {
    pub(super) fn counter_ability_and_remove_source_abilities(&mut self, item: &SpellView) {
        for bound in &item.targets {
            let Some(victim) = self.bound_stack_item(bound).cloned() else {
                continue;
            };
            if victim.kind == StackKind::Spell {
                continue;
            }
            let recipient = victim.source.and_then(|source| {
                (self.on_battlefield(source.object.card)
                    && self.object_ref(source.object.card) == source.object
                    && self.current_kinds(source.object.card) & (1 | 16 | 64) != 0)
                    .then_some(source.object)
            });
            if !self.counter_stack_item(victim.id, false) {
                continue;
            }
            if let Some(recipient) = recipient {
                let mut removal = item.clone();
                removal.targets = vec![BoundTarget {
                    group: 0,
                    target: Target::Card(recipient.card),
                    object: Some(recipient),
                }];
                self.resolve_source_linked_effect(
                    &removal,
                    SourceLinkedEffectKind::RemoveAbilities,
                );
            }
        }
    }

    pub(super) fn remove_abilities_then_destroy_creatures_mana_at_most(
        &mut self,
        item: &SpellView,
        amount: AmountValue,
    ) {
        let maximum = self.item_amount(amount, item);
        // Freeze the affected group before ability removal changes characteristics.
        let ids: Vec<_> = (0..self.state.objects.len())
            .map(|id| id as CardId)
            .filter(|&id| {
                self.on_battlefield(id)
                    && self.is_creature(id)
                    && u128::from(self.object_mana_value(id)) <= maximum
            })
            .collect();
        let mut removal = item.clone();
        removal.targets = ids
            .iter()
            .map(|&id| BoundTarget {
                group: 0,
                target: Target::Card(id),
                object: Some(self.object_ref(id)),
            })
            .collect();
        self.remove_target_abilities_until_end_of_turn(&removal);
        let ids: Vec<_> = ids
            .into_iter()
            .filter(|&id| !self.effect_destruction_prevented(id))
            .collect();
        let sources = self.death_trigger_sources();
        let events = self.move_objects_raw(&ids, Zone::Graveyard);
        self.finish_zone_events(&events, &sources);
    }
    pub(super) fn source_land_until_next_turn(&mut self, item: &SpellView) {
        let Some(source) = item.source else {
            return;
        };
        let mut effect = item.clone();
        effect.targets = vec![BoundTarget {
            group: 0,
            target: Target::Card(source.id),
            object: Some(source.object),
        }];
        self.resolve_source_linked_effect(
            &effect,
            SourceLinkedEffectKind::LandUntilControllerNextTurn,
        );
    }
    pub(super) fn expire_source_land_forms(&mut self, player: Player) {
        let changed = self.state.roles.as_mut().is_some_and(|roles| {
            let before = roles.source_linked_effects.len();
            roles.source_linked_effects.retain(|row| {
                row.kind != SourceLinkedEffectKind::LandUntilControllerNextTurn
                    || row.controller != player
            });
            before != roles.source_linked_effects.len()
        });
        if changed {
            self.refresh_copy_grant_availability();
        }
    }
}
