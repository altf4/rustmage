//! Public creature-type choice and one owned, replacement-aware token batch.
use super::*;
use crate::cards::CreatureType;

#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
#[derive(Clone, Debug, Serialize)]
pub struct CreatureTypeChoiceView {
    pub player: Player,
    pub selected: Option<CreatureType>,
}
#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
pub struct SecretCreatureTypeView {
    pub source: ObjectRef,
    pub chooser: Player,
    pub creature_type: CreatureType,
}

#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) struct SecretCreatureType {
    view: SecretCreatureTypeView,
    definition: Card,
    origin: Option<u64>,
}

#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
impl Game {
    pub(super) fn visit_creature_type_tokens(&self, emit: &mut impl FnMut(Action) -> bool) -> bool {
        let resolution = self.state.resolving.as_ref().unwrap();
        let names = match resolution.program[resolution.pc - 1] {
            Instruction::ChooseSecretCreatureType(types) => types,
            _ => crate::CREATURE_TYPES,
        };
        names.iter().any(|name| {
            emit(Action::ChooseCreatureType {
                creature_type: CreatureType::from_name(name).expect("validated taxonomy"),
            })
        })
    }
    pub(super) fn choose_type_for_tokens(&mut self, creature_type: CreatureType) {
        let r = self.state.resolving.as_mut().unwrap();
        let instruction = r.program[r.pc - 1];
        if !matches!(instruction, Instruction::ChooseSecretCreatureType(_)) {
            r.chosen_creature_type = Some(creature_type);
        }
        let player = r.item.controller;
        match instruction {
            Instruction::ChooseCreatureType => {}
            Instruction::ChooseSecretCreatureType(_) => {
                let row = SecretCreatureType {
                    view: SecretCreatureTypeView {
                        source: r.item.source.expect("object-backed choice").object,
                        chooser: player,
                        creature_type,
                    },
                    definition: r.item.trigger_definition(),
                    origin: r.item.source_ability_origin,
                };
                if self.on_battlefield(row.view.source.card)
                    && self.object_ref(row.view.source.card) == row.view.source
                {
                    let state = self.state.linked_choices.get_or_insert_default();
                    state.secret_creature_types.retain(|old| {
                        old.view.source != row.view.source
                            || old.definition != row.definition
                            || old.origin != row.origin
                    });
                    state.secret_creature_types.push(row);
                }
            }
            Instruction::CreateEmblemForChosenCreatureType(factory) => {
                let id = CardId::try_from(self.state.objects.len())
                    .capacity("emblem object slots exhausted");
                self.create_emblem(player, factory);
                self.set_entry_value(
                    self.object_ref(id),
                    EntryChoiceValue::CreatureType(creature_type),
                );
            }
            Instruction::CreateTokensForChosenCreatureType { token } => {
                let count = self
                    .state
                    .objects
                    .iter()
                    .enumerate()
                    .filter(|(id, o)| {
                        o.zone == Zone::Battlefield
                            && o.controller == player
                            && self.is_creature(*id as CardId)
                            && self.has_subtype(*id as CardId, creature_type.name())
                    })
                    .count() as u128;
                self.create_tokens(player, token, count, false);
            }
            Instruction::DestroyCreaturesExceptChosenType => {
                let mut ids: Vec<_> = self
                    .state
                    .objects
                    .iter()
                    .enumerate()
                    .filter_map(|(id, o)| {
                        (o.zone == Zone::Battlefield
                            && self.is_creature(id as CardId)
                            && !self.has_subtype(id as CardId, creature_type.name()))
                        .then_some(id as CardId)
                    })
                    .collect();
                ids.retain(|&id| !self.effect_destruction_prevented(id));
                let sources = self.death_trigger_sources();
                let events = self.move_objects_raw(&ids, Zone::Graveyard);
                self.finish_zone_events(&events, &sources);
            }
            _ => unreachable!("creature-type choice continuation"),
        }
        self.run_program();
    }
    pub(super) fn secret_creature_type(
        &self,
        source: ObjectRef,
        definition: Card,
        chooser: Player,
        origin: Option<u64>,
    ) -> Option<CreatureType> {
        self.state
            .linked_choices
            .as_ref()?
            .secret_creature_types
            .iter()
            .find(|row| {
                row.view.source == source
                    && row.view.chooser == chooser
                    && row.definition == definition
                    && row.origin == origin
            })
            .map(|row| row.view.creature_type)
    }

    pub(super) fn reveal_chosen_creature_type_for_payment(&self, item: &mut SpellView) {
        let source = item.source.expect("linked choice activation");
        let kind = self
            .secret_creature_type(
                source.object,
                source.card,
                item.controller,
                item.source_ability_origin,
            )
            .expect("legal secret reveal cost");
        item.set_event_object(EventObjectView {
            object: source.object,
            facts: EventObjectFacts::RevealedCreatureType(kind),
            semantic_objects: None,
            conniver_creature: None,
            subtype_lki: None,
        });
    }

    pub(super) fn forget_secret_creature_type(&mut self, object: ObjectRef) {
        if let Some(state) = &mut self.state.linked_choices {
            state
                .secret_creature_types
                .retain(|row| row.view.source != object);
        }
    }

    pub(super) fn secret_creature_types_view(&self, viewer: Player) -> Vec<SecretCreatureTypeView> {
        self.state
            .linked_choices
            .as_ref()
            .into_iter()
            .flat_map(|state| &state.secret_creature_types)
            .filter(|row| self.shares_information(viewer, row.view.chooser))
            .map(|row| row.view)
            .collect()
    }

    pub(super) fn return_resolution_chosen_type(&mut self, controller: Player) {
        let kind = self
            .state
            .resolving
            .as_ref()
            .unwrap()
            .chosen_creature_type
            .expect("preceding creature type choice");
        let moves = self
            .state
            .objects
            .iter()
            .enumerate()
            .filter(|(id, object)| {
                object.zone == Zone::Graveyard
                    && object.owner == controller
                    && object.is_card()
                    && self.current_kinds(*id as CardId) & 1 != 0
                    && self.has_subtype(*id as CardId, kind.name())
            })
            .map(|(id, _)| {
                let mut m = self.entry_move(id as CardId, Zone::Battlefield, false);
                m.controller = controller;
                m
            })
            .collect();
        self.stage_entries(moves, None);
    }
    pub(super) fn creature_type_tokens_view(&self) -> Option<CreatureTypeChoiceView> {
        let r = self.state.resolving.as_ref()?;
        if !matches!(self.state.pending, Pending::CreatureTypeChoice { .. })
            && r.chosen_creature_type.is_none()
        {
            return None;
        }
        Some(CreatureTypeChoiceView {
            player: r.item.controller,
            selected: r.chosen_creature_type,
        })
    }
}
