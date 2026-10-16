use super::tests::{act, cast, pass_pair, scenario, spawn};
use super::*;

const NAMES: [&str; 6] = [
    "Brambleback Brute",
    "Sandstorm Verge",
    "Sower of Chaos",
    "Summit Intimidator",
    "War Squeak",
    "Zog, Triceraton Castaway",
];
fn named(g: &mut Game, p: Player, name: &str, z: Zone) -> CardId {
    let card = Card::from_name(name).unwrap();
    assert!(card.is_supported(), "{name}");
    spawn(g, p, card, z)
}
fn setup(p: Player) -> Game {
    let mut g = scenario();
    g.state.active = p;
    g.state.priority = p;
    for s in &mut g.state.players {
        s.mana = [20; 6];
    }
    g
}
fn replay(g: &mut Game, a: Action) {
    let saved = g.snapshot();
    let revision = g.revision();
    let p = g.actor().unwrap();
    let mut fork = g.clone();
    act(&mut fork, a.clone());
    act(g, a.clone());
    let expected = g.snapshot();
    assert_eq!(expected, fork.snapshot());
    g.restore(&saved).unwrap();
    assert_eq!(g.apply_index(p, revision, 0), Err(Error::StaleRevision));
    act(g, a);
    assert_eq!(g.snapshot(), expected);
}
fn reject(g: &mut Game, a: Action) {
    let saved = g.snapshot();
    let revision = g.revision();
    assert!(g.apply(g.actor().unwrap(), a).is_err());
    assert_eq!(g.snapshot(), saved);
    assert_eq!(g.revision(), revision);
}
fn settle(g: &mut Game, target: CardId) {
    for _ in 0..40 {
        if g.state.stack.is_empty() && g.state.pending == Pending::Priority {
            return;
        }
        if g.state.announcing.as_ref().is_some_and(|a| {
            a.item.source_card.name() == "War Squeak"
                && matches!(a.item.kind, StackKind::TriggeredAbility { .. })
        }) {
            let owner = g.state.announcing.as_ref().unwrap().item.controller;
            if let Some(id) = g.state.objects.iter().enumerate().find_map(|(i, o)| {
                (o.zone == Zone::Battlefield && o.owner == owner && o.card.is_creature())
                    .then_some(i as CardId)
            }) {
                let wrong = Action::ChooseTarget {
                    target: Target::Card(id),
                };
                assert!(!g.legal_actions().contains(&wrong));
                reject(g, wrong);
            }
        }
        let a = g
            .legal_actions()
            .into_iter()
            .find(|a| match a {
                Action::ChooseTarget {
                    target: Target::Card(id),
                } => *id == target,
                Action::PutTrigger { .. } | Action::Pass => true,
                _ => false,
            })
            .expect("fixture settlement");
        replay(g, a);
    }
    panic!("not settled");
}
fn slot(g: &Game, id: CardId) -> u16 {
    g.object_view(id)
        .card
        .activations()
        .iter()
        .position(|a| {
            a.program.contains(&Instruction::GrantKeywords {
                subject: Subject::Targets,
                keywords: Keywords::CANT_BLOCK,
            })
        })
        .unwrap() as u16
}
fn activate(g: &mut Game, id: CardId, target: CardId) {
    let slot = slot(g, id);
    let a = g
        .legal_actions()
        .into_iter()
        .find(|a| matches!(a,Action::AnnounceActivate{card,ability,..}if *card==id&&*ability==slot))
        .expect("activation");
    replay(g, a);
    replay(
        g,
        Action::ChooseTarget {
            target: Target::Card(target),
        },
    );
    let a = g
        .legal_actions()
        .into_iter()
        .find(|a| matches!(a, Action::PayCosts { .. }))
        .expect("payment");
    replay(g, a);
}
fn combat(g: &mut Game, p: Player, denied: CardId, allowed: CardId) {
    assert!(g.keywords(denied).contains(Keywords::CANT_BLOCK));
    assert!(!g.keywords(allowed).contains(Keywords::CANT_BLOCK));
    let attacker = named(g, p, "Bear Cub", Zone::Battlefield);
    g.state.step = Step::Attackers;
    g.state.pending = Pending::Attackers;
    replay(g, Action::Attack { card: attacker });
    replay(g, Action::FinishAttackers);
    pass_pair(g);
    let illegal = Action::Block {
        blocker: denied,
        attacker,
    };
    assert!(!g.legal_actions().contains(&illegal));
    reject(g, illegal);
    assert!(g.legal_actions().contains(&Action::Block {
        blocker: allowed,
        attacker
    }));
    replay(g, Action::FinishBlockers);
    let turn = g.state.turn;
    for _ in 0..30 {
        if g.state.turn != turn {
            break;
        }
        replay(g, Action::Pass);
    }
    assert_ne!(g.state.turn, turn);
    assert!(!g.keywords(denied).contains(Keywords::CANT_BLOCK));
}

