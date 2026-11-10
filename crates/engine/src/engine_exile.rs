//! Face-up exile and the paired one-shot returns of CR610.3.
use super::*;
/// Independent paid play permission for one exact face-up exile incarnation.
/// The deadline is inclusive through priority in that turn, and expires during
/// cleanup alongside other until-end-of-turn effects (CR514.2).
#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
pub struct ExilePlayPermission {
    pub object: ObjectRef,
    pub player: Player,
    /// Exact permanent that supplies a source-dependent permission or lore history.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source: Option<ObjectRef>,
    pub expiry: ExilePlayExpiry,
    pub spend_any_mana: bool,
    /// A cast permission never authorizes a land play.
    #[serde(skip_serializing_if = "std::ops::Not::not")]
    pub cast_only: bool,
    pub exile_on_graveyard: bool,
    /// This permission requires life instead of the chosen spell face's mana cost.
    #[serde(skip_serializing_if = "std::ops::Not::not")]
    pub life_instead_of_mana: bool,
    /// This origin permission requires the free alternative, not a paid cast.
    #[serde(skip_serializing_if = "std::ops::Not::not")]
    pub without_paying_mana_cost: bool,
    /// A distinct permission to cast for {2} instead of its mana cost.
    #[serde(skip_serializing_if = "std::ops::Not::not")]
    pub airbend: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub requires_subtype: Option<&'static str>,
    /// A distinct play permission: charge this generic fee for spells, or
    /// put a land played via this permission onto the battlefield tapped.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub play_surcharge: Option<std::num::NonZeroU32>,
}
/// Deadline boundaries refer to actual turns, never to skipped turn candidates.
#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
pub enum ExilePlayExpiry {
    WhileExiled,
    Turn(u32),
    EndOfNextTurn,
    StartOfNextTurn(Player),
    StartOfNextEndStep(Player),
    /// Replaced by the same player's next actual exile with this exact source.
    UntilSourceExilesAnother(ObjectRef),
    /// Active in turns with a positive lore placement by the granted player on the source.
    SourceLoreTurns,
}
#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
impl ExilePlayExpiry {
    pub(super) fn active(self, turn: u32) -> bool {
        match self {
            Self::Turn(deadline) => deadline >= turn,
            _ => true,
        }
    }
}
/// Exact top-card selection, bounded only by the finite library's length.
/// Keep the full rules scalar until that bound makes conversion address-safe.
#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
pub(super) fn top_library_cards(game: &Game, player: Player, amount: u128) -> Vec<CardId> {
    let library = &game.state.players[player.index()].library;
    let count = usize::try_from(amount.min(library.len() as u128)).unwrap();
    library
        .iter()
        .rev()
        .copied()
        .filter(|&id| game.state.objects[id as usize].is_card())
        .take(count)
        .collect()
}
#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
impl Game {
    /// The linked permission is controlled by the source's current controller
    /// and follows its ability and threshold, not the entry trigger's controller.
    pub(super) fn threshold_linked_source(&self, card: CardId) -> Option<(ObjectRef, Option<u64>)> {
        let reference = self.object_ref(card);
        self.state
            .linked_choices
            .as_ref()?
            .null_cast_links
            .iter()
            .find(|link| link.exiled == reference)
            .map(|link| (link.source, link.ability_origin))
    }
    fn can_cast_threshold_linked_exile(&self, player: Player, card: CardId) -> bool {
        let Some(object) = self.state.objects.get(card as usize) else {
            return false;
        };
        object.zone == Zone::Exile
            && !object.card.is_land()
            && self
                .threshold_linked_source(card)
                .is_some_and(|source| threshold_linked_source_available(self, player, source))
    }
    pub(super) fn can_play_exiled(&self, player: Player, card: CardId) -> bool {
        self.can_cast_prepared(player, card)
            || self.can_play_adventure(player, card)
            || self.can_cast_exiled_card(player, card)
    }
    /// Permissions to cast the card for a mana cost, including an alternate
    /// spell face. Adventure's own right applies only to its normal face.
    pub(super) fn can_cast_exiled_card(&self, player: Player, card: CardId) -> bool {
        self.can_cast_exiled_independently(player, card)
            || self.linked_exile_play(player, card).is_some()
    }
    pub(super) fn can_cast_exiled_independently(&self, player: Player, card: CardId) -> bool {
        self.can_cast_threshold_linked_exile(player, card)
            || self.can_play_foreign_stash(player, card)
            || self.can_cast_warped(player, card)
            || self.has_printed_cost_exile_permission(player, card)
    }
    pub(super) fn has_printed_cost_exile_permission(&self, player: Player, card: CardId) -> bool {
        let object = self.object_ref(card);
        self.state.exile_permissions.iter().any(|p| {
            p.play_surcharge.is_none()
                && !p.life_instead_of_mana
                && !p.without_paying_mana_cost
                && !p.airbend
                && p.player == player
                && p.object == object
                && p.expiry.active(self.state.turn)
                && self.exile_permission_guard(p)
        })
    }
    pub(super) fn can_play_exiled_land(&self, player: Player, card: CardId) -> bool {
        self.can_play_exiled_land_independently(player, card)
            || self.linked_exile_play(player, card).is_some()
    }
    pub(super) fn can_play_exiled_land_independently(&self, player: Player, card: CardId) -> bool {
        self.can_play_foreign_stash(player, card)
            || self.can_play_adventure(player, card)
            || self.state.exile_permissions.iter().any(|p| {
                p.play_surcharge.is_none()
                    && !p.airbend
                    && p.player == player
                    && p.object == self.object_ref(card)
                    && p.expiry.active(self.state.turn)
                    && self.exile_permission_guard(p)
                    && !p.cast_only
            })
    }
    pub(super) fn exile_cast_modifiers(&self, player: Player, card: CardId) -> (bool, bool) {
        if self.state.objects[card as usize].zone != Zone::Exile {
            return (false, false);
        }
        self.state
            .exile_permissions
            .iter()
            .filter(|row| {
                row.player == player
                    && row.object == self.object_ref(card)
                    && row.expiry.active(self.state.turn)
                    && self.exile_permission_guard(row)
            })
            .fold((false, false), |flags, row| {
                (
                    flags.0 || row.spend_any_mana,
                    flags.1 || row.exile_on_graveyard,
                )
            })
    }
    pub(super) fn end_source_bound_exile_permissions(&mut self, source: ObjectRef) {
        self.state.exile_permissions.retain(|row| {
            row.source != Some(source) || row.expiry == ExilePlayExpiry::SourceLoreTurns
        });
    }
    pub(super) fn grant_source_bound_exile_cast(
        &mut self,
        source: ObjectRef,
        player: Player,
        object: ObjectRef,
    ) {
        if self.object_ref(source.card) != source
            || !self.on_battlefield(source.card)
            || self.state.objects[source.card as usize].controller != player
            || self.object_ref(object.card) != object
            || self.state.objects[object.card as usize].zone != Zone::Exile
            || self.current_kinds(object.card) & 12 == 0
        {
            return;
        }
        self.grant_exile_play_permission(player, object, false);
        let row = self
            .state
            .exile_permissions
            .iter_mut()
            .find(|r| r.object == object && r.player == player)
            .unwrap();
        row.source = Some(source);
        row.cast_only = true;
        row.spend_any_mana = true;
        row.expiry = ExilePlayExpiry::WhileExiled;
    }
    pub(super) fn exile_until_mana_value_free_cast(
        &mut self,
        owner: Player,
        player: Player,
        minimum: u16,
    ) {
        let ids = top_library_cards(self, owner, u128::MAX);
        let mut total = 0u128;
        for id in ids {
            if total >= u128::from(minimum) {
                break;
            }
            if self.state.objects[id as usize].zone != Zone::Library {
                continue;
            }
            let event = self.move_object(id, Zone::Exile);
            if event.moved && event.destination == Zone::Exile {
                total = total
                    .checked_add(u128::from(ability_world::CharacteristicWorld::mana_value(
                        self, id,
                    )))
                    .capacity("exiled mana value exhausted");
                self.grant_exile_free_permission_until_end(player, event.after);
            }
        }
    }
    pub(super) fn exile_targets_for_modified_cast(&mut self, item: &SpellView) {
        let ids = item
            .targets
            .iter()
            .filter(|&&bound| self.same_target_object(bound))
            .filter_map(|bound| {
                if let Target::Card(id) = bound.target {
                    Some(id)
                } else {
                    None
                }
            })
            .collect::<Vec<_>>();
        let events = self.move_objects_raw(&ids, Zone::Exile);
        for event in &events {
            if event.moved && event.destination == Zone::Exile && event.was_card {
                self.grant_exile_play_permission(item.controller, event.after, false);
                for row in &mut self.state.exile_permissions {
                    if row.player == item.controller && row.object == event.after {
                        row.spend_any_mana = true;
                        row.exile_on_graveyard = true;
                    }
                }
            }
        }
        self.finish_zone_events(&events, &[]);
    }
    pub(super) fn exile_top_play(&mut self, player: Player, amount: u128, next_turn: bool) {
        self.exile_top_play_from(player, player, amount, next_turn);
    }
    fn random_graveyard_card(
        &mut self,
        player: Player,
        eligible: impl Fn(Card) -> bool,
    ) -> Option<CardId> {
        let candidates: Vec<_> = self
            .state
            .objects
            .iter()
            .enumerate()
            .filter_map(|(id, object)| {
                (object.zone == Zone::Graveyard
                    && object.owner == player
                    && object.is_card()
                    && eligible(object.card))
                .then_some(id as CardId)
            })
            .collect();
        let index = match candidates.len() {
            0 => return None,
            1 => 0,
            len => self.state.rng.below(len as u64) as usize,
        };
        Some(candidates[index])
    }

