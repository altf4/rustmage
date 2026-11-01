//! Continuous restrictions are checked when declaring attackers and blockers.
use super::*;
#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) struct AttackPayment {
    pub player: Player,
    pub generic: u32,
    pub attackers: Vec<ObjectRef>,
}
#[derive(Clone, Debug, Serialize)]
pub struct AttackPaymentView {
    pub player: Player,
    pub generic: u32,
}
impl AttackPayment {
    pub(super) fn bytes(&self) -> usize {
        std::mem::size_of::<Self>() + self.attackers.len() * std::mem::size_of::<ObjectRef>()
    }
}
/// Cardinality of a capacity-one defender assignment. Distinct edge sets are
/// necessary when separate restrictions affect different attackers.
pub(super) fn maximum_attack_matching(edges: &[Vec<usize>], slots: usize) -> usize {
    fn augment(
        i: usize,
        edges: &[Vec<usize>],
        seen: &mut [bool],
        owners: &mut [Option<usize>],
    ) -> bool {
        for &slot in &edges[i] {
            if seen[slot] {
                continue;
            }
            seen[slot] = true;
            if owners[slot].is_none() || augment(owners[slot].unwrap(), edges, seen, owners) {
                owners[slot] = Some(i);
                return true;
            }
        }
        false
    }
    let mut owners = vec![None; slots];
    for i in 0..edges.len() {
        augment(i, edges, &mut vec![false; slots], &mut owners);
    }
    owners.iter().filter(|owner| owner.is_some()).count()
}
impl Game {
    /// Costs are attached to attacking a player, not their planeswalkers.
    pub(super) fn attack_controller_tax(&self, defender: Player) -> u32 {
        self.state.static_sources.iter().copied().filter(|&id| {
            self.on_battlefield(id) && self.state.objects[id as usize].controller == defender
        }).map(|id| {
            self.state.objects[id as usize].card.static_ability_groups().iter().flat_map(|group| group.fragments.iter()).filter_map(|fragment| {
                let crate::cards::StaticAbilityFragment::AttackControllerTax { amount, condition } = *fragment else { return None; };
                (self.continuous_fragment_available(id, |f| matches!(f, crate::cards::StaticAbilityFragment::AttackControllerTax { amount: a, condition: c } if *a == amount && *c == condition))
                    && self.stat_value(condition, self.object_ref(id), defender) != 0).then_some(amount)
            }).fold(0u32, |a, b| a.checked_add(b).capacity("attack tax exhausted"))
        }).fold(0u32, |a, b| a.checked_add(b).capacity("attack tax exhausted"))
    }
    pub(super) fn selected_attack_tax(&self) -> u32 {
        self.state
            .selected_attackers
            .iter()
            .filter_map(|attack| match attack.defender {
                Some(CombatDefender::Player(p)) => Some(self.attack_controller_tax(p)),
                _ => None,
            })
            .fold(0u32, |a, b| {
                a.checked_add(b).capacity("attack tax exhausted")
            })
    }

