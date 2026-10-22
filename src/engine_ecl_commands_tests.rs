//! Every unordered pair of Command modes resolves in printed order with independent targets.
use super::ability_timeline_tests::{checked, named, settle, setup};
use super::*;
fn cast_command(g: &mut Game, card: CardId, mode: Option<u16>, targets: &[Target]) {
    checked(g, Action::AnnounceCast { card });
    let before = g.snapshot();
    let revision = g.revision();
    assert_eq!(
        g.apply(g.actor().unwrap(), Action::ChooseMode { mode: u16::MAX }),
        Err(Error::IllegalAction)
    );
    assert_eq!(g.snapshot(), before);
    assert_eq!(g.revision(), revision);
    checked(
        g,
        Action::ChooseMode {
            mode: mode.unwrap(),
        },
    );
    for &target in targets {
        for _ in 0..4 {
            if g.legal_actions().contains(&Action::ChooseTarget { target }) {
                break;
            }
            checked(g, Action::FinishTargets);
        }
        checked(g, Action::ChooseTarget { target });
    }
    for _ in 0..5 {
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
    panic!("command announcement did not finish");
}
const PAIRS: [(usize, usize); 6] = [(0, 1), (0, 2), (0, 3), (1, 2), (1, 3), (2, 3)];
fn copies(g: &Game, p: Player) -> usize {
    g.state
        .objects
        .iter()
        .filter(|o| {
            o.zone == Zone::Battlefield
                && o.controller == p
                && o.is_token()
                && o.card == Card::from_name("Chitinous Graspling").unwrap()
        })
        .count()
}
fn tokens(g: &Game, p: Player, subtype: &str) -> usize {
    g.state
        .objects
        .iter()
        .enumerate()
        .filter(|(id, o)| {
            o.zone == Zone::Battlefield
                && o.controller == p
                && o.is_token()
                && g.has_subtype(*id as CardId, subtype)
        })
        .count()
}
#[test]
fn ecl_ashling_command_all_six_pairs_copy_draw_damage_and_opponents_treasure() {
    for p in [Player::P0, Player::P1] {
        for (mode, (a, b)) in PAIRS.into_iter().enumerate() {
            let mut g = setup(p);
            let source = named(&mut g, p, "Chitinous Graspling", Zone::Battlefield);
            let foe = named(&mut g, p.other(), "Hill Giant", Zone::Battlefield);
            let spell = named(&mut g, p, "Ashling's Command", Zone::Hand);
            g.state.players[p.index()].mana = [0, 1, 0, 1, 0, 3];
            let hand = g.hand_count(p.other());
            let life = g.observe(p).players[p.other().index()].life;
            let targets: Vec<_> = [a, b]
                .into_iter()
                .map(|m| {
                    if m == 0 {
                        Target::Card(source)
                    } else {
                        Target::Player(p.other())
                    }
                })
                .collect();
            cast_command(&mut g, spell, Some(mode as u16), &targets);
            assert_eq!(g.state.players[p.index()].mana, [0; 6]);
            settle(&mut g);
            assert_eq!(copies(&g, p), usize::from(a == 0));
            assert_eq!(
                g.hand_count(p.other()),
                hand + if a == 1 || b == 1 { 2 } else { 0 }
            );
            assert_eq!(
                g.state.objects[foe as usize].damage,
                if a == 2 || b == 2 { 2 } else { 0 }
            );
            assert_eq!(
                tokens(&g, p.other(), "Treasure"),
                if b == 3 { 2 } else { 0 }
            );
            assert_eq!(g.state.objects[source as usize].damage, 0);
            assert_eq!(g.observe(p).players[p.other().index()].life, life);
        }
    }
}
#[test]
fn ecl_sygg_command_all_six_pairs_player_group_lifelink_draw_and_stun() {
    for p in [Player::P0, Player::P1] {
        for (mode, (a, b)) in PAIRS.into_iter().enumerate() {
            let mut g = setup(p);
            let source = named(&mut g, p, "Chitinous Graspling", Zone::Battlefield);
            let foe = named(&mut g, p.other(), "Hill Giant", Zone::Battlefield);
            let spell = named(&mut g, p, "Sygg's Command", Zone::Hand);
            g.state.players[p.index()].mana = [1, 1, 0, 0, 0, 1];
            let hand = g.hand_count(p.other());
            let targets: Vec<_> = [a, b]
                .into_iter()
                .map(|m| match m {
                    0 => Target::Card(source),
                    3 => Target::Card(foe),
                    _ => Target::Player(p.other()),
                })
                .collect();
            cast_command(&mut g, spell, Some(mode as u16), &targets);
            settle(&mut g);
            assert_eq!(copies(&g, p), usize::from(a == 0));
            assert_eq!(
                g.hand_count(p.other()),
                hand + usize::from(a == 2 || b == 2)
            );
            assert_eq!(
                g.keywords(foe).contains(Keywords::LIFELINK),
                a == 1 || b == 1
            );
            assert!(!g.keywords(source).contains(Keywords::LIFELINK));
            assert_eq!(g.state.objects[foe as usize].tapped, b == 3);
            assert_eq!(g.counter_count(foe, CounterKind::Stun), u64::from(b == 3));
            // A new creature was not in the resolving group and gets no ability.
            let late = named(&mut g, p.other(), "Bear Cub", Zone::Battlefield);
            assert!(!g.keywords(late).contains(Keywords::LIFELINK));
            g.state.step = Step::Cleanup;
            g.finish_cleanup();
            assert!(!g.keywords(foe).contains(Keywords::LIFELINK));
        }
    }
}
#[test]
fn ecl_brigid_command_all_six_pairs_copy_opponents_token_boost_then_fight() {
    for p in [Player::P0, Player::P1] {
        for (mode, (a, b)) in PAIRS.into_iter().enumerate() {
            let mut g = setup(p);
            let source = named(&mut g, p, "Chitinous Graspling", Zone::Battlefield);
            let foe = named(&mut g, p.other(), "Hill Giant", Zone::Battlefield);
            let spell = named(&mut g, p, "Brigid's Command", Zone::Hand);
            g.state.players[p.index()].mana = [1, 0, 0, 0, 1, 1];
            let mut targets = Vec::new();
            for m in [a, b] {
                match m {
                    0 | 2 => targets.push(Target::Card(source)),
                    1 => targets.push(Target::Player(p.other())),
                    3 => targets.extend([Target::Card(source), Target::Card(foe)]),
                    _ => unreachable!(),
                }
            }
            cast_command(&mut g, spell, Some(mode as u16), &targets);
            settle(&mut g);
            assert_eq!(copies(&g, p), usize::from(a == 0));
            assert_eq!(
                tokens(&g, p.other(), "Kithkin"),
                usize::from(a == 1 || b == 1)
            );
            assert_eq!(g.stats(source).power, if a == 2 || b == 2 { 6 } else { 3 });
            assert_eq!(
                g.state.objects[source as usize].damage,
                if b == 3 { 3 } else { 0 }
            );
            assert_eq!(
                g.object_view(foe).zone,
                if b == 3 {
                    Zone::Graveyard
                } else {
                    Zone::Battlefield
                }
            );
            if a == 0 {
                let copy = g
                    .state
                    .objects
                    .iter()
                    .enumerate()
                    .find(|(_, o)| o.zone == Zone::Battlefield && o.is_token() && o.controller == p)
                    .unwrap()
                    .0 as CardId;
                assert_eq!(g.stats(copy).power, 3);
            }
            g.state.step = Step::Cleanup;
            g.finish_cleanup();
            assert_eq!(g.stats(source).power, 3);
        }
    }
}
#[test]
fn ecl_command_partial_target_failure_and_unavailable_mode_are_transactional() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let source = named(&mut g, p, "Chitinous Graspling", Zone::Battlefield);
        let spell = named(&mut g, p, "Ashling's Command", Zone::Hand);
        let hand = g.hand_count(p.other());
        cast_command(
            &mut g,
            spell,
            Some(0),
            &[Target::Card(source), Target::Player(p.other())],
        );
        g.move_object(source, Zone::Exile);
        g.move_object(source, Zone::Battlefield);
        settle(&mut g);
        assert_eq!(copies(&g, p), 0);
        assert_eq!(g.hand_count(p.other()), hand + 2);
        // No Elemental on the battlefield: modes requiring a copy target vanish.
        g.move_object(source, Zone::Exile);
        g.state.players[p.index()].mana = [30; 6];
        let spell = named(&mut g, p, "Ashling's Command", Zone::Hand);
        checked(&mut g, Action::AnnounceCast { card: spell });
        let before = g.snapshot();
        let revision = g.revision();
        assert_eq!(
            g.apply(p, Action::ChooseMode { mode: 0 }),
            Err(Error::IllegalAction)
        );
        assert_eq!(g.snapshot(), before);
        assert_eq!(g.revision(), revision);
        assert!(g.legal_actions().contains(&Action::ChooseMode { mode: 3 }));
    }
}