    pub(super) fn exile_random_graveyard_play(&mut self, player: Player) {
        let Some(card) = self.random_graveyard_card(player, |_| true) else {
            return;
        };
        let events = self.move_objects_raw(&[card], Zone::Exile);
        for event in &events {
            if event.moved && event.destination == Zone::Exile {
                self.grant_exile_play_permission(player, event.after, false);
            }
        }
        self.finish_zone_events(&events, &[]);
    }

    pub(super) fn exile_random_graveyard_permanent(&mut self, player: Player) -> Option<Card> {
        let card = self.random_graveyard_card(player, |card| {
            card.kind_mask() & (1 | 2 | 16 | 32 | 64 | crate::cards::BATTLE_KIND) != 0
        })?;
        let events = self.move_objects_raw(&[card], Zone::Exile);
        self.finish_zone_events(&events, &[]);
        let event = &events[0];
        let exiled = &self.state.objects[event.after.card as usize];
        (event.moved
            && event.was_card
            && event.destination == Zone::Exile
            && exiled.zone == Zone::Exile
            && self.object_ref(event.after.card) == event.after)
            .then(|| exiled.card.front_face())
    }

    pub(super) fn exile_top_play_from(
        &mut self,
        library_owner: Player,
        player: Player,
        amount: u128,
        next_turn: bool,
    ) {
        let ids = top_library_cards(self, library_owner, amount);
        if ids.is_empty() {
            return;
        }
        let events = self.move_objects_raw(&ids, Zone::Exile);
        for event in &events {
            if event.moved && event.destination == Zone::Exile {
                self.grant_exile_play_permission(player, event.after, next_turn);
            }
        }
        self.finish_zone_events(&events, &[]);
    }

