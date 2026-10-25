//! Whole Mar-Vell: real opponent casts, live as-though permission and timing limits.
use super::assurance::big_helpers::{cast, checked, named, next_turn, settle, setup};
use super::*;
fn paid(g: &mut Game, p: Player, name: &str, targets: &[Target]) -> CardId {
    let id = named(g, p, name, Zone::Hand);
    cast(g, id, None, targets);
    settle(g);
    id
}
fn marvell(g: &mut Game, p: Player) -> CardId {
    g.state.players[p.index()].mana = [1, 0, 0, 0, 0, 4];
    let id = paid(g, p, "Captain Mar-Vell, Space-Born", &[]);
    assert_eq!(g.state.players[p.index()].mana, [0; 6]);
    g.state.players[p.index()].mana = [30; 6];
    id
}
fn reject(g: &mut Game, a: Action) {
    let s = g.snapshot();
    let r = g.revision();
    assert_eq!(g.apply(g.actor().unwrap(), a), Err(Error::IllegalAction));
    assert_eq!(g.snapshot(), s);
    assert_eq!(g.revision(), r);
}
fn opposing_cast(g: &mut Game, p: Player) {
    checked(g, Action::Pass);
    let id = named(g, p.other(), "Shock", Zone::Hand);
    cast(g, id, None, &[Target::Player(p)]);
    assert_eq!(g.state.players[p.other().index()].spells_cast_this_turn, 1);
    checked(g, Action::Pass);
    assert_eq!(g.actor(), Some(p));
}
#[test]
fn msh_marvell_paid_body_flying_vigilance_and_actual_legal_blocking() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let c = marvell(&mut g, p);
        assert_eq!((g.stats(c).power, g.stats(c).toughness), (4, 4));
        assert!(
            g.keywords(c)
                .contains(Keywords::FLYING.union(Keywords::VIGILANCE))
        );
        let b = named(&mut g, p.other(), "Bear Cub", Zone::Battlefield);
        g.state.objects[c as usize].summoning_sick = false;
        g.state.step = Step::Attackers;
        g.state.pending = Pending::Attackers;
        checked(&mut g, Action::Attack { card: c });
        checked(&mut g, Action::FinishAttackers);
        assert!(!g.object_view(c).tapped);
        while g.state.pending != Pending::Blockers {
            checked(&mut g, Action::Pass);
        }
        reject(
            &mut g,
            Action::Block {
                attacker: c,
                blocker: b,
            },
        );
        checked(&mut g, Action::FinishBlockers);
    }
}
#[test]
fn msh_marvell_own_spells_and_opponent_mana_abilities_do_not_qualify() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        marvell(&mut g, p);
        paid(&mut g, p, "Shock", &[Target::Player(p.other())]);
        let bear = named(&mut g, p, "Bear Cub", Zone::Hand);
        g.state.step = Step::End;
        reject(&mut g, Action::AnnounceCast { card: bear });
        let land = named(&mut g, p.other(), "Forest", Zone::Battlefield);
        checked(&mut g, Action::Pass);
        checked(&mut g, Action::TapMana { card: land });
        checked(&mut g, Action::Pass);
        assert_eq!(g.actor(), Some(p));
        reject(&mut g, Action::AnnounceCast { card: bear });
    }
}
#[test]
fn msh_marvell_real_opponent_cast_unlocks_creature_sorcery_artifact_and_aura_response() {
    for p in [Player::P0, Player::P1] {
        for name in [
            "Bear Cub",
            "Divination",
            "Vibranium Energy Daggers",
            "Witness Protection",
        ] {
            let mut g = setup(p);
            let c = marvell(&mut g, p);
            let own = named(&mut g, p, name, Zone::Hand);
            opposing_cast(&mut g, p);
            assert!(!g.keywords(c).contains(Keywords::FLASH));
            let targets = if name == "Witness Protection" {
                vec![Target::Card(c)]
            } else {
                vec![]
            };
            cast(&mut g, own, None, &targets);
            settle(&mut g);
            assert_ne!(g.object_view(own).zone, Zone::Hand);
            assert_eq!(g.state.players[p.index()].life, 18);
        }
    }
}
#[test]
fn msh_marvell_history_before_entry_and_turn_reset() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        opposing_cast(&mut g, p);
        settle(&mut g);
        marvell(&mut g, p);
        g.state.step = Step::End;
        let bear = named(&mut g, p, "Bear Cub", Zone::Hand);
        cast(&mut g, bear, None, &[]);
        settle(&mut g);
        assert_eq!(g.object_view(bear).zone, Zone::Battlefield);
        next_turn(&mut g);
        assert_eq!(g.state.players[p.other().index()].spells_cast_this_turn, 0);
        let other = named(&mut g, p, "Bear Cub", Zone::Hand);
        g.state.priority = p;
        reject(&mut g, Action::AnnounceCast { card: other });
    }
}
#[test]
fn msh_marvell_suppression_departure_and_costs_still_apply() {
    for p in [Player::P0, Player::P1] {
        for suppress in [false, true] {
            let mut g = setup(p);
            let c = marvell(&mut g, p);
            opposing_cast(&mut g, p);
            settle(&mut g);
            if suppress {
                paid(&mut g, p, "Witness Protection", &[Target::Card(c)]);
            } else {
                paid(&mut g, p, "Murder", &[Target::Card(c)]);
            }
            g.state.step = Step::End;
            let bear = named(&mut g, p, "Bear Cub", Zone::Hand);
            reject(&mut g, Action::AnnounceCast { card: bear });
        }
        let mut g = setup(p);
        marvell(&mut g, p);
        opposing_cast(&mut g, p);
        let land = named(&mut g, p, "Forest", Zone::Hand);
        reject(&mut g, Action::AnnounceCast { card: land });
        let bear = named(&mut g, p, "Bear Cub", Zone::Hand);
        g.state.players[p.index()].mana = [0; 6];
        reject(&mut g, Action::AnnounceCast { card: bear });
    }
}
#[test]
fn msh_marvell_current_controller_opponent_and_private_observations() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let c = named(&mut g, p, "Captain Mar-Vell, Space-Born", Zone::Battlefield);
        opposing_cast(&mut g, p);
        let bear = Card::from_name("Bear Cub").unwrap();
        assert!(g.cast_timing_granted(p, bear));
        g.record_control(g.object_ref(c), p.other(), None);
        assert!(!g.cast_timing_granted(p, bear));
        assert!(!g.cast_timing_granted(p.other(), bear));
        let hidden = named(&mut g, p, "Shock", Zone::Hand);
        let mut branch = g.clone();
        branch.state.objects[hidden as usize].card = Card::from_name("Forest").unwrap();
        assert_eq!(
            serde_json::to_value(g.observe(p.other())).unwrap(),
            serde_json::to_value(branch.observe(p.other())).unwrap()
        );
    }
}
