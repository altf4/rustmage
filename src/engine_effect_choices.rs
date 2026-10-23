//! Simultaneous public sacrifices and private exact resolution-cost payments.
use super::*;
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum HandCreatureBound {
    ManaValue(u64),
    TotalStats(u64),
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) struct HandCreatureEntry {
    pub bound: HandCreatureBound,
    pub attacking: bool,
    pub haste_sacrifice: bool,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) struct EffectChoice {
    pub kind: EffectChoiceKind,
    pub filter: SacrificeFilter,
    pub amount: u16,
    pub source: ObjectRef,
    pub required: [u16; 2],
    pub selected: [Vec<ObjectRef>; 2],
    pub skip_to: Option<usize>,
    pub greatest_power: [Option<i128>; 2],
    pub discard_mana_value_life: Option<Player>,
    pub hand_defender: Option<CombatDefender>,
    pub hand_creature_entry: Option<HandCreatureEntry>,
    pub blight_copy_attacking: bool,
    pub counter_removal: Option<Box<CounterRemovalDomain>>,
    pub proliferate: Option<Box<proliferate::ProliferateSelection>>,
    pub hand_reveal: Option<Box<hand_reveal::HandRevealChoice>>,
    pub entry_life_bonus: Option<(&'static str, u16)>,
}
impl EffectChoice {
    pub fn bytes(&self) -> usize {
        std::mem::size_of::<Self>()
            + self.hand_reveal.as_ref().map_or(0, |choice| choice.bytes())
            + self.proliferate.as_ref().map_or(0, |_| {
                std::mem::size_of::<proliferate::ProliferateSelection>()
            })
            + self.counter_removal.as_ref().map_or(0, |domain| {
                std::mem::size_of::<CounterRemovalDomain>() + domain.available.owned_bytes()
            })
            + self.selected.iter().map(Vec::len).sum::<usize>() * std::mem::size_of::<ObjectRef>()
    }
}
impl Game {
    pub(super) fn effect_choice_objects(
        &self,
        choice: &EffectChoice,
    ) -> impl Iterator<Item = ObjectRef> + '_ {
        let fixed = choice.kind == EffectChoiceKind::CopyEntryObject;
        let event = if fixed {
            self.state
                .resolving
                .as_ref()
                .and_then(|r| r.item.entry_batch_event.as_ref())
                .map(|e| e.objects.as_slice())
        } else {
            None
        };
        event.into_iter().flatten().copied().chain(
            (0..if fixed { 0 } else { self.state.objects.len() })
                .map(|id| self.object_ref(id as CardId)),
        )
    }

    pub(super) fn effect_choice_candidate(
        &self,
        choice: &EffectChoice,
        player: Player,
        object: ObjectRef,
    ) -> bool {
        if choice.hand_reveal.is_some() {
            return self.hand_reveal_candidate(choice, object);
        }
        if choice.counter_removal.is_some() {
            return false;
        }
        if choice.kind == EffectChoiceKind::CopyEntryObject {
            return self.entry_batch_copy_recipe(object).is_some()
                && !choice.selected[player.index()].contains(&object);
        }
        if choice.kind == EffectChoiceKind::KeepSharedCreatureTypes {
            return self.keep_shared_type_candidate(choice, object);
        }
        let o = &self.state.objects[object.card as usize];
        if choice.proliferate.is_some() {
            return o.zone == Zone::Battlefield
                && self.object_ref(object.card) == object
                && self.counter_amounts(object.card).has_any()
                && !choice.selected[player.index()].contains(&object);
        }
        if matches!(
            choice.kind,
            EffectChoiceKind::Blight(_) | EffectChoiceKind::BlightCost(_)
        ) {
            return o.zone == Zone::Battlefield
                && o.controller == player
                && self.is_creature(object.card)
                && (matches!(
                    choice.kind,
                    EffectChoiceKind::Blight(_) | EffectChoiceKind::BlightCost(0)
                ) || self.can_receive_counters(object.card))
                && self.object_ref(object.card) == object
                && !choice.selected[player.index()].contains(&object);
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
        if choice.kind == EffectChoiceKind::ExileGraveyardCard {
            let filter = choice.filter;
            return o.zone == Zone::Graveyard
                && o.owner == player
                && !o.is_token()
                && !o.is_spell_copy()
                && self.object_ref(object.card) == object
                && !choice.selected[player.index()].contains(&object)
                && (filter.kinds == 0 || o.card.kind_mask() & u16::from(filter.kinds) != 0)
                && (!filter.nonland || !o.card.is_land())
                && (!filter.other || object != choice.source)
                && (!filter.source_only || object == choice.source)
                && o.card.keywords().contains(filter.keywords)
                && filter.subtype.is_none_or(|kind| o.card.has_subtype(kind));
        }
        let hand_choice = matches!(
            choice.kind,
            EffectChoiceKind::BottomHand
                | EffectChoiceKind::Discard
                | EffectChoiceKind::ExileFromHand
                | EffectChoiceKind::HandCreature
                | EffectChoiceKind::HandCreatureAndLand
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
            EffectChoiceKind::BottomHand
                | EffectChoiceKind::Discard
                | EffectChoiceKind::ExileFromHand
                | EffectChoiceKind::HandCreature
                | EffectChoiceKind::HandCreatureAndLand
                | EffectChoiceKind::HandLand { .. }
        ) {
            return o.zone == Zone::Hand
                && match choice.kind {
                    EffectChoiceKind::HandCreatureAndLand => {
                        let mask = o.card.kind_mask() & 3;
                        match choice.selected[player.index()].as_slice() {
                            [] => mask != 0,
                            [first] => {
                                let first =
                                    self.state.objects[first.card as usize].card.kind_mask();
                                (first & 1 != 0 && mask & 2 != 0)
                                    || (first & 2 != 0 && mask & 1 != 0)
                            }
                            _ => false,
                        }
                    }
                    EffectChoiceKind::HandCreature => {
                        o.card.is_creature()
                            && choice
                                .hand_creature_entry
                                .is_none_or(|entry| match entry.bound {
                                    HandCreatureBound::ManaValue(maximum) => {
                                        o.card.fixed_mana_value() <= maximum
                                    }
                                    HandCreatureBound::TotalStats(maximum) => {
                                        let stats = self.stats(object.card);
                                        stats.power.saturating_add(stats.toughness)
                                            <= i128::from(maximum)
                                    }
                                })
                    }
                    EffectChoiceKind::HandLand { .. } => o.card.is_land() && o.controller == player,
                    EffectChoiceKind::Discard => {
                        choice.filter.kinds == 0
                            || o.card.kind_mask() & u16::from(choice.filter.kinds) != 0
                    }
                    _ => true,
                };
        }
        if choice.greatest_power[player.index()]
            .is_some_and(|max| self.object_view(object.card).power != max)
        {
            return false;
        }
        let f = choice.filter;
        o.zone == Zone::Battlefield
            && (choice.kind != EffectChoiceKind::SacrificeCreatureOrToken
                || self.is_creature(object.card)
                || o.is_token())
            && (choice.kind != EffectChoiceKind::SacrificeLandOrToken
                || self.current_kinds(object.card) & 2 != 0
                || o.is_token())
            && (!matches!(choice.kind, EffectChoiceKind::TapPermanents(_)) || !o.tapped)
            && self.keywords(object.card).contains(f.keywords)
            && (f.kinds == 0 || self.current_kinds(object.card) & u16::from(f.kinds) != 0)
            && (!f.source_only || object == choice.source)
            && (!f.nonland || self.current_kinds(object.card) & 2 == 0)
            && (!f.other || object != choice.source)
            && f.tokens.is_none_or(|token| o.is_token() == token)
            && f.subtype
                .is_none_or(|subtype| self.has_subtype(object.card, subtype))
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
    pub(super) fn start_hand_creature_attacking(&mut self, item: &SpellView, maximum: u64) -> bool {
        let pending = self.start_hand_creature(item);
        self.state
            .resolving
            .as_mut()
            .unwrap()
            .choice
            .as_mut()
            .unwrap()
            .hand_creature_entry = Some(HandCreatureEntry {
            bound: HandCreatureBound::ManaValue(maximum),
            attacking: true,
            haste_sacrifice: false,
        });
        pending
    }
    pub(super) fn start_hand_creature_haste_sacrifice(
        &mut self,
        item: &SpellView,
        maximum: u64,
    ) -> bool {
        let pending = self.start_hand_creature(item);
        self.state
            .resolving
            .as_mut()
            .unwrap()
            .choice
            .as_mut()
            .unwrap()
            .hand_creature_entry = Some(HandCreatureEntry {
            bound: HandCreatureBound::TotalStats(maximum),
            attacking: false,
            haste_sacrifice: true,
        });
        pending
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
            let ids: Vec<_> = choice
                .selected
                .iter()
                .flatten()
                .filter(|object| self.object_ref(object.card) == **object)
                .map(|object| object.card)
                .collect();
            self.untap_objects(&ids, untap_rules::UntapCause::Effect);
            return;
        }
        let EffectChoiceKind::LandCards { tapped, .. } = choice.kind else {
            unreachable!()
        };
        let player = self.state.resolving.as_ref().unwrap().item.controller;
        let movements = choice.selected[player.index()]
            .iter()
            .map(|reference| {
                assert_eq!(self.object_ref(reference.card), *reference);
                let object = &self.state.objects[reference.card as usize];
                assert!(
                    object.owner == player
                        && matches!(object.zone, Zone::Hand | Zone::Graveyard)
                        && object.card.is_land()
                );
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
        let pc = self.state.resolving.as_ref().unwrap().pc;
        let mut choice = EffectChoice {
            kind,
            filter,
            amount,
            discard_mana_value_life,
            hand_defender: None,
            hand_creature_entry: None,
            counter_removal: None,
            proliferate: None,
            entry_life_bonus: None,
            hand_reveal: None,
            blight_copy_attacking: false,
            source: item.object,
            required: [0; 2],
            selected: std::array::from_fn(|_| Vec::new()),
            greatest_power: [None; 2],
            skip_to: optional_skip.map(|skip| pc.checked_add(skip).expect("program offset")),
        };
        if let Some(skip_to) = choice.skip_to {
            assert!(skip_to <= self.state.resolving.as_ref().unwrap().program.len());
        }
        // A private filtered payment must offer the same phase even when hidden
        // contents cannot pay; declining it is distinct from satisfying a zero cost.
        let private_filtered_discard = kind == EffectChoiceKind::Discard && filter.kinds != 0;
        for player in self.program_players(players, item) {
            if greatest {
                choice.greatest_power[player.index()] = self
                    .state
                    .objects
                    .iter()
                    .enumerate()
                    .filter(|(id, o)| {
                        o.zone == Zone::Battlefield
                            && o.controller == player
                            && self.is_creature(*id as CardId)
                    })
                    .map(|(id, _)| self.object_view(id as CardId).power)
                    .max();
            }
            let available = self
                .effect_choice_objects(&choice)
                .filter(|&object| self.effect_choice_candidate(&choice, player, object))
                .count();
            if kind == EffectChoiceKind::PowerDifferenceBoost && available < usize::from(amount) {
                return false;
            }
            if let Some(skip_to) = choice.skip_to
                && !matches!(
                    kind,
                    EffectChoiceKind::BottomHand
                        | EffectChoiceKind::HandCreature
                        | EffectChoiceKind::HandCreatureAndLand
                        | EffectChoiceKind::HandLand { .. }
                )
                && !private_filtered_discard
                && available < usize::from(amount)
            {
                self.state.resolving.as_mut().unwrap().pc = skip_to;
                return false;
            }
            // Optional private hand placement always presents the same actor/phase,
            // even with no eligible card. The player can decline without revealing
            // whether their hidden hand contains a creature or land.
            choice.required[player.index()] = if private_filtered_discard
                || matches!(
                    kind,
                    EffectChoiceKind::BottomHand
                        | EffectChoiceKind::HandCreature
                        | EffectChoiceKind::HandCreatureAndLand
                        | EffectChoiceKind::HandLand { .. }
                ) {
                amount
            } else {
                amount.min(u16::try_from(available).unwrap_or(u16::MAX))
            };
        }
        self.state.resolving.as_mut().unwrap().choice = Some(Box::new(choice));
        self.advance_effect_choice()
    }

    pub(super) fn finish_hand_selection(&mut self) {
        let resolution = self.state.resolving.as_mut().unwrap();
        let player = resolution.item.controller;
        let choice = resolution.choice.take().unwrap();
        assert_eq!(choice.kind, EffectChoiceKind::HandCreatureAndLand);
        let moves = choice.selected[player.index()]
            .iter()
            .map(|reference| {
                assert_eq!(self.object_ref(reference.card), *reference);
                let object = &self.state.objects[reference.card as usize];
                assert!(object.zone == Zone::Hand && object.owner == player);
                let mut movement = self.entry_move(reference.card, Zone::Battlefield, false);
                movement.controller = player;
                movement
            })
            .collect::<Vec<_>>();
        if !moves.is_empty() {
            self.stage_entries(moves, None);
        }
    }
    fn advance_effect_choice(&mut self) -> bool {
        let resolution = self.state.resolving.as_ref().unwrap();
        let player = resolution.item.controller;
        let choice = resolution.choice.as_ref().unwrap();
        if choice.kind == EffectChoiceKind::HandCreatureAndLand {
            if choice.selected[player.index()].len() >= 2 {
                self.finish_hand_selection();
                return false;
            }
            self.state.pending = Pending::EffectChoice { player };
            return true;
        }

        if self
            .state
            .resolving
            .as_ref()
            .unwrap()
            .choice
            .as_ref()
            .unwrap()
            .hand_reveal
            .is_some()
        {
            return self.advance_hand_reveal();
        }
        if self
            .state
            .resolving
            .as_ref()
            .unwrap()
            .choice
            .as_ref()
            .unwrap()
            .kind
            == EffectChoiceKind::KeepSharedCreatureTypes
        {
            return self.advance_keep_shared_types();
        }
        let choice = self
            .state
            .resolving
            .as_ref()
            .unwrap()
            .choice
            .as_ref()
            .unwrap();
        if let EffectChoiceKind::UntapLands { maximum } = choice.kind {
            let player = self.state.resolving.as_ref().unwrap().item.controller;
            if choice.selected[player.index()].len() >= usize::from(maximum) {
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
            if choice.selected[player.index()].len() < usize::from(choice.required[player.index()])
            {
                self.state.pending = Pending::EffectChoice { player };
                return true;
            }
        }
        if matches!(
            choice.kind,
            EffectChoiceKind::HandCreature
                | EffectChoiceKind::HandCreatureAndLand
                | EffectChoiceKind::HandLand { .. }
        ) {
            let player = self.state.resolving.as_ref().unwrap().item.controller;
            let Some(reference) = choice.selected[player.index()].first().copied() else {
                self.state.resolving.as_mut().unwrap().choice = None;
                return false;
            };
            let enchantment = choice.kind == EffectChoiceKind::HandCreature
                && self.state.objects[reference.card as usize].card.kind_mask() & 32 != 0;
            let legacy_enchantment = enchantment && choice.hand_creature_entry.is_none();
            let attacking = choice
                .hand_creature_entry
                .map_or(enchantment, |entry| entry.attacking);
            let forced_tapped = choice
                .hand_creature_entry
                .is_some_and(|entry| entry.attacking)
                || matches!(choice.kind, EffectChoiceKind::HandLand { tapped: true });
            if attacking && choice.hand_defender.is_none() {
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
                forced_tapped || legacy_enchantment,
            );
            movement.controller = player;
            movement.haste_sacrifice_after = choice
                .hand_creature_entry
                .is_some_and(|entry| entry.haste_sacrifice);
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
        if choice.kind == EffectChoiceKind::CopyEntryObject {
            let controller = self.state.resolving.as_ref().unwrap().item.controller;
            if let Some(&object) = choice.selected[controller.index()].first()
                && let Some(recipe) = self.entry_batch_copy_recipe(object)
            {
                self.stage_copy_creation(controller, recipe, 1);
            }
        } else if let EffectChoiceKind::Blight(amount) | EffectChoiceKind::BlightCost(amount) =
            choice.kind
        {
            if choice.blight_copy_attacking {
                let object = choice
                    .selected
                    .iter()
                    .flatten()
                    .copied()
                    .next()
                    .expect("blighted creature");
                let mut recipe = self.copiable_recipe(object);
                recipe.exceptions = recipe.exceptions.union(CopyExceptions::SACRIFICE_ONLY);
                let continuation = &mut self.state.resolving.as_mut().unwrap().continuation;
                assert!(continuation.is_none());
                *continuation = Some(Box::new(
                    programs::ProgramContinuation::CreateAttackingCopy(recipe),
                ));
            }
            let choices = [self.state.active, self.state.active.other()]
                .into_iter()
                .flat_map(|p| {
                    choice.selected[p.index()]
                        .iter()
                        .copied()
                        .map(move |object| (p, object))
                })
                .collect();
            self.blight_counters(
                choices,
                amount,
                matches!(choice.kind, EffectChoiceKind::Blight(_)),
                replacements::ResumeMode::Program,
            );
        } else if matches!(choice.kind, EffectChoiceKind::TapPermanents(_)) {
            let ids = choice
                .selected
                .iter()
                .flatten()
                .map(|object| object.card)
                .collect::<Vec<_>>();
            self.tap_objects(&ids);
        } else if choice.kind == EffectChoiceKind::PowerDifferenceBoost {
            let item = self.state.resolving.as_ref().unwrap().item.clone();
            let selected = &choice.selected[item.controller.index()];
            assert_eq!(selected.len(), 2);
            // Freeze X before either creature receives growth. This choice is not targeting.
            let amount = self
                .stats(selected[0].card)
                .power
                .abs_diff(self.stats(selected[1].card).power);
            let growth =
                i128::try_from(amount).capacity("power difference representation exhausted");
            self.draw_many(item.controller, amount);
            let ids: Vec<_> = selected.iter().map(|object| object.card).collect();
            for &id in &ids {
                self.state.objects[id as usize].growth += Stats {
                    power: growth,
                    toughness: growth,
                };
            }
            self.grant_keyword_effects(&ids, item.object, Keywords::TRAMPLE);
        } else if matches!(
            choice.kind,
            EffectChoiceKind::ExileFromHand | EffectChoiceKind::ExileGraveyardCard
        ) {
            let zone = if choice.kind == EffectChoiceKind::ExileFromHand {
                Zone::Hand
            } else {
                Zone::Graveyard
            };
            let mut ids = Vec::new();
            for player in [self.state.active, self.state.active.other()] {
                for object in &choice.selected[player.index()] {
                    if self.object_ref(object.card) == *object
                        && self.state.objects[object.card as usize].zone == zone
                        && self.state.objects[object.card as usize].owner == player
                    {
                        ids.push(object.card);
                    }
                }
            }
            let events = self.move_objects_raw(&ids, Zone::Exile);
            self.finish_zone_events(&events, &[]);
        } else if choice.kind == EffectChoiceKind::BottomHand {
            let item = self.state.resolving.as_ref().unwrap().item.clone();
            for object in &choice.selected[item.controller.index()] {
                if self.object_ref(object.card) == *object
                    && self.state.objects[object.card as usize].zone == Zone::Hand
                {
                    self.move_private_hand_to_bottom(item.object, *object);
                }
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
            self.discard_batch_caused(&ids, None, Some(cause));
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
        if self
            .state
            .resolving
            .as_ref()
            .unwrap()
            .choice
            .as_ref()
            .unwrap()
            .hand_reveal
            .is_some()
        {
            self.choose_hand_reveal_object(object);
            return;
        }
        let choice = self
            .state
            .resolving
            .as_ref()
            .unwrap()
            .choice
            .as_ref()
            .unwrap();
        let player = if choice.kind == EffectChoiceKind::KeepSharedCreatureTypes {
            self.keep_shared_type_player(choice)
                .expect("unfinished creature choice")
        } else {
            player
        };
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
        if let Some(reveal) = &choice.hand_reveal {
            return self.hand_reveal_view(viewer, choice, reveal);
        }
        if matches!(
            choice.kind,
            EffectChoiceKind::BottomHand
                | EffectChoiceKind::Discard
                | EffectChoiceKind::ExileFromHand
                | EffectChoiceKind::HandCreature
                | EffectChoiceKind::HandCreatureAndLand
                | EffectChoiceKind::HandLand { .. }
        ) && viewer != actor
        {
            return None;
        }
        let public_only =
            viewer != actor && matches!(choice.kind, EffectChoiceKind::LandCards { .. });
        let candidates = self
            .effect_choice_objects(choice)
            .filter(|&object| self.effect_choice_candidate(choice, actor, object))
            .filter(|object| {
                !public_only || self.state.objects[object.card as usize].zone == Zone::Graveyard
            })
            .collect();
        Some(EffectChoiceView {
            kind: choice.kind,
            hand_reveal: None,
            counter_removal: choice.counter_removal.as_deref().cloned(),
            proliferate: self.proliferate_choice_view(choice),
            amount: choice.amount,
            optional: choice.skip_to.is_some() || choice.proliferate.is_some(),
            required: choice.required,
            selected: std::array::from_fn(|index| {
                choice.selected[index]
                    .iter()
                    .copied()
                    .filter(|object| {
                        (!matches!(
                            choice.kind,
                            EffectChoiceKind::ExileFromHand
                                | EffectChoiceKind::BottomHand
                                | EffectChoiceKind::Discard
                                | EffectChoiceKind::HandCreature
                                | EffectChoiceKind::HandCreatureAndLand
                                | EffectChoiceKind::HandLand { .. }
                        ) || index == viewer.index())
                            && (!public_only
                                || self.state.objects[object.card as usize].zone == Zone::Graveyard)
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