    pub(super) fn exile_top_play_until_next_end_step(&mut self, player: Player, amount: u128) {
        let ids = top_library_cards(self, player, amount);
        let events = self.move_objects_raw(&ids, Zone::Exile);
        for event in &events {
            if event.moved && event.destination == Zone::Exile {
                self.grant_exile_play_permission(player, event.after, false);
                for row in &mut self.state.exile_permissions {
                    if row.player == player && row.object == event.after {
                        row.expiry = ExilePlayExpiry::StartOfNextEndStep(player);
                    }
                }
            }
        }
        self.finish_zone_events(&events, &[]);
    }

    pub(super) fn exile_top_play_until_source_exiles_another(&mut self, item: &SpellView) {
        let player = item.controller;
        let (Some(source), Some(card)) = (item.source, self.top_library_card(player)) else {
            return;
        };
        let event = self.move_object(card, Zone::Exile);
        if event.moved && event.destination == Zone::Exile {
            let expiry = ExilePlayExpiry::UntilSourceExilesAnother(source.object);
            self.state
                .exile_permissions
                .retain(|row| row.player != player || row.expiry != expiry);
            self.grant_exile_play_permission_with_expiry(player, event.after, None, expiry);
        }
    }

    pub(super) fn exile_top_play_while_source_controlled(&mut self, item: &SpellView) {
        let player = item.controller;
        let Some(card) = self.top_library_card(player) else {
            return;
        };
        let event = self.move_object(card, Zone::Exile);
        if event.moved
            && event.destination == Zone::Exile
            && let Some(source) = item.source
            && self.on_battlefield(source.id)
            && self.object_ref(source.id) == source.object
            && self.state.objects[source.id as usize].controller == player
        {
            self.grant_exile_play_permission_with_expiry(
                player,
                event.after,
                Some(source.object),
                ExilePlayExpiry::WhileExiled,
            );
        }
    }

    pub(super) fn exile_each_top_play_own_and_lesser_free(&mut self, player: Player) {
        let cards: Vec<_> = [self.state.active, self.state.active.other()]
            .into_iter()
            .filter_map(|owner| self.top_library_card(owner))
            .collect();
        let events = self.move_objects_raw(&cards, Zone::Exile);
        let exiled = || {
            events
                .iter()
                .filter(|event| event.moved && event.destination == Zone::Exile)
        };
        if let Some(owned) = exiled().find(|event| event.before.owner == player) {
            let maximum = self.object_mana_value(owned.after.card);
            for event in exiled() {
                if event.after == owned.after || self.object_mana_value(event.after.card) < maximum
                {
                    self.grant_exile_free_permission_with_expiry(
                        player,
                        event.after,
                        ExilePlayExpiry::Turn(self.state.turn),
                        false,
                    );
                }
            }
        }
        self.finish_zone_events(&events, &[]);
    }

    pub(super) fn exile_top_play_on_source_lore_turns(&mut self, item: &SpellView) {
        let player = item.controller;
        let Some(id) = self.top_library_card(player) else {
            return;
        };
        let event = self.move_object(id, Zone::Exile);
        if event.moved && event.destination == Zone::Exile {
            let source = item.source.expect("lore permission has a source").object;
            if (self.on_battlefield(source.card) && self.object_ref(source.card) == source)
                || counter_placement_history::lore_placed(
                    &self.state.counter_placement_history,
                    source,
                    player,
                )
            {
                self.grant_exile_play_permission_with_expiry(
                    player,
                    event.after,
                    Some(source),
                    ExilePlayExpiry::SourceLoreTurns,
                );
            }
        }
    }

    pub(super) fn grant_exile_play_permission(
        &mut self,
        player: Player,
        object: ObjectRef,
        next_turn: bool,
    ) {
        let expiry = if next_turn {
            ExilePlayExpiry::EndOfNextTurn
        } else {
            ExilePlayExpiry::Turn(self.state.turn)
        };
        self.grant_exile_play_permission_with_expiry(player, object, None, expiry);
    }

    pub(super) fn grant_exile_play_permission_with_expiry(
        &mut self,
        player: Player,
        object: ObjectRef,
        source: Option<ObjectRef>,
        expiry: ExilePlayExpiry,
    ) -> Option<&mut ExilePlayPermission> {
        if self.object_ref(object.card) != object
            || self.state.objects[object.card as usize].zone != Zone::Exile
        {
            return None;
        }
        let at = self
            .state
            .exile_permissions
            .partition_point(|p| p.object.card < object.card);
        self.state.exile_permissions.insert(
            at,
            ExilePlayPermission {
                source,
                object,
                player,
                expiry,
                spend_any_mana: false,
                cast_only: false,
                exile_on_graveyard: false,
                life_instead_of_mana: false,
                without_paying_mana_cost: false,
                airbend: false,
                requires_subtype: None,
                play_surcharge: None,
            },
        );
        Some(&mut self.state.exile_permissions[at])
    }

    pub(super) fn grant_face_down_exile_play(
        &mut self,
        object: ObjectRef,
        beneficiaries: &[Player],
        spend_any_mana: bool,
    ) {
        let viewers = beneficiaries
            .iter()
            .fold(0u8, |mask, player| mask | (1 << player.index()));
        self.remember_face_down_exile(object, viewers, None);
        for &player in beneficiaries {
            let permission = self
                .grant_exile_play_permission_with_expiry(
                    player,
                    object,
                    None,
                    ExilePlayExpiry::WhileExiled,
                )
                .expect("face-down play permission requires an exact exile object");
            permission.spend_any_mana = spend_any_mana;
        }
    }

    pub(super) fn grant_taxed_exile_play(
        &mut self,
        player: Player,
        object: ObjectRef,
        surcharge: std::num::NonZeroU32,
    ) {
        if let Some(permission) = self.grant_exile_play_permission_with_expiry(
            player,
            object,
            None,
            ExilePlayExpiry::WhileExiled,
        ) {
            permission.play_surcharge = Some(surcharge);
        }
    }