    pub(super) fn tap_selected_attackers(&mut self) {
        let ids: Vec<_> = self
            .state
            .selected_attackers
            .iter()
            .filter(|a| !self.keywords(a.card).contains(Keywords::VIGILANCE))
            .map(|a| a.card)
            .collect();
        self.tap_objects(&ids);
    }
    /// A printed provider may become active as a result of tapping attackers.
    pub(super) fn has_attack_tax_provider(&self) -> bool {
        self.state.static_sources.iter().any(|&id| {
            self.on_battlefield(id)
                && self.state.objects[id as usize]
                    .card
                    .static_ability_groups()
                    .iter()
                    .flat_map(|group| group.fragments.iter())
                    .any(|fragment| {
                        matches!(
                            fragment,
                            crate::cards::StaticAbilityFragment::AttackControllerTax { .. }
                        )
                    })
        })
    }
    pub(super) fn after_attacker_taps<T>(&self, f: impl FnOnce(&Game) -> T) -> T {
        let mut preview = self.clone();
        preview.tap_selected_attackers();
        preview.refresh_enduring_stories();
        f(&preview)
    }
    pub(super) fn projected_attack_tax(&self) -> u32 {
        if !self.has_attack_tax_provider() {
            return 0;
        }
        self.after_attacker_taps(|preview| preview.selected_attack_tax())
    }
    pub(super) fn attack_declaration_fundable(&self, actor: Player) -> bool {
        if !self.has_attack_tax_provider() {
            return true;
        }
        self.after_attacker_taps(|preview| {
            preview.attack_tax_fundable(actor, preview.selected_attack_tax())
        })
    }
    /// Shared eligibility for declaration candidates and mandatory-attack checks.
    /// Selection does not tap an attacker; costs and tapping happen at commit.
    pub(super) fn can_declare_attacker(&self, actor: Player, id: CardId) -> bool {
        let Some(object) = self.state.objects.get(id as usize) else {
            return false;
        };
        if object.controller != actor
            || object.zone != Zone::Battlefield
            || !self.is_creature(id)
            || self.current_kinds(id) & crate::cards::BATTLE_KIND != 0
            || object.tapped
        {
            return false;
        }
        let keywords = self.keywords(id);
        (!object.summoning_sick || keywords.contains(Keywords::HASTE))
            && (!keywords.contains(Keywords::DEFENDER)
                || self.state.static_sources.iter().copied().any(|source| {
                    self.on_battlefield(source)
                        && self.state.objects[source as usize].controller == actor
                        && self.state.objects[source as usize].card.static_ability_groups().iter()
                            .any(|group| group.fragments.iter().any(|fragment| matches!(fragment,
                                crate::cards::StaticAbilityFragment::ControlledAttackWithoutDefender)))
                        && self.continuous_fragment_available(source, |fragment| matches!(fragment,
                            crate::cards::StaticAbilityFragment::ControlledAttackWithoutDefender))
                })
                || (self.state.players[actor.index()].scried_or_surveilled_this_turn
                    && object.card.static_ability_groups().iter().any(|group| {
                        group.fragments.iter().any(|fragment| {
                            matches!(
                                fragment,
                                crate::cards::StaticAbilityFragment::AttackWithoutDefenderIfScried
                            )
                        })
                    })
                    && self.continuous_fragment_available(id, |fragment| {
                        matches!(
                            fragment,
                            crate::cards::StaticAbilityFragment::AttackWithoutDefenderIfScried
                        )
                    })))
            && !self.combat_restricted(id, true)
    }

    pub(super) fn attacks_each_combat(&self, id: CardId) -> bool {
        if self.on_battlefield(id)
            && self
                .state
                .self_granted_abilities
                .as_ref()
                .is_some_and(|rows| {
                    rows.iter().any(|row| {
                        row.recipient == self.object_ref(id)
                            && row.kind == SelfGrantedAbilityKind::AttacksEachCombat
                            && row.expires_turn.is_none_or(|turn| turn == self.state.turn)
                            && self
                                .characteristic_query()
                                .self_grant_available(row.recipient, row.timestamp)
                    })
                })
        {
            return true;
        }
        self.on_battlefield(id)
            && self.state.objects[id as usize]
                .card
                .static_ability_groups()
                .iter()
                .any(|group| {
                    group.fragments.iter().any(|fragment| {
                        matches!(
                            fragment,
                            crate::cards::StaticAbilityFragment::AttacksEachCombat
                        )
                    })
                })
            && self.continuous_fragment_available(id, |fragment| {
                matches!(
                    fragment,
                    crate::cards::StaticAbilityFragment::AttacksEachCombat
                )
            })
    }

