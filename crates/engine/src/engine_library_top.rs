//! Private CR401.5 operation boundaries and public, historical one-card reveals.
use super::*;
use crate::cards::{CastFilter, StaticAbilityFragment};

/// One exact static ability supplying a once-per-turn library-top cast.
#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct LibraryTopCastPermission {
    pub source: ObjectRef,
    pub definition: Card,
    pub ability: u16,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub copy_origin: Option<u64>,
}

#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
pub struct LibraryTopCastUse {
    pub player: Player,
    pub permission: LibraryTopCastPermission,
}

#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct LibraryRevealEvent {
    pub sequence: u64,
    pub source: ObjectRef,
    pub player: Player,
    pub revealed: RevealedLibraryCard,
    /// Historical result, not an assertion about the card's current zone.
    pub drawn: Option<ObjectRef>,
}
#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) struct VisibilityOperation {
    pub starting_top: [Option<ObjectRef>; 2],
    pub changed: [bool; 2],
    pub land: bool,
}
#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
type PrivateKnownTop = [Option<(ObjectRef, u8)>; 2];

#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub(super) struct LibraryTopState {
    pub operation: Option<VisibilityOperation>,
    pub reveal: Option<LibraryRevealEvent>,
    /// Private exact current-top knowledge, indexed by library owner.
    pub private_known_top: Option<Box<PrivateKnownTop>>,
    pub cast_uses: Vec<LibraryTopCastUse>,
}
#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
impl Game {
    pub(super) fn visit_limited_library_top_casts(
        &self,
        actor: Player,
        sorcery: bool,
        emit: &mut impl FnMut(Action) -> bool,
    ) -> bool {
        if self.state.static_sources.is_empty() {
            return false;
        }
        let Some(card) = self.top_library_card(actor) else {
            return false;
        };
        let top = &self.state.objects[card as usize];
        let physical = top.card;
        let faces = [
            (CastFace::Normal, Some(physical)),
            (CastFace::Adventure, physical.adventure_spell()),
            (CastFace::Omen, physical.omen_spell()),
            (CastFace::Back, physical.back_face()),
        ]
        .map(|(face, definition)| {
            (
                face,
                definition.filter(|definition| {
                    CastFilter::InstantSorcery.matches(*definition)
                        && !matches!(definition.effect(), Effect::Unsupported)
                }),
            )
        });
        if top.zone != Zone::Library
            || top.owner != actor
            || faces.iter().all(|(_, definition)| definition.is_none())
        {
            return false;
        }
        let mut evaluation = None;
        for &id in &self.state.static_sources {
            let provider = &self.state.objects[id as usize];
            if provider.zone != Zone::Battlefield || provider.controller != actor {
                continue;
            }
            let source = self.object_ref(id);
            for (ability, group) in provider.card.static_ability_groups().iter().enumerate() {
                if !group.fragments.iter().any(|fragment| {
                    matches!(
                        fragment,
                        StaticAbilityFragment::CastInstantSorceryFromLibraryTopOnceEachTurn
                    )
                }) || !entry_modes::group_enabled(self, source, group)
                {
                    continue;
                }
                let permission = LibraryTopCastPermission {
                    source,
                    definition: provider.card,
                    ability: u16::try_from(ability).expect("static ability index"),
                    copy_origin: self.copy_ability_origin(source),
                };
                if self.state.library_top.as_ref().is_some_and(|top| {
                    top.cast_uses.contains(&LibraryTopCastUse {
                        player: actor,
                        permission,
                    })
                }) || evaluation
                    .get_or_insert_with(|| self.evaluated_abilities())
                    .as_ref()
                    .is_some_and(|evaluation| !evaluation.group_available(source, ability))
                {
                    continue;
                }
                for (face, definition) in faces {
                    let Some(definition) = definition else {
                        continue;
                    };
                    for method in [CastMethod::Normal, CastMethod::Free] {
                        let method_available = match method {
                            CastMethod::Normal => !definition.info().mana_cost.is_empty(),
                            CastMethod::Free => {
                                self.filtered_free_cast_available(actor, definition)
                            }
                            _ => unreachable!("library-top casting method"),
                        };
                        if !method_available {
                            continue;
                        }
                        for flash_surcharge in [false, true] {
                            if self.permitted_cast_available(
                                actor,
                                card,
                                method,
                                face,
                                sorcery,
                                flash_surcharge,
                            ) && emit(Action::AnnounceLibraryTop {
                                object: self.object_ref(card),
                                face,
                                method,
                                flash_surcharge,
                                permission,
                            }) {
                                return true;
                            }
                        }
                    }
                    if self.visit_web_slinging_face(
                        actor,
                        card,
                        face,
                        sorcery,
                        WebSlingingOrigin::LibraryTop { permission },
                        emit,
                    ) {
                        return true;
                    }
                }
            }
        }
        false
    }