    pub(super) fn exile_top_play_while_control_subtype(
        &mut self,
        player: Player,
        subtype: &'static str,
    ) {
        let Some(top) = self.top_library_card(player) else {
            return;
        };
        let events = self.move_objects_raw(&[top], Zone::Exile);
        for event in &events {
            if event.moved
                && event.destination == Zone::Exile
                && let Some(permission) = self.grant_exile_play_permission_with_expiry(
                    player,
                    event.after,
                    None,
                    ExilePlayExpiry::WhileExiled,
                )
            {
                permission.requires_subtype = Some(subtype);
            }
        }
        self.finish_zone_events(&events, &[]);
    }

    pub(super) fn exile_until_nonland_cast_this_turn(&mut self, player: Player) {
        while let Some(top) = self.top_library_card(player) {
            let events = self.move_objects_raw(&[top], Zone::Exile);
            let nonland = events.iter().find(|event| {
                event.moved && event.destination == Zone::Exile && !event.before.card.is_land()
            });
            if let Some(event) = nonland
                && let Some(permission) = self.grant_exile_play_permission_with_expiry(
                    player,
                    event.after,
                    None,
                    ExilePlayExpiry::Turn(self.state.turn),
                )
            {
                permission.cast_only = true;
            }
            self.finish_zone_events(&events, &[]);
            if nonland.is_some() || !events.iter().any(|event| event.moved) {
                break;
            }
        }
    }

    pub(super) fn visit_taxed_exile_plays(
        &self,
        actor: Player,
        card: CardId,
        sorcery: bool,
        casting_prohibited: bool,
        emit: &mut impl FnMut(Action) -> bool,
    ) -> bool {
        let current = &self.state.objects[card as usize];
        if current.zone != Zone::Exile || !current.is_card() || !current.card.is_supported() {
            return false;
        }
        let object = self.object_ref(card);
        let permissions = &self.state.exile_permissions;
        let first = permissions.partition_point(|permission| permission.object.card < card);
        let mut offered = Vec::new();
        for permission in &permissions[first..] {
            if permission.object.card != card {
                break;
            }
            let Some(surcharge) = permission.play_surcharge else {
                continue;
            };
            if permission.object != object
                || permission.player != actor
                || !permission.expiry.active(self.state.turn)
                || !self.exile_permission_guard(permission)
                || offered.contains(&surcharge)
            {
                continue;
            }
            if current.card.is_land() {
                return !permission.cast_only
                    && sorcery
                    && (!self.state.land_played
                        || self
                            .continuous_land_plays(actor)
                            .checked_add(self.state.players[actor.index()].extra_land_plays_granted)
                            .capacity("land allowance exhausted")
                            > self.state.continuous_land_plays_used)
                    && emit(Action::PlayTaxedExileLand { object });
            }
            offered.push(surcharge);
            if !casting_prohibited
                && !self.nonhand_cast_prohibited(actor)
                && self.visit_taxed_exile_casts(actor, object, sorcery, surcharge, emit)
            {
                return true;
            }
        }
        false
    }

    fn visit_taxed_exile_casts(
        &self,
        actor: Player,
        object: ObjectRef,
        sorcery: bool,
        surcharge: std::num::NonZeroU32,
        emit: &mut impl FnMut(Action) -> bool,
    ) -> bool {
        let physical = self.state.objects[object.card as usize].card;
        for (face, definition) in [
            (CastFace::Normal, Some(physical)),
            (CastFace::Adventure, physical.adventure_spell()),
            (CastFace::Omen, physical.omen_spell()),
            (CastFace::Back, physical.back_face()),
        ] {
            let Some(definition) = definition.filter(|card| !card.is_land()) else {
                continue;
            };
            let free = self.filtered_free_cast_available(actor, definition)
                || self.turn_free_creature_available(actor, definition);
            let sneak = face == CastFace::Normal
                && self.state.active == actor
                && self.state.step == Step::Blockers
                && self.has_sneak_cost(actor, object.card);
            if self.visit_web_slinging_face(
                actor,
                object.card,
                face,
                sorcery,
                WebSlingingOrigin::TaxedExile { surcharge },
                emit,
            ) {
                return true;
            }
            let methods = [
                Some(CastMethod::Normal),
                free.then_some(CastMethod::Free),
                sneak.then_some(CastMethod::Sneak),
            ];
            for method in methods.into_iter().flatten() {
                for flash_surcharge in [false, true] {
                    if flash_surcharge
                        && (method == CastMethod::Sneak || definition.flash_surcharge().is_none())
                    {
                        continue;
                    }
                    if self.permitted_cast_with_alternative(
                        actor,
                        object.card,
                        method,
                        face,
                        sorcery,
                        flash_surcharge,
                        None,
                        None,
                        None,
                        false,
                        cast_permissions::OriginAlternative {
                            exile_play_surcharge: surcharge.get(),
                            ..Default::default()
                        },
                    ) && emit(Action::AnnounceTaxedExile {
                        object,
                        face,
                        method,
                        flash_surcharge,
                        surcharge,
                    }) {
                        return true;
                    }
                }
            }
        }
        let disguised =
            || Card::from_name("@face-down/disguise").expect("pinned catalogue identity");
        physical.disguise_cost().is_some()
            && (sorcery || self.cast_timing_granted(actor, disguised()))
            && self.disguise_mana_fundable_with_surcharge(actor, object.card, surcharge.get())
            && emit(Action::AnnounceTaxedExile {
                object,
                face: CastFace::Normal,
                method: CastMethod::Disguise,
                flash_surcharge: false,
                surcharge,
            })
    }

    pub(super) fn configure_taxed_exile_cast(item: &mut SpellView, surcharge: u32) {
        let cast = item.cast.as_mut().expect("taxed exile cast");
        cast.exile_play_surcharge = surcharge;
        // Separate permissions cannot lend terms to the route being used.
        cast.spend_any_mana = false;
        cast.exile_on_graveyard = false;
        cast.stash_origin = false;
    }

