//! Owned face-down identity, public causes, and private choices.
use super::ability_world::CharacteristicWorld;
use super::*;
use crate::cards::{
    DisguiseCostModifier, DisguiseSpec, StaticAbilityFragment, TurnUpCounterAmount,
};

#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
pub(super) fn face_up_prohibited<W: CharacteristicWorld + ?Sized>(
    world: &W,
    object: ObjectRef,
) -> bool {
    let mut query = None;
    let prohibits = |fragment: &StaticAbilityFragment| {
        matches!(fragment, StaticAbilityFragment::AttachedFaceUpProhibited)
    };
    if world.attachment_sources().iter().copied().any(|id| {
        let source = world.object(id);
        source.zone == Zone::Battlefield
            && source.attached_to == Some(object)
            && world
                .definition(id)
                .static_ability_groups()
                .iter()
                .any(|group| group.fragments.iter().any(prohibits))
            && query
                .get_or_insert_with(|| characteristic_query::CharacteristicQuery::in_world(world))
                .continuous_fragment_available(id, prohibits)
    }) {
        return true;
    }

    let active = world.game().state.active;
    let prevents_opponent = |fragment: &StaticAbilityFragment| {
        matches!(
            fragment,
            StaticAbilityFragment::OpponentFaceUpProhibitedOnOwnTurn
        )
    };
    world.object(object.card).controller != active
        && world.static_sources().iter().copied().any(|id| {
            let source = world.object(id);
            source.zone == Zone::Battlefield
                && source.controller == active
                && world
                    .definition(id)
                    .static_ability_groups()
                    .iter()
                    .any(|group| group.fragments.iter().any(prevents_opponent))
                && query
                    .get_or_insert_with(|| {
                        characteristic_query::CharacteristicQuery::in_world(world)
                    })
                    .continuous_fragment_available(id, prevents_opponent)
        })
}

/// The public rule or effect that made an object face down (CR 708.6).
#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum FaceDownCause {
    Disguise,
    Manifest,
    Cloak,
    OtherEffect,
}

#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
impl FaceDownCause {
    pub(super) fn definition(self) -> Card {
        let identity = match self {
            Self::Disguise | Self::Cloak => "@face-down/disguise",
            Self::Manifest | Self::OtherEffect => "@face-down/manifest",
        };
        Card::from_name(identity).expect("pinned face-down definition")
    }

    fn allows_creature_mana(self) -> bool {
        matches!(self, Self::Manifest | Self::Cloak)
    }
}

#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) struct FaceDownEntry {
    pub physical: Card,
    pub cause: FaceDownCause,
    pub known: bool,
    /// Original slot plus one, retained only until the hidden cloak move commits.
    pub obscured_origin: Option<std::num::NonZeroU32>,
}

/// The public exact incarnation and any underlying card the viewer may know.
/// The current controller can look; a card revealed during a face-up special
/// action remains public knowledge (CR 708.5, 701.34b).
#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
pub struct FaceDownPermanentView {
    pub object: ObjectRef,
    pub cause: FaceDownCause,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub underlying: Option<Card>,
}

/// Private look at the cards from which manifest dread selects.
#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct ManifestChoiceView {
    pub player: Player,
    pub candidates: Vec<(ObjectRef, Card)>,
}
#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct ManifestDreadEvent {
    pub player: Player,
    pub manifested: Option<ObjectRef>,
    pub graveyard_cards: Vec<ObjectRef>,
}
#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
pub struct FaceUpPaymentView {
    pub object: ObjectRef,
    pub controller: Player,
    pub cost: ManaCost,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub x: Option<u32>,
    #[serde(skip)]
    pub(super) x_symbols: Option<std::num::NonZeroU8>,
}

#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) struct FaceDownPermanent {
    pub object: ObjectRef,
    pub underlying: Card,
    pub cause: FaceDownCause,
    pub known: bool,
    /// Players who learned this exact face-down incarnation's identity.
    pub known_to: u8,
}

#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) struct ManifestChoice {
    pub player: Player,
    pub candidates: Vec<ObjectRef>,
    pub selected: Option<ObjectRef>,
    pub result: Option<ObjectRef>,
}

#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) struct ManifestSequence {
    player: Player,
    remaining: u128,
    pub results: Vec<ObjectRef>,
}

#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
impl ManifestSequence {
    pub(super) fn owned_bytes(&self) -> usize {
        std::mem::size_of::<Self>() + std::mem::size_of_val(self.results.as_slice())
    }
}
#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) struct ManifestDeath {
    pub object: ObjectRef,
    pub source: SpellView,
}

#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub(super) struct ManifestState {
    pub permanents: Vec<FaceDownPermanent>,
    /// Exact physical identities for spells cast face down; never exposed to opponents.
    pub spells: Vec<FaceDownPermanent>,
    /// One choice per participant, in APNAP order. Selected cards remain in
    /// their libraries until the entire round is ready to commit.
    pub choices: Vec<ManifestChoice>,
    pub payment: Option<FaceUpPaymentView>,
    pub deaths: Vec<ManifestDeath>,
    pub last_event: Option<ManifestDreadEvent>,
    pub last_face_up: Option<ObjectRef>,
    pub temporary_opposing_look: [bool; 2],
    /// A face-up event whose counter replacements have not finished yet.
    pub face_up_completion: Option<(ObjectRef, Player, u32)>,
}
#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
impl ManifestState {
    pub(super) fn pending_choice(&self) -> Option<&ManifestChoice> {
        self.choices.iter().find(|choice| choice.needs_selection())
    }

