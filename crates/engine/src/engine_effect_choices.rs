//! Owned non-targeted effect choices and private resolution-cost payments.
use super::*;
use ability_world::CharacteristicWorld;
/// The chosen quantity and exact recipients survive interruptible draws.
#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct PowerDifferenceView {
    pub recipients: Box<[ObjectRef]>,
    pub amount: i128,
    pub source: ObjectRef,
}
#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) struct DiscardDrawChoice {
    players: [bool; 2],
    finished: [bool; 2],
    bonus: u16,
    limit: Option<usize>,
}
#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) struct RepeatedLifeChoice {
    remaining: u128,
    life: u16,
}
#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) struct EffectChoice {
    discard_draw: Option<DiscardDrawChoice>,
    pub discarded: Option<Box<[ObjectRef]>>,
    pub kind: EffectChoiceKind,
    pub filter: SacrificeFilter,
    pub amount: u32,
    pub source: ObjectRef,
    pub required: [u32; 2],
    pub selected: [Vec<ObjectRef>; 2],
    pub skip_to: Option<usize>,
    pub greatest_value: [Option<i128>; 2],
    pub discard_mana_value_life: Option<Player>,
    pub hand_defender: Option<CombatDefender>,
    pub entry_life_bonus: Option<(&'static str, u16)>,
    pub hand_entry_filter: Option<crate::SearchFilter>,
    pub equipment_recipient: Option<ObjectRef>,
    pub blight_copy_attacking: bool,
}
#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
impl EffectChoice {
    pub fn bytes(&self) -> usize {
        std::mem::size_of::<Self>()
            + (self.selected.iter().map(Vec::len).sum::<usize>()
                + self.discarded.as_ref().map_or(0, |rows| rows.len()))
                * std::mem::size_of::<ObjectRef>()
    }
}
#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
impl Game {
    pub(super) fn finish_power_difference_boost(&mut self) {
        let Some(effect) = self
            .state
            .resolving
            .as_mut()
            .and_then(|resolution| resolution.power_difference.take())
        else {
            return;
        };
        let recipients: Vec<_> = effect
            .recipients
            .iter()
            .copied()
            .filter(|recipient| {
                self.on_battlefield(recipient.card) && self.object_ref(recipient.card) == *recipient
            })
            .map(|recipient| recipient.card)
            .collect();
        for &id in &recipients {
            self.state.objects[id as usize].growth += Stats {
                power: effect.amount,
                toughness: effect.amount,
            };
        }
        self.grant_keyword_effects(&recipients, effect.source, Keywords::TRAMPLE);
    }
    pub(super) fn start_color_counter_choice(
        &mut self,
        color: crate::cards::ChoiceColor,
        subtype: &'static str,
    ) {
        let r = self.state.resolving.as_ref().unwrap();
        let sequence = r.color_sequence.as_ref().unwrap();
        let player = sequence.player;
        let mut required = [0; 2];
        required[player.index()] = sequence.selected.len() as u32 + 1;
        let mut selected = [Vec::new(), Vec::new()];
        selected[player.index()] = sequence
            .selected
            .iter()
            .map(|(_, object)| *object)
            .collect();
        let source = r.item.source.expect("object-backed color choice").object;
        self.state.resolving.as_mut().unwrap().choice = Some(Box::new(EffectChoice {
            discard_draw: None,
            discarded: None,
            kind: EffectChoiceKind::ColorCounter { color, subtype },
            filter: evidence::cost(0).filter,
            amount: 1,
            source,
            required,
            selected,
            skip_to: None,
            greatest_value: [None; 2],
            discard_mana_value_life: None,
            hand_defender: None,
            entry_life_bonus: None,
            hand_entry_filter: None,
            equipment_recipient: None,
            blight_copy_attacking: false,
        }));
        self.state.pending = Pending::EffectChoice { player };
    }
    pub(super) fn start_ward_discard_choice(&mut self, item: &SpellView, payer: Player) {
        let mut required = [0; 2];
        required[payer.index()] = 1;
        self.state.resolving.as_mut().unwrap().choice = Some(Box::new(EffectChoice {
            discard_draw: None,
            discarded: None,
            kind: EffectChoiceKind::Discard,
            filter: evidence::cost(0).filter,
            amount: 1,
            source: item
                .source
                .expect("effect choice requires object source")
                .object,
            required,
            selected: std::array::from_fn(|_| Vec::new()),
            skip_to: None,
            greatest_value: [None; 2],
            discard_mana_value_life: None,
            hand_defender: None,
            entry_life_bonus: None,
            hand_entry_filter: None,
            equipment_recipient: None,
            blight_copy_attacking: false,
        }));
    }
    pub(super) fn start_resolution_counter_payment(&mut self, item: &SpellView) -> bool {
        if !self.visit_resolution_counter_payments(item.controller, &mut |_| true) {
            return false;
        }
        let mut required = [0; 2];
        required[item.controller.index()] = 1;
        self.state.resolving.as_mut().unwrap().choice = Some(Box::new(EffectChoice {
            discard_draw: None,
            discarded: None,
            kind: EffectChoiceKind::RemoveCreatureCounters(1),
            filter: Self::creature_counter_spec(1).filter,
            amount: 1,
            source: item
                .source
                .expect("effect choice requires object source")
                .object,
            required,
            selected: std::array::from_fn(|_| Vec::new()),
            skip_to: None,
            greatest_value: [None; 2],
            discard_mana_value_life: None,
            hand_defender: None,
            entry_life_bonus: None,
            hand_entry_filter: None,
            equipment_recipient: None,
            blight_copy_attacking: false,
        }));
        self.state.pending = Pending::EffectChoice {
            player: item.controller,
        };
        true
    }
    pub(super) fn effect_choice_candidate(
        &self,
        choice: &EffectChoice,
        player: Player,
        object: ObjectRef,
    ) -> bool {
        self.effect_choice_candidate_inner(choice, player, object, true)
    }

