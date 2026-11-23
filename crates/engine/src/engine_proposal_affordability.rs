//! Reject only spell and ability proposals with a proven current payment shortage.
//!
//! Build the source and price summaries once per menu. Every recognized candidate
//! then checks its own typed cost options against those summaries. An unmodeled
//! cost, source, or way to contribute keeps the proposal available.
use super::*;
use crate::cards::AbilityLayer;
use payment_capacity::PaymentCapacity;
use proposal_price::ProposalPriceFacts;

pub(super) struct ProposalAffordability {
    capacity: PaymentCapacity,
    price: ProposalPriceFacts,
    sacrifice_kinds: Option<u16>,
}

impl ProposalAffordability {
    pub(super) fn for_player(game: &Game, actor: Player) -> Option<Self> {
        Some(Self {
            capacity: PaymentCapacity::for_player(game, actor)?,
            price: ProposalPriceFacts::new(game, actor),
            sacrifice_kinds: stable_sacrifice_kinds(game, actor),
        })
    }

    pub(super) fn may_be_affordable(&self, game: &Game, action: &Action) -> bool {
        if let Action::AnnounceActivate { card, ability, .. } = *action {
            return self.activation_may_be_affordable(game, card, ability);
        }
        let Some(spell) = SpellProposal::from_action(game, action) else {
            return true;
        };
        let object = &game.state.objects[spell.card as usize];
        let definition = if spell.method == CastMethod::Disguise {
            manifest::FaceDownCause::Disguise.definition()
        } else if let Some(definition) = spell.face.definition(object.card) {
            definition
        } else {
            return true;
        };
        let permits = |price: ManaCost| {
            game.spell_mana_lower_bound(
                definition,
                spell.card,
                object.zone,
                spell.method,
                price,
                &self.price,
            )
            .is_none_or(|cost| {
                self.capacity
                    .permits_spell(cost.colored, cost.generic, definition, object.zone)
            })
        };
        if let Some(cost) = spell.cost {
            return permits(cost);
        }
        if spell.method == CastMethod::Normal
            && object.zone == Zone::Hand
            && self.requires_mana_instead_of_sacrifice(definition)
        {
            if let Some(fee) = definition.explicit_additional_mana() {
                return definition.cost_options().any(|base| {
                    permits(ManaCost {
                        colored: std::array::from_fn(|i| {
                            base.colored[i].saturating_add(fee.colored[i])
                        }),
                        generic: base.generic.saturating_add(fee.generic),
                    })
                });
            }
            let mut alternatives = definition
                .cost_options()
                .filter(|&cost| definition.mana_alternative_replaces_additional(cost));
            return alternatives
                .next()
                .is_none_or(|cost| permits(cost) || alternatives.any(permits));
        }
        match spell.method {
            CastMethod::Airbend => permits(ManaCost {
                colored: [0; 6],
                generic: 2,
            }),
            CastMethod::Disguise => permits(ManaCost {
                colored: [0; 6],
                generic: 3,
            }),
            CastMethod::Flashback => {
                definition.flashback_cost().is_none()
                    || definition.flashback_cost_options().any(permits)
            }
            CastMethod::Harmonize => definition.harmonize_cost().is_none_or(permits),
            CastMethod::Foretell => definition.foretell_cost().is_none_or(permits),
            CastMethod::Warp => definition.warp_spec().is_none_or(|warp| {
                std::iter::once(warp.mana)
                    .chain(warp.alternatives.iter().copied())
                    .any(permits)
            }),
            CastMethod::Mayhem => definition
                .mayhem_cost_options()
                .is_none_or(|mut costs| costs.any(permits)),
            CastMethod::GrantedHarmonize | CastMethod::GrantedMayhem => {
                crate::cards::mana_symbols::printed_cost(definition.info().mana_cost)
                    .is_none_or(|cost| cost.options.into_iter().any(permits))
            }
            CastMethod::Normal | CastMethod::GrantedFlashback => {
                definition.cost_options().any(permits)
                    || (spell.method == CastMethod::Normal
                        && (definition.evoke_costs().iter().copied().any(permits)
                            || definition
                                .impending_spec()
                                .is_some_and(|spec| permits(spec.cost))))
            }
            // A new or variable route must opt in explicitly.
            _ => true,
        }
    }

    fn requires_mana_instead_of_sacrifice(&self, definition: Card) -> bool {
        let Some(kinds) = self.sacrifice_kinds else {
            return false;
        };
        // Optional declarations and alternative spell modes can replace the
        // printed material requirement. Keep those branches with the caller.
        if definition.has_bargain()
            || definition.collect_evidence_cost().is_some()
            || definition.kicker_additional_cost().is_some()
            || definition.impending_spec().is_some()
            || !definition.evoke_costs().is_empty()
            || definition.mana_additional_alternative().is_none()
        {
            return false;
        }
        definition.additional_cost().is_some_and(|cost| {
            cost.kind == EffectChoiceKind::Sacrifice
                && cost.filter.kinds != 0
                && kinds & u16::from(cost.filter.kinds) == 0
        })
    }