    pub(super) fn bytes(&self) -> usize {
        std::mem::size_of::<Self>()
            + (self.permanents.len() + self.spells.len()) * std::mem::size_of::<FaceDownPermanent>()
            + self.last_event.as_ref().map_or(0, |event| {
                std::mem::size_of::<ManifestDreadEvent>()
                    + event.graveyard_cards.len() * std::mem::size_of::<ObjectRef>()
            })
            + self
                .deaths
                .iter()
                .map(|death| std::mem::size_of::<ManifestDeath>() + death.source.owned_bytes())
                .sum::<usize>()
            + self
                .choices
                .iter()
                .map(|choice| {
                    std::mem::size_of::<ManifestChoice>()
                        + std::mem::size_of_val(choice.candidates.as_slice())
                })
                .sum::<usize>()
    }
}

#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
impl ManifestChoice {
    fn needs_selection(&self) -> bool {
        self.selected.is_none() && self.candidates.len() > 1
    }
}
#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
impl Game {
    pub(super) fn face_down_knowledge_mask(&self, controller: Player) -> u8 {
        [Player::P0, Player::P1]
            .into_iter()
            .filter(|&viewer| self.shares_information(viewer, controller))
            .fold(0, |mask, viewer| mask | (1 << viewer.index()))
    }
    pub(super) fn remember_face_down_controller(&mut self, object: ObjectRef, controller: Player) {
        let mask = self.face_down_knowledge_mask(controller);
        if let Some(state) = &mut self.state.manifest {
            for row in state.permanents.iter_mut().chain(state.spells.iter_mut()) {
                if row.object == object {
                    row.known_to |= mask;
                }
            }
        }
    }
    pub(super) fn remember_controlled_player_faces(&mut self, player: Player, viewer: Player) {
        let objects = &self.state.objects;
        if let Some(state) = &mut self.state.manifest {
            for row in state.permanents.iter_mut().chain(state.spells.iter_mut()) {
                if objects.get(row.object.card as usize).is_some_and(|object| {
                    object.generation == row.object.generation && object.controller == player
                }) {
                    row.known_to |= 1 << viewer.index();
                }
            }
        }
    }

    /// Looking is always available while the rule applies. Record each view
    /// at the event boundary so that later loss of the rule cannot erase it.
    pub(super) fn remember_opposing_face_down_creatures(&mut self) {
        let Some(state) = &self.state.manifest else {
            return;
        };
        let hidden = |row: &FaceDownPermanent| !row.known && row.known_to != 0b11;
        if !state.permanents.iter().any(hidden) {
            return;
        }

        let look_rule = |fragment: &StaticAbilityFragment| {
            matches!(
                fragment,
                StaticAbilityFragment::LookAtOpponentFaceDownCreatures
            )
        };
        let mut query = None;
        let mut viewers = [0; 2];
        for controller in [Player::P0, Player::P1] {
            if state.temporary_opposing_look[controller.index()] {
                viewers[controller.other().index()] |= self.face_down_knowledge_mask(controller);
            }
        }
        for &id in self.state.static_sources.iter() {
            let source = &self.state.objects[id as usize];
            if source.zone == Zone::Battlefield
                && source
                    .card
                    .static_ability_groups()
                    .iter()
                    .any(|group| group.fragments.iter().any(look_rule))
                && query
                    .get_or_insert_with(|| self.characteristic_query())
                    .continuous_fragment_available(id, look_rule)
            {
                viewers[source.controller.other().index()] |=
                    self.face_down_knowledge_mask(source.controller);
            }
        }
        if viewers == [0; 2] {
            return;
        }

        let learned: Vec<_> = state
            .permanents
            .iter()
            .enumerate()
            .filter(|(_, row)| hidden(row))
            .filter_map(|(index, row)| {
                let id = row.object.card;
                let object = &self.state.objects[id as usize];
                let mask = viewers[object.controller.index()];
                (row.known_to & mask != mask
                    && object.zone == Zone::Battlefield
                    && self.object_ref(id) == row.object
                    && self.current_kinds(id) & 1 != 0)
                    .then_some((index, mask))
            })
            .collect();
        for (index, mask) in learned {
            self.state.manifest.as_mut().unwrap().permanents[index].known_to |= mask;
        }
    }
    pub(super) fn restore_public_face_down(
        &mut self,
        current: &State,
        objects: &[ObjectRef],
        obscured: &[ObjectRef],
    ) {
        if let Some(state) = &mut self.state.manifest {
            for row in &mut state.permanents {
                if obscured.contains(&row.object) {
                    row.known = false;
                    row.known_to = 0;
                } else {
                    if objects.contains(&row.object) {
                        row.known = true;
                    }
                    if let Some(current_row) = current.manifest.as_ref().and_then(|state| {
                        state
                            .permanents
                            .iter()
                            .find(|other| other.object == row.object)
                    }) {
                        row.known_to |= current_row.known_to;
                    }
                }
            }
        }
    }
    pub(super) fn obscure_rollback_face_down(&mut self, card: CardId) {
        if let Some(rollback) = &mut self.state.cast_rollback {
            if let Some(reference) = rollback.before.manifest.as_ref().and_then(|state| {
                state
                    .permanents
                    .iter()
                    .find(|row| row.object.card == card)
                    .map(|row| row.object)
            }) && !rollback.obscured_face_down.contains(&reference)
            {
                rollback.obscured_face_down.push(reference);
            }
            rollback
                .public_face_down
                .retain(|object| object.card != card);
        }
    }
    pub(super) fn face_up_payment_fundable(&self) -> bool {
        let Some(request) = self.face_up_payment() else {
            return false;
        };
        if !self.is_face_down(request.object)
            || !self.on_battlefield(request.object.card)
            || self.object_ref(request.object.card) != request.object
            || self.state.objects[request.object.card as usize].controller != request.controller
        {
            return false;
        }
        self.can_fund_face_up(
            FundingGoal {
                actor: request.controller,
                required: request.cost.colored,
                generic: request.cost.generic,
                reserve: None,
                source: None,
            },
            request.object,
        )
    }
    pub(super) fn register_manifest_death(&mut self, item: &SpellView) {
        let Some(bound) = item.targets.first().copied() else {
            return;
        };
        let Some(object) = bound.object else {
            return;
        };
        if !self.same_target_object(bound) || !self.on_battlefield(object.card) {
            return;
        }
        self.state
            .manifest
            .get_or_insert_with(Default::default)
            .deaths
            .push(ManifestDeath {
                object,
                source: item.clone(),
            });
    }
    pub(super) fn queue_manifest_deaths(&mut self, events: &[ZoneChange]) {
        let Some(state) = &mut self.state.manifest else {
            return;
        };
        let mut triggered = Vec::new();
        state.deaths.retain(|death| {
            let dies = events.iter().any(|event| {
                event.moved
                    && event.before.zone == Zone::Battlefield
                    && event.destination == Zone::Graveyard
                    && event.before.reference == death.object
            });
            if dies {
                triggered.push(death.source.clone());
            }
            !dies
        });
        for mut item in triggered {
            item.id = self.next_stack_id();
            item.kind = StackKind::DelayedManifestDread;
            item.targets.clear();
            item.target_type_slots = None;
            item.mode = None;
            item.cast = None;
            item.is_copy = false;
            item.related_stack = None;
            item.additional_triggers = 0;
            self.state.pending_triggers.push(item);
        }
    }
    pub(super) fn expire_manifest_deaths(&mut self) {
        if let Some(state) = &mut self.state.manifest {
            state.deaths.clear();
        }
    }
    /// Feasibility is evaluated with the public creature spell's characteristics,
    /// never with the private card's colors, types, or printed cost.
    pub(super) fn disguise_cast_fundable(&self, player: Player, id: CardId) -> bool {
        let object = &self.state.objects[id as usize];
        let origin_permitted = match object.zone {
            Zone::Hand => object.owner == player,
            Zone::Graveyard => self.has_graveyard_cast(player, id, GraveyardCastKind::Cast),
            Zone::Exile => !object.is_spell_copy() && self.can_play_exiled(player, id),
            Zone::Library => self.can_cast_library_face(
                player,
                id,
                Card::from_name("@face-down/disguise").expect("pinned catalogue identity"),
            ),
            _ => false,
        };
        origin_permitted && self.disguise_mana_fundable(player, id)
    }