    pub(super) fn use_library_top_cast_permission(
        &mut self,
        player: Player,
        permission: LibraryTopCastPermission,
    ) {
        self.state
            .library_top
            .get_or_insert_default()
            .cast_uses
            .push(LibraryTopCastUse { player, permission });
    }

    pub(super) fn prepare_library_visibility(&mut self, action: &Action) {
        if self
            .state
            .library_top
            .as_ref()
            .is_some_and(|s| s.operation.is_some())
        {
            return;
        }
        if !self.state.players.iter().any(|p| p.library_top_life_play)
            && !self.state.objects.iter().any(|o| {
                o.card.static_ability_groups().iter().any(|g| {
                    g.fragments
                        .iter()
                        .any(|f| matches!(f, crate::cards::StaticAbilityFragment::LookAtLibraryTop))
                })
            })
        {
            return;
        }
        let starting_top = std::array::from_fn(|i| {
            self.state.players[i]
                .library
                .last()
                .map(|&id| self.object_ref(id))
        });
        self.state.library_top.get_or_insert_default().operation = Some(VisibilityOperation {
            starting_top,
            changed: [false; 2],
            land: matches!(
                action,
                Action::PlayLand { .. }
                    | Action::PlayTaxedExileLand { .. }
                    | Action::PlayGraveyardPermanent {
                        permanent_type: PermanentType::Land,
                        ..
                    }
            ),
        });
    }
    pub(super) fn finish_library_visibility(&mut self) {
        let ongoing = self.state.announcing.as_ref().is_some_and(|a| !a.trigger)
            || self.state.mana_announcement_stack.is_some()
            || self.plot_payment().is_some()
            || (self.state.entry_batch.is_some()
                && self
                    .state
                    .library_top
                    .as_ref()
                    .and_then(|s| s.operation.as_ref())
                    .is_some_and(|o| o.land));
        if !ongoing && let Some(s) = &mut self.state.library_top {
            s.operation = None;
            if s.reveal.is_none() && s.private_known_top.is_none() && s.cast_uses.is_empty() {
                self.state.library_top = None;
            }
        }
    }
    pub(super) fn remember_private_library_top(
        &mut self,
        owner: Player,
        viewer: Player,
        object: ObjectRef,
    ) {
        assert_eq!(self.top_library_card(owner), Some(object.card));
        assert_eq!(self.object_ref(object.card), object);
        let tops = self
            .state
            .library_top
            .get_or_insert_default()
            .private_known_top
            .get_or_insert_with(|| Box::new([None; 2]));
        let mask = 1 << viewer.index();
        if let Some((prior, viewers)) = &mut tops[owner.index()]
            && *prior == object
        {
            *viewers |= mask;
        } else {
            tops[owner.index()] = Some((object, mask));
        }
    }
    pub(super) fn private_library_top(&self, viewer: Player, owner: Player) -> Option<ObjectRef> {
        let (object, mask) = self
            .state
            .library_top
            .as_ref()?
            .private_known_top
            .as_ref()?[owner.index()]?;
        (mask & self.information_mask(viewer) != 0
            && self.top_library_card(owner) == Some(object.card)
            && self.object_ref(object.card) == object
            && self.state.objects[object.card as usize].zone == Zone::Library)
            .then_some(object)
    }
    pub(super) fn library_top_changed(&mut self, player: Player) {
        if let Some(state) = &mut self.state.library_top
            && let Some(tops) = &mut state.private_known_top
        {
            tops[player.index()] = None;
            if tops.iter().all(Option::is_none) {
                state.private_known_top = None;
            }
        }
        if let Some(o) = self
            .state
            .library_top
            .as_mut()
            .and_then(|s| s.operation.as_mut())
        {
            o.changed[player.index()] = true;
        }
    }
    pub(super) fn library_top_deferred(&self, player: Player) -> bool {
        self.state
            .library_top
            .as_ref()
            .and_then(|s| s.operation.as_ref())
            .is_some_and(|o| {
                o.changed[player.index()]
                    || o.starting_top[player.index()]
                        != self.state.players[player.index()]
                            .library
                            .last()
                            .map(|&id| self.object_ref(id))
            })
    }
    fn reveal_top_for(&mut self, item: &SpellView) -> Option<LibraryRevealEvent> {
        let player = item.controller;
        let id = self.top_library_card(player)?;
        let revealed = RevealedLibraryCard {
            object: self.object_ref(id),
            card: self.state.objects[id as usize].card,
        };
        self.remember_library_top(player, revealed.object);
        let sequence = self
            .state
            .library_top
            .as_ref()
            .and_then(|s| s.reveal.as_ref())
            .map_or(0, |e| e.sequence)
            .checked_add(1)
            .capacity("library reveal sequence exhausted");
        Some(LibraryRevealEvent {
            sequence,
            source: item.source.expect("library effect requires source").object,
            player,
            revealed,
            drawn: None,
        })
    }

