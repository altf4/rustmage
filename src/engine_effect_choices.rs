//! Simultaneous public sacrifices and private exact resolution-cost payments.
use super::*;
#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) struct DiscardDrawChoice {
    players: [bool; 2],
    finished: [bool; 2],
    bonus: u16,
}
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
}
impl EffectChoice {
    pub fn bytes(&self) -> usize {
        std::mem::size_of::<Self>()
            + (self.selected.iter().map(Vec::len).sum::<usize>()
                + self.discarded.as_ref().map_or(0, |rows| rows.len()))
                * std::mem::size_of::<ObjectRef>()
    }
}
impl Game {
    pub(super) fn effect_choice_candidate(
        &self,
        choice: &EffectChoice,
        player: Player,
        object: ObjectRef,
    ) -> bool {
        let o = &self.state.objects[object.card as usize];
        if matches!(choice.kind, EffectChoiceKind::DiscardedLand { .. }) {
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
        if choice.kind == EffectChoiceKind::ExileGraveyard {
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
        if matches!(choice.kind, EffectChoiceKind::TapPermanents(_)) && o.tapped {
            return false;
        }
        if let EffectChoiceKind::UntapLands { maximum } = choice.kind {
            return o.zone == Zone::Battlefield
                && self.current_kinds(object.card) & 2 != 0
                && self.object_ref(object.card) == object
                && !choice.selected[player.index()].contains(&object)
                && choice.selected[player.index()].len() < usize::from(maximum);
        }
        if let EffectChoiceKind::LandCards { maximum, .. } = choice.kind {
            return o.owner == player
                && self.object_ref(object.card) == object
                && matches!(o.zone, Zone::Hand | Zone::Graveyard)
                && o.card.is_land()
                && !choice.selected[player.index()].contains(&object)
                && choice.selected[player.index()].len() < maximum as usize;
        }
        if let EffectChoiceKind::SacrificeOrDiscard { nonland } = choice.kind
            && o.zone == Zone::Hand
        {
            return o.owner == player
                && self.object_ref(object.card) == object
                && !choice.selected[player.index()].contains(&object)
                && (!nonland || !o.card.is_land());
        }
        let hand_choice = matches!(
            choice.kind,
            EffectChoiceKind::Discard
                | EffectChoiceKind::HandToTop
                | EffectChoiceKind::HandExileCopy
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
            choice.kind,
            EffectChoiceKind::Discard
                | EffectChoiceKind::HandToTop
                | EffectChoiceKind::HandExileCopy
                | EffectChoiceKind::HandCreature
                | EffectChoiceKind::HandPermanent { .. }
                | EffectChoiceKind::HandLand { .. }
        ) {
            return o.zone == Zone::Hand
                && match choice.kind {
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
                    }
                    EffectChoiceKind::HandExileCopy => o.card.is_creature() || o.card.is_artifact(),
                    EffectChoiceKind::HandLand { .. } => o.card.is_land() && o.controller == player,
                    _ => true,
                };
        }
        if choice.greatest_value[player.index()].is_some_and(|max| {
            let value = if choice.kind == EffectChoiceKind::ExileGreatestManaValue {
                i128::from(self.copiable_recipe(object).definition.fixed_mana_value())
            } else {
                self.object_view(object.card).power
            };
            value != max
        }) {
            return false;
        }
        let f = choice.filter;
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
            source: item.object,
            required: [0; 2],
            selected: std::array::from_fn(|_| Vec::new()),
            skip_to: None,
            greatest_value: [None; 2],
            discard_mana_value_life: None,
            hand_defender: None,
            entry_life_bonus: None,
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
                        o.zone == Zone::Hand && o.owner == player
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
                .filter(|(_, o)| o.zone == Zone::Hand && participants[o.owner.index()])
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
            }),
            kind: EffectChoiceKind::Discard,
            filter: evidence::cost(0).filter,
            amount: 0,
            source: item.object,
            required: [0; 2],
            selected: std::array::from_fn(|_| Vec::new()),
            skip_to: Some(pc),
            greatest_value: [None; 2],
            discard_mana_value_life: None,
            hand_defender: None,
            entry_life_bonus: None,
        }));
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
        let source = self.state.resolving.as_ref().unwrap().item.object;
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
        if matches!(choice.kind, EffectChoiceKind::UntapLands { .. }) {
            for object in choice.selected.iter().flatten() {
                self.state.objects[object.card as usize].tapped = false;
            }
            return;
        }
        let (tapped, discarded) = match choice.kind {
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
        let mut choice = EffectChoice {
            discard_draw: None,
            discarded: None,
            kind,
            filter,
            amount: u32::from(amount),
            discard_mana_value_life,
            hand_defender: None,
            entry_life_bonus: None,
            source: item.object,
            required: [0; 2],
            selected: std::array::from_fn(|_| Vec::new()),
            greatest_value: [None; 2],
            skip_to: optional_skip.map(|skip| pc.checked_add(skip).expect("program offset")),
        };
        if let Some(skip_to) = choice.skip_to {
            assert!(skip_to <= self.state.resolving.as_ref().unwrap().program.len());
        }
        let private_optional_discard = kind == EffectChoiceKind::Discard && optional_skip.is_some();
        for player in self.program_players(players, item) {
            if greatest || kind == EffectChoiceKind::ExileGreatestManaValue {
                choice.greatest_value[player.index()] = self
                    .state
                    .objects
                    .iter()
                    .enumerate()
                    .filter(|(id, o)| {
                        o.zone == Zone::Battlefield
                            && o.controller == player
                            && if kind == EffectChoiceKind::ExileGreatestManaValue {
                                self.current_kinds(*id as CardId) & 65 != 0
                            } else {
                                self.is_creature(*id as CardId)
                            }
                    })
                    .map(|(id, _)| {
                        if kind == EffectChoiceKind::ExileGreatestManaValue {
                            i128::from(
                                self.copiable_recipe(self.object_ref(id as CardId))
                                    .definition
                                    .fixed_mana_value(),
                            )
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
                if draw.players[player.index()] && !draw.finished[player.index()] {
                    self.state.pending = Pending::EffectChoice { player };
                    return true;
                }
            }
        }
        if let EffectChoiceKind::UntapLands { maximum } = choice.kind {
            let player = self.state.resolving.as_ref().unwrap().item.controller;
            if choice.selected[player.index()].len() >= usize::from(maximum) {
                self.finish_land_selection();
                return false;
            }
            self.state.pending = Pending::EffectChoice { player };
            return true;
        }
        if matches!(choice.kind, EffectChoiceKind::DiscardedLand { .. }) {
            let player = self.state.resolving.as_ref().unwrap().item.controller;
            if !choice.selected[player.index()].is_empty() {
                self.finish_land_selection();
                return false;
            }
            self.state.pending = Pending::EffectChoice { player };
            return true;
        }
        if let EffectChoiceKind::LandCards { maximum, .. } = choice.kind {
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
        if choice.kind == EffectChoiceKind::HandToTop {
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
            choice.kind,
            EffectChoiceKind::HandExileCopy
                | EffectChoiceKind::HandCreature
                | EffectChoiceKind::HandPermanent { .. }
                | EffectChoiceKind::HandLand { .. }
        ) {
            let player = self.state.resolving.as_ref().unwrap().item.controller;
            let Some(reference) = choice.selected[player.index()].first().copied() else {
                self.state.resolving.as_mut().unwrap().choice = None;
                return false;
            };
            if choice.kind == EffectChoiceKind::HandExileCopy {
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
            let enchantment = choice.kind == EffectChoiceKind::HandCreature
                && self.state.objects[reference.card as usize].card.kind_mask() & 32 != 0;
            let forced_tapped = matches!(
                choice.kind,
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
        if matches!(
            choice.kind,
            EffectChoiceKind::ExileGraveyard | EffectChoiceKind::ExileGreatestManaValue
        ) || matches!(choice.kind, EffectChoiceKind::TapPermanents(_))
        {
            let ids: Vec<_> = choice
                .selected
                .iter()
                .flat_map(|rows| rows.iter().map(|r| r.card))
                .collect();
            if matches!(
                choice.kind,
                EffectChoiceKind::ExileGraveyard | EffectChoiceKind::ExileGreatestManaValue
            ) {
                let events = self.move_objects_raw(&ids, Zone::Exile);
                self.finish_zone_events(&events, &[]);
            } else {
                self.tap_objects(&ids);
            }
        } else if matches!(choice.kind, EffectChoiceKind::SacrificeOrDiscard { .. }) {
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
        } else if choice.kind == EffectChoiceKind::Discard {
            let ids: Vec<_> = [self.state.active, self.state.active.other()]
                .into_iter()
                .flat_map(|p| choice.selected[p.index()].iter().map(|object| object.card))
                .collect();
            let mana_value: i128 = ids
                .iter()
                .map(|&id| i128::from(self.state.objects[id as usize].card.fixed_mana_value()))
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
                self.sacrifice_batch(&sacrifices);
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
        let player = self.actor().expect("effect choice actor");
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
        self.state.resolving.as_mut().unwrap().pc = choice.skip_to.expect("optional payment");
        self.run_program();
    }

    pub(super) fn effect_choice_view(&self, viewer: Player) -> Option<EffectChoiceView> {
        let choice = self.state.resolving.as_ref()?.choice.as_ref()?;
        let actor = self.actor()?;
        if matches!(
            choice.kind,
            EffectChoiceKind::Discard
                | EffectChoiceKind::SacrificeOrDiscard { .. }
                | EffectChoiceKind::HandToTop
                | EffectChoiceKind::HandExileCopy
                | EffectChoiceKind::HandCreature
                | EffectChoiceKind::HandPermanent { .. }
                | EffectChoiceKind::HandLand { .. }
        ) && viewer != actor
        {
            return None;
        }
        let public_only =
            viewer != actor && matches!(choice.kind, EffectChoiceKind::LandCards { .. });
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
                    .filter(|_| choice.kind != EffectChoiceKind::Discard || index == viewer.index())
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
        })
    }
}
