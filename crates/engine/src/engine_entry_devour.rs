//! Owned, non-targeting Devour choices and one simultaneous sacrifice event.
use super::*;
use ability_world::CharacteristicWorld;

#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub(super) struct Preparation {
    pub(super) selected: Vec<ObjectRef>,
    pub(super) closed: bool,
    sacrificed: Option<u64>,
}

#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
impl Preparation {
    pub(super) fn bytes(&self) -> usize {
        std::mem::size_of::<Self>() + self.selected.len() * std::mem::size_of::<ObjectRef>()
    }
}

#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct EntryDevourChoiceView {
    pub entrant: ObjectRef,
    pub selected: Vec<ObjectRef>,
    pub candidates: Vec<ObjectRef>,
}

#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
pub(super) enum Advance {
    Choosing,
    NestedEntry,
    Ready,
}

#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
pub(super) fn needs_devour(movement: &entries::EntryMove) -> bool {
    movement.destination == Zone::Battlefield
        && movement.before.zone != Zone::Battlefield
        && !movement.skipped
        && movement.manifest_original.is_none()
        && movement
            .intrinsic_entry_card()
            .is_some_and(|card| card.devour().is_some())
        && movement
            .devour_preparation
            .as_ref()
            .is_none_or(|prep| prep.sacrificed.is_none())
}

#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
pub(super) fn selection_index(moves: &[entries::EntryMove], active: Player) -> Option<usize> {
    moves
        .iter()
        .enumerate()
        .filter(|(_, movement)| {
            needs_devour(movement)
                && movement
                    .devour_preparation
                    .as_ref()
                    .is_none_or(|prep| !prep.closed)
        })
        .min_by_key(|(index, movement)| (movement.controller != active, *index))
        .map(|(index, _)| index)
}

#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
pub(super) fn is_candidate(
    world: &impl CharacteristicWorld,
    moves: &[entries::EntryMove],
    index: usize,
    id: CardId,
) -> bool {
    let object = world.object(id);
    let movement = &moves[index];
    if object.zone != Zone::Battlefield || object.controller != movement.controller {
        return false;
    }
    let reference = world.reference(id);
    let spec = movement.before.card.devour().expect("Devour selection");
    world.kinds(id) & spec.kinds != 0
        && !world.sacrifice_prohibited(id)
        && !moves.iter().any(|other| {
            (!other.skipped && other.before.reference == reference)
                || other
                    .devour_preparation
                    .as_ref()
                    .is_some_and(|prep| prep.selected.contains(&reference))
        })
}

#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
pub(super) fn selections(
    moves: &[entries::EntryMove],
) -> impl Iterator<Item = (Player, ObjectRef)> + '_ {
    moves
        .iter()
        .filter(|movement| needs_devour(movement))
        .flat_map(|movement| {
            movement
                .devour_preparation
                .as_ref()
                .expect("Devour selections are closed")
                .selected
                .iter()
                .copied()
                .map(|object| (movement.controller, object))
        })
}

#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
pub(super) fn record_results(
    moves: &mut [entries::EntryMove],
    was_sacrificed: impl Fn(ObjectRef) -> bool,
) {
    for movement in moves.iter_mut().filter(|movement| needs_devour(movement)) {
        let prep = movement.devour_preparation.as_mut().unwrap();
        let count = prep
            .selected
            .iter()
            .filter(|&&object| was_sacrificed(object))
            .count();
        prep.sacrificed = Some(u64::try_from(count).capacity("devour sacrifice count exhausted"));
    }
}

#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
pub(super) fn entry_counters(movement: &entries::EntryMove) -> u64 {
    let Some(count) = movement
        .devour_preparation
        .as_ref()
        .and_then(|prep| prep.sacrificed)
    else {
        return 0;
    };
    let spec = movement.before.card.devour().expect("Devour result");
    count
        .checked_mul(u64::from(spec.counters_per_permanent))
        .capacity("devour counter amount exhausted")
}