    pub(super) fn start_taxed_exile_cast(
        &mut self,
        actor: Player,
        object: ObjectRef,
        face: CastFace,
        method: CastMethod,
        flash_surcharge: bool,
        surcharge: std::num::NonZeroU32,
    ) {
        let mut item = if method == CastMethod::Disguise {
            self.cast_disguised_item(actor, object.card)
        } else {
            self.cast_item(actor, object.card, [0; 6])
        };
        self.set_cast_face(&mut item, face);
        let cast = item.cast.as_mut().expect("taxed exile cast");
        cast.method = method;
        cast.flash_surcharge = flash_surcharge;
        if method == CastMethod::Sneak {
            Self::configure_sneak(&mut item);
        } else if method == CastMethod::Free && item.source.unwrap().card.x_symbols() > 0 {
            item.x = Some(0);
        }
        Self::configure_taxed_exile_cast(&mut item, surcharge.get());
        self.open_payment_announcement(item, Counters::default(), None);
    }

    pub(super) fn grant_flexible_exile_permission_this_turn(
        &mut self,
        player: Player,
        object: ObjectRef,
        cast_only: bool,
    ) {
        let expiry = ExilePlayExpiry::Turn(self.state.turn);
        if let Some(row) =
            self.grant_exile_play_permission_with_expiry(player, object, None, expiry)
        {
            row.spend_any_mana = true;
            row.cast_only = cast_only;
        }
    }

    pub(super) fn exile_top_choose_play(
        &mut self,
        player: Player,
        amount: u128,
        next_turn: bool,
    ) -> bool {
        let ids = top_library_cards(self, player, amount);
        let events = self.move_objects_raw(&ids, Zone::Exile);
        self.state.resolving.as_mut().unwrap().milled = events
            .iter()
            .filter(|e| e.moved && e.destination == Zone::Exile)
            .map(|e| e.after)
            .collect();
        self.finish_zone_events(&events, &[]);
        self.start_milled_selection(MilledSelectionSpec {
            filter: MilledFilter::Any,
            count: MilledCount::Range {
                minimum: 1,
                maximum: 1,
            },
            destination: MilledDestination::PlayPermission { next_turn },
        })
    }

    pub(super) fn exile_until_source_leaves(&mut self, item: &SpellView) {
        let Some(source) = item.source else {
            return;
        };
        let objects = item
            .targets
            .iter()
            .copied()
            .filter(|&b| self.same_target_object(b))
            .filter_map(|b| b.object)
            .collect::<Vec<_>>();
        self.exile_objects_until_source_leaves_origin(
            source.object,
            &objects,
            item.source_ability_origin,
        );
    }
    /// A non-target hand choice and an ordinary bound target share the same exact
    /// movement/link transaction. Only an actual arrival in exile creates a link.
    #[cfg(feature = "test-support")]
    pub(super) fn exile_objects_until_source_leaves(
        &mut self,
        source: ObjectRef,
        objects: &[ObjectRef],
    ) {
        self.exile_objects_until_source_leaves_origin(
            source,
            objects,
            self.copy_ability_origin(source),
        );
    }
    pub(super) fn exile_objects_until_source_leaves_origin(
        &mut self,
        source: ObjectRef,
        objects: &[ObjectRef],
        ability_origin: Option<u64>,
    ) {
        if !self.on_battlefield(source.card) || self.object_ref(source.card) != source {
            return;
        }
        let ids: Vec<_> = objects
            .iter()
            .filter(|r| self.object_ref(r.card) == **r)
            .map(|r| r.card)
            .collect();
        let events = self.move_objects_raw(&ids, Zone::Exile);
        for event in &events {
            if event.moved && event.destination == Zone::Exile {
                self.state
                    .exile_links
                    .extend(event.objects_after().map(|exiled| ExileLink {
                        ability_origin,
                        source,
                        exiled,
                        return_zone: event.before.zone,
                    }));
            }
        }
        self.finish_zone_events(&events, &[]);
    }

    pub(super) fn linked_exile_to_graveyard_tokens(&mut self, item: &SpellView, token: Card) {
        let Some(source) = item.source else {
            return;
        };
        let ids = self
            .state
            .exile_links
            .iter()
            .filter(|link| {
                link.source == source.object
                    && link.ability_origin == item.source_ability_origin
                    && self.object_ref(link.exiled.card) == link.exiled
                    && self.state.objects[link.exiled.card as usize].zone == Zone::Exile
                    && self.state.objects[link.exiled.card as usize].is_card()
            })
            .map(|link| link.exiled.card)
            .collect::<Vec<_>>();
        let events = self.move_objects_raw(&ids, Zone::Graveyard);
        let count = events
            .iter()
            .filter(|e| e.moved && e.destination == Zone::Graveyard)
            .count();
        self.finish_zone_events(&events, &[]);
        if count > 0 {
            self.create_tokens(item.controller, token, count as u128, false);
        }
    }