    /// A partial declaration may need to be revised after consuming a limited slot.
    pub(super) fn has_planeswalker_attack_limit_provider(&self) -> bool {
        self.state.static_sources.iter().any(|&id| {
            self.on_battlefield(id) && self.state.objects[id as usize].card
                .static_ability_groups().iter().any(|group| group.fragments.iter().any(|fragment|
                    matches!(fragment, crate::cards::StaticAbilityFragment::ControlledPlaneswalkerAttackLimit)))
        })
    }

    /// A layer-six grant on this exact planeswalker limits its declared attackers.
    pub(super) fn planeswalker_attack_limited(&self, walker: ObjectRef) -> bool {
        let Some(object) = self.state.objects.get(walker.card as usize) else {
            return false;
        };
        if object.zone != Zone::Battlefield
            || self.object_ref(walker.card) != walker
            || self.current_kinds(walker.card) & 64 == 0
        {
            return false;
        }
        let view = self.object_view(walker.card);
        self.state.static_sources.iter().copied().any(|provider| {
            let giver = &self.state.objects[provider as usize];
            giver.zone == Zone::Battlefield
                && giver.controller == object.controller
                && giver.card.static_ability_groups().iter().any(|g| {
                    g.fragments.iter().any(|f| {
                        matches!(
                            f,
                            crate::cards::StaticAbilityFragment::ControlledPlaneswalkerAttackLimit
                        )
                    })
                })
                && view.static_grant_available(
                    self.object_ref(provider),
                    giver.card,
                    ability_evaluation::GrantedStaticRule::PlaneswalkerAttackLimit,
                )
        })
    }
    pub(super) fn attack_slot_free(&self, defender: ObjectRef, except: Option<CardId>) -> bool {
        !self.planeswalker_attack_limited(defender)
            || !self.state.selected_attackers.iter().any(|attack| {
                Some(attack.card) != except
                    && attack.defender == Some(CombatDefender::Planeswalker(defender))
            })
    }
    /// Maximum number of simultaneously satisfiable attack requirements. Unlimited
    /// defenders need no matching; limited planeswalkers are capacity-one slots.
    fn maximum_mandatory_attackers(&self, actor: Player) -> usize {
        let walkers: Vec<_> = self
            .state
            .objects
            .iter()
            .enumerate()
            .filter_map(|(i, o)| {
                let id = i as CardId;
                (o.zone == Zone::Battlefield
                    && o.controller == actor.other()
                    && self.current_kinds(id) & 64 != 0)
                    .then_some(self.object_ref(id))
            })
            .collect();
        let mut free = 0;
        let mut edges = Vec::new();
        for &id in &self.state.static_sources {
            if !self.attacks_each_combat(id) || !self.can_declare_attacker(actor, id) {
                continue;
            }
            // A voluntarily selected taxable attack is already paid for purposes
            // of maximizing the remaining requirements; no unchosen attacker is
            // compelled to pay the tax.
            let selected_paid = self.attack_controller_tax(actor.other()) != 0
                && self.state.selected_attackers.iter().any(|a| {
                    a.card == id && a.defender == Some(CombatDefender::Player(actor.other()))
                });
            let player_free = selected_paid
                || self.attack_controller_tax(actor.other()) == 0
                    && self.can_attack_defender(id, CombatDefender::Player(actor.other()));
            let battle_free = self.state.objects.iter().enumerate().any(|(n, _)| {
                let r = self.object_ref(n as CardId);
                self.battle_can_be_attacked_by(r, actor)
                    && self.can_attack_defender(id, CombatDefender::Battle(r))
            });
            let walker_free = walkers.iter().any(|&r| {
                !self.planeswalker_attack_limited(r)
                    && self.can_attack_defender(id, CombatDefender::Planeswalker(r))
            });
            if player_free || battle_free || walker_free {
                free += 1;
                continue;
            }
            edges.push(
                walkers
                    .iter()
                    .enumerate()
                    .filter_map(|(n, &r)| {
                        (self.planeswalker_attack_limited(r)
                            && self.can_attack_defender(id, CombatDefender::Planeswalker(r)))
                        .then_some(n)
                    })
                    .collect::<Vec<_>>(),
            );
        }
        free + maximum_attack_matching(&edges, walkers.len())
    }

