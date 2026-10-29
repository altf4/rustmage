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
    pub opponent_hand: Option<Vec<ObjectView>>,
    pub selected: Vec<(ObjectRef, EntryChoiceValue)>,
}
pub(super) fn owned_bytes(rows: &Rows) -> usize {
    rows.as_ref().map_or(0, |r| {
        std::mem::size_of::<Vec<ChoiceRow>>() + r.len() * std::mem::size_of::<ChoiceRow>()
    })
}
impl Game {
    #[cfg(test)]
    pub(super) fn entry_value(&self, object: ObjectRef) -> Option<EntryChoiceValue> {
        CharacteristicWorld::entry_value(self, object)
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
        if matches!(self.state.pending, Pending::LandCardName { .. }) {
            return crate::CardName::lands().any(|name| emit(EntryChoiceValue::CardName(name)));
        }
        let batch = self.state.entry_batch.as_ref().unwrap();
        let movement = &batch.moves[self.current_entry_value_index().unwrap()];
        match movement.before.card.entry_choice().unwrap() {
            EntryChoiceKind::LandCardName => {
                crate::CardName::lands().any(|name| emit(EntryChoiceValue::CardName(name)))
            }
            EntryChoiceKind::CardNameAfterOpponentHand => {
                crate::CardName::all().any(|name| emit(EntryChoiceValue::CardName(name)))
            }
            EntryChoiceKind::ManaValueParity => [ManaValueParity::Odd, ManaValueParity::Even]
                .into_iter()
                .any(|parity| emit(EntryChoiceValue::ManaValueParity(parity))),
            EntryChoiceKind::Color => ChoiceColor::ALL
                .into_iter()
                .any(|c| emit(EntryChoiceValue::Color(c))),
            EntryChoiceKind::BasicLandType => crate::cards::BasicLandType::ALL
                .into_iter()
                .any(|kind| emit(EntryChoiceValue::BasicLandType(kind))),
            EntryChoiceKind::CreatureType => crate::CREATURE_TYPES.iter().any(|name| {
                emit(EntryChoiceValue::CreatureType(
                    crate::cards::CreatureType::from_name(name).unwrap(),
                ))
            }),
        }
    }
    pub(super) fn choose_entry_value(&mut self, value: EntryChoiceValue) {
        if matches!(self.state.pending, Pending::LandCardName { .. }) {
            assert!(EntryChoiceKind::LandCardName.matches(value));
            let source = self
                .state
                .resolving
                .as_ref()
                .and_then(|r| r.item.source)
                .map(|s| s.object);
            if let Some(source) = source
                && self.on_battlefield(source.card)
                && self.object_ref(source.card) == source
            {
                self.set_entry_value(source, value);
            }
            self.run_program();
            return;
        }
        let batch = self.state.entry_batch.as_ref().unwrap();
        let index = self.current_entry_value_index().unwrap();
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
        if matches!(self.state.pending, Pending::LandCardName { .. }) {
            let item = &self.state.resolving.as_ref()?.item;
            let source = item.source?;
            return Some(EntryChoiceView {
                object: source.object,
                card: source.card,
                kind: EntryChoiceKind::LandCardName,
                opponent_hand: None,
                selected: Vec::new(),
            });
        }
        if !matches!(self.state.pending, Pending::EntryValue { .. }) {
            return None;
        }
        let batch = self.state.entry_batch.as_ref()?;
        let movement = &batch.moves[self.current_entry_value_index()?];
        if !entries::visible_mask(
            movement,
            self.information_mask(viewer),
            &self.state.known_hand,
        ) {
            return None;
        }
        Some(EntryChoiceView {
            object: movement.before.reference,
            card: movement.before.card,
            kind: movement.before.card.entry_choice()?,
            opponent_hand: (self.shares_information(viewer, movement.controller)
                && movement.before.card.entry_choice()
                    == Some(EntryChoiceKind::CardNameAfterOpponentHand))
            .then(|| {
                self.state
                    .objects
                    .iter()
                    .enumerate()
                    .filter(|(_, o)| o.zone == Zone::Hand && o.is_card() && o.owner == movement.controller.other())
                    .map(|(id, _)| self.object_view(id as CardId))
                    .collect()
            }),
            selected: batch
                .moves
                .iter()
                .filter(|m| {
                    entries::visible_mask(m, self.information_mask(viewer), &self.state.known_hand)
                })
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
    let matches = match (modifier.kind, world.entry_value(world.reference(source))) {
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
impl<W: CharacteristicWorld + ?Sized> super::characteristic_query::CharacteristicQuery<'_, W> {
    pub(super) fn chosen_bonus(&self, target: CardId) -> Stats {
        let mut result = Stats::default();
        for row in self.entry_values() {
            let source = row.object.card;
            if self.object_ref(source) != row.object {
                continue;
            }
            if self.object(source).zone != Zone::Battlefield {
                continue;
            }
            for group in self.object(source).card.static_ability_groups() {
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

impl Game {
    pub(super) fn named_activation_prohibited(&self, id: CardId) -> bool {
        self.named_activation_prohibited_for_name(CharacteristicWorld::name(self, id))
    }
    pub(super) fn named_activation_prohibited_for_name(&self, name: &str) -> bool {
        let Some(rows) = self.state.entry_values.as_ref() else {
            return false;
        };
        rows.iter().any(|row| {
            let source = &self.state.objects[row.object.card as usize];
            source.zone == Zone::Battlefield
                && self.object_ref(row.object.card) == row.object
                && source.card.named_activation_lock()
                && self.copy_ability_origin(row.object).is_none()
                && self.printed_abilities_available(row.object.card)
                && matches!(row.value, EntryChoiceValue::CardName(chosen) if chosen.matches(name))
        })
    }
}
