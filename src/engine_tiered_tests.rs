use super::tests::{scenario, spawn};
use super::*;

fn named(g: &mut Game, p: Player, name: &str, zone: Zone) -> CardId {
    let card = Card::from_name(name).expect(name);
    assert!(card.is_supported(), "{name}");
    spawn(g, p, card, zone)
}
fn checked(g: &mut Game, action: Action) {
    let actor = g.actor().unwrap();
    let revision = g.revision();
    let index = g
        .legal_actions()
        .iter()
        .position(|a| *a == action)
        .unwrap_or_else(|| {
            panic!(
                "illegal fixture {action:?} at {:?}; legal {:?}",
                g.state.pending,
                g.legal_actions()
            )
        });
    let before = g.snapshot();
    assert_eq!(
        g.apply(actor.other(), action.clone()),
        Err(Error::WrongActor)
    );
    assert_eq!(g.snapshot(), before);
    let mut branch = {
        let parent = g.clone();
        parent.clone()
    };
    branch.apply_index(actor, revision, index).unwrap();
    g.apply(actor, action.clone()).unwrap();
    assert_eq!(g.snapshot(), branch.snapshot());
    for player in [Player::P0, Player::P1] {
        assert_eq!(
            serde_json::to_value(g.observe(player)).unwrap(),
            serde_json::to_value(branch.observe(player)).unwrap()
        );
    }
    let after = g.snapshot();
    g.restore(&before).unwrap();
    assert_eq!(
        g.apply_index(actor, revision, index),
        Err(Error::StaleRevision)
    );
    g.apply(actor, action).unwrap();
    assert_eq!(g.snapshot(), after);
}
fn settle_on(g: &mut Game, chosen: Option<Target>) {
    g.prepare_triggers();
    for _ in 0..100 {
        if g.state.pending == Pending::Priority
            && g.state.stack.is_empty()
            && g.state.announcing.is_none()
            && g.state.resolving.is_none()
            && g.state.pending_triggers.is_empty()
        {
            return;
        }
        let actions = g.legal_actions();
        let action = if let Some(target) =
            chosen.filter(|t| actions.contains(&Action::ChooseTarget { target: *t }))
        {
            Action::ChooseTarget { target }
        } else if g.state.pending == Pending::Priority {
            Action::Pass
        } else {
            actions
                .iter()
                .find(|a| matches!(a, Action::PayCosts { .. }))
                .or_else(|| actions.iter().find(|a| matches!(a, Action::FinishTargets)))
                .or_else(|| actions.iter().find(|a| !matches!(a, Action::Concede)))
                .unwrap()
                .clone()
        };
        checked(g, action);
    }
    panic!("unsettled {:?}", g.state.pending);
}
fn setup(p: Player) -> Game {
    let mut g = scenario();
    g.state.active = p;
    g.state.priority = p;
    for state in &mut g.state.players {
        state.mana = [30; 6];
    }
    g
}

fn settle(g: &mut Game) {
    settle_on(g, None);
}
fn reject(g: &mut Game, a: Action) {
    let before = g.snapshot();
    let rev = g.revision();
    assert!(g.apply(g.actor().unwrap(), a).is_err());
    assert_eq!(g.snapshot(), before);
    assert_eq!(g.revision(), rev);
}
fn cast(g: &mut Game, id: CardId, mode: Option<u16>, targets: &[Target]) {
    checked(g, Action::AnnounceCast { card: id });
    if let Some(mode) = mode {
        reject(g, Action::ChooseMode { mode: 3 });
        checked(g, Action::ChooseMode { mode });
    }
    for &target in targets {
        for _ in 0..3 {
            if g.legal_actions().contains(&Action::ChooseTarget { target }) {
                break;
            }
            checked(g, Action::FinishTargets);
        }
        checked(g, Action::ChooseTarget { target });
    }
    for _ in 0..8 {
        if g.state.announcing.is_none() {
            return;
        }
        let a = g
            .legal_actions()
            .into_iter()
            .find(|a| matches!(a, Action::FinishTargets))
            .or_else(|| {
                g.legal_actions()
                    .into_iter()
                    .find(|a| matches!(a, Action::PayCosts { .. }))
            })
            .unwrap();
        checked(g, a);
    }
    panic!("unpaid");
}

