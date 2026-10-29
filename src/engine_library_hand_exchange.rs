//! Exact exile results and APNAP private hand selections with simultaneous entry.
use super::*;
#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) struct HandDeployment {
    players: [Player; 2],
    next: usize,
    selected: [Vec<ObjectRef>; 2],
    counts: [Option<usize>; 2],
}
#[derive(Clone, Debug, Serialize)]
pub struct HandDeploymentView {
    pub actor: Option<Player>,
    pub selected_counts: [Option<usize>; 2],
}
impl HandDeployment {
    pub(super) fn bytes(&self) -> usize {
        std::mem::size_of::<Self>()
            + self.selected.iter().map(Vec::len).sum::<usize>() * std::mem::size_of::<ObjectRef>()
    }
}
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
                    && !self.state.objects[r.card as usize].is_token()
            })
            .map(|r| self.entry_move(r.card, Zone::Hand, false))
            .collect();
        self.stage_entries(moves, None);
    }
    pub(super) fn hand_deployment(&self) -> Option<&HandDeployment> {
        self.state.linked_choices.as_ref()?.deployment.as_deref()
    }
    pub(super) fn start_hand_deployment(&mut self) {
        let player = self.state.active;
        self.state.linked_choices.get_or_insert_default().deployment =
            Some(Box::new(HandDeployment {
                players: [player, player.other()],
                next: 0,
                selected: std::array::from_fn(|_| Vec::new()),
                counts: [None; 2],
            }));
        self.state.pending = Pending::HandCardChoice { player };
    }
    fn deployment_candidate(&self, player: Player, object: ObjectRef) -> bool {
        let Some(o) = self.state.objects.get(object.card as usize) else {
            return false;
        };
        self.object_ref(object.card) == object
            && o.zone == Zone::Hand
            && o.owner == player
            && !o.is_token()
            && o.card.is_creature()
            && !self.hand_deployment().unwrap().selected[player.index()].contains(&object)
    }
    pub(super) fn visit_hand_deployment(&self, emit: &mut impl FnMut(Action) -> bool) -> bool {
        let d = self.hand_deployment().unwrap();
        let player = d.players[d.next];
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
            actor: d.players.get(d.next).copied(),
            selected_counts: d.counts,
        })
    }
    pub(super) fn hand_deployment_view(&self, viewer: Player) -> Option<HandChoiceView> {
        let d = self.hand_deployment()?;
        let actor = d.players[d.next];
        if !self.shares_information(viewer, actor) {
            return None;
        }
        Some(HandChoiceView {
            kind: HandChoiceKind::DeployCreatures,
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
        if d.next < 2 {
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
        let selected = std::mem::take(&mut d.selected);
        let moves = players
            .into_iter()
            .flat_map(|p| selected[p.index()].iter().copied())
            .filter(|r| {
                self.object_ref(r.card) == *r
                    && self.state.objects[r.card as usize].zone == Zone::Hand
            })
            .map(|r| {
                let mut m = self.entry_move(r.card, Zone::Battlefield, false);
                m.controller = m.before.owner;
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
        if item.source.is_some_and(|source| self.on_battlefield(source.id) && self.object_ref(source.id) == source.object)
        {
            let skip_to = self.state.resolving.as_ref().unwrap().pc;
            self.state.pending = Pending::EffectOptional {
                player: item.controller,
                skip_to,
            };
            true
        } else {
            self.grant_exiled_result_cast_any_type(item.controller);
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
            self.grant_exiled_result_cast_any_type(item.controller);
        }
    }
    fn grant_exiled_result_cast_any_type(&mut self, player: Player) {
        let objects = std::mem::take(&mut self.state.resolving.as_mut().unwrap().milled);
        for object in objects {
            if self.object_ref(object.card) != object
                || self.state.objects[object.card as usize].zone != Zone::Exile
            {
                continue;
            }
            self.grant_exile_play_permission(player, object, false);
            // Modify only the newly inserted permission; unrelated grants retain their routes.
            let row = self
                .state
                .exile_permissions
                .iter_mut()
                .find(|p| {
                    p.object == object && p.player == player && !p.spend_any_mana && !p.cast_only
                })
                .unwrap();
            row.spend_any_mana = true;
            row.cast_only = true;
        }
    }
}
