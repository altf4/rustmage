//! Provisional whole-card scenario while Oko and crime history remain gated.
use super::big_helpers::{cast as paid_cast, checked, settle as finish};
use super::*;

fn gated_oko(g: &mut Game, player: Player) -> CardId {
    let id = put(g, player, "Bear Cub", Zone::Battlefield);
    g.state.objects[id as usize].card = Card::from_name("Oko, the Ringleader").unwrap();
    g.state.objects[id as usize].counters.loyalty = 3;
    g.register_battlefield_sources(id);
    id
}

#[test]
fn oko_ringleader_full_card_provisional() {
    let elk = Card::from_name("@token/otj-oko-elk").unwrap();
    for player in [Player::P0, Player::P1] {
        for crime in [false, true] {
            let mut g = scenario(player);
            let oko = gated_oko(&mut g, player);
            let shock = put(&mut g, player, "Shock", Zone::Hand);
            let target = if crime { player.other() } else { player };
            paid_cast(&mut g, shock, None, &[Target::Player(target)]);
            finish(&mut g);
            assert_eq!(
                g.observe(player).players[player.index()].committed_crime_this_turn,
                crime
            );
            checked(
                &mut g,
                Action::AnnounceActivate {
                    card: oko,
                    ability: 0,
                    counter_payment: Counters::default(),
                },
            );
            pay(&mut g);
            finish(&mut g);
            assert_eq!(g.hand_count(player), usize::from(crime));
        }
        let mut tokens = scenario(player);
        let oko = gated_oko(&mut tokens, player);
        checked(
            &mut tokens,
            Action::AnnounceActivate {
                card: oko,
                ability: 1,
                counter_payment: Counters::default(),
            },
        );
        pay(&mut tokens);
        finish(&mut tokens);
        let elk_id = tokens
            .state
            .objects
            .iter()
            .position(|o| o.card == elk && o.zone == Zone::Battlefield)
            .unwrap() as CardId;
        assert_eq!(stats(&tokens, elk_id), (3, 3));
        assert_eq!(tokens.object_view(elk_id).colors, 16);
        assert!(tokens.has_subtype(elk_id, "Elk"));

        let mut combat = scenario(player);
        let oko = gated_oko(&mut combat, player);
        let bear = put(&mut combat, player, "Bear Cub", Zone::Battlefield);
        combat.begin_step(Step::BeginCombat);
        combat.give_priority();
        combat.prepare_triggers();
        checked(
            &mut combat,
            Action::ChooseTarget {
                target: Target::Card(bear),
            },
        );
        if combat.legal_actions().contains(&Action::FinishTargets) {
            checked(&mut combat, Action::FinishTargets);
        }
        finish(&mut combat);
        assert_eq!(stats(&combat, oko), (2, 2));
        assert!(combat.keywords(oko).contains(Keywords::HEXPROOF));
        combat.finish_cleanup();
        assert_eq!(combat.object_view(oko).card.name(), "Oko, the Ringleader");

        let mut copies = scenario(player);
        let oko = gated_oko(&mut copies, player);
        copies.state.objects[oko as usize].counters.loyalty = 5;
        let bear = put(&mut copies, player, "Bear Cub", Zone::Battlefield);
        let land = put(&mut copies, player, "Forest", Zone::Battlefield);
        checked(
            &mut copies,
            Action::AnnounceActivate {
                card: oko,
                ability: 2,
                counter_payment: Counters::default(),
            },
        );
        pay(&mut copies);
        finish(&mut copies);
        assert_eq!(
            copies
                .state
                .objects
                .iter()
                .filter(|o| o.zone == Zone::Battlefield
                    && o.card == copies.state.objects[bear as usize].card)
                .count(),
            2
        );
        assert_eq!(
            copies
                .state
                .objects
                .iter()
                .filter(|o| o.zone == Zone::Battlefield
                    && o.card == copies.state.objects[land as usize].card)
                .count(),
            1
        );
    }
}
