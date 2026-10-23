//! Source-independent linked casting, simultaneous exchange, and attacking blink returns.
use super::*;
impl Game {
    pub(super) fn exchange_creature_control(&mut self, item: &SpellView) {
        let objects: Vec<_> = item
            .targets
            .iter()
            .filter(|bound| {
                self.same_target_object(**bound)
                    && self
                        .target_spec(item)
                        .group(bound.group)
                        .is_some_and(|spec| self.item_target_matches(item, spec.rule, bound.target))
            })
            .filter_map(|bound| match bound.target {
                Target::Card(id) if self.on_battlefield(id) && self.is_creature(id) => {
                    Some(self.object_ref(id))
                }
                _ => None,
            })
            .collect();
        let [left, right] = objects.as_slice() else {
            return;
        };
        let a = self.state.objects[left.card as usize].controller;
        let b = self.state.objects[right.card as usize].controller;
        if a == b || left == right {
            return;
        }
        self.ensure_control_base(*left);
        self.ensure_control_base(*right);
        let timestamp = self.next_role_timestamp();
        for (object, controller) in [(*left, b), (*right, a)] {
            self.state
                .controls
                .as_mut()
                .unwrap()
                .effects
                .push(continuous_control::ControlEffect {
                    object,
                    controller,
                    timestamp,
                    expires_turn: None,
                });
        }
        self.refresh_controllers();
    }
    pub(super) fn exile_opponent_until_nonland_linked(&mut self, item: &SpellView) {
        let owner = item.controller.other();
        while let Some(&id) = self.state.players[owner.index()].library.last() {
            let event = self.move_object(id, Zone::Exile);
            if !event.moved {
                break;
            }
            if event.destination == Zone::Exile {
                self.remember_linked_exile_for_origin(item.object, item.ability_origin(), &[id]);
                if !self.state.objects[id as usize].card.is_land() {
                    break;
                }
            }
        }
    }
    pub(super) fn start_one_linked_cast(&mut self, item: &SpellView) -> bool {
        let remaining: Vec<_> = self
            .state
            .linked_choices
            .as_ref()
            .into_iter()
            .flat_map(|data| data.links.iter())
            .filter(|link| {
                link.source == item.object
                    && link.ability_origin == item.ability_origin()
                    && self.object_ref(link.exiled.card) == link.exiled
                    && self.state.objects[link.exiled.card as usize].zone == Zone::Exile
            })
            .map(|link| link.exiled)
            .collect();
        if remaining.is_empty() {
            return false;
        }
        self.state.resolution_free_casts = Some(Box::new(ResolutionFreeCasts {
            player: item.controller,
            remaining,
            maximum_mana_value: None,
            uncast_to_hand: false,
            remaining_casts: Some(1),
        }));
        self.state.pending = Pending::ResolutionCasts {
            player: item.controller,
        };
        true
    }
    pub(super) fn blink_attacking(&mut self, item: &SpellView) {
        let Some(id) = item
            .targets
            .iter()
            .filter(|bound| self.same_target_object(**bound))
            .find_map(|bound| match bound.target {
                Target::Card(id) if self.on_battlefield(id) => Some(id),
                _ => None,
            })
        else {
            return;
        };
        let event = self.move_object(id, Zone::Exile);
        if event.moved
            && matches!(
                event.destination,
                Zone::Exile | Zone::Graveyard | Zone::Command
            )
            && !event.before.is_token()
        {
            self.after_departure(programs::ProgramContinuation::ReturnAttacking(
                ReturnLinkView {
                    object: event.after,
                    zone: event.destination,
                    controller: item.controller,
                    counter: None,
                    tapped: true,
                    additional: Vec::new(),
                },
            ));
        }
    }
    pub(super) fn stage_attacking_linked_return(&mut self, link: &ReturnLinkView) {
        let object = &self.state.objects[link.object.card as usize];
        if object.zone != link.zone
            || self.object_ref(link.object.card) != link.object
            || object.is_token()
        {
            return;
        }
        let mut movement = self.entry_move(link.object.card, Zone::Battlefield, true);
        movement.controller = link.controller;
        movement.choose_attacking_defender = true;
        self.stage_entries(vec![movement], None);
    }
}

impl Game {
    pub(super) fn capture_delayed_return_target(&mut self, item: &SpellView) {
        self.state.resolving.as_mut().unwrap().item.return_link = None;
        let Some(id) = item
            .targets
            .iter()
            .filter(|bound| self.same_target_object(**bound))
            .find_map(|bound| match bound.target {
                Target::Card(id) if self.on_battlefield(id) => Some(id),
                _ => None,
            })
        else {
            return;
        };
        let owner = self.state.objects[id as usize].owner;
        let event = self.move_object(id, Zone::Exile);
        if event.moved
            && !event.before.is_token()
            && matches!(
                event.destination,
                Zone::Exile | Zone::Graveyard | Zone::Command
            )
        {
            self.state.resolving.as_mut().unwrap().item.return_link =
                Some(Box::new(ReturnLinkView {
                    object: event.after,
                    zone: event.destination,
                    controller: owner,
                    counter: None,
                    tapped: false,
                    additional: Vec::new(),
                }));
        }
    }
}