    pub(super) fn disguise_mana_fundable(&self, player: Player, id: CardId) -> bool {
        self.disguise_mana_fundable_with_surcharge(player, id, 0)
    }

    pub(super) fn disguise_cast_definition(&self, id: CardId) -> Option<Card> {
        let object = &self.state.objects[id as usize];
        let physical = object.card;
        if object.is_token()
            || physical.disguise_spec().is_none()
            || !(physical.is_supported()
                || cfg!(feature = "test-support")
                    && (physical
                        == Card::from_name("Bubble Smuggler").expect("pinned catalogue identity")
                        || physical
                            == Card::from_name("Concealed Weapon")
                                .expect("pinned catalogue identity")))
        {
            return None;
        }
        Card::from_name("@face-down/disguise")
    }

    pub(super) fn disguise_mana_fundable_with_surcharge(
        &self,
        player: Player,
        id: CardId,
        surcharge: u32,
    ) -> bool {
        let Some(definition) = self.disguise_cast_definition(id) else {
            return false;
        };
        3u32.checked_add(surcharge).is_some_and(|generic| {
            self.spell_can_fund_for_definition(
                player,
                id,
                ManaCost {
                    colored: [0; 6],
                    generic,
                },
                definition,
            )
        })
    }

    /// Begin a face-down spell with only its public copiable values on the
    /// actual stack object. The identity stays in the owned private registry.
    pub(super) fn cast_disguised_item(&mut self, player: Player, id: CardId) -> SpellView {
        let reference = self.object_ref(id);
        let object = &self.state.objects[id as usize];
        let physical = object.card;
        let origin = object.zone;
        let known = match origin {
            Zone::Graveyard => true,
            Zone::Exile => self.can_view_hideaway(player.other(), reference),
            Zone::Library => self.public_library_top(object.owner) == Some(reference),
            _ => false,
        };
        // Permission guards inspect the public card in its origin zone, so
        // capture these before applying face-down spell characteristics.
        let modifiers = self.exile_cast_modifiers(player, id);
        let stash_origin = self.foreign_stash(player, id);
        let id = if origin == Zone::Hand {
            // Remaining known disguise candidates would identify the card by
            // elimination. The cast checkpoint retains the pre-announcement
            // hand knowledge if the proposed cast has to be reversed.
            let objects = &self.state.objects;
            self.state.known_hand.retain(|known| {
                let card = &objects[known.card as usize];
                card.owner != player || card.card.disguise_spec().is_none()
            });
            let (fresh, original) = self
                .fresh_hidden_card_slot(reference)
                .expect("validated private disguise origin");
            self.finish_hidden_card_identity(original, fresh, Zone::Hand, true);
            fresh.card
        } else {
            id
        };
        self.state.objects[id as usize].card =
            Card::from_name("@face-down/disguise").expect("pinned catalogue identity");
        let mut item = self.cast_item(player, id, [0; 6]);
        if let Some(cast) = &mut item.cast {
            cast.spend_any_mana = modifiers.0;
            cast.exile_on_graveyard = modifiers.1;
            cast.stash_origin = stash_origin;
        }
        let known_to = self.face_down_knowledge_mask(player);
        self.state
            .manifest
            .get_or_insert_with(Default::default)
            .spells
            .push(FaceDownPermanent {
                object: item.source.expect("face-down cast has source").object,
                underlying: physical,
                cause: FaceDownCause::Disguise,
                known,
                known_to,
            });
        item
    }

