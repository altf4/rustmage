//! Exact exile results and APNAP private hand selections with simultaneous entry.
use super::*;
#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum HandDeploymentMode {
    CreaturesEach,
    ControllerPermanents,
}
#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) struct HandDeployment {
    players: [Player; 2],
    mode: HandDeploymentMode,
    next: usize,
    selected: [Vec<ObjectRef>; 2],
    counts: [Option<usize>; 2],
}
#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
#[derive(Clone, Debug, Serialize)]
pub struct HandDeploymentView {
    pub actor: Option<Player>,
    pub selected_counts: [Option<usize>; 2],
}
#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
impl HandDeployment {
    fn player_count(&self) -> usize {
        match self.mode {
            HandDeploymentMode::CreaturesEach => 2,
            HandDeploymentMode::ControllerPermanents => 1,
        }
    }
    fn actor(&self) -> Option<Player> {
        (self.next < self.player_count()).then(|| self.players[self.next])
    }
    pub(super) fn bytes(&self) -> usize {
        std::mem::size_of::<Self>()
            + self.selected.iter().map(Vec::len).sum::<usize>() * std::mem::size_of::<ObjectRef>()
    }
}
#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
impl Game {
    pub(super) fn exile_all_creatures_remember(&mut self) {
        let ids = self
            .state
            .objects
            .iter()
            .enumerate()
            .filter(|(i, o)| o.zone == Zone::Battlefield && self.is_creature(*i as CardId))
            .map(|(i, _)| i as CardId)
            .collect::<Vec<_>>();
        let before = self.death_trigger_sources();
        let events = self.move_objects_raw(&ids, Zone::Exile);
        self.state.resolving.as_mut().unwrap().milled = events
            .iter()
            .filter(|e| e.moved && e.destination == Zone::Exile && !e.before.is_token())
            .map(|e| e.after)
            .collect();
        self.finish_zone_events(&events, &before);
    }
    pub(super) fn exile_all_creatures_count(&mut self) {
        let ids = self
            .state
            .objects
            .iter()
            .enumerate()
            .filter(|(i, o)| o.zone == Zone::Battlefield && self.is_creature(*i as CardId))
            .map(|(i, _)| i as CardId)
            .collect::<Vec<_>>();
        let before = self.death_trigger_sources();
        let events = self.move_objects_raw(&ids, Zone::Exile);
        let count = events
            .iter()
            .filter(|event| event.moved && event.destination == Zone::Exile)
            .count();
        self.state.resolving.as_mut().unwrap().moved_count =
            Some(u64::try_from(count).capacity("exiled creature count exhausted"));
        self.finish_zone_events(&events, &before);
    }
    pub(super) fn return_exiled_result_to_hands(&mut self) {
        let objects = std::mem::take(&mut self.state.resolving.as_mut().unwrap().milled);
        let moves = objects
            .into_iter()
            .filter(|r| {
                self.object_ref(r.card) == *r
                    && self.state.objects[r.card as usize].zone == Zone::Exile
                    && self.state.objects[r.card as usize].is_card()
            })
            .map(|r| self.entry_move(r.card, Zone::Hand, false))
            .collect();
        self.stage_entries(moves, None);
    }
    pub(super) fn hand_deployment(&self) -> Option<&HandDeployment> {
        self.state.linked_choices.as_ref()?.deployment.as_deref()
    }
    pub(super) fn hand_deployment_active(&self) -> bool {
        self.hand_deployment()
            .is_some_and(|deployment| deployment.actor().is_some())
    }
    pub(super) fn start_hand_deployment(&mut self) {
        self.start_hand_deployment_mode(self.state.active, HandDeploymentMode::CreaturesEach);
    }
    pub(super) fn start_controller_hand_permanent_deployment(&mut self, item: &SpellView) {
        self.start_hand_deployment_mode(item.controller, HandDeploymentMode::ControllerPermanents);
    }
    fn start_hand_deployment_mode(&mut self, player: Player, mode: HandDeploymentMode) {
        self.state.linked_choices.get_or_insert_default().deployment =
            Some(Box::new(HandDeployment {
                players: [player, player.other()],
                mode,
                next: 0,
                selected: std::array::from_fn(|_| Vec::new()),
                counts: [None; 2],
            }));
        self.state.pending = Pending::HandCardChoice { player };
    }
    fn deployment_object_eligible(
        &self,
        player: Player,
        object: ObjectRef,
        mode: HandDeploymentMode,
    ) -> bool {
        let Some(o) = self.state.objects.get(object.card as usize) else {
            return false;
        };
        self.object_ref(object.card) == object
            && o.zone == Zone::Hand
            && o.owner == player
            && o.is_card()
            && match mode {
                HandDeploymentMode::CreaturesEach => o.card.is_creature(),
                HandDeploymentMode::ControllerPermanents => PermanentType::ALL
                    .into_iter()
                    .any(|kind| o.card.kind_mask() & kind.mask() != 0),
            }
    }
    fn deployment_candidate(&self, player: Player, object: ObjectRef) -> bool {
        let deployment = self.hand_deployment().unwrap();
        self.deployment_object_eligible(player, object, deployment.mode)
            && !deployment.selected[player.index()].contains(&object)
    }
    pub(super) fn visit_hand_deployment(&self, emit: &mut impl FnMut(Action) -> bool) -> bool {
        let d = self.hand_deployment().unwrap();
        let Some(player) = d.actor() else {
            return false;
        };
        if emit(Action::ChooseHandCard { object: None }) {
            return true;
        }
        self.state.objects.iter().enumerate().any(|(i, _)| {
            let object = self.object_ref(i as CardId);
            self.deployment_candidate(player, object)
                && emit(Action::ChooseHandCard {
                    object: Some(object),
                })
        })
    }
    pub(super) fn public_hand_deployment_view(&self) -> Option<HandDeploymentView> {
        let d = self.hand_deployment()?;
        Some(HandDeploymentView {
            actor: d.actor(),
            selected_counts: d.counts,
        })
    }
    pub(super) fn hand_deployment_view(&self, viewer: Player) -> Option<HandChoiceView> {
        let d = self.hand_deployment()?;
        let actor = d.actor()?;
        if !self.shares_information(viewer, actor) {
            return None;
        }
        Some(HandChoiceView {
            kind: match d.mode {
                HandDeploymentMode::CreaturesEach => HandChoiceKind::DeployCreatures,
                HandDeploymentMode::ControllerPermanents => HandChoiceKind::DeployPermanents,
            },
            inspected_cards: None,
            actor,
            victim: Some(actor),
            required: 0,
            selected: d.selected[actor.index()].clone(),
            candidates: self
                .state
                .objects
                .iter()
                .enumerate()
                .filter_map(|(i, _)| {
                    let r = self.object_ref(i as CardId);
                    self.deployment_candidate(actor, r).then_some(r)
                })
                .collect(),
            optional: true,
        })
    }
    pub(super) fn choose_hand_deployment(&mut self, object: Option<ObjectRef>) {
        let d = self
            .state
            .linked_choices
            .as_mut()
            .unwrap()
            .deployment
            .as_mut()
            .unwrap();
        let player = d.players[d.next];
        if let Some(object) = object {
            d.selected[player.index()].push(object);
            return;
        }
        d.counts[player.index()] = Some(d.selected[player.index()].len());
        d.next += 1;
        if d.next < d.player_count() {
            self.state.pending = Pending::HandCardChoice {
                player: d.players[d.next],
            };
            return;
        }
        let d = self
            .state
            .linked_choices
            .as_mut()
            .unwrap()
            .deployment
            .as_mut()
            .unwrap();
        let players = d.players;
        let player_count = d.player_count();
        let mode = d.mode;
        let selected = std::mem::take(&mut d.selected);
        let moves = players
            .into_iter()
            .take(player_count)
            .flat_map(|player| {
                selected[player.index()]
                    .iter()
                    .copied()
                    .map(move |r| (player, r))
            })
            .filter(|(player, r)| self.deployment_object_eligible(*player, *r, mode))
            .map(|(player, r)| {
                let mut m = self.entry_move(r.card, Zone::Battlefield, false);
                m.controller = player;
                m
            })
            .collect();
        self.stage_entries(moves, None);
        self.run_program();
    }
    pub(super) fn exile_damaged_library_until_nonland(&mut self, item: &SpellView) {
        let player = item.damage_event.expect("damaged player packet").player;
        let ids = self.state.players[player.index()]
            .library
            .iter()
            .rev()
            .copied()
            .filter(|&id| self.state.objects[id as usize].is_card())
            .collect::<Vec<_>>();
        self.state.resolving.as_mut().unwrap().milled.clear();
        for id in ids {
            let events = self.move_objects_raw(&[id], Zone::Exile);
            let nonland = events
                .iter()
                .find(|e| e.moved && e.destination == Zone::Exile && !e.before.card.is_land())
                .map(|e| e.after);
            self.finish_zone_events(&events, &[]);
            if let Some(object) = nonland {
                self.state.resolving.as_mut().unwrap().milled.push(object);
                break;
            }
        }
    }
    pub(super) fn offer_source_counter_or_cast(&mut self, item: &SpellView) -> bool {
        if item.source.is_some_and(|source| {
            self.on_battlefield(source.id)
                && self.object_ref(source.id) == source.object
                && !replacements::counters_prohibited(self, source.object)
        }) {
            let skip_to = self.state.resolving.as_ref().unwrap().pc;
            self.state.pending = Pending::EffectOptional {
                player: item.controller,
                skip_to,
            };
            true
        } else {
            self.grant_exiled_result_flexible_play(item.controller, true);
            false
        }
    }
    pub(super) fn complete_optional_source_counter_cast(&mut self, accept: bool) {
        let r = self.state.resolving.as_ref().unwrap();
        if r.program.get(r.pc.saturating_sub(1))
            != Some(&Instruction::MayCounterSourceOtherwiseCast)
        {
            return;
        }
        let item = r.item.clone();
        if accept {
            self.state.resolving.as_mut().unwrap().milled.clear();
            if let Some(source) = item.source
                && self.on_battlefield(source.id)
                && self.object_ref(source.id) == source.object
            {
                self.effect_counter_by(
                    source.id,
                    CounterKind::PlusOne,
                    1,
                    item.controller,
                    replacements::ResumeMode::Program,
                );
            }
        } else {
            self.grant_exiled_result_flexible_play(item.controller, true);
        }
    }
    pub(super) fn grant_exiled_result_flexible_play(&mut self, player: Player, cast_only: bool) {
        let objects = std::mem::take(&mut self.state.resolving.as_mut().unwrap().milled);
        for object in objects {
            self.grant_flexible_exile_permission_this_turn(player, object, cast_only);
        }
    }
}
