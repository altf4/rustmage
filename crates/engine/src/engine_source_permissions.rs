//! Exact source-owned permissions for one play during each of its controller's turns.
use super::outside_choices::Selection;
use super::*;
use crate::cards::{PERMANENT_KINDS, StaticAbilityFragment};

#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum SourcePlayKind {
    GraveyardArtifactCreatureFinality,
    GraveyardOrOutsidePermanent,
}

impl SourcePlayKind {
    fn matches(self, fragment: &StaticAbilityFragment) -> bool {
        matches!(
            (self, fragment),
            (
                Self::GraveyardArtifactCreatureFinality,
                StaticAbilityFragment::CastArtifactCreatureFromGraveyardOnceEachTurnWithFinality
            ) | (
                Self::GraveyardOrOutsidePermanent,
                StaticAbilityFragment::PlayPermanentFromGraveyardOrOutsideOnceEachTurn
            )
        )
    }

    pub(super) fn accepts(self, definition: Card) -> bool {
        match self {
            Self::GraveyardArtifactCreatureFinality => {
                definition.is_artifact() && definition.is_creature()
            }
            Self::GraveyardOrOutsidePermanent => definition.kind_mask() & PERMANENT_KINDS != 0,
        }
    }
}

#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
pub struct SourcePlayUse {
    pub source: ObjectRef,
    pub kind: SourcePlayKind,
    pub turn: u32,
}

#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
pub struct ExiledCopyGrant {
    pub recipient: ObjectRef,
    pub template: Card,
    pub ability: u16,
}

#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) struct OutsidePlay {
    pub player: Player,
    pub source: ObjectRef,
    pub selection: Selection,
    pub object: ObjectRef,
}

#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
impl Game {
    pub(super) fn exile_until_nonland_grant_copy_cast(&mut self, item: &SpellView, ability: u16) {
        let Some(recipient) = self.exile_until_first_nonland(item.controller) else {
            return;
        };
        let template = item.trigger_definition();
        let rules = template.activation(ability);
        assert!(template.self_granted_activation_template(ability));
        assert!(matches!(
            rules.cost.additional,
            Some(crate::cards::AdditionalCost {
                kind: crate::cards::EffectChoiceKind::ExileFromHand,
                ..
            })
        ));
        assert!(matches!(
            rules.program,
            [Instruction::CopyExiledSourceOfferPaidCast]
        ));
        self.state
            .exiled_copy_grants
            .get_or_insert_default()
            .push(ExiledCopyGrant {
                recipient,
                template,
                ability,
            });
    }

    fn exiled_copy_packet(&self, row: ExiledCopyGrant, actor: Player) -> SpellView {
        let mut item = self.activation_packet(actor, row.recipient.card, row.ability);
        item.set_source_definition(row.template);
        // CR 707.12 and 608.2h: the ability uses the card's last copiable
        // characteristics even if the source leaves exile before resolution.
        item.source_copy_recipe = Some(Box::new(self.copiable_recipe(row.recipient)));
        item
    }

    pub(super) fn visit_exiled_copy_grants(
        &self,
        actor: Player,
        emit: &mut impl FnMut(Action) -> bool,
    ) -> bool {
        if self.state.active != actor || self.nonmana_actions_prohibited() {
            return false;
        }
        let Some(rows) = self.state.exiled_copy_grants.as_ref() else {
            return false;
        };
        for &row in rows.iter() {
            let id = row.recipient.card;
            if self.object_ref(id) != row.recipient
                || self.state.objects[id as usize].zone != Zone::Exile
                || self.state.objects[id as usize].owner != actor
                || self.named_activation_prohibited(id)
                || ability_world::CharacteristicWorld::activation_prohibited(self, actor, id)
            {
                continue;
            }
            let item = self.exiled_copy_packet(row, actor);
            if self.initial_activation_packet_target_fundable(&item, &Counters::default())
                && emit(Action::AnnounceExiledCopyGrant {
                    source: row.recipient,
                })
            {
                return true;
            }
        }
        false
    }

    pub(super) fn announce_exiled_copy_grant(&mut self, actor: Player, source: ObjectRef) {
        let row = *self
            .state
            .exiled_copy_grants
            .as_ref()
            .unwrap()
            .iter()
            .find(|row| row.recipient == source)
            .unwrap();
        let mut item = self.exiled_copy_packet(row, actor);
        item.id = self.next_stack_id();
        self.state.passes = 0;
        self.open_payment_announcement(item, Counters::default(), None);
    }

