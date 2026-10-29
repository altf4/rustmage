//! CR702.192: recurring first-main-phase copies, independent of the exiled source.
//! XMage's pinned ParadigmAbility is a TODO; the official SOS release notes and
//! Comprehensive Rules 2026-06-19 supply the authoritative behavior.
use super::*;

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct ResolvedSpellNameView {
    pub player: Player,
    pub name: &'static str,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct ParadigmView {
    pub controller: Player,
    pub definition: Card,
    pub name: &'static str,
}
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub(super) struct ParadigmState {
    resolved: SharedVec<ResolvedSpellNameView>,
    delayed: SharedVec<SpellView>,
    ephemeral: Option<ObjectRef>,
}
impl ParadigmState {
    #[cfg(test)]
    pub(super) fn packets(&self) -> impl Iterator<Item = &SpellView> {
        self.delayed.iter()
    }
}
pub(super) fn owned_bytes(data: &Option<Box<ParadigmState>>) -> usize {
    data.as_ref().map_or(0, |data| {
        std::mem::size_of::<ParadigmState>()
            + std::mem::size_of::<Vec<ResolvedSpellNameView>>()
            + std::mem::size_of::<Vec<SpellView>>()
            + data.resolved.len() * std::mem::size_of::<ResolvedSpellNameView>()
            + data
                .delayed
                .iter()
                .map(|row| std::mem::size_of::<SpellView>() + row.owned_bytes())
                .sum::<usize>()
    })
}
/// Derive immutable relevant names from authored typed programs once per process.
/// Ordinary games never allocate historical rows for names which cannot matter
/// to paradigm. A same-name spell still qualifies even without the ability.
fn possible_paradigm_name(name: &str) -> bool {
    static NAMES: std::sync::OnceLock<Box<[&'static str]>> = std::sync::OnceLock::new();
    NAMES
        .get_or_init(|| {
            let paradigm = |program: &[Instruction]| {
                program
                    .iter()
                    .any(|instruction| matches!(instruction, Instruction::Paradigm { .. }))
            };
            let mut names: Vec<&'static str> = Vec::new();
            for card in Card::catalogue() {
                let relevant = match card.effect() {
                    Effect::Spell(_, program) => paradigm(program),
                    Effect::Modal(modes) => modes.iter().any(|mode| paradigm(mode.program)),
                    _ => false,
                };
                if relevant
                    && !names
                        .iter()
                        .any(|previous| crate::card_names::same_name(previous, card.name()))
                {
                    names.push(card.name());
                }
            }
            names.into_boxed_slice()
        })
        .iter()
        .any(|candidate| crate::card_names::same_name(candidate, name))
}
impl Game {
    pub(super) fn record_resolved_spell_name(&mut self, item: &SpellView) {
        if item.kind != StackKind::Spell {
            return;
        }
        let Some(source) = item.source else { return; };
        let name = source.card.name();
        if !possible_paradigm_name(name) {
            return;
        }
        let data = self.state.paradigm.get_or_insert_with(Default::default);
        if !data.resolved.iter().any(|row| {
            row.player == item.controller && crate::card_names::same_name(row.name, name)
        }) {
            data.resolved.push(ResolvedSpellNameView {
                player: item.controller,
                name,
            });
        }
    }
    pub(super) fn resolve_paradigm(&mut self, item: &SpellView, ability: u16) {
        if item.kind != StackKind::Spell {
            return;
        }
        let Some(source) = item.source else { return; };
        let name = source.card.name();
        let already_resolved = self.state.paradigm.as_ref().is_some_and(|data| {
            data.resolved.iter().any(|row| {
                row.player == item.controller && crate::card_names::same_name(row.name, name)
            })
        });
        if !already_resolved {
            let mut delayed = item.clone();
            delayed.kind = StackKind::TriggeredAbility { ability };
            delayed.cast = None;
            delayed.is_copy = false;
            delayed.mode = None;
            delayed.x = None;
            delayed.targets.clear();
            delayed.target_type_slots = None;
            delayed.target_controller_lki = None;
            delayed.target_kinds_lki = None;
            delayed.target_allocations = None;
            delayed.related_stack = None;
            delayed.copy_event = None;
            delayed.cast_event = None;
            delayed.gift = None;
            delayed.mana_copy_sources = None;
            self.state
                .paradigm
                .get_or_insert_with(Default::default)
                .delayed
                .push(delayed);
        }
        if self.object_ref(source.id) == source.object
            && self.state.objects[source.id as usize].zone == Zone::Stack
        {
            self.move_object(source.id, Zone::Exile);
        }
    }
    pub(super) fn queue_paradigm_triggers(&mut self) {
        let templates: Vec<_> = self.state.paradigm.as_ref().map_or_else(Vec::new, |data| {
            data.delayed
                .iter()
                .filter(|item| item.controller == self.state.active)
                .cloned()
                .collect()
        });
        for mut trigger in templates {
            trigger.id = self.next_stack_id();
            self.state.pending_triggers.push(trigger);
        }
    }
    pub(super) fn cast_paradigm_copy(&mut self, item: &SpellView) {
        let Some(source) = item.source else { return; };
        let object = self.allocate_card_copy(source.card, item.controller);
        self.state
            .paradigm
            .get_or_insert_with(Default::default)
            .ephemeral = Some(object);
        self.state.resolution_free_casts = Some(Box::new(ResolutionFreeCasts {
            siege_transformed: false,
            paid: None,
            casts_remaining: None,
            pending_cast: None,
            remaining_mana_value: None,
            alternative_cost: None,
            maximum_mana_value: None,
            origin: Zone::Exile,
            exile_on_graveyard: false,
            allow_lands: false,
            player: item.controller,
            remaining: vec![object],
        }));
        self.state.pending = Pending::ResolutionCasts {
            player: item.controller,
        };
    }
    /// Only an uncast copy is discarded. A paid spell has become a new incarnation.
    pub(super) fn finish_paradigm_copy_offer(&mut self) {
        let object = self
            .state
            .paradigm
            .as_mut()
            .and_then(|data| data.ephemeral.take());
        if let Some(object) = object {
            self.cease_card_copy(object);
        }
    }
    pub(super) fn exile_until_mana_value_cast_free(&mut self, player: Player, minimum: u16) {
        let mut remaining = Vec::new();
        let mut total = 0u64;
        while total < u64::from(minimum) {
            let Some(&card) = self.state.players[player.index()].library.last() else {
                break;
            };
            let value = self.state.objects[card as usize].card.fixed_mana_value();
            let event = self.move_object(card, Zone::Exile);
            if event.moved && event.destination == Zone::Exile {
                total = total
                    .checked_add(value)
                    .capacity("exiled mana value total exhausted");
                remaining.push(event.after);
            }
            if !event.moved {
                break;
            }
        }
        self.state.resolution_free_casts = Some(Box::new(ResolutionFreeCasts {
            siege_transformed: false,
            paid: None,
            casts_remaining: None,
            pending_cast: None,
            remaining_mana_value: None,
            alternative_cost: None,
            maximum_mana_value: None,
            origin: Zone::Exile,
            exile_on_graveyard: false,
            player,
            remaining,
            allow_lands: false,
        }));
        self.state.pending = Pending::ResolutionCasts { player };
    }
    pub(super) fn paradigm_views(&self) -> Vec<ParadigmView> {
        self.state.paradigm.as_ref().map_or_else(Vec::new, |data| {
            data.delayed
                .iter()
                .filter_map(|item| item.source.map(|source| ParadigmView {
                    controller: item.controller,
                    definition: source.card,
                    name: source.card.name(),
                }))
                .collect()
        })
    }
    pub(super) fn resolved_spell_name_views(&self) -> Vec<ResolvedSpellNameView> {
        self.state
            .paradigm
            .as_ref()
            .map_or_else(Vec::new, |data| data.resolved.to_vec())
    }
}
