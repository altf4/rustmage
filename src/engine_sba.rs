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
}
impl SbaBatch {
    pub fn bytes(&self) -> usize {
        std::mem::size_of::<Self>()
            + (self.deaths.len() + self.ceased.len()) * std::mem::size_of::<CardId>()
            + self.pairs.len() * std::mem::size_of::<(ObjectRef, u64)>()
            + self.detached.len() * std::mem::size_of::<ObjectRef>()
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
            lost: self
                .state
                .players
                .each_ref()
                .map(|p| p.life <= 0 || p.attempted_empty_draw),
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
            if object.card.aura().is_some() {
                if !object
                    .attached_to
                    .is_some_and(|target| self.aura_can_enchant(object.card, object.owner, target))
                {
                    batch.deaths.push(id);
                }
            } else if let Some(target) = object.attached_to
                && (object.card.is_creature()
                    || target == self.object_ref(id)
                    || !self.on_battlefield(target.card)
                    || self.object_ref(target.card) != target
                    || !self.state.objects[target.card as usize].card.is_creature())
            {
                batch.detached.push(self.object_ref(id));
            }
            if object.card.is_legendary() {
                legends[object.owner.index()]
                    .entry(object.card.name())
                    .or_default()
                    .push(self.object_ref(id));
            }
            if object.card.is_creature() {
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
        batch
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
                || batch.lost.iter().any(|&lost| lost);
            // The check consumes this interval's deathtouch history, even when
            // indestructible prevents destruction. New entrants start clear.
            for object in &mut self.state.objects {
                object.deathtouch_since_sba = false;
            }
            if !changed {
                return;
            }
            let choice = batch.groups.first().map(|group| group.controller);
            if let Some(player) = choice {
                self.state.sba_batch = Some(Box::new(batch));
                self.state.pending = Pending::Legend { player };
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
        if let Some(group) = batch.groups.iter().find(|group| group.keep.is_none()) {
            self.state.pending = Pending::Legend {
                player: group.controller,
            };
        } else {
            let batch = *self.state.sba_batch.take().unwrap();
            self.commit_sba_batch(batch);
        }
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
        batch.deaths.sort_unstable();
        batch.deaths.dedup();
        let sources = if batch.deaths.is_empty() {
            Vec::new()
        } else {
            self.death_trigger_sources()
        };
        let events = self.move_objects_raw(&batch.deaths, Zone::Graveyard);
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
