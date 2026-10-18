//! A single owned forage operation, shared by costs and resolving programs.
use super::*;
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum ForageRoute {
    ExileThree,
    Food,
    Black,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) struct ForageSelection {
    pub route: Option<ForageRoute>,
    pub selected: Vec<ObjectRef>,
    pub skip_to: usize,
}
impl ForageSelection {
    pub fn complete(&self) -> bool {
        match self.route {
            Some(ForageRoute::Black) => true,
            Some(ForageRoute::Food) => self.selected.len() == 1,
            Some(ForageRoute::ExileThree) => self.selected.len() == 3,
            None => false,
        }
    }
    pub fn bytes(&self) -> usize {
        std::mem::size_of::<Self>() + self.selected.len() * std::mem::size_of::<ObjectRef>()
    }
}
#[derive(Clone, Debug, Serialize)]
pub struct ForageView {
    pub player: Player,
    pub optional: bool,
    pub paid: bool,
    pub routes: Vec<ForageRoute>,
    pub route: Option<ForageRoute>,
    pub selected: Vec<ObjectRef>,
    pub candidates: Vec<ObjectRef>,
}
pub(super) fn is_forage_cost(spec: AdditionalCost) -> bool {
    matches!(
        spec.kind,
        EffectChoiceKind::Forage | EffectChoiceKind::ForageOrBlack
    )
}
pub(super) fn food_cost() -> AdditionalCost {
    AdditionalCost {
        kind: EffectChoiceKind::Sacrifice,
        filter: SacrificeFilter {
            artifact_enchantment_or_token: false,
            keywords: Keywords::NONE,
            source_only: false,
            kinds: 0,
            nonland: false,
            other: false,
            tokens: None,
            subtype: Some("Food"),
        },
    }
}
impl Game {
    pub(super) fn food_sacrifice_triggers(
        &mut self,
        sacrifices: &[(Player, CardId)],
        sources: &[ObjectView],
    ) {
        for source in sources {
            if sacrifices
                .iter()
                .any(|&(player, id)| player == source.controller && self.has_subtype(id, "Food"))
            {
                for (index, a) in source.card.triggers().iter().enumerate() {
                    if a.condition.event() == TriggerCondition::FoodsSacrificed {
                        self.queue_trigger(source, index);
                    }
                }
            }
        }
    }

