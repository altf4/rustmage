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
        if choice.kind == EffectChoiceKind::Discard {
            return o.zone == Zone::Hand;
        }
        let f = choice.filter;
        o.zone == Zone::Battlefield
            && o.keywords().contains(f.keywords)
            && (f.kinds == 0 || o.card.kind_mask() & f.kinds != 0)
            && (!f.source_only || object == choice.source)
            && (!f.nonland || !o.card.is_land())
            && (!f.other || object != choice.source)
            && f.tokens.is_none_or(|token| o.card.is_token() == token)
            && f.subtype.is_none_or(|subtype| {
                o.card
                    .info()
                    .type_line
                    .split_once(" — ")
                    .is_some_and(|(_, types)| types.split_whitespace().any(|word| word == subtype))
            })
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
        let pc = self.state.resolving.as_ref().unwrap().pc;
        let mut choice = EffectChoice {
            kind,
            filter,
            amount,
            source: item.object,
            required: [0; 2],
            selected: std::array::from_fn(|_| Vec::new()),
            skip_to: optional_skip.map(|skip| pc.checked_add(skip).expect("program offset")),
        };
        if let Some(skip_to) = choice.skip_to {
            assert!(skip_to <= self.state.resolving.as_ref().unwrap().program.len());
        }
        for player in self.program_players(players, item) {
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
        let choice = self
            .state
            .resolving
            .as_mut()
            .unwrap()
            .choice
            .take()
            .unwrap();
        let ids: Vec<_> = [self.state.active, self.state.active.other()]
            .into_iter()
            .flat_map(|p| choice.selected[p.index()].iter().map(|object| object.card))
            .collect();
        if !ids.is_empty() {
            let sources = if choice.kind == EffectChoiceKind::Sacrifice {
                self.death_trigger_sources()
            } else {
                Vec::new()
            };
            let events = self.move_objects_raw(&ids, Zone::Graveyard);
            self.finish_zone_events(&events, &sources);
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
        if choice.kind == EffectChoiceKind::Discard && viewer != actor {
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
            candidates,
        })
    }
}