    fn move_revealed_top_to_hand(&mut self, id: CardId) {
        let event = self.move_object(id, Zone::Hand);
        if event.moved && event.destination == Zone::Hand {
            self.remember_public_hand(event.after);
        }
    }

    pub(super) fn reveal_top_move_if_permanent(&mut self, item: &SpellView) {
        if let Some(reveal) = self.reveal_top_for(item) {
            let revealed = reveal.revealed;
            self.state.library_top.get_or_insert_default().reveal = Some(reveal);
            if revealed.card.kind_mask() & (1 | 2 | 16 | 32 | 64 | 128 | 256) != 0 {
                self.move_revealed_top_to_hand(revealed.object.card);
            }
        }
    }

    pub(super) fn reveal_top_to_hand_lose_mana_value(&mut self, item: &SpellView) {
        if let Some(reveal) = self.reveal_top_for(item) {
            let id = reveal.revealed.object.card;
            let amount = i128::from(self.object_mana_value(id));
            self.state.library_top.get_or_insert_default().reveal = Some(reveal);
            self.move_revealed_top_to_hand(id);
            self.lose_life(item.controller, amount);
        }
    }

    pub(super) fn reveal_target_tops_to_hand_lose_other_mana_value(&mut self, item: &SpellView) {
        let plan = self.target_spec(item);
        let legal = [Player::P0, Player::P1].map(|player| {
            item.targets.iter().any(|target| {
                target.target == Target::Player(player)
                    && plan.group(target.group).is_some_and(|group| {
                        self.item_target_matches(item, group.rule, target.target)
                    })
            })
        });
        let order = [self.state.active, self.state.active.other()];
        let tops: [Option<(ObjectRef, Card, u64)>; 2] = [Player::P0, Player::P1].map(|player| {
            if !legal[player.index()] {
                return None;
            }
            let id = self.top_library_card(player)?;
            Some((
                self.object_ref(id),
                self.state.objects[id as usize].card,
                self.object_mana_value(id),
            ))
        });
        for player in order {
            if let Some((object, card, _)) = tops[player.index()] {
                self.remember_library_top(player, object);
                self.record_zone_reveal(card, player, Zone::Library);
            }
        }
        self.lose_life_simultaneously(std::array::from_fn(|i| {
            if legal[i] {
                tops[1 - i].map_or(0, |(_, _, value)| i128::from(value))
            } else {
                0
            }
        }));
        let cards: Vec<_> = order
            .into_iter()
            .filter_map(|player| tops[player.index()].map(|(object, _, _)| object))
            .filter(|object| {
                self.object_ref(object.card) == *object
                    && self.state.objects[object.card as usize].zone == Zone::Library
            })
            .map(|object| object.card)
            .collect();
        let events = self.move_objects_raw(&cards, Zone::Hand);
        self.finish_zone_events(&events, &[]);
    }

    pub(super) fn reveal_top_draw_if_artifact(&mut self, item: &SpellView) {
        let Some(mut event) = self.reveal_top_for(item) else {
            return;
        };
        let id = event.revealed.object.card;
        if event.revealed.card.kind_mask() & 16 != 0 {
            self.draw_many(event.player, 1);
            let result = self.object_ref(id);
            if result != event.revealed.object && self.state.objects[id as usize].zone == Zone::Hand
            {
                event.drawn = Some(result);
                self.remember_public_hand(result);
            }
        }
        // Reveal is public even if this ordinary draw completes within one action.
        self.state.library_top.get_or_insert_default().reveal = Some(event);
    }
    pub(super) fn exile_top_play_boost_source(
        &mut self,
        item: &SpellView,
        subtype: &str,
        power: i32,
        toughness: i32,
    ) {
        let Some(top) = self.top_library_card(item.controller) else {
            return;
        };
        let events = self.move_objects_raw(&[top], Zone::Exile);
        for event in &events {
            if event.moved && event.destination == Zone::Exile {
                self.grant_exile_play_permission(item.controller, event.after, false);
                if self.state.objects[event.after.card as usize]
                    .card
                    .has_subtype(subtype)
                    && self.on_battlefield(item.source.expect("library effect requires source").id)
                    && self.object_ref(item.source.expect("library effect requires source").id)
                        == item.source.expect("library effect requires source").object
                {
                    self.state.objects
                        [item.source.expect("library effect requires source").id as usize]
                        .growth += Stats {
                        power: i128::from(power),
                        toughness: i128::from(toughness),
                    };
                }
            }
        }
        self.finish_zone_events(&events, &[]);
    }
}