    pub(super) fn take_face_down_spell(
        &mut self,
        reference: ObjectRef,
    ) -> Option<FaceDownPermanent> {
        let state = self.state.manifest.as_mut()?;
        let index = state
            .spells
            .iter()
            .position(|row| row.object == reference)?;
        Some(state.spells.remove(index))
    }
    pub(super) fn face_up_payment(&self) -> Option<&FaceUpPaymentView> {
        self.state.manifest.as_ref()?.payment.as_ref()
    }
    pub(super) fn begin_face_up(
        &mut self,
        player: Player,
        object: ObjectRef,
        cost: ManaCost,
        x_symbols: Option<std::num::NonZeroU8>,
    ) {
        self.reveal_face_down_permanent(object)
            .expect("face-up payment reveals the exact permanent");
        self.state.manifest.as_mut().unwrap().payment = Some(FaceUpPaymentView {
            object,
            controller: player,
            cost,
            x: None,
            x_symbols,
        });
        self.state.pending = Pending::FaceUpPayment { player };
        self.state.passes = 0;
    }

    pub(super) fn face_up_x_choice(&self) -> Option<XChoiceView> {
        let request = self.face_up_payment()?;
        let symbols = u32::from(request.x_symbols?.get());
        let mut minimum = 0;
        let mut maximum = (u32::MAX - request.cost.generic) / symbols;
        while minimum < maximum {
            let midpoint = minimum + (maximum - minimum).div_ceil(2);
            if self.can_fund_face_up(
                FundingGoal {
                    actor: request.controller,
                    required: request.cost.colored,
                    generic: request.cost.generic + symbols * midpoint,
                    reserve: None,
                    source: None,
                },
                request.object,
            ) {
                minimum = midpoint;
            } else {
                maximum = midpoint - 1;
            }
        }
        Some(XChoiceView {
            card: request.object,
            intervals: vec![XInterval {
                minimum: 0,
                maximum,
            }],
        })
    }

    pub(super) fn choose_face_up_x(&mut self, x: u32) {
        let request = self
            .state
            .manifest
            .as_mut()
            .unwrap()
            .payment
            .as_mut()
            .unwrap();
        let symbols = u32::from(request.x_symbols.take().expect("variable disguise").get());
        request.cost.generic += symbols * x;
        request.x = Some(x);
    }
    pub(super) fn finish_face_up(&mut self, payment: Mana) {
        let request = self
            .state
            .manifest
            .as_mut()
            .unwrap()
            .payment
            .take()
            .unwrap();
        self.spend_mana_for(
            request.controller,
            payment,
            mana_lots::Purpose::face_up(self, request.object),
        );
        // The replacement engine captures this as its resume state. Set it
        // before beginning the transition so a suspended choice is never lost.
        self.state.pending = Pending::Priority;
        self.state.priority = request.controller;
        self.state.passes = 0;
        // CR 701.58g can reveal a cloaked instant or sorcery and leave
        // it face down even after paying a copied disguise ability's cost.
        self.turn_manifest_face_up_with_x(
            request.object,
            request.controller,
            request.x.unwrap_or(0),
        );
    }
    pub(super) fn face_down_physical(&self, object: ObjectRef) -> Option<Card> {
        self.state
            .manifest
            .as_ref()?
            .permanents
            .iter()
            .find(|row| row.object == object)
            .map(|row| row.underlying)
    }

    pub(super) fn is_face_down(&self, object: ObjectRef) -> bool {
        self.state
            .manifest
            .as_ref()
            .is_some_and(|state| state.permanents.iter().any(|row| row.object == object))
    }

    pub(super) fn first_bound_face_down_permanent(&self, item: &SpellView) -> Option<ObjectRef> {
        item.targets
            .iter()
            .filter_map(|bound| bound.object)
            .find(|&object| {
                self.on_battlefield(object.card)
                    && self.object_ref(object.card) == object
                    && self.is_face_down(object)
            })
    }

    pub(super) fn reveal_face_down_permanent(&mut self, object: ObjectRef) -> Option<Card> {
        let row = self
            .state
            .manifest
            .as_mut()?
            .permanents
            .iter_mut()
            .find(|row| row.object == object)?;
        row.known = true;
        let physical = row.underlying;
        self.record_zone_reveal(
            physical,
            self.state.objects[object.card as usize].owner,
            Zone::Battlefield,
        );
        Some(physical)
    }

    /// Complete a status change caused by a special action or by a resolving effect.
    /// The object keeps its battlefield incarnation.
    pub(super) fn turn_manifest_face_up(&mut self, object: ObjectRef, player: Player) -> bool {
        self.turn_manifest_face_up_with_x(object, player, 0)
    }