    pub(super) fn finish_zone_events(
        &mut self,
        events: &[ZoneChange],
        death_sources: &[ObjectView],
    ) {
        self.refresh_enduring_stories();
        self.advance_source_prevention_permanents(events);
        for event in events {
            if event.moved
                && let Some(player) = event.sacrificed_by
            {
                let count = &mut self.state.players[player.index()].permanents_sacrificed_this_turn;
                *count = count
                    .checked_add(1)
                    .capacity("permanent sacrifice history exhausted");
            }
        }
        if events
            .iter()
            .any(|event| event.moved && event.before.zone == Zone::Battlefield)
        {
            self.prune_source_bound_copies();
        }
        // Every member has moved before a gain consults live replacement providers.
        // Keep one gain event per actual linked exile, even for the same controller.
        for (player, amount) in events.iter().filter_map(|event| event.linked_exile_life) {
            self.gain_life(player, i128::from(amount));
        }
        self.queue_earthbend_returns(events);
        self.record_entry_turn_facts(events);
        self.record_creature_entry_history(events);
        self.queue_manifest_deaths(events);
        self.death_batch_triggers(events, death_sources);
        self.creature_cards_enter_graveyard_triggers(events);
        self.permanent_cards_enter_graveyard_triggers(events);
        self.creatures_exiled_batch_triggers(events);
        self.graveyard_departure_triggers(events, death_sources);
        self.returned_hand_triggers(events);
        for event in events {
            if event.moved
                && event.destination == Zone::Battlefield
                && self
                    .copiable_recipe(event.after)
                    .definition
                    .enters_prepared()
                && self.printed_abilities_available(event.after.card)
            {
                self.prepare_permanent(event.after);
            }
        }
        // No zone changes occur while observers are queued. Freeze their
        // post-event views once instead of rescanning the arena per newcomer.
        let sources: Vec<_> = if events
            .iter()
            .any(|e| e.moved && e.destination == Zone::Battlefield)
        {
            self.state
                .objects
                .iter()
                .enumerate()
                .filter(|(_, o)| {
                    (matches!(o.zone, Zone::Battlefield | Zone::Command)
                        && (o.zone == Zone::Battlefield || o.card.is_emblem())
                        && !o.card.triggers().is_empty())
                        || (o.zone == Zone::Graveyard
                            && o.card
                                .triggers()
                                .iter()
                                .any(|a| a.condition.origin() == Some(TargetZone::Graveyard)))
                })
                .map(|(i, _)| self.object_view(i as CardId))
                .collect()
        } else {
            Vec::new()
        };
        // Static sources are already registered for every member of the batch.
        // No moves occur during dispatch, so compute this union just once.
        let entry_suppression = if !events
            .iter()
            .any(|event| event.moved && event.destination == Zone::Battlefield)
        {
            0
        } else {
            self.state.static_sources.iter().fold(0, |mask, &id| {
                if self.printed_abilities_available(id) {
                    mask | self.state.objects[id as usize]
                        .card
                        .entry_trigger_suppression()
                } else {
                    mask
                }
            })
        };
        self.batch_graveyard_entry_triggers(events, &sources, entry_suppression);
        // One trigger per observing source for the entire simultaneous entry event.
        let mut token_entered = [Vec::new(), Vec::new()];
        let mut exile_creatures = Vec::new();
        for event in events {
            if event.moved && event.destination == Zone::Battlefield {
                let entered = self.object_view(event.after.card);
                if entered.kinds & entry_suppression != 0 {
                    continue;
                }
                if entered.is_token() {
                    token_entered[entered.controller.index()].push(entered.reference);
                } else if entered.is_creature()
                    && (event.before.zone == Zone::Exile || event.cast_origin == Some(Zone::Exile))
                {
                    exile_creatures.push(entered.reference);
                }
            }
        }
        for source in &sources {
            let tokens = &token_entered[source.controller.index()];
            if source.zone != Zone::Battlefield || (tokens.is_empty() && exile_creatures.is_empty())
            {
                continue;
            }
            for (index, ability) in source.card.triggers().iter().enumerate() {
                let entrants = match ability.condition.event() {
                    TriggerCondition::TokensEnterYou => tokens,
                    TriggerCondition::NontokenCreaturesEnterFromExile => &exile_creatures,
                    _ => continue,
                };
                if !entrants.is_empty() && self.queue_trigger(source, index) {
                    self.add_entry_trigger_multiplicity(source, entrants);
                    self.state
                        .pending_triggers
                        .last_mut()
                        .unwrap()
                        .refer_objects(entrants.iter().copied());
                }
            }
        }
        for event in events {
            // These zones are public in the supported rules subset (all exile
            // is face up). Preserve the exact new hand incarnation for both
            // observers; library draws and private searches reveal nothing.
            if event.moved
                && event.destination == Zone::Hand
                && matches!(
                    event.before.zone,
                    Zone::Battlefield | Zone::Graveyard | Zone::Stack | Zone::Exile
                )
                && (event.before.zone != Zone::Exile
                    || !self.exiled_face_down(event.before.reference))
                && event.was_card
                && self.object_ref(event.after.card) == event.after
                && self.state.objects[event.after.card as usize].zone == Zone::Hand
                && !self.state.known_hand.contains(&event.after)
            {
                self.state.known_hand.push(event.after);
            }
            self.zone_triggers(event, death_sources, &sources, entry_suppression);
            if event.moved && event.destination == Zone::Battlefield {
                let entrant = self.object_view(event.after.card);
                if entrant.kinds & 1 != 0 && entrant.kinds & entry_suppression == 0 {
                    self.queue_turn_callbacks(
                        crate::cards::TurnCallbackEvent::CreatureEntry,
                        entrant.controller,
                        Some(event.after),
                    );
                }
                // Entering with counters is a placement event on every entry path,
                // including raw returns and tokens, exactly once after the whole batch.
                let placed_by = event
                    .entry_counter_player
                    .expect("battlefield entry controller");
                let counters = self.counter_amounts(event.after.card).to_owned();
                // A successful true zone entry has a fresh exact incarnation.
                self.controlled_creature_counter_events(
                    std::iter::once((event.after, &counters, placed_by)),
                    true,
                );
                for kind in CounterKind::ALL {
                    self.self_counters_added(event.after, kind, counters.get(kind), placed_by);
                }
                self.lore_counters_added(event.after, 0);
            }
        }
        if self.has_counter_batch_observers() {
            let placements: Vec<_> = events
                .iter()
                .filter(|event| event.moved && event.destination == Zone::Battlefield)
                .map(|event| {
                    (
                        event.after,
                        self.counter_amounts(event.after.card).to_owned(),
                        event
                            .entry_counter_player
                            .expect("battlefield entry controller"),
                    )
                })
                .collect();
            self.counter_batch_added(
                placements
                    .iter()
                    .map(|(object, amount, player)| (*object, amount, *player)),
            );
        }
        for event in events {
            if event.moved && event.before.zone == Zone::Battlefield {
                self.end_source_bound_exile_permissions(event.before.reference);
            }
        }
        self.prune_linked_exiles();
        self.bind_first_public_source_moves(events);
        if self.state.exile_links.is_empty()
            || !events
                .iter()
                .any(|event| event.moved && event.before.zone == Zone::Battlefield)
        {
            return;
        }
        let mut returning = Vec::new();
        let mut index = 0;
        while index < self.state.exile_links.len() {
            let link = self.state.exile_links[index];
            let valid = self.object_ref(link.exiled.card) == link.exiled
                && self.state.objects[link.exiled.card as usize].zone == Zone::Exile;
            if valid
                && self.on_battlefield(link.source.card)
                && self.object_ref(link.source.card) == link.source
            {
                index += 1;
            } else {
                self.state.exile_links.swap_remove(index);
                if valid && !self.state.objects[link.exiled.card as usize].is_token() {
                    returning.push((link.exiled.card, link.return_zone));
                }
            }
        }
        // All effects created by the same departure batch return simultaneously.
        // They use no stack and offer no intervening choice or priority.
        let moves = returning
            .into_iter()
            .map(|(id, destination)| self.entry_move(id, destination, false))
            .collect();
        self.stage_entries(moves, None);
    }
}