    fn effect_choice_candidate_inner(
        &self,
        choice: &EffectChoice,
        player: Player,
        object: ObjectRef,
        enforce_sacrifice_prohibition: bool,
    ) -> bool {
        if choice.kind == EffectChoiceKind::KeepSharedCreatureTypes {
            return self.keep_shared_type_candidate(choice, object);
        }
        if choice.kind == EffectChoiceKind::TurnControlledCreatureFaceUp {
            let card = &self.state.objects[object.card as usize];
            return card.zone == Zone::Battlefield
                && card.controller == player
                && self.object_ref(object.card) == object
                && self.is_face_down(object)
                && self.is_creature(object.card)
                && !manifest::face_up_prohibited(self, object);
        }
        if choice.kind == EffectChoiceKind::CloakFromHand {
            let card = &self.state.objects[object.card as usize];
            return card.zone == Zone::Hand
                && card.owner == player
                && card.is_card()
                && self.object_ref(object.card) == object
                && choice.selected[player.index()].is_empty();
        }
        if let EffectChoiceKind::SaddleRider(effect) = choice.kind {
            return !choice.selected[player.index()].contains(&object)
                && self.saddle_rider_candidate(choice.source, player, object, effect);
        }
        if choice.kind.payment_kind() == EffectChoiceKind::RemoveCreatureCounters(1) {
            return self.resolution_counter_candidate(player, object)
                && self.counter_amounts(object.card).has_any();
        }
        if matches!(
            choice.kind.payment_kind(),
            EffectChoiceKind::TargetAttackingReturn { .. }
        ) {
            return false;
        }
        let excluded = match choice.kind {
            EffectChoiceKind::SacrificeExceptAttackingAttachment => self
                .state
                .resolving
                .as_ref()
                .unwrap()
                .item
                .attachment_trigger
                .as_ref()
                .map(|event| event.recipient),
            EffectChoiceKind::SacrificeExceptSourceAttachment => self
                .attached_permanent(&self.state.resolving.as_ref().unwrap().item)
                .map(|id| self.object_ref(id)),
            _ => None,
        };
        if excluded == Some(object) {
            return false;
        }
        let o = &self.state.objects[object.card as usize];
        if choice.kind == EffectChoiceKind::DestroyNonbasicLandsForEachPlayer {
            return self.nonbasic_land_choice(object)
                && choice.selected[player.index()].iter().all(|selected| {
                    self.state.objects[selected.card as usize].controller != o.controller
                });
        }
        if let EffectChoiceKind::KeepCreaturesByPowerForEachPlayer(maximum) = choice.kind {
            return self.object_ref(object.card) == object
                && self.creature_survives_sweep(object.card, maximum)
                && choice.selected[player.index()].iter().all(|survivor| {
                    self.state.objects[survivor.card as usize].controller != o.controller
                });
        }
        if let EffectChoiceKind::ReturnAttachmentRecipient { attachment } = choice.kind {
            return choice.selected[player.index()].is_empty()
                && self.returned_attachment_recipient(attachment, object, player);
        }
        if choice.kind == EffectChoiceKind::EquipmentRecipient {
            return o.controller == player
                && choice.selected[player.index()].is_empty()
                && self.equipment_can_attach(choice.source, object);
        }
        if let EffectChoiceKind::ReflexiveEquipmentRecipient { subtype } = choice.kind {
            return self.on_battlefield(object.card)
                && o.controller == player
                && choice.selected[player.index()].is_empty()
                && self.has_subtype(object.card, subtype)
                && self
                    .reflexive_equipment_for(&self.state.resolving.as_ref().unwrap().item, object)
                    .next()
                    .is_some();
        }
        if let EffectChoiceKind::AttachedEquipmentToDestroy { recipient } = choice.kind {
            return choice.selected[player.index()].is_empty()
                && self.state.resolving.as_ref().is_some_and(|resolution| {
                    self.equipment_attached_to_target(&resolution.item, recipient, object)
                });
        }
        if matches!(choice.kind, EffectChoiceKind::LinkedCreature { .. }) {
            return o.controller != player
                && self.on_battlefield(object.card)
                && self.object_ref(object.card) == object
                && self.is_creature(object.card)
                && choice.selected[player.index()].is_empty();
        }
        if matches!(
            choice.kind,
            EffectChoiceKind::Blight(_) | EffectChoiceKind::BlightCost(_)
        ) {
            return o.zone == Zone::Battlefield
                && o.controller == player
                && self.object_ref(object.card) == object
                && self.is_creature(object.card)
                && !choice.selected[player.index()].contains(&object)
                && (matches!(
                    choice.kind,
                    EffectChoiceKind::Blight(_) | EffectChoiceKind::BlightCost(0)
                ) || !replacements::counters_prohibited(self, object));
        }
        if let EffectChoiceKind::ColorCounter { color, subtype } = choice.kind {
            return self.color_counter_candidate(player, color, subtype, object);
        }
        if let EffectChoiceKind::CountersOnTarget { group, .. }
        | EffectChoiceKind::DamageAmongTargetGroup { group, .. } = choice.kind
        {
            // Resolution has already discarded illegal targets. This choice
            // follows those exact permanents through any earlier instructions.
            return o.zone == Zone::Battlefield
                && self.object_ref(object.card) == object
                && !choice.selected[player.index()].contains(&object)
                && self
                    .state
                    .resolving
                    .as_ref()
                    .unwrap()
                    .item
                    .targets
                    .iter()
                    .any(|bound| {
                        bound.group == group
                            && bound.target == Target::Card(object.card)
                            && bound.object == Some(object)
                    });
        }
        if choice.kind == EffectChoiceKind::ReturnCraftedCreatureAttacking {
            return choice.selected[player.index()].is_empty()
                && self.crafted_exiled_creature(choice.source, object);
        }
        if choice.kind == EffectChoiceKind::ExileFaceUpDisguiseCreatures {
            return !choice.selected[player.index()].contains(&object)
                && self.face_up_disguise_creature(player, object);
        }
        if choice.kind == EffectChoiceKind::TransferSuspectToEventCreature {
            let item = &self.state.resolving.as_ref().unwrap().item;
            return player == item.controller
                && choice.selected[player.index()].is_empty()
                && self.can_transfer_suspect(item, object);
        }
        if choice.kind.payment_kind() == EffectChoiceKind::CopyExiledTriggerCreature {
            return o.zone == Zone::Exile
                && o.is_card()
                && self.object_ref(object.card) == object
                && self.current_kinds(object.card) & 1 != 0
                && self
                    .state
                    .resolving
                    .as_ref()
                    .unwrap()
                    .item
                    .exiled_trigger_cards
                    .as_ref()
                    .is_some_and(|cards| cards.contains(&object))
                && !choice.selected[player.index()].contains(&object);
        }
        if choice.kind.payment_kind() == EffectChoiceKind::CoverUpGraveyardCard {
            return o.zone == Zone::Graveyard
                && o.owner != player
                && o.is_card()
                && self.object_ref(object.card) == object
                && !choice.selected[player.index()].contains(&object);
        }
        if let EffectChoiceKind::NamedSearch {
            owner,
            name,
            maximum,
        } = choice.kind.payment_kind()
        {
            return choice.selected[player.index()].len() < usize::from(maximum)
                && o.owner == owner
                && o.is_card()
                && matches!(o.zone, Zone::Graveyard | Zone::Hand | Zone::Library)
                && self.object_ref(object.card) == object
                && outside_names::matches(o.card, name)
                && !choice.selected[player.index()].contains(&object);
        }
        if let EffectChoiceKind::CoverUpSearch { owner, name } = choice.kind.payment_kind() {
            return o.owner == owner
                && o.is_card()
                && matches!(o.zone, Zone::Graveyard | Zone::Hand | Zone::Library)
                && self.object_ref(object.card) == object
                && crate::card_names::same_name(o.card.name(), name)
                && !choice.selected[player.index()].contains(&object);
        }
        if let EffectChoiceKind::MeldPartner(pair) = choice.kind {
            return meld::matching_partner(
                self,
                choice.source,
                object,
                player,
                pair.partner.name(),
            ) && !choice.selected[player.index()].contains(&object);
        }
        if choice.kind == EffectChoiceKind::ReturnManifestGraveyardToHand {
            return o.zone == Zone::Graveyard
                && o.owner == player
                && o.is_card()
                && self.object_ref(object.card) == object
                && choice.selected[player.index()].is_empty()
                && self.state.resolving.as_ref().is_some_and(|resolution| {
                    resolution
                        .item
                        .manifest_event
                        .as_ref()
                        .is_some_and(|event| {
                            event.player == player && event.graveyard_cards.contains(&object)
                        })
                });
        }
        if let EffectChoiceKind::ReturnGraveyardToHandMatching(filter) = choice.kind {
            return o.zone == Zone::Graveyard
                && o.owner == player
                && o.is_card()
                && self.object_ref(object.card) == object
                && self.card_search_filter_matches(filter, player, object.card)
                && !choice.selected[player.index()].contains(&object);
        }
        let graveyard_filter = match choice.kind {
            EffectChoiceKind::ReturnGraveyardToHand { kinds }
            | EffectChoiceKind::ReturnUpToGraveyardToHand { kinds } => Some((kinds, None)),
            EffectChoiceKind::ReturnGraveyardToHandOrSubtype { kinds, subtype } => {
                Some((kinds, Some(subtype)))
            }
            _ => None,
        };
        if let Some((kinds, subtype)) = graveyard_filter {
            return o.zone == Zone::Graveyard
                && o.owner == player
                && o.is_card()
                && self.object_ref(object.card) == object
                && ((kinds == 0 && subtype.is_none())
                    || self.current_kinds(object.card) & kinds != 0
                    || subtype.is_some_and(|subtype| self.has_subtype(object.card, subtype)))
                && !choice.selected[player.index()].contains(&object);
        }
        if choice.kind.payment_kind() == EffectChoiceKind::ExileCardFromAnyGraveyard {
            return o.zone == Zone::Graveyard
                && o.is_card()
                && self.object_ref(object.card) == object
                && !choice.selected[player.index()].contains(&object)
                && (choice.kind != EffectChoiceKind::ExileCardFromAnyGraveyardRemember
                    || !ability_world::CharacteristicWorld::graveyard_exile_prohibited(
                        self,
                        object.card,
                    ));
        }
        if matches!(
            choice.kind.payment_kind(),
            EffectChoiceKind::DiscardedLand { .. }
        ) {
            return self.state.resolving.as_ref().unwrap().item.controller == player
                && self.object_ref(object.card) == object
                && choice
                    .discarded
                    .as_ref()
                    .is_some_and(|rows| rows.contains(&object))
                && matches!(
                    o.zone,
                    Zone::Graveyard | Zone::Exile | Zone::Battlefield | Zone::Stack | Zone::Command
                )
                && self.current_kinds(object.card) & 2 != 0
                && choice.selected[player.index()].is_empty();
        }
        if let EffectChoiceKind::ExileSourceAndNamed { name, .. } = choice.kind {
            return additional_costs::graveyard_exile_card_matches(
                self,
                player,
                object,
                Some(name),
            ) && !choice.selected[player.index()].contains(&object);
        }
        if matches!(
            choice.kind,
            EffectChoiceKind::ExileGraveyardCreaturesReflexive { .. }
        ) {
            return o.zone == Zone::Graveyard
                && o.owner == player
                && o.is_card()
                && o.card.is_creature()
                && self.object_ref(object.card) == object
                && !choice.selected[player.index()].contains(&object)
                && !ability_world::CharacteristicWorld::graveyard_exile_prohibited(
                    self,
                    object.card,
                );
        }
        if choice.kind.payment_kind() == EffectChoiceKind::ExileGraveyard {
            if ability_world::CharacteristicWorld::graveyard_exile_prohibited(self, object.card) {
                return false;
            }
            return !choice.selected[player.index()].contains(&object)
                && self.additional_candidate(
                    AdditionalCost {
                        kind: choice.kind,
                        filter: choice.filter,
                    },
                    player,
                    choice.source,
                    object,
                );
        }
        if matches!(
            choice.kind.payment_kind(),
            EffectChoiceKind::TapPermanents(_) | EffectChoiceKind::TapAnyUntappedPermanents
        ) && o.tapped
        {
            return false;
        }
        if let EffectChoiceKind::UntapLands { maximum } = choice.kind.payment_kind() {
            return o.zone == Zone::Battlefield
                && self.untap_allowed(object.card, untap_rules::UntapCause::Effect)
                && self.current_kinds(object.card) & 2 != 0
                && self.object_ref(object.card) == object
                && !choice.selected[player.index()].contains(&object)
                && choice.selected[player.index()].len() < usize::from(maximum);
        }
        if let EffectChoiceKind::GraveyardCreatures {
            maximum,
            total_mana_value,
        } = choice.kind.payment_kind()
        {
            let selected = &choice.selected[player.index()];
            let spent = selected.iter().fold(0u64, |sum, r| {
                sum.saturating_add(self.state.objects[r.card as usize].card.fixed_mana_value())
            });
            return o.zone == Zone::Graveyard
                && (o.owner == player || self.graveyard_return_allows_either_owner())
                && o.is_card()
                && o.card.is_creature()
                && self.object_ref(object.card) == object
                && !selected.contains(&object)
                && selected.len() < maximum as usize
                && o.card.fixed_mana_value() <= total_mana_value.saturating_sub(spent)
                && (!self.target_graveyard_return_active()
                    || self
                        .state
                        .resolving
                        .as_ref()
                        .unwrap()
                        .item
                        .targets
                        .iter()
                        .any(|bound| bound.object == Some(object)));
        }
        if choice.kind == EffectChoiceKind::ReturnRememberedCreature {
            return o.zone == Zone::Graveyard
                && o.is_card()
                && o.card.is_creature()
                && self.object_ref(object.card) == object
                && choice.selected[player.index()].is_empty()
                && self
                    .state
                    .resolving
                    .as_ref()
                    .unwrap()
                    .remembered_objects
                    .as_ref()
                    .is_some_and(|receipts| receipts.binary_search(&object).is_ok());
        }
        if let EffectChoiceKind::LandCards { maximum, .. } = choice.kind.payment_kind() {
            return o.owner == player
                && o.is_card()
                && self.object_ref(object.card) == object
                && self.land_selection_zone(o.zone)
                && o.card.is_land()
                && !choice.selected[player.index()].contains(&object)
                && choice.selected[player.index()].len() < maximum as usize;
        }
        if let EffectChoiceKind::DeployFromHandOrGraveyard { filter, .. } = choice.kind {
            return o.owner == player
                && o.is_card()
                && self.object_ref(object.card) == object
                && matches!(o.zone, Zone::Hand | Zone::Graveyard)
                && o.card.is_creature()
                && self.card_search_filter_matches(filter, player, object.card)
                && choice.selected[player.index()].is_empty();
        }
        if let EffectChoiceKind::HandCreatures { maximum } = choice.kind {
            return o.zone == Zone::Hand
                && o.owner == player
                && o.is_card()
                && o.card.is_creature()
                && self.object_ref(object.card) == object
                && !choice.selected[player.index()].contains(&object)
                && choice.selected[player.index()].len() < usize::from(maximum);
        }
        if choice.kind == EffectChoiceKind::BlinkControlledCreatures {
            return o.zone == Zone::Battlefield
                && o.controller == player
                && self.is_creature(object.card)
                && self.object_ref(object.card) == object
                && !choice.selected[player.index()].contains(&object);
        }
        if let EffectChoiceKind::MoveFaceUpExileToGraveyard { owner } = choice.kind {
            return o.zone == Zone::Exile
                && o.owner == owner
                && o.is_card()
                && self.object_ref(object.card) == object
                && !self.exiled_face_down(object)
                && choice.selected[player.index()].is_empty();
        }
        if let EffectChoiceKind::SacrificeOrDiscard { nonland } = choice.kind.payment_kind()
            && o.zone == Zone::Hand
        {
            return o.owner == player
                && o.is_card()
                && self.object_ref(object.card) == object
                && !choice.selected[player.index()].contains(&object)
                && (!nonland || !o.card.is_land());
        }
        let hand_choice = matches!(
            choice.kind.payment_kind(),
            EffectChoiceKind::Discard
                | EffectChoiceKind::ExileFromHand
                | EffectChoiceKind::HandToTop
                | EffectChoiceKind::BottomHand
                | EffectChoiceKind::HandExileCopy
                | EffectChoiceKind::HandPlot { .. }
                | EffectChoiceKind::HandCreature
                | EffectChoiceKind::HandCreatureAttacking { .. }
                | EffectChoiceKind::HandCreatureHasteSacrifice { .. }
                | EffectChoiceKind::HandCreatureAndLand
                | EffectChoiceKind::HandPermanent { .. }
                | EffectChoiceKind::HandLand { .. }
        );
        let entitled_player = if hand_choice { o.owner } else { o.controller };
        if entitled_player != player
            || self.object_ref(object.card) != object
            || choice.selected[player.index()].contains(&object)
        {
            return false;
        }
        if matches!(
            choice.kind.payment_kind(),
            EffectChoiceKind::Discard
                | EffectChoiceKind::ExileFromHand
                | EffectChoiceKind::HandToTop
                | EffectChoiceKind::BottomHand
                | EffectChoiceKind::HandExileCopy
                | EffectChoiceKind::HandPlot { .. }
                | EffectChoiceKind::HandCreature
                | EffectChoiceKind::HandCreatureAttacking { .. }
                | EffectChoiceKind::HandCreatureHasteSacrifice { .. }
                | EffectChoiceKind::HandCreatureAndLand
                | EffectChoiceKind::HandPermanent { .. }
                | EffectChoiceKind::HandLand { .. }
        ) {
            return o.zone == Zone::Hand
                && o.is_card()
                && match choice.kind.payment_kind() {
                    // Hand costs use the same typed characteristics and filters as
                    // announced discard costs, including artifact lands in hand.
                    EffectChoiceKind::Discard => match choice.kind {
                        EffectChoiceKind::DiscardOneMatchingOr { filter, .. } => {
                            self.card_search_filter_matches(filter, player, object.card)
                        }
                        _ => self.additional_candidate(
                            AdditionalCost {
                                kind: choice.kind,
                                filter: choice.filter,
                            },
                            player,
                            choice.source,
                            object,
                        ),
                    },
                    EffectChoiceKind::HandCreature => o.card.is_creature(),
                    EffectChoiceKind::HandCreatureAttacking { maximum } => {
                        o.card.is_creature() && o.card.fixed_mana_value() <= u64::from(maximum)
                    }
                    EffectChoiceKind::HandCreatureHasteSacrifice {
                        maximum_total_stats,
                    } => {
                        let view = self.object_view(object.card);
                        view.is_creature()
                            && view
                                .power
                                .checked_add(view.toughness)
                                .expect("total statistics")
                                .max(0)
                                <= i128::from(maximum_total_stats)
                    }
                    EffectChoiceKind::ExileFromHand => o.is_card(),
                    EffectChoiceKind::HandCreatureAndLand => {
                        match choice.selected[player.index()].as_slice() {
                            [] => o.card.is_creature() || o.card.is_land(),
                            [first] => {
                                let first = self.state.objects[first.card as usize].card;
                                (first.is_creature() && o.card.is_land())
                                    || (first.is_land() && o.card.is_creature())
                            }
                            _ => false,
                        }
                    }
                    EffectChoiceKind::HandPermanent { maximum, .. } => {
                        o.card.kind_mask() & (1 | 2 | 16 | 32 | 64 | 256) != 0
                            && o.card.fixed_mana_value() <= maximum
                            && choice.hand_entry_filter.is_none_or(|filter| {
                                self.card_search_filter_matches(filter, player, object.card)
                                    && self.hand_entry_payment_legal(player, object)
                            })
                    }
                    EffectChoiceKind::HandExileCopy => o.card.is_creature() || o.card.is_artifact(),
                    EffectChoiceKind::HandPlot { maximum } => {
                        !o.card.is_land() && o.card.fixed_mana_value() <= u64::from(maximum)
                    }
                    EffectChoiceKind::HandLand { .. } => o.card.is_land() && o.controller == player,
                    _ => true,
                };
        }
        if choice.greatest_value[player.index()].is_some_and(|max| {
            let value = if matches!(
                choice.kind.payment_kind(),
                EffectChoiceKind::ExileGreatestManaValue
                    | EffectChoiceKind::SacrificeGreatestManaValue
            ) {
                i128::from(self.object_mana_value(object.card))
            } else {
                self.object_view(object.card).power
            };
            value != max
        }) {
            return false;
        }
        let f = choice.filter;
        let kinds = self.current_kinds(object.card);
        if !choice.kind.cost_matches_any(
            o.card,
            kinds,
            self.granted_subtypes(object.card),
            o.is_token(),
            o.dealt_damage_this_turn,
            || ability_world::CharacteristicWorld::legendary(self, object.card),
            || self.is_suspected(object.card),
        ) {
            return false;
        }
        if matches!(
            choice.kind.payment_kind(),
            EffectChoiceKind::Sacrifice
                | EffectChoiceKind::SacrificeGreatestManaValue
                | EffectChoiceKind::SacrificeDifferentNames(_)
                | EffectChoiceKind::SacrificeOrDiscard { .. }
        ) && enforce_sacrifice_prohibition
            && self.sacrifice_prohibited(object.card)
        {
            return false;
        }
        o.zone == Zone::Battlefield
            && f.matches_kinds_and_token(kinds, o.is_token())
            && self.keywords(object.card).contains(f.keywords)
            && (!f.source_only || object == choice.source)
            && (!f.other || object != choice.source)
            && f.subtype
                .is_none_or(|subtype| self.has_subtype(object.card, subtype))
            && (!matches!(
                choice.kind,
                EffectChoiceKind::RemoveLoreForSourceCounters(_)
            ) || (self.counter_count(object.card, CounterKind::Lore) > 0
                && self.has_subtype(object.card, "Saga")))
    }

