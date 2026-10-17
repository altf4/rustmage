//! Remaining paid Gate Colossus obligations; graveyard/Phoenix cases live in
//! engine_graveyard_origin_tests.rs rather than being duplicated here.
use super::ability_timeline_tests::{cast, checked, named, next_turn, settle, setup};
use super::tests::spawn;
use super::*;

fn reject(g: &mut Game, action: Action) {
    let before = g.snapshot();
    let revision = g.revision();
    assert!(!g.legal_actions().contains(&action));
    assert_eq!(
        g.apply(g.actor().unwrap(), action),
        Err(Error::IllegalAction)
    );
    assert_eq!(g.snapshot(), before);
    assert_eq!(g.revision(), revision);
}

#[test]
fn fdn_eight_gate_affinity_paid_controlled_battlefield_gates_only_and_zero_floor() {
    for p in [Player::P0, Player::P1] {
        for gates in [0u32, 1, 7, 8, 10] {
            let mut g = setup(p);
            for _ in 0..gates {
                named(&mut g, p, "Azorius Guildgate", Zone::Battlefield);
            }
            // Neither opposing Gates, Gates in other zones, ordinary lands nor
            // non-Gate artifacts contribute to this affinity count.
            for _ in 0..4 {
                named(&mut g, p.other(), "Azorius Guildgate", Zone::Battlefield);
            }
            named(&mut g, p, "Azorius Guildgate", Zone::Hand);
            named(&mut g, p, "Azorius Guildgate", Zone::Graveyard);
            named(&mut g, p, "Forest", Zone::Battlefield);
            named(&mut g, p, "Juggernaut", Zone::Battlefield);
            let id = spawn(
                &mut g,
                p,
                Card::from_name("Gate Colossus").unwrap(),
                Zone::Hand,
            );
            let cost = 8u32.saturating_sub(gates);
            // Supply only colorless mana, avoiding tap-mana alternatives from
            // the Gates themselves by tapping the fixture permanents.
            for object in &mut g.state.objects {
                if object.zone == Zone::Battlefield {
                    object.tapped = true;
                }
            }
            g.state.players[p.index()].mana = [0; 6];
            if cost > 0 {
                g.state.players[p.index()].mana[5] = cost - 1;
                reject(&mut g, Action::AnnounceCast { card: id });
            }
            g.state.players[p.index()].mana[5] = cost;
            checked(&mut g, Action::AnnounceCast { card: id });
            if g.legal_actions().contains(&Action::FinishTargets) {
                checked(&mut g, Action::FinishTargets);
            }
            assert!(g.observe(p.other()).actions.is_empty());
            let payment = [0, 0, 0, 0, 0, cost];
            reject(
                &mut g,
                Action::PayCosts {
                    payment: [0, 0, 0, 0, 0, cost + 1],
                },
            );
            checked(&mut g, Action::PayCosts { payment });
            assert_eq!(g.state.players[p.index()].mana, [0; 6]);
            settle(&mut g);
            assert_eq!(g.object_view(id).zone, Zone::Battlefield);
            assert_eq!(
                g.stats(id),
                Stats {
                    power: 8,
                    toughness: 8
                }
            );
            assert!(g.keywords(id).contains(Keywords::NO_POWER_LE_TWO_BLOCKERS));
            // Affinity belongs only to Colossus: even ten Gates do not make a
            // different artifact creature free to cast.
            let other = named(&mut g, p, "Juggernaut", Zone::Hand);
            reject(&mut g, Action::AnnounceCast { card: other });
        }
    }
}

fn until(g: &mut Game, stop: impl Fn(&Game) -> bool) {
    for _ in 0..80 {
        if stop(g) {
            return;
        }
        checked(g, Action::Pass);
    }
    panic!("did not reach combat decision: {:?}", g.state.pending);
}

#[test]
fn fdn_eight_gate_power_two_illegal_three_legal_and_paid_live_growth_both_seats() {
    for p in [Player::P0, Player::P1] {
        for (name, pump) in [
            ("Bear Cub", false),
            ("Hill Giant", false),
            ("Bear Cub", true),
        ] {
            let mut g = setup(p);
            let gate = spawn(
                &mut g,
                p,
                Card::from_name("Gate Colossus").unwrap(),
                Zone::Hand,
            );
            cast(&mut g, gate, None, &[]);
            settle(&mut g);
            let blocker = named(&mut g, p.other(), name, Zone::Battlefield);
            next_turn(&mut g);
            next_turn(&mut g);
            assert_eq!(g.state.active, p);
            assert!(!g.object_view(gate).summoning_sick);
            assert_eq!(g.can_block_pair(blocker, gate), name == "Hill Giant");
            until(&mut g, |g| g.state.pending == Pending::Attackers);
            checked(&mut g, Action::Attack { card: gate });
            checked(&mut g, Action::FinishAttackers);
            if pump {
                // A real paid instant changes the restriction's answer before
                // blockers, using current power instead of printed/LKI power.
                until(&mut g, |g| g.actor() == Some(p.other()));
                g.state.players[p.other().index()].mana = [0, 0, 0, 0, 1, 0];
                let growth = named(&mut g, p.other(), "Giant Growth", Zone::Hand);
                cast(&mut g, growth, None, &[Target::Card(blocker)]);
                settle(&mut g);
                assert_eq!(g.stats(blocker).power, 5);
            }
            until(&mut g, |g| g.state.pending == Pending::Blockers);
            let action = Action::Block {
                blocker,
                attacker: gate,
            };
            if name == "Bear Cub" && !pump {
                reject(&mut g, action);
            } else {
                checked(&mut g, action);
            }
            checked(&mut g, Action::FinishBlockers);
            assert_eq!(
                g.state.blocks.contains(&(blocker, gate)),
                name != "Bear Cub" || pump
            );
        }
    }
}
