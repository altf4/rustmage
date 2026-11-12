//! Activated-ability instances read while their layer-six effect is applied.
use super::ability_order::{
    GrantedAbilityKey, GrantedAbilityKind, LayerSixAction, LayerSixEffect, LayerSixObject,
    ManaGrantOrigin,
};
use super::ability_world::CharacteristicWorld;
use super::*;
use std::collections::{BTreeMap, BTreeSet};

#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
enum DonorAbility {
    Printed(Option<u64>),
    Retained(Option<u64>),
    Granted {
        source: ObjectRef,
        timestamp: u64,
        fragment: Option<(u16, u16)>,
    },
    SelfGranted(u64),
    Borrowed {
        slot: u64,
        link_cohort: u64,
    },
    Inherited {
        slot: u64,
        link_cohort: u64,
    },
    Mana {
        source: ObjectRef,
        origin: ManaGrantOrigin,
    },
}

#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub(super) struct Acquisition {
    borrower_origin: Option<u64>,
    line: u16,
    donor: ObjectRef,
    ability: DonorAbility,
}

#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
impl Acquisition {
    pub(super) fn slot_identity(self) -> Self {
        let ability = match self.ability {
            DonorAbility::Borrowed { slot, .. } => DonorAbility::Borrowed {
                slot,
                link_cohort: 0,
            },
            DonorAbility::Inherited { slot, .. } => DonorAbility::Inherited {
                slot,
                link_cohort: 0,
            },
            unchanged => unchanged,
        };
        Self { ability, ..self }
    }

    pub(super) fn shares_link_cohort(self, other: Self) -> bool {
        self.borrower_origin == other.borrower_origin
            && self.line == other.line
            && self.donor == other.donor
            && match (self.ability, other.ability) {
                (
                    DonorAbility::Borrowed {
                        link_cohort: left, ..
                    },
                    DonorAbility::Borrowed {
                        link_cohort: right, ..
                    },
                )
                | (
                    DonorAbility::Inherited {
                        link_cohort: left, ..
                    },
                    DonorAbility::Inherited {
                        link_cohort: right, ..
                    },
                ) => left == right,
                (
                    DonorAbility::Granted {
                        source: left,
                        timestamp: left_time,
                        fragment: left_fragment,
                    },
                    DonorAbility::Granted {
                        source: right,
                        timestamp: right_time,
                        fragment: right_fragment,
                    },
                ) => {
                    left == right
                        && left_time == right_time
                        && left_fragment.map(|(line, _)| line)
                            == right_fragment.map(|(line, _)| line)
                }
                (left, right) => left == right,
            }
    }
}

#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
#[derive(Clone, Copy)]
struct Template {
    definition: Card,
    ability: u16,
    origin: DonorAbility,
    mana: Option<ManaProduction>,
    attachment: Option<AttachmentTriggerView>,
}

impl Template {
    fn new(definition: Card, ability: u16, origin: DonorAbility) -> Self {
        Self {
            definition,
            ability,
            origin,
            mana: None,
            attachment: None,
        }
    }
}

#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
pub(super) fn donors<W: CharacteristicWorld + ?Sized>(world: &W, id: CardId) -> Vec<CardId> {
    let recipient = world.object(id);
    let name = world.name(id);
    (0..world.game().state.objects.len())
        .map(|id| id as CardId)
        .filter(|&id| {
            let donor = world.object(id);
            donor.zone == Zone::Battlefield
                && donor.controller == recipient.controller
                && world.kinds(id) & 1 != 0
                && !crate::card_names::same_name(world.name(id), name)
        })
        .collect()
}

#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
pub(super) fn has_provider<W: CharacteristicWorld + ?Sized>(world: &W) -> bool {
    world.static_sources().iter().any(|&id| {
        let object = world.object(id);
        object.zone == Zone::Battlefield && object.card.inherits_creature_activations()
    })
}