    pub(super) fn start_half_choice(
        &mut self,
        item: &SpellView,
        players: PlayerSet,
        kind: EffectChoiceKind,
        filter: SacrificeFilter,
        round_up: bool,
    ) -> bool {
        let payment = kind.payment_kind();
        assert!(matches!(
            payment,
            EffectChoiceKind::Discard | EffectChoiceKind::Sacrifice
        ));
        let mut choice = EffectChoice {
            discard_draw: None,
            discarded: None,
            kind,
            filter,
            amount: 0,
            source: item
                .source
                .expect("effect choice requires object source")
                .object,
            required: [0; 2],
            selected: std::array::from_fn(|_| Vec::new()),
            skip_to: None,
            greatest_value: [None; 2],
            discard_mana_value_life: None,
            hand_defender: None,
            entry_life_bonus: None,
            hand_entry_filter: None,
            equipment_recipient: None,
            blight_copy_attacking: false,
        };
        // Freeze all quotas before choices or replacements can move anything.
        for player in self.program_players(players, item) {
            let mut matching = 0usize;
            let mut available = 0usize;
            let zone = if payment == EffectChoiceKind::Discard {
                Zone::Hand
            } else {
                Zone::Battlefield
            };
            for (id, candidate) in self.state.objects.iter().enumerate() {
                if candidate.zone != zone {
                    continue;
                }
                let object = self.object_ref(id as CardId);
                if self.effect_choice_candidate_inner(&choice, player, object, false) {
                    matching += 1;
                    if payment != EffectChoiceKind::Sacrifice
                        || !self.sacrifice_prohibited(object.card)
                    {
                        available += 1;
                    }
                }
            }
            // Prohibited permanents count toward the amount to sacrifice.
            // If there are too few legal selections, do as much as possible.
            let half = if round_up {
                matching.div_ceil(2)
            } else {
                matching / 2
            };
            choice.required[player.index()] =
                u32::try_from(half.min(available)).capacity("half choice count exhausted");
        }
        self.state.resolving.as_mut().unwrap().choice = Some(Box::new(choice));
        self.advance_effect_choice()
    }
    pub(super) fn start_discard_any_draw(
        &mut self,
        item: &SpellView,
        players: PlayerSet,
        bonus: u16,
        required_all: bool,
    ) -> bool {
        let mut participants = [false; 2];
        for player in self.program_players(players, item) {
            participants[player.index()] = true;
        }
        if required_all {
            let ids: Vec<_> = self
                .state
                .objects
                .iter()
                .enumerate()
                .filter(|(_, o)| {
                    o.zone == Zone::Hand && o.is_card() && participants[o.owner.index()]
                })
                .map(|(id, _)| id as CardId)
                .collect();
            self.discard_batch_with_after(
                &ids,
                None,
                Some(item.controller),
                Some(discard_events::DiscardAfter::DrawActual {
                    players: participants,
                    bonus,
                }),
            );
            return false;
        }
        let pc = self.state.resolving.as_ref().unwrap().pc;
        self.state.resolving.as_mut().unwrap().choice = Some(Box::new(EffectChoice {
            discarded: None,
            discard_draw: Some(DiscardDrawChoice {
                players: participants,
                finished: [false; 2],
                bonus,
                limit: None,
            }),
            kind: EffectChoiceKind::Discard,
            filter: evidence::cost(0).filter,
            amount: 0,
            source: item
                .source
                .expect("effect choice requires object source")
                .object,
            required: [0; 2],
            selected: std::array::from_fn(|_| Vec::new()),
            skip_to: Some(pc),
            greatest_value: [None; 2],
            discard_mana_value_life: None,
            hand_defender: None,
            entry_life_bonus: None,
            hand_entry_filter: None,
            equipment_recipient: None,
            blight_copy_attacking: false,
        }));
        self.advance_effect_choice()
    }
    pub(super) fn start_discard_up_to_one_draw(&mut self, item: &SpellView) -> bool {
        self.start_discard_up_to_draw(item, 1)
    }
    pub(super) fn start_discard_up_to_draw(&mut self, item: &SpellView, maximum: u16) -> bool {
        self.start_discard_any_draw(item, PlayerSet::Controller, 0, false);
        if let Some(choice) = self
            .state
            .resolving
            .as_mut()
            .and_then(|r| r.choice.as_mut())
            && let Some(draw) = &mut choice.discard_draw
        {
            draw.limit = Some(usize::from(maximum));
        }
        self.advance_effect_choice()
    }
    pub(super) fn start_greatest_power_choice(&mut self, item: &SpellView) -> bool {
        let mut filter = evidence::cost(0).filter;
        filter.kinds = 1;
        self.start_effect_choice_inner(
            item,
            EffectChoiceKind::Sacrifice,
            PlayerSet::Opponent,
            filter,
            1,
            None,
            true,
            None,
        )
    }
    fn creature_survives_sweep(&self, card: CardId, maximum: i128) -> bool {
        self.on_battlefield(card) && self.is_creature(card) && self.stats(card).power <= maximum
    }

    pub(super) fn start_repeated_life_choice(&mut self, repetitions: u128, life: u16) {
        if repetitions != 0 {
            self.state.resolving.as_mut().unwrap().repeated_life_choice =
                Some(Box::new(RepeatedLifeChoice {
                    remaining: repetitions,
                    life,
                }));
        }
    }
    pub(super) fn advance_repeated_life_choice(&mut self) -> bool {
        let resolution = self.state.resolving.as_mut().unwrap();
        let Some(sequence) = &mut resolution.repeated_life_choice else {
            return false;
        };
        if sequence.remaining == 0 {
            resolution.repeated_life_choice = None;
            return false;
        }
        sequence.remaining -= 1;
        let item = resolution.item.clone();
        self.start_effect_choice(
            &item,
            EffectChoiceKind::SacrificeOrDiscard { nonland: false },
            PlayerSet::Opponent,
            SacrificeFilter {
                nonland: true,
                ..evidence::cost(0).filter
            },
            1,
            Some(0),
        )
    }

    pub(super) fn start_effect_choice(
        &mut self,
        item: &SpellView,
        kind: EffectChoiceKind,
        players: PlayerSet,
        filter: SacrificeFilter,
        amount: u16,
        optional_skip: Option<usize>,
    ) -> bool {
        self.start_effect_choice_inner(
            item,
            kind,
            players,
            filter,
            u32::from(amount),
            optional_skip,
            false,
            None,
        )
    }
    pub(super) fn start_scaled_sacrifice(
        &mut self,
        item: &SpellView,
        players: PlayerSet,
        filter: SacrificeFilter,
        amount: AmountValue,
    ) -> bool {
        let count = u32::try_from(self.item_amount(amount, item)).unwrap_or(u32::MAX);
        count != 0
            && self.start_effect_choice_inner(
                item,
                EffectChoiceKind::Sacrifice,
                players,
                filter,
                count,
                None,
                false,
                None,
            )
    }
    fn hand_entry_payment_legal(&self, player: Player, object: ObjectRef) -> bool {
        let o = &self.state.objects[object.card as usize];
        o.zone == Zone::Hand
            && o.owner == player
            && self.object_ref(object.card) == object
            && o.is_card()
            && (o.card.aura().is_none()
                || self.state.objects.iter().enumerate().any(|(id, _)| {
                    self.aura_can_enchant(o.card, player, self.object_ref(id as CardId))
                }))
    }
    pub(super) fn start_filtered_hand_entry(
        &mut self,
        item: &SpellView,
        filter: crate::SearchFilter,
        skip: usize,
    ) -> bool {
        // HandPermanent always retains its private optional phase even when empty.
        // Eligibility never changes the public phase or mandatory choice quota.
        let pending = self.start_effect_choice_inner(
            item,
            EffectChoiceKind::HandPermanent {
                maximum: u64::MAX,
                tapped: false,
            },
            PlayerSet::Controller,
            evidence::cost(0).filter,
            1,
            Some(0),
            false,
            None,
        );
        let resolution = self.state.resolving.as_mut().unwrap();
        let choice = resolution.choice.as_mut().expect("private hand phase");
        choice.hand_entry_filter = Some(filter);
        choice.skip_to = Some(resolution.pc.checked_add(skip).expect("program offset"));
        assert!(choice.skip_to.unwrap() <= resolution.program.len());
        pending
    }
    pub(super) fn start_hand_creature_haste_sacrifice(
        &mut self,
        item: &SpellView,
        maximum_total_stats: u64,
    ) -> bool {
        self.start_effect_choice_inner(
            item,
            EffectChoiceKind::HandCreatureHasteSacrifice {
                maximum_total_stats,
            },
            PlayerSet::Controller,
            evidence::cost(0).filter,
            1,
            Some(0),
            false,
            None,
        )
    }
    pub(super) fn start_hand_creature(&mut self, item: &SpellView) -> bool {
        self.start_effect_choice_inner(
            item,
            EffectChoiceKind::HandCreature,
            PlayerSet::Controller,
            evidence::cost(0).filter,
            1,
            Some(0),
            false,
            None,
        )
    }
    pub(super) fn start_hand_creatures(&mut self, item: &SpellView, maximum: u16) -> bool {
        maximum > 0
            && self.start_effect_choice_inner(
                item,
                EffectChoiceKind::HandCreatures { maximum },
                PlayerSet::Controller,
                evidence::cost(0).filter,
                0,
                None,
                false,
                None,
            )
    }
    pub(super) fn finish_hand_creatures(&mut self) {
        let resolution = self.state.resolving.as_mut().unwrap();
        let player = resolution.item.controller;
        let choice = resolution.choice.take().unwrap();
        let moves = choice.selected[player.index()]
            .iter()
            .map(|reference| {
                let mut movement = self.entry_move(reference.card, Zone::Battlefield, false);
                movement.controller = player;
                movement
            })
            .collect::<Vec<_>>();
        if !moves.is_empty() {
            self.stage_entries(moves, None);
        }
    }
    pub(super) fn finish_repeat_blink_choice(&mut self) {
        let resolution = self.state.resolving.as_mut().unwrap();
        let player = resolution.item.controller;
        let choice = resolution.choice.take().unwrap();
        let ids = choice.selected[player.index()]
            .iter()
            .map(|reference| reference.card)
            .collect::<Vec<_>>();
        let remaining = resolution
            .repeat_blink_remaining
            .take()
            .expect("repeat blink count");
        if remaining > 0 {
            resolution.repeat_blink_remaining = Some(remaining - 1);
            resolution.pc -= 1;
        }
        if !ids.is_empty() {
            self.blink_battlefield_objects_immediately(&ids);
        }
    }
    pub(super) fn start_hand_permanent(
        &mut self,
        item: &SpellView,
        maximum: AmountValue,
        tapped: bool,
    ) -> bool {
        let maximum = u64::try_from(self.item_amount(maximum, item)).unwrap_or(u64::MAX);
        self.start_effect_choice_inner(
            item,
            EffectChoiceKind::HandPermanent { maximum, tapped },
            PlayerSet::Controller,
            evidence::cost(0).filter,
            1,
            Some(0),
            false,
            None,
        )
    }
    pub(super) fn start_hand_creature_attacking(
        &mut self,
        item: &SpellView,
        maximum: AmountValue,
    ) -> bool {
        let maximum = u32::try_from(self.item_amount(maximum, item)).unwrap_or(u32::MAX);
        self.start_effect_choice_inner(
            item,
            EffectChoiceKind::HandCreatureAttacking { maximum },
            PlayerSet::Controller,
            evidence::cost(0).filter,
            1,
            Some(0),
            false,
            None,
        )
    }
    pub(super) fn start_hand_land(&mut self, item: &SpellView, tapped: bool) -> bool {
        self.start_effect_choice_inner(
            item,
            EffectChoiceKind::HandLand { tapped },
            PlayerSet::Controller,
            evidence::cost(0).filter,
            1,
            Some(0),
            false,
            None,
        )
    }
    pub(super) fn start_graveyard_return_selection(
        &mut self,
        item: &SpellView,
        maximum: u32,
        optional_mana_budget: Option<u64>,
    ) -> bool {
        if maximum == 0 {
            return false;
        }
        self.start_effect_choice_inner(
            item,
            EffectChoiceKind::GraveyardCreatures {
                maximum,
                total_mana_value: optional_mana_budget.unwrap_or(u64::MAX),
            },
            PlayerSet::Controller,
            evidence::cost(0).filter,
            0,
            optional_mana_budget.map(|_| 0),
            false,
            None,
        )
    }

    fn target_graveyard_return_active(&self) -> bool {
        self.state.resolving.as_ref().is_some_and(|resolution| {
            matches!(
                resolution.program.get(resolution.pc.saturating_sub(1)),
                Some(Instruction::ReturnGraveyardTargetsToHandOrBattlefield { .. })
            )
        })
    }

    fn graveyard_return_allows_either_owner(&self) -> bool {
        self.state.resolving.as_ref().is_some_and(|resolution| {
            matches!(
                resolution.program.get(resolution.pc.saturating_sub(1)),
                Some(Instruction::ChooseGraveyardReturnFromEither)
            )
        })
    }