    fn forage_selection(&self) -> &ForageSelection {
        if let Some(a) = &self.state.announcing {
            a.forage.as_deref().expect("forage declaration")
        } else {
            self.state
                .resolving
                .as_ref()
                .unwrap()
                .forage
                .as_deref()
                .expect("forage continuation")
        }
    }
    fn forage_selection_mut(&mut self) -> &mut ForageSelection {
        if let Some(a) = &mut self.state.announcing {
            a.forage.as_deref_mut().unwrap()
        } else {
            self.state
                .resolving
                .as_mut()
                .unwrap()
                .forage
                .as_deref_mut()
                .unwrap()
        }
    }
    pub(super) fn announcement_reserve(
        &self,
        a: &Announcement,
    ) -> Option<(ObjectRef, AdditionalCost)> {
        if let Some(f) = &a.forage
            && f.route == Some(ForageRoute::Food)
        {
            return f.selected.first().copied().map(|r| (r, food_cost()));
        }
        a.additional.map(|r| (r, a.additional_cost().unwrap()))
    }
    fn forage_graveyard_count(&self, player: Player) -> usize {
        self.state
            .objects
            .iter()
            .filter(|o| o.zone == Zone::Graveyard && o.owner == player && !o.is_token())
            .take(3)
            .count()
    }
    pub(super) fn forage_cost_fundable(
        &self,
        goal: FundingGoal,
        spec: AdditionalCost,
        route: Option<ForageRoute>,
    ) -> bool {
        let purpose = self.funding_purpose(&goal);
        self.forage_cost_fundable_for(goal, spec, route, purpose)
    }
    pub(super) fn forage_cost_fundable_for(
        &self,
        goal: FundingGoal,
        spec: AdditionalCost,
        route: Option<ForageRoute>,
        purpose: mana_lots::Purpose,
    ) -> bool {
        if route.is_none_or(|r| r == ForageRoute::ExileThree)
            && self.forage_graveyard_count(goal.actor) >= 3
            && self.can_fund_for(goal.clone(), purpose)
        {
            return true;
        }
        if route.is_none_or(|r| r == ForageRoute::Food)
            && self.state.objects.iter().enumerate().any(|(id, _)| {
                let r = self.object_ref(id as CardId);
                self.additional_candidate(
                    food_cost(),
                    goal.actor,
                    goal.source.as_ref().map_or(r, |s| s.0),
                    r,
                ) && self.can_fund_for(
                    FundingGoal {
                        reserve: Some((r, food_cost())),
                        ..goal.clone()
                    },
                    purpose,
                )
            })
        {
            return true;
        }
        if spec.kind == EffectChoiceKind::ForageOrBlack
            && route.is_none_or(|r| r == ForageRoute::Black)
        {
            let mut black = goal;
            black.required[2] = black.required[2]
                .checked_add(1)
                .capacity("forage alternative mana cost exhausted");
            return self.can_fund_for(black, purpose);
        }
        false
    }
    fn forage_route_available(&self, route: ForageRoute) -> bool {
        let p = self.actor().unwrap();
        if let Some(a) = &self.state.announcing {
            let mut goal = self.funding_goal(a);
            // The current total already includes the black choice after declaration.
            if a.forage
                .as_ref()
                .is_some_and(|f| f.route == Some(ForageRoute::Black))
            {
                goal.required[2] -= 1;
            }
            return self.forage_cost_fundable(goal, a.additional_cost().unwrap(), Some(route));
        }
        match route {
            ForageRoute::Black => false,
            ForageRoute::ExileThree => self.forage_graveyard_count(p) >= 3,
            ForageRoute::Food => self.state.objects.iter().enumerate().any(|(id, _)| {
                self.additional_candidate(
                    food_cost(),
                    p,
                    self.object_ref(id as CardId),
                    self.object_ref(id as CardId),
                )
            }),
        }
    }
    fn forage_candidate(&self, r: ObjectRef) -> bool {
        let Some(o) = self.state.objects.get(r.card as usize) else {
            return false;
        };
        let f = self.forage_selection();
        let p = self.actor().unwrap();
        if self.object_ref(r.card) != r || f.selected.contains(&r) {
            return false;
        }
        match f.route {
            Some(ForageRoute::ExileThree) => {
                o.zone == Zone::Graveyard && o.owner == p && !o.is_token()
            }
            Some(ForageRoute::Food) => {
                self.additional_candidate(food_cost(), p, r, r)
                    && self.state.announcing.as_ref().is_none_or(|a| {
                        self.can_fund_for(FundingGoal {
                            reserve: Some((r, food_cost())),
                            ..self.funding_goal(a)
                        })
                    })
            }
            _ => false,
        }
    }
    pub(super) fn visit_forage_actions(&self, emit: &mut impl FnMut(Action) -> bool) -> bool {
        if self.state.announcing.is_none() && emit(Action::DeclineForage) {
            return true;
        }
        let method = matches!(self.state.pending, Pending::ForageMethod { .. });
        if method || self.forage_selection().route.is_none() {
            for route in [
                ForageRoute::ExileThree,
                ForageRoute::Food,
                ForageRoute::Black,
            ] {
                if self.forage_route_available(route) && emit(Action::ChooseForageRoute { route }) {
                    return true;
                }
            }
        } else {
            for id in 0..self.state.objects.len() {
                let object = self.object_ref(id as CardId);
                if self.forage_candidate(object) && emit(Action::ChooseForage { object }) {
                    return true;
                }
            }
        }
        false
    }
    pub(super) fn choose_forage_route(&mut self, route: ForageRoute) {
        if matches!(self.state.pending, Pending::ForageMethod { .. }) {
            self.state.announcing.as_mut().unwrap().forage = Some(Box::new(ForageSelection {
                route: Some(route),
                selected: Vec::new(),
                skip_to: 0,
            }));
            self.continue_announcement_choices(false);
        } else {
            self.forage_selection_mut().route = Some(route);
        }
    }
    pub(super) fn choose_forage(&mut self, object: ObjectRef) {
        self.forage_selection_mut().selected.push(object);
        if !self.forage_selection().complete() {
            return;
        }
        if self.state.announcing.is_some() {
            self.state.pending = Pending::PayCosts {
                player: self.actor().unwrap(),
            };
        } else {
            let selection = self
                .state
                .resolving
                .as_mut()
                .unwrap()
                .forage
                .take()
                .unwrap();
            self.commit_forage(self.actor().unwrap(), &selection);
            self.run_program();
        }
    }
    pub(super) fn decline_forage(&mut self) {
        let r = self.state.resolving.as_mut().unwrap();
        let selection = r.forage.take().unwrap();
        r.pc = selection.skip_to;
        self.run_program();
    }
    pub(super) fn start_resolution_forage(&mut self, skip: usize) {
        let r = self.state.resolving.as_mut().unwrap();
        let skip_to = r.pc.checked_add(skip).expect("forage program offset");
        assert!(skip_to <= r.program.len());
        let player = r.item.controller;
        r.forage = Some(Box::new(ForageSelection {
            route: None,
            selected: Vec::new(),
            skip_to,
        }));
        self.state.pending = Pending::Forage { player };
    }
    pub(super) fn commit_forage(&mut self, player: Player, selection: &ForageSelection) {
        assert!(selection.complete());
        for &r in &selection.selected {
            assert_eq!(self.object_ref(r.card), r);
            let o = &self.state.objects[r.card as usize];
            if selection.route == Some(ForageRoute::Food) {
                assert_eq!(o.controller, player);
            } else {
                assert_eq!(o.owner, player);
            }
            assert_eq!(
                o.zone,
                if selection.route == Some(ForageRoute::Food) {
                    Zone::Battlefield
                } else {
                    Zone::Graveyard
                }
            );
        }
        match selection.route.unwrap() {
            ForageRoute::Food => self.sacrifice_batch(&[(player, selection.selected[0].card)]),
            ForageRoute::ExileThree => {
                let ids: Vec<_> = selection.selected.iter().map(|r| r.card).collect();
                let events = self.move_objects_raw(&ids, Zone::Exile);
                self.finish_zone_events(&events, &[]);
            }
            ForageRoute::Black => unreachable!(),
        }
        let sources: Vec<_> = self
            .state
            .objects
            .iter()
            .enumerate()
            .filter(|(_, o)| {
                o.zone == Zone::Battlefield
                    && o.controller == player
                    && o.card
                        .triggers()
                        .iter()
                        .any(|a| a.condition.event() == TriggerCondition::Foraged)
            })
            .map(|(id, _)| self.object_view(id as CardId))
            .collect();
        for source in sources {
            for (index, a) in source.card.triggers().iter().enumerate() {
                if a.condition.event() == TriggerCondition::Foraged {
                    self.queue_trigger(&source, index);
                }
            }
        }
    }
    pub(super) fn forage_view(&self, viewer: Player) -> Option<ForageView> {
        let player = self
            .state
            .announcing
            .as_ref()
            .map(|a| a.item.controller)
            .or_else(|| self.state.resolving.as_ref().map(|r| r.item.controller))?;
        if self.actor() != Some(viewer) || viewer != player {
            return None;
        }
        let method = matches!(self.state.pending, Pending::ForageMethod { .. });
        let choosing = matches!(self.state.pending, Pending::Forage { .. });
        let selection = self
            .state
            .announcing
            .as_ref()
            .and_then(|a| a.forage.as_deref())
            .or_else(|| {
                self.state
                    .resolving
                    .as_ref()
                    .and_then(|r| r.forage.as_deref())
            });
        if !method && selection.is_none() {
            return None;
        }
        let picking_route = method || choosing && selection.is_some_and(|f| f.route.is_none());
        Some(ForageView {
            player,
            optional: self.state.announcing.is_none(),
            paid: self
                .state
                .announcing
                .as_ref()
                .is_some_and(|a| a.additional_paid),
            routes: if picking_route {
                [
                    ForageRoute::ExileThree,
                    ForageRoute::Food,
                    ForageRoute::Black,
                ]
                .into_iter()
                .filter(|&r| self.forage_route_available(r))
                .collect()
            } else {
                Vec::new()
            },
            route: selection.and_then(|f| f.route),
            selected: selection.map_or_else(Vec::new, |f| f.selected.clone()),
            candidates: if choosing && !picking_route {
                (0..self.state.objects.len())
                    .map(|id| self.object_ref(id as CardId))
                    .filter(|&r| self.forage_candidate(r))
                    .collect()
            } else {
                Vec::new()
            },
        })
    }
}