#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
pub(super) fn abilities<W: CharacteristicWorld + ?Sized>(
    world: &W,
    id: CardId,
) -> Vec<BorrowedDescriptor> {
    let object = world.object(id);
    if object.zone != Zone::Battlefield || !object.card.inherits_creature_activations() {
        return Vec::new();
    }
    let mut evaluation = ability_evaluation::evaluate_inherited(world);
    std::mem::take(&mut evaluation.result.objects[id as usize].inherited)
}

#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
pub(super) fn descriptor<W: CharacteristicWorld + ?Sized>(
    world: &W,
    source: ObjectRef,
    slot: u64,
) -> Option<BorrowedDescriptor> {
    world.game().state.objects.get(source.card as usize)?;
    if world.reference(source.card) != source {
        return None;
    }
    abilities(world, source.card)
        .into_iter()
        .find(|ability| ability.slot == slot)
}

#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
pub struct InheritedAbilityView {
    pub source: ObjectRef,
    pub donor: ObjectRef,
    pub definition: Card,
    pub ability: u16,
    pub slot: u64,
    pub used: bool,
}

#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
impl Game {
    pub(super) fn inherited_ability_views(&self) -> Vec<InheritedAbilityView> {
        if !has_provider(self) {
            return Vec::new();
        }
        ability_evaluation::evaluate_inherited(self)
            .result
            .objects
            .into_iter()
            .flat_map(|object| object.inherited)
            .map(|ability| InheritedAbilityView {
                source: ability.source,
                donor: ability.donor,
                definition: ability.definition,
                ability: ability.ability,
                slot: ability.slot,
                used: ability.used,
            })
            .collect()
    }
}

struct DonorInventory {
    definition: Card,
    printed: Vec<Template>,
    self_granted: Vec<(u64, Template)>,
    borrowed: Vec<BorrowedDescriptor>,
}

impl DonorInventory {
    fn new<W: CharacteristicWorld + ?Sized>(world: &W, id: CardId) -> Self {
        let definition = world.definition(id);
        let source = world.reference(id);
        let origin = world.ability_origin(id);
        let mut printed: Vec<_> = definition
            .activations()
            .iter()
            .enumerate()
            .filter_map(|(index, _)| {
                let ability = index as u16;
                (!definition.grants_activation(ability)
                    && !definition.self_granted_activation_template(ability)
                    && !definition.legacy_subtype_mana_activation(ability))
                .then(|| Template::new(definition, ability, DonorAbility::Printed(origin)))
            })
            .collect();
        if definition.mana_color().is_some()
            && !(definition.is_land()
                && definition.printed_intrinsic_mana_colors().next().is_some())
        {
            printed.push(Template::new(
                definition,
                u16::MAX,
                DonorAbility::Printed(origin),
            ));
        }
        if let Some(retained) = world.recipe(id).exceptions.retained_activation {
            printed.push(Template::new(
                retained.definition,
                retained.ability,
                DonorAbility::Retained(origin),
            ));
        }
        let self_granted = world
            .game()
            .state
            .self_granted_abilities
            .as_ref()
            .into_iter()
            .flat_map(|rows| rows.iter())
            .filter(|row| row.recipient == source)
            .filter_map(|row| {
                Some((
                    row.timestamp,
                    Template::new(
                        row.granting_card,
                        row.kind.activation()?,
                        DonorAbility::SelfGranted(row.timestamp),
                    ),
                ))
            })
            .collect();
        let borrowed = borrowed_activations::inventory(
            world,
            id,
            borrowed_activations::InventoryStage::BeforeLayerSix,
        );
        Self {
            definition,
            printed,
            self_granted,
            borrowed,
        }
    }

    fn grant_has_activation(&self, grant: &GrantedAbilityKey) -> bool {
        match grant.kind {
            GrantedAbilityKind::ActivatedTemplate(_) => true,
            GrantedAbilityKind::LinkedExileActivations => self
                .borrowed
                .iter()
                .any(|ability| ability.grantor == Some(grant.source)),
            _ => false,
        }
    }

