//! Source-linked casting permissions preserve exact objects and ability acquisition.
use super::*;
use crate::cards::StaticAbilityFragment;
#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
pub struct LinkedFreeCastUseView {
    pub source: ObjectRef,
    pub ability_origin: Option<u64>,
    pub player: Player,
    pub turn: u32,
}
#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
pub struct LinkedFreeCastApprovalView {
    pub usage: LinkedFreeCastUseView,
    pub stack_id: u64,
}
/// Receipt for the selected exact provider, with its entry effect frozen at proposal.
#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) struct LinkedExilePaidCast {
    pub source: ObjectRef,
    pub entry_counter: Option<CounterKind>,
}
#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
impl Game {
    fn live_linked_source(&self, player: Player, source: ObjectRef) -> Option<(Card, Option<u64>)> {
        let provider = self.state.objects.get(source.card as usize)?;
        (provider.zone == Zone::Battlefield
            && provider.controller == player
            && self.object_ref(source.card) == source)
            .then(|| (provider.card, self.copy_ability_origin(source)))
    }

    fn linked_source_definition(
        &self,
        player: Player,
        source: ObjectRef,
        card: CardId,
        this_turn: bool,
    ) -> Option<Card> {
        let (definition, origin) = self.live_linked_source(player, source)?;
        let exiled = self.state.objects.get(card as usize)?;
        if exiled.zone != Zone::Exile || !exiled.is_card() {
            return None;
        }
        self.state
            .linked_choices
            .as_ref()?
            .links
            .iter()
            .any(|link| {
                link.source == source
                    && link.ability_origin == origin
                    && link.exiled == self.object_ref(card)
                    && (!this_turn || link.exiled_turn == self.state.turn)
            })
            .then_some(definition)
    }
    pub(super) fn has_linked_life_play_permission(&self, player: Player, card: CardId) -> bool {
        if self.state.active != player
            || !self
                .state
                .objects
                .get(card as usize)
                .is_some_and(|object| object.zone == Zone::Exile && object.is_card())
        {
            return false;
        }
        let exact = self.object_ref(card);
        self.state.linked_choices.as_ref().is_some_and(|state| {
            state.links.iter().any(|link| {
                link.exiled == exact
                    && self
                        .live_linked_source(player, link.source)
                        .is_some_and(|(_, origin)| origin == link.ability_origin)
                    && self.active_rule_fragment(link.source.card, |fragment| {
                        matches!(
                            fragment,
                            StaticAbilityFragment::PlayLinkedExileOnOwnTurnForLife
                        )
                    })
            })
        })
    }

