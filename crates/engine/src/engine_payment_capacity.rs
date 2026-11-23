//! Necessary mana conditions at priority. The quoted capacity is deliberately
//! generous: spending restrictions, life costs and many activation guards can
//! reduce the available mana. Unbounded or unrecognized funding paths abstain.
use super::*;
use crate::cards::{
    ActivationGuard, ActivationOrigin, BasicLandType, EntryChoiceValue, StaticAbilityFragment as F,
};

#[derive(Clone, Copy, Default)]
struct Source {
    colors: u8,
    maximum: u64,
}

impl Source {
    fn mode(&mut self, colors: u8, amount: u64) {
        if amount != 0 {
            self.colors |= colors;
            self.maximum = self.maximum.max(amount);
        }
    }

    fn production(&mut self, production: ManaProduction) -> Option<()> {
        match production.base() {
            ManaProduction::Fixed(mana) => {
                let mut mask = 0;
                for (index, &amount) in mana.iter().enumerate() {
                    if amount != 0 {
                        mask |= 1 << index;
                    }
                }
                self.mode(mask, mana.into_iter().map(u64::from).sum());
            }
            ManaProduction::Choice { mask, amount }
            | ManaProduction::Combination { mask, amount } => {
                self.mode(mask, u64::from(amount));
            }
            ManaProduction::ChosenColor => self.mode(31, 1),
            ManaProduction::ChosenColorAmount(amount) => self.mode(31, u64::from(amount)),
            _ => return None,
        }
        Some(())
    }
}

pub(super) struct PaymentCapacity {
    // A subset counts every physical source that could produce any of its colors once.
    // Generously grant that source's whole largest output to the subset.
    subset: [u128; 64],
    sources: Vec<Source>,
}