#[test]
fn complete_activated_bodies_pay_exact_costs_forbid_blocks_then_expire() {
    for p in [Player::P0, Player::P1] {
        for name in &NAMES[..3] {
            let mut g = setup(p);
            let source = named(
                &mut g,
                p,
                name,
                if *name == "Sandstorm Verge" {
                    Zone::Battlefield
                } else {
                    Zone::Hand
                },
            );
            if *name != "Sandstorm Verge" {
                cast(&mut g, source, None);
                pass_pair(&mut g);
            }
            if *name == "Brambleback Brute" {
                let v = g.object_view(source);
                assert_eq!(v.counters.minus_one, 2);
                assert_eq!((v.power, v.toughness), (2, 3));
            }
            let target = named(&mut g, p.other(), "Bear Cub", Zone::Battlefield);
            let other = named(&mut g, p.other(), "Bear Cub", Zone::Battlefield);
            let pool = g.state.players[p.index()].mana;
            activate(&mut g, source, target);
            assert!(!g.keywords(target).contains(Keywords::CANT_BLOCK));
            if *name == "Brambleback Brute" {
                assert_eq!(g.object_view(source).counters.minus_one, 1);
                assert_eq!((g.stats(source).power, g.stats(source).toughness), (3, 4));
            }
            if *name == "Sandstorm Verge" {
                assert!(g.object_view(source).tapped);
            }
            assert_eq!(
                pool.iter().sum::<u32>() - g.state.players[p.index()].mana.iter().sum::<u32>(),
                if *name == "Brambleback Brute" { 2 } else { 3 }
            );
            settle(&mut g, target);
            combat(&mut g, p, target, other);
        }
    }
}

#[test]
fn complete_entry_bodies_attach_aura_preserve_keywords_and_forbid_only_chosen_blocker() {
    for p in [Player::P0, Player::P1] {
        for name in &NAMES[3..] {
            let mut g = setup(p);
            let target = named(&mut g, p.other(), "Bear Cub", Zone::Battlefield);
            let other = named(&mut g, p.other(), "Bear Cub", Zone::Battlefield);
            let own = named(&mut g, p, "Bear Cub", Zone::Battlefield);
            let source = named(&mut g, p, name, Zone::Hand);
            cast(
                &mut g,
                source,
                if *name == "War Squeak" {
                    Some(Target::Card(own))
                } else {
                    None
                },
            );
            settle(&mut g, target);
            let v = g.object_view(source);
            match *name {
                "War Squeak" => {
                    assert_eq!(v.attached_to, Some(g.object_ref(own)));
                    let c = g.object_view(own);
                    assert_eq!((c.power, c.toughness), (3, 3));
                    assert!(c.keywords.contains(Keywords::HASTE));
                }
                "Summit Intimidator" => {
                    assert_eq!((v.power, v.toughness), (4, 3));
                    assert!(v.keywords.contains(Keywords::REACH));
                }
                _ => {
                    assert_eq!((v.power, v.toughness), (5, 4));
                    assert!(
                        v.keywords
                            .contains(Keywords::REACH.union(Keywords::TRAMPLE))
                    );
                }
            }
            combat(&mut g, p, target, other);
        }
    }
}

