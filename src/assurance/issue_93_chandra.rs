//! Chandra's printed abilities exercised through ordinary actions.
use super::big_helpers::{cast as paid_cast, checked, settle as finish};
use super::*;

#[test]
fn chandra_spark_hunter_full_card() {
    let vehicle = Card::from_name("@token/dft-chandra-vehicle").unwrap();
    let emblem = Card::from_name("@emblem/dft-chandra-spark-hunter").unwrap();
    for player in [Player::P0, Player::P1] {
        let mut g = scenario(player);
        let chandra = put(&mut g, player, "Chandra, Spark Hunter", Zone::Hand);
        paid_cast(&mut g, chandra, None, &[]);
        finish(&mut g);
        assert_eq!(g.counter_count(chandra, CounterKind::Loyalty), 4);
        checked(
            &mut g,
            Action::AnnounceActivate {
                card: chandra,
                ability: 1,
                counter_payment: Counters::default(),
            },
        );
        pay(&mut g);
        finish(&mut g);
        let token = g
            .state
            .objects
            .iter()
            .position(|o| o.card == vehicle && o.zone == Zone::Battlefield)
            .unwrap() as CardId;
        assert!(!g.is_creature(token));
        assert_eq!(g.object_view(token).colors, 0);
        let pilot = put(&mut g, player, "Bear Cub", Zone::Battlefield);
        checked(
            &mut g,
            Action::AnnounceActivate {
                card: token,
                ability: 0,
                counter_payment: Counters::default(),
            },
        );
        let object = g.object_ref(pilot);
        checked(&mut g, Action::ChooseCrewer { object });
        checked(&mut g, Action::FinishCrew);
        finish(&mut g);
        assert_eq!(stats(&g, token), (3, 2));
        g.finish_cleanup();
        assert!(!g.is_creature(token));
        g.state.active = player;
        g.begin_step(Step::BeginCombat);
        g.give_priority();
        g.prepare_triggers();
        checked(
            &mut g,
            Action::ChooseTarget {
                target: Target::Card(token),
            },
        );
        if g.legal_actions().contains(&Action::FinishTargets) {
            checked(&mut g, Action::FinishTargets);
        }
        finish(&mut g);
        assert_eq!(stats(&g, token), (3, 2));
        assert!(g.keywords(token).contains(Keywords::HASTE));
        g.finish_cleanup();
        assert!(!g.is_creature(token));

        let mut loot = scenario(player);
        let chandra = put(
            &mut loot,
            player,
            "Chandra, Spark Hunter",
            Zone::Battlefield,
        );
        loot.state.objects[chandra as usize].counters.loyalty = 4;
        let discard = put(&mut loot, player, "Bear Cub", Zone::Hand);
        checked(
            &mut loot,
            Action::AnnounceActivate {
                card: chandra,
                ability: 0,
                counter_payment: Counters::default(),
            },
        );
        pay(&mut loot);
        loot.resolve_top();
        checked(&mut loot, Action::Discard { card: discard });
        finish(&mut loot);
        assert_eq!(loot.object_view(discard).zone, Zone::Graveyard);
        assert_eq!(loot.hand_count(player), 1);
        assert_eq!(loot.counter_count(chandra, CounterKind::Loyalty), 6);

        let mut ultimate = scenario(player);
        let chandra = put(
            &mut ultimate,
            player,
            "Chandra, Spark Hunter",
            Zone::Battlefield,
        );
        ultimate.state.objects[chandra as usize].counters.loyalty = 7;
        checked(
            &mut ultimate,
            Action::AnnounceActivate {
                card: chandra,
                ability: 2,
                counter_payment: Counters::default(),
            },
        );
        pay(&mut ultimate);
        finish(&mut ultimate);
        assert!(
            ultimate
                .state
                .objects
                .iter()
                .any(|o| o.card == emblem && o.zone == Zone::Command)
        );
        let artifact = put(&mut ultimate, player, "Clockwork Percussionist", Zone::Hand);
        paid_cast(&mut ultimate, artifact, None, &[]);
        ultimate.resolve_top();
        ultimate.give_priority();
        ultimate.prepare_triggers();
        checked(
            &mut ultimate,
            Action::ChooseTarget {
                target: Target::Player(player.other()),
            },
        );
        if ultimate.legal_actions().contains(&Action::FinishTargets) {
            checked(&mut ultimate, Action::FinishTargets);
        }
        finish(&mut ultimate);
        assert_eq!(ultimate.state.players[player.other().index()].life, 17);
    }
}
