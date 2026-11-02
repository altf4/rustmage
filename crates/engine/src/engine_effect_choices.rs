//! Simultaneous public sacrifices and private exact resolution-cost payments.
use super::*;
use ability_world::CharacteristicWorld;
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
        let o = &self.state.objects[object.card as usize];
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
        if choice.kind.payment_kind() == EffectChoiceKind::ExileCardFromAnyGraveyard {
            return o.zone == Zone::Graveyard
                && o.is_card()
                && self.object_ref(object.card) == object
                && !choice.selected[player.index()].contains(&object);
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
        if let EffectChoiceKind::ExileSourceAndNamed { name, .. } = choice.kind.payment_kind() {
            let o = &self.state.objects[object.card as usize];
            return o.zone == Zone::Graveyard
                && o.owner == player
                && self.object_ref(object.card) == object
                && !o.is_token()
                && !o.is_spell_copy()
                && !ability_world::CharacteristicWorld::graveyard_exile_prohibited(
                    self,
                    object.card,
                )
                && crate::card_names::same_name(o.card.name(), name)
                && !choice.selected[player.index()].contains(&object);
        }
        if choice.kind.payment_kind() == EffectChoiceKind::ExileGraveyard {
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
            EffectChoiceKind::TapPermanents(_)
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
                && o.owner == player
                && o.is_card()
                && o.card.is_creature()
                && self.object_ref(object.card) == object
                && !selected.contains(&object)
                && selected.len() < usize::from(maximum)
                && o.card.fixed_mana_value() <= total_mana_value.saturating_sub(spent);
        }
        if let EffectChoiceKind::LandCards { maximum, .. } = choice.kind.payment_kind() {
            return o.owner == player
                && o.is_card()
                && self.object_ref(object.card) == object
                && matches!(o.zone, Zone::Hand | Zone::Graveyard)
                && o.card.is_land()
                && !choice.selected[player.index()].contains(&object)
                && choice.selected[player.index()].len() < maximum as usize;
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
                | EffectChoiceKind::HandToTop
                | EffectChoiceKind::HandExileCopy
                | EffectChoiceKind::HandPlot { .. }
                | EffectChoiceKind::HandCreature
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
                | EffectChoiceKind::HandToTop
                | EffectChoiceKind::HandExileCopy
                | EffectChoiceKind::HandPlot { .. }
                | EffectChoiceKind::HandCreature
                | EffectChoiceKind::HandPermanent { .. }
                | EffectChoiceKind::HandLand { .. }
        ) {
            return o.zone == Zone::Hand
                && o.is_card()
                && match choice.kind.payment_kind() {
                    // Hand costs use the same typed characteristics and filters as
                    // announced discard costs, including artifact lands in hand.
                    EffectChoiceKind::Discard => self.additional_candidate(
                        AdditionalCost {
                            kind: choice.kind,
                            filter: choice.filter,
                        },
                        player,
                        choice.source,
                        object,
                    ),
                    EffectChoiceKind::HandCreature => o.card.is_creature(),
                    EffectChoiceKind::HandPermanent { maximum, .. } => {
                        o.card.kind_mask() & (1 | 2 | 16 | 32 | 64 | 256) != 0
                            && o.card.fixed_mana_value() <= u64::from(maximum)
                            && choice.hand_entry_filter.is_none_or(|filter| {
                                filter.matches(o.card)
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
        if !choice.kind.cost_matches_any(
            o.card,
            self.current_kinds(object.card),
            self.granted_subtypes(object.card),
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
            && self.keywords(object.card).contains(f.keywords)
            && (f.kinds == 0 || self.current_kinds(object.card) & u16::from(f.kinds) != 0)
            && (!f.source_only || object == choice.source)
            && (!f.nonland || self.current_kinds(object.card) & 2 == 0)
            && (!f.other || object != choice.source)
            && f.tokens.is_none_or(|token| o.is_token() == token)
            && f.subtype
                .is_none_or(|subtype| self.has_subtype(object.card, subtype))
    }

    pub(super) fn start_half_choice(&mut self, item: &SpellView, kind: EffectChoiceKind) -> bool {
        assert!(matches!(
            kind,
            EffectChoiceKind::Discard | EffectChoiceKind::Sacrifice
        ));
        let mut choice = EffectChoice {
            discard_draw: None,
            discarded: None,
            kind,
            filter: evidence::cost(0).filter,
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
        };
        // Freeze both quotas before either player chooses. The following printed
        // instruction computes new quotas after this entire event and its replacements.
        for player in [self.state.active, self.state.active.other()] {
            let available = self
                .state
                .objects
                .iter()
                .filter(|o| {
                    if kind == EffectChoiceKind::Discard {
                        o.zone == Zone::Hand && o.owner == player && o.is_card()
                    } else {
                        o.zone == Zone::Battlefield && o.controller == player
                    }
                })
                .count();
            choice.required[player.index()] =
                u32::try_from(available / 2).capacity("half choice count exhausted");
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
            amount,
            optional_skip,
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
                maximum: u32::MAX,
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
    pub(super) fn start_hand_permanent(
        &mut self,
        item: &SpellView,
        maximum: AmountValue,
        tapped: bool,
    ) -> bool {
        let maximum = u32::try_from(self.item_amount(maximum, item)).unwrap_or(u32::MAX);
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
        maximum: u16,
        total_mana_value: u64,
    ) -> bool {
        if maximum == 0 {
            return false;
        }
        self.start_effect_choice_inner(
            item,
            EffectChoiceKind::GraveyardCreatures {
                maximum,
                total_mana_value,
            },
            PlayerSet::Controller,
            evidence::cost(0).filter,
            0,
            Some(0),
            false,
            None,
        )
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
        let player = self.state.resolving.as_ref().unwrap().item.controller;
        let movements = choice.selected[player.index()]
            .iter()
            .filter(|r| {
                self.object_ref(r.card) == **r
                    && self.state.objects[r.card as usize].zone == Zone::Graveyard
                    && self.state.objects[r.card as usize].owner == player
                    && self.state.objects[r.card as usize].is_card()
                    && self.state.objects[r.card as usize].card.is_creature()
            })
            .map(|r| {
                let mut movement = self.entry_move(r.card, Zone::Battlefield, false);
                movement.controller = player;
                movement
            })
            .collect::<Vec<_>>();
        if !movements.is_empty() {
            self.stage_entries(movements, None);
        }
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
                        && matches!(object.zone, Zone::Hand | Zone::Graveyard)
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
        amount: u16,
        optional_skip: Option<usize>,
        greatest: bool,
        discard_mana_value_life: Option<Player>,
    ) -> bool {
        if matches!(kind, EffectChoiceKind::SacrificeOrDiscard { .. }) {
            assert_eq!(amount, 1, "one alternative resolution cost");
            assert_eq!(players, PlayerSet::Controller, "captured resolution payer");
        }
        let pc = self.state.resolving.as_ref().unwrap().pc;
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
        let mut choice = EffectChoice {
            discard_draw: None,
            discarded: None,
            kind,
            filter,
            amount: u32::from(amount),
            discard_mana_value_life,
            hand_defender: None,
            entry_life_bonus: None,
            hand_entry_filter: None,
            equipment_recipient: None,
            source: item
                .source
                .expect("effect choice requires object source")
                .object,
            required: [0; 2],
            selected: std::array::from_fn(|_| Vec::new()),
            greatest_value: [None; 2],
            skip_to: optional_skip.map(|skip| pc.checked_add(skip).expect("program offset")),
        };
        if let Some(skip_to) = choice.skip_to {
            assert!(skip_to <= self.state.resolving.as_ref().unwrap().program.len());
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
            if let Some(skip_to) = choice.skip_to
                && !private_optional_discard
                && !matches!(
                    kind,
                    EffectChoiceKind::SacrificeOrDiscard { .. }
                        | EffectChoiceKind::HandExileCopy
                        | EffectChoiceKind::HandPlot { .. }
                        | EffectChoiceKind::HandCreature
                        | EffectChoiceKind::HandPermanent { .. }
                        | EffectChoiceKind::HandLand { .. }
                )
                && available < usize::from(amount)
            {
                self.state.resolving.as_mut().unwrap().pc = skip_to;
                return false;
            }
            // Optional private choices always present the same actor/phase,
            // even with no eligible card. The player can decline without revealing
            // whether their hidden hand contains a matching card.
            choice.required[player.index()] = if private_optional_discard
                || matches!(
                    kind,
                    EffectChoiceKind::SacrificeOrDiscard { .. }
                        | EffectChoiceKind::HandExileCopy
                        | EffectChoiceKind::HandPlot { .. }
                        | EffectChoiceKind::HandCreature
                        | EffectChoiceKind::HandPermanent { .. }
                        | EffectChoiceKind::HandLand { .. }
                ) {
                u32::from(amount)
            } else {
                u32::from(amount).min(u32::try_from(available).capacity("choice count exhausted"))
            };
        }
        self.state.resolving.as_mut().unwrap().choice = Some(Box::new(choice));
        self.advance_effect_choice()
    }

    pub(super) fn advance_effect_choice(&mut self) -> bool {
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
        if matches!(
            choice.kind.payment_kind(),
            EffectChoiceKind::CoverUpSearch { .. }
        ) {
            self.state.pending = Pending::EffectChoice {
                player: self.state.resolving.as_ref().unwrap().item.controller,
            };
            return true;
        }
        if let EffectChoiceKind::GraveyardCreatures { maximum, .. } = choice.kind.payment_kind() {
            let player = self.state.resolving.as_ref().unwrap().item.controller;
            if choice.selected[player.index()].len() >= usize::from(maximum) {
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
        if choice.kind.payment_kind() == EffectChoiceKind::HandToTop {
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
            self.forget_library_top(player);
            let events = self.move_objects_raw(&ids, Zone::Library);
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
            let events = self.move_objects_raw(&[reference.card], Zone::Exile);
            for event in &events {
                if event.moved && event.destination == Zone::Exile {
                    self.remember_plotted(player, event.after);
                }
            }
            self.finish_zone_events(&events, &[]);
            return false;
        }
        if matches!(
            choice.kind.payment_kind(),
            EffectChoiceKind::HandExileCopy
                | EffectChoiceKind::HandCreature
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
            if enchantment && choice.hand_defender.is_none() {
                if self.state.objects.iter().enumerate().any(|(id, o)| {
                    o.zone == Zone::Battlefield
                        && o.controller == player.other()
                        && self.current_kinds(id as CardId) & 64 != 0
                }) {
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
                forced_tapped || enchantment,
            );
            movement.controller = player;
            movement.attach_equipment_to = choice.equipment_recipient;
            movement.attacking = choice.hand_defender;
            movement.life_bonus = choice
                .entry_life_bonus
                .map(|(subtype, life)| (player, subtype, life));
            self.stage_entries(vec![movement], None);
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
        // A ward cost is paid as one event: revalidate the complete selection
        // before making any sacrifice, including live restrictions.
        let ward = self
            .state
            .resolving
            .as_ref()
            .unwrap()
            .program
            .get(self.state.resolving.as_ref().unwrap().pc.saturating_sub(1))
            .is_some_and(|i| matches!(i, Instruction::CounterRelatedUnlessSacrifice(_)));
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
        if choice.kind.payment_kind() == EffectChoiceKind::CopyExiledTriggerCreature {
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
        } else if let EffectChoiceKind::ExileSourceAndNamed { amount, .. } =
            choice.kind.payment_kind()
        {
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
                | EffectChoiceKind::ExileCardFromAnyGraveyard
                | EffectChoiceKind::ExileGreatestManaValue
        ) || matches!(
            choice.kind.payment_kind(),
            EffectChoiceKind::TapPermanents(_)
        ) {
            let ids: Vec<_> = choice
                .selected
                .iter()
                .flat_map(|rows| rows.iter().map(|r| r.card))
                .collect();
            if matches!(
                choice.kind.payment_kind(),
                EffectChoiceKind::ExileGraveyard
                    | EffectChoiceKind::ExileCardFromAnyGraveyard
                    | EffectChoiceKind::ExileGreatestManaValue
            ) {
                let events = self.move_objects_raw(&ids, Zone::Exile);
                self.finish_zone_events(&events, &[]);
            } else {
                self.tap_objects(&ids);
            }
        } else if matches!(
            choice.kind.payment_kind(),
            EffectChoiceKind::SacrificeOrDiscard { .. }
        ) {
            // The resolving packet captures the payer/cause. A legal chosen cost
            // is paid even if its zone-change destination is replaced.
            let payer = self.state.resolving.as_ref().unwrap().item.controller;
            let reference = choice.selected[payer.index()][0];
            assert_eq!(self.object_ref(reference.card), reference);
            if self.state.objects[reference.card as usize].zone == Zone::Hand {
                self.discard_batch_caused(&[reference.card], None, Some(payer));
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

    pub(super) fn choose_effect_object(&mut self, player: Player, object: ObjectRef) {
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
        if matches!(
            choice.kind,
            EffectChoiceKind::HandPermanent { .. }
                | EffectChoiceKind::HandCreature
                | EffectChoiceKind::HandLand { .. }
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
        let ward_victim = resolution
            .program
            .get(resolution.pc.saturating_sub(1))
            .is_some_and(|instruction| {
                matches!(instruction, Instruction::CounterRelatedUnlessSacrifice(_))
            })
            .then_some(resolution.item.related_stack)
            .flatten();
        self.state.resolving.as_mut().unwrap().pc = choice.skip_to.expect("optional payment");
        if let Some(victim) = ward_victim {
            self.counter_stack_item(victim.id, false);
        }
        self.run_program();
    }

    pub(super) fn start_named_card_search(&mut self, item: &SpellView, maximum: u16) -> bool {
        let Some(owner) = self
            .program_players(PlayerSet::Targeted, item)
            .into_iter()
            .next()
        else {
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
                | EffectChoiceKind::SacrificeOrDiscard { .. }
                | EffectChoiceKind::HandToTop
                | EffectChoiceKind::HandExileCopy
                | EffectChoiceKind::HandPlot { .. }
                | EffectChoiceKind::HandCreature
                | EffectChoiceKind::HandPermanent { .. }
                | EffectChoiceKind::HandLand { .. }
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
            optional: choice.skip_to.is_some(),
            required: choice.required,
            selected: std::array::from_fn(|index| {
                choice.selected[index]
                    .iter()
                    .filter(|_| {
                        choice.kind.payment_kind() != EffectChoiceKind::Discard
                            || self.shares_information(
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