fn mana_sum(mana: Mana) -> u64 {
    mana.into_iter().map(u64::from).sum()
}
#[test]
fn all_eighteen_tiers_are_paid_and_enforced_in_both_seats() {
    for p in [Player::P0, Player::P1] {
        for name in [
            "Fire Magic",
            "Ice Magic",
            "Restoration Magic",
            "Thunder Magic",
            "Tifa's Limit Break",
            "Vincent's Limit Break",
        ] {
            for mode in 0..3u16 {
                let mut g = setup(p);
                let q = p.other();
                let bear = named(&mut g, p, "Grizzly Bears", Zone::Battlefield);
                let enemy = named(&mut g, q, "Grizzly Bears", Zone::Battlefield);
                let land = named(&mut g, p, "Forest", Zone::Battlefield);
                let id = named(&mut g, p, name, Zone::Hand);
                let card = g.state.objects[id as usize].card;
                let (base, generic) = card.cost();
                let extra = card.tiered_costs().unwrap()[mode as usize];
                let before = g.state.players[p.index()].mana;
                checked(&mut g, Action::AnnounceCast { card: id });
                assert_eq!(g.state.pending, Pending::Modes { player: p });
                assert!(g.observe(p).payment_request.is_none());
                assert!(g.observe(q).mode_choices.is_empty());
                let views = g.observe(p).mode_choices;
                assert_eq!(views.len(), 3);
                assert_eq!(views[mode as usize].additional_mana, Some(extra));
                reject(&mut g, Action::ChooseMode { mode: 3 });
                checked(&mut g, Action::ChooseMode { mode });
                let target = match name {
                    "Fire Magic" => None,
                    "Restoration Magic" if mode == 2 => None,
                    "Restoration Magic" => Some(land),
                    "Thunder Magic" => Some(enemy),
                    _ => Some(bear),
                };
                if let Some(target) = target {
                    checked(
                        &mut g,
                        Action::ChooseTarget {
                            target: Target::Card(target),
                        },
                    );
                }
                for _ in 0..6 {
                    if g.state.announcing.is_none() {
                        break;
                    }
                    let a = g
                        .legal_actions()
                        .into_iter()
                        .find(|a| matches!(a, Action::FinishTargets))
                        .or_else(|| {
                            g.legal_actions()
                                .into_iter()
                                .find(|a| matches!(a, Action::PayCosts { .. }))
                        })
                        .unwrap();
                    checked(&mut g, a);
                }
                let item = g.state.stack.last().unwrap();
                assert_eq!(item.mode, Some(mode));
                assert_eq!(
                    item.cast
                        .unwrap()
                        .payment
                        .into_iter()
                        .map(u64::from)
                        .sum::<u64>(),
                    base.into_iter().map(u64::from).sum::<u64>()
                        + u64::from(generic)
                        + mana_sum(extra.colored)
                        + u64::from(extra.generic)
                );
                assert_eq!(
                    mana_sum(before) - mana_sum(g.state.players[p.index()].mana),
                    mana_sum(item.cast.unwrap().payment)
                );
                settle(&mut g);
                match name {
                    "Fire Magic" => {
                        for b in [bear, enemy] {
                            if mode == 0 {
                                assert_eq!(g.state.objects[b as usize].damage, 1);
                                assert!(g.on_battlefield(b));
                            } else {
                                assert_eq!(g.state.objects[b as usize].zone, Zone::Graveyard);
                            }
                        }
                    }
                    "Ice Magic" => {
                        assert_eq!(
                            g.state.objects[bear as usize].zone,
                            if mode == 0 { Zone::Hand } else { Zone::Library }
                        );
                        if mode == 1 {
                            assert!(g.observe(p).last_target_library_move.is_some());
                        }
                        if mode == 2 {
                            assert_eq!(
                                g.observe(q).last_target_library_move.unwrap().destination,
                                LibraryDestination::Shuffle
                            );
                        }
                    }
                    "Restoration Magic" => {
                        assert!(
                            g.keywords(land)
                                .contains(Keywords::HEXPROOF.union(Keywords::INDESTRUCTIBLE))
                        );
                        assert_eq!(g.keywords(bear).contains(Keywords::HEXPROOF), mode == 2);
                        assert!(!g.keywords(enemy).contains(Keywords::INDESTRUCTIBLE));
                        assert_eq!(
                            g.state.players[p.index()].life,
                            20 + [0, 3, 6][mode as usize]
                        );
                    }
                    "Thunder Magic" => {
                        assert_eq!(g.state.objects[enemy as usize].zone, Zone::Graveyard)
                    }
                    "Tifa's Limit Break" => {
                        let expected = [4, 4, 6][mode as usize];
                        assert_eq!(
                            g.stats(bear),
                            Stats {
                                power: expected,
                                toughness: expected
                            }
                        );
                        g.finish_cleanup();
                        assert_eq!(
                            g.stats(bear),
                            Stats {
                                power: 2,
                                toughness: 2
                            }
                        );
                    }
                    "Vincent's Limit Break" => {
                        assert_eq!(
                            g.stats(bear),
                            Stats {
                                power: [3, 5, 7][mode as usize],
                                toughness: 2
                            }
                        );
                        let old = g.object_ref(bear);
                        g.state.priority = q;
                        let murder = named(&mut g, q, "Murder", Zone::Hand);
                        cast(&mut g, murder, None, &[Target::Card(bear)]);
                        settle(&mut g);
                        assert_ne!(g.object_ref(bear), old);
                        assert!(g.on_battlefield(bear));
                        assert!(g.state.objects[bear as usize].tapped);
                        assert_eq!(g.state.objects[bear as usize].controller, p);
                        assert_eq!(
                            g.stats(bear),
                            Stats {
                                power: 2,
                                toughness: 2
                            }
                        );
                    }
                    _ => unreachable!(),
                }
            }
        }
    }
}
#[test]
fn colored_tiers_are_mandatory_and_cannot_be_reselected_or_skipped() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let target = named(&mut g, p, "Grizzly Bears", Zone::Battlefield);
        let ice = named(&mut g, p, "Ice Magic", Zone::Hand);
        g.state.players[p.index()].mana = [0, 1, 0, 0, 0, 20];
        checked(&mut g, Action::AnnounceCast { card: ice });
        assert_eq!(g.observe(p).mode_choices.len(), 2);
        reject(&mut g, Action::ChooseMode { mode: 2 });
        reject(
            &mut g,
            Action::PayCosts {
                payment: [0, 1, 0, 0, 0, 1],
            },
        );
        checked(&mut g, Action::ChooseMode { mode: 1 });
        reject(&mut g, Action::ChooseMode { mode: 0 });
        checked(
            &mut g,
            Action::ChooseTarget {
                target: Target::Card(target),
            },
        );
        if g.legal_actions().contains(&Action::FinishTargets) {
            checked(&mut g, Action::FinishTargets);
        }
        let request = g.observe(p).payment_request.unwrap();
        assert_eq!(request.required, [0, 1, 0, 0, 0, 0]);
        assert_eq!(request.generic, 3);
        reject(
            &mut g,
            Action::PayCosts {
                payment: [0, 1, 0, 0, 0, 1],
            },
        );
        let payment = g
            .legal_actions()
            .into_iter()
            .find(|a| matches!(a, Action::PayCosts { .. }))
            .unwrap();
        checked(&mut g, payment);
        settle(&mut g);
        assert_eq!(g.state.objects[target as usize].zone, Zone::Library);
    }
}
#[test]
fn plotted_free_base_still_pays_colored_and_generic_tier() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let id = named(&mut g, p, "Ice Magic", Zone::Exile);
        named(&mut g, p, "Grizzly Bears", Zone::Battlefield);
        let object = g.object_ref(id);
        let turn = g.state.turn;
        g.state
            .plot
            .get_or_insert_with(Default::default)
            .cards
            .push(PlottedCard {
                object,
                owner: p,
                plotted_turn: turn - 1,
            });
        g.state.players[p.index()].mana = [0, 0, 0, 0, 0, 20];
        checked(&mut g, Action::AnnouncePlotted { card: id });
        assert_eq!(g.observe(p).mode_choices.len(), 2);
        reject(&mut g, Action::ChooseMode { mode: 2 });
        checked(&mut g, Action::ChooseMode { mode: 1 });
        let target = g
            .legal_actions()
            .into_iter()
            .find(|a| matches!(a, Action::ChooseTarget { .. }))
            .unwrap();
        checked(&mut g, target);
        if g.legal_actions().contains(&Action::FinishTargets) {
            checked(&mut g, Action::FinishTargets);
        }
        let request = g.observe(p).payment_request.unwrap();
        assert_eq!(request.required, [0; 6]);
        assert_eq!(request.generic, 2);
        reject(&mut g, Action::PayCosts { payment: [0; 6] });
        let payment = g
            .legal_actions()
            .into_iter()
            .find(|a| matches!(a, Action::PayCosts { .. }))
            .unwrap();
        checked(&mut g, payment);
        assert_eq!(
            mana_sum(g.state.stack.last().unwrap().cast.unwrap().payment),
            2
        );
        settle(&mut g);
    }
}
#[test]
fn multiply_uses_signed_resolution_snapshot_and_does_not_scale_later_changes() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let b = named(&mut g, p, "Grizzly Bears", Zone::Battlefield);
        let weak = named(&mut g, p, "Grizzly Bears", Zone::Battlefield);
        g.state.objects[weak as usize].growth = Stats {
            power: -4,
            toughness: 2,
        };
        let spell = named(&mut g, p, "Tifa's Limit Break", Zone::Hand);
        cast(&mut g, spell, Some(2), &[Target::Card(weak)]);
        settle(&mut g);
        assert_eq!(
            g.stats(weak),
            Stats {
                power: -6,
                toughness: 12
            }
        );
        let growth = named(&mut g, p, "Giant Growth", Zone::Hand);
        cast(&mut g, growth, None, &[Target::Card(weak)]);
        settle(&mut g);
        assert_eq!(
            g.stats(weak),
            Stats {
                power: -3,
                toughness: 15
            }
        );
        let zero = named(&mut g, p, "Multiply by Zero", Zone::Hand);
        cast(&mut g, zero, None, &[Target::Card(weak)]);
        settle(&mut g);
        assert_eq!(
            g.stats(weak),
            Stats {
                power: -5,
                toughness: 13
            }
        );
        assert_eq!(
            g.stats(b),
            Stats {
                power: 2,
                toughness: 2
            }
        );
        g.finish_cleanup();
        assert_eq!(
            g.stats(weak),
            Stats {
                power: 2,
                toughness: 2
            }
        );
    }
}