    fn linked_free_cast_limit(
        &self,
        player: Player,
        source: ObjectRef,
        card: CardId,
    ) -> Option<u64> {
        let definition = self.linked_source_definition(player, source, card, true)?;
        let origin = self.copy_ability_origin(source);
        if self
            .state
            .linked_choices
            .as_ref()?
            .free_cast_uses
            .iter()
            .any(|used| {
                used.source == source
                    && used.ability_origin == origin
                    && used.player == player
                    && used.turn == self.state.turn
            })
        {
            return None;
        }
        let subtypes = definition
            .static_ability_groups()
            .iter()
            .flat_map(|group| group.fragments)
            .find_map(|fragment| {
                let StaticAbilityFragment::LinkedExileFreeCastBySubtypeUnion(types) = fragment else {
                    return None;
                };
                self.continuous_fragment_available(source.card, |fragment| {
                    matches!(fragment, StaticAbilityFragment::LinkedExileFreeCastBySubtypeUnion(t) if t == types)
                })
                .then_some(*types)
            })?;
        Some(
            self.state
                .objects
                .iter()
                .enumerate()
                .filter(|(id, o)| {
                    o.zone == Zone::Battlefield
                        && o.controller == player
                        && subtypes.iter().any(|t| self.has_subtype(*id as CardId, t))
                })
                .count() as u64,
        )
    }
    pub(super) fn linked_counter_permission(
        &self,
        player: Player,
        source: ObjectRef,
        card: CardId,
        face: CastFace,
    ) -> Option<u16> {
        let definition = self.linked_source_definition(player, source, card, false)?;
        let exiled = &self.state.objects[card as usize];
        let spell = face.definition(exiled.card)?;
        if self.state.active != player || exiled.owner != player || !spell.is_creature() {
            return None;
        }
        definition
            .static_ability_groups()
            .iter()
            .flat_map(|group| group.fragments)
            .find_map(|fragment| {
                let StaticAbilityFragment::LinkedExileCreatureCounterCasting(amount) = fragment else {
                    return None;
                };
                self.continuous_fragment_available(source.card, |fragment| {
                    matches!(fragment, StaticAbilityFragment::LinkedExileCreatureCounterCasting(n) if n == amount)
                })
                .then_some(*amount)
            })
    }
    pub(super) fn linked_paid_permission(
        &self,
        player: Player,
        source: ObjectRef,
        object: ObjectRef,
        face: CastFace,
    ) -> Option<LinkedExilePaidCast> {
        let provider = self.linked_source_definition(player, source, object.card, false)?;
        let exiled = &self.state.objects[object.card as usize];
        if self.object_ref(object.card) != object || exiled.owner != player {
            return None;
        }
        let spell = face.definition(exiled.card)?;
        if spell.is_land() || matches!(spell.effect(), Effect::Unsupported) {
            return None;
        }
        provider
            .static_ability_groups()
            .iter()
            .flat_map(|group| group.fragments)
            .find_map(|fragment| {
                let StaticAbilityFragment::LinkedExilePaidCast {
                    filter,
                    entry_counter,
                } = *fragment
                else {
                    return None;
                };
                (filter.matches(spell)
                    && self.continuous_fragment_available(source.card, |found| {
                        std::ptr::eq(found, fragment)
                    }))
                .then_some(LinkedExilePaidCast {
                    source,
                    entry_counter,
                })
            })
    }
    pub(super) fn visit_linked_source_casts(
        &self,
        player: Player,
        card: CardId,
        sorcery: bool,
        exile_zero: &[ObjectRef],
        mana_alternatives: &[(ObjectRef, ManaCost)],
        emit: &mut impl FnMut(Action) -> bool,
    ) -> bool {
        for &id in &self.state.static_sources {
            let source = self.object_ref(id);
            if let Some(limit) = self.linked_free_cast_limit(player, source, card) {
                let physical = self.state.objects[card as usize].card;
                for face in CastFace::choices(self.state.objects[card as usize].card) {
                    let Some(definition) = face.definition(physical) else {
                        continue;
                    };
                    if definition.is_land() || definition.fixed_mana_value() > limit {
                        continue;
                    }
                    for flash_surcharge in [false, true] {
                        if self.permitted_exile_free_cast_available(
                            player,
                            card,
                            face,
                            sorcery,
                            flash_surcharge,
                        ) && emit(Action::AnnounceLinkedFree {
                            card,
                            source,
                            face,
                            flash_surcharge,
                        }) {
                            return true;
                        }
                    }
                }
            }
            for face in CastFace::choices(self.state.objects[card as usize].card) {
                let object = self.object_ref(card);
                if let Some(permission) = self.linked_paid_permission(player, source, object, face)
                {
                    if self.visit_exile_zero_face(
                        player,
                        object,
                        face,
                        sorcery,
                        ExileZeroOrigin::LinkedPaid { source },
                        exile_zero,
                        emit,
                    ) || self.visit_paid_cast_alternative_face(
                        player,
                        card,
                        face,
                        sorcery,
                        PaidCastOrigin::LinkedExilePaid { object, source },
                        mana_alternatives,
                        emit,
                    ) {
                        return true;
                    }
                    for flash_surcharge in [false, true] {
                        if self.permitted_cast_with_alternative(
                            player,
                            card,
                            CastMethod::Normal,
                            face,
                            sorcery,
                            flash_surcharge,
                            None,
                            None,
                            None,
                            false,
                            cast_permissions::OriginAlternative {
                                linked_exile_paid: Some(permission),
                                ..Default::default()
                            },
                        ) && emit(Action::AnnounceLinkedPaid {
                            object,
                            source,
                            face,
                            flash_surcharge,
                        }) {
                            return true;
                        }
                    }
                }
                let Some(amount) = self.linked_counter_permission(player, source, card, face)
                else {
                    continue;
                };
                if self.visit_exile_zero_face(
                    player,
                    object,
                    face,
                    sorcery,
                    ExileZeroOrigin::LinkedCounters { source },
                    exile_zero,
                    emit,
                ) || self.visit_paid_cast_alternative_face(
                    player,
                    card,
                    face,
                    sorcery,
                    PaidCastOrigin::LinkedExileCounters { source },
                    mana_alternatives,
                    emit,
                ) {
                    return true;
                }
                for flash_surcharge in [false, true] {
                    if self.permitted_cast_with_origin_cost(
                        player,
                        card,
                        CastMethod::LinkedExileCounterCost,
                        face,
                        sorcery,
                        flash_surcharge,
                        Some(amount),
                    ) && emit(Action::AnnounceLinkedExileCounterCost {
                        card,
                        source,
                        face,
                        flash_surcharge,
                    }) {
                        return true;
                    }
                }
            }
        }
        false
    }
    pub(super) fn configure_linked_paid_cast(item: &mut SpellView) {
        let cast = item.cast.as_mut().expect("paid spell");
        cast.spend_any_mana = false;
        cast.exile_on_graveyard = false;
        cast.stash_origin = false;
    }

