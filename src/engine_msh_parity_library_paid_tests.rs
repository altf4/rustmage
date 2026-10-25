//! Complete paid producer obligations; source seeds stay bounded until admission.
use super::assurance::big_helpers::{cast, checked, named, next_turn, settle, setup};
use super::*;
const THANOS: &str = "Thanos, the Mad Titan";
const STRAT: &str = "Trickster's Stratagem";
fn fixture(g: &mut Game, p: Player, name: &str, zone: Zone) -> CardId {
    if Card::from_name(name).unwrap().is_supported() {
        named(g, p, name, zone)
    } else {
        super::parity_library_core_tests::gated(g, p, name, zone)
    }
}
fn priority(g: &mut Game, p: Player) {
    if g.actor() != Some(p) {
        checked(g, Action::Pass);
    }
    assert_eq!(g.actor(), Some(p));
}
fn paid(g: &mut Game, p: Player, name: &str, targets: &[Target]) -> CardId {
    priority(g, p);
    let id = fixture(g, p, name, Zone::Hand);
    let (mut mana, generic) = Card::from_name(name).unwrap().cost();
    mana[5] += generic;
    g.state.players[p.index()].mana = mana;
    cast(g, id, None, targets);
    assert_eq!(g.state.players[p.index()].mana, [0; 6]);
    settle(g);
    id
}
fn reject(g: &mut Game, action: Action) {
    let before = g.snapshot();
    let revision = g.revision();
    let bytes = g.state_bytes();
    assert_eq!(
        g.apply(g.actor().unwrap(), action),
        Err(Error::IllegalAction)
    );
    assert_eq!(g.snapshot(), before);
    assert_eq!(g.revision(), revision);
    assert_eq!(g.state_bytes(), bytes);
}
fn announce(card: CardId) -> Action {
    Action::AnnounceActivate {
        card,
        ability: 0,
        counter_payment: Counters::default(),
    }
}
fn finish(g: &mut Game) {
    for _ in 0..12 {
        if g.state.announcing.is_none() {
            return;
        }
        let action = g
            .legal_actions()
            .into_iter()
            .find(|a| {
                matches!(
                    a,
                    Action::PayCosts { .. }
                        | Action::FinishTargets
                        | Action::ChooseActivationCost { .. }
                )
            })
            .unwrap();
        checked(g, action);
    }
    panic!("unpaid announcement");
}
fn until(g: &mut Game, stop: impl Fn(&Game) -> bool) {
    g.prepare_triggers();
    for _ in 0..160 {
        if stop(g) {
            return;
        }
        let action = g
            .legal_actions()
            .into_iter()
            .find(|a| {
                matches!(
                    a,
                    Action::Pass
                        | Action::PutTrigger { .. }
                        | Action::FinishTargets
                        | Action::FinishAttackers
                        | Action::FinishBlockers
                )
            })
            .unwrap_or_else(|| panic!("unexpected {:?}", g.state.pending));
        checked(g, action);
    }
    panic!("stalled {:?}", g.state.pending);
}
fn choose_parity(g: &mut Game, parity: ManaValueParity) {
    until(g, |g| {
        matches!(g.state.pending, Pending::ManaValueParity { .. })
    });
    checked(g, Action::ChooseManaValueParity { parity });
    settle(g);
}
fn strat(g: &mut Game, p: Player, target: CardId, own: Option<CardId>) -> CardId {
    priority(g, p);
    let id = fixture(g, p, STRAT, Zone::Hand);
    g.state.players[p.index()].mana = [0, 1, 0, 0, 0, 3];
    let targets: Vec<_> = std::iter::once(Target::Card(target))
        .chain(own.map(Target::Card))
        .collect();
    cast(g, id, None, &targets);
    assert_eq!(g.state.players[p.index()].mana, [0; 6]);
    id
}
#[test]
fn msh_parity_paid_thanos_exact_entry_discount_colorless_sickness_tap_and_lifetime_use() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let source = paid(&mut g, p, THANOS, &[]);
        let view = g.object_view(source);
        assert_eq!((view.power, view.toughness), (4, 4));
        assert!(
            view.keywords
                .contains(Keywords::DEATHTOUCH.union(Keywords::LIFELINK))
        );
        assert!(view.entered_this_turn && g.state.objects[source as usize].summoning_sick);
        for bad in [[0, 1, 0, 0, 1, 0], [0, 1, 0, 0, 0, 1], [1, 1, 0, 0, 1, 0]] {
            g.state.players[p.index()].mana = bad;
            reject(&mut g, announce(source));
        }
        // This ability has no tap cost and is usable at instant speed while tapped.
        g.state.objects[source as usize].tapped = true;
        g.state.players[p.index()].mana = [0, 1, 0, 0, 1, 1];
        checked(&mut g, announce(source));
        let request = g.announcement_payment_request(p).unwrap();
        assert_eq!((request.required, request.generic), ([0, 1, 0, 0, 1, 1], 0));
        reject(&mut g, Action::PayCosts { payment: [0; 6] });
        finish(&mut g);
        assert_eq!(g.state.players[p.index()].mana, [0; 6]);
        assert_eq!(g.object_view(source).power_up_uses[0].count, 1);
        choose_parity(&mut g, ManaValueParity::Even);
        assert_eq!(g.counter_count(source, CounterKind::PlusOne), 2);
        g.state.players[p.index()].mana = [30; 6];
        reject(&mut g, announce(source));
        next_turn(&mut g);
        next_turn(&mut g);
        priority(&mut g, p);
        g.state.players[p.index()].mana = [30; 6];
        reject(&mut g, announce(source));
        // Bounded control transition isolates lifetime use: changing controller never resets it.
        g.change_controller(g.object_ref(source), p.other());
        if g.actor() != Some(p.other()) {
            checked(&mut g, Action::Pass);
        }
        g.state.players[p.other().index()].mana = [30; 6];
        reject(&mut g, announce(source));
    }
}
#[test]
fn msh_parity_paid_thanos_later_full_cost_and_opponent_turn_activation() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let source = paid(&mut g, p, THANOS, &[]);
        next_turn(&mut g);
        assert_eq!(g.state.active, p.other());
        priority(&mut g, p);
        assert!(!g.object_view(source).entered_this_turn);
        for color in 0..6 {
            let mut mana = [1; 6];
            mana[color] = 0;
            g.state.players[p.index()].mana = mana;
            reject(&mut g, announce(source));
        }
        g.state.players[p.index()].mana = [1; 6];
        checked(&mut g, announce(source));
        let request = g.announcement_payment_request(p).unwrap();
        assert_eq!((request.required, request.generic), ([1; 6], 0));
        finish(&mut g);
        assert_eq!(g.state.players[p.index()].mana, [0; 6]);
        choose_parity(&mut g, ManaValueParity::Odd);
        assert!(g.on_battlefield(source));
        assert_eq!(g.counter_count(source, CounterKind::PlusOne), 2);
    }
}
#[test]
fn msh_parity_paid_thanos_removed_and_new_incarnation_does_not_receive_old_counters_or_exclusion() {
    for p in [Player::P0, Player::P1] {
        for recast in [false, true] {
            let mut g = setup(p);
            let source = paid(&mut g, p, THANOS, &[]);
            let old = g.object_ref(source);
            g.state.players[p.index()].mana = [0, 1, 0, 0, 1, 1];
            checked(&mut g, announce(source));
            finish(&mut g);
            // Pay a real response, then stop after it resolves rather than settling the older ability.
            priority(&mut g, p);
            let bounce = named(&mut g, p, "Unsummon", Zone::Hand);
            g.state.players[p.index()].mana = [0, 1, 0, 0, 0, 0];
            cast(&mut g, bounce, None, &[Target::Card(source)]);
            until(&mut g, |g| g.object_view(source).zone == Zone::Hand);
            if recast {
                // Sorcery recasting here has no permission. Explicit physical reentry is a bounded
                // exact-incarnation probe, separate from the real paid activation and response.
                g.move_object(source, Zone::Battlefield);
                assert_ne!(g.object_ref(source), old);
            }
            choose_parity(&mut g, ManaValueParity::Odd);
            assert_eq!(g.counter_count(source, CounterKind::PlusOne), 0);
            assert_eq!(
                g.object_view(source).zone,
                if recast { Zone::Graveyard } else { Zone::Hand }
            );
        }
    }
}
#[test]
fn msh_parity_paid_strat_owner_choice_then_optional_connive_land_and_nonland() {
    for p in [Player::P0, Player::P1] {
        for dest in [
            LibraryDestination::SecondFromTop,
            LibraryDestination::Bottom,
        ] {
            for nonland in [false, true] {
                let mut g = setup(p);
                let victim = named(&mut g, p.other(), "Hill Giant", Zone::Battlefield);
                let own = named(&mut g, p, "Bear Cub", Zone::Battlefield);
                let discard = named(
                    &mut g,
                    p,
                    if nonland { "Shock" } else { "Forest" },
                    Zone::Hand,
                );
                let top = *g.state.players[p.other().index()].library.last().unwrap();
                let len = g.state.players[p.other().index()].library.len();
                let hand = g.hand_count(p);
                let id = strat(&mut g, p, victim, Some(own));
                until(&mut g, |g| {
                    matches!(g.state.pending, Pending::TargetLibrary { .. })
                });
                assert_eq!(g.actor(), Some(p.other()));
                assert!(g.observe(p).target_library.is_none());
                let object = g.object_ref(victim);
                reject(
                    &mut g,
                    Action::ChooseLibraryDestination {
                        object,
                        destination: LibraryDestination::Top,
                    },
                );
                checked(
                    &mut g,
                    Action::ChooseLibraryDestination {
                        object,
                        destination: dest,
                    },
                );
                let library = &g.state.players[p.other().index()].library;
                assert_eq!(library.len(), len + 1);
                assert_eq!(library.last(), Some(&top));
                assert_eq!(
                    library[if dest == LibraryDestination::Bottom {
                        0
                    } else {
                        len - 1
                    }],
                    victim
                );
                assert_eq!(g.actor(), Some(p));
                assert!(matches!(g.state.pending, Pending::EffectDiscard { .. }));
                assert_eq!(g.hand_count(p), hand + 1);
                checked(&mut g, Action::Discard { card: discard });
                settle(&mut g);
                assert_eq!(g.hand_count(p), hand);
                assert_eq!(
                    g.counter_count(own, CounterKind::PlusOne),
                    u64::from(nonland)
                );
                assert_eq!(g.object_view(id).zone, Zone::Graveyard);
            }
        }
    }
}
#[test]
fn msh_parity_paid_strat_partial_and_all_invalid_targets_after_real_responses() {
    for p in [Player::P0, Player::P1] {
        for mask in 0..4 {
            let mut g = setup(p);
            let enemy = named(&mut g, p.other(), "Hill Giant", Zone::Battlefield);
            let own = named(&mut g, p, "Bear Cub", Zone::Battlefield);
            let discard = named(&mut g, p, "Shock", Zone::Hand);
            let spell = strat(&mut g, p, enemy, Some(own));
            for (bit, target) in [(1, enemy), (2, own)] {
                if mask & bit != 0 {
                    priority(&mut g, p);
                    let response = named(&mut g, p, "Unsummon", Zone::Hand);
                    g.state.players[p.index()].mana = [0, 1, 0, 0, 0, 0];
                    cast(&mut g, response, None, &[Target::Card(target)]);
                    until(&mut g, |g| g.object_view(target).zone == Zone::Hand);
                }
            }
            let before = g.hand_count(p);
            if mask == 3 {
                settle(&mut g);
                assert_eq!(g.hand_count(p), before);
            } else {
                if mask & 1 == 0 {
                    until(&mut g, |g| {
                        matches!(g.state.pending, Pending::TargetLibrary { .. })
                    });
                    let object = g.object_ref(enemy);
                    checked(
                        &mut g,
                        Action::ChooseLibraryDestination {
                            object,
                            destination: LibraryDestination::SecondFromTop,
                        },
                    );
                }
                if mask & 2 == 0 {
                    until(&mut g, |g| {
                        matches!(g.state.pending, Pending::EffectDiscard { .. })
                    });
                    assert_eq!(g.hand_count(p), before + 1);
                    checked(&mut g, Action::Discard { card: discard });
                }
                settle(&mut g);
                assert_eq!(g.hand_count(p), before);
            }
            assert_eq!(g.object_view(spell).zone, Zone::Graveyard);
            assert_eq!(
                g.counter_count(own, CounterKind::PlusOne),
                u64::from(mask & 2 == 0 && mask != 3)
            );
            assert_eq!(
                g.object_view(enemy).zone,
                if mask & 1 == 0 {
                    Zone::Library
                } else {
                    Zone::Hand
                }
            );
        }
    }
}
#[test]
fn msh_parity_paid_strat_target_relations_mandatory_first_optional_second_and_sorcery_timing() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let spell = fixture(&mut g, p, STRAT, Zone::Hand);
        let own = named(&mut g, p, "Bear Cub", Zone::Battlefield);
        let enemy = named(&mut g, p.other(), "Hill Giant", Zone::Battlefield);
        let artifact = named(&mut g, p.other(), "Arc Reactor", Zone::Battlefield);
        g.state.players[p.index()].mana = [0, 1, 0, 0, 0, 3];
        checked(&mut g, Action::AnnounceCast { card: spell });
        reject(&mut g, Action::FinishTargets);
        for card in [own, artifact] {
            reject(
                &mut g,
                Action::ChooseTarget {
                    target: Target::Card(card),
                },
            );
        }
        checked(
            &mut g,
            Action::ChooseTarget {
                target: Target::Card(enemy),
            },
        );
        if g.legal_actions().contains(&Action::FinishTargets) {
            checked(&mut g, Action::FinishTargets);
        }
        // First group is fixed; choosing an enemy for the independent optional own group is illegal.
        reject(
            &mut g,
            Action::ChooseTarget {
                target: Target::Card(enemy),
            },
        );
        finish(&mut g);
        until(&mut g, |g| {
            matches!(g.state.pending, Pending::TargetLibrary { .. })
        });
        let object = g.object_ref(enemy);
        checked(
            &mut g,
            Action::ChooseLibraryDestination {
                object,
                destination: LibraryDestination::Bottom,
            },
        );
        settle(&mut g);
        assert_eq!(g.counter_count(own, CounterKind::PlusOne), 0);
        let second = fixture(&mut g, p, STRAT, Zone::Hand);
        next_turn(&mut g);
        priority(&mut g, p);
        g.state.players[p.index()].mana = [0, 1, 0, 0, 0, 3];
        reject(&mut g, Action::AnnounceCast { card: second });
    }
}
#[test]
fn msh_parity_paid_thanos_untargeted_destruction_ignores_ward_and_hexproof() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let source = paid(&mut g, p, THANOS, &[]);
        let ward = named(&mut g, p.other(), "The Serpent Society", Zone::Battlefield);
        let value = Card::from_name("The Serpent Society")
            .unwrap()
            .fixed_mana_value();
        let shield = named(
            &mut g,
            p.other(),
            if value.is_multiple_of(2) {
                "Hill Giant"
            } else {
                "Brazen Scourge"
            },
            Zone::Battlefield,
        );
        g.state.objects[shield as usize].gained_keywords = Keywords::HEXPROOF;
        g.state.players[p.index()].mana = [0, 1, 0, 0, 1, 1];
        checked(&mut g, announce(source));
        finish(&mut g);
        choose_parity(
            &mut g,
            if value.is_multiple_of(2) {
                ManaValueParity::Even
            } else {
                ManaValueParity::Odd
            },
        );
        assert_eq!(g.object_view(ward).zone, Zone::Graveyard);
        assert_eq!(g.state.players[p.index()].mana, [0; 6]);
        assert!(g.state.pending_triggers.is_empty());
        assert!(!g.on_battlefield(shield));
    }
}
#[test]
fn msh_parity_paid_thanos_printed_combat_deathtouch_and_lifelink() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let source = paid(&mut g, p, THANOS, &[]);
        let turtle = named(&mut g, p.other(), "Aegis Turtle", Zone::Battlefield);
        // Advance real steps and turns; no haste or untap permission is invented.
        let entered = g.state.turn;
        until(&mut g, |g| {
            g.state.turn > entered && g.state.active == p && g.state.pending == Pending::Attackers
        });
        checked(&mut g, Action::Attack { card: source });
        checked(&mut g, Action::FinishAttackers);
        until(&mut g, |g| g.state.pending == Pending::Blockers);
        checked(
            &mut g,
            Action::Block {
                blocker: turtle,
                attacker: source,
            },
        );
        checked(&mut g, Action::FinishBlockers);
        until(&mut g, |g| g.state.step == Step::Main2);
        assert_eq!(g.object_view(turtle).zone, Zone::Graveyard);
        assert!(g.on_battlefield(source));
        assert_eq!(g.state.players[p.index()].life, 24);
        assert_eq!(g.state.players[p.other().index()].life, 20);
    }
}
#[test]
fn msh_parity_paid_thanos_real_counter_replacements_finish_before_mandatory_parity() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        paid(&mut g, p, "Doubling Season", &[]);
        paid(&mut g, p, "Doubling Season", &[]);
        let source = paid(&mut g, p, THANOS, &[]);
        g.state.players[p.index()].mana = [0, 1, 0, 0, 1, 1];
        checked(&mut g, announce(source));
        finish(&mut g);
        until(&mut g, |g| {
            matches!(g.state.pending, Pending::Replacement { .. })
        });
        assert_eq!(g.counter_count(source, CounterKind::PlusOne), 0);
        while matches!(g.state.pending, Pending::Replacement { .. }) {
            reject(
                &mut g,
                Action::ChooseManaValueParity {
                    parity: ManaValueParity::Odd,
                },
            );
            let action = g
                .legal_actions()
                .into_iter()
                .find(|a| matches!(a, Action::ChooseReplacement { .. }))
                .unwrap();
            checked(&mut g, action);
        }
        until(&mut g, |g| {
            matches!(g.state.pending, Pending::ManaValueParity { .. })
        });
        assert_eq!(g.counter_count(source, CounterKind::PlusOne), 8);
        choose_parity(&mut g, ManaValueParity::Even);
        assert_eq!(g.stats(source).power, 12);
    }
}
#[test]
fn msh_parity_paid_strat_real_stolen_owner_chooses_its_own_library_destination() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p.other());
        let victim = named(&mut g, p, "Bear Cub", Zone::Battlefield);
        paid(&mut g, p.other(), "Confiscate", &[Target::Card(victim)]);
        assert_eq!(g.object_view(victim).controller, p.other());
        next_turn(&mut g);
        until(&mut g, |g| {
            g.state.active == p
                && g.state.step == Step::Main1
                && g.state.pending == Pending::Priority
        });
        let spell = strat(&mut g, p, victim, None);
        until(&mut g, |g| {
            matches!(g.state.pending, Pending::TargetLibrary { .. })
        });
        assert_eq!(g.actor(), Some(p));
        assert!(g.observe(p).target_library.is_some());
        assert!(g.observe(p.other()).target_library.is_none());
        let object = g.object_ref(victim);
        checked(
            &mut g,
            Action::ChooseLibraryDestination {
                object,
                destination: LibraryDestination::Bottom,
            },
        );
        settle(&mut g);
        assert_eq!(g.state.players[p.index()].library[0], victim);
        assert!(!g.state.players[p.other().index()].library.contains(&victim));
        assert_eq!(g.object_view(spell).zone, Zone::Graveyard);
    }
}
#[test]
fn msh_parity_paid_strat_nonland_discard_exile_replacement_still_pays_connive() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        paid(&mut g, p, "Rest in Peace", &[]);
        let victim = named(&mut g, p.other(), "Hill Giant", Zone::Battlefield);
        let own = named(&mut g, p, "Bear Cub", Zone::Battlefield);
        let discard = named(&mut g, p, "Shock", Zone::Hand);
        let spell = strat(&mut g, p, victim, Some(own));
        until(&mut g, |g| {
            matches!(g.state.pending, Pending::TargetLibrary { .. })
        });
        let object = g.object_ref(victim);
        checked(
            &mut g,
            Action::ChooseLibraryDestination {
                object,
                destination: LibraryDestination::SecondFromTop,
            },
        );
        until(&mut g, |g| {
            matches!(g.state.pending, Pending::EffectDiscard { .. })
        });
        checked(&mut g, Action::Discard { card: discard });
        settle(&mut g);
        assert_eq!(g.object_view(discard).zone, Zone::Exile);
        assert_eq!(g.counter_count(own, CounterKind::PlusOne), 1);
        assert_eq!(g.object_view(spell).zone, Zone::Exile);
    }
}
