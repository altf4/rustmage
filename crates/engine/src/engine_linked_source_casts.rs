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
#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
impl Game {
    fn linked_source_definition(
        &self,
        player: Player,
        source: ObjectRef,
        card: CardId,
        this_turn: bool,
    ) -> Option<Card> {
        let provider = self.state.objects.get(source.card as usize)?;
        let exiled = self.state.objects.get(card as usize)?;
        if self.object_ref(source.card) != source
            || provider.zone != Zone::Battlefield
            || provider.controller != player
            || exiled.zone != Zone::Exile
            || !exiled.is_card()
        {
            return None;
        }
        let origin = self.copy_ability_origin(source);
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
            .then_some(provider.card)
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
        let spell = match face {
            CastFace::Normal => Some(exiled.card),
            CastFace::Adventure => exiled.card.adventure_spell(),
            CastFace::Omen => exiled.card.omen_spell(),
            CastFace::Back => exiled.card.back_face(),
        }?;
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
    pub(super) fn visit_linked_source_casts(
        &self,
        player: Player,
        card: CardId,
        sorcery: bool,
        emit: &mut impl FnMut(Action) -> bool,
    ) -> bool {
        for &id in &self.state.static_sources {
            let source = self.object_ref(id);
            if let Some(limit) = self.linked_free_cast_limit(player, source, card) {
                let physical = self.state.objects[card as usize].card;
                for face in [
                    CastFace::Normal,
                    CastFace::Adventure,
                    CastFace::Omen,
                    CastFace::Back,
                ] {
                    let Some(definition) = (match face {
                        CastFace::Normal => Some(physical),
                        CastFace::Adventure => physical.adventure_spell(),
                        CastFace::Omen => physical.omen_spell(),
                        CastFace::Back => physical.back_face(),
                    }) else {
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
            for face in [
                CastFace::Normal,
                CastFace::Adventure,
                CastFace::Omen,
                CastFace::Back,
            ] {
                let Some(amount) = self.linked_counter_permission(player, source, card, face)
                else {
                    continue;
                };
                if self.visit_web_slinging_face(
                    player,
                    card,
                    face,
                    sorcery,
                    WebSlingingOrigin::LinkedExileCounters { source },
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