#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
impl Game {
    pub(super) fn configure_exile_free_cast(item: &mut SpellView, flash_surcharge: bool) {
        let cast = item.cast.as_mut().unwrap();
        cast.method = CastMethod::Free;
        cast.flash_surcharge = flash_surcharge;
        cast.spend_any_mana = false;
        cast.exile_on_graveyard = false;
        cast.stash_origin = false;
        if item.source.expect("cast spell has source").card.x_symbols() > 0 {
            item.x = Some(0);
        }
    }
    pub(super) fn has_exile_free_permission(&self, player: Player, card: CardId) -> bool {
        self.state.objects[card as usize].zone == Zone::Exile
            && self.state.exile_permissions.iter().any(|p| {
                p.without_paying_mana_cost
                    && p.player == player
                    && p.object == self.object_ref(card)
                    && p.expiry.active(self.state.turn)
                    && self.exile_permission_guard(p)
            })
    }
    pub(super) fn grant_exile_free_permission(&mut self, player: Player, object: ObjectRef) {
        self.grant_exile_free_permission_with_expiry(
            player,
            object,
            ExilePlayExpiry::WhileExiled,
            true,
        );
    }
    pub(super) fn grant_exile_free_permission_until_end(
        &mut self,
        player: Player,
        object: ObjectRef,
    ) {
        self.grant_exile_free_permission_with_expiry(
            player,
            object,
            ExilePlayExpiry::Turn(self.state.turn),
            true,
        );
    }
    fn grant_exile_free_permission_with_expiry(
        &mut self,
        player: Player,
        object: ObjectRef,
        expiry: ExilePlayExpiry,
        cast_only: bool,
    ) {
        if self.object_ref(object.card) != object
            || self.state.objects[object.card as usize].zone != Zone::Exile
            || self.state.objects[object.card as usize].is_token()
        {
            return;
        }
        if let Some(existing) = self.state.exile_permissions.iter_mut().find(|p| {
            p.object == object
                && p.player == player
                && p.without_paying_mana_cost
                && p.expiry == expiry
        }) {
            existing.cast_only &= cast_only;
            return;
        }
        let at = self
            .state
            .exile_permissions
            .partition_point(|p| p.object.card < object.card);
        self.state.exile_permissions.insert(
            at,
            ExilePlayPermission {
                source: None,
                object,
                player,
                expiry,
                spend_any_mana: false,
                cast_only,
                exile_on_graveyard: false,
                life_instead_of_mana: false,
                without_paying_mana_cost: true,
                airbend: false,
                requires_subtype: None,
                play_surcharge: None,
            },
        );
    }
    pub(super) fn visit_exile_free_casts(
        &self,
        player: Player,
        card: CardId,
        sorcery: bool,
        emit: &mut impl FnMut(Action) -> bool,
    ) -> bool {
        if !self.has_exile_free_permission(player, card)
            || self.state.objects[card as usize].is_token()
        {
            return false;
        }
        for face in [
            CastFace::Normal,
            CastFace::Adventure,
            CastFace::Omen,
            CastFace::Back,
        ] {
            let physical = self.state.objects[card as usize].card;
            let definition = match face {
                CastFace::Normal => Some(physical),
                CastFace::Adventure => physical.adventure_spell(),
                CastFace::Omen => physical.omen_spell(),
                CastFace::Back => physical.back_face(),
            };
            if !definition.is_some_and(|d| !d.is_land()) {
                continue;
            }
            for flash_surcharge in [false, true] {
                if self.permitted_exile_free_cast_available(
                    player,
                    card,
                    face,
                    sorcery,
                    flash_surcharge,
                ) && emit(Action::AnnounceExileFree {
                    card,
                    face,
                    flash_surcharge,
                }) {
                    return true;
                }
            }
        }
        false
    }
}

