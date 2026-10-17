//! Whole-card paid acceptance for the three death/combat consumers.
use super::ability_timeline_tests::{cast, checked, named, next_turn, settle, setup};
use super::*;

fn paid(g: &mut Game, p: Player, name: &str, targets: &[Target]) -> CardId {
    let id = named(g, p, name, Zone::Hand);
    let before = g.state.players[p.index()].mana.iter().sum::<u32>();
    cast(g, id, None, targets);
    settle(g);
    if let Some(cost) = match name {
        "Garna, Bloodfist of Keld" | "Desecration Demon" => Some(4),
        "Crossway Troublemakers" => Some(6),
        _ => None,
    } {
        assert_eq!(
            before - g.state.players[p.index()].mana.iter().sum::<u32>(),
            cost
        );
    }
    id
}
fn until(g: &mut Game, stop: impl Fn(&Game) -> bool) {
    g.prepare_triggers();
    for _ in 0..100 {
        if stop(g) {
            return;
        }
        let actions = g.legal_actions();
        let action = if g.state.pending == Pending::Priority {
            Action::Pass
        } else {
            actions
                .into_iter()
                .find(|a| {
                    matches!(
                        a,
                        Action::PutTrigger { .. }
                            | Action::FinishTargets
                            | Action::PayCosts { .. }
                            | Action::FinishAttackers
                            | Action::FinishBlockers
                    )
                })
                .unwrap_or_else(|| panic!("unexpected {:?}", g.state.pending))
        };
        checked(g, action);
    }
    panic!("stalled {:?}", g.state.pending)
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
fn attack(g: &mut Game, ids: &[CardId]) {
    g.state.step = Step::Attackers;
    g.state.pending = Pending::Attackers;
    for &id in ids {
        g.state.objects[id as usize].summoning_sick = false;
        checked(g, Action::Attack { card: id });
    }
    checked(g, Action::FinishAttackers);
}
fn begin_combat(g: &mut Game) {
    assert_eq!(g.state.step, Step::Main1);
    checked(g, Action::Pass);
    checked(g, Action::Pass);
    assert_eq!(g.state.step, Step::BeginCombat);
}

#[test]
fn fdn_death_demon_three_garna_lifelink_damage_ignores_player_hexproof_and_wrath_sees_other_deaths()
{
    for p in [Player::P0, Player::P1] {
        for protected in [false, true] {
            let mut g = setup(p);
            let garna = paid(&mut g, p, "Garna, Bloodfist of Keld", &[]);
            assert_eq!(
                (g.object_view(garna).power, g.object_view(garna).toughness),
                (4, 3)
            );
            if protected {
                named(&mut g, p.other(), "Crystal Barricade", Zone::Battlefield);
            }
            let collar = paid(&mut g, p, "Basilisk Collar", &[]);
            let equip = g.legal_actions().into_iter().find(|a| matches!(a, Action::Activate { card, target: Some(Target::Card(t)), .. } if *card == collar && *t == garna)).unwrap();
            checked(&mut g, equip);
            settle(&mut g);
            let bear = named(&mut g, p, "Bear Cub", Zone::Battlefield);
            paid(&mut g, p, "Shock", &[Target::Card(bear)]);
            // Actual damage grants lifelink; this nontargeted ability ignores
            // the opposing player's hexproof rather than losing its effect.
            assert_eq!(g.state.players[p.other().index()].life, 19);
            assert_eq!(g.state.players[p.index()].life, 21);
        }
        let mut g = setup(p);
        let garna = paid(&mut g, p, "Garna, Bloodfist of Keld", &[]);
        let ally = named(&mut g, p, "Bear Cub", Zone::Battlefield);
        let enemy = named(&mut g, p.other(), "Bear Cub", Zone::Battlefield);
        let draws = g.state.players[p.index()].cards_drawn_this_turn;
        paid(&mut g, p, "Day of Judgment", &[]);
        for id in [garna, ally, enemy] {
            assert_eq!(g.object_view(id).zone, Zone::Graveyard);
        }
        assert_eq!(g.state.players[p.other().index()].life, 19);
        assert_eq!(g.state.players[p.index()].cards_drawn_this_turn, draws);
    }
}
#[test]
fn fdn_death_demon_three_garna_attack_death_draw_survives_paid_source_bounce_privately() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let garna = paid(&mut g, p, "Garna, Bloodfist of Keld", &[]);
        let bear = named(&mut g, p, "Bear Cub", Zone::Battlefield);
        attack(&mut g, &[bear]);
        let shock = named(&mut g, p, "Shock", Zone::Hand);
        cast(&mut g, shock, None, &[Target::Card(bear)]);
        until(&mut g, |g| {
            g.object_view(bear).zone == Zone::Graveyard && g.state.pending == Pending::Priority
        });
        let top = *g.state.players[p.index()].library.last().unwrap();
        let draws = g.state.players[p.index()].cards_drawn_this_turn;
        paid(&mut g, p, "Unsummon", &[Target::Card(garna)]);
        assert_eq!(g.object_view(garna).zone, Zone::Hand);
        assert_eq!(g.state.players[p.index()].cards_drawn_this_turn, draws + 1);
        assert_eq!(g.object_view(top).zone, Zone::Hand);
        assert!(!g.observe(p.other()).objects.iter().any(|o| o.id == top));
        assert_eq!(g.state.players[p.other().index()].life, 20);
    }
}
#[test]
fn fdn_death_demon_three_crossway_actual_attacking_vampire_damage_and_removed_grants() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let crossway = paid(&mut g, p, "Crossway Troublemakers", &[]);
        assert_eq!(
            (
                g.object_view(crossway).power,
                g.object_view(crossway).toughness
            ),
            (5, 5)
        );
        let vampire = named(&mut g, p, "Vampire Interloper", Zone::Battlefield);
        let waiting = named(&mut g, p, "Vampire Interloper", Zone::Battlefield);
        let bear = named(&mut g, p, "Bear Cub", Zone::Battlefield);
        let enemy = named(&mut g, p.other(), "Vampire Interloper", Zone::Battlefield);
        let big = named(&mut g, p.other(), "Gigantosaurus", Zone::Battlefield);
        attack(&mut g, &[crossway, vampire, bear]);
        let both = Keywords::DEATHTOUCH.union(Keywords::LIFELINK);
        assert!(g.keywords(crossway).contains(both));
        assert!(g.keywords(vampire).contains(both));
        for id in [waiting, bear, enemy] {
            assert!(!g.keywords(id).contains(Keywords::LIFELINK));
        }
        until(&mut g, |g| g.state.pending == Pending::Blockers);
        checked(
            &mut g,
            Action::Block {
                blocker: big,
                attacker: crossway,
            },
        );
        checked(&mut g, Action::FinishBlockers);
        until(&mut g, |g| g.object_view(big).zone == Zone::Graveyard);
        // Five blocked and two unblocked Vampire damage both have lifelink.
        assert_eq!(g.state.players[p.index()].life, 27);
        assert_eq!(g.object_view(crossway).zone, Zone::Graveyard);
        until(&mut g, |g| {
            matches!(g.state.pending, Pending::EffectOptional { .. })
        });
        checked(&mut g, Action::ChooseOptional { accept: false });
        settle(&mut g);
        assert!(!g.keywords(vampire).contains(Keywords::LIFELINK));

        let mut g = setup(p);
        let c = paid(&mut g, p, "Crossway Troublemakers", &[]);
        let v = named(&mut g, p, "Vampire Interloper", Zone::Battlefield);
        let stop = paid(&mut g, p, "Stop Cold", &[Target::Card(c)]);
        attack(&mut g, &[v]);
        assert!(!g.keywords(v).contains(Keywords::LIFELINK));
        g.state.players[p.index()].mana = [30; 6];
        paid(&mut g, p, "Disenchant", &[Target::Card(stop)]);
        assert!(g.keywords(v).contains(both));
    }
}
#[test]
fn fdn_death_demon_three_crossway_queued_payment_survives_paid_bounce_and_current_control() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let c = paid(&mut g, p, "Crossway Troublemakers", &[]);
        let v = named(&mut g, p, "Vampire Interloper", Zone::Battlefield);
        let shock = named(&mut g, p, "Shock", Zone::Hand);
        cast(&mut g, shock, None, &[Target::Card(v)]);
        until(&mut g, |g| {
            g.object_view(v).zone == Zone::Graveyard && g.state.pending == Pending::Priority
        });
        let bounce = named(&mut g, p, "Unsummon", Zone::Hand);
        cast(&mut g, bounce, None, &[Target::Card(c)]);
        until(&mut g, |g| {
            matches!(g.state.pending, Pending::EffectOptional { .. })
        });
        assert_eq!(g.object_view(c).zone, Zone::Hand);
        assert_eq!(g.actor(), Some(p));
        let top = *g.state.players[p.index()].library.last().unwrap();
        checked(&mut g, Action::ChooseOptional { accept: true });
        settle(&mut g);
        assert_eq!(g.state.players[p.index()].life, 18);
        assert_eq!(g.state.players[p.index()].life_lost_this_turn, 2);
        assert_eq!(g.object_view(top).zone, Zone::Hand);
        assert!(!g.observe(p.other()).objects.iter().any(|o| o.id == top));
    }
}
#[test]
fn fdn_death_demon_three_demon_both_players_combats_paid_decline_and_empty_choices() {
    for p in [Player::P0, Player::P1] {
        for their_combat in [false, true] {
            for route in 0..3 {
                let mut g = setup(p);
                let d = paid(&mut g, p, "Desecration Demon", &[]);
                assert_eq!((g.object_view(d).power, g.object_view(d).toughness), (6, 6));
                assert!(g.keywords(d).contains(Keywords::FLYING));
                if their_combat {
                    next_turn(&mut g);
                    until(&mut g, |g| g.state.step == Step::Main1);
                }
                let resource = if route == 2 {
                    None
                } else {
                    Some(named(&mut g, p.other(), "Bear Cub", Zone::Battlefield))
                };
                let artifact = named(&mut g, p.other(), "Swiftfoot Boots", Zone::Battlefield);
                begin_combat(&mut g);
                if let Some(resource) = resource {
                    until(&mut g, |g| {
                        matches!(g.state.pending, Pending::EffectChoice { .. })
                    });
                    assert_eq!(g.actor(), Some(p.other()));
                    let object = g.object_ref(artifact);
                    reject(&mut g, Action::Sacrifice { object });
                    let object = g.object_ref(d);
                    reject(&mut g, Action::Sacrifice { object });
                    if route == 0 {
                        let object = g.object_ref(resource);
                        checked(&mut g, Action::Sacrifice { object });
                    } else {
                        checked(&mut g, Action::DeclineEffectPayment);
                    }
                }
                settle(&mut g);
                assert_eq!(
                    g.object_view(d).counters.plus_one,
                    if route == 0 { 1 } else { 0 }
                );
                assert_eq!(g.object_view(d).tapped, route == 0);
                if let Some(resource) = resource {
                    assert_eq!(
                        g.object_view(resource).zone,
                        if route == 0 {
                            Zone::Graveyard
                        } else {
                            Zone::Battlefield
                        }
                    );
                }
                // A distinct combat-step event has no once-per-turn restriction.
                let resource = named(&mut g, p.other(), "Bear Cub", Zone::Battlefield);
                g.begin_step(Step::BeginCombat);
                g.give_priority();
                until(&mut g, |g| {
                    matches!(g.state.pending, Pending::EffectChoice { .. })
                });
                let object = g.object_ref(resource);
                checked(&mut g, Action::Sacrifice { object });
                settle(&mut g);
                assert_eq!(
                    g.object_view(d).counters.plus_one,
                    if route == 0 { 2 } else { 1 }
                );
                assert!(g.object_view(d).tapped);
            }
        }
    }
}
#[test]
fn fdn_death_demon_three_demon_paid_source_bounce_keeps_sacrifice_and_new_incarnation_is_untouched()
{
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let d = paid(&mut g, p, "Desecration Demon", &[]);
        let original = g.object_ref(d);
        let resource = named(&mut g, p.other(), "Bear Cub", Zone::Battlefield);
        begin_combat(&mut g);
        g.state.players[p.index()].mana = [30; 6];
        let bounce = named(&mut g, p, "Unsummon", Zone::Hand);
        cast(&mut g, bounce, None, &[Target::Card(d)]);
        until(&mut g, |g| {
            matches!(g.state.pending, Pending::EffectChoice { .. })
        });
        assert_eq!(g.object_view(d).zone, Zone::Hand);
        assert_ne!(g.object_ref(d), original);
        let object = g.object_ref(resource);
        checked(&mut g, Action::Sacrifice { object });
        settle(&mut g);
        assert_eq!(g.object_view(resource).zone, Zone::Graveyard);
        assert_eq!(g.object_view(d).counters.plus_one, 0);
        until(&mut g, |g| {
            g.state.step == Step::Main2 && g.state.pending == Pending::Priority
        });
        g.state.players[p.index()].mana = [30; 6];
        cast(&mut g, d, None, &[]);
        settle(&mut g);
        assert!(!g.object_view(d).tapped);
        assert_eq!(g.object_view(d).counters.plus_one, 0);
    }
}

