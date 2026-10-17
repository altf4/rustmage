//! Paid whole-card combat acceptance; shared source filters and thresholds have core tests.
use super::ability_timeline_tests::{cast, checked, named, settle, setup};
use super::tests::spawn;
use super::*;

fn paid(g: &mut Game, p: Player, name: &str, cost: u32) -> CardId {
    let id = spawn(g, p, Card::from_name(name).unwrap(), Zone::Hand);
    let before: u32 = g.state.players[p.index()].mana.iter().sum();
    cast(g, id, None, &[]);
    settle(g);
    assert_eq!(
        before - g.state.players[p.index()].mana.iter().sum::<u32>(),
        cost
    );
    assert_eq!(g.object_view(id).zone, Zone::Battlefield);
    id
}
fn attack(g: &mut Game, id: CardId) {
    // Begin the scenario's combat after the creature's summoning turn.
    g.state.step = Step::Attackers;
    g.state.pending = Pending::Attackers;
    g.state.objects[id as usize].summoning_sick = false;
    g.state.objects[id as usize].tapped = false;
    checked(g, Action::Attack { card: id });
    checked(g, Action::FinishAttackers);
    checked(g, Action::Pass);
    checked(g, Action::Pass);
    assert_eq!(g.state.pending, Pending::Blockers);
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
#[test]
fn fdn_eight_qualified_paid_knights_kill_in_first_strike_with_live_color_bonus() {
    for p in [Player::P0, Player::P1] {
        for (name, color_source) in [
            ("Knight of Grace", "Diregraf Ghoul"),
            ("Knight of Malice", "Healer's Hawk"),
        ] {
            let mut g = setup(p);
            let knight = paid(&mut g, p, name, 2);
            assert_eq!((g.stats(knight).power, g.stats(knight).toughness), (2, 2));
            named(&mut g, p.other(), color_source, Zone::Battlefield);
            assert_eq!((g.stats(knight).power, g.stats(knight).toughness), (3, 2));
            let blocker = named(&mut g, p.other(), "Bear Cub", Zone::Battlefield);
            attack(&mut g, knight);
            checked(
                &mut g,
                Action::Block {
                    blocker,
                    attacker: knight,
                },
            );
            checked(&mut g, Action::FinishBlockers);
            checked(&mut g, Action::Pass);
            checked(&mut g, Action::Pass);
            assert_eq!(g.state.step, Step::FirstStrikeDamage);
            assert_eq!(g.object_view(blocker).zone, Zone::Graveyard);
            assert_eq!(g.object_view(knight).zone, Zone::Battlefield);
            assert_eq!(g.object_view(knight).damage, 0);
            checked(&mut g, Action::Pass);
            checked(&mut g, Action::Pass);
            assert_eq!(g.state.step, Step::CombatDamage);
            assert_eq!(g.object_view(knight).damage, 0);
            assert_eq!(g.state.players[p.other().index()].life, STARTING_LIFE);
        }
    }
}
#[test]
fn fdn_eight_qualified_paid_elenda_lifelink_thresholds_and_live_menace_blocking() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let elenda = paid(&mut g, p, "Elenda, Saint of Dusk", 4);
        assert_eq!((g.stats(elenda).power, g.stats(elenda).toughness), (4, 4));
        assert!(!g.keywords(elenda).contains(Keywords::MENACE));
        attack(&mut g, elenda);
        checked(&mut g, Action::FinishBlockers);
        checked(&mut g, Action::Pass);
        checked(&mut g, Action::Pass);
        assert_eq!(g.state.step, Step::CombatDamage);
        assert_eq!(g.state.players[p.index()].life, STARTING_LIFE + 4);
        assert_eq!((g.stats(elenda).power, g.stats(elenda).toughness), (5, 5));
        assert!(
            g.keywords(elenda)
                .contains(Keywords::LIFELINK.union(Keywords::MENACE))
        );
        // Finish combat naturally; the next life gain is a paid sorcery in main two.
        for _ in 0..4 {
            checked(&mut g, Action::Pass);
        }
        assert_eq!(g.state.step, Step::Main2);
        // Phase changes emptied the original pool; fund this spell with actual lands.
        for _ in 0..8 {
            let swamp = named(&mut g, p, "Swamp", Zone::Battlefield);
            checked(&mut g, Action::TapMana { card: swamp });
        }
        let drain = named(&mut g, p, "Exsanguinate", Zone::Hand);
        checked(&mut g, Action::AnnounceCast { card: drain });
        checked(&mut g, Action::ChooseX { value: 6 });
        if g.legal_actions().contains(&Action::FinishTargets) {
            checked(&mut g, Action::FinishTargets);
        }
        let payment = g
            .legal_actions()
            .into_iter()
            .find(|a| matches!(a, Action::PayCosts { .. }))
            .unwrap();
        checked(&mut g, payment);
        settle(&mut g);
        assert_eq!(g.state.players[p.index()].life, STARTING_LIFE + 10);
        assert_eq!((g.stats(elenda).power, g.stats(elenda).toughness), (10, 10));
        // A real opposing ability drops below the larger bonus without removing menace.
        let fire = named(&mut g, p.other(), "Fanatical Firebrand", Zone::Battlefield);
        checked(&mut g, Action::Pass);
        let ping=g.legal_actions().into_iter().find(|a|matches!(a,Action::Activate {card,target:Some(Target::Player(target)),..} if *card==fire && *target==p)).unwrap();
        checked(&mut g, ping);
        settle(&mut g);
        assert_eq!(g.state.players[p.index()].life, STARTING_LIFE + 9);
        assert_eq!((g.stats(elenda).power, g.stats(elenda).toughness), (5, 5));
        let first = named(&mut g, p.other(), "Bear Cub", Zone::Battlefield);
        attack(&mut g, elenda);
        reject(
            &mut g,
            Action::Block {
                blocker: first,
                attacker: elenda,
            },
        );
        let second = named(&mut g, p.other(), "Bear Cub", Zone::Battlefield);
        checked(
            &mut g,
            Action::Block {
                blocker: first,
                attacker: elenda,
            },
        );
        reject(&mut g, Action::FinishBlockers);
        checked(
            &mut g,
            Action::Block {
                blocker: second,
                attacker: elenda,
            },
        );
        checked(&mut g, Action::FinishBlockers);
    }
}