#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
impl Game {
    pub(super) fn exile_permission_guard(&self, permission: &ExilePlayPermission) -> bool {
        if permission.expiry == ExilePlayExpiry::SourceLoreTurns {
            return permission.source.is_some_and(|source| {
                counter_placement_history::lore_placed(
                    &self.state.counter_placement_history,
                    source,
                    permission.player,
                )
            });
        }
        if permission.source.is_some_and(|source| {
            self.object_ref(source.card) != source
                || !self.on_battlefield(source.card)
                || self.state.objects[source.card as usize].controller != permission.player
        }) {
            return false;
        }
        permission.requires_subtype.is_none_or(|subtype| {
            self.state.objects.iter().enumerate().any(|(id, o)| {
                o.zone == Zone::Battlefield
                    && o.controller == permission.player
                    && self.has_subtype(id as CardId, subtype)
            })
        })
    }
    pub(super) fn exile_opponent_top_face_down_play_x(&mut self, item: &SpellView) {
        if item.source.is_none() {
            return;
        }
        let Some(opponent) = item.targets.iter().find_map(|target| match target.target {
            Target::Player(player) => Some(player),
            _ => None,
        }) else {
            return;
        };
        let ids = top_library_cards(self, opponent, u128::from(item.x.unwrap_or(0)));
        let events = self.move_objects_raw(&ids, Zone::Exile);
        for event in &events {
            if !event.moved || event.destination != Zone::Exile {
                continue;
            }
            self.grant_face_down_exile_play(event.after, &[item.controller], true);
        }
        self.finish_zone_events(&events, &[]);
    }
    pub(super) fn exile_top_face_down_play(
        &mut self,
        item: &SpellView,
        amount: u16,
        subtype: &'static str,
    ) {
        if item.source.is_none() {
            return;
        }
        let ids = top_library_cards(self, item.controller, u128::from(amount));
        let events = self.move_objects_raw(&ids, Zone::Exile);
        for event in &events {
            if !event.moved || event.destination != Zone::Exile {
                continue;
            }
            // Look access belongs to the captured player, independently of the
            // source and the condition on future plays. Reuse exact hidden rows.
            self.remember_face_down_exile(event.after, 1 << item.controller.index(), None);
            let at = self
                .state
                .exile_permissions
                .partition_point(|p| p.object.card < event.after.card);
            self.state.exile_permissions.insert(
                at,
                ExilePlayPermission {
                    source: None,
                    object: event.after,
                    player: item.controller,
                    expiry: ExilePlayExpiry::WhileExiled,
                    spend_any_mana: false,
                    cast_only: false,
                    exile_on_graveyard: false,
                    life_instead_of_mana: false,
                    without_paying_mana_cost: false,
                    airbend: false,
                    requires_subtype: Some(subtype),
                    play_surcharge: None,
                },
            );
        }
        self.finish_zone_events(&events, &[]);
    }
}

/// Re-evaluate the continuous permission in the live or speculative world.
#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
pub(super) fn threshold_linked_source_available<W: ability_world::CharacteristicWorld + ?Sized>(
    world: &W,
    player: Player,
    linked: (ObjectRef, Option<u64>),
) -> bool {
    let (source, ability_origin) = linked;
    let object = world.object(source.card);
    object.zone == Zone::Battlefield
        && world.reference(source.card) == source
        && object.controller == player
        && world.ability_origin(source.card) == ability_origin
        && activations::graveyard_cards_at_least(world, player, 0, 7)
        && object.card.static_ability_groups().iter().any(|group| {
            group.fragments.iter().any(|fragment| {
                matches!(
                    fragment,
                    crate::cards::StaticAbilityFragment::CastLinkedExileAtThreshold
                )
            })
        })
        && characteristic_query::CharacteristicQuery::in_world(world).continuous_fragment_available(
            source.card,
            |fragment| {
                matches!(
                    fragment,
                    crate::cards::StaticAbilityFragment::CastLinkedExileAtThreshold
                )
            },
        )
}

#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
impl Game {
    pub(super) fn configure_airbend_cast(item: &mut SpellView, flash_surcharge: bool) {
        let cast = item.cast.as_mut().expect("cast information");
        cast.method = CastMethod::Airbend;
        cast.flash_surcharge = flash_surcharge;
        cast.spend_any_mana = false;
        cast.exile_on_graveyard = false;
        cast.stash_origin = false;
        if item.source.expect("cast source").card.x_symbols() > 0 {
            item.x = Some(0);
        }
    }
    pub(super) fn has_airbend_permission(&self, player: Player, card: CardId) -> bool {
        self.state.objects[card as usize].zone == Zone::Exile
            && self.state.exile_permissions.iter().any(|row| {
                row.airbend
                    && row.player == player
                    && row.object == self.object_ref(card)
                    && row.expiry.active(self.state.turn)
                    && self.exile_permission_guard(row)
            })
    }
    pub(super) fn grant_airbend_permission(&mut self, player: Player, object: ObjectRef) {
        if self.object_ref(object.card) != object
            || self.state.objects[object.card as usize].zone != Zone::Exile
            || self.state.objects[object.card as usize].is_token()
            || self.state.objects[object.card as usize].is_spell_copy()
        {
            return;
        }
        if self.has_airbend_permission(player, object.card) {
            return;
        }
        let at = self
            .state
            .exile_permissions
            .partition_point(|row| row.object.card < object.card);
        self.state.exile_permissions.insert(
            at,
            ExilePlayPermission {
                source: None,
                object,
                player,
                expiry: ExilePlayExpiry::WhileExiled,
                spend_any_mana: false,
                cast_only: true,
                exile_on_graveyard: false,
                life_instead_of_mana: false,
                without_paying_mana_cost: false,
                airbend: true,
                requires_subtype: None,
                play_surcharge: None,
            },
        );
    }
    pub(super) fn visit_airbend_casts(
        &self,
        player: Player,
        card: CardId,
        sorcery: bool,
        emit: &mut impl FnMut(Action) -> bool,
    ) -> bool {
        if !self.has_airbend_permission(player, card)
            || self.state.objects[card as usize].is_token()
        {
            return false;
        }
        let physical = self.state.objects[card as usize].card;
        for face in [
            CastFace::Normal,
            CastFace::Adventure,
            CastFace::Omen,
            CastFace::Back,
        ] {
            let definition = match face {
                CastFace::Normal => Some(physical),
                CastFace::Adventure => physical.adventure_spell(),
                CastFace::Omen => physical.omen_spell(),
                CastFace::Back => physical.back_face(),
            };
            if !definition.is_some_and(|d| !d.is_land()) {
                continue;
            }
            for flash_surcharge in [false, true] {
                if self.permitted_cast_available(
                    player,
                    card,
                    CastMethod::Airbend,
                    face,
                    sorcery,
                    flash_surcharge,
                ) && emit(Action::AnnounceAirbend {
                    card,
                    face,
                    flash_surcharge,
                }) {
                    return true;
                }
            }
        }
        false
    }
}