    fn has_printed_activation(&self, state: &LayerSixObject) -> bool {
        state.printed
            && (!self.printed.is_empty() || self.borrowed.iter().any(|a| a.grantor.is_none()))
    }

    fn visit<W: CharacteristicWorld + ?Sized>(
        &self,
        world: &W,
        state: &LayerSixObject,
        mut emit: impl FnMut(Template) -> bool,
    ) -> bool {
        (state.printed && self.printed.iter().copied().any(&mut emit))
            || state
                .grants
                .iter()
                .filter_map(|grant| {
                    let GrantedAbilityKind::ActivatedTemplate(ability) = grant.kind else {
                        return None;
                    };
                    let attachment = grant.fragment.and_then(|(line, fragment)| {
                        let group = grant
                            .granting_card
                            .static_ability_groups()
                            .iter()
                            .find(|group| group.line == line)?;
                        matches!(
                            group.fragments.get(usize::from(fragment)),
                            Some(crate::cards::StaticAbilityFragment::AttachmentActivation { .. })
                        )
                        .then(|| AttachmentTriggerView {
                            grantor: grant.source,
                            granting_card: grant.granting_card,
                            // Rebound to the current recipient when the inheritance is applied.
                            recipient: grant.source,
                            defender: None,
                            granted: true,
                            origin: world.provided_origin(grant.source),
                        })
                    });
                    Some(Template {
                        attachment,
                        ..Template::new(
                            grant.granting_card,
                            ability,
                            DonorAbility::Granted {
                                source: grant.source,
                                timestamp: grant.timestamp,
                                fragment: grant.fragment,
                            },
                        )
                    })
                })
                .any(&mut emit)
            || self
                .self_granted
                .iter()
                .filter_map(|&(time, template)| {
                    state.self_grants.contains(&time).then_some(template)
                })
                .any(&mut emit)
            || self
                .borrowed
                .iter()
                .filter_map(|ability| {
                    let active = ability.grantor.map_or(state.printed, |source| {
                        state.grants.iter().any(|grant| {
                            grant.source == source
                                && grant.kind == GrantedAbilityKind::LinkedExileActivations
                        })
                    });
                    active.then(|| {
                        Template::new(
                            ability.definition,
                            ability.ability,
                            DonorAbility::Borrowed {
                                slot: ability.slot,
                                link_cohort: ability.link_cohort(),
                            },
                        )
                    })
                })
                .any(&mut emit)
            || state
                .mana
                .iter()
                .filter_map(|mana| {
                    let origin = mana.origin?;
                    let ability = if mana.sacrifice {
                        Card::GRANTED_SACRIFICE_MANA
                    } else {
                        Card::GRANTED_TAP_MANA
                    };
                    Some(Template {
                        mana: Some(mana.production),
                        ..Template::new(
                            self.definition,
                            ability,
                            DonorAbility::Mana {
                                source: mana.source,
                                origin,
                            },
                        )
                    })
                })
                .any(&mut emit)
            || state.inherited.iter().any(|ability| {
                emit(Template {
                    mana: ability.mana,
                    attachment: ability.attachment,
                    ..Template::new(
                        ability.definition,
                        ability.ability,
                        DonorAbility::Inherited {
                            slot: ability.slot,
                            link_cohort: ability.link_cohort(),
                        },
                    )
                })
            })
    }
}

#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
pub(super) struct DonorInventories(BTreeMap<CardId, DonorInventory>);

#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
impl DonorInventories {
    pub(super) fn new<W: CharacteristicWorld + ?Sized>(
        world: &W,
        effects: &[LayerSixEffect],
    ) -> Self {
        let donors: BTreeSet<_> = effects
            .iter()
            .filter_map(|effect| match &effect.action {
                LayerSixAction::InheritCreatureActivations { donors, .. } => {
                    Some(donors.iter().copied())
                }
                _ => None,
            })
            .flatten()
            .collect();
        Self(
            donors
                .into_iter()
                .map(|id| (id, DonorInventory::new(world, id)))
                .collect(),
        )
    }

