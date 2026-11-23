//! Reject only spell and ability proposals with a proven current payment shortage.
//!
//! Build the source and price summaries once per menu. Every recognized candidate
//! then checks its own typed cost options against those summaries. An unmodeled
//! cost, source, or way to contribute keeps the proposal available.
use super::*;
use payment_capacity::PaymentCapacity;
use proposal_price::ProposalPriceFacts;

pub(super) struct ProposalAffordability {
    capacity: PaymentCapacity,
    price: ProposalPriceFacts,
}

impl ProposalAffordability {
    pub(super) fn for_player(game: &Game, actor: Player) -> Option<Self> {
        Some(Self {
            capacity: PaymentCapacity::for_player(game, actor)?,
            price: ProposalPriceFacts::new(game, actor),
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
            .is_none_or(|cost| self.capacity.permits(cost.colored, cost.generic, None))
        };
        if let Some(cost) = spell.cost {
            return permits(cost);
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
