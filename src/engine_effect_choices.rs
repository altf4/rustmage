//! Simultaneous public sacrifices and private exact resolution-cost payments.
use super::*;
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
}
impl EffectChoice {
    pub fn bytes(&self) -> usize {
        std::mem::size_of::<Self>()
            + self.selected.iter().map(Vec::len).sum::<usize>() * std::mem::size_of::<ObjectRef>()
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
        if o.owner != player
            || self.object_ref(object.card) != object
            || choice.selected[player.index()].contains(&object)
        {
            return false;
        }
        if matches!(
            choice.kind,
            EffectChoiceKind::Discard | EffectChoiceKind::HandCreature
        ) {
            return o.zone == Zone::Hand
                && (choice.kind != EffectChoiceKind::HandCreature || o.card.is_creature());
        }
        if choice.greatest_power[player.index()]
            .is_some_and(|max| self.object_view(object.card).power != max)
        {
            return false;
        }
        let f = choice.filter;
        o.zone == Zone::Battlefield
            && self.keywords(object.card).contains(f.keywords)
            && (f.kinds == 0 || self.current_kinds(object.card) & u16::from(f.kinds) != 0)
            && (!f.source_only || object == choice.source)
            && (!f.nonland || !o.card.is_land())
            && (!f.other || object != choice.source)
            && f.tokens.is_none_or(|token| o.card.is_token() == token)
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
                && o.owner == player.other()
                && o.card.is_planeswalker()
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
            source: item.object,
            required: [0; 2],
            selected: std::array::from_fn(|_| Vec::new()),
            greatest_power: [None; 2],
            skip_to: optional_skip.map(|skip| pc.checked_add(skip).expect("program offset")),
        };
        if let Some(skip_to) = choice.skip_to {
            assert!(skip_to <= self.state.resolving.as_ref().unwrap().program.len());
        }
        for player in self.program_players(players, item) {
            if greatest {
                choice.greatest_power[player.index()] = self
                    .state
                    .objects
                    .iter()
                    .enumerate()
                    .filter(|(id, o)| {
                        o.zone == Zone::Battlefield
                            && o.owner == player
                            && self.is_creature(*id as CardId)
                    })
                    .map(|(id, _)| self.object_view(id as CardId).power)
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
                && available < usize::from(amount)
            {
                self.state.resolving.as_mut().unwrap().pc = skip_to;
                return false;
            }
            choice.required[player.index()] =
                amount.min(u16::try_from(available).unwrap_or(u16::MAX));
        }
        self.state.resolving.as_mut().unwrap().choice = Some(Box::new(choice));
        self.advance_effect_choice()
    }

    fn advance_effect_choice(&mut self) -> bool {
        let choice = self
            .state
            .resolving
            .as_ref()
            .unwrap()
            .choice
            .as_ref()
            .unwrap();
        for player in [self.state.active, self.state.active.other()] {
            if choice.selected[player.index()].len() < usize::from(choice.required[player.index()])
            {
                self.state.pending = Pending::EffectChoice { player };
                return true;
            }
        }
        if choice.kind == EffectChoiceKind::HandCreature {
            let player = self.state.resolving.as_ref().unwrap().item.controller;
            let Some(reference) = choice.selected[player.index()].first().copied() else {
                self.state.resolving.as_mut().unwrap().choice = None;
                return false;
            };
            let enchantment =
                self.state.objects[reference.card as usize].card.kind_mask() & 32 != 0;
            if enchantment && choice.hand_defender.is_none() {
                if self.state.objects.iter().any(|o| {
                    o.zone == Zone::Battlefield
                        && o.owner == player.other()
                        && o.card.is_planeswalker()
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
                enchantment || self.enters_tapped(reference.card),
            );
            movement.attacking = choice.hand_defender;
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
        if choice.kind == EffectChoiceKind::Discard {
            let ids: Vec<_> = [self.state.active, self.state.active.other()]
                .into_iter()
                .flat_map(|p| choice.selected[p.index()].iter().map(|object| object.card))
                .collect();
            let mana_value: i128 = ids
                .iter()
                .map(|&id| i128::from(self.state.objects[id as usize].card.fixed_mana_value()))
                .sum();
            self.discard_batch(&ids, None);
            if let Some(player) = choice.discard_mana_value_life {
                self.lose_life(player, mana_value);
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
            EffectChoiceKind::Discard | EffectChoiceKind::HandCreature
        ) && viewer != actor
        {
            return None;
        }
        let candidates = self
            .state
            .objects
            .iter()
            .enumerate()
            .map(|(id, _)| self.object_ref(id as CardId))
            .filter(|&object| self.effect_choice_candidate(choice, actor, object))
            .collect();
        Some(EffectChoiceView {
            kind: choice.kind,
            amount: choice.amount,
            optional: choice.skip_to.is_some(),
            required: choice.required,
            selected: choice.selected.clone(),
            candidates: if matches!(self.state.pending, Pending::HandDefender { .. }) {
                Vec::new()
            } else {
                candidates
            },
            defender_choice: matches!(self.state.pending, Pending::HandDefender { .. }),
        })
    }
}