    fn activation_may_be_affordable(&self, game: &Game, source: CardId, ability: u16) -> bool {
        let definition = game.state.objects[source as usize].card;
        let activation = definition.activation(ability);
        if activation.equip_discard_alternative() {
            return true;
        }
        let reserved = activation.cost.tap.then_some(source);
        definition.activation_cost_options(ability).any(|price| {
            game.activation_mana_lower_bound(definition, source, ability, price, &self.price)
                .is_none_or(|cost| self.capacity.permits(cost.colored, cost.generic, reserved))
        })
    }
}

/// Optimistic union of materials available for a sacrifice. Allow any
/// permanent of the printed kinds even if the particular cost excludes it.
/// Unsettled type, control, or linked-entry effects leave the union unknown.
fn stable_sacrifice_kinds(game: &Game, actor: Player) -> Option<u16> {
    let state = &game.state;
    if !state.exile_links.is_empty()
        || state.controls.is_some()
        || state.temporary_controls.is_some()
        || state.copy_recipes.is_some()
        || state.temporary_copies.is_some()
        || state.manifest.is_some()
        || state.earthbend_returns.is_some()
        || state.roles.as_ref().is_some_and(|roles| {
            !roles.animations.is_empty()
                || !roles.source_linked_effects.is_empty()
                || roles.control_change_sources.is_some()
        })
    {
        return None;
    }
    let mut kinds = 0;
    for object in &state.objects {
        if !matches!(object.zone, Zone::Battlefield | Zone::Command) {
            continue;
        }
        if object
            .card
            .static_ability_groups()
            .iter()
            .any(|group| group.first_layer <= AbilityLayer::Type)
        {
            return None;
        }
        if object.zone == Zone::Battlefield && object.controller == actor {
            kinds |= object.card.kind_mask();
        }
    }
    Some(kinds)
}

struct SpellProposal {
    card: CardId,
    face: CastFace,
    method: CastMethod,
    cost: Option<ManaCost>,
}

impl SpellProposal {
    /// Routes emitted by the native priority visitor. Additional taxes and
    /// optional fees can only increase these base prices.
    fn from_action(game: &Game, action: &Action) -> Option<Self> {
        use Action::*;
        use CastFace::Normal;
        let (card, face, method, cost) = match *action {
            AnnounceCast { card }
            | AnnounceExile { card }
            | PlayGraveyardPermanent { card, .. } => (card, Normal, CastMethod::Normal, None),
            AnnounceFlashSurcharge { card } => {
                let method = if game.state.objects[card as usize].zone == Zone::Graveyard {
                    CastMethod::Flashback
                } else {
                    CastMethod::Normal
                };
                (card, Normal, method, None)
            }
            AnnounceGraveyardFlashSurcharge { card } => (card, Normal, CastMethod::Normal, None),
            AnnounceAdventure { card } => (card, CastFace::Adventure, CastMethod::Normal, None),
            AnnounceOmen { card } => (card, CastFace::Omen, CastMethod::Normal, None),
            AnnounceBack { card } => (card, CastFace::Back, CastMethod::Normal, None),
            AnnounceRoom { card, door, .. } => (
                card,
                if door == RoomDoor::Left {
                    CastFace::RoomLeft
                } else {
                    CastFace::RoomRight
                },
                CastMethod::Normal,
                None,
            ),
            AnnounceSplit {
                card, half, method, ..
            } => (card, CastFace::from_split_half(half), method, None),
            AnnounceAirbend { card, face, .. } => (card, face, CastMethod::Airbend, None),
            AnnounceDisguise { card } | AnnounceDisguiseGraveyard { card, .. } => {
                (card, Normal, CastMethod::Disguise, None)
            }
            AnnounceFlashback { card } => (card, Normal, CastMethod::Flashback, None),
            AnnounceGrantedFlashback { card, .. } => {
                (card, Normal, CastMethod::GrantedFlashback, None)
            }
            AnnounceGrantedFixedFlashback { card, cost, .. } => (
                card,
                Normal,
                CastMethod::GrantedFixedFlashback(cost),
                Some(cost),
            ),
            AnnounceHarmonize { card } => (card, Normal, CastMethod::Harmonize, None),
            AnnounceGrantedHarmonize { card, face } => {
                (card, face, CastMethod::GrantedHarmonize, None)
            }
            AnnounceMayhem { card } => (card, Normal, CastMethod::Mayhem, None),
            AnnounceGrantedMayhem { card, face, .. } => {
                (card, face, CastMethod::GrantedMayhem, None)
            }
            AnnounceForetold { object, face, .. } => {
                (object.card, face, CastMethod::Foretell, None)
            }
            AnnounceWarp { card } => (card, Normal, CastMethod::Warp, None),
            AnnounceGrantedWarp {
                card, face, cost, ..
            } => (card, face, CastMethod::Warp, Some(cost)),
            AnnounceCastWithManaAlternative {
                object, face, cost, ..
            } => (object.card, face, CastMethod::Normal, Some(cost)),
            AnnounceWebSlinging {
                card, face, cost, ..
            } => (card, face, CastMethod::WebSlinging, Some(cost)),
            AnnounceBeamMeUp {
                card,
                face,
                cost,
                provider,
                ..
            } => (
                card,
                face,
                provider.map_or(CastMethod::BeamMeUp, CastMethod::BeamMeUpGranted),
                Some(cost),
            ),
            _ => return None,
        };
        Some(Self {
            card,
            face,
            method,
            cost,
        })
    }
}
