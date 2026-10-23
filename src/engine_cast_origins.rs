//! Live casting-origin and mana-type permissions, shared by real and funding worlds.
use super::*;
use crate::cards::StaticAbilityFragment;
impl Card {
    pub(super) fn has_cast_origin_abilities(self) -> bool {
        self.static_ability_groups().iter().any(|g| {
            g.fragments.iter().any(|f| {
                matches!(
                    f,
                    StaticAbilityFragment::LookAtLibraryTop
                        | StaticAbilityFragment::CastCreatureFromLibraryTop
                        | StaticAbilityFragment::PlayLibraryTopSubtypeLandsWithCounter(..)
                        | StaticAbilityFragment::SpendAnyManaForCreatures
                        | StaticAbilityFragment::GraveyardPermanentTypes
                        | StaticAbilityFragment::GraveyardSmallCreaturesWithFinality
                        | StaticAbilityFragment::GrantGraveyardCreatureSneak { .. }
                        | StaticAbilityFragment::CastGraveyardCreaturesUsingSneak { .. }
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
    world.game().state.static_sources.iter().any(|&id| {
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
pub(super) fn spends_any_type<W: ability_world::CharacteristicWorld + ?Sized>(
    world: &W,
    player: Player,
    purpose: mana_lots::Purpose,
) -> bool {
    matches!(purpose, mana_lots::Purpose::Spell(source) if source.exile_any_mana)
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
    pub(super) fn can_cast_library_top(&self, player: Player, card: CardId) -> bool {
        let object = &self.state.objects[card as usize];
        object.owner == player
            && object.zone == Zone::Library
            && self.state.players[player.index()].library.last() == Some(&card)
            && has_fragment(self, player, |f| match *f {
                StaticAbilityFragment::CastCreatureFromLibraryTop => object.card.is_creature(),
                StaticAbilityFragment::PlayLibraryTopSubtypeLandsWithCounter(types) => {
                    object.card.is_land() || types.iter().any(|s| object.card.has_subtype(s))
                }
                _ => false,
            })
    }
    pub(super) fn visible_library_top(&self, viewer: Player, owner: Player) -> Option<ObjectRef> {
        let next_top_hidden = self.state.announcing.as_ref().is_some_and(|a| {
            a.item.cast.is_some_and(|c| c.origin == Zone::Library) && a.item.controller == owner
        });
        if viewer == owner
            && !next_top_hidden
            && has_fragment(self, viewer, |f| {
                matches!(f, StaticAbilityFragment::LookAtLibraryTop)
            })
        {
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

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct ResolutionFreeCasts {
    pub player: Player,
    pub remaining: Vec<ObjectRef>,
    pub maximum_mana_value: Option<u128>,
    pub uncast_to_hand: bool,
    pub remaining_casts: Option<u64>,
}
impl Game {
    pub(super) fn exile_nonland_life_free_cast(&mut self, player: Player) -> bool {
        let cards: Vec<_> = self.state.players[player.index()]
            .library
            .iter()
            .rev()
            .copied()
            .collect();
        for id in cards {
            if self.state.objects[id as usize].zone != Zone::Library {
                continue;
            }
            let event = self.move_object(id, Zone::Exile);
            if event.moved && event.destination == Zone::Exile && !event.before.card.is_land() {
                self.state.resolution_free_casts = Some(Box::new(ResolutionFreeCasts {
                    player,
                    remaining: vec![event.after],
                    maximum_mana_value: Some(
                        self.state.players[player.index()].life_gained_this_turn,
                    ),
                    uncast_to_hand: true,
                    remaining_casts: None,
                }));
                self.state.pending = Pending::ResolutionCasts { player };
                return true;
            }
        }
        false
    }
    pub(super) fn finish_resolution_free_casts(&mut self) {
        let free = self.state.resolution_free_casts.take().unwrap();
        if free.uncast_to_hand {
            for object in free.remaining {
                if self.object_ref(object.card) == object
                    && self.state.objects[object.card as usize].zone == Zone::Exile
                {
                    let event = self.move_object(object.card, Zone::Hand);
                    if event.moved
                        && event.destination == Zone::Hand
                        && !self.state.known_hand.contains(&event.after)
                    {
                        self.state.known_hand.push(event.after);
                    }
                }
            }
        }
        self.run_program();
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
            player,
            remaining,
            maximum_mana_value: None,
            uncast_to_hand: false,
            remaining_casts: None,
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
        if self.state.casting_prohibited || self.static_casting_prohibited(player) {
            return false;
        }
        let Some(free) = &self.state.resolution_free_casts else {
            return false;
        };
        for &object in &free.remaining {
            if self.object_ref(object.card) != object
                || self.state.objects[object.card as usize].zone != Zone::Exile
            {
                continue;
            }
            for face in [CastFace::Normal, CastFace::Adventure, CastFace::Omen] {
                let physical = self.state.objects[object.card as usize].card;
                let definition = match face {
                    CastFace::Normal => Some(physical),
                    CastFace::Adventure => physical.adventure_spell(),
                    CastFace::Omen => physical.omen_spell(),
                };
                if definition.is_some_and(|card| {
                    !card.is_land()
                        && free
                            .maximum_mana_value
                            .is_none_or(|maximum| u128::from(card.fixed_mana_value()) <= maximum)
                }) && self.permitted_cast_available(
                    player,
                    object.card,
                    CastMethod::Free,
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
        let free = self.state.resolution_free_casts.as_mut().unwrap();
        free.remaining.retain(|&r| r != object);
        if let Some(count) = &mut free.remaining_casts {
            *count = count.checked_sub(1).expect("free casting allowance");
            if *count == 0 {
                free.remaining.clear();
            }
        }
        let mut item = self.cast_item(player, object.card, [0; 6]);
        self.set_cast_face(&mut item, face);
        item.cast.as_mut().unwrap().method = CastMethod::Free;
        if item.source_card.x_symbols() > 0 {
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
    /// Static permission and alternative cost are separate choices (CR118.9).
    pub(super) fn visit_graveyard_spell_permissions(
        &self,
        actor: Player,
        sorcery: bool,
        emit: &mut impl FnMut(Action) -> bool,
    ) -> bool {
        if actor != self.state.active {
            return false;
        }
        for &source in &self.state.static_sources {
            let provider = &self.state.objects[source as usize];
            if provider.zone != Zone::Battlefield
                || provider.controller != actor
                || !self.printed_abilities_available(source)
            {
                continue;
            }
            let fragments = || {
                provider
                    .card
                    .static_ability_groups()
                    .iter()
                    .flat_map(|g| g.fragments)
            };
            let small = fragments().any(|f| {
                matches!(
                    f,
                    StaticAbilityFragment::GraveyardSmallCreaturesWithFinality
                )
            });
            let typed =
                fragments().any(|f| matches!(f, StaticAbilityFragment::GraveyardPermanentTypes));
            let ninja=fragments().any(|f|matches!(f,StaticAbilityFragment::CastGraveyardCreaturesUsingSneak {minimum_class_level} if self.class_level(self.object_ref(source))>=*minimum_class_level));
            if !small && !typed && !ninja {
                continue;
            }
            let permission = self.object_ref(source);
            for (i, object) in self.state.objects.iter().enumerate() {
                if object.zone != Zone::Graveyard
                    || object.owner != actor
                    || object.is_token()
                    || !object.card.is_supported()
                    || object.card.is_land()
                {
                    continue;
                }
                let card = i as CardId;
                let stats = self.stats(card);
                let small_eligible = small
                    && object.card.is_creature()
                    && (stats.power <= 1 || stats.toughness <= 1);
                for permanent_type in
                    std::iter::once(None).chain(PermanentType::ALL.into_iter().map(Some))
                {
                    match permanent_type {
                        None if !small_eligible && !(ninja && object.card.is_creature()) => {
                            continue;
                        }
                        Some(kind)
                            if !typed
                                || kind == PermanentType::Land
                                || object.card.kind_mask() & kind.mask() == 0
                                || self.state.graveyard_type_uses.as_ref().is_some_and(
                                    |rows| {
                                        rows.iter().any(|row| {
                                            row.source == permission
                                                && row.turn == self.state.turn
                                                && row.permanent_type == kind
                                        })
                                    },
                                ) =>
                        {
                            continue;
                        }
                        _ => {}
                    }
                    for flash_surcharge in [false, true] {
                        if flash_surcharge && object.card.flash_surcharge().is_none() {
                            continue;
                        }
                        for method in [CastMethod::Normal, CastMethod::Warp] {
                            if (permanent_type.is_none() && !small_eligible)
                                || method == CastMethod::Normal && permanent_type.is_some()
                                || method == CastMethod::Warp && object.card.warp_cost().is_none()
                            {
                                continue;
                            }
                            if self.permitted_cast_available(
                                actor,
                                card,
                                method,
                                CastFace::Normal,
                                sorcery,
                                flash_surcharge,
                            ) && emit(Action::AnnounceGraveyardSpell {
                                card,
                                permission,
                                permanent_type,
                                method,
                                attacker: None,
                                flash_surcharge,
                            }) {
                                return true;
                            }
                        }
                        if self.visit_sneak_candidates(
                            actor,
                            card,
                            flash_surcharge,
                            &mut |attacker| {
                                emit(Action::AnnounceGraveyardSpell {
                                    card,
                                    permission,
                                    permanent_type,
                                    method: CastMethod::Sneak,
                                    attacker: Some(attacker),
                                    flash_surcharge,
                                })
                            },
                        ) {
                            return true;
                        }
                    }
                }
            }
        }
        false
    }
}