    pub(super) fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    pub(super) fn effect_changes_abilities<W: CharacteristicWorld + ?Sized>(
        &self,
        world: &W,
        effect: &LayerSixEffect,
        donor: CardId,
        objects: &[LayerSixObject],
    ) -> bool {
        let inventory = &self.0[&donor];
        let state = &objects[donor as usize];
        let recipient = effect.recipients.contains(&donor);
        let grants_abilities = |rows: &[ability_order::LayerSixGrant]| {
            rows.iter().any(|row| {
                row.recipient == donor
                    && (!row.mana.is_empty()
                        || row
                            .abilities
                            .iter()
                            .any(|grant| inventory.grant_has_activation(grant)))
            })
        };
        match &effect.action {
            LayerSixAction::InheritCreatureActivations { donors, .. } => {
                recipient
                    && donors
                        .iter()
                        .any(|&id| self.0[&id].visit(world, &objects[id as usize], |_| true))
            }
            LayerSixAction::Remove => recipient && inventory.visit(world, state, |_| true),
            LayerSixAction::RemoveAndGrant(rows) => {
                recipient && inventory.visit(world, state, |_| true) || grants_abilities(rows)
            }
            LayerSixAction::RemovePrintedAndGrant(rows) => {
                recipient
                    && (inventory.has_printed_activation(state)
                        || state.mana.iter().any(|mana| mana.subtype_derived))
                    || grants_abilities(rows)
            }
            LayerSixAction::RemoveSubtypeMana => {
                recipient && state.mana.iter().any(|mana| mana.subtype_derived)
            }
            LayerSixAction::GrantSelf(time) => {
                recipient
                    && inventory
                        .self_granted
                        .iter()
                        .any(|(timestamp, _)| timestamp == time)
            }
            LayerSixAction::SelfGrantedMana { .. } => recipient,
            LayerSixAction::SelfGrantedKeywords { .. } => false,
            LayerSixAction::GrantEach(rows)
            | LayerSixAction::RemoveKeywordsAndGrant { grants: rows, .. } => grants_abilities(rows),
            LayerSixAction::Grant { abilities, .. } => {
                recipient
                    && abilities
                        .iter()
                        .any(|grant| inventory.grant_has_activation(grant))
            }
        }
    }

    pub(super) fn inherit<W: CharacteristicWorld + ?Sized>(
        &self,
        world: &W,
        recipient: CardId,
        line: u16,
        donors: &[CardId],
        objects: &mut [LayerSixObject],
        slots: &mut Vec<borrowed_activations::SlotRow>,
    ) {
        let source = world.reference(recipient);
        let borrower_origin = world.ability_origin(recipient);
        let mut inherited = Vec::new();
        for &id in donors {
            let donor = world.reference(id);
            self.0[&id].visit(world, &objects[id as usize], |template| {
                let mut ability = BorrowedDescriptor {
                    source,
                    grantor: None,
                    donor,
                    zone: Zone::Battlefield,
                    definition: template.definition,
                    ability: template.ability,
                    slot: 0,
                    link: None,
                    used: false,
                    once_per_turn: false,
                    acquisition: 0,
                    inherited: Some(Acquisition {
                        borrower_origin,
                        line,
                        donor,
                        ability: template.origin,
                    }),
                    mana: template.mana,
                    attachment: template.attachment.map(|grant| AttachmentTriggerView {
                        recipient: source,
                        ..grant
                    }),
                };
                borrowed_activations::assign_slot(world, slots, &mut ability);
                inherited.push(ability);
                false
            });
        }
        borrowed_activations::assign_inherited_links(slots, &mut inherited);
        objects[recipient as usize].inherited.extend(inherited);
    }
}