#[test]
fn adjacent_boros_and_epic_fight_have_all_three_complete_legal_choices_both_seats() {
    for p in [Player::P0, Player::P1] {
        for mode in 0..3u16 {
            let mut g = setup(p);
            let q = p.other();
            let ally = named(&mut g, p, "Grizzly Bears", Zone::Battlefield);
            let land = named(&mut g, p, "Forest", Zone::Battlefield);
            let enemy = named(&mut g, q, "Grizzly Bears", Zone::Battlefield);
            let id = named(&mut g, p, "Boros Charm", Zone::Hand);
            let targets = match mode {
                0 => vec![Target::Player(q)],
                1 => vec![],
                _ => vec![Target::Card(ally)],
            };
            cast(&mut g, id, Some(mode), &targets);
            settle(&mut g);
            match mode {
                0 => assert_eq!(g.state.players[q.index()].life, 16),
                1 => {
                    assert!(g.keywords(ally).contains(Keywords::INDESTRUCTIBLE));
                    assert!(g.keywords(land).contains(Keywords::INDESTRUCTIBLE));
                    assert!(!g.keywords(enemy).contains(Keywords::INDESTRUCTIBLE));
                    let later = named(&mut g, p, "Grizzly Bears", Zone::Hand);
                    cast(&mut g, later, None, &[]);
                    settle(&mut g);
                    assert!(!g.keywords(later).contains(Keywords::INDESTRUCTIBLE));
                }
                _ => assert!(g.keywords(ally).contains(Keywords::DOUBLE_STRIKE)),
            };
            let mut g = setup(p);
            let ally = named(&mut g, p, "Grizzly Bears", Zone::Battlefield);
            let enemy = named(&mut g, q, "Grizzly Bears", Zone::Battlefield);
            let other = named(&mut g, q, "Grizzly Bears", Zone::Battlefield);
            let id = named(&mut g, p, "Epic Fight", Zone::Hand);
            let targets = match mode {
                0 => vec![Target::Card(ally)],
                1 => vec![Target::Card(ally), Target::Card(enemy)],
                _ => vec![Target::Card(ally), Target::Card(ally), Target::Card(enemy)],
            };
            cast(&mut g, id, Some(mode), &targets);
            settle(&mut g);
            assert_eq!(
                g.stats(other),
                Stats {
                    power: 2,
                    toughness: 2
                }
            );
            if mode == 0 {
                assert_eq!(
                    g.stats(ally),
                    Stats {
                        power: 4,
                        toughness: 4
                    }
                );
                assert!(g.on_battlefield(enemy));
            } else {
                assert_eq!(g.state.objects[enemy as usize].zone, Zone::Graveyard);
                if mode == 1 {
                    assert_eq!(g.state.objects[ally as usize].zone, Zone::Graveyard);
                } else {
                    assert_eq!(
                        g.stats(ally),
                        Stats {
                            power: 4,
                            toughness: 4
                        }
                    );
                    assert_eq!(g.state.objects[ally as usize].damage, 2);
                }
            }
        }
    }
}
#[test]
fn mode_combination_keeps_illegal_double_group_from_touching_other_fight_targets() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let q = p.other();
        let double = named(&mut g, q, "Grizzly Bears", Zone::Battlefield);
        let fight = named(&mut g, p, "Grizzly Bears", Zone::Battlefield);
        let enemy = named(&mut g, q, "Grizzly Bears", Zone::Battlefield);
        let id = named(&mut g, p, "Epic Fight", Zone::Hand);
        cast(
            &mut g,
            id,
            Some(2),
            &[
                Target::Card(double),
                Target::Card(fight),
                Target::Card(enemy),
            ],
        );
        g.state.objects[double as usize].gained_keywords = Keywords::HEXPROOF;
        settle(&mut g);
        assert_eq!(
            g.stats(double),
            Stats {
                power: 2,
                toughness: 2
            }
        );
        assert_eq!(g.state.objects[fight as usize].zone, Zone::Graveyard);
        assert_eq!(g.state.objects[enemy as usize].zone, Zone::Graveyard);
    }
}
#[test]
fn tier_cost_combines_before_power_reduction_and_locks_before_mana_sacrifice() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let sword = named(
            &mut g,
            p,
            "Glamdring, Foe-hammer // Gleam of Death",
            Zone::Battlefield,
        );
        let bunny = named(&mut g, p, "Regal Bunnicorn", Zone::Battlefield);
        let treasure = Card::catalogue()
            .find(|c| c.token_info().is_some_and(|i| i.name == "Treasure Token"))
            .unwrap();
        let token = spawn(&mut g, p, treasure, Zone::Battlefield);
        g.state.objects[bunny as usize]
            .counters
            .add(CounterKind::MinusOne, 1);
        let ability = g.state.objects[sword as usize]
            .card
            .activations()
            .iter()
            .position(|a| a.program.contains(&Instruction::AttachSource))
            .unwrap();
        checked(
            &mut g,
            Action::Activate {
                card: sword,
                ability: ability as u16,
                target: Some(Target::Card(bunny)),
                payment: [0, 0, 0, 0, 0, 2],
                counter_payment: Counters::default(),
            },
        );
        settle(&mut g);
        g.state.priority = p;
        assert_eq!(g.stats(bunny).power, 2);
        let spell = named(&mut g, p, "Ice Magic", Zone::Hand);
        g.state.players[p.index()].mana = [0, 1, 0, 0, 0, 4];
        checked(&mut g, Action::AnnounceCast { card: spell });
        checked(&mut g, Action::ChooseMode { mode: 2 });
        checked(
            &mut g,
            Action::ChooseTarget {
                target: Target::Card(bunny),
            },
        );
        if g.legal_actions().contains(&Action::FinishTargets) {
            checked(&mut g, Action::FinishTargets);
        }
        let request = g.observe(p).payment_request.unwrap();
        assert_eq!(request.generic, 4);
        assert_eq!(request.required, [0, 2, 0, 0, 0, 0]);
        let a=g.legal_actions().into_iter().find(|a|matches!(a,Action::ActivateMana{card,production,..}if *card==token&&*production==[0,1,0,0,0,0])).unwrap();
        checked(&mut g, a);
        assert_eq!(g.stats(bunny).power, 1);
        assert_eq!(g.observe(p).payment_request.unwrap().generic, 4);
        let a = g
            .legal_actions()
            .into_iter()
            .find(|a| matches!(a, Action::PayCosts { .. }))
            .unwrap();
        checked(&mut g, a);
        assert_eq!(
            g.state.stack.last().unwrap().source_card.fixed_mana_value(),
            2
        );
        settle(&mut g);
        assert_eq!(g.state.objects[bunny as usize].zone, Zone::Library);
    }
}
#[test]
fn owner_library_choice_is_private_and_foreign_control_shuffle_uses_owner_rng_library() {
    for p in [Player::P0, Player::P1] {
        let q = p.other();
        for mode in [1, 2] {
            let mut g = setup(p);
            let target = named(&mut g, q, "Grizzly Bears", Zone::Battlefield);
            g.state.objects[target as usize].controller = p;
            let id = named(&mut g, p, "Ice Magic", Zone::Hand);
            cast(&mut g, id, Some(mode), &[Target::Card(target)]);
            let prior = g.state.players[p.index()].library.clone();
            let mut expected = g.state.players[q.index()].library.clone();
            expected.push(target);
            let mut rng = g.state.rng;
            if mode == 2 {
                rng.shuffle(&mut expected);
                settle(&mut g);
                assert_eq!(g.state.players[q.index()].library, expected);
                assert_eq!(g.state.rng, rng);
            } else {
                for _ in 0..20 {
                    if matches!(g.state.pending, Pending::TargetLibrary { .. }) {
                        break;
                    }
                    checked(&mut g, Action::Pass);
                }
                assert_eq!(g.actor(), Some(q));
                assert!(g.observe(q).target_library.is_some());
                assert!(g.observe(p).target_library.is_none());
                let object = g.object_ref(target);
                checked(
                    &mut g,
                    Action::ChooseLibraryDestination {
                        object,
                        destination: LibraryDestination::Bottom,
                    },
                );
                settle(&mut g);
                assert_eq!(g.state.players[q.index()].library.first(), Some(&target));
            }
            assert_eq!(g.state.players[p.index()].library, prior);
            assert_eq!(g.state.objects[target as usize].owner, q);
        }
    }
}
#[test]
fn all_targets_illegal_never_shuffle_and_multiplier_overflow_is_atomic() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let b = named(&mut g, p.other(), "Grizzly Bears", Zone::Battlefield);
        let id = named(&mut g, p, "Ice Magic", Zone::Hand);
        cast(&mut g, id, Some(2), &[Target::Card(b)]);
        g.state.objects[b as usize].gained_keywords = Keywords::HEXPROOF;
        let rng = g.state.rng;
        let lib = g.state.players[p.other().index()].library.clone();
        settle(&mut g);
        assert_eq!(g.state.rng, rng);
        assert_eq!(g.state.players[p.other().index()].library, lib);
        assert!(g.on_battlefield(b));
        let mut g = setup(p);
        let b = named(&mut g, p, "Grizzly Bears", Zone::Battlefield);
        g.state.objects[b as usize].growth.power = i128::MAX / 2;
        let id = named(&mut g, p, "Tifa's Limit Break", Zone::Hand);
        cast(&mut g, id, Some(2), &[Target::Card(b)]);
        checked(&mut g, Action::Pass);
        let before = g.snapshot();
        let rev = g.revision();
        let actor = g.actor().unwrap();
        assert!(matches!(
            g.apply(actor, Action::Pass),
            Err(Error::CapacityExceeded(_))
        ));
        assert_eq!(g.snapshot(), before);
        assert_eq!(g.revision(), rev);
    }
}
#[test]
fn vincent_foreign_owner_duplicate_grants_replacements_and_cleanup_keep_exact_incarnations() {
    for p in [Player::P0, Player::P1] {
        for exile in [false, true] {
            let mut g = setup(p);
            let q = p.other();
            let bear = named(&mut g, q, "Grizzly Bears", Zone::Battlefield);
            g.state.objects[bear as usize].controller = p;
            for mode in [2, 0] {
                let spell = named(&mut g, p, "Vincent's Limit Break", Zone::Hand);
                cast(&mut g, spell, Some(mode), &[Target::Card(bear)]);
                settle(&mut g);
            }
            assert_eq!(
                g.stats(bear),
                Stats {
                    power: 3,
                    toughness: 2
                }
            );
            assert_eq!(g.observe(p).temporary_abilities.len(), 2);
            let old = g.object_ref(bear);
            g.state.priority = q;
            let removal = named(
                &mut g,
                q,
                if exile { "Pinecone Strike" } else { "Murder" },
                Zone::Hand,
            );
            cast(&mut g, removal, exile.then_some(0), &[Target::Card(bear)]);
            settle(&mut g);
            assert_ne!(g.object_ref(bear), old);
            assert!(g.observe(p).temporary_abilities.is_empty());
            if exile {
                assert_eq!(g.state.objects[bear as usize].zone, Zone::Exile);
            } else {
                assert!(g.on_battlefield(bear));
                assert_eq!(g.state.objects[bear as usize].owner, q);
                assert_eq!(g.state.objects[bear as usize].controller, q);
                assert!(g.state.objects[bear as usize].tapped);
                assert_eq!(
                    g.stats(bear),
                    Stats {
                        power: 2,
                        toughness: 2
                    }
                );
                g.state.priority = p;
                let again = named(&mut g, p, "Murder", Zone::Hand);
                cast(&mut g, again, None, &[Target::Card(bear)]);
                settle(&mut g);
                assert_eq!(g.state.objects[bear as usize].zone, Zone::Graveyard);
            }
        }
        let mut g = setup(p);
        let bear = named(&mut g, p, "Grizzly Bears", Zone::Battlefield);
        let spell = named(&mut g, p, "Vincent's Limit Break", Zone::Hand);
        cast(&mut g, spell, Some(2), &[Target::Card(bear)]);
        settle(&mut g);
        g.finish_cleanup();
        assert_eq!(
            g.stats(bear),
            Stats {
                power: 2,
                toughness: 2
            }
        );
        assert!(g.observe(p).temporary_abilities.is_empty());
        let next = p.other();
        assert_eq!(g.actor(), Some(next));
        g.state.players[next.index()].mana = [30; 6];
        let kill = named(&mut g, next, "Murder", Zone::Hand);
        cast(&mut g, kill, None, &[Target::Card(bear)]);
        settle(&mut g);
        assert_eq!(g.state.objects[bear as usize].zone, Zone::Graveyard);
    }
}
#[test]
fn real_paid_korvold_foreign_tiered_sources_charge_actual_caster_all_six_bodies() {
    for p in [Player::P0, Player::P1] {
        let q = p.other();
        for name in [
            "Fire Magic",
            "Ice Magic",
            "Restoration Magic",
            "Thunder Magic",
            "Tifa's Limit Break",
            "Vincent's Limit Break",
        ] {
            let mut g = setup(p);
            let sources = [0; 3].map(|_| named(&mut g, q, name, Zone::Library));
            for _ in 0..2 {
                named(&mut g, q, "Forest", Zone::Library);
            }
            let saga = named(&mut g, p, "Korvold and the Noble Thief", Zone::Hand);
            cast(&mut g, saga, None, &[]);
            settle(&mut g);
            for _ in 0..1500 {
                if sources.iter().all(|&id| g.can_play_exiled(p, id)) {
                    break;
                }
                let actions = g.legal_actions();
                let action = actions
                    .iter()
                    .find(|a| matches!(a, Action::Pass | Action::FinishTargets))
                    .or_else(|| actions.iter().find(|a| !matches!(a, Action::Concede)))
                    .unwrap()
                    .clone();
                checked(&mut g, action);
            }
            assert_eq!(g.state.objects[saga as usize].zone, Zone::Graveyard);
            assert!(
                sources
                    .iter()
                    .all(|&id| g.can_play_exiled(p, id) && !g.can_play_exiled(q, id))
            );
            for state in &mut g.state.players {
                state.mana = [30; 6];
            }
            let ally = named(&mut g, p, "Grizzly Bears", Zone::Battlefield);
            let enemy = named(&mut g, q, "Grizzly Bears", Zone::Battlefield);
            let q_mana = g.state.players[q.index()].mana;
            let source = sources[0];
            checked(&mut g, Action::AnnounceExile { card: source });
            checked(&mut g, Action::ChooseMode { mode: 2 });
            let target = match name {
                "Fire Magic" | "Restoration Magic" => None,
                "Ice Magic" | "Thunder Magic" => Some(enemy),
                _ => Some(ally),
            };
            if let Some(id) = target {
                checked(
                    &mut g,
                    Action::ChooseTarget {
                        target: Target::Card(id),
                    },
                );
            }
            for _ in 0..8 {
                if g.state.announcing.is_none() {
                    break;
                }
                let actions = g.legal_actions();
                let action = actions
                    .iter()
                    .find(|a| matches!(a, Action::FinishTargets))
                    .or_else(|| {
                        actions
                            .iter()
                            .find(|a| matches!(a, Action::PayCosts { .. }))
                    })
                    .unwrap()
                    .clone();
                checked(&mut g, action);
            }
            let item = g.state.stack.last().unwrap();
            assert_eq!(item.controller, p);
            assert_eq!(item.mode, Some(2));
            assert_eq!(g.state.objects[source as usize].owner, q);
            assert!(mana_sum(item.cast.unwrap().payment) > 0);
            settle(&mut g);
            assert_eq!(g.state.objects[source as usize].zone, Zone::Graveyard);
            assert_eq!(g.state.objects[source as usize].owner, q);
            assert_eq!(g.state.players[q.index()].mana, q_mana);
            if name == "Vincent's Limit Break" {
                assert_eq!(
                    g.stats(ally),
                    Stats {
                        power: 7,
                        toughness: 2
                    }
                );
            }
        }
    }
}

