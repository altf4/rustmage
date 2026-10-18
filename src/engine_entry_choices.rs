//! Mandatory as-entry values, owned by exact incarnation and shared by search branches.
use super::ability_world::CharacteristicWorld;
use super::*;
use crate::cards::{ChoiceColor, ChosenModifier};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) struct ChoiceRow {
    pub object: ObjectRef,
    pub value: EntryChoiceValue,
}
pub(super) type Rows = Option<SharedVec<ChoiceRow>>;
#[derive(Clone, Debug, Serialize)]
pub struct EntryChoiceView {
    pub object: ObjectRef,
    pub card: Card,
    pub kind: EntryChoiceKind,
    pub selected: Vec<(ObjectRef, EntryChoiceValue)>,
}
pub(super) fn owned_bytes(rows: &Rows) -> usize {
    rows.as_ref().map_or(0, |r| {
        std::mem::size_of::<Vec<ChoiceRow>>() + r.len() * std::mem::size_of::<ChoiceRow>()
    })
}
impl Game {
    pub(super) fn entry_value(&self, object: ObjectRef) -> Option<EntryChoiceValue> {
        self.state
            .entry_values
            .as_ref()?
            .iter()
            .find(|row| row.object == object)
            .map(|row| row.value)
    }
    pub(super) fn set_entry_value(&mut self, object: ObjectRef, value: EntryChoiceValue) {
        self.clear_entry_value(object);
        self.state
            .entry_values
            .get_or_insert_with(Default::default)
            .push(ChoiceRow { object, value });
    }
    pub(super) fn clear_entry_value(&mut self, object: ObjectRef) {
        // Do not detach a shared buffer for unrelated ordinary zone moves.
        if !self
            .state
            .entry_values
            .as_ref()
            .is_some_and(|r| r.iter().any(|r| r.object == object))
        {
            return;
        }
        let rows = self.state.entry_values.as_mut().unwrap();
        rows.retain(|row| row.object != object);
        if rows.is_empty() {
            self.state.entry_values = None;
        }
    }
    pub(super) fn visit_entry_values(
        &self,
        mut emit: impl FnMut(EntryChoiceValue) -> bool,
    ) -> bool {
        let batch = self.state.entry_batch.as_ref().unwrap();
        let movement = &batch.moves[batch.choices[batch.next]];
        match movement.before.card.entry_choice().unwrap() {
            EntryChoiceKind::Color => ChoiceColor::ALL
                .into_iter()
                .any(|c| emit(EntryChoiceValue::Color(c))),
            EntryChoiceKind::CreatureType => crate::CREATURE_TYPES.iter().any(|name| {
                emit(EntryChoiceValue::CreatureType(
                    crate::cards::CreatureType::from_name(name).unwrap(),
                ))
            }),
        }
    }
    pub(super) fn choose_entry_value(&mut self, value: EntryChoiceValue) {
        let batch = self.state.entry_batch.as_ref().unwrap();
        let index = batch.choices[batch.next];
        let movement = &batch.moves[index];
        assert!(movement.before.card.entry_choice().unwrap().matches(value));
        let counter = movement.before.card.chosen_entry_counters();
        let count = if let EntryChoiceValue::CreatureType(typ) = value {
            self.state
                .objects
                .iter()
                .enumerate()
                .filter(|(id, o)| {
                    o.zone == Zone::Battlefield
                        && o.controller == movement.controller
                        && self.current_kinds(*id as CardId) & 1 != 0
                        && self.has_subtype(*id as CardId, typ.name())
                })
                .count() as u64
        } else {
            0
        };
        let batch = self.state.entry_batch.as_mut().unwrap();
        batch.moves[index].choice = Some(value);
        if let Some(kind) = counter {
            batch.moves[index].counters.add(kind, count);
            if count != 0 && !batch.counter_order.contains(&index) {
                batch.counter_order.push(index);
                batch
                    .counter_order
                    .sort_by_key(|&i| (batch.moves[i].controller != self.state.active, i));
            }
        }
    }
    pub(super) fn entry_choice_view(&self, viewer: Player) -> Option<EntryChoiceView> {
        if !matches!(self.state.pending, Pending::EntryValue { .. }) {
            return None;
        }
        let batch = self.state.entry_batch.as_ref()?;
        let movement = &batch.moves[*batch.choices.get(batch.next)?];
        if !entries::visible(movement, viewer, &self.state.known_hand) {
            return None;
        }
        Some(EntryChoiceView {
            object: movement.before.reference,
            card: movement.before.card,
            kind: movement.before.card.entry_choice()?,
            selected: batch
                .moves
                .iter()
                .filter(|m| entries::visible(m, viewer, &self.state.known_hand))
                .filter_map(|m| Some((m.before.reference, m.choice?)))
                .collect(),
        })
    }
}
/// Shared earlier-characteristic matching. It never derives stats recursively.
pub(super) fn modifier_bonus<W: CharacteristicWorld + ?Sized>(
    world: &W,
    source: CardId,
    target: CardId,
    modifier: ChosenModifier,
) -> Stats {
    let provider = world.object(source);
    let object = world.object(target);
    if !world.printed_abilities_available(source)
        || provider.zone != Zone::Battlefield
        || object.zone != Zone::Battlefield
        || provider.controller != object.controller
        || world.kinds(target) & 1 == 0
        || (modifier.other && source == target)
    {
        return Stats::default();
    }
    let matches = match (
        modifier.kind,
        world.game().entry_value(world.reference(source)),
    ) {
        (EntryChoiceKind::Color, Some(EntryChoiceValue::Color(c))) => {
            world.colors(target) & (1 << c.index()) != 0
        }
        (EntryChoiceKind::CreatureType, Some(EntryChoiceValue::CreatureType(t))) => {
            world.subtypes(target).has(object.card, t.name())
        }
        _ => false,
    };
    if !matches {
        return Stats::default();
    }
    let amount = modifier
        .source_counter
        .map_or(1, |kind| i128::from(world.counter_count(source, kind)));
    Stats {
        power: modifier
            .stats
            .power
            .checked_mul(amount)
            .capacity("chosen power exhausted"),
        toughness: modifier
            .stats
            .toughness
            .checked_mul(amount)
            .capacity("chosen toughness exhausted"),
    }
}
impl super::characteristic_query::CharacteristicQuery<'_> {
    pub(super) fn chosen_bonus(&self, target: CardId) -> Stats {
        let mut result = Stats::default();
        let Some(rows) = self.state.entry_values.as_ref() else {
            return result;
        };
        for row in rows {
            let source = row.object.card;
            if self.object_ref(source) != row.object {
                continue;
            }
            for group in self.state.objects[source as usize]
                .card
                .static_ability_groups()
            {
                for fragment in group.fragments {
                    if let crate::cards::StaticAbilityFragment::ChosenModifier(modifier)=*fragment
                        && self.continuous_fragment_available(source,|f| matches!(f,crate::cards::StaticAbilityFragment::ChosenModifier(m) if *m==modifier)) {
                        result += modifier_bonus(self,source,target,modifier);
                    }
                }
            }
        }
        result
    }
}