    fn turn_manifest_face_up_with_x(&mut self, object: ObjectRef, player: Player, x: u32) -> bool {
        if !self.on_battlefield(object.card)
            || self.object_ref(object.card) != object
            || face_up_prohibited(self, object)
        {
            return false;
        }
        // CR701.40g: an effect cannot turn an instant or sorcery card face up;
        // reveal it and leave the same permanent face down instead.
        let Some(physical) = self.reveal_face_down_permanent(object) else {
            return false;
        };
        if physical.kind_mask() & (4 | 8) != 0 {
            return false;
        }
        // Evaluate the resulting copy and layer state without exposing a half
        // completed face-up transition to an affected player's choice.
        let mut prospective = self.clone();
        prospective.materialize_face_up(object);
        let card = prospective.state.objects[object.card as usize].card;
        let o = &prospective.state.objects[object.card as usize];
        let mut amount = Counters::default();
        let counters = card.as_turned_face_up_counters();
        if !counters.is_empty() && prospective.printed_abilities_available(object.card) {
            for &(kind, value) in counters {
                let count = match value {
                    TurnUpCounterAmount::Fixed(count) => count,
                    TurnUpCounterAmount::Stat(value) => {
                        u64::try_from(prospective.stat_value(value, object, o.controller).max(0))
                            .capacity("face-up counter amount exhausted")
                    }
                };
                amount.add(kind, count);
            }
        }
        if amount.has_any() {
            let placement = replacements::CounterPlacement {
                object,
                placed_by: o.controller,
                card,
                owner: o.owner,
                controller: o.controller,
                protector: prospective.battle_protector(object),
                subtypes: prospective.granted_subtypes(object.card),
                kind: prospective.current_kinds(object.card),
                power: prospective.object_view(object.card).power,
                amount,
                effect: true,
                entry_index: None,
                private: false,
            };
            self.state.manifest.as_mut().unwrap().face_up_completion = Some((object, player, x));
            let mode = if self.state.resolving.is_some() {
                replacements::ResumeMode::Program
            } else {
                replacements::ResumeMode::Pending
            };
            self.stage_counter_placements(vec![placement], mode);
        } else {
            self.materialize_face_up(object);
            self.queue_face_up_triggers(object, player, x);
        }
        true
    }

    /// Apply the copiable face and re-register providers without reporting the
    /// event. Counter replacement commits call this immediately before adding
    /// the finalized counters, so observers see a single completed transition.
    pub(super) fn materialize_face_up(&mut self, object: ObjectRef) {
        let control_watch = self.freeze_control_watch();
        let physical = self
            .take_face_down_physical(object)
            .expect("face-down incarnation");
        self.forget_hand_size_source(object);
        self.remove_face_providers(object.card);
        self.state.objects[object.card as usize].card = physical;
        if !self.reveal_temporary_copy(object, physical) {
            self.register_battlefield_providers(object.card);
        }
        self.retimestamp_self_granted_statics(object);
        self.refresh_controllers_after(Some(control_watch));
    }

    pub(super) fn finish_face_up_counter_replacement(&mut self) {
        let Some((object, player, x)) = self
            .state
            .manifest
            .as_mut()
            .and_then(|state| state.face_up_completion.take())
        else {
            return;
        };
        self.queue_face_up_triggers(object, player, x);
    }
    fn queue_face_up_triggers(&mut self, object: ObjectRef, player: Player, x: u32) {
        let Some(turned) = self.state.objects.get(object.card as usize) else {
            return;
        };
        if turned.zone != Zone::Battlefield || self.object_ref(object.card) != object {
            return;
        }
        let controller = turned.controller;
        let creature = self.current_kinds(object.card) & 1 != 0;
        self.state.players[player.index()]
            .turn_facts
            .face_down_entry_or_turn_up = true;
        self.state
            .manifest
            .get_or_insert_with(Default::default)
            .last_face_up = Some(object);
        let ids: Vec<_> = self
            .state
            .objects
            .iter()
            .enumerate()
            .filter(|(_, source)| source.zone == Zone::Battlefield)
            .map(|(id, _)| id as CardId)
            .collect();
        for (source, grant) in self.indexed_trigger_sources(&ids) {
            let definition = grant
                .as_ref()
                .map_or(source.card, |grant| grant.granting_card);
            for (index, ability) in definition.triggers().iter().enumerate() {
                let matched = match ability.condition.event() {
                    TriggerCondition::SelfTurnedFaceUp
                    | TriggerCondition::SelfEntersOrTurnsFaceUp => source.reference == object,
                    TriggerCondition::FilteredEntersOrTurnsFaceUpYou {
                        face_up_subtype, ..
                    } => {
                        source.controller == controller
                            && self.has_subtype(object.card, face_up_subtype)
                    }
                    TriggerCondition::SelfOrControlledCreatureTurnedFaceUp => {
                        source.reference == object || (creature && source.controller == controller)
                    }
                    TriggerCondition::PermanentYouControlTurnedFaceUp
                    | TriggerCondition::FaceDownPermanentEntersOrTurnsUpYou => {
                        source.controller == controller
                    }
                    TriggerCondition::YouTurnPermanentFaceUp => source.controller == player,
                    _ => false,
                };
                if matched && self.queue_trigger_from(&source, index, grant.map(Box::new)) {
                    let facts = if ability.program.iter().any(|instruction| {
                        matches!(instruction, Instruction::EnteringPowerDamagePlayers(_))
                    }) {
                        EventObjectFacts::DamageActor(self.damage_source(object.card))
                    } else {
                        EventObjectFacts::Controller(controller)
                    };
                    let packet = self.state.pending_triggers.last_mut().unwrap();
                    packet.event_player = Some(player);
                    if source.reference == object {
                        packet.x = Some(x);
                    }
                    packet.set_event_object(EventObjectView {
                        object,
                        facts,
                        semantic_objects: None,
                        conniver_creature: None,
                        subtype_lki: None,
                    });
                }
            }
        }
    }
    pub(super) fn queue_manifest_dread_triggers(&mut self, event: ManifestDreadEvent) {
        self.state
            .manifest
            .get_or_insert_with(Default::default)
            .last_event = Some(event.clone());
        let ids: Vec<_> = self
            .state
            .objects
            .iter()
            .enumerate()
            .filter(|(_, source)| source.zone == Zone::Battlefield)
            .map(|(id, _)| id as CardId)
            .collect();
        for (source, grant) in self.indexed_trigger_sources(&ids) {
            let definition = grant
                .as_ref()
                .map_or(source.card, |grant| grant.granting_card);
            for (index, ability) in definition.triggers().iter().enumerate() {
                if ability.condition.event() == TriggerCondition::YouManifestDread
                    && source.controller == event.player
                    && self.queue_trigger_from(&source, index, grant.map(Box::new))
                {
                    self.state
                        .pending_triggers
                        .last_mut()
                        .unwrap()
                        .manifest_event = Some(Box::new(event.clone()));
                }
            }
        }
    }
    pub(super) fn begin_manifest_dread(&mut self, player: Player, count: u128) {
        self.state.resolving.as_mut().unwrap().manifest_sequence = (count != 0).then(|| {
            Box::new(ManifestSequence {
                player,
                remaining: count,
                results: Vec::new(),
            })
        });
    }

