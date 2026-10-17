//! Exact-source resolving endure choices; the continuation is part of Pending.
use super::*;

impl Game {
    pub(super) fn choose_endure(&mut self, put_counters: bool) {
        let Pending::Endure {
            player,
            source,
            amount,
            token,
        } = self.state.pending
        else {
            unreachable!()
        };
        // No action other than the actor's choice or concession can interleave
        // while resolution is suspended. The source was validated at creation.
        assert!(self.on_battlefield(source.card) && self.object_ref(source.card) == source);
        if put_counters {
            self.effect_counter(
                source.card,
                CounterKind::PlusOne,
                u64::from(amount),
                replacements::ResumeMode::Program,
            );
        } else {
            self.create_tokens(player, token, 1, false);
        }
        self.run_program();
    }
}

#[cfg(test)]
mod tests {
    use super::super::tests::{scenario, spawn};
    use super::*;
    const CARDS: [(&str, u16); 8] = [
        ("Anafenza, Unyielding Lineage", 2),
        ("Descendant of Storms", 1),
        ("Dusyut Earthcarver", 3),
        ("Fortress Kin-Guard", 1),
        ("Inspirited Vanguard", 2),
        ("Kin-Tree Nurturer", 1),
        ("Sandskitter Outrider", 2),
        ("Sinkhole Surveyor", 1),
    ];
    fn replay(g: &mut Game, action: Action) {
        let actor = g.actor().unwrap();
        let before = g.snapshot();
        let revision = g.revision();
        let mut child = g.clone();
        child.apply(actor, action.clone()).unwrap();
        g.apply(actor, action.clone()).unwrap();
        assert_eq!(g.snapshot(), child.snapshot());
        let after = g.snapshot();
        g.restore(&before).unwrap();
        assert_eq!(g.apply_index(actor, revision, 0), Err(Error::StaleRevision));
        g.apply(actor, action).unwrap();
        assert_eq!(g.snapshot(), after);
    }
    fn drive(g: &mut Game, stop: bool, counters: bool, pay: bool) {
        for _ in 0..80 {
            if g.state.resolving.is_none()
                && g.state.stack.is_empty()
                && g.state.announcing.is_none()
                && g.state.pending_triggers.is_empty()
                && g.state.pending == Pending::Priority
            {
                return;
            }
            if stop && matches!(g.state.pending, Pending::Endure { .. }) {
                return;
            }
            let actions = g.legal_actions();
            let action = match g.state.pending {
                Pending::Endure { .. } => Some(Action::ChooseEndure {
                    put_counters: counters,
                }),
                Pending::ResolutionMana { .. } if !pay => Some(Action::DeclineResolutionMana),
                _ => actions
                    .iter()
                    .find(|a| {
                        matches!(
                            a,
                            Action::PayCosts { .. }
                                | Action::PayResolutionMana { .. }
                                | Action::Pass
                                | Action::PutTrigger { .. }
                                | Action::FinishTargets
                        )
                    })
                    .cloned(),
            }
            .unwrap_or_else(|| panic!("unhandled {:?}: {actions:?}", g.state.pending));
            replay(g, action);
        }
        panic!("resolution failed to finish");
    }
    fn cast(name: &str, p: Player, stop: bool, counters: bool) -> (Game, CardId) {
        let mut g = scenario();
        g.state.active = p;
        g.state.priority = p;
        let card = Card::from_name(name).unwrap();
        let cost = card.cost_options().next().unwrap();
        let mut payment = cost.colored;
        payment[5] += cost.generic;
        g.state.players[p.index()].mana = payment;
        let source = spawn(&mut g, p, card, Zone::Hand);
        replay(
            &mut g,
            Action::Cast {
                card: source,
                target: None,
                payment,
            },
        );
        drive(&mut g, stop, counters, true);
        assert!(g.on_battlefield(source));
        (g, source)
    }
    fn attack(g: &mut Game, source: CardId) {
        let p = g.state.objects[source as usize].owner;
        g.state.objects[source as usize].summoning_sick = false;
        g.state.active = p;
        g.state.priority = p;
        g.state.step = Step::Attackers;
        g.state.pending = Pending::Attackers;
        replay(g, Action::Attack { card: source });
        replay(g, Action::FinishAttackers);
    }
    fn trigger(g: &mut Game, source: CardId, name: &str) {
        let p = g.state.objects[source as usize].owner;
        if name.starts_with("Anafenza") {
            let other = spawn(g, p, Card::GrizzlyBears, Zone::Battlefield);
            g.move_object(other, Zone::Graveyard);
            g.give_priority();
        } else if name == "Descendant of Storms" || name == "Sinkhole Surveyor" {
            g.state.players[p.index()].mana = [1, 0, 0, 0, 0, 1];
            attack(g, source);
        }
    }
    fn spirits(g: &Game, p: Player) -> Vec<ObjectView> {
        g.state
            .objects
            .iter()
            .enumerate()
            .filter(|(_, o)| {
                o.owner == p
                    && o.zone == Zone::Battlefield
                    && o.is_token()
                    && o.card.has_subtype("Spirit")
            })
            .map(|(id, _)| g.object_view(id as CardId))
            .collect()
    }
    #[test]
    fn all_eight_real_paid_bodies_both_seats_and_both_complete_results() {
        for (name, amount) in CARDS {
            for p in [Player::P0, Player::P1] {
                for counters in [false, true] {
                    let (mut g, source) = cast(name, p, true, counters);
                    trigger(&mut g, source, name);
                    drive(&mut g, true, counters, true);
                    assert!(matches!(g.state.pending, Pending::Endure { .. }), "{name}");
                    let view = g.observe(p).endure_choice.unwrap();
                    assert_eq!((view.source, view.amount), (g.object_ref(source), amount));
                    assert!(g.observe(p.other()).endure_choice.is_none());
                    let before = g.snapshot();
                    assert!(
                        g.apply(
                            p.other(),
                            Action::ChooseEndure {
                                put_counters: counters
                            }
                        )
                        .is_err()
                    );
                    assert_eq!(g.snapshot(), before);
                    replay(
                        &mut g,
                        Action::ChooseEndure {
                            put_counters: counters,
                        },
                    );
                    drive(&mut g, false, counters, true);
                    assert_eq!(
                        g.state.objects[source as usize].counters.plus_one,
                        if counters { u64::from(amount) } else { 0 }
                    );
                    let tokens = spirits(&g, p);
                    assert_eq!(tokens.len(), usize::from(!counters));
                    if let Some(token) = tokens.first() {
                        assert_eq!(
                            (token.power, token.toughness),
                            (i128::from(amount), i128::from(amount))
                        );
                        assert_eq!(token.card.colors(), 1);
                        assert!(!token.keywords.contains(Keywords::FLYING));
                    }
                    if name == "Sinkhole Surveyor" {
                        assert_eq!(g.state.players[p.index()].life, 19);
                    }
                }
            }
        }
    }
    #[test]
    fn departed_and_reentered_source_automatically_creates_token_without_counter_choice() {
        for p in [Player::P0, Player::P1] {
            let (mut g, source) = cast("Descendant of Storms", p, false, false);
            g.state.players[p.index()].mana = [1, 0, 0, 0, 0, 1];
            attack(&mut g, source);
            let old = g.object_ref(source);
            g.move_object(source, Zone::Hand);
            g.move_object(source, Zone::Battlefield);
            assert_ne!(g.object_ref(source), old);
            drive(&mut g, true, true, true);
            assert!(!matches!(g.state.pending, Pending::Endure { .. }));
            assert_eq!(spirits(&g, p).len(), 1);
            assert_eq!(g.object_view(source).counters.plus_one, 0);
        }
    }
    #[test]
    fn optional_real_colored_payment_can_decline_and_never_counters_a_stack_item() {
        for p in [Player::P0, Player::P1] {
            let (mut g, source) = cast("Descendant of Storms", p, false, false);
            g.state.players[p.index()].mana = [0; 6];
            attack(&mut g, source);
            // Its genuine mana window exists even though Endure's choice has none.
            for _ in 0..10 {
                if matches!(g.state.pending, Pending::ResolutionMana { .. }) {
                    break;
                }
                replay(&mut g, Action::Pass);
            }
            let request = g.observe(p).resolution_payment.unwrap();
            assert_eq!(request.victim, None);
            assert_eq!(request.cost.colored, [1, 0, 0, 0, 0, 0]);
            assert_eq!(request.cost.generic, 1);
            assert!(
                !g.legal_actions()
                    .iter()
                    .any(|a| matches!(a, Action::PayResolutionMana { .. }))
            );
            replay(&mut g, Action::DeclineResolutionMana);
            drive(&mut g, false, true, false);
            assert_eq!(g.object_view(source).counters.plus_one, 0);
            assert!(spirits(&g, p).is_empty());
        }
    }
    #[test]
    fn late_counter_overflow_restores_owned_choice_and_token_branch_remains_legal() {
        let (mut g, source) = cast("Dusyut Earthcarver", Player::P0, true, false);
        g.state.objects[source as usize].counters.plus_one = u64::MAX - 1;
        let before = g.snapshot();
        let revision = g.revision();
        assert!(matches!(
            g.apply(Player::P0, Action::ChooseEndure { put_counters: true }),
            Err(Error::CapacityExceeded(_))
        ));
        assert_eq!(g.snapshot(), before);
        assert_eq!(g.revision(), revision);
        let mut child = g.clone();
        drop(g);
        replay(
            &mut child,
            Action::ChooseEndure {
                put_counters: false,
            },
        );
        assert_eq!(spirits(&child, Player::P0).len(), 1);
    }
    #[test]
    fn sinkhole_life_loss_precedes_private_choice_and_state_actions_wait_for_resolution() {
        let (mut g, source) = cast("Sinkhole Surveyor", Player::P0, false, false);
        g.state.players[0].life = 1;
        attack(&mut g, source);
        drive(&mut g, true, false, true);
        assert_eq!(g.state.players[0].life, 0);
        assert!(g.state.outcome.is_none());
        assert!(matches!(g.state.pending, Pending::Endure { .. }));
        replay(
            &mut g,
            Action::ChooseEndure {
                put_counters: false,
            },
        );
        assert!(g.state.outcome.is_some());
        assert_eq!(spirits(&g, Player::P0).len(), 1);
    }
    #[test]
    fn anafenza_ignores_tokens_and_self_but_observes_other_simultaneous_nontoken_deaths() {
        for p in [Player::P0, Player::P1] {
            for simultaneous in [false, true] {
                let (mut g, source) = cast("Anafenza, Unyielding Lineage", p, false, false);
                let token = Card::catalogue()
                    .find(|c| {
                        c.is_token()
                            && c.has_subtype("Spirit")
                            && c.effect() == Effect::Creature(1, 1)
                    })
                    .unwrap();
                let token_id = spawn(&mut g, p, token, Zone::Battlefield);
                g.sacrifice_batch(&[(p, token_id)]);
                g.give_priority();
                drive(&mut g, false, true, true);
                assert_eq!(g.object_view(source).counters.plus_one, 0);
                assert!(g.state.stack.is_empty());
                if simultaneous {
                    let bear = spawn(&mut g, p, Card::GrizzlyBears, Zone::Battlefield);
                    g.sacrifice_batch(&[(p, source), (p, bear)]);
                } else {
                    g.sacrifice_batch(&[(p, source)]);
                }
                g.give_priority();
                drive(&mut g, false, true, true);
                assert_eq!(spirits(&g, p).len(), usize::from(simultaneous));
                if simultaneous {
                    assert_eq!(spirits(&g, p)[0].power, 2);
                }
            }
        }
    }
    #[test]
    fn vanguard_repeats_the_whole_action_on_actual_attack_and_zero_endure_is_inert() {
        for p in [Player::P0, Player::P1] {
            let (mut g, source) = cast("Inspirited Vanguard", p, false, false);
            assert_eq!(spirits(&g, p).len(), 1);
            attack(&mut g, source);
            drive(&mut g, true, true, true);
            replay(&mut g, Action::ChooseEndure { put_counters: true });
            drive(&mut g, false, true, true);
            assert_eq!(g.object_view(source).counters.plus_one, 2);
            assert_eq!(spirits(&g, p).len(), 1);
            let (mut g, source) = cast("Fortress Kin-Guard", p, true, false);
            const ZERO: &[Instruction] = &[Instruction::Endure {
                amount: 0,
                token: Card::Forest,
            }];
            let resolution = g.state.resolving.as_mut().unwrap();
            resolution.program = ZERO;
            resolution.pc = 0;
            g.run_program();
            assert!(g.observe(p).endure_choice.is_none());
            assert_eq!(g.object_view(source).counters.plus_one, 0);
            assert!(spirits(&g, p).is_empty());
        }
    }
    #[test]
    fn late_token_entry_observer_allocation_failure_restores_entire_choice() {
        for p in [Player::P0, Player::P1] {
            let (mut g, _) = cast("Fortress Kin-Guard", p, true, false);
            spawn(
                &mut g,
                p,
                Card::from_name("Good-Fortune Unicorn").unwrap(),
                Zone::Battlefield,
            );
            let next = g.state.next_stack_id;
            g.state.next_stack_id = u64::MAX;
            let before = g.snapshot();
            let revision = g.revision();
            assert!(matches!(
                g.apply(
                    p,
                    Action::ChooseEndure {
                        put_counters: false
                    }
                ),
                Err(Error::CapacityExceeded(_))
            ));
            assert_eq!(g.snapshot(), before);
            assert_eq!(g.revision(), revision);
            assert!(spirits(&g, p).is_empty());
            g.state.next_stack_id = next;
            replay(
                &mut g,
                Action::ChooseEndure {
                    put_counters: false,
                },
            );
            drive(&mut g, false, false, true);
            let tokens = spirits(&g, p);
            assert_eq!(tokens.len(), 1);
            assert_eq!(tokens[0].counters.plus_one, 1);
            assert_eq!(tokens[0].power, 2);
        }
    }
}