    pub(super) fn select_linked_paid_cast(&mut self, item: &mut SpellView) {
        Self::configure_linked_paid_cast(item);
        if let Some(data) = &mut self.state.linked_choices {
            data.play_approvals.retain(|row| row.0 != item.id);
        }
    }

    pub(super) fn start_linked_paid_cast(
        &mut self,
        player: Player,
        object: ObjectRef,
        source: ObjectRef,
        face: CastFace,
        flash_surcharge: bool,
    ) {
        let permission = self
            .linked_paid_permission(player, source, object, face)
            .expect("approved exact linked permission");
        let mut item = self.cast_item(player, object.card, [0; 6]);
        self.set_cast_face(&mut item, face);
        self.select_linked_paid_cast(&mut item);
        item.cast.as_mut().unwrap().flash_surcharge = flash_surcharge;
        self.open_payment_announcement_configured(item, Counters::default(), None, None, |a| {
            a.limited_cast_permission =
                Some(Box::new(LimitedCastPermission::LinkedExilePaid(permission)));
        });
    }

    pub(super) fn start_linked_free_cast(
        &mut self,
        player: Player,
        card: CardId,
        source: ObjectRef,
        face: CastFace,
        flash_surcharge: bool,
    ) {
        let usage = LinkedFreeCastUseView {
            source,
            ability_origin: self.copy_ability_origin(source),
            player,
            turn: self.state.turn,
        };
        let mut item = self.cast_item(player, card, [0; 6]);
        self.set_cast_face(&mut item, face);
        Self::configure_exile_free_cast(&mut item, flash_surcharge);
        let data = self.state.linked_choices.get_or_insert_default();
        data.play_approvals.retain(|row| row.0 != item.id);
        data.free_cast_approvals.push(LinkedFreeCastApprovalView {
            usage,
            stack_id: item.id,
        });
        self.open_payment_announcement(item, Counters::default(), None);
    }
    pub(super) fn start_linked_counter_cast(
        &mut self,
        player: Player,
        card: CardId,
        source: ObjectRef,
        face: CastFace,
        flash_surcharge: bool,
    ) {
        let amount = self
            .linked_counter_permission(player, source, card, face)
            .expect("approved linked alternative");
        let mut item = self.cast_item(player, card, [0; 6]);
        self.set_cast_face(&mut item, face);
        item.cast.as_mut().unwrap().flash_surcharge = flash_surcharge;
        item.cast.as_mut().unwrap().method = CastMethod::LinkedExileCounterCost;
        if let Some(data) = &mut self.state.linked_choices {
            data.play_approvals.retain(|row| row.0 != item.id);
        }
        self.open_payment_announcement_with_origin_rules(
            item,
            Counters::default(),
            None,
            Some(amount),
            None,
            None,
        );
    }
    pub(super) fn commit_linked_free_cast(&mut self, item: &SpellView) {
        let Some(data) = &mut self.state.linked_choices else {
            return;
        };
        if let Some(index) = data
            .free_cast_approvals
            .iter()
            .position(|r| r.stack_id == item.id)
        {
            let approval = data.free_cast_approvals.remove(index);
            data.free_cast_uses.push(approval.usage);
        }
    }
}