    pub(super) fn manifest_dread_for_linked_exile_owners(&mut self, item: &SpellView) {
        let owners = self
            .linked_exile_for_item(item)
            .iter()
            .map(|object| &self.state.objects[object.card as usize])
            .filter(|object| object.is_card())
            .fold(0, |mask, object| mask | (1 << object.owner.index()));
        if owners != 0 {
            self.state.resolving.as_mut().unwrap().manifest_sequence = None;
            self.start_manifest_round(
                [self.state.active, self.state.active.other()]
                    .into_iter()
                    .filter(|player| owners & (1 << player.index()) != 0),
            );
        }
    }

    pub(super) fn advance_manifest_dread(&mut self) -> bool {
        if self
            .state
            .manifest
            .as_ref()
            .is_some_and(|state| !state.choices.is_empty())
        {
            return true;
        }
        while let Some(sequence) = self
            .state
            .resolving
            .as_mut()
            .and_then(|resolution| resolution.manifest_sequence.as_mut())
        {
            if sequence.remaining == 0 {
                break;
            }
            sequence.remaining -= 1;
            let player = sequence.player;
            if self.start_manifest_round([player]) {
                return true;
            }
        }
        false
    }

    /// CR 101.4 and 701.62. Each player looks privately and chooses in APNAP
    /// order. The selected cards enter together before the remaining moves.
    fn start_manifest_round(&mut self, players: impl IntoIterator<Item = Player>) -> bool {
        let choices = players
            .into_iter()
            .map(|player| {
                let candidates: Vec<_> = self.state.players[player.index()]
                    .library
                    .iter()
                    .rev()
                    .filter(|&&id| self.state.objects[id as usize].is_card())
                    .take(2)
                    .map(|&id| self.object_ref(id))
                    .collect();
                let selected = (candidates.len() == 1).then(|| candidates[0]);
                ManifestChoice {
                    player,
                    candidates,
                    selected,
                    result: None,
                }
            })
            .collect();
        self.state
            .manifest
            .get_or_insert_with(Default::default)
            .choices = choices;
        self.stage_manifest_round()
    }

    pub(super) fn choose_manifest(&mut self, selected: ObjectRef) {
        let choice = self
            .state
            .manifest
            .as_mut()
            .unwrap()
            .choices
            .iter_mut()
            .find(|choice| choice.needs_selection())
            .unwrap();
        assert!(choice.candidates.contains(&selected));
        choice.selected = Some(selected);
        self.stage_manifest_round();
    }

    fn stage_manifest_round(&mut self) -> bool {
        let state = self.state.manifest.as_ref().unwrap();
        if let Some(choice) = state.pending_choice() {
            self.state.pending = Pending::Manifest {
                player: choice.player,
            };
            return true;
        }
        // The consumed choice must not be restored as an entry continuation.
        if matches!(self.state.pending, Pending::Manifest { .. }) {
            self.state.pending = Pending::Priority;
        }
        let moves: Vec<_> = state
            .choices
            .iter()
            .filter_map(|choice| {
                let selected = choice.selected?;
                Some(
                    self.manifest_entry(selected, choice.player)
                        .expect("selected manifest card remains in library"),
                )
            })
            .collect();
        if moves.is_empty() {
            self.finish_manifest_entry();
            false
        } else {
            self.stage_entries(moves, None);
            true
        }
    }