    fn graveyard_return_targets<'a>(
        &'a self,
        item: &'a SpellView,
    ) -> impl Iterator<Item = ObjectRef> + 'a {
        item.targets
            .iter()
            .filter_map(|bound| bound.object)
            .filter(move |&object| {
                let card = &self.state.objects[object.card as usize];
                self.object_ref(object.card) == object
                    && card.zone == Zone::Graveyard
                    && card.owner == item.controller
                    && card.is_card()
            })
    }

    fn target_graveyard_return_entries(
        &self,
        item: &SpellView,
        battlefield: Option<ObjectRef>,
    ) -> Vec<entries::EntryMove> {
        let mut moves = Vec::<entries::EntryMove>::new();
        for object in self.graveyard_return_targets(item) {
            if moves
                .iter()
                .any(|movement| movement.before.reference == object)
            {
                continue;
            }
            let destination = if battlefield == Some(object) {
                Zone::Battlefield
            } else {
                Zone::Hand
            };
            let mut movement = self.entry_move(object.card, destination, false);
            movement.controller = item.controller;
            moves.push(movement);
        }
        moves
    }

    pub(super) fn start_target_graveyard_return(
        &mut self,
        item: &SpellView,
        maximum_mana_value: u64,
        bargain_only: bool,
    ) -> bool {
        if (!bargain_only || item.cast.is_some_and(|cast| cast.bargain == Some(true)))
            && self.graveyard_return_targets(item).any(|object| {
                let card = self.state.objects[object.card as usize].card;
                card.is_creature() && card.fixed_mana_value() <= maximum_mana_value
            })
        {
            return self.start_graveyard_return_selection(item, 1, Some(maximum_mana_value));
        }
        let moves = self.target_graveyard_return_entries(item, None);
        if !moves.is_empty() {
            self.stage_entries(moves, None);
        }
        false
    }

    pub(super) fn finish_graveyard_return_selection(&mut self) {
        let choice = self
            .state
            .resolving
            .as_mut()
            .unwrap()
            .choice
            .take()
            .unwrap();
        let resolution = self.state.resolving.as_ref().unwrap();
        let item = &resolution.item;
        let player = item.controller;
        let entry_counter = match resolution.program.get(resolution.pc.saturating_sub(1)) {
            Some(Instruction::ChooseGraveyardReturnWithCounter(counter)) => Some((*counter, 1)),
            _ => None,
        };
        if matches!(
            choice.kind,
            EffectChoiceKind::ReturnUpToGraveyardToHand { .. }
        ) {
            self.return_effect_choice_to_hand(*choice);
            return;
        }
        if self.target_graveyard_return_active() {
            let battlefield = choice.selected[player.index()].first().copied();
            let moves = self.target_graveyard_return_entries(item, battlefield);
            if !moves.is_empty() {
                self.stage_entries(moves, None);
            }
            return;
        }
        let either_owner = self.graveyard_return_allows_either_owner();
        let movements = choice.selected[player.index()]
            .iter()
            .filter(|r| {
                self.object_ref(r.card) == **r
                    && self.state.objects[r.card as usize].zone == Zone::Graveyard
                    && (self.state.objects[r.card as usize].owner == player || either_owner)
                    && self.state.objects[r.card as usize].is_card()
                    && self.state.objects[r.card as usize].card.is_creature()
            })
            .map(|r| {
                let mut movement = self.entry_move(r.card, Zone::Battlefield, false);
                movement.controller = player;
                movement.effect_counter = entry_counter;
                movement
            })
            .collect::<Vec<_>>();
        if !movements.is_empty() {
            self.stage_entries(movements, None);
        }
    }

    pub(super) fn graveyard_land_return_active(&self) -> bool {
        self.state.resolving.as_ref().is_some_and(|resolution| {
            matches!(
                resolution.program.get(resolution.pc.saturating_sub(1)),
                Some(Instruction::ChooseOwnGraveyardLandReturn { .. })
            )
        })
    }

    fn land_selection_zone(&self, zone: Zone) -> bool {
        let resolution = self.state.resolving.as_ref().unwrap();
        match resolution.program.get(resolution.pc.saturating_sub(1)) {
            Some(Instruction::ChooseOwnGraveyardLandReturn { .. }) => zone == Zone::Graveyard,
            Some(Instruction::PutHandLands { .. }) => zone == Zone::Hand,
            _ => matches!(zone, Zone::Hand | Zone::Graveyard),
        }
    }

    pub(super) fn start_graveyard_land_return(&mut self, item: &SpellView, tapped: bool) -> bool {
        if !self.state.objects.iter().any(|object| {
            object.zone == Zone::Graveyard
                && object.owner == item.controller
                && object.is_card()
                && object.card.is_land()
        }) {
            return false;
        }
        self.start_effect_choice(
            item,
            EffectChoiceKind::LandCards { maximum: 1, tapped },
            PlayerSet::Controller,
            evidence::cost(0).filter,
            1,
            None,
        )
    }

    pub(super) fn start_land_selection(
        &mut self,
        item: &SpellView,
        maximum: u32,
        tapped: bool,
    ) -> bool {
        if maximum == 0 {
            return false;
        }
        self.start_effect_choice_inner(
            item,
            EffectChoiceKind::LandCards { maximum, tapped },
            PlayerSet::Controller,
            evidence::cost(0).filter,
            0,
            Some(0),
            false,
            None,
        )
    }

    pub(super) fn start_discarded_land_choice(
        &mut self,
        player: Player,
        tapped: bool,
        objects: Vec<ObjectRef>,
    ) {
        if objects.is_empty() {
            return;
        }
        let source = self
            .state
            .resolving
            .as_ref()
            .unwrap()
            .item
            .source
            .expect("effect choice requires object source")
            .object;
        let pc = self.state.resolving.as_ref().unwrap().pc;
        self.state.resolving.as_mut().unwrap().choice = Some(Box::new(EffectChoice {
            discard_draw: None,
            discarded: Some(objects.into_boxed_slice()),
            kind: EffectChoiceKind::DiscardedLand { tapped },
            filter: evidence::cost(0).filter,
            amount: 1,
            source,
            required: [0; 2],
            selected: std::array::from_fn(|_| Vec::new()),
            skip_to: Some(pc),
            greatest_value: [None; 2],
            discard_mana_value_life: None,
            hand_defender: None,
            entry_life_bonus: None,
            hand_entry_filter: None,
            equipment_recipient: None,
            blight_copy_attacking: false,
        }));
        self.state.pending = Pending::EffectChoice { player };
    }
    pub(super) fn finish_land_selection(&mut self) {
        let choice = self
            .state
            .resolving
            .as_mut()
            .unwrap()
            .choice
            .take()
            .unwrap();
        if matches!(
            choice.kind.payment_kind(),
            EffectChoiceKind::UntapLands { .. }
        ) {
            let ids: Vec<_> = choice
                .selected
                .iter()
                .flatten()
                .filter(|object| {
                    self.on_battlefield(object.card) && self.object_ref(object.card) == **object
                })
                .map(|object| object.card)
                .collect();
            self.untap_objects(&ids, untap_rules::UntapCause::Effect);
            return;
        }
        let (tapped, discarded) = match choice.kind.payment_kind() {
            EffectChoiceKind::LandCards { tapped, .. } => (tapped, false),
            EffectChoiceKind::DiscardedLand { tapped } => (tapped, true),
            _ => unreachable!(),
        };
        let player = self.state.resolving.as_ref().unwrap().item.controller;
        let movements = choice.selected[player.index()]
            .iter()
            .filter(|reference| {
                !discarded || self.state.objects[reference.card as usize].zone != Zone::Battlefield
            })
            .map(|reference| {
                assert_eq!(self.object_ref(reference.card), *reference);
                let object = &self.state.objects[reference.card as usize];
                assert!(if discarded {
                    choice.discarded.as_ref().unwrap().contains(reference)
                        && self.current_kinds(reference.card) & 2 != 0
                } else {
                    object.owner == player
                        && self.land_selection_zone(object.zone)
                        && object.card.is_land()
                });
                let mut movement = self.entry_move(reference.card, Zone::Battlefield, tapped);
                movement.controller = player;
                movement
            })
            .collect::<Vec<_>>();
        if !movements.is_empty() {
            self.stage_entries(movements, None);
        }
    }

    pub(super) fn return_graveyard_tapped_attacking(&mut self, object: ObjectRef, player: Player) {
        let card = &self.state.objects[object.card as usize];
        if self.object_ref(object.card) != object || card.zone != Zone::Graveyard {
            return;
        }
        let mut movement = self.entry_move(object.card, Zone::Battlefield, true);
        movement.controller = player;
        movement.choose_attacking_defender = true;
        self.stage_entries(vec![movement], None);
    }

    pub(super) fn attacking_return_combat(&self, player: Player) -> bool {
        self.state.active == player
            && matches!(
                self.state.step,
                Step::BeginCombat
                    | Step::Attackers
                    | Step::Blockers
                    | Step::FirstStrikeDamage
                    | Step::CombatDamage
                    | Step::EndCombat
            )
    }
    pub(super) fn start_target_attacking_return(
        &mut self,
        item: &SpellView,
        counter: CounterKind,
    ) -> bool {
        let Some(bound) = item
            .targets
            .iter()
            .copied()
            .find(|&b| self.same_target_object(b) && matches!(b.target, Target::Card(_)))
        else {
            return false;
        };
        let Target::Card(id) = bound.target else {
            unreachable!()
        };
        let reference = self.object_ref(id);
        let player = item.controller;
        self.state.resolving.as_mut().unwrap().choice = Some(Box::new(EffectChoice {
            discard_draw: None,
            discarded: None,
            kind: EffectChoiceKind::TargetAttackingReturn { counter },
            filter: evidence::cost(0).filter,
            amount: 1,
            source: item
                .source
                .expect("effect choice requires object source")
                .object,
            required: [0; 2],
            selected: std::array::from_fn(|i| {
                if i == player.index() {
                    vec![reference]
                } else {
                    Vec::new()
                }
            }),
            skip_to: None,
            greatest_value: [None; 2],
            discard_mana_value_life: None,
            hand_defender: None,
            entry_life_bonus: None,
            hand_entry_filter: None,
            equipment_recipient: None,
            blight_copy_attacking: false,
        }));
        self.advance_target_attacking_return()
    }
    fn advance_target_attacking_return(&mut self) -> bool {
        let resolution = self.state.resolving.as_ref().unwrap();
        let choice = resolution.choice.as_ref().unwrap();
        let EffectChoiceKind::TargetAttackingReturn { counter } = choice.kind.payment_kind() else {
            unreachable!()
        };
        let player = resolution.item.controller;
        let reference = choice.selected[player.index()][0];
        if self.object_ref(reference.card) != reference
            || self.state.objects[reference.card as usize].zone != Zone::Graveyard
        {
            self.state.resolving.as_mut().unwrap().choice = None;
            return false;
        }
        let combat = self.attacking_return_combat(player);
        if combat
            && choice.hand_defender.is_none()
            && self.state.objects.iter().enumerate().any(|(id, o)| {
                o.zone == Zone::Battlefield
                    && o.controller == player.other()
                    && self.current_kinds(id as CardId) & 64 != 0
            })
        {
            self.state.pending = Pending::HandDefender { player };
            return true;
        }
        let choice = self
            .state
            .resolving
            .as_mut()
            .unwrap()
            .choice
            .take()
            .unwrap();
        let mut movement = self.entry_move(reference.card, Zone::Battlefield, true);
        movement.controller = player;
        movement.effect_counter = Some((counter, 1));
        if combat {
            movement.attacking = Some(
                choice
                    .hand_defender
                    .unwrap_or(CombatDefender::Player(player.other())),
            );
        }
        self.stage_entries(vec![movement], None);
        false
    }
    pub(super) fn choose_hand_defender(&mut self, defender: CombatDefender) {
        self.state
            .resolving
            .as_mut()
            .unwrap()
            .choice
            .as_mut()
            .unwrap()
            .hand_defender = Some(defender);
        if !self.advance_effect_choice() {
            self.run_program();
        }
    }
    pub(super) fn visit_hand_defenders(&self, emit: &mut impl FnMut(Action) -> bool) -> bool {
        let player = self.state.resolving.as_ref().unwrap().item.controller;
        if emit(Action::ChooseHandDefender {
            defender: CombatDefender::Player(player.other()),
        }) {
            return true;
        }
        for (id, o) in self.state.objects.iter().enumerate() {
            let battle = self.object_ref(id as CardId);
            if self.battle_can_be_attacked_by(battle, player)
                && emit(Action::ChooseHandDefender {
                    defender: CombatDefender::Battle(battle),
                })
            {
                return true;
            }
            if o.zone == Zone::Battlefield
                && o.controller == player.other()
                && self.current_kinds(id as CardId) & 64 != 0
                && emit(Action::ChooseHandDefender {
                    defender: CombatDefender::Planeswalker(self.object_ref(id as CardId)),
                })
            {
                return true;
            }
        }
        false
    }
    pub(super) fn start_discard_mana_value_life(&mut self, item: &SpellView) -> bool {
        self.start_effect_choice_inner(
            item,
            EffectChoiceKind::Discard,
            PlayerSet::Controller,
            evidence::cost(0).filter,
            1,
            None,
            false,
            Some(item.damage_event.expect("damaged player").player),
        )
    }
    #[allow(clippy::too_many_arguments)]
    fn start_effect_choice_inner(
        &mut self,
        item: &SpellView,
        kind: EffectChoiceKind,
        players: PlayerSet,
        filter: SacrificeFilter,
        amount: u32,
        optional_skip: Option<usize>,
        greatest: bool,
        discard_mana_value_life: Option<Player>,
    ) -> bool {
        if matches!(kind, EffectChoiceKind::SacrificeOrDiscard { .. }) {
            assert_eq!(amount, 1, "one alternative resolution cost");
            assert!(matches!(
                players,
                PlayerSet::Controller | PlayerSet::Opponent
            ));
        }
        if let EffectChoiceKind::RememberPermanents {
            require_battlefield_source,
        } = kind
        {
            self.state.resolving.as_mut().unwrap().remembered_objects = None;
            if require_battlefield_source
                && !item.source.is_some_and(|source| {
                    self.on_battlefield(source.id) && self.object_ref(source.id) == source.object
                })
            {
                self.state.resolving.as_mut().unwrap().pc += optional_skip.unwrap_or(0);
                return false;
            }
        }
        if matches!(
            kind,
            EffectChoiceKind::ExileCardFromAnyGraveyardRemember
                | EffectChoiceKind::ExileFaceUpDisguiseCreatures
        ) {
            self.state.resolving.as_mut().unwrap().remembered_objects = None;
        }
        if kind == EffectChoiceKind::ReturnRememberedCreature {
            assert_eq!(amount, 1, "one remembered creature return");
            assert_eq!(players, PlayerSet::Controller, "captured return controller");
            assert!(optional_skip.is_none(), "remembered return is mandatory");
        }
        if let EffectChoiceKind::DamageAmongTargetGroup {
            group,
            amount: damage,
        } = kind
        {
            assert_eq!(amount, 1, "choose a single damage recipient");
            self.state.resolving.as_mut().unwrap().remembered_objects = None;
            let mut targets = item.targets.iter().filter_map(|bound| {
                bound.object.filter(|object| {
                    bound.group == group
                        && bound.target == Target::Card(object.card)
                        && self.on_battlefield(object.card)
                        && self.object_ref(object.card) == *object
                })
            });
            let Some(first) = targets.next() else {
                return false;
            };
            if !targets.any(|object| object != first) {
                self.damage_chosen_target(first, damage);
                return false;
            }
        }
        let any_number = kind.any_number_selection();
        if any_number {
            assert_eq!(
                players,
                PlayerSet::Controller,
                "any-number resolution choice belongs to the controller"
            );
            assert_eq!(amount, 0, "any-number resolution choice has no minimum");
            assert!(
                optional_skip.is_none(),
                "any-number resolution choice finishes with any subset"
            );
        }
        if matches!(
            kind,
            EffectChoiceKind::ReturnToHandCounting
                | EffectChoiceKind::ReturnToHandCountingOtherSubtype(_)
                | EffectChoiceKind::TapAnyUntappedPermanents
        ) {
            self.state.resolving.as_mut().unwrap().moved_count = Some(0);
        }
        let resolution = self.state.resolving.as_ref().unwrap();
        let pc = resolution.pc;
        if matches!(kind, EffectChoiceKind::ExileSourceAndNamed { .. })
            && item.source.is_none_or(|source| {
                self.object_ref(source.id) != source.object
                    || self.state.objects[source.id as usize].zone != Zone::Stack
            })
        {
            self.state.resolving.as_mut().unwrap().pc =
                pc + optional_skip.expect("optional source cost");
            return false;
        }
        let source = if matches!(
            resolution.program.get(pc.saturating_sub(1)),
            Some(Instruction::MaySacrificeForGrantedReflexive { .. })
        ) {
            let Some(grant) = item
                .attachment_trigger
                .as_ref()
                .filter(|event| event.granted)
            else {
                return false;
            };
            grant.grantor
        } else {
            item.source
                .expect("effect choice requires object source")
                .object
        };
        let mut choice = EffectChoice {
            discard_draw: None,
            discarded: None,
            kind,
            filter,
            amount,
            discard_mana_value_life,
            hand_defender: None,
            entry_life_bonus: None,
            hand_entry_filter: None,
            equipment_recipient: None,
            blight_copy_attacking: false,
            source,
            required: [0; 2],
            selected: std::array::from_fn(|_| Vec::new()),
            greatest_value: [None; 2],
            skip_to: optional_skip.map(|skip| pc.checked_add(skip).expect("program offset")),
        };
        if let Some(skip_to) = choice.skip_to {
            assert!(skip_to <= self.state.resolving.as_ref().unwrap().program.len());
        }
        if let EffectChoiceKind::KeepCreaturesByPowerForEachPlayer(maximum) = kind {
            let mut eligible = [false; 2];
            for (index, object) in self.state.objects.iter().enumerate() {
                let slot = &mut eligible[object.controller.index()];
                if !*slot && self.creature_survives_sweep(index as CardId, maximum) {
                    *slot = true;
                }
            }
            choice.required[item.controller.index()] = eligible.into_iter().map(u32::from).sum();
            self.state.resolving.as_mut().unwrap().choice = Some(Box::new(choice));
            return self.advance_effect_choice();
        }
        // Unfiltered affordability follows the public hand count. A typed
        // restriction makes eligibility private, so retain its phase and quota.
        let private_optional_discard = kind == EffectChoiceKind::Discard
            && optional_skip.is_some()
            && filter != evidence::cost(0).filter;
        for player in self.program_players(players, item) {
            if greatest
                || matches!(
                    kind,
                    EffectChoiceKind::ExileGreatestManaValue
                        | EffectChoiceKind::SacrificeGreatestManaValue
                )
            {
                choice.greatest_value[player.index()] = self
                    .state
                    .objects
                    .iter()
                    .enumerate()
                    .filter(|(id, o)| {
                        o.zone == Zone::Battlefield
                            && o.controller == player
                            && if kind == EffectChoiceKind::SacrificeGreatestManaValue {
                                // Greatest value describes the controlled set, not
                                // only the subset currently able to be sacrificed.
                                self.effect_choice_candidate_inner(
                                    &choice,
                                    player,
                                    self.object_ref(*id as CardId),
                                    false,
                                )
                            } else if kind == EffectChoiceKind::ExileGreatestManaValue {
                                self.current_kinds(*id as CardId) & 65 != 0
                            } else {
                                self.is_creature(*id as CardId)
                            }
                    })
                    .map(|(id, _)| {
                        if matches!(
                            kind,
                            EffectChoiceKind::ExileGreatestManaValue
                                | EffectChoiceKind::SacrificeGreatestManaValue
                        ) {
                            i128::from(self.object_mana_value(id as CardId))
                        } else {
                            self.object_view(id as CardId).power
                        }
                    })
                    .max();
            }
            let available = self
                .state
                .objects
                .iter()
                .enumerate()
                .filter(|(id, _)| {
                    self.effect_choice_candidate(&choice, player, self.object_ref(*id as CardId))
                })
                .count();
            if let EffectChoiceKind::GraveyardCreatures {
                maximum,
                total_mana_value,
            } = kind
                && optional_skip.is_none()
            {
                assert_eq!(
                    players,
                    PlayerSet::Controller,
                    "one exact graveyard return chooser"
                );
                assert_eq!(
                    total_mana_value,
                    u64::MAX,
                    "exact count has no combined mana budget"
                );
                let count = u32::try_from(available.min(maximum as usize))
                    .capacity("choice count exhausted");
                if count == 0 {
                    return false;
                }
                choice.amount = count;
                choice.required[player.index()] = count;
                choice.kind = EffectChoiceKind::GraveyardCreatures {
                    maximum: count,
                    total_mana_value,
                };
                continue;
            }
            if available == 0 && (any_number || kind == EffectChoiceKind::PowerDifferenceBoost) {
                return false;
            }
            if any_number {
                choice.amount = if kind == EffectChoiceKind::DestroyNonbasicLandsForEachPlayer {
                    self.state
                        .objects
                        .iter()
                        .enumerate()
                        .filter(|(id, _)| self.nonbasic_land_choice(self.object_ref(*id as CardId)))
                        .fold(0u8, |seats, (_, land)| {
                            seats | (1 << land.controller.index())
                        })
                        .count_ones()
                } else {
                    u32::try_from(available).capacity("choice count exhausted")
                };
                continue;
            }
            // Optional private choices always present the same actor/phase,
            // even with no eligible card. The player can decline without revealing
            // whether their hidden hand contains a matching card.
            let private_choice = private_optional_discard
                || matches!(
                    kind,
                    EffectChoiceKind::SacrificeOrDiscard { .. }
                        | EffectChoiceKind::DiscardOneMatchingOr { .. }
                        | EffectChoiceKind::BottomHand
                        | EffectChoiceKind::HandExileCopy
                        | EffectChoiceKind::HandPlot { .. }
                        | EffectChoiceKind::HandCreature
                        | EffectChoiceKind::HandCreatureAttacking { .. }
                        | EffectChoiceKind::HandCreatureHasteSacrifice { .. }
                        | EffectChoiceKind::HandCreatureAndLand
                        | EffectChoiceKind::HandPermanent { .. }
                        | EffectChoiceKind::HandLand { .. }
                        | EffectChoiceKind::DeployFromHandOrGraveyard { .. }
                );
            if let Some(skip_to) = choice.skip_to
                && !private_choice
                && available < amount as usize
            {
                self.state.resolving.as_mut().unwrap().pc = skip_to;
                return false;
            }
            choice.required[player.index()] = if private_choice {
                amount
            } else {
                amount.min(u32::try_from(available).capacity("choice count exhausted"))
            };
        }
        self.state.resolving.as_mut().unwrap().choice = Some(Box::new(choice));
        self.advance_effect_choice()
    }

    pub(super) fn finish_creature_and_land_selection(&mut self) {
        let resolution = self.state.resolving.as_mut().unwrap();
        let player = resolution.item.controller;
        let choice = resolution.choice.take().unwrap();
        let moves = choice.selected[player.index()]
            .iter()
            .map(|r| {
                assert_eq!(self.object_ref(r.card), *r);
                self.entry_move(r.card, Zone::Battlefield, false)
            })
            .collect();
        self.stage_entries(moves, None);
    }

    pub(super) fn advance_effect_choice(&mut self) -> bool {
        if self
            .state
            .resolving
            .as_ref()
            .unwrap()
            .choice
            .as_ref()
            .is_some_and(|c| c.kind == EffectChoiceKind::KeepSharedCreatureTypes)
        {
            return self.advance_keep_shared_types();
        }
        if self
            .state
            .resolving
            .as_ref()
            .unwrap()
            .choice
            .as_ref()
            .is_some_and(|c| matches!(c.kind, EffectChoiceKind::TargetAttackingReturn { .. }))
        {
            return self.advance_target_attacking_return();
        }

        let choice = self
            .state
            .resolving
            .as_ref()
            .unwrap()
            .choice
            .as_ref()
            .unwrap();
        if let Some(draw) = &choice.discard_draw {
            for player in [self.state.active, self.state.active.other()] {
                if draw.players[player.index()]
                    && !draw.finished[player.index()]
                    && draw
                        .limit
                        .is_none_or(|limit| choice.selected[player.index()].len() < limit)
                {
                    self.state.pending = Pending::EffectChoice { player };
                    return true;
                }
            }
        }
        if let EffectChoiceKind::NamedSearch { maximum, .. } = choice.kind.payment_kind() {
            let player = self.state.resolving.as_ref().unwrap().item.controller;
            if choice.selected[player.index()].len() >= usize::from(maximum) {
                self.complete_name_search();
                return false;
            }
            self.state.pending = Pending::EffectChoice { player };
            return true;
        }
        if let EffectChoiceKind::HandCreatures { maximum } = choice.kind {
            let player = self.state.resolving.as_ref().unwrap().item.controller;
            if choice.selected[player.index()].len() >= usize::from(maximum) {
                self.finish_hand_creatures();
                return false;
            }
            self.state.pending = Pending::EffectChoice { player };
            return true;
        }
        if choice.kind.any_number_selection()
            || matches!(
                choice.kind.payment_kind(),
                EffectChoiceKind::CoverUpSearch { .. }
            )
        {
            self.state.pending = Pending::EffectChoice {
                player: self.state.resolving.as_ref().unwrap().item.controller,
            };
            return true;
        }
        if let EffectChoiceKind::GraveyardCreatures { maximum, .. } = choice.kind.payment_kind() {
            let player = self.state.resolving.as_ref().unwrap().item.controller;
            if choice.selected[player.index()].len() >= maximum as usize {
                self.finish_graveyard_return_selection();
                return false;
            }
            self.state.pending = Pending::EffectChoice { player };
            return true;
        }
        if let EffectChoiceKind::UntapLands { maximum } = choice.kind.payment_kind() {
            let player = self.state.resolving.as_ref().unwrap().item.controller;
            if choice.selected[player.index()].len() >= usize::from(maximum) {
                self.finish_land_selection();
                return false;
            }
            self.state.pending = Pending::EffectChoice { player };
            return true;
        }
        if matches!(
            choice.kind.payment_kind(),
            EffectChoiceKind::DiscardedLand { .. }
        ) {
            let player = self.state.resolving.as_ref().unwrap().item.controller;
            if !choice.selected[player.index()].is_empty() {
                self.finish_land_selection();
                return false;
            }
            self.state.pending = Pending::EffectChoice { player };
            return true;
        }
        if let EffectChoiceKind::LandCards { maximum, .. } = choice.kind.payment_kind() {
            let player = self.state.resolving.as_ref().unwrap().item.controller;
            if choice.selected[player.index()].len() >= maximum as usize {
                self.finish_land_selection();
                return false;
            }
            // The same optional phase is presented even when the hidden hand
            // contains no eligible land; no availability proof is required.
            self.state.pending = Pending::EffectChoice { player };
            return true;
        }
        for player in [self.state.active, self.state.active.other()] {
            if choice.selected[player.index()].len() < choice.required[player.index()] as usize {
                self.state.pending = Pending::EffectChoice { player };
                return true;
            }
        }
        if choice.kind == EffectChoiceKind::TurnControlledCreatureFaceUp {
            let player = self.state.resolving.as_ref().unwrap().item.controller;
            let selected = choice.selected[player.index()].first().copied();
            self.state.resolving.as_mut().unwrap().choice = None;
            if let Some(object) = selected {
                self.turn_manifest_face_up(object, player);
            }
            return false;
        }
        if choice.kind == EffectChoiceKind::CloakFromHand {
            let player = self.state.resolving.as_ref().unwrap().item.controller;
            let selected = choice.selected[player.index()].first().copied();
            self.state.resolving.as_mut().unwrap().choice = None;
            if selected.is_some() {
                self.obscure_private_hand_selection(player);
            }
            self.stage_hidden_cloaks(player, selected);
            return false;
        }
        if let EffectChoiceKind::DeployFromHandOrGraveyard { counter, .. } = choice.kind {
            let player = self.state.resolving.as_ref().unwrap().item.controller;
            let object = choice.selected[player.index()][0];
            self.state.resolving.as_mut().unwrap().choice = None;
            let mut entry = self.entry_move(object.card, Zone::Battlefield, false);
            assert_eq!(
                entry.before.reference, object,
                "selected exact deployment card"
            );
            entry.controller = player;
            entry.effect_counter = Some((counter, 1));
            self.stage_entries(vec![entry], None);
            return false;
        }
        if let EffectChoiceKind::SaddleRider(effect) = choice.kind {
            self.finish_saddle_rider_choice(effect);
            return false;
        }
        if choice.kind == EffectChoiceKind::HandCreatureAndLand {
            self.finish_creature_and_land_selection();
            return false;
        }
        if choice.kind.payment_kind() == EffectChoiceKind::CoverUpGraveyardCard {
            let player = self.state.resolving.as_ref().unwrap().item.controller;
            let selected = choice.selected[player.index()].first().copied();
            if let Some(reference) = selected {
                let owner = self.state.objects[reference.card as usize].owner;
                let name = self.object_name(reference.card);
                let events = self.move_objects_raw(&[reference.card], Zone::Exile);
                self.finish_zone_events(&events, &[]);
                let choice = self
                    .state
                    .resolving
                    .as_mut()
                    .unwrap()
                    .choice
                    .as_mut()
                    .unwrap();
                choice.kind = EffectChoiceKind::CoverUpSearch { owner, name };
                choice.selected[player.index()].clear();
                self.state.pending = Pending::EffectChoice { player };
                return true;
            }
            self.state.resolving.as_mut().unwrap().choice = None;
            return false;
        }
        if matches!(
            choice.kind.payment_kind(),
            EffectChoiceKind::HandToTop | EffectChoiceKind::BottomHand
        ) {
            let choice = self
                .state
                .resolving
                .as_mut()
                .unwrap()
                .choice
                .take()
                .unwrap();
            let player = self.state.resolving.as_ref().unwrap().item.controller;
            let ids: Vec<_> = choice.selected[player.index()]
                .iter()
                .map(|r| r.card)
                .collect();
            if !ids.is_empty() {
                self.obscure_private_hand_selection(player);
            }
            let bottom = choice.kind == EffectChoiceKind::BottomHand;
            let prior_top = bottom.then(|| self.public_library_top(player)).flatten();
            self.forget_library_top(player);
            let events = self.move_objects_raw(&ids, Zone::Library);
            if bottom {
                let library = &mut self.state.players[player.index()].library;
                for event in events
                    .iter()
                    .rev()
                    .filter(|e| e.moved && e.destination == Zone::Library)
                {
                    let position = library
                        .iter()
                        .position(|&id| id == event.after.card)
                        .unwrap();
                    let id = library.remove(position);
                    library.insert(0, id);
                }
                if let Some(prior) = prior_top
                    && self.top_library_card(player) == Some(prior.card)
                    && self.object_ref(prior.card) == prior
                {
                    self.remember_library_top(player, prior);
                }
            }
            self.finish_zone_events(&events, &[]);
            return false;
        }
        if matches!(
            choice.kind.payment_kind(),
            EffectChoiceKind::HandPlot { .. }
        ) {
            let player = self.state.resolving.as_ref().unwrap().item.controller;
            let reference = choice.selected[player.index()][0];
            self.state.resolving.as_mut().unwrap().choice = None;
            let event = self.move_object(reference.card, Zone::Exile);
            self.commit_plotted_move(&event);
            return false;
        }
        if matches!(
            choice.kind.payment_kind(),
            EffectChoiceKind::HandExileCopy
                | EffectChoiceKind::HandCreature
                | EffectChoiceKind::HandCreatureAttacking { .. }
                | EffectChoiceKind::HandCreatureHasteSacrifice { .. }
                | EffectChoiceKind::HandPermanent { .. }
                | EffectChoiceKind::HandLand { .. }
        ) {
            let player = self.state.resolving.as_ref().unwrap().item.controller;
            let Some(reference) = choice.selected[player.index()].first().copied() else {
                if let Some(frame) = self
                    .state
                    .resolving
                    .as_mut()
                    .unwrap()
                    .search_result
                    .as_mut()
                    && frame.capture_next_entry
                {
                    frame.capture_next_entry = false;
                    frame.closed = true;
                }
                self.state.resolving.as_mut().unwrap().choice = None;
                return false;
            };
            if choice.kind.payment_kind() == EffectChoiceKind::HandExileCopy {
                let definition = self.state.objects[reference.card as usize].card;
                self.state.resolving.as_mut().unwrap().choice = None;
                let events = self.move_objects_raw(&[reference.card], Zone::Exile);
                let copied = events
                    .iter()
                    .any(|e| e.moved && e.destination == Zone::Exile);
                self.finish_zone_events(&events, &[]);
                if copied {
                    let mut recipe = CopyRecipe::new(definition);
                    recipe.exceptions = CopyExceptions::GOLEM;
                    self.stage_copy_creation(player, recipe, 1);
                }
                return false;
            }
            let enchantment = choice.kind.payment_kind() == EffectChoiceKind::HandCreature
                && self.state.objects[reference.card as usize].card.kind_mask() & 32 != 0;
            let forced_tapped = matches!(
                choice.kind.payment_kind(),
                EffectChoiceKind::HandLand { tapped: true }
                    | EffectChoiceKind::HandPermanent { tapped: true, .. }
            );
            let tapped_and_attacking = enchantment
                || matches!(choice.kind, EffectChoiceKind::HandCreatureAttacking { .. });
            if tapped_and_attacking
                && self.attacking_return_combat(player)
                && choice.hand_defender.is_none()
            {
                let mut defenders = 0;
                self.visit_hand_defenders(&mut |_| {
                    defenders += 1;
                    defenders > 1
                });
                if defenders > 1 {
                    self.state.pending = Pending::HandDefender { player };
                    return true;
                }
                self.state
                    .resolving
                    .as_mut()
                    .unwrap()
                    .choice
                    .as_mut()
                    .unwrap()
                    .hand_defender = Some(CombatDefender::Player(player.other()));
            }
            let choice = self
                .state
                .resolving
                .as_mut()
                .unwrap()
                .choice
                .take()
                .unwrap();
            let mut movement = self.entry_move(
                reference.card,
                Zone::Battlefield,
                forced_tapped || tapped_and_attacking,
            );
            movement.controller = player;
            movement.attach_equipment_to = choice
                .equipment_recipient
                .map(entries::EquipmentAttachment::AfterEntry);
            movement.attacking = choice.hand_defender;
            movement.life_bonus = choice
                .entry_life_bonus
                .map(|(subtype, life)| (player, subtype, life));
            if matches!(
                choice.kind.payment_kind(),
                EffectChoiceKind::HandCreatureHasteSacrifice { .. }
            ) {
                let item = self.state.resolving.as_ref().unwrap().item.clone();
                movement.keywords_after = Some((choice.source, Keywords::HASTE));
                movement.keywords_after_permanent = true;
                self.stage_entries_with_delayed_sacrifice(vec![movement], &item);
            } else {
                self.stage_entries(vec![movement], None);
            }
            return false;
        }
        let choice = self
            .state
            .resolving
            .as_mut()
            .unwrap()
            .choice
            .take()
            .unwrap();
        if matches!(
            choice.kind,
            EffectChoiceKind::KeepCreaturesByPowerForEachPlayer(_)
        ) {
            let player = self.state.resolving.as_ref().unwrap().item.controller;
            let survivors = &choice.selected[player.index()];
            let destroyed = self
                .state
                .objects
                .iter()
                .enumerate()
                .filter_map(|(index, object)| {
                    let card = index as CardId;
                    let reference = self.object_ref(card);
                    (object.zone == Zone::Battlefield
                        && self.is_creature(card)
                        && !survivors.contains(&reference))
                    .then_some(reference)
                })
                .collect::<Vec<_>>();
            self.destroy_objects_outcome(destroyed);
            return false;
        }
        if matches!(choice.kind, EffectChoiceKind::RememberPermanents { .. }) {
            let mut selected: Vec<_> = choice.selected.into_iter().flatten().collect();
            selected.sort_unstable();
            self.state.resolving.as_mut().unwrap().remembered_objects =
                (!selected.is_empty()).then(|| selected.into());
            return false;
        }
        if let EffectChoiceKind::ReturnAttachmentRecipient { attachment } = choice.kind {
            let player = self.state.resolving.as_ref().unwrap().item.controller;
            self.finish_returned_attachment(
                attachment,
                choice.selected[player.index()].first().copied(),
                player,
            );
            return false;
        }
        if choice.kind == EffectChoiceKind::EquipmentRecipient {
            let player = self.state.resolving.as_ref().unwrap().item.controller;
            if let Some(&recipient) = choice.selected[player.index()].first() {
                self.attach_equipment(choice.source, recipient);
            }
            return false;
        }
        if matches!(
            choice.kind,
            EffectChoiceKind::ReflexiveEquipmentRecipient { .. }
        ) {
            let item = self.state.resolving.as_ref().unwrap().item.clone();
            if let Some(&recipient) = choice.selected[item.controller.index()].first() {
                return self.choose_reflexive_equipment(&item, recipient);
            }
            return false;
        }
        if matches!(
            choice.kind,
            EffectChoiceKind::AttachedEquipmentToDestroy { .. }
        ) {
            let player = self.state.resolving.as_ref().unwrap().item.controller;
            if let Some(&equipment) = choice.selected[player.index()].first() {
                self.destroy_objects_outcome([equipment]);
            }
            return false;
        }
        if choice.kind == EffectChoiceKind::ReturnRememberedCreature {
            let resolution = self.state.resolving.as_mut().unwrap();
            let player = resolution.item.controller;
            resolution.remembered_objects = None;
            if let Some(&object) = choice.selected[player.index()].first() {
                let mut movement = self.entry_move(object.card, Zone::Battlefield, false);
                movement.controller = player;
                self.stage_entries(vec![movement], None);
            }
            return false;
        }
        if choice.kind == EffectChoiceKind::ReturnCraftedCreatureAttacking {
            let player = self.state.resolving.as_ref().unwrap().item.controller;
            if let Some(&object) = choice.selected[player.index()].first()
                && self.crafted_exiled_creature(choice.source, object)
            {
                let mut movement = self.entry_move(object.card, Zone::Battlefield, true);
                movement.controller = player;
                movement.choose_attacking_defender = true;
                self.stage_entries(vec![movement], None);
            }
            return false;
        }
        if choice.kind == EffectChoiceKind::ExileControlledLinked {
            let item = self.state.resolving.as_ref().unwrap().item.clone();
            let ids = choice
                .selected
                .into_iter()
                .flatten()
                .filter(|&object| {
                    self.on_battlefield(object.card) && self.object_ref(object.card) == object
                })
                .map(|object| object.card)
                .collect::<Vec<_>>();
            self.exile_objects_linked(&item, &ids);
            return false;
        }
        if let EffectChoiceKind::LinkedCreature {
            other_power,
            other_toughness,
        } = choice.kind
        {
            let controller = self.state.resolving.as_ref().unwrap().item.controller;
            let selected = choice.selected[controller.index()].first().copied();
            self.finish_chosen_creature(
                selected,
                Stats {
                    power: i128::from(other_power),
                    toughness: i128::from(other_toughness),
                },
            );
            return false;
        }
        let resolution = self.state.resolving.as_ref().unwrap();
        let instruction = resolution.program.get(resolution.pc.saturating_sub(1));
        if let Some(Instruction::MaySacrificeForGrantedReflexive { ability, .. }) =
            instruction.copied()
        {
            let controller = resolution.item.controller;
            if let Some(&object) = choice.selected[controller.index()].first()
                && self.object_ref(object.card) == object
                && self
                    .sacrifice_cost_batch_outcome(&[(controller, object.card)])
                    .is_some_and(|receipt| receipt.count != 0)
            {
                // The bearer might have been the sacrifice. Use the packet with
                // its updated departure facts to create the independent trigger.
                let item = self.state.resolving.as_ref().unwrap().item.clone();
                self.queue_reflexive_objects(&item, ability, item.reflexive_action_objects());
            }
            return false;
        }
        if matches!(
            instruction,
            Some(Instruction::MaySacrificeForManaValue { .. })
        ) {
            let player = resolution.item.controller;
            let receipt = choice.selected[player.index()]
                .first()
                .copied()
                .filter(|object| self.object_ref(object.card) == *object)
                .and_then(|object| {
                    let mana_value = self.object_mana_value(object.card);
                    // CR 118.11–12: a replaced sacrifice still pays this cost.
                    self.sacrifice_cost_batch_outcome(&[(player, object.card)])
                        .map(|_| mana_value)
                });
            let resolution = self.state.resolving.as_mut().unwrap();
            resolution.sacrificed_mana_value = receipt;
            if receipt.is_none() {
                resolution.pc = choice.skip_to.unwrap();
            }
            return false;
        }
        // A ward cost is paid as one event: revalidate the complete selection
        // before making any sacrifice, including live restrictions.
        let ward = self
            .state
            .resolving
            .as_ref()
            .unwrap()
            .program
            .get(self.state.resolving.as_ref().unwrap().pc.saturating_sub(1))
            .is_some_and(|i| {
                matches!(
                    i,
                    Instruction::CounterRelatedUnlessSacrifice(_)
                        | Instruction::CounterRelatedUnlessSacrificeSubtype(_)
                )
            });
        if ward {
            let valid = choice.selected.iter().enumerate().all(|(seat, rows)| {
                let player = if seat == 0 { Player::P0 } else { Player::P1 };
                let mut empty = choice.clone();
                empty.selected = std::array::from_fn(|_| Vec::new());
                rows.iter()
                    .all(|&r| self.effect_choice_candidate(&empty, player, r))
            });
            let sacrifices: Vec<_> = [self.state.active, self.state.active.other()]
                .into_iter()
                .flat_map(|p| choice.selected[p.index()].iter().map(move |r| (p, r.card)))
                .collect();
            if !valid || !self.sacrifice_cost_batch_atomic(&sacrifices) {
                let related = self.state.resolving.as_ref().unwrap().item.related_stack;
                if let Some(related) = related {
                    self.counter_stack_item(related.id, false);
                }
            }
            return false;
        }
        if matches!(
            choice.kind.payment_kind(),
            EffectChoiceKind::ReturnToHand
                | EffectChoiceKind::ReturnToHandCounting
                | EffectChoiceKind::ReturnToHandCountingOtherSubtype(_)
                | EffectChoiceKind::ReturnGraveyardToHand { .. }
        ) {
            self.return_effect_choice_to_hand(*choice);
            return false;
        }
        if matches!(
            choice.kind,
            EffectChoiceKind::MoveFaceUpExileToGraveyard { .. }
        ) {
            let player = self.state.resolving.as_ref().unwrap().item.controller;
            if let Some(object) = choice.selected[player.index()].first() {
                self.move_object(object.card, Zone::Graveyard);
            }
            return false;
        }
        if let EffectChoiceKind::MeldPartner(pair) = choice.kind {
            let player = self.state.resolving.as_ref().unwrap().item.controller;
            let partner = choice.selected[player.index()].first().copied();
            self.meld_chosen_pair(player, choice.source, partner, pair);
            return false;
        }
        if let EffectChoiceKind::CountersOnTarget { kind, amount, .. } = choice.kind {
            let changes = choice
                .selected
                .iter()
                .flatten()
                .map(|&object| (object, Counters::of(kind, u64::from(amount))))
                .collect();
            self.effect_counters(changes, replacements::ResumeMode::Program);
            return false;
        }
        if let EffectChoiceKind::DamageAmongTargetGroup { amount, .. } = choice.kind {
            if let Some(&target) = choice.selected.iter().flatten().next() {
                self.damage_chosen_target(target, amount);
            }
            return false;
        }
        if choice.kind == EffectChoiceKind::TransferSuspectToEventCreature {
            if let Some(&object) = choice.selected.iter().flatten().next() {
                self.transfer_suspect(choice.source, object);
            }
            return false;
        }
        if choice.kind.payment_kind() == EffectChoiceKind::PowerDifferenceBoost {
            let controller = self.state.resolving.as_ref().unwrap().item.controller;
            let selected = &choice.selected[controller.index()];
            let amount = match selected.as_slice() {
                [one, other] => self
                    .stats(one.card)
                    .power
                    .abs_diff(self.stats(other.card).power),
                [_] => 0,
                _ => unreachable!("the mandatory choice selects one or two creatures"),
            };
            self.state.resolving.as_mut().unwrap().power_difference =
                Some(Box::new(PowerDifferenceView {
                    recipients: Box::from(selected.as_slice()),
                    amount: i128::try_from(amount).capacity("power difference exhausted"),
                    source: choice.source,
                }));
            self.draw_many(controller, amount);
        } else if choice.kind.payment_kind() == EffectChoiceKind::CopyExiledTriggerCreature {
            let item = self.state.resolving.as_ref().unwrap().item.clone();
            if let Some(&card) = choice.selected[item.controller.index()].first()
                && self.object_ref(card.card) == card
                && self.state.objects[card.card as usize].zone == Zone::Exile
                && let Some(recipient) = item.targets.iter().find_map(|bound| {
                    if self.same_target_object(*bound) {
                        bound.object.filter(|r| {
                            self.on_battlefield(r.card)
                                && self.state.objects[r.card as usize].is_token()
                        })
                    } else {
                        None
                    }
                })
            {
                let mut recipe = CopyRecipe::new(self.state.objects[card.card as usize].card);
                recipe.overrides = Some(CopyOverrides {
                    keywords: Keywords::FLYING,
                    ..CopyOverrides::NONE
                });
                self.apply_temporary_copy(recipient, recipe, CopyDeadline::EndOfTurn);
            }
        } else if let EffectChoiceKind::ExileSourceAndNamed { amount, .. } = choice.kind {
            let item = self.state.resolving.as_ref().unwrap().item.clone();
            let selected = &choice.selected[item.controller.index()];
            // Validate every component before committing any movement. An
            // interrupted/replayed choice must not partially pay a stale cost.
            let source_valid = item.source.is_some_and(|source| {
                self.object_ref(source.id) == source.object
                    && self.state.objects[source.id as usize].zone == Zone::Stack
            });
            let eligible = EffectChoice {
                selected: std::array::from_fn(|_| Vec::new()),
                ..(*choice).clone()
            };
            let unique = selected
                .iter()
                .enumerate()
                .all(|(i, r)| !selected[..i].contains(r));
            if !source_valid
                || selected.len() != usize::from(amount)
                || !unique
                || !selected
                    .iter()
                    .all(|&r| self.effect_choice_candidate(&eligible, item.controller, r))
            {
                self.state.resolving.as_mut().unwrap().pc =
                    choice.skip_to.expect("optional resolution cost");
                return false;
            }
            let failure_pc = choice.skip_to.expect("optional resolution cost");
            let checkpoint = self.atomic_zone_cost_checkpoint(Some(failure_pc));
            let mut moves = Vec::with_capacity(selected.len() + 1);
            moves.push((
                item.source.expect("valid resolution cost source").object,
                Zone::Exile,
            ));
            moves.extend(selected.iter().map(|&reference| (reference, Zone::Exile)));
            let Some(plans) = self.plan_atomic_zone_cost(&moves) else {
                self.state.resolving.as_mut().unwrap().pc = failure_pc;
                return false;
            };
            if plans
                .iter()
                .any(|(_, plan)| !plan.prevented && plan.destination == Zone::Battlefield)
            {
                self.stage_atomic_zone_cost(plans, checkpoint);
            } else if let Some(events) = self.commit_atomic_zone_cost(plans) {
                self.finish_zone_events(&events, &[]);
            } else {
                self.state.resolving.as_mut().unwrap().pc = failure_pc;
            }
        } else if let EffectChoiceKind::Blight(amount) | EffectChoiceKind::BlightCost(amount) =
            choice.kind
        {
            let choices: Vec<_> = [self.state.active, self.state.active.other()]
                .into_iter()
                .flat_map(|p| {
                    choice.selected[p.index()]
                        .iter()
                        .copied()
                        .map(move |object| (p, object))
                })
                .collect();
            if choice.blight_copy_attacking
                && let Some((player, object)) = choices.first().copied()
            {
                self.remember_blight_copy(object, player);
            }
            self.blight_counters(
                choices,
                amount,
                matches!(choice.kind, EffectChoiceKind::Blight(_)),
                replacements::ResumeMode::Program,
            );
        } else if let EffectChoiceKind::AmassGoblins(amount) = choice.kind.payment_kind() {
            let item = self.state.resolving.as_ref().unwrap().item.clone();
            // The owned selection seat is the amassing player. The resolving
            // ability retains its original controller, which can be another player.
            if let Some((player, recipient)) =
                [Player::P0, Player::P1].into_iter().find_map(|player| {
                    choice.selected[player.index()]
                        .first()
                        .copied()
                        .map(|recipient| (player, recipient))
                })
            {
                self.finish_amass_goblins(&item, player, recipient, amount);
            }
        } else if let EffectChoiceKind::EmpowerJace(amount) = choice.kind.payment_kind() {
            if let Some(recipient) = choice.selected[self
                .state
                .resolving
                .as_ref()
                .unwrap()
                .item
                .controller
                .index()]
            .first()
            .copied()
            {
                let player = self.state.resolving.as_ref().unwrap().item.controller;
                self.finish_empower_jace(player, recipient, amount);
            }
        } else if choice.kind.payment_kind() == EffectChoiceKind::ExileCreatureAndGraveyard {
            let item = &self.state.resolving.as_ref().unwrap().item;
            let player = item
                .targets
                .iter()
                .find_map(|bound| match bound.target {
                    Target::Player(player) => Some(player),
                    _ => None,
                })
                .expect("targeted player");
            let mut ids: Vec<_> = self
                .state
                .objects
                .iter()
                .enumerate()
                .filter(|(_, object)| {
                    object.zone == Zone::Graveyard && object.owner == player && object.is_card()
                })
                .map(|(id, _)| id as CardId)
                .collect();
            ids.extend(
                choice.selected[player.index()]
                    .iter()
                    .map(|reference| reference.card),
            );
            let events = self.move_objects_raw(&ids, Zone::Exile);
            self.finish_zone_events(&events, &[]);
        } else if matches!(
            choice.kind.payment_kind(),
            EffectChoiceKind::ExileGraveyard
                | EffectChoiceKind::ExileControlled
                | EffectChoiceKind::ExileFromHand
                | EffectChoiceKind::ExileCardFromAnyGraveyard
                | EffectChoiceKind::ExileGreatestManaValue
        ) || matches!(
            choice.kind.payment_kind(),
            EffectChoiceKind::TapPermanents(_)
        ) {
            if matches!(
                choice.kind.payment_kind(),
                EffectChoiceKind::ExileGraveyard
                    | EffectChoiceKind::ExileControlled
                    | EffectChoiceKind::ExileFromHand
                    | EffectChoiceKind::ExileCardFromAnyGraveyard
                    | EffectChoiceKind::ExileGreatestManaValue
            ) {
                let ids: Vec<_> = choice
                    .selected
                    .iter()
                    .flat_map(|rows| rows.iter().map(|r| r.card))
                    .collect();
                let events = self.move_objects_raw(&ids, Zone::Exile);
                if choice.kind == EffectChoiceKind::ExileCardFromAnyGraveyardRemember {
                    let mut actual: Vec<_> = events
                        .iter()
                        .filter(|event| event.moved && event.destination == Zone::Exile)
                        .map(|event| event.after)
                        .collect();
                    actual.sort_unstable();
                    self.state.resolving.as_mut().unwrap().remembered_objects =
                        (!actual.is_empty()).then(|| actual.into());
                }
                if let EffectChoiceKind::ExileFromHandWithPlaySurcharge(surcharge) = choice.kind {
                    for event in &events {
                        if event.moved && event.destination == Zone::Exile {
                            self.grant_taxed_exile_play(event.before.owner, event.after, surcharge);
                        }
                    }
                }
                self.finish_zone_events(&events, &[]);
            } else {
                let taps = [Player::P0, Player::P1].into_iter().flat_map(|player| {
                    choice.selected[player.index()]
                        .iter()
                        .map(move |object| (player, object.card))
                });
                self.tap_objects_by(taps);
            }
        } else if matches!(
            choice.kind.payment_kind(),
            EffectChoiceKind::SacrificeOrDiscard { .. }
        ) {
            // The resolving packet captures the payer/cause. A legal chosen cost
            // is paid even if its zone-change destination is replaced.
            let controller = self.state.resolving.as_ref().unwrap().item.controller;
            let (payer, reference) = [Player::P0, Player::P1]
                .into_iter()
                .find_map(|player| {
                    choice.selected[player.index()]
                        .first()
                        .map(|&object| (player, object))
                })
                .expect("one chosen payer");
            assert_eq!(self.object_ref(reference.card), reference);
            if self.state.objects[reference.card as usize].zone == Zone::Hand {
                self.discard_batch_caused(&[reference.card], None, Some(controller));
            } else {
                self.sacrifice_batch(&[(payer, reference.card)]);
            }
        } else if choice.kind.payment_kind() == EffectChoiceKind::Discard {
            let ids: Vec<_> = [self.state.active, self.state.active.other()]
                .into_iter()
                .flat_map(|p| choice.selected[p.index()].iter().map(|object| object.card))
                .collect();
            let mana_value: i128 = ids
                .iter()
                .map(|&id| i128::from(self.object_mana_value(id)))
                .sum();
            let cause = self.state.resolving.as_ref().unwrap().item.controller;
            let after = choice
                .discard_draw
                .map(|draw| discard_events::DiscardAfter::DrawActual {
                    players: draw.players,
                    bonus: draw.bonus,
                });
            self.discard_batch_with_after(&ids, None, Some(cause), after);
            if let Some(player) = choice.discard_mana_value_life {
                if let Some(context) = self
                    .state
                    .entry_batch
                    .as_mut()
                    .and_then(|b| b.discard_context.as_mut())
                {
                    context.after =
                        Some(discard_events::DiscardAfter::LoseLife(player, mana_value));
                } else {
                    self.lose_life(player, mana_value);
                }
            }
        } else {
            let sacrifices: Vec<_> = [self.state.active, self.state.active.other()]
                .into_iter()
                .flat_map(|p| {
                    choice.selected[p.index()]
                        .iter()
                        .map(move |object| (p, object.card))
                })
                .collect();
            let remembers_controller_sacrifice = self
                .state
                .resolving
                .as_ref()
                .unwrap()
                .program
                .iter()
                .any(|instruction| {
                    matches!(
                        instruction,
                        Instruction::CreateTokenIfControllerSacrificed(_)
                    )
                });
            let controller = self.state.resolving.as_ref().unwrap().item.controller;
            let controller_subject = choice.selected[controller.index()].first().copied();
            if remembers_controller_sacrifice {
                self.state.resolving.as_mut().unwrap().moved_count = Some(0);
            }
            if !sacrifices.is_empty() {
                if self
                    .state
                    .resolving
                    .as_ref()
                    .unwrap()
                    .program
                    .iter()
                    .any(|i| {
                        matches!(
                            i,
                            Instruction::PutCountersScaled {
                                amount: AmountValue::SacrificedPower,
                                ..
                            }
                        )
                    })
                {
                    // Capture before ANY selected permanent leaves (including lords),
                    // and before linked zone events can suspend this program.
                    let power = sacrifices
                        .iter()
                        .map(|&(_, id)| self.stats(id).power)
                        .try_fold(0i128, |sum, value| sum.checked_add(value))
                        .capacity("sacrificed power exhausted");
                    self.state.resolving.as_mut().unwrap().sacrificed_power = Some(Box::new(power));
                }
                if self
                    .state
                    .resolving
                    .as_ref()
                    .unwrap()
                    .program
                    .iter()
                    .any(|i| {
                        matches!(
                            i,
                            Instruction::GainLifeScaled(_, AmountValue::SacrificedToughness)
                        )
                    })
                {
                    let toughness = sacrifices
                        .iter()
                        .map(|&(_, id)| self.stats(id).toughness)
                        .try_fold(0i128, |sum, n| sum.checked_add(n))
                        .capacity("sacrificed toughness exhausted");
                    self.state
                        .resolving
                        .as_mut()
                        .unwrap()
                        .item
                        .sacrificed_toughness = Some(Box::new(toughness));
                }
                let reflexive_power = self
                    .state
                    .resolving
                    .as_ref()
                    .unwrap()
                    .program
                    .iter()
                    .any(|i| matches!(i, Instruction::ReflexiveSacrificedPower { .. }))
                    .then(|| {
                        assert_eq!(sacrifices.len(), 1, "one sacrifice for reflexive power");
                        let id = sacrifices[0].1;
                        (self.object_ref(id), self.stats(id).power)
                    });
                self.sacrifice_batch(&sacrifices);
                if remembers_controller_sacrifice {
                    let succeeded = controller_subject.is_some_and(|object| {
                        self.object_ref(object.card) != object || !self.on_battlefield(object.card)
                    });
                    self.state.resolving.as_mut().unwrap().moved_count = Some(u64::from(succeeded));
                }
                if let Some((object, power)) = reflexive_power
                    && (!self.on_battlefield(object.card) || self.object_ref(object.card) != object)
                {
                    self.state.resolving.as_mut().unwrap().item.sacrificed_power =
                        Some(Box::new(power));
                }
            }
        }
        false
    }

    fn return_effect_choice_to_hand(&mut self, mut choice: EffectChoice) {
        let selected = std::mem::take(&mut choice.selected);
        let ids: Vec<_> = [self.state.active, self.state.active.other()]
            .into_iter()
            .flat_map(|player| {
                selected[player.index()]
                    .iter()
                    .copied()
                    .map(move |object| (player, object))
            })
            .filter_map(|(player, object)| {
                self.effect_choice_candidate(&choice, player, object)
                    .then_some(object.card)
            })
            .collect();
        if !ids.is_empty() {
            let events = self.move_objects_raw(&ids, Zone::Hand);
            if matches!(
                choice.kind,
                EffectChoiceKind::ReturnToHandCounting
                    | EffectChoiceKind::ReturnToHandCountingOtherSubtype(_)
            ) {
                let count = events
                    .iter()
                    .filter(|event| {
                        event.moved
                            && event.destination == Zone::Hand
                            && match choice.kind {
                                EffectChoiceKind::ReturnToHandCountingOtherSubtype(subtype) => {
                                    event.before.reference != choice.source
                                        && event.before.has_subtype(subtype)
                                }
                                _ => true,
                            }
                    })
                    .count();
                self.state.resolving.as_mut().unwrap().moved_count = Some(count as u64);
            }
            self.finish_zone_events(&events, &[]);
        }
    }

    pub(super) fn finish_sacrifice_selection(&mut self) {
        let resolution = self.state.resolving.as_mut().unwrap();
        let player = resolution.item.controller;
        let choice = resolution.choice.take().unwrap();
        let spec = AdditionalCost {
            kind: EffectChoiceKind::Sacrifice,
            filter: choice.filter,
        };
        let sacrifices: Vec<_> = choice.selected[player.index()]
            .iter()
            .copied()
            .filter(|&object| self.additional_candidate(spec, player, choice.source, object))
            .map(|object| (player, object.card))
            .collect();
        if !sacrifices.is_empty() {
            let events = self
                .sacrifice_cost_batch_events(&sacrifices)
                .expect("validated sacrifice selections");
            let actual = events
                .iter()
                .filter(|event| event.sacrificed_by == Some(player));
            let resolution = self.state.resolving.as_mut().unwrap();
            resolution.moved_count = Some(actual.clone().count() as u64);
            if choice.kind == EffectChoiceKind::SacrificeSelectionForPower {
                let power = actual
                    .map(|event| event.before.power)
                    .try_fold(0i128, |sum, power| sum.checked_add(power))
                    .capacity("sacrificed power exhausted");
                resolution.sacrificed_power = Some(Box::new(power));
            }
        }
    }

    fn damage_chosen_target(&mut self, object: ObjectRef, amount: i32) {
        let resolution = self.state.resolving.as_mut().unwrap();
        resolution.remembered_objects = Some(vec![object].into());
        let source = self.program_damage_source(&self.state.resolving.as_ref().unwrap().item);
        if self.on_battlefield(object.card) && self.object_ref(object.card) == object {
            self.apply_damage_batch(&[DamageEvent {
                source,
                target: Target::Card(object.card),
                amount: i128::from(amount),
                combat: false,
                preventable: true,
            }]);
        }
    }

    pub(super) fn finish_tap_selection(&mut self) {
        let resolution = self.state.resolving.as_mut().unwrap();
        let player = resolution.item.controller;
        let mut choice = resolution.choice.take().unwrap();
        let selected = std::mem::take(&mut choice.selected[player.index()]);
        let taps = selected
            .into_iter()
            .filter(|&object| self.effect_choice_candidate(&choice, player, object))
            .map(|object| (player, object.card))
            .collect::<Vec<_>>();
        let count = self.tap_objects_by(taps) as u64;
        self.state.resolving.as_mut().unwrap().moved_count = Some(count);
    }

    pub(super) fn finish_reflexive_graveyard_exile(&mut self, ability: u16) {
        let resolution = self.state.resolving.as_mut().unwrap();
        let player = resolution.item.controller;
        let choice = resolution.choice.take().unwrap();
        let ids: Vec<_> = choice.selected[player.index()]
            .iter()
            .map(|object| object.card)
            .collect();
        if ids.is_empty() {
            return;
        }
        let events = self.move_objects_raw(&ids, Zone::Exile);
        self.finish_zone_events(&events, &[]);
        let item = self.state.resolving.as_ref().unwrap().item.clone();
        self.queue_reflexive_objects(
            &item,
            ability,
            events.iter().filter_map(|event| {
                (event.moved && event.destination == Zone::Exile).then_some(event.after)
            }),
        );
    }

    pub(super) fn finish_lore_counter_selection(&mut self) {
        let resolution = self.state.resolving.as_mut().unwrap();
        let player = resolution.item.controller;
        let mut choice = resolution.choice.take().unwrap();
        let EffectChoiceKind::RemoveLoreForSourceCounters(kind) = choice.kind else {
            unreachable!("lore counter selection");
        };
        let selected = std::mem::take(&mut choice.selected[player.index()]);
        let lore = Counters::of(CounterKind::Lore, 1);
        let mut removed = 0;
        for object in selected {
            if self.effect_choice_candidate(&choice, player, object) {
                self.pay_counters(object.card, &lore);
                removed += 1;
            }
        }
        if removed != 0 {
            self.effect_counters(
                vec![(choice.source, Counters::of(kind, removed))],
                replacements::ResumeMode::Program,
            );
        }
    }

    pub(super) fn choose_effect_object(&mut self, player: Player, object: ObjectRef) {
        if self
            .state
            .resolving
            .as_ref()
            .unwrap()
            .choice
            .as_ref()
            .is_some_and(|c| matches!(c.kind, EffectChoiceKind::ColorCounter { .. }))
        {
            self.choose_color_counter_recipient(object);
            self.run_program();
            return;
        }
        self.state
            .resolving
            .as_mut()
            .unwrap()
            .choice
            .as_mut()
            .unwrap()
            .selected[player.index()]
        .push(object);
        if !self.advance_effect_choice() {
            self.run_program();
        }
    }

    pub(super) fn decline_effect_payment(&mut self) {
        let player = self.semantic_actor().expect("effect choice actor");
        if let Some(draw) = self
            .state
            .resolving
            .as_mut()
            .unwrap()
            .choice
            .as_mut()
            .unwrap()
            .discard_draw
            .as_mut()
        {
            draw.finished[player.index()] = true;
            if !self.advance_effect_choice() {
                self.run_program();
            }
            return;
        }
        let choice = self
            .state
            .resolving
            .as_mut()
            .unwrap()
            .choice
            .take()
            .unwrap();
        if let EffectChoiceKind::DiscardOneMatchingOr { otherwise, .. } = choice.kind {
            self.state.resolving.as_mut().unwrap().discards[player.index()] =
                usize::from(otherwise);
            self.run_program();
            return;
        }
        if matches!(
            choice.kind,
            EffectChoiceKind::HandPermanent { .. }
                | EffectChoiceKind::HandCreature
                | EffectChoiceKind::HandCreatureAttacking { .. }
                | EffectChoiceKind::HandCreatureHasteSacrifice { .. }
                | EffectChoiceKind::HandLand { .. }
                | EffectChoiceKind::DeployFromHandOrGraveyard { .. }
        ) && let Some(frame) = self
            .state
            .resolving
            .as_mut()
            .unwrap()
            .search_result
            .as_mut()
            && frame.capture_next_entry
        {
            frame.capture_next_entry = false;
            frame.closed = true;
        }
        let resolution = self.state.resolving.as_ref().unwrap();
        if matches!(choice.kind, EffectChoiceKind::SacrificeOrDiscard { .. })
            && let Some(sequence) = &resolution.repeated_life_choice
        {
            self.lose_life(player, i128::from(sequence.life));
            self.run_program();
            return;
        }
        let ward_victim = resolution
            .program
            .get(resolution.pc.saturating_sub(1))
            .is_some_and(|instruction| {
                matches!(
                    instruction,
                    Instruction::CounterRelatedUnlessSacrifice(_)
                        | Instruction::CounterRelatedUnlessSacrificeSubtype(_)
                )
            })
            .then_some(resolution.item.related_stack)
            .flatten();
        self.state.resolving.as_mut().unwrap().pc = choice.skip_to.expect("optional payment");
        if let Some(victim) = ward_victim {
            self.counter_stack_item(victim.id, false);
        }
        self.run_program();
    }

    pub(super) fn exile_then_search_same_name(&mut self, item: &SpellView) {
        let [bound] = item.targets.as_slice() else {
            return;
        };
        let Target::Card(id) = bound.target else {
            return;
        };
        if !self.same_target_object(*bound) || !self.on_battlefield(id) {
            return;
        }
        let owner = self.state.objects[id as usize].controller;
        let name = self.object_name(id);
        self.state.resolving.as_mut().unwrap().exile_name_search = Some(Box::new((owner, name)));
        self.move_target_batch(&item.targets, Zone::Exile);
    }

    pub(super) fn resume_exile_name_search(&mut self) -> bool {
        let resolution = self.state.resolving.as_mut().unwrap();
        let Some(search) = resolution.exile_name_search.take() else {
            return false;
        };
        let (owner, name) = *search;
        let item = resolution.item.clone();
        self.start_effect_choice(
            &item,
            EffectChoiceKind::CoverUpSearch { owner, name },
            PlayerSet::Controller,
            evidence::cost(0).filter,
            0,
            None,
        )
    }

    pub(super) fn start_named_card_search(&mut self, item: &SpellView, maximum: u16) -> bool {
        let Some(owner) = self.program_players(PlayerSet::Targeted, item).next() else {
            return false;
        };
        let name = self
            .state
            .resolving
            .as_ref()
            .unwrap()
            .chosen_card_name
            .expect("preceding card-name choice");
        self.start_effect_choice(
            item,
            EffectChoiceKind::NamedSearch {
                owner,
                name,
                maximum,
            },
            PlayerSet::Controller,
            evidence::cost(0).filter,
            0,
            None,
        )
    }
    pub(super) fn finish_cover_up_search(&mut self) {
        self.complete_name_search();
        self.run_program();
    }
    fn complete_name_search(&mut self) {
        let choice = self
            .state
            .resolving
            .as_mut()
            .unwrap()
            .choice
            .take()
            .unwrap();
        let (owner, draws_for_hand) = match choice.kind.payment_kind() {
            EffectChoiceKind::CoverUpSearch { owner, .. } => (owner, true),
            EffectChoiceKind::NamedSearch { owner, .. } => (owner, false),
            _ => unreachable!(),
        };
        let player = self.state.resolving.as_ref().unwrap().item.controller;
        if owner == player {
            self.library_searched(player);
        }
        let ids: Vec<_> = choice.selected[player.index()]
            .iter()
            .map(|reference| reference.card)
            .collect();
        let events = self.move_objects_raw(&ids, Zone::Exile);
        let hand_count = events
            .iter()
            .filter(|event| {
                event.moved && event.destination == Zone::Exile && event.before.zone == Zone::Hand
            })
            .count();
        self.finish_zone_events(&events, &[]);
        self.forget_library_top(owner);
        self.shuffle_library_cards(owner);
        if draws_for_hand {
            self.draw_many(owner, hand_count as u128);
        }
    }
    pub(super) fn effect_choice_view(&self, viewer: Player) -> Option<EffectChoiceView> {
        let choice = self.state.resolving.as_ref()?.choice.as_ref()?;
        let actor = self.semantic_actor()?;
        if matches!(
            choice.kind.payment_kind(),
            EffectChoiceKind::Discard
                | EffectChoiceKind::ExileFromHand
                | EffectChoiceKind::SacrificeOrDiscard { .. }
                | EffectChoiceKind::HandToTop
                | EffectChoiceKind::BottomHand
                | EffectChoiceKind::HandExileCopy
                | EffectChoiceKind::HandPlot { .. }
                | EffectChoiceKind::HandCreature
                | EffectChoiceKind::HandCreatureAttacking { .. }
                | EffectChoiceKind::HandCreatureHasteSacrifice { .. }
                | EffectChoiceKind::HandCreatureAndLand
                | EffectChoiceKind::HandPermanent { .. }
                | EffectChoiceKind::HandLand { .. }
                | EffectChoiceKind::HandCreatures { .. }
                | EffectChoiceKind::CloakFromHand
        ) && !self.shares_information(viewer, actor)
        {
            return None;
        }
        if matches!(
            choice.kind.payment_kind(),
            EffectChoiceKind::CoverUpSearch { .. } | EffectChoiceKind::NamedSearch { .. }
        ) && !self.shares_information(viewer, actor)
        {
            return None;
        }
        let inspected_cards = match choice.kind.payment_kind() {
            EffectChoiceKind::CoverUpSearch { owner, .. }
            | EffectChoiceKind::NamedSearch { owner, .. }
                if self.shares_information(viewer, actor) =>
            {
                Some(
                    self.state
                        .objects
                        .iter()
                        .enumerate()
                        .filter(|(_, object)| {
                            object.owner == owner
                                && object.is_card()
                                && matches!(object.zone, Zone::Hand | Zone::Library)
                        })
                        .map(|(id, _)| self.object_view(id as CardId))
                        .collect(),
                )
            }
            _ => None,
        };
        let public_only = !self.shares_information(viewer, actor)
            && matches!(
                choice.kind.payment_kind(),
                EffectChoiceKind::LandCards { .. }
                    | EffectChoiceKind::DeployFromHandOrGraveyard { .. }
            );
        let candidates = self
            .state
            .objects
            .iter()
            .enumerate()
            .map(|(id, _)| self.object_ref(id as CardId))
            .filter(|&object| self.effect_choice_candidate(choice, actor, object))
            .filter(|object| {
                !public_only || self.state.objects[object.card as usize].zone == Zone::Graveyard
            })
            .collect();
        Some(EffectChoiceView {
            kind: choice.kind,
            amount: choice.amount,
            optional: choice.skip_to.is_some()
                || choice.kind.any_number_selection()
                || matches!(
                    choice.kind,
                    EffectChoiceKind::ReturnUpToGraveyardToHand { .. }
                        | EffectChoiceKind::HandCreatures { .. }
                        | EffectChoiceKind::SaddleRider(SaddleRiderEffect::BlinkSourceAndUpToOne)
                ),
            required: choice.required,
            selected: std::array::from_fn(|index| {
                choice.selected[index]
                    .iter()
                    .filter(|_| {
                        !matches!(
                            choice.kind.payment_kind(),
                            EffectChoiceKind::Discard | EffectChoiceKind::ExileFromHand
                        ) || self.shares_information(
                            viewer,
                            if index == 0 { Player::P0 } else { Player::P1 },
                        )
                    })
                    .copied()
                    .filter(|object| {
                        !public_only
                            || self.state.objects[object.card as usize].zone == Zone::Graveyard
                    })
                    .collect()
            }),
            candidates: if matches!(self.state.pending, Pending::HandDefender { .. }) {
                Vec::new()
            } else {
                candidates
            },
            defender_choice: matches!(self.state.pending, Pending::HandDefender { .. }),
            inspected_cards,
        })
    }
}