#[test]
fn sorcery_only_activation_guards_do_not_restrict_instant_sower() {
    for name in &NAMES[..3] {
        let mut g = setup(Player::P0);
        let id = named(&mut g, Player::P0, name, Zone::Battlefield);
        g.state.objects[id as usize].counters.minus_one = 2;
        let target = named(&mut g, Player::P1, "Bear Cub", Zone::Battlefield);
        let ability = slot(&g, id);
        for (active, step, stack) in [
            (Player::P1, Step::Main1, false),
            (Player::P0, Step::Upkeep, false),
            (Player::P0, Step::Main1, true),
        ] {
            g.state.active = active;
            g.state.priority = Player::P0;
            g.state.step = step;
            if stack {
                let card = named(&mut g, Player::P0, "Shock", Zone::Hand);
                cast(&mut g, card, Some(Target::Card(target)));
            }
            let available=g.legal_actions().iter().any(|a|matches!(a,Action::AnnounceActivate{card,ability:a,..}if *card==id&&*a==ability));
            assert_eq!(available, *name == "Sower of Chaos");
        }
    }
}

#[test]
fn repeated_effects_do_not_stack_but_old_target_cannot_bind_a_new_incarnation() {
    let mut g = setup(Player::P0);
    let source = named(&mut g, Player::P0, "Sower of Chaos", Zone::Battlefield);
    let target = named(&mut g, Player::P1, "Bear Cub", Zone::Battlefield);
    activate(&mut g, source, target);
    activate(&mut g, source, target);
    settle(&mut g, target);
    assert_eq!(
        g.state.objects[target as usize].gained_keywords,
        Keywords::CANT_BLOCK
    );
    g.move_object(target, Zone::Hand);
    g.move_object(target, Zone::Battlefield);
    assert!(!g.keywords(target).contains(Keywords::CANT_BLOCK));
    activate(&mut g, source, target);
    let old = g.object_ref(target);
    g.move_object(target, Zone::Hand);
    g.move_object(target, Zone::Battlefield);
    assert_ne!(g.object_ref(target), old);
    settle(&mut g, target);
    assert!(!g.keywords(target).contains(Keywords::CANT_BLOCK));
}

#[test]
fn sandstorm_source_cannot_fund_its_reserved_tap_cost() {
    let mut g = setup(Player::P0);
    let source = named(&mut g, Player::P0, "Sandstorm Verge", Zone::Battlefield);
    let target = named(&mut g, Player::P1, "Bear Cub", Zone::Battlefield);
    let ability = slot(&g, source);
    g.state.players[0].mana = [0; 6];
    for _ in 0..2 {
        named(&mut g, Player::P0, "Mountain", Zone::Battlefield);
    }
    let announce = Action::AnnounceActivate {
        card: source,
        ability,
        counter_payment: Counters::default(),
    };
    reject(&mut g, announce.clone());
    named(&mut g, Player::P0, "Mountain", Zone::Battlefield);
    replay(&mut g, announce);
    replay(
        &mut g,
        Action::ChooseTarget {
            target: Target::Card(target),
        },
    );
    assert!(!g.legal_actions().iter().any(
        |a| matches!(a,Action::ActivateMana{card,..}|Action::AnnounceMana{card,..}if *card==source)
    ));
    for _ in 0..3 {
        let a = g
            .legal_actions()
            .into_iter()
            .find(|a| matches!(a, Action::TapMana { .. }))
            .unwrap();
        replay(&mut g, a);
    }
    let pay = g
        .legal_actions()
        .into_iter()
        .find(|a| matches!(a, Action::PayCosts { .. }))
        .unwrap();
    replay(&mut g, pay);
    assert!(g.object_view(source).tapped);
    settle(&mut g, target);
    assert!(g.keywords(target).contains(Keywords::CANT_BLOCK));
}