#[test]
fn ecl_trystan_command_all_six_pairs_copy_two_returns_destroy_and_group_untap() {
    for p in [Player::P0, Player::P1] {
        for (mode, (a, b)) in PAIRS.into_iter().enumerate() {
            let mut g = setup(p);
            let source = named(&mut g, p, "Chitinous Graspling", Zone::Battlefield);
            let foe = named(&mut g, p.other(), "Hill Giant", Zone::Battlefield);
            let other = named(&mut g, p.other(), "Bear Cub", Zone::Battlefield);
            g.state.objects[foe as usize].tapped = true;
            g.state.objects[other as usize].tapped = true;
            let grave = named(&mut g, p, "Forest", Zone::Graveyard);
            let grave2 = named(&mut g, p, "Springleaf Drum", Zone::Graveyard);
            let spell = named(&mut g, p, "Trystan's Command", Zone::Hand);
            g.state.players[p.index()].mana = [0, 0, 1, 0, 1, 4];
            let mut targets = Vec::new();
            for m in [a, b] {
                match m {
                    0 => targets.push(Target::Card(source)),
                    1 => targets.extend([Target::Card(grave), Target::Card(grave2)]),
                    2 => targets.push(Target::Card(foe)),
                    3 => targets.push(Target::Player(p.other())),
                    _ => unreachable!(),
                }
            }
            cast_command(&mut g, spell, Some(mode as u16), &targets);
            settle(&mut g);
            assert_eq!(copies(&g, p), usize::from(a == 0));
            for id in [grave, grave2] {
                assert_eq!(
                    g.object_view(id).zone,
                    if a == 1 || b == 1 {
                        Zone::Hand
                    } else {
                        Zone::Graveyard
                    }
                );
            }
            assert_eq!(
                g.object_view(foe).zone,
                if a == 2 || b == 2 {
                    Zone::Graveyard
                } else {
                    Zone::Battlefield
                }
            );
            assert_eq!(g.stats(other).power, if b == 3 { 5 } else { 2 });
            assert_eq!(g.state.objects[other as usize].tapped, b != 3);
            assert_eq!(g.stats(source).power, 3);
            g.state.step = Step::Cleanup;
            g.finish_cleanup();
            assert_eq!(g.stats(other).power, 2);
        }
    }
}
