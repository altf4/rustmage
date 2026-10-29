//! Live casting-origin and mana-type permissions, shared by real and funding worlds.
use super::*;
use crate::cards::StaticAbilityFragment;
fn is_false(value: &bool) -> bool {
    !*value
}
impl Card {
    pub(super) fn has_cast_origin_abilities(self) -> bool {
        self.static_ability_groups().iter().any(|g| {
            g.fragments.iter().any(|f| {
                matches!(
                    f,
                    StaticAbilityFragment::HandMiracle { .. }
                        | StaticAbilityFragment::CascadeFromHand { .. }
                        | StaticAbilityFragment::LookAtLibraryTop
                        | StaticAbilityFragment::CastCreatureFromLibraryTop
                        | StaticAbilityFragment::PlayLandFromLibraryTop
                        | StaticAbilityFragment::PlayLandFromGraveyard
                        | StaticAbilityFragment::SpendAnyManaForCreatures
                        | StaticAbilityFragment::GraveyardPermanentTypes
                        | StaticAbilityFragment::ForeignStashPlay
                )
            })
        })
    }
}
fn has_fragment<W: ability_world::CharacteristicWorld + ?Sized>(
    world: &W,
    player: Player,
    predicate: impl Fn(&StaticAbilityFragment) -> bool,
) -> bool {
    world.static_sources().iter().any(|&id| {
        let object = world.object(id);
        object.zone == Zone::Battlefield
            && object.controller == player
            && object
                .card
                .static_ability_groups()
                .iter()
                .any(|g| g.fragments.iter().any(&predicate))
            && world.printed_abilities_available(id)
    })
}
/// A live rule changes only the origin of an ordinary land play. It never grants
/// a spell cast, a per-source allowance, or an action during mana payment.
pub(super) fn land_origin_available<W: ability_world::CharacteristicWorld + ?Sized>(
    world: &W,
    player: Player,
    origin: Zone,
) -> bool {
    has_fragment(world, player, |f| {
        matches!(
            (origin, f),
            (Zone::Library, StaticAbilityFragment::PlayLandFromLibraryTop)
                | (
                    Zone::Graveyard,
                    StaticAbilityFragment::PlayLandFromGraveyard
                )
        )
    })
}
pub(super) fn spends_any_type<W: ability_world::CharacteristicWorld + ?Sized>(
    world: &W,
    player: Player,
    purpose: mana_lots::Purpose,
) -> bool {
    matches!(purpose, mana_lots::Purpose::Spell(source) if source.spend_any_mana
        || source.linked_mana_source.is_some_and(|linked|
            exile::threshold_linked_source_available(world, player, linked)))
        || (matches!(purpose, mana_lots::Purpose::Spell(source) if source.kinds & 1 != 0)
            && has_fragment(world, player, |f| {
                matches!(f, StaticAbilityFragment::SpendAnyManaForCreatures)
            }))
        || (matches!(purpose, mana_lots::Purpose::Spell(source) if source.foreign_stash)
            && world.game().state.active == player
            && has_fragment(world, player, |f| {
                matches!(f, StaticAbilityFragment::ForeignStashPlay)
            }))
}
pub(super) fn payment_requirements<W: ability_world::CharacteristicWorld + ?Sized>(
    world: &W,
    player: Player,
    purpose: mana_lots::Purpose,
    required: Mana,
    generic: u32,
) -> (Mana, u32) {
    if spends_any_type(world, player, purpose) {
        (
            [0; 6],
            required
                .into_iter()
                .try_fold(generic, u32::checked_add)
                .capacity("mana substitution cost exhausted"),
        )
    } else {
        (required, generic)
    }
}
pub(super) fn payment_goal<W: ability_world::CharacteristicWorld + ?Sized>(
    world: &W,
    goal: &FundingGoal,
    purpose: mana_lots::Purpose,
) -> FundingGoal {
    let (required, generic) =
        payment_requirements(world, goal.actor, purpose, goal.required, goal.generic);
    FundingGoal {
        required,
        generic,
        ..goal.clone()
    }
}
impl Game {
    pub(super) fn can_play_static_land(&self, player: Player, card: CardId) -> bool {
        let object = &self.state.objects[card as usize];
        object.owner == player
            && object.card.is_land()
            && !object.is_token()
            && land_origin_available(self, player, object.zone)
            && (object.zone != Zone::Library
                || self.state.players[player.index()].library.last() == Some(&card))
    }
    pub(super) fn can_cast_library_top(&self, player: Player, card: CardId) -> bool {
        self.can_cast_library_face(player, card, self.state.objects[card as usize].card)
    }
    pub(super) fn can_cast_library_face(
        &self,
        player: Player,
        card: CardId,
        definition: Card,
    ) -> bool {
        let object = &self.state.objects[card as usize];
        object.owner == player
            && object.zone == Zone::Library
            && definition.is_creature()
            && self.state.players[player.index()].library.last() == Some(&card)
            && has_fragment(self, player, |f| {
                matches!(f, StaticAbilityFragment::CastCreatureFromLibraryTop)
            })
    }
    pub(super) fn can_look_library_top(&self, player: Player) -> bool {
        has_fragment(self, player, |f| {
            matches!(f, StaticAbilityFragment::LookAtLibraryTop)
        })
    }
    pub(super) fn visible_library_top(&self, viewer: Player, owner: Player) -> Option<ObjectRef> {
        if self.library_top_deferred(owner) {
            return None;
        }
        if self.shares_information(viewer, owner) && self.can_look_library_top(owner) {
            self.state.players[owner.index()]
                .library
                .last()
                .map(|&id| self.object_ref(id))
        } else {
            self.public_library_top(owner)
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
pub struct GraveyardTypeUse {
    pub source: ObjectRef,
    pub turn: u32,
    pub permanent_type: PermanentType,
}
impl Game {
    pub(super) fn visit_graveyard_permanent_plays(
        &self,
        actor: Player,
        sorcery: bool,
        casting_prohibited: bool,
        emit: &mut impl FnMut(Action) -> bool,
    ) -> bool {
        if actor != self.state.active {
            return false;
        }
        for &id in &self.state.static_sources {
            let object = &self.state.objects[id as usize];
            if object.zone != Zone::Battlefield
                || object.controller != actor
                || !object.card.static_ability_groups().iter().any(|g| {
                    g.fragments
                        .iter()
                        .any(|f| matches!(f, StaticAbilityFragment::GraveyardPermanentTypes))
                })
                || !self.printed_abilities_available(id)
            {
                continue;
            }
            let permission = self.object_ref(id);
            for (i, object) in self.state.objects.iter().enumerate() {
                if object.zone != Zone::Graveyard || object.owner != actor || object.is_token() {
                    continue;
                }
                let card = i as CardId;
                for permanent_type in PermanentType::ALL {
                    if object.card.kind_mask() & permanent_type.mask() == 0
                        || self.state.graveyard_type_uses.as_ref().is_some_and(|rows| {
                            rows.iter().any(|row| {
                                row.source == permission
                                    && row.turn == self.state.turn
                                    && row.permanent_type == permanent_type
                            })
                        })
                    {
                        continue;
                    }
                    let available = if permanent_type == PermanentType::Land {
                        sorcery
                            && (!self.state.land_played
                                || self
                                    .continuous_land_plays(actor)
                                    .checked_add(
                                        self.state.players[actor.index()].extra_land_plays_granted,
                                    )
                                    .capacity("land allowance exhausted")
                                    > self.state.continuous_land_plays_used)
                    } else {
                        !object.card.is_land()
                            && !casting_prohibited
                            && self.permitted_cast_available(
                                actor,
                                card,
                                CastMethod::Normal,
                                CastFace::Normal,
                                sorcery,
                                false,
                            )
                    };
                    if permanent_type == PermanentType::Creature
                        && !casting_prohibited
                        && (sorcery
                            || self.cast_timing_granted(
                                actor,
                                Card::from_name("@face-down/disguise")
                                    .expect("pinned catalogue identity"),
                            ))
                        && self.disguise_mana_fundable(actor, card)
                        && emit(Action::AnnounceDisguiseGraveyard {
                            card,
                            permission,
                            permanent_type,
                        })
                    {
                        return true;
                    }
                    if permanent_type != PermanentType::Land
                        && !object.card.is_land()
                        && !casting_prohibited
                        && object.card.sneak_cost().is_some()
                        && self.state.step == Step::Blockers
                        && self.permitted_cast_available(
                            actor,
                            card,
                            CastMethod::Sneak,
                            CastFace::Normal,
                            false,
                            false,
                        )
                        && emit(Action::AnnounceSneak {
                            card,
                            permission: Some((permission, permanent_type)),
                        })
                    {
                        return true;
                    }
                    if available
                        && emit(Action::PlayGraveyardPermanent {
                            card,
                            permission,
                            permanent_type,
                        })
                    {
                        return true;
                    }
                }
            }
        }
        false
    }
    pub(super) fn consume_graveyard_type(
        &mut self,
        source: ObjectRef,
        permanent_type: PermanentType,
    ) {
        let turn = self.state.turn;
        let rows = self
            .state
            .graveyard_type_uses
            .get_or_insert_with(Default::default);
        rows.retain(|row| row.turn == turn);
        rows.try_reserve(1)
            .capacity("graveyard permission use allocation exhausted");
        rows.push(GraveyardTypeUse {
            source,
            turn,
            permanent_type,
        });
    }
}

impl Game {
    pub(super) fn foreign_stash(&self, player: Player, card: CardId) -> bool {
        let object = &self.state.objects[card as usize];
        object.zone == Zone::Exile
            && object.owner != player
            && self.counter_count(card, CounterKind::Stash) > 0
    }
    pub(super) fn can_play_foreign_stash(&self, player: Player, card: CardId) -> bool {
        self.state.active == player
            && self.foreign_stash(player, card)
            && has_fragment(self, player, |f| {
                matches!(f, StaticAbilityFragment::ForeignStashPlay)
            })
    }
    pub(super) fn exile_discarded_with_stash(&mut self, item: &SpellView) {
        let Some(subject) = &item.reflexive_subject else {
            return;
        };
        let object = subject.object;
        if self.object_ref(object.card) != object
            || self.state.objects[object.card as usize].zone != Zone::Graveyard
        {
            return;
        }
        let event = self.move_object(object.card, Zone::Exile);
        if event.moved && event.destination == Zone::Exile {
            self.effect_counter(
                event.after.card,
                CounterKind::Stash,
                1,
                replacements::ResumeMode::Program,
            );
        }
    }
}

/// Printed-cost casting permission; kinds are checked against the chosen spell face.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
pub struct ResolutionPaidCast {
    pub kinds: u8,
    pub exile_instants_sorceries: bool,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct ResolutionFreeCasts {
    #[serde(skip_serializing_if = "is_false")]
    pub siege_transformed: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub paid: Option<ResolutionPaidCast>,
    /// Successful spells still permitted by this immediate casting instruction.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub casts_remaining: Option<u16>,
    /// Exact proposed child; only its committed cast consumes a quota.
    #[serde(skip)]
    pub(super) pending_cast: Option<u64>,
    /// Inclusive total spell mana value remaining for this free-cast instruction.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub remaining_mana_value: Option<u64>,
    pub alternative_cost: Option<ManaCost>,
    pub maximum_mana_value: Option<u64>,
    pub origin: Zone,
    pub exile_on_graveyard: bool,
    pub allow_lands: bool,
    pub player: Player,
    pub remaining: Vec<ObjectRef>,
}
impl Game {
    pub(super) fn exile_top_offer_paid_cast(&mut self, player: Player) -> bool {
        let Some(card) = self.state.players[player.index()].library.last().copied() else {
            return false;
        };
        let event = self.move_object(card, Zone::Exile);
        if !event.moved || event.destination != Zone::Exile {
            return false;
        }
        self.state.resolution_free_casts = Some(Box::new(ResolutionFreeCasts {
            siege_transformed: false,
            paid: Some(ResolutionPaidCast {
                kinds: u8::MAX,
                exile_instants_sorceries: false,
            }),
            casts_remaining: Some(1),
            pending_cast: None,
            remaining_mana_value: None,
            alternative_cost: None,
            maximum_mana_value: None,
            origin: Zone::Exile,
            exile_on_graveyard: false,
            allow_lands: false,
            player,
            remaining: vec![event.after],
        }));
        self.state.pending = Pending::ResolutionCasts { player };
        true
    }
    pub(super) fn offer_paid_graveyard_cast(
        &mut self,
        item: &SpellView,
        kinds: u8,
        exile_instants_sorceries: bool,
    ) {
        // Keep all exact cards: legality depends on the chosen face, not the
        // physical front. Ineligible cards produce no legal cast action.
        let remaining = self
            .state
            .objects
            .iter()
            .enumerate()
            .filter(|(_, o)| {
                o.owner == item.controller && o.zone == Zone::Graveyard && !o.is_token()
            })
            .map(|(id, _)| self.object_ref(id as CardId))
            .collect();
        self.state.resolution_free_casts = Some(Box::new(ResolutionFreeCasts {
            siege_transformed: false,
            paid: Some(ResolutionPaidCast {
                kinds,
                exile_instants_sorceries,
            }),
            casts_remaining: Some(1),
            pending_cast: None,
            remaining_mana_value: None,
            alternative_cost: None,
            maximum_mana_value: None,
            origin: Zone::Graveyard,
            exile_on_graveyard: false,
            allow_lands: false,
            player: item.controller,
            remaining,
        }));
        self.state.pending = Pending::ResolutionCasts {
            player: item.controller,
        };
    }
    pub(super) fn offer_targeted_graveyard_free_cast(&mut self, item: &SpellView) -> bool {
        let remaining: Vec<_> = item
            .targets
            .iter()
            .filter_map(|bound| {
                bound.object.filter(|object| {
                    self.same_target_object(*bound)
                        && self.state.objects[object.card as usize].zone == Zone::Graveyard
                })
            })
            .collect();
        if remaining.is_empty() {
            return false;
        }
        self.state.resolution_free_casts = Some(Box::new(ResolutionFreeCasts {
            siege_transformed: false,
            paid: None,
            casts_remaining: None,
            pending_cast: None,
            remaining_mana_value: None,
            alternative_cost: None,
            maximum_mana_value: None,
            origin: Zone::Graveyard,
            exile_on_graveyard: true,
            allow_lands: false,
            player: item.controller,
            remaining,
        }));
        self.state.pending = Pending::ResolutionCasts {
            player: item.controller,
        };
        true
    }
    pub(super) fn exile_each_top_cast_free(&mut self, player: Player) {
        let cards: Vec<_> = [self.state.active, self.state.active.other()]
            .into_iter()
            .filter_map(|p| self.state.players[p.index()].library.last().copied())
            .collect();
        let events = self.move_objects_raw(&cards, Zone::Exile);
        let remaining = events
            .iter()
            .filter(|event| event.moved && event.destination == Zone::Exile)
            .map(|event| event.after)
            .collect();
        self.finish_zone_events(&events, &[]);
        self.state.resolution_free_casts = Some(Box::new(ResolutionFreeCasts {
            siege_transformed: false,
            paid: None,
            casts_remaining: None,
            pending_cast: None,
            remaining_mana_value: None,
            alternative_cost: None,
            maximum_mana_value: None,
            origin: Zone::Exile,
            exile_on_graveyard: false,
            player,
            remaining,
            allow_lands: false,
        }));
        self.state.pending = Pending::ResolutionCasts { player };
    }
    pub(super) fn visit_resolution_casts(
        &self,
        player: Player,
        emit: &mut impl FnMut(Action) -> bool,
    ) -> bool {
        if emit(Action::FinishResolutionCasts) {
            return true;
        }
        let casting_prohibited =
            self.state.casting_prohibited || self.static_casting_prohibited(player);
        let Some(free) = &self.state.resolution_free_casts else {
            return false;
        };
        // The cumulative budget models casting without paying mana costs.
        // Paid and alternative-cost offers can choose nonzero X and need a
        // separate announced-value legality check before they can use it.
        if free.casts_remaining == Some(0)
            || (free.remaining_mana_value.is_some()
                && (free.paid.is_some() || free.alternative_cost.is_some()))
        {
            return false;
        }
        for &object in &free.remaining {
            if self.object_ref(object.card) != object
                || self.state.objects[object.card as usize].zone != free.origin
            {
                continue;
            }
            let physical = self.state.objects[object.card as usize].card;
            if free.allow_lands
                && physical.is_land()
                && self.state.active == player
                && (!self.state.land_played
                    || self
                        .continuous_land_plays(player)
                        .checked_add(self.state.players[player.index()].extra_land_plays_granted)
                        .capacity("land allowance exhausted")
                        > self.state.continuous_land_plays_used)
                && emit(Action::PlayLand { card: object.card })
            {
                return true;
            }
            if casting_prohibited {
                continue;
            }
            // A paid permission permits a card's alternative disguise cost.
            // Disguise is a creature spell with mana value zero regardless of
            // the printed face; free-cast permissions cannot pay this cost.
            if free
                .paid
                .is_some_and(|paid| paid.kinds == u8::MAX || paid.kinds & 1 != 0)
                && !free.siege_transformed
                && free.maximum_mana_value.is_none_or(|maximum| maximum > 0)
                && self.disguise_mana_fundable(player, object.card)
                && emit(Action::AnnounceDisguise { card: object.card })
            {
                return true;
            }
            for face in [
                CastFace::Normal,
                CastFace::Adventure,
                CastFace::Omen,
                CastFace::Back,
            ] {
                if free.siege_transformed && face != CastFace::Back {
                    continue;
                }
                let physical = self.state.objects[object.card as usize].card;
                let definition = match face {
                    CastFace::Normal => Some(physical),
                    CastFace::Adventure => physical.adventure_spell(),
                    CastFace::Omen => physical.omen_spell(),
                    CastFace::Back => physical.back_face(),
                };
                if definition.is_some_and(|card| {
                    !card.is_land()
                        && free.paid.is_none_or(|paid| {
                            paid.kinds == u8::MAX || card.kind_mask() & u16::from(paid.kinds) != 0
                        })
                        && free
                            .maximum_mana_value
                            .is_none_or(|maximum| card.fixed_mana_value() < maximum)
                        && free
                            .remaining_mana_value
                            .is_none_or(|left| card.fixed_mana_value() <= left)
                }) && self.permitted_cast_available(
                    player,
                    object.card,
                    if free.paid.is_some() {
                        CastMethod::Normal
                    } else if free.alternative_cost.is_some() {
                        CastMethod::Miracle
                    } else {
                        CastMethod::Free
                    },
                    face,
                    true,
                    false,
                ) && emit(Action::CastExiledDuringResolution { object, face })
                {
                    return true;
                }
            }
        }
        false
    }
    pub(super) fn cast_exiled_during_resolution(
        &mut self,
        player: Player,
        object: ObjectRef,
        face: CastFace,
    ) {
        let paid = self.state.resolution_free_casts.as_ref().unwrap().paid;
        let alternative_cost = self
            .state
            .resolution_free_casts
            .as_ref()
            .unwrap()
            .alternative_cost;
        let exile_on_graveyard = self
            .state
            .resolution_free_casts
            .as_ref()
            .unwrap()
            .exile_on_graveyard;
        self.state
            .resolution_free_casts
            .as_mut()
            .unwrap()
            .remaining
            .retain(|&r| r != object);
        self.reveal_hideaway_object(object);
        let mut item = self.cast_item(player, object.card, [0; 6]);
        if let Some(free) = self.state.resolution_free_casts.as_mut()
            && (free.casts_remaining.is_some() || free.remaining_mana_value.is_some())
        {
            free.pending_cast = Some(item.id);
        }
        self.set_cast_face(&mut item, face);
        item.cast.as_mut().unwrap().method = if paid.is_some() {
            CastMethod::Normal
        } else if alternative_cost.is_some() {
            CastMethod::Miracle
        } else {
            CastMethod::Free
        };
        item.cast.as_mut().unwrap().exile_on_graveyard = exile_on_graveyard
            || paid.is_some_and(|p| {
                p.exile_instants_sorceries && item.source_card.kind_mask() & (4 | 8) != 0
            });
        if paid.is_none() && item.source_card.x_symbols() > 0 {
            item.x = Some(0);
        }
        self.open_payment_announcement(item, Counters::default(), None);
    }
}

impl Game {
    pub(super) fn forget_graveyard_type_source(&mut self, source: ObjectRef) {
        if !self
            .state
            .graveyard_type_uses
            .as_ref()
            .is_some_and(|rows| rows.iter().any(|row| row.source == source))
        {
            return;
        }
        let rows = self.state.graveyard_type_uses.as_mut().unwrap();
        rows.retain(|row| row.source != source);
        if rows.is_empty() {
            self.state.graveyard_type_uses = None;
        }
    }
}

impl Game {
    pub(super) fn consume_resolution_cast_quota(&mut self, player: Player, stack: Option<usize>) {
        let Some(item) = stack.map(|i| &self.state.stack[i]) else {
            return;
        };
        let (id, value) = (item.id, item.mana_value());
        if let Some(free) = self.state.resolution_free_casts.as_mut()
            && free.player == player
            && free.pending_cast == Some(id)
        {
            if let Some(left) = &mut free.casts_remaining {
                *left = left.checked_sub(1).expect("a legal quota cast");
            }
            if let Some(left) = &mut free.remaining_mana_value {
                *left = left.checked_sub(value).expect("a legal budget cast");
            }
            free.pending_cast = None;
        }
    }
    pub(super) fn exile_target_top_cast_free(
        &mut self,
        item: &SpellView,
        amount: u16,
        maximum_casts: u16,
    ) -> bool {
        let target = item
            .targets
            .iter()
            .filter(|b| {
                self.target_spec(item)
                    .group(b.group)
                    .is_some_and(|spec| self.item_target_matches(item, spec.rule, b.target))
            })
            .find_map(|b| {
                if let Target::Player(p) = b.target {
                    Some(p)
                } else {
                    None
                }
            });
        let Some(owner) = target else {
            return false;
        };
        let ids = exile::top_library_cards(
            &self.state.players[owner.index()].library,
            u128::from(amount),
        );
        let events = self.move_objects_raw(&ids, Zone::Exile);
        let remaining = events
            .iter()
            .filter(|e| e.moved && e.destination == Zone::Exile)
            .map(|e| e.after)
            .collect();
        self.finish_zone_events(&events, &[]);
        self.state.resolution_free_casts = Some(Box::new(ResolutionFreeCasts {
            siege_transformed: false,
            paid: None,
            casts_remaining: Some(maximum_casts),
            pending_cast: None,
            remaining_mana_value: None,
            alternative_cost: None,
            maximum_mana_value: None,
            origin: Zone::Exile,
            exile_on_graveyard: false,
            allow_lands: false,
            player: item.controller,
            remaining,
        }));
        self.state.pending = Pending::ResolutionCasts {
            player: item.controller,
        };
        true
    }
}