#[test]
fn hexproof_and_ward_apply_to_blocking_abilities_without_refunding_source_costs() {
    for paid in [false, true] {
        let mut g = setup(Player::P0);
        let source = named(&mut g, Player::P0, "Sower of Chaos", Zone::Battlefield);
        let hidden = named(&mut g, Player::P1, "Bear Cub", Zone::Battlefield);
        g.state.objects[hidden as usize].gained_keywords = Keywords::HEXPROOF;
        let own = named(&mut g, Player::P0, "Bear Cub", Zone::Battlefield);
        g.state.objects[own as usize].gained_keywords = Keywords::HEXPROOF;
        let target = named(&mut g, Player::P1, "Sleep-Cursed Faerie", Zone::Battlefield);
        let ability = slot(&g, source);
        let a=g.legal_actions().into_iter().find(|a|matches!(a,Action::AnnounceActivate{card,ability:a,..}if *card==source&&*a==ability)).unwrap();
        replay(&mut g, a);
        reject(
            &mut g,
            Action::ChooseTarget {
                target: Target::Card(hidden),
            },
        );
        assert!(g.legal_actions().contains(&Action::ChooseTarget {
            target: Target::Card(own)
        }));
        replay(
            &mut g,
            Action::ChooseTarget {
                target: Target::Card(target),
            },
        );
        let pay = g
            .legal_actions()
            .into_iter()
            .find(|a| matches!(a, Action::PayCosts { .. }))
            .unwrap();
        replay(&mut g, pay);
        let pool = g.state.players[0].mana;
        pass_pair(&mut g);
        assert!(g.observe(Player::P0).resolution_payment.is_some());
        let a = if paid {
            g.legal_actions()
                .into_iter()
                .find(|a| matches!(a, Action::PayResolutionMana { .. }))
                .unwrap()
        } else {
            Action::DeclineResolutionMana
        };
        replay(&mut g, a);
        if paid {
            settle(&mut g, target);
        } else {
            assert_eq!(g.state.players[0].mana, pool);
            assert!(g.state.stack.is_empty());
        }
        assert_eq!(g.keywords(target).contains(Keywords::CANT_BLOCK), paid);
    }
}

#[test]
fn zog_keeps_hand_only_mountaincycling_with_revealed_typed_search_and_replay() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let id = named(&mut g, p, "Zog, Triceraton Castaway", Zone::Hand);
        let mountain = named(&mut g, p, "Commercial District", Zone::Library);
        let wrong = named(&mut g, p, "Wastes", Zone::Library);
        let a = g
            .legal_actions()
            .into_iter()
            .find(|a| matches!(a,Action::Activate{card,..}if *card==id))
            .unwrap();
        replay(&mut g, a);
        assert_eq!(g.object_view(id).zone, Zone::Graveyard);
        pass_pair(&mut g);
        assert!(g.observe(p.other()).search_choice.is_none());
        assert!(
            g.legal_actions()
                .contains(&Action::ChooseSearch { card: mountain })
        );
        reject(&mut g, Action::ChooseSearch { card: wrong });
        replay(&mut g, Action::ChooseSearch { card: mountain });
        replay(&mut g, Action::FinishSearch);
        assert_eq!(g.object_view(mountain).zone, Zone::Hand);
        assert_eq!(g.state.known_hand, vec![g.object_ref(mountain)]);
        assert!(!g.legal_actions().iter().any(
            |a| matches!(a,Action::Activate{card,..}|Action::AnnounceActivate{card,..}if *card==id)
        ));
    }
}

#[test]
fn brambleback_removes_any_supported_counter_kind_and_requires_an_actual_counter() {
    for kind in CounterKind::ALL {
        let mut g = setup(Player::P0);
        let source = named(&mut g, Player::P0, "Brambleback Brute", Zone::Battlefield);
        let target = named(&mut g, Player::P1, "Bear Cub", Zone::Battlefield);
        let ability = slot(&g, source);
        let empty = Action::AnnounceActivate {
            card: source,
            ability,
            counter_payment: Counters::default(),
        };
        reject(&mut g, empty);
        g.state.objects[source as usize].counters.add(kind, 1);
        activate(&mut g, source, target);
        assert_eq!(g.object_view(source).counters, Counters::default());
        settle(&mut g, target);
        assert!(g.keywords(target).contains(Keywords::CANT_BLOCK));
        assert!(
            !g.legal_actions()
                .iter()
                .any(|a| matches!(a,Action::AnnounceActivate{card,..}if *card==source))
        );
    }
}
