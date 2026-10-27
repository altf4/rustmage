//! Sandman's blocking threshold in an actual combat.
use super::*;

#[test]
fn sandman_rejects_two_power_blocker_and_allows_three_power() {
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        for _ in 0..3 {
            put(&mut g, p, "Forest", Zone::Battlefield);
        }
        let sandman = put(&mut g, p, "Sandman, Shifting Scoundrel", Zone::Battlefield);
        let small = put(&mut g, p.other(), "Bear Cub", Zone::Battlefield);
        let large = put(&mut g, p.other(), "Hill Giant", Zone::Battlefield);
        g.state.objects[sandman as usize].summoning_sick = false;
        assert_eq!(g.stats(sandman).power, 3);
        g.state.step = Step::Attackers;
        g.state.pending = Pending::Attackers;
        big_helpers::checked(&mut g, Action::Attack { card: sandman });
        big_helpers::checked(&mut g, Action::FinishAttackers);
        for _ in 0..40 {
            if matches!(g.state.pending, Pending::Blockers) {
                break;
            }
            big_helpers::checked(&mut g, Action::Pass);
        }
        assert!(matches!(g.state.pending, Pending::Blockers));
        reject(
            &mut g,
            Action::Block {
                blocker: small,
                attacker: sandman,
            },
        );
        big_helpers::checked(
            &mut g,
            Action::Block {
                blocker: large,
                attacker: sandman,
            },
        );
        big_helpers::checked(&mut g, Action::FinishBlockers);
    }
}

#[test]
fn sandman_and_land_return_with_creature_and_landfall_observers() {
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        put(&mut g, p, "Haliya, Guided by Light", Zone::Battlefield);
        let landfall = put(&mut g, p, "Icetill Explorer", Zone::Battlefield);
        let source = put(&mut g, p, "Sandman, Shifting Scoundrel", Zone::Graveyard);
        let land = put(&mut g, p, "Uncharted Haven", Zone::Graveyard);
        let life = g.state.players[p.index()].life;
        let library_before = g.state.players[p.index()].library.len();
        big_helpers::checked(
            &mut g,
            Action::AnnounceActivate {
                card: source,
                ability: 0,
                counter_payment: Counters::default(),
            },
        );
        big_helpers::checked(
            &mut g,
            Action::ChooseTarget {
                target: Target::Card(land),
            },
        );
        pay(&mut g);
        for _ in 0..40 {
            if matches!(g.state.pending, Pending::EntryValue { .. }) {
                break;
            }
            big_helpers::checked(&mut g, Action::Pass);
        }
        assert!(matches!(g.state.pending, Pending::EntryValue { .. }));
        let batch = g.state.entry_batch.as_ref().unwrap();
        assert_eq!(batch.moves.len(), 2);
        assert!(batch.moves.iter().any(|m| m.before.id == source));
        assert!(batch.moves.iter().any(|m| m.before.id == land));
        assert_eq!(g.object_view(source).zone, Zone::Graveyard);
        assert_eq!(g.object_view(land).zone, Zone::Graveyard);
        let saved = g.snapshot();
        let mut branch = g.clone();
        big_helpers::settle(&mut g);
        assert_eq!(g.object_view(source).zone, Zone::Battlefield);
        assert_eq!(g.object_view(land).zone, Zone::Battlefield);
        assert_eq!(g.state.players[p.index()].life, life + 1);
        assert_eq!(g.state.players[p.index()].library.len(), library_before - 1);
        assert_eq!(g.object_view(landfall).zone, Zone::Battlefield);
        branch.restore(&saved).unwrap();
        big_helpers::settle(&mut branch);
        assert_eq!(branch.snapshot(), g.snapshot());
    }
}
