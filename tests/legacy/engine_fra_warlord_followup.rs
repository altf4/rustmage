//! Independent Avatar and Warlord grant interactions.
use super::*;

#[test]
fn avatar_grant_counts_live_lands_after_payment_and_obeys_counter_replacement() {
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        let avatar = put(&mut g, p, "Avatar of Burgeoning Echoes", Zone::Battlefield);
        let walker = put(&mut g, p, "Ajani, Caller of the Pride", Zone::Battlefield);
        g.state.objects[walker as usize].counters.loyalty = 10;
        // Doubling Season applies to counters placed on your own permanents.
        let target = put(&mut g, p, "Bear Cub", Zone::Battlefield);
        let old_land = put(&mut g, p, "Forest", Zone::Battlefield);
        put(&mut g, p.other(), "Forest", Zone::Battlefield);
        put(&mut g, p, "Doubling Season", Zone::Battlefield);
        let source = g.object_ref(walker);
        let grantor = g.object_ref(avatar);
        act(
            &mut g,
            Action::AnnouncePlaneswalkerGrant {
                source,
                grantor,
                ability: 0,
            },
        );
        act(
            &mut g,
            Action::ChooseTarget {
                target: Target::Card(target),
            },
        );
        pay(&mut g);
        g.move_object(old_land, Zone::Graveyard);
        put(&mut g, p, "Island", Zone::Battlefield);
        put(&mut g, p, "Island", Zone::Battlefield);
        g.move_object(avatar, Zone::Graveyard);
        let saved = g.snapshot();
        super::super::contracts::branch_contract(g.clone());
        for replay in 0..2 {
            if replay == 1 {
                g.restore(&saved).unwrap();
            }
            settle(&mut g);
            assert_eq!(g.counter_count(target, CounterKind::PlusOne), 4);
        }
    }
}

#[test]
fn warlord_dead_planeswalker_still_deals_to_valid_creature_if_player_becomes_illegal() {
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        let way = put(&mut g, p, "Way of the Warlord", Zone::Battlefield);
        let walker = put(&mut g, p, "Ajani, Caller of the Pride", Zone::Battlefield);
        g.state.objects[walker as usize].counters.loyalty = 4;
        let target = put(&mut g, p.other(), "Bear Cub", Zone::Battlefield);
        let source = g.object_ref(walker);
        let grantor = g.object_ref(way);
        act(
            &mut g,
            Action::AnnouncePlaneswalkerGrant {
                source,
                grantor,
                ability: 0,
            },
        );
        act(
            &mut g,
            Action::ChooseTarget {
                target: Target::Card(target),
            },
        );
        act(
            &mut g,
            Action::ChooseTarget {
                target: Target::Player(p.other()),
            },
        );
        pay(&mut g);
        g.state.players[p.other().index()].hexproof_until_end_turn = true;
        let saved = g.snapshot();
        super::super::contracts::branch_contract(g.clone());
        for replay in 0..2 {
            if replay == 1 {
                g.restore(&saved).unwrap();
            }
            settle(&mut g);
            assert_eq!(g.object_view(target).zone, Zone::Graveyard);
            assert_eq!(g.state.players[p.other().index()].life, 20);
        }
    }
}