#[test]
fn mode_funding_never_pairs_an_unfunded_nontargeted_tier_with_cheaper_missing_targets() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let source = named(&mut g, p, "Restoration Magic", Zone::Hand);
        g.state.players[p.index()].mana = [1, 0, 0, 0, 0, 0];
        assert!(
            !g.legal_actions()
                .contains(&Action::AnnounceCast { card: source })
        );
        reject(&mut g, Action::AnnounceCast { card: source });
        g.state.players[p.index()].mana = [2, 0, 0, 0, 0, 3];
        checked(&mut g, Action::AnnounceCast { card: source });
        assert_eq!(
            g.observe(p)
                .mode_choices
                .iter()
                .map(|v| v.mode)
                .collect::<Vec<_>>(),
            vec![2]
        );
        reject(&mut g, Action::ChooseMode { mode: 0 });
        checked(&mut g, Action::ChooseMode { mode: 2 });
        let request = g.observe(p).payment_request.unwrap();
        assert_eq!(request.required, [2, 0, 0, 0, 0, 0]);
        assert_eq!(request.generic, 3);
        let action = g
            .legal_actions()
            .into_iter()
            .find(|a| matches!(a, Action::PayCosts { .. }))
            .unwrap();
        checked(&mut g, action);
        settle(&mut g);
        assert_eq!(g.state.players[p.index()].life, 26);
    }
}