    /// Every mandatory attacker must attack if at least one legal defender exists.
    /// Defender-specific restrictions do not create a requirement to attack a player.
    pub(super) fn attack_declaration_complete(&self, actor: Player) -> bool {
        let chosen = &self.state.selected_attackers;
        for (index, attacker) in chosen.iter().enumerate() {
            if !self.can_declare_attacker(actor, attacker.card)
                || chosen[..index]
                    .iter()
                    .any(|other| other.card == attacker.card)
                || attacker
                    .defender
                    .is_none_or(|defender| !self.can_attack_defender(attacker.card, defender))
                || !match attacker.defender {
                    Some(CombatDefender::Player(player)) => player == actor.other(),
                    Some(CombatDefender::Planeswalker(reference)) => {
                        self.attack_slot_free(reference, Some(attacker.card))
                            && self.state.objects.get(reference.card as usize).is_some_and(
                                |object| {
                                    object.zone == Zone::Battlefield
                                        && object.generation == reference.generation
                                        && object.controller == actor.other()
                                        && self.current_kinds(reference.card) & 64 != 0
                                },
                            )
                    }
                    Some(CombatDefender::Battle(reference)) => {
                        self.battle_can_be_attacked_by(reference, actor)
                    }
                    None => false,
                }
            {
                return false;
            }
        }
        if !self.has_planeswalker_attack_limit_provider() {
            return self.state.static_sources.iter().all(|&id| {
                !self.attacks_each_combat(id)
                    || !self.can_declare_attacker(actor, id)
                    || !(if self.attack_controller_tax(actor.other()) == 0 {
                        self.can_attack_any_defender(actor, id)
                    } else {
                        self.state.objects.iter().enumerate().any(|(n, o)| {
                            let reference = self.object_ref(n as CardId);
                            (o.zone == Zone::Battlefield
                                && o.controller == actor.other()
                                && self.current_kinds(n as CardId) & 64 != 0
                                && self.can_attack_defender(
                                    id,
                                    CombatDefender::Planeswalker(reference),
                                ))
                                || (self.battle_can_be_attacked_by(reference, actor)
                                    && self
                                        .can_attack_defender(id, CombatDefender::Battle(reference)))
                        })
                    })
                    || chosen.iter().any(|attacker| attacker.card == id)
            });
        }
        let fulfilled = chosen
            .iter()
            .filter(|attack| self.attacks_each_combat(attack.card))
            .count();
        fulfilled >= self.maximum_mandatory_attackers(actor)
    }

    fn combat_restriction_from(&self, source: CardId, subject: CardId, attack: bool) -> bool {
        let object = &self.state.objects[source as usize];
        let Some(rule) = object.card.combat_restriction() else {
            return false;
        };
        object.zone == Zone::Battlefield
            && self.continuous_fragment_available(source, |fragment| {
                matches!(
                    fragment,
                    crate::cards::StaticAbilityFragment::CombatRestriction(..)
                )
            })
            && if attack { rule.attack } else { rule.block }
            && match rule.subject {
                Subject::Source => source == subject,
                Subject::Attached => object.attached_to == Some(self.object_ref(subject)),
                _ => false,
            }
            && rule.unless.is_none_or(|guard| {
                !self.step_guard(
                    guard,
                    object.controller,
                    self.object_ref(source),
                    object.tapped,
                )
            })
    }
    pub(super) fn combat_restricted(&self, id: CardId, attack: bool) -> bool {
        self.combat_restriction_from(id, id, attack)
            || self
                .state
                .attachment_sources
                .iter()
                .any(|&source| self.combat_restriction_from(source, id, attack))
    }
}