    pub(super) fn copy_exiled_source_offer_paid_cast(&mut self, item: &SpellView) -> bool {
        let Some(recipe) = item.source_copy_recipe.as_ref() else {
            return false;
        };
        let recipe = **recipe;
        let copy = self.allocate_card_copy(recipe.definition, item.controller);
        self.set_copy_recipe(copy, recipe);
        self.offer_exiled_cards_paid_cast(item.controller, vec![copy], Some(1));
        true
    }
    pub(super) fn source_play_available(
        &self,
        actor: Player,
        source: ObjectRef,
        kind: SourcePlayKind,
    ) -> bool {
        if self.state.active != actor
            || self.object_ref(source.card) != source
            || self.state.source_play_uses.as_ref().is_some_and(|uses| {
                uses.iter().any(|row| {
                    row.source == source && row.kind == kind && row.turn == self.state.turn
                })
            })
        {
            return false;
        }
        let object = &self.state.objects[source.card as usize];
        object.zone == Zone::Battlefield
            && object.controller == actor
            && object.card.static_ability_groups().iter().any(|group| {
                group.fragments.iter().any(|fragment| {
                    kind.matches(fragment)
                        && self.continuous_fragment_available(source.card, |found| {
                            std::ptr::eq(found, fragment)
                        })
                })
            })
    }

    pub(super) fn source_play_sources(
        &self,
        actor: Player,
        kind: SourcePlayKind,
    ) -> Vec<ObjectRef> {
        self.state
            .static_sources
            .iter()
            .filter_map(|&id| {
                let source = self.object_ref(id);
                self.source_play_available(actor, source, kind)
                    .then_some(source)
            })
            .collect()
    }

    pub(super) fn use_source_play(&mut self, source: ObjectRef, kind: SourcePlayKind) {
        let turn = self.state.turn;
        let uses = self.state.source_play_uses.get_or_insert_default();
        uses.retain(|row| row.turn == turn);
        assert!(
            !uses
                .iter()
                .any(|row| row.source == source && row.kind == kind)
        );
        uses.push(SourcePlayUse { source, kind, turn });
    }

    pub(super) fn visit_source_graveyard_plays(
        &self,
        actor: Player,
        sorcery: bool,
        casting_prohibited: bool,
        cast_alternatives: &[PaidCastAlternative],
        emit: &mut impl FnMut(Action) -> bool,
    ) -> bool {
        for kind in [
            SourcePlayKind::GraveyardArtifactCreatureFinality,
            SourcePlayKind::GraveyardOrOutsidePermanent,
        ] {
            for source in self.source_play_sources(actor, kind) {
                for (index, object) in self.state.objects.iter().enumerate() {
                    if object.zone != Zone::Graveyard || object.owner != actor || !object.is_card()
                    {
                        continue;
                    }
                    let card = index as CardId;
                    for face in CastFace::choices(object.card) {
                        let Some(definition) = face.definition(object.card) else {
                            continue;
                        };
                        if !kind.accepts(definition) {
                            continue;
                        }
                        if definition.is_land() {
                            if kind == SourcePlayKind::GraveyardOrOutsidePermanent
                                && face == CastFace::Normal
                                && sorcery
                                && (!self.state.land_played
                                    || self
                                        .continuous_land_plays(actor)
                                        .checked_add(
                                            self.state.players[actor.index()]
                                                .extra_land_plays_granted,
                                        )
                                        .capacity("land allowance exhausted")
                                        > self.state.continuous_land_plays_used)
                                && emit(Action::PlaySourceGraveyardLand {
                                    card,
                                    source,
                                    play_kind: kind,
                                })
                            {
                                return true;
                            }
                        } else if !casting_prohibited {
                            if self.visit_paid_cast_alternative_face(
                                actor,
                                card,
                                face,
                                sorcery,
                                PaidCastOrigin::SourcePlay {
                                    source,
                                    kind,
                                    outside: false,
                                },
                                cast_alternatives,
                                emit,
                            ) {
                                return true;
                            }
                            for flash_surcharge in [false, true] {
                                if self.permitted_cast_available(
                                    actor,
                                    card,
                                    CastMethod::Normal,
                                    face,
                                    sorcery,
                                    flash_surcharge,
                                ) && emit(Action::AnnounceSourceGraveyard {
                                    card,
                                    source,
                                    play_kind: kind,
                                    face,
                                    flash_surcharge,
                                }) {
                                    return true;
                                }
                            }
                        }
                    }
                }
            }
        }
        false
    }