#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
impl Game {
    pub(super) fn current_entry_devour_index(&self) -> Option<usize> {
        selection_index(&self.state.entry_batch.as_ref()?.moves, self.state.active)
    }

    pub(super) fn entry_devour_candidates(&self) -> Vec<ObjectRef> {
        let Some(index) = self.current_entry_devour_index() else {
            return Vec::new();
        };
        let moves = &self.state.entry_batch.as_ref().unwrap().moves;
        (0..self.state.objects.len())
            .map(|id| id as CardId)
            .filter(|&id| is_candidate(self, moves, index, id))
            .map(|id| self.object_ref(id))
            .collect()
    }

    pub(super) fn choose_entry_devour(&mut self, object: ObjectRef) {
        let index = self.current_entry_devour_index().unwrap();
        self.state.entry_batch.as_mut().unwrap().moves[index]
            .devour_preparation
            .get_or_insert_default()
            .selected
            .push(object);
    }

    pub(super) fn finish_entry_devour(&mut self) {
        let index = self.current_entry_devour_index().unwrap();
        self.state.entry_batch.as_mut().unwrap().moves[index]
            .devour_preparation
            .get_or_insert_default()
            .closed = true;
    }

    pub(super) fn advance_entry_devour(&mut self) -> Advance {
        while let Some(index) = self.current_entry_devour_index() {
            let movement = &self.state.entry_batch.as_ref().unwrap().moves[index];
            if !self.entry_devour_candidates().is_empty()
                && self
                    .ghost_entry_preview(&movement.replacement_preview())
                    .is_some_and(|view| view.printed_abilities_available())
            {
                self.state.pending = Pending::EntryDevour {
                    player: movement.controller,
                };
                return Advance::Choosing;
            }
            self.finish_entry_devour();
        }
        if !self
            .state
            .entry_batch
            .as_ref()
            .unwrap()
            .moves
            .iter()
            .any(needs_devour)
        {
            return Advance::Ready;
        }

        // No state changes during selection. Freeze and perform all selected
        // sacrifices together. Their zone events can immediately return linked
        // cards, so suspend this still-uncommitted batch during that entry.
        let mut batch = self.state.entry_batch.take().unwrap();
        let sacrifices = selections(&batch.moves)
            .map(|(controller, object)| (controller, object.card))
            .collect::<Vec<_>>();
        if !sacrifices.is_empty() {
            assert!(
                self.sacrifice_cost_batch_outcome(&sacrifices).is_some(),
                "valid Devour selections"
            );
        }
        record_results(&mut batch.moves, |object| {
            self.object_ref(object.card) != object
        });
        let nested = self.state.entry_batch.is_some();
        self.resume_entry_after_nested(batch);
        if nested {
            Advance::NestedEntry
        } else {
            Advance::Ready
        }
    }

    pub(super) fn resume_entry_after_nested(&mut self, batch: Box<entries::EntryBatch>) {
        let mut next = &mut self.state.entry_batch;
        while let Some(nested) = next {
            next = &mut nested.resume_entry;
        }
        *next = Some(batch);
    }

    pub(super) fn entry_devour_view(&self, viewer: Player) -> Option<EntryDevourChoiceView> {
        if !matches!(self.state.pending, Pending::EntryDevour { .. }) {
            return None;
        }
        let movement =
            &self.state.entry_batch.as_ref()?.moves[self.current_entry_devour_index()?];
        if !entries::visible_mask(
            movement,
            self.information_mask(viewer),
            &self.state.known_hand,
        ) {
            return None;
        }
        Some(EntryDevourChoiceView {
            entrant: movement.before.reference,
            selected: movement
                .devour_preparation
                .as_ref()
                .map_or_else(Vec::new, |prep| prep.selected.clone()),
            candidates: self.entry_devour_candidates(),
        })
    }
}