#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
impl Game {
    pub(super) fn keep_shared_type_player(
        &self,
        choice: &effect_choices::EffectChoice,
    ) -> Option<Player> {
        [self.state.active, self.state.active.other()]
            .into_iter()
            .find(|player| {
                choice.selected[player.index()].len() < choice.required[player.index()] as usize
            })
    }
    pub(super) fn keep_shared_type_candidate(
        &self,
        choice: &effect_choices::EffectChoice,
        object: ObjectRef,
    ) -> bool {
        self.keep_shared_type_player(choice).is_some_and(|player| {
            self.on_battlefield(object.card)
                && self.object_ref(object.card) == object
                && self.state.objects[object.card as usize].controller == player
                && self.is_creature(object.card)
        })
    }
    pub(super) fn start_keep_shared_types(&mut self, item: &SpellView) -> bool {
        let required = std::array::from_fn(|index| {
            u32::from(self.state.objects.iter().enumerate().any(|(id, object)| {
                object.zone == Zone::Battlefield
                    && object.controller.index() == index
                    && self.is_creature(id as CardId)
            }))
        });
        self.state.resolving.as_mut().unwrap().choice =
            Some(Box::new(effect_choices::EffectChoice {
                kind: EffectChoiceKind::KeepSharedCreatureTypes,
                filter: evidence::cost(0).filter,
                amount: 1,
                source: item.source.expect("choice source").object,
                required,
                selected: std::array::from_fn(|_| Vec::new()),
                skip_to: None,
                greatest_value: [None; 2],
                discard_mana_value_life: None,
                hand_defender: None,
                entry_life_bonus: None,
                hand_entry_filter: None,
                equipment_recipient: None,
                blight_copy_attacking: false,
                discard_draw: None,
                discarded: None,
            }));
        self.advance_keep_shared_types()
    }
    pub(super) fn advance_keep_shared_types(&mut self) -> bool {
        let resolution = self.state.resolving.as_ref().unwrap();
        let choice = resolution.choice.as_ref().unwrap();
        if self.keep_shared_type_player(choice).is_some() {
            self.state.pending = Pending::EffectChoice {
                player: resolution.item.controller,
            };
            return true;
        }
        let choice = self
            .state
            .resolving
            .as_mut()
            .unwrap()
            .choice
            .take()
            .unwrap();
        // Freeze characteristics before any sacrifice can remove a subtype provider.
        let anchors: [Option<ObjectView>; 2] = std::array::from_fn(|index| {
            choice.selected[index]
                .first()
                .map(|r| self.object_view(r.card))
        });
        let sacrifices: Vec<_> = self
            .state
            .objects
            .iter()
            .enumerate()
            .filter_map(|(id, object)| {
                let id = id as CardId;
                if object.zone != Zone::Battlefield || !self.is_creature(id) {
                    return None;
                }
                let anchor = anchors[object.controller.index()].as_ref()?;
                if anchor.reference == self.object_ref(id) {
                    return None;
                }
                let recipient = self.object_view(id);
                let shares = crate::cards::CREATURE_TYPES
                    .iter()
                    .any(|kind| anchor.has_subtype(kind) && recipient.has_subtype(kind));
                (!shares).then_some((object.controller, id))
            })
            .collect();
        self.sacrifice_batch(&sacrifices);
        false
    }
}