    /// Called after the staged entry commits. Cards that weren't manifested,
    /// including a card whose entry was prevented, are moved simultaneously.
    pub(super) fn finish_manifest_entry(&mut self) {
        let choices = std::mem::take(&mut self.state.manifest.as_mut().unwrap().choices);
        let remaining: Vec<_> = choices
            .iter()
            .flat_map(|choice| choice.candidates.iter().copied())
            .filter(|&object| {
                self.state
                    .objects
                    .get(object.card as usize)
                    .is_some_and(|card| {
                        card.zone == Zone::Library && self.object_ref(object.card) == object
                    })
            })
            .map(|object| object.card)
            .collect();
        if let Some(sequence) = self
            .state
            .resolving
            .as_mut()
            .and_then(|resolution| resolution.manifest_sequence.as_mut())
        {
            sequence
                .results
                .extend(choices.iter().filter_map(|choice| choice.result));
        }
        let events = if remaining.is_empty() {
            Vec::new()
        } else {
            let events = self.move_objects_raw(&remaining, Zone::Graveyard);
            self.finish_zone_events(&events, &[]);
            events
        };
        for choice in choices {
            let graveyard_cards = events
                .iter()
                .filter(|event| {
                    event.moved
                        && event.destination == Zone::Graveyard
                        && choice.candidates.contains(&event.before.reference)
                })
                .map(|event| event.after)
                .collect();
            // CR 701.62b: completion is an event even with no cards to move.
            self.queue_manifest_dread_triggers(ManifestDreadEvent {
                player: choice.player,
                manifested: choice.result,
                graveyard_cards,
            });
        }
    }
    pub(super) fn take_face_down_physical(&mut self, reference: ObjectRef) -> Option<Card> {
        if let Some(rollback) = &mut self.state.cast_rollback
            && rollback
                .before
                .manifest
                .as_ref()
                .is_some_and(|state| state.permanents.iter().any(|row| row.object == reference))
            && !rollback.obscured_face_down.contains(&reference)
            && !rollback.public_face_down.contains(&reference)
        {
            rollback.public_face_down.push(reference);
        }
        let state = self.state.manifest.as_mut()?;
        let index = state
            .permanents
            .iter()
            .position(|row| row.object == reference)?;
        Some(state.permanents.remove(index).underlying)
    }
    /// Build a staged, private battlefield entry using CR 708.2 copiable
    /// characteristics without exposing or mutating the library card.
    pub(super) fn manifest_entry(
        &self,
        object: ObjectRef,
        controller: Player,
    ) -> Option<entries::EntryMove> {
        let current = self.state.objects.get(object.card as usize)?;
        if current.zone != Zone::Library {
            return None;
        }
        self.face_down_entry(
            object,
            controller,
            FaceDownCause::Manifest,
            self.public_library_top(current.owner) == Some(object),
        )
    }

    pub(super) fn face_down_entry(
        &self,
        object: ObjectRef,
        controller: Player,
        cause: FaceDownCause,
        known: bool,
    ) -> Option<entries::EntryMove> {
        let current = self.state.objects.get(object.card as usize)?;
        if !matches!(
            current.zone,
            Zone::Library | Zone::Hand | Zone::Graveyard | Zone::Exile
        ) || self.object_ref(object.card) != object
            || !current.is_card()
        {
            return None;
        }
        let mut movement = self.entry_move(object.card, Zone::Battlefield, false);
        let view = &mut movement.before;
        view.card = cause.definition();
        view.name = "";
        view.kinds = 1;
        view.colors = 0;
        view.effective_legendary = None;
        view.granted_subtypes = GrantedSubtypes::default();
        view.entry_choice = None;
        view.abilities = None;
        view.power = 2;
        view.toughness = 2;
        view.power_above_base = false;
        view.keywords = view.card.keywords();
        view.face = None;
        view.copy_recipe = None;
        view.temporary_copy = None;
        view.provided_ability_origin = None;
        view.token = None;
        view.emblem = None;
        movement.controller = controller;
        movement.face_down = Some(FaceDownEntry {
            physical: current.card,
            cause,
            known,
            obscured_origin: None,
        });
        Some(movement)
    }

    pub(super) fn return_departed_permanent_face_down(&mut self, item: &SpellView) {
        let Some(EventObjectFacts::DepartureArrival(object)) =
            item.event_object.as_ref().map(|event| &event.facts)
        else {
            return;
        };
        let current = &self.state.objects[object.card as usize];
        if current.zone != Zone::Graveyard
            || current.card.kind_mask() & crate::cards::PERMANENT_KINDS == 0
        {
            return;
        }
        if let Some(movement) =
            self.face_down_entry(*object, current.owner, FaceDownCause::OtherEffect, true)
        {
            self.stage_entries(vec![movement], None);
        }
    }

    pub(super) fn cloak_library_top(&mut self, player: Player, owner: Player) {
        let object = self.top_library_card(owner).map(|id| self.object_ref(id));
        let known = object.is_some() && object == self.public_library_top(owner);
        self.cloak_one(object, player, known);
    }

    pub(super) fn cloak_one(&mut self, object: Option<ObjectRef>, player: Player, known: bool) {
        if let Some(movement) = object
            .and_then(|object| self.face_down_entry(object, player, FaceDownCause::Cloak, known))
        {
            self.stage_entries(vec![movement], None);
        } else if let Some(frame) = self
            .state
            .resolving
            .as_mut()
            .and_then(|r| r.search_result.as_mut())
            && frame.capture_next_entry
        {
            frame.capture_next_entry = false;
            frame.closed = true;
        }
    }

