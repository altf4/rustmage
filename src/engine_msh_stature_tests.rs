//! Whole Stature: paid scalar Power-up, derived blocking and exact owned lifetimes.
use super::ability_world::CharacteristicWorld;
use super::assurance::big_helpers::{cast, checked, named, next_turn, settle, setup};
use super::*;

fn announce(card: CardId) -> Action {
    Action::AnnounceActivate {
        card,
        ability: 0,
        counter_payment: Counters::default(),
    }
}
fn reject(g: &mut Game, action: Action) {
    let before = g.snapshot();
    let revision = g.revision();
    assert_eq!(
        g.apply(g.actor().unwrap(), action),
        Err(Error::IllegalAction)
    );
    assert_eq!(g.snapshot(), before);
    assert_eq!(g.revision(), revision);
}
fn finish(g: &mut Game) {
    for _ in 0..12 {
        if g.state.announcing.is_none() {
            return;
        }
        let a = g
            .legal_actions()
            .into_iter()
            .find(|a| {
                matches!(
                    a,
                    Action::FinishTargets
                        | Action::ChooseActivationCost { .. }
                        | Action::PayCosts { .. }
                )
            })
            .unwrap();
        checked(g, a);
    }
    panic!("unfinished Stature payment");
}
fn paid(g: &mut Game, p: Player, name: &str, targets: &[Target]) -> CardId {
    if g.actor() != Some(p) {
        checked(g, Action::Pass);
    }
    let id = named(g, p, name, Zone::Hand);
    cast(g, id, None, targets);
    settle(g);
    id
}
fn stature(g: &mut Game, p: Player) -> CardId {
    paid(g, p, "Stature, Size Shifter", &[])
}
fn power(g: &mut Game, id: CardId, x: u32) {
    let p = g.object_view(id).controller;
    if g.actor() != Some(p) {
        checked(g, Action::Pass);
    }
    // The indexed scalar decision exposes interval endpoints. Fund this printed
    // activation exactly so the chosen X is its upper legal endpoint.
    let cost = g
        .activation_mana_costs(id, g.object_view(id).card, 0, x)
        .next()
        .unwrap();
    g.state.players[p.index()].mana = cost.colored;
    g.state.players[p.index()].mana[5] += cost.generic;
    checked(g, announce(id));
    checked(g, Action::ChooseX { value: x });
    finish(g);
}
fn until(g: &mut Game, stop: impl Fn(&Game) -> bool) {
    g.prepare_triggers();
    for _ in 0..100 {
        if stop(g) {
            return;
        }
        let a = g
            .legal_actions()
            .into_iter()
            .find(|a| {
                matches!(
                    a,
                    Action::Pass
                        | Action::PutTrigger { .. }
                        | Action::FinishTargets
                        | Action::PayCosts { .. }
                        | Action::FinishAttackers
                        | Action::FinishBlockers
                )
            })
            .unwrap();
        checked(g, a);
    }
    panic!("Stature scenario stalled {:?}", g.state.pending);
}
fn declare(g: &mut Game, p: Player, id: CardId) {
    // The paid creature now represents a permanent ready for this combat.
    g.state.objects[id as usize].summoning_sick = false;
    g.state.active = p;
    g.state.priority = p;
    g.state.step = Step::Attackers;
    g.state.pending = Pending::Attackers;
    checked(g, Action::Attack { card: id });
    checked(g, Action::FinishAttackers);
    until(g, |g| g.state.pending == Pending::Blockers);
}
#[test]
fn msh_stature_paid_body_boundary_and_counter_power_change_real_block_actions() {
    for p in [Player::P0, Player::P1] {
        for upgraded in [false, true] {
            let mut g = setup(p);
            g.state.players[p.index()].mana = [0, 1, 0, 0, 0, 0];
            let id = stature(&mut g, p);
            assert_eq!(g.state.players[p.index()].mana, [0; 6]);
            let v = g.object_view(id);
            assert_eq!((v.power, v.toughness, v.colors), (1, 1, 2));
            assert!(v.has_subtype("Human") && v.has_subtype("Hero"));
            assert!(v.card.info().legendary);
            let blocker = named(&mut g, p.other(), "Aegis Turtle", Zone::Battlefield);
            if upgraded {
                g.state.players[p.index()].mana = [0, 1, 0, 0, 0, 1];
                power(&mut g, id, 1);
                settle(&mut g);
                assert_eq!(g.stats(id).power, 2);
            }
            declare(&mut g, p, id);
            let action = Action::Block {
                blocker,
                attacker: id,
            };
            if upgraded {
                checked(&mut g, action);
            } else {
                reject(&mut g, action);
                for name in ["Serra Angel", "Kree Sentinel"] {
                    let blocker = named(&mut g, p.other(), name, Zone::Battlefield);
                    reject(
                        &mut g,
                        Action::Block {
                            blocker,
                            attacker: id,
                        },
                    );
                }
            }
            checked(&mut g, Action::FinishBlockers);
            until(&mut g, |g| g.state.step == Step::EndCombat);
            assert_eq!(g.life(p.other()), if upgraded { 20 } else { 19 });
        }
    }
}
#[test]
fn msh_stature_live_growth_and_depower_before_blocks_include_negative_power() {
    for p in [Player::P0, Player::P1] {
        for depower in [false, true] {
            let mut g = setup(p);
            let id = stature(&mut g, p);
            paid(&mut g, p, "Giant Growth", &[Target::Card(id)]);
            assert_eq!(g.stats(id).power, 4);
            if depower {
                paid(&mut g, p, "Depower", &[Target::Card(id)]);
                assert_eq!(g.stats(id).power, 0);
                paid(&mut g, p, "Depower", &[Target::Card(id)]);
                assert_eq!(g.stats(id).power, -4);
            }
            let blocker = named(&mut g, p.other(), "Aegis Turtle", Zone::Battlefield);
            declare(&mut g, p, id);
            let action = Action::Block {
                blocker,
                attacker: id,
            };
            if depower {
                reject(&mut g, action);
            } else {
                checked(&mut g, action);
            }
            checked(&mut g, Action::FinishBlockers);
        }
    }
}
#[test]
fn msh_stature_reducing_after_committed_blocks_keeps_blocked_status() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let id = stature(&mut g, p);
        paid(&mut g, p, "Giant Growth", &[Target::Card(id)]);
        let blocker = named(&mut g, p.other(), "Aegis Turtle", Zone::Battlefield);
        declare(&mut g, p, id);
        checked(
            &mut g,
            Action::Block {
                blocker,
                attacker: id,
            },
        );
        checked(&mut g, Action::FinishBlockers);
        g.state.players[p.index()].mana = [30; 6];
        paid(&mut g, p, "Depower", &[Target::Card(id)]);
        assert_eq!(g.stats(id).power, 0);
        assert!(
            g.state
                .attackers
                .iter()
                .find(|a| a.card == id)
                .unwrap()
                .blocked
        );
        assert!(g.state.blocks.contains(&(blocker, id)));
        // Raise from zero to one: the legal block remains and deals no player damage.
        power(&mut g, id, 1);
        settle(&mut g);
        until(&mut g, |g| g.state.step == Step::EndCombat);
        assert_eq!(g.life(p.other()), 20);
        assert_eq!(g.object_view(blocker).damage, 1);
    }
}
#[test]
fn msh_stature_raising_after_unblocked_declaration_never_creates_retroactive_block() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let id = stature(&mut g, p);
        let blocker = named(&mut g, p.other(), "Aegis Turtle", Zone::Battlefield);
        declare(&mut g, p, id);
        reject(
            &mut g,
            Action::Block {
                blocker,
                attacker: id,
            },
        );
        checked(&mut g, Action::FinishBlockers);
        power(&mut g, id, 2);
        settle(&mut g);
        until(&mut g, |g| g.state.step == Step::EndCombat);
        assert_eq!(g.life(p.other()), 17);
        assert_eq!(g.object_view(blocker).damage, 0);
    }
}
#[test]
fn msh_stature_printed_suppression_and_resolution_rule_have_separate_lifetimes() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let id = stature(&mut g, p);
        let blocker = named(&mut g, p.other(), "Aegis Turtle", Zone::Battlefield);
        let aura = paid(&mut g, p, "Witness Protection", &[Target::Card(id)]);
        assert_eq!(g.stats(id).power, 1);
        assert!(!g.printed_abilities_available(id));
        assert!(g.can_block_pair(blocker, id));
        let mut blanked = g.clone();
        declare(&mut blanked, p, id);
        checked(
            &mut blanked,
            Action::Block {
                blocker,
                attacker: id,
            },
        );
        checked(&mut blanked, Action::FinishBlockers);
        drop(blanked);
        reject(&mut g, announce(id));
        g.move_object(aura, Zone::Graveyard);
        assert!(!g.can_block_pair(blocker, id));
        paid(&mut g, p, "Pym Particles", &[Target::Card(id)]);
        paid(&mut g, p, "Witness Protection", &[Target::Card(id)]);
        assert!(!g.printed_abilities_available(id));
        declare(&mut g, p, id);
        reject(
            &mut g,
            Action::Block {
                blocker,
                attacker: id,
            },
        );
        checked(&mut g, Action::FinishBlockers);
    }
}
#[test]
fn msh_stature_scalar_entry_and_later_costs_zero_use_private_choice_and_atomicity() {
    for p in [Player::P0, Player::P1] {
        for later in [false, true] {
            for x in [0, 1, 2, 7] {
                let mut g = setup(p);
                let id = stature(&mut g, p);
                let secret = named(&mut g, p, "Shock", Zone::Hand);
                if later {
                    next_turn(&mut g);
                    next_turn(&mut g);
                }
                g.state.players[p.index()].mana = [0, if later { 2 } else { 1 }, 0, 0, 0, x];
                checked(&mut g, announce(id));
                assert_eq!(
                    g.x_choice().unwrap().intervals,
                    vec![XInterval {
                        minimum: 0,
                        maximum: x
                    }]
                );
                assert!(g.observe(p.other()).x_choice.is_none());
                assert!(!g.observe(p.other()).objects.iter().any(|v| v.id == secret));
                reject(&mut g, Action::ChooseX { value: x + 1 });
                checked(&mut g, Action::ChooseX { value: x });
                let request = g.announcement_payment_request(p).unwrap();
                assert_eq!(
                    (request.required, request.generic),
                    ([0, if later { 2 } else { 1 }, 0, 0, 0, 0], x)
                );
                reject(&mut g, Action::PayCosts { payment: [0; 6] });
                finish(&mut g);
                assert_eq!(g.state.players[p.index()].mana, [0; 6]);
                assert_eq!(g.object_view(id).power_up_uses[0].count, 1);
                settle(&mut g);
                assert_eq!(g.counter_count(id, CounterKind::PlusOne), u64::from(x));
                assert_eq!(g.stats(id).power, i128::from(x) + 1);
                g.state.players[p.index()].mana = [30; 6];
                reject(&mut g, announce(id));
                assert_eq!(
                    g.observe(p)
                        .objects
                        .iter()
                        .find(|v| v.id == id)
                        .unwrap()
                        .power_up_uses,
                    g.observe(p.other())
                        .objects
                        .iter()
                        .find(|v| v.id == id)
                        .unwrap()
                        .power_up_uses
                );
            }
        }
    }
}
#[test]
fn msh_stature_actual_land_funding_upper_x_and_required_blue() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let id = stature(&mut g, p);
        let island = named(&mut g, p, "Island", Zone::Battlefield);
        let forests: Vec<_> = (0..3)
            .map(|_| named(&mut g, p, "Forest", Zone::Battlefield))
            .collect();
        g.state.players[p.index()].mana = [0; 6];
        checked(&mut g, announce(id));
        assert_eq!(
            g.x_choice().unwrap().intervals,
            vec![XInterval {
                minimum: 0,
                maximum: 3
            }]
        );
        checked(&mut g, Action::ChooseX { value: 3 });
        assert!(g.announcement_payment_request(p.other()).is_none());
        checked(&mut g, Action::TapMana { card: island });
        for card in forests {
            checked(&mut g, Action::TapMana { card });
        }
        finish(&mut g);
        settle(&mut g);
        assert_eq!(g.counter_count(id, CounterKind::PlusOne), 3);
        assert_eq!(g.state.players[p.index()].mana, [0; 6]);
        g.move_object(id, Zone::Exile);
        g.move_object(id, Zone::Battlefield);
        g.state.players[p.index()].mana = [0, 0, 0, 0, 0, 30];
        reject(&mut g, announce(id));
    }
}
#[test]
fn msh_stature_counter_doubling_is_one_hero_event_and_zero_x_is_no_event() {
    for p in [Player::P0, Player::P1] {
        for x in [0, 2] {
            let mut g = setup(p);
            paid(&mut g, p, "Invisible Woman, Sue Storm", &[]);
            paid(&mut g, p, "Doubling Season", &[]);
            let id = stature(&mut g, p);
            power(&mut g, id, x);
            settle(&mut g);
            assert_eq!(g.counter_count(id, CounterKind::PlusOne), u64::from(x) * 2);
            let walls = g
                .state
                .objects
                .iter()
                .filter(|o| {
                    o.zone == Zone::Battlefield
                        && o.controller == p
                        && o.card.identity() == "@token/msh-wall"
                })
                .count();
            assert_eq!(walls, if x == 0 { 0 } else { 2 });
            assert_eq!(g.object_view(id).power_up_uses[0].count, 1);
        }
    }
}
#[test]
fn msh_stature_counter_placer_stays_captured_after_paid_source_theft() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        paid(&mut g, p.other(), "High Fae Trickster", &[]);
        paid(&mut g, p.other(), "Invisible Woman, Sue Storm", &[]);
        let id = stature(&mut g, p);
        power(&mut g, id, 2);
        checked(&mut g, Action::Pass);
        let theft = named(&mut g, p.other(), "Involuntary Employment", Zone::Hand);
        cast(&mut g, theft, None, &[Target::Card(id)]);
        settle(&mut g);
        assert_eq!(g.object_view(id).controller, p.other());
        assert_eq!(g.counter_count(id, CounterKind::PlusOne), 2);
        assert!(
            !g.state
                .objects
                .iter()
                .any(|o| o.zone == Zone::Battlefield && o.card.identity() == "@token/msh-wall")
        );
        assert!(g.plus_one_placed_this_turn(g.object_ref(id), p));
        assert!(!g.plus_one_placed_this_turn(g.object_ref(id), p.other()));
        if g.actor() != Some(p.other()) {
            checked(&mut g, Action::Pass);
        }
        reject(&mut g, announce(id));
    }
}
#[test]
fn msh_stature_lifetime_cleanup_theft_blink_and_old_packet_never_rebind() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let id = stature(&mut g, p);
        power(&mut g, id, 0);
        settle(&mut g);
        next_turn(&mut g);
        next_turn(&mut g);
        reject(&mut g, announce(id));
        until(&mut g, |g| g.state.step == Step::Main1);
        g.state.players[p.index()].mana = [30; 6];
        paid(&mut g, p, "Involuntary Employment", &[Target::Card(id)]);
        reject(&mut g, announce(id));
        g.move_object(id, Zone::Exile);
        g.move_object(id, Zone::Battlefield);
        assert!(g.object_view(id).entered_this_turn);
        assert!(g.object_view(id).power_up_uses.is_empty());
        g.state.players[p.index()].mana = [30; 6];
        power(&mut g, id, 3);
        let old = g.object_ref(id);
        g.move_object(id, Zone::Exile);
        g.move_object(id, Zone::Battlefield);
        assert_ne!(old, g.object_ref(id));
        settle(&mut g);
        assert_eq!(g.counter_count(id, CounterKind::PlusOne), 0);
        assert!(g.object_view(id).power_up_uses.is_empty());
        power(&mut g, id, 1);
        settle(&mut g);
        assert_eq!(g.counter_count(id, CounterKind::PlusOne), 1);
    }
}
#[test]
fn msh_stature_wonder_man_additional_use_does_not_refresh_consumed_count() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let id = stature(&mut g, p);
        power(&mut g, id, 1);
        settle(&mut g);
        reject(&mut g, announce(id));
        g.state.players[p.index()].mana = [30; 6];
        let wonder = paid(&mut g, p, "Wonder Man, Hollywood Hero", &[]);
        power(&mut g, id, 2);
        settle(&mut g);
        assert_eq!(g.counter_count(id, CounterKind::PlusOne), 3);
        reject(&mut g, announce(id));
        g.move_object(wonder, Zone::Exile);
        g.state.players[p.index()].mana = [30; 6];
        paid(&mut g, p, "Wonder Man, Hollywood Hero", &[]);
        reject(&mut g, announce(id));
    }
}
#[test]
fn msh_stature_hulk_reduction_after_x_and_locked_payment_preserves_blue() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let hulk = paid(&mut g, p, "Hulk, Gamma Goliath", &[]);
        let id = stature(&mut g, p);
        g.state.players[p.index()].mana = [0, 1, 0, 0, 0, 2];
        checked(&mut g, announce(id));
        assert_eq!(
            g.x_choice().unwrap().intervals,
            vec![XInterval {
                minimum: 0,
                maximum: 5
            }]
        );
        checked(&mut g, Action::ChooseX { value: 5 });
        assert_eq!(g.announcement_payment_request(p).unwrap().generic, 2);
        g.move_object(hulk, Zone::Exile);
        assert_eq!(g.announcement_payment_request(p).unwrap().generic, 2);
        finish(&mut g);
        settle(&mut g);
        assert_eq!(g.counter_count(id, CounterKind::PlusOne), 5);
        assert_eq!(g.state.players[p.index()].mana, [0; 6]);
    }
}
#[test]
fn msh_stature_copy_recipe_and_incarnation_own_independent_power_up_uses() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let id = stature(&mut g, p);
        power(&mut g, id, 1);
        settle(&mut g);
        let definition = g.object_view(id).card;
        // The existing generic production copy primitive supplies an independent object.
        let copy = g.create_copy_groups_inner(
            p.other(),
            &[TokenGroupView {
                token: definition,
                amount: 1,
                tapped: false,
            }],
            &[Some(CopyRecipe::new(definition))],
            None,
        )[0]
        .card;
        g.advance_entries();
        assert!(g.object_view(copy).power_up_uses.is_empty());
        assert_eq!(g.counter_count(copy, CounterKind::PlusOne), 0);
        g.state.priority = p.other();
        power(&mut g, copy, 2);
        settle(&mut g);
        assert_eq!(g.counter_count(copy, CounterKind::PlusOne), 2);
        assert_eq!(g.counter_count(id, CounterKind::PlusOne), 1);
        assert_eq!(g.object_view(copy).power_up_uses[0].count, 1);
    }
}
