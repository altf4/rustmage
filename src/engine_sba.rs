//! Frozen state-based events and snapshot-owned legend choices (CR704/101.4).
use super::*;

#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) struct LegendGroup {
    pub controller: Player,
    pub name: &'static str,
    pub objects: Vec<ObjectRef>,
    pub keep: Option<ObjectRef>,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) struct SbaBatch {
    resume: Pending,
    deaths: Vec<CardId>,
    ceased: Vec<CardId>,
    detached: Vec<ObjectRef>,
    pairs: Vec<(ObjectRef, u64)>,
    lost: [bool; 2],
    pub groups: Vec<LegendGroup>,
    pub roles: Vec<RoleGroupView>,
}
impl SbaBatch {
    pub fn bytes(&self) -> usize {
        std::mem::size_of::<Self>()
            + (self.deaths.len() + self.ceased.len()) * std::mem::size_of::<CardId>()
            + self.pairs.len() * std::mem::size_of::<(ObjectRef, u64)>()
            + self.detached.len() * std::mem::size_of::<ObjectRef>()
            + self.roles.len() * std::mem::size_of::<RoleGroupView>()
            + self.roles.iter().map(|g| g.objects.len()).sum::<usize>()
                * std::mem::size_of::<ObjectRef>()
            + self.groups.len() * std::mem::size_of::<LegendGroup>()
            + self.groups.iter().map(|g| g.objects.len()).sum::<usize>()
                * std::mem::size_of::<ObjectRef>()
    }
}
impl Game {
    fn collect_sba_batch(&self) -> SbaBatch {
        let mut batch = SbaBatch {
            resume: self.state.pending.clone(),
            deaths: Vec::new(),
            ceased: Vec::new(),
            detached: Vec::new(),
            pairs: Vec::new(),
            groups: Vec::new(),
            roles: Vec::new(),
            lost: self
                .state
                .players
                .each_ref()
                .map(|p| p.life <= 0 || p.poison_counters >= 10 || p.attempted_empty_draw),
        };
        let mut legends: [std::collections::BTreeMap<&'static str, Vec<ObjectRef>>; 2] =
            std::array::from_fn(|_| std::collections::BTreeMap::new());
        for (index, object) in self.state.objects.iter().enumerate() {
            let id = index as CardId;
            if object.card.is_token() && !matches!(object.zone, Zone::Battlefield | Zone::Outside) {
                batch.ceased.push(id);
            }
            if object.zone != Zone::Battlefield {
                continue;
            }
            if self.saga_sacrifice_required(id) {
                batch.deaths.push(id);
            }
            if object.card.is_planeswalker() && object.counters.loyalty == 0 {
                batch.deaths.push(id);
            }
            if object.card.aura().is_some() {
                if !self.printed_abilities_available(id)
                    || !object.attached_to.is_some_and(|target| {
                        self.aura_can_enchant(object.card, object.controller, target)
                    })
                {
                    batch.deaths.push(id);
                }
            } else if let Some(target) = object.attached_to
                && (self.is_creature(id)
                    || target == self.object_ref(id)
                    || !self.on_battlefield(target.card)
                    || self.object_ref(target.card) != target
                    || !self.is_creature(target.card))
            {
                batch.detached.push(self.object_ref(id));
            }
            if object.card.is_legendary() {
                legends[object.controller.index()]
                    .entry(self.object_name(id))
                    .or_default()
                    .push(self.object_ref(id));
            }
            if self.is_creature(id) {
                let toughness = self.stats(id).toughness;
                if toughness <= 0
                    || (!self.keywords(id).contains(Keywords::INDESTRUCTIBLE)
                        && (object.damage >= toughness || object.deathtouch_since_sba))
                {
                    batch.deaths.push(id);
                }
            }
            let pairs = object.counters.plus_one.min(object.counters.minus_one);
            if pairs > 0 {
                batch.pairs.push((self.object_ref(id), pairs));
            }
        }
        for controller in [self.state.active, self.state.active.other()] {
            let groups = std::mem::take(&mut legends[controller.index()]);
            for (name, objects) in groups {
                if objects.len() > 1 {
                    batch.groups.push(LegendGroup {
                        controller,
                        name,
                        objects,
                        keep: None,
                    });
                }
            }
        }
        batch.roles = self.collect_role_choices(&mut batch.deaths);
        batch
    }

    pub(super) fn sba_required(&self) -> bool {
        let b = self.collect_sba_batch();
        !b.deaths.is_empty()
            || !b.detached.is_empty()
            || !b.ceased.is_empty()
            || !b.pairs.is_empty()
            || !b.groups.is_empty()
            || !b.roles.is_empty()
            || b.lost.iter().any(|&lost| lost)
    }
    pub(super) fn state_based_actions(&mut self) {
        if self.state.sba_batch.is_some()
            || self.state.outcome.is_some()
            || self.state.entry_batch.is_some()
        {
            return;
        }
        loop {
            let batch = self.collect_sba_batch();
            let changed = !batch.deaths.is_empty()
                || !batch.detached.is_empty()
                || !batch.ceased.is_empty()
                || !batch.pairs.is_empty()
                || !batch.groups.is_empty()
                || !batch.roles.is_empty()
                || batch.lost.iter().any(|&lost| lost);
            // The check consumes this interval's deathtouch history, even when
            // indestructible prevents destruction. New entrants start clear.
            // Preserve shared hot rows when there is no history to consume.
            if self
                .state
                .objects
                .iter()
                .any(|object| object.deathtouch_since_sba)
            {
                for object in &mut self.state.objects {
                    object.deathtouch_since_sba = false;
                }
            }
            if !changed {
                return;
            }
            if !batch.groups.is_empty() || !batch.roles.is_empty() {
                self.state.sba_batch = Some(Box::new(batch));
                self.advance_sba_choices();
                return;
            }
            self.commit_sba_batch(batch);
            if self.state.outcome.is_some() || self.state.entry_batch.is_some() {
                return;
            }
        }
    }

    pub(super) fn keep_legend(&mut self, object: ObjectRef) {
        let batch = self.state.sba_batch.as_mut().unwrap();
        let group = batch
            .groups
            .iter_mut()
            .find(|group| group.keep.is_none() && group.objects.contains(&object))
            .unwrap();
        group.keep = Some(object);
        self.advance_sba_choices();
    }

    pub(super) fn advance_sba_choices(&mut self) {
        let batch = self.state.sba_batch.as_ref().unwrap();
        for player in [self.state.active, self.state.active.other()] {
            if batch
                .groups
                .iter()
                .any(|g| g.controller == player && g.keep.is_none())
            {
                self.state.pending = Pending::Legend { player };
                return;
            }
            if batch
                .roles
                .iter()
                .any(|g| g.controller == player && g.keep.is_none())
            {
                self.state.pending = Pending::Role { player };
                return;
            }
        }
        let batch = *self.state.sba_batch.take().unwrap();
        self.commit_sba_batch(batch);
    }

    fn commit_sba_batch(&mut self, mut batch: SbaBatch) {
        for group in &batch.groups {
            let keep = group.keep.expect("every legend choice precedes the event");
            batch.deaths.extend(
                group
                    .objects
                    .iter()
                    .filter(|&&object| object != keep)
                    .map(|object| object.card),
            );
        }
        for group in &batch.roles {
            let keep = group
                .keep
                .expect("every Role choice precedes the simultaneous event");
            batch
                .deaths
                .extend(group.objects.iter().filter(|&&r| r != keep).map(|r| r.card));
        }
        batch.deaths.sort_unstable();
        batch.deaths.dedup();
        let sources = if batch.deaths.is_empty() {
            Vec::new()
        } else {
            self.death_trigger_sources()
        };
        let sacrifices: Vec<_> = batch
            .deaths
            .iter()
            .filter(|&&id| self.saga_sacrifice_required(id))
            .map(|&id| (self.state.objects[id as usize].controller, id))
            .collect();
        self.food_sacrifice_triggers(&sacrifices, &sources);
        let mut events = self.move_objects_raw(&batch.deaths, Zone::Graveyard);
        for event in &mut events {
            if let Some(&(actor, _)) = sacrifices.iter().find(|&&(_, id)| id == event.before.id) {
                event.sacrificed_by = Some(actor);
            }
        }
        for source in batch.detached {
            if self.object_ref(source.card) == source {
                self.state.objects[source.card as usize].attached_to = None;
            }
        }
        // Cancellation belongs to the frozen event, never a newly returned object.
        for (object, pairs) in batch.pairs {
            if self.object_ref(object.card) == object {
                let counters = &mut self.state.objects[object.card as usize].counters;
                counters.plus_one -= pairs;
                counters.minus_one -= pairs;
            }
        }
        for id in batch.ceased {
            self.cease_token(id);
        }
        self.finish_zone_events(&events, &sources);
        self.state.pending = batch.resume;
        self.state.outcome = match batch.lost {
            [true, true] => Some(Outcome::Draw),
            [true, false] => Some(Outcome::Winner(Player::P1)),
            [false, true] => Some(Outcome::Winner(Player::P0)),
            _ => None,
        };
        if self.state.outcome.is_some() {
            self.state.pending = Pending::Terminal;
        }
    }
}