    /// CR 702.168d checks the disguise ability on the prospective face-up
    /// permanent. Pay the creature's printed mana cost only under the
    /// independent cloak or manifest special action.
    fn face_up_disguise_spec(&self, row: FaceDownPermanent) -> Option<DisguiseSpec> {
        let has_copy = self.temporary_copy_row(row.object).is_some();
        if !has_copy {
            let cost = row.underlying.disguise_spec()?;
            if !self.has_ability_removal() {
                return Some(cost);
            }
        }
        let mut prospective = self.clone();
        prospective.materialize_face_up(row.object);
        if !prospective.printed_abilities_available(row.object.card) {
            return None;
        }
        prospective.state.objects[row.object.card as usize]
            .card
            .disguise_spec()
    }

    pub(super) fn visit_face_up_actions(
        &self,
        player: Player,
        object: ObjectRef,
        emit: &mut impl FnMut(Action) -> bool,
    ) -> bool {
        if self.state.pending != Pending::Priority || self.semantic_actor() != Some(player) {
            return false;
        }
        let Some(row) = self
            .state
            .manifest
            .as_ref()
            .and_then(|state| state.permanents.iter().find(|row| row.object == object))
        else {
            return false;
        };
        let Some(permanent) = self.state.objects.get(object.card as usize) else {
            return false;
        };
        if permanent.zone != Zone::Battlefield
            || self.object_ref(object.card) != object
            || permanent.controller != player
            || face_up_prohibited(self, object)
            || !(row.underlying.is_supported()
                || cfg!(feature = "test-support")
                    && (row.underlying
                        == Card::from_name("Bubble Smuggler").expect("pinned catalogue identity")
                        || row.underlying
                            == Card::from_name("Concealed Weapon")
                                .expect("pinned catalogue identity")))
        {
            return false;
        }
        let use_creature_mana = row.cause.allows_creature_mana()
            && row.underlying.is_creature()
            && !row.underlying.info().mana_cost.is_empty();
        let printed = || row.underlying.cost_options().filter(|_| use_creature_mana);
        let fundable = |cost: ManaCost| {
            self.can_fund_face_up(
                FundingGoal {
                    actor: player,
                    required: cost.colored,
                    generic: cost.generic,
                    reserve: None,
                    source: None,
                },
                object,
            )
        };
        if printed().any(|cost| fundable(cost) && emit(Action::BeginFaceUp { object, cost })) {
            return true;
        }
        let Some(spec) = self.face_up_disguise_spec(*row) else {
            return false;
        };
        let reduction = match spec.modifier {
            DisguiseCostModifier::GenericReduction(stat) => {
                u32::try_from(self.stat_value(stat, object, player).max(0)).unwrap_or(u32::MAX)
            }
            _ => 0,
        };
        std::iter::once(spec.mana)
            .chain(spec.alternatives.iter().copied())
            .map(|base| ManaCost {
                generic: base.generic.saturating_sub(reduction),
                ..base
            })
            .any(|cost| match spec.modifier {
                DisguiseCostModifier::X(x_symbols) => {
                    fundable(cost)
                        && emit(Action::BeginFaceUpX {
                            object,
                            cost,
                            x_symbols,
                        })
                }
                _ => {
                    !printed().any(|printed_cost| cost == printed_cost)
                        && fundable(cost)
                        && emit(Action::BeginFaceUp { object, cost })
                }
            })
    }
    pub(super) fn manifest_choice_view(&self, viewer: Player) -> Option<ManifestChoiceView> {
        let state = self.state.manifest.as_ref()?;
        let visible = |choice: &&ManifestChoice| self.shares_information(viewer, choice.player);
        let choice = state
            .pending_choice()
            .filter(visible)
            .or_else(|| state.choices.iter().find(visible))?;
        let candidates = choice
            .candidates
            .iter()
            .filter_map(|&reference| {
                let object = self.state.objects.get(reference.card as usize)?;
                (object.zone == Zone::Library && self.object_ref(reference.card) == reference)
                    .then_some((reference, object.card))
            })
            .collect();
        Some(ManifestChoiceView {
            player: choice.player,
            candidates,
        })
    }
    pub(super) fn face_down_spell_views(&self, viewer: Player) -> Vec<FaceDownPermanentView> {
        self.state.manifest.as_ref().map_or_else(Vec::new, |state| {
            state
                .spells
                .iter()
                .filter_map(|row| {
                    let object = self.state.objects.get(row.object.card as usize)?;
                    (object.zone == Zone::Stack && self.object_ref(row.object.card) == row.object)
                        .then(|| FaceDownPermanentView {
                            object: row.object,
                            cause: row.cause,
                            underlying: (self.state.outcome.is_some()
                                || row.known
                                || row.known_to & (1 << viewer.index()) != 0
                                || self.shares_information(viewer, object.controller))
                            .then_some(row.underlying),
                        })
                })
                .collect()
        })
    }
    pub(super) fn face_down_views(&self, viewer: Player) -> Vec<FaceDownPermanentView> {
        self.state.manifest.as_ref().map_or_else(Vec::new, |state| {
            state
                .permanents
                .iter()
                .filter_map(|row| {
                    let object = self.state.objects.get(row.object.card as usize)?;
                    if object.zone != Zone::Battlefield
                        || self.object_ref(row.object.card) != row.object
                    {
                        return None;
                    }
                    Some(FaceDownPermanentView {
                        object: row.object,
                        cause: row.cause,
                        underlying: (self.state.outcome.is_some()
                            || row.known
                            || row.known_to & (1 << viewer.index()) != 0
                            || self.shares_information(viewer, object.controller))
                        .then_some(row.underlying),
                    })
                })
                .collect()
        })
    }
}
