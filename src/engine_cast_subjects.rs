//! A cast trigger's frozen affected set is independent of its own targets.
use super::*;

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct CastCreatureSubjectsView {
    pub spell: u64,
    pub objects: Vec<ObjectRef>,
}

impl Game {
    pub(super) fn capture_cast_creature_subjects(&mut self, stack: Option<usize>, first: usize) {
        if !self.state.pending_triggers[first..].iter().any(|item| {
            item.program().is_some_and(|(_, program)| {
                program.iter().any(|instruction| {
                    matches!(instruction, Instruction::GrantCastCreatureKeywords(_))
                })
            })
        }) {
            return;
        }
        let Some(spell) = stack.map(|index| &self.state.stack[index]) else {
            return;
        };
        let mut objects = Vec::new();
        for bound in &spell.targets {
            let (Target::Card(id), Some(reference)) = (bound.target, bound.object) else {
                continue;
            };
            if reference.card == id
                && self.on_battlefield(id)
                && self.object_ref(id) == reference
                && self.is_creature(id)
                && !objects.contains(&reference)
            {
                objects.push(reference);
            }
        }
        let event = CastCreatureSubjectsView {
            spell: spell.id,
            objects,
        };
        for item in &mut self.state.pending_triggers[first..] {
            if item.program().is_some_and(|(_, program)| {
                program.iter().any(|instruction| {
                    matches!(instruction, Instruction::GrantCastCreatureKeywords(_))
                })
            }) {
                item.cast_creature_subjects = Some(Box::new(event.clone()));
            }
        }
    }

    pub(super) fn grant_cast_creature_keywords(&mut self, item: &SpellView, keywords: Keywords) {
        let Some(source) = item.source else {
            return;
        };
        let Some(event) = &item.cast_creature_subjects else {
            return;
        };
        let ids: Vec<_> = event
            .objects
            .iter()
            .filter(|reference| {
                self.on_battlefield(reference.card)
                    && self.object_ref(reference.card) == **reference
            })
            .map(|reference| reference.card)
            .collect();
        self.grant_keyword_effects(&ids, source.object, keywords);
    }

    fn flying_combat_rule(&self, source: CardId) -> bool {
        self.on_battlefield(source)
            && self.state.objects[source as usize]
                .card
                .static_ability_groups()
                .iter()
                .any(|group| {
                    group.fragments.iter().any(|fragment| {
                        matches!(
                            fragment,
                            crate::cards::StaticAbilityFragment::FlyingCombatRestriction
                        )
                    })
                })
            && self.continuous_fragment_available(source, |fragment| {
                matches!(
                    fragment,
                    crate::cards::StaticAbilityFragment::FlyingCombatRestriction
                )
            })
    }

    pub(super) fn can_attack_defender(&self, attacker: CardId, defender: CombatDefender) -> bool {
        if let CombatDefender::Planeswalker(walker) | CombatDefender::Battle(walker) = defender {
            let object = &self.state.objects[walker.card as usize];
            if object.zone == Zone::Battlefield
                && self.object_ref(walker.card) == walker
                && object.attached_to.is_some_and(|recipient| {
                    self.on_battlefield(recipient.card)
                        && self.object_ref(recipient.card) == recipient
                        && self.is_creature(recipient.card)
                })
                && object.card.static_ability_groups().iter().any(|group| {
                    group.fragments.iter().any(|fragment| {
                        matches!(fragment,
                        crate::cards::StaticAbilityFragment::AttachedCreaturePreventsSourceAttack)
                    })
                })
                && self.continuous_fragment_available(walker.card, |fragment| {
                    matches!(
                        fragment,
                        crate::cards::StaticAbilityFragment::AttachedCreaturePreventsSourceAttack
                    )
                })
            {
                return false;
            }
        }
        let CombatDefender::Player(player) = defender else {
            return true;
        };
        !self.keywords(attacker).contains(Keywords::FLYING)
            || !self.state.static_sources.iter().any(|&source| {
                self.state.objects[source as usize].controller == player
                    && self.flying_combat_rule(source)
            })
    }

    pub(super) fn can_attack_any_defender(&self, actor: Player, attacker: CardId) -> bool {
        self.can_attack_defender(attacker, CombatDefender::Player(actor.other()))
            || self.state.objects.iter().enumerate().any(|(id, _)| {
                let reference = self.object_ref(id as CardId);
                self.battle_can_be_attacked_by(reference, actor)
                    && self.can_attack_defender(attacker, CombatDefender::Battle(reference))
            })
            || self.state.objects.iter().enumerate().any(|(id, object)| {
                object.zone == Zone::Battlefield
                    && object.controller == actor.other()
                    && self.current_kinds(id as CardId) & 64 != 0
                    && self.can_attack_defender(
                        attacker,
                        CombatDefender::Planeswalker(self.object_ref(id as CardId)),
                    )
            })
    }

    pub(super) fn flying_block_forbidden(&self, blocker: CardId, attacker: CardId) -> bool {
        self.keywords(blocker).contains(Keywords::FLYING)
            && self.state.static_sources.iter().any(|&source| {
                self.state.objects[source as usize].controller
                    == self.state.objects[attacker as usize].controller
                    && self.flying_combat_rule(source)
            })
    }
}