#[test]
fn paid_genemorph_resolution_setter_and_tiered_multiply_vincent_return_compose() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let bear = named(&mut g, p, "Grizzly Bears", Zone::Battlefield);
        for _ in 0..5 {
            named(&mut g, p, "Forest", Zone::Battlefield);
        }
        let imago = named(&mut g, p, "Genemorph Imago", Zone::Hand);
        cast(&mut g, imago, None, &[]);
        settle(&mut g);
        let double = named(&mut g, p, "Tifa's Limit Break", Zone::Hand);
        cast(&mut g, double, Some(1), &[Target::Card(bear)]);
        settle(&mut g);
        assert_eq!(
            g.stats(bear),
            Stats {
                power: 4,
                toughness: 4
            }
        );
        let land = named(&mut g, p, "Forest", Zone::Hand);
        checked(&mut g, Action::PlayLand { card: land });
        settle_on(&mut g, Some(Target::Card(bear)));
        assert_eq!(
            g.stats(bear),
            Stats {
                power: 8,
                toughness: 8
            }
        );
        let triple = named(&mut g, p, "Tifa's Limit Break", Zone::Hand);
        cast(&mut g, triple, Some(2), &[Target::Card(bear)]);
        settle(&mut g);
        assert_eq!(
            g.stats(bear),
            Stats {
                power: 24,
                toughness: 24
            }
        );
        let vincent = named(&mut g, p, "Vincent's Limit Break", Zone::Hand);
        cast(&mut g, vincent, Some(0), &[Target::Card(bear)]);
        settle(&mut g);
        assert_eq!(
            g.stats(bear),
            Stats {
                power: 21,
                toughness: 20
            }
        );
        let old = g.object_ref(bear);
        let murder = named(&mut g, p, "Murder", Zone::Hand);
        cast(&mut g, murder, None, &[Target::Card(bear)]);
        settle(&mut g);
        assert_ne!(g.object_ref(bear), old);
        assert!(g.on_battlefield(bear));
        assert_eq!(
            g.stats(bear),
            Stats {
                power: 2,
                toughness: 2
            }
        );
    }
}

#[path = "engine_direct_cards_tests.rs"]
mod direct_cards;