    pub(super) fn visit_source_outside_plays(
        &self,
        actor: Player,
        sorcery: bool,
        casting_prohibited: bool,
        emit: &mut impl FnMut(Action) -> bool,
    ) -> bool {
        if self.decision_controller(actor) != actor {
            return false;
        }
        let Some(outside) = self.state.outside_game.as_ref() else {
            return false;
        };
        let kind = SourcePlayKind::GraveyardOrOutsidePermanent;
        for source in self.source_play_sources(actor, kind) {
            for selection in outside.candidates(actor) {
                let card = outside.card(actor, selection);
                let can_land = card.is_land()
                    && sorcery
                    && (!self.state.land_played
                        || self
                            .continuous_land_plays(actor)
                            .checked_add(self.state.players[actor.index()].extra_land_plays_granted)
                            .capacity("land allowance exhausted")
                            > self.state.continuous_land_plays_used);
                let can_spell = !casting_prohibited
                    && CastFace::choices(card).any(|face| {
                        face.definition(card)
                            .is_some_and(|d| !d.is_land() && kind.accepts(d))
                    });
                if (can_land || can_spell)
                    && emit(Action::SelectOutsideSourcePlay { source, selection })
                {
                    return true;
                }
            }
        }
        false
    }

    pub(super) fn select_outside_source_play(
        &mut self,
        actor: Player,
        source: ObjectRef,
        selection: Selection,
    ) {
        let card = self
            .state
            .outside_game
            .as_ref()
            .unwrap()
            .card(actor, selection);
        let object = self.allocate_outside_play_object(actor, card);
        self.state.outside_play = Some(OutsidePlay {
            player: actor,
            source,
            selection,
            object,
        });
        self.state.pending = Pending::OutsideSourcePlay { player: actor };
    }

    pub(super) fn visit_selected_outside_source_play(
        &self,
        cast_alternatives: &[PaidCastAlternative],
        emit: &mut impl FnMut(Action) -> bool,
    ) -> bool {
        if emit(Action::CancelOutsideSourcePlay) {
            return true;
        }
        let offer = self.state.outside_play.as_ref().unwrap();
        let actor = offer.player;
        let card = offer.object.card;
        let definition = self.state.objects[card as usize].card;
        let sorcery = self.state.active == actor
            && matches!(self.state.step, Step::Main1 | Step::Main2)
            && self.state.stack.is_empty();
        if definition.is_land() {
            return sorcery
                && (!self.state.land_played
                    || self
                        .continuous_land_plays(actor)
                        .checked_add(self.state.players[actor.index()].extra_land_plays_granted)
                        .capacity("land allowance exhausted")
                        > self.state.continuous_land_plays_used)
                && emit(Action::PlaySelectedOutsideLand);
        }
        if self.state.casting_prohibited
            || self.static_casting_prohibited(actor)
            || self.nonhand_cast_prohibited(actor)
        {
            return false;
        }
        for face in CastFace::choices(definition) {
            if !face.definition(definition).is_some_and(|d| {
                !d.is_land() && SourcePlayKind::GraveyardOrOutsidePermanent.accepts(d)
            }) {
                continue;
            }
            if self.visit_paid_cast_alternative_face(
                actor,
                card,
                face,
                sorcery,
                PaidCastOrigin::SourcePlay {
                    source: offer.source,
                    kind: SourcePlayKind::GraveyardOrOutsidePermanent,
                    outside: true,
                },
                cast_alternatives,
                emit,
            ) {
                return true;
            }
            for flash_surcharge in [false, true] {
                if self.permitted_cast_available(
                    actor,
                    card,
                    CastMethod::Normal,
                    face,
                    sorcery,
                    flash_surcharge,
                ) && emit(Action::AnnounceSelectedOutsidePermanent {
                    face,
                    flash_surcharge,
                }) {
                    return true;
                }
            }
        }
        false
    }

    pub(super) fn cancel_outside_source_play(&mut self) {
        let offer = self.state.outside_play.take().unwrap();
        assert_eq!(offer.object.card as usize + 1, self.state.objects.len());
        self.state.objects.pop();
        self.state.pending = Pending::Priority;
    }

    pub(super) fn commit_outside_source_play(&mut self) -> CardId {
        let offer = self.state.outside_play.take().unwrap();
        let card = self.state.outside_game.as_mut().unwrap().take_card(
            offer.player,
            offer.selection,
            false,
        );
        assert_eq!(self.state.objects[offer.object.card as usize].card, card);
        self.use_source_play(offer.source, SourcePlayKind::GraveyardOrOutsidePermanent);
        self.state.pending = Pending::Priority;
        offer.object.card
    }
}