impl PaymentCapacity {
    pub(super) fn for_player(game: &Game, actor: Player) -> Option<Self> {
        let s = &game.state;
        if !s.exile_links.is_empty()
            || s.borrowed_links.is_some()
            || s.graveyard_borrowing.is_some()
            || s.self_granted_abilities.is_some()
            || s.temporary_abilities.is_some()
            || s.controls.is_some()
            || s.temporary_controls.is_some()
            || s.copy_recipes.is_some()
            || s.temporary_copies.is_some()
            || s.manifest.is_some()
            || s.earthbend_returns.is_some()
            || s.zone_rules.is_some()
            || s.replacements.is_some()
            || s.players[actor.index()].mountain_mana_bonus != 0
            || s.roles.as_ref().is_some_and(|r| {
                !r.resolved_ability_modifications.is_empty()
                    || !r.source_linked_effects.is_empty()
                    || !r.basic_land_types.is_empty()
                    || !r.animations.is_empty()
                    || !r.harnessed.is_empty()
                    || !r.melded_permanents.is_empty()
                    || !r.ability_removal_sources.is_empty()
                    || r.control_change_sources.is_some()
            })
        {
            return None;
        }

        let mut chosen = vec![0u8; s.objects.len()];
        if let Some(rows) = &s.entry_values {
            for row in rows.iter() {
                if let EntryChoiceValue::BasicLandType(kind) = row.value
                    && game.object_ref(row.object.card) == row.object
                {
                    chosen[row.object.card as usize] |= 1 << kind.index();
                }
            }
        }

        let mut land_types = 0u8;
        for (id, object) in s.objects.iter().enumerate() {
            if !matches!(object.zone, Zone::Battlefield | Zone::Command) {
                continue;
            }
            for group in object.card.static_ability_groups() {
                for &fragment in group.fragments {
                    // These effects cannot create, duplicate or untap a resource during
                    // funding. Mana-dependent stat recipes are intentionally unsupported.
                    if !matches!(
                        fragment,
                        F::CostReduction(..)
                            | F::PlotCostReduction(..)
                            | F::ChosenBasicLandType
                            | F::DeathBaronLord
                            | F::StaticModifier(..)
                            | F::ScalingModifier(..)
                            | F::IntrinsicKeywords(_)
                            | F::ControlledCreatureScaling { .. }
                            | F::ControlledLegendaryCreatureBonus(_)
                            | F::ControlledCreatureBonusPerSubtype { .. }
                            | F::CreatureGroupBonus { .. }
                            | F::CreatureGroupScaling { .. }
                            | F::SourceConditionalKeywords { .. }
                            | F::StartYourEngines
                    ) {
                        return None;
                    }
                }
            }
            if object.zone == Zone::Battlefield
                && object.controller == actor
                && object.card.is_land()
            {
                // ControlSubtypeAny can look at tapped lands as well.
                for color in object.card.printed_intrinsic_mana_colors() {
                    land_types |= 1 << color;
                }
                land_types |= chosen[id];
            }
        }

        let mut sources = vec![Source::default(); s.objects.len()];
        let mut histogram = [0u128; 64];
        for (id, object) in s.objects.iter().enumerate() {
            let controlled = object.zone == Zone::Battlefield && object.controller == actor;
            if !(controlled
                || (object.owner == actor
                    && matches!(
                        object.zone,
                        Zone::Battlefield | Zone::Hand | Zone::Graveyard
                    )))
            {
                // Never consult the identity of an opposing hand or library card.
                continue;
            }
            // Returning a double-faced card to hand or sacrificing it restores the
            // physical front. Its zone-activated mana rules can differ from this face.
            if object.owner == actor && object.zone == Zone::Battlefield && object.is_card() {
                let front = object
                    .card
                    .room_parent()
                    .or_else(|| object.card.split_parent())
                    .unwrap_or_else(|| object.card.front_face());
                if front != object.card
                    && front.activations().iter().any(|ability| {
                        ability.origin != ActivationOrigin::Battlefield
                            && ability.mana_production().is_some()
                    })
                {
                    return None;
                }
            }
            let source = &mut sources[id];
            if controlled && !object.tapped {
                for color in object
                    .card
                    .printed_intrinsic_mana_colors()
                    .chain(object.card.mana_color())
                {
                    source.mode(1 << color, 1);
                }
                source.mode(chosen[id], u64::from(chosen[id] != 0));
            }
            for ability in object.card.activations() {
                let Some(production) = ability.mana_production() else {
                    continue;
                };
                if ability.origin != ActivationOrigin::Battlefield {
                    // Payment costs can move an owned permanent to the hand or graveyard.
                    return None;
                }
                if !controlled {
                    continue;
                }
                // A tapped permanent can still activate a non-tap mana ability, e.g.
                // sacrificing an Eldrazi Spawn. Its production needs a different bound.
                if !ability.cost.tap {
                    return None;
                }
                if object.tapped {
                    continue;
                }
                // Extra prerequisites only reduce capacity. Effects beyond producing mana
                // can change what other sources may do during the payment window.
                if !ability.cost.alternatives.is_empty()
                    || !matches!(ability.program, [Instruction::AddMana(_)])
                    || ability.cost.additional.is_some_and(|cost| {
                        !matches!(cost.kind, EffectChoiceKind::TapPermanents(_))
                    })
                {
                    return None;
                }
                if let ActivationGuard::ControlSubtypeAny(types) = ability.guard
                    && !types.iter().any(|subtype| {
                        BasicLandType::ALL
                            .into_iter()
                            .find(|kind| kind.name() == *subtype)
                            .is_none_or(|kind| land_types & (1 << kind.index()) != 0)
                    })
                {
                    continue;
                }
                source.production(production)?;
            }
            histogram[source.colors as usize] += u128::from(source.maximum);
        }

        let mut pool = s.players[actor.index()].mana.map(u64::from);
        pool[3] += u64::from(s.players[actor.index()].combat_red_mana);
        let mut subset = [0u128; 64];
        // Six colors give a fixed-size subset-sum transform, independent of board size.
        for bit in 0..6 {
            for mask in 0..64 {
                if mask & (1 << bit) != 0 {
                    histogram[mask] += histogram[mask ^ (1 << bit)];
                    subset[mask] += u128::from(pool[bit]);
                }
            }
        }
        for (mask, upper) in subset.iter_mut().enumerate() {
            *upper += histogram[63] - histogram[63 ^ mask];
        }
        Some(Self { subset, sources })
    }

    pub(super) fn permits(
        &self,
        colored: Mana,
        generic: u32,
        reserved_tap: Option<CardId>,
    ) -> bool {
        let reserved = reserved_tap.map_or(Source::default(), |id| self.sources[id as usize]);
        let upper = |mask: usize| {
            self.subset[mask]
                - if usize::from(reserved.colors) & mask != 0 {
                    u128::from(reserved.maximum)
                } else {
                    0
                }
        };
        let mut colors = 0;
        let mut total = u128::from(generic);
        for (bit, &amount) in colored.iter().enumerate() {
            total += u128::from(amount);
            if amount != 0 {
                colors |= 1usize << bit;
            }
        }
        if total > upper(63) {
            return false;
        }
        let mut mask = colors;
        while mask != 0 {
            let needed: u128 = colored
                .iter()
                .enumerate()
                .filter(|(i, _)| mask & (1 << i) != 0)
                .map(|(_, amount)| u128::from(*amount))
                .sum();
            if needed > upper(mask) {
                return false;
            }
            mask = (mask - 1) & colors;
        }
        true
    }
}