#[test]
fn fdn_death_demon_three_paid_control_change_sets_current_opponent_and_payment_controller() {
    for p in [Player::P0, Player::P1] {
        for name in ["Crossway Troublemakers", "Desecration Demon"] {
            let mut g = setup(p);
            let source = paid(&mut g, p, name, &[]);
            next_turn(&mut g);
            until(&mut g, |g| g.state.step == Step::Main1);
            g.state.players[p.other().index()].mana = [30; 6];
            paid(
                &mut g,
                p.other(),
                "Involuntary Employment",
                &[Target::Card(source)],
            );
            assert_eq!(g.object_view(source).controller, p.other());
            if name == "Desecration Demon" {
                let resource = named(&mut g, p, "Bear Cub", Zone::Battlefield);
                begin_combat(&mut g);
                until(&mut g, |g| {
                    matches!(g.state.pending, Pending::EffectChoice { .. })
                });
                assert_eq!(g.actor(), Some(p));
                let object = g.object_ref(resource);
                checked(&mut g, Action::Sacrifice { object });
                settle(&mut g);
                assert_eq!(g.object_view(source).counters.plus_one, 1);
                assert!(g.object_view(source).tapped);
            } else {
                let v = named(&mut g, p.other(), "Vampire Interloper", Zone::Battlefield);
                attack(&mut g, &[v]);
                assert!(
                    g.keywords(v)
                        .contains(Keywords::LIFELINK.union(Keywords::DEATHTOUCH))
                );
                let shock = named(&mut g, p.other(), "Shock", Zone::Hand);
                cast(&mut g, shock, None, &[Target::Card(v)]);
                until(&mut g, |g| {
                    matches!(g.state.pending, Pending::EffectOptional { .. })
                });
                assert_eq!(g.actor(), Some(p.other()));
                checked(&mut g, Action::ChooseOptional { accept: true });
                settle(&mut g);
                assert_eq!(g.state.players[p.other().index()].life, 18);
                assert_eq!(g.state.players[p.index()].life, 20);
            }
        }
    }
}
