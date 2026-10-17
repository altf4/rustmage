//! Borrowed earlier-layer facts shared by actual and speculative mana worlds.
//! The overlay decides presence, control, counters and life; no cached ability
//! status from the real game is carried across a virtual sacrifice or payment.
use super::*;
use crate::cards::{LiveStatRecipients, LiveStaticEffect};

pub(super) trait CharacteristicWorld {
    fn game(&self) -> &Game;
    fn object(&self, id: CardId) -> &Object;
    fn life(&self, player: Player) -> i128;
    fn gained_life(&self, player: Player) -> bool;
    fn attacking(&self, id: CardId) -> bool;

    fn has_ability_removal(&self) -> bool {
        self.game().state.roles.as_ref().is_some_and(|state| {
            state.ability_removal_sources.iter().any(|&source| {
                self.object(source.card).zone == Zone::Battlefield
                    && self.reference(source.card) == source
            })
        })
    }
    fn printed_abilities_available(&self, id: CardId) -> bool {
        self.object(id).zone != Zone::Battlefield
            || ability_evaluation::evaluate_if_needed(self)
                .is_none_or(|evaluation| evaluation.result.objects[id as usize].printed)
    }

    fn reference(&self, id: CardId) -> ObjectRef {
        ObjectRef {
            card: id,
            generation: self.object(id).generation,
        }
    }
    fn kinds(&self, id: CardId) -> u16 {
        self.game().object_kinds(id, self.object(id))
    }
    fn subtypes(&self, id: CardId) -> GrantedSubtypes {
        self.game()
            .granted_subtypes_with(id, self.object(id), |source| Some(self.object(source)))
    }
    fn value(&self, value: StatValue, source: CardId) -> i128 {
        let game = self.game();
        let controller = self.object(source).controller;
        game.stat_value_with(
            value,
            self.reference(source),
            controller,
            &|| {
                game.state.objects.iter().enumerate().map(|(id, _)| {
                    let id = id as CardId;
                    (id, self.object(id))
                })
            },
            &|id| self.subtypes(id),
            &|id, object| game.object_kinds(id, object),
            self.life(controller),
            game.state.attack_declared && self.attacking(source),
        )
    }
    fn granted_mana(&self, id: CardId) -> Vec<ability_order::GrantedManaAbility> {
        if let Some(evaluation) = ability_evaluation::evaluate_if_needed(self) {
            return evaluation.result.objects[id as usize].mana.clone();
        }
        let mut result = Vec::new();
        for &source in &self.game().state.attachment_sources {
            let reference = self.reference(source);
            if self.kinds(source) & 1 != 0 || !self.attachment_link(reference, self.reference(id)) {
                continue;
            }
            for group in self.object(source).card.static_ability_groups() {
                for fragment in group.fragments {
                    if let crate::cards::StaticAbilityFragment::AttachmentMana(production) =
                        *fragment
                    {
                        result.push(ability_order::GrantedManaAbility {
                            source: reference,
                            production,
                        });
                    }
                }
            }
        }
        result
    }
    fn attachment_link(&self, source: ObjectRef, recipient: ObjectRef) -> bool {
        let object = self.object(source.card);
        object.zone == Zone::Battlefield
            && self.reference(source.card) == source
            && self.object(recipient.card).zone == Zone::Battlefield
            && self.reference(recipient.card) == recipient
            && object.attached_to == Some(recipient)
    }
    fn attached(&self, source: ObjectRef, recipient: CardId) -> bool {
        self.attachment_link(source, self.reference(recipient))
            && self.kinds(source.card) & 1 == 0
            && self.kinds(recipient) & 1 != 0
    }
    fn live_applies(&self, source: ObjectRef, effect: &LiveStaticEffect, id: CardId) -> bool {
        let target = self.object(id);
        let provider = self.object(source.card);
        if target.zone != Zone::Battlefield
            || self.kinds(id) & 1 == 0
            || provider.zone != Zone::Battlefield
            || self.reference(source.card) != source
        {
            return false;
        }
        let member = match effect.recipients {
            LiveStatRecipients::Attached => provider.attached_to == Some(self.reference(id)),
            LiveStatRecipients::Creatures {
                relation,
                other,
                enchanted_by_controller,
            } => {
                (match relation {
                    0 => target.controller == provider.controller,
                    1 => target.controller != provider.controller,
                    _ => true,
                }) && (!other || self.reference(id) != source)
                    && (!enchanted_by_controller
                        || self.game().state.attachment_sources.iter().any(|&a| {
                            let aura = self.object(a);
                            aura.zone == Zone::Battlefield
                                && aura.card.aura().is_some()
                                && aura.controller == provider.controller
                                && aura.attached_to == Some(self.reference(id))
                        }))
            }
        };
        member
            && effect.condition.is_none_or(|condition| {
                let matching = target.controller == provider.controller
                    && (target.card.has_subtype(condition.subtype)
                        || self.subtypes(id).contains(condition.subtype));
                matching != condition.negated
            })
    }
}
impl CharacteristicWorld for Game {
    fn game(&self) -> &Game {
        self
    }
    fn object(&self, id: CardId) -> &Object {
        &self.state.objects[id as usize]
    }
    fn life(&self, player: Player) -> i128 {
        self.state.players[player.index()].life
    }
    fn gained_life(&self, player: Player) -> bool {
        self.state.players[player.index()].life_gained_this_turn > 0
    }
    fn attacking(&self, id: CardId) -> bool {
        self.state
            .attackers
            .iter()
            .any(|attacker| attacker.card == id)
    }
}
