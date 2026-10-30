//! Independent granted Beast loyalty and sacrifice replacement interactions.
use super::*;

fn jace(g: &mut Game, p: Player, loyalty: u64) -> CardId {
    g.create_tokens(p, Card::test_index(5435), 1, false);
    let walker = fra_jaces(g, p)[0];
    g.state.objects[walker as usize].counters.loyalty = loyalty;
    walker
}
fn grant(g: &Game, walker: CardId, provider: CardId) -> Action {
    Action::AnnouncePlaneswalkerGrant {
        source: g.object_ref(walker),
        grantor: g.object_ref(provider),
        ability: 0,
    }
}
fn beast_count(g: &Game, p: Player) -> usize {
    g.state
        .objects
        .iter()
        .filter(|o| {
            o.zone == Zone::Battlefield
                && o.controller == p
                && o.is_token()
                && o.card == Card::test_index(5239)
        })
        .count()
}

#[test]
fn deathbringer_sacrifice_exiled_by_replacement_still_creates_beasts() {
    for p in [Player::P0, Player::P1] {
        for doubled in [false, true] {
            let mut g = scenario(p);
            put(&mut g, p, "Rest in Peace", Zone::Battlefield);
            let way = put(&mut g, p, "Way of the Deathbringer", Zone::Battlefield);
            let walker = jace(&mut g, p, 5);
            if doubled {
                put(&mut g, p, "Doubling Season", Zone::Battlefield);
            }
            let fodder = put(&mut g, p, "Bear Cub", Zone::Battlefield);
            let action = grant(&g, walker, way);
            act(&mut g, action);
            pay(&mut g);
            assert_eq!(g.counter_count(walker, CounterKind::Loyalty), 3);
            for _ in 0..32 {
                if matches!(g.state.pending, Pending::EffectChoice { .. }) {
                    break;
                }
                choose(&mut g, |a| {
                    matches!(a, Action::Pass | Action::PutTrigger { .. })
                });
            }
            let saved = g.snapshot();
            super::super::contracts::branch_contract(g.clone());
            for replay in 0..2 {
                if replay == 1 {
                    g.restore(&saved).unwrap();
                }
                let object = g.object_ref(fodder);
                act(&mut g, Action::Sacrifice { object });
                settle(&mut g);
                assert_eq!(g.object_view(fodder).zone, Zone::Exile);
                assert_eq!(beast_count(&g, p), if doubled { 2 } else { 1 });
            }
        }
    }
}

#[test]
fn wildspeaker_requires_four_loyalty_and_shares_once_per_turn_limit() {
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        let way = put(&mut g, p, "Way of the Wildspeaker", Zone::Battlefield);
        let walker = jace(&mut g, p, 3);
        let action = grant(&g, walker, way);
        reject(&mut g, action.clone());
        assert_eq!(g.counter_count(walker, CounterKind::Loyalty), 3);
        g.state.objects[walker as usize].counters.loyalty = 8;
        act(&mut g, action.clone());
        pay(&mut g);
        assert_eq!(g.counter_count(walker, CounterKind::Loyalty), 4);
        settle(&mut g);
        assert_eq!(beast_count(&g, p), 1);
        reject(&mut g, action);
        reject(
            &mut g,
            Action::AnnounceActivate {
                card: walker,
                ability: 0,
                counter_payment: Counters::default(),
            },
        );
        super::super::contracts::branch_contract(g);
    }
}

#[test]
fn wildspeaker_grant_tracks_provider_control_and_stack_item_keeps_controller() {
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        let way = put(&mut g, p, "Way of the Wildspeaker", Zone::Battlefield);
        let walker = jace(&mut g, p, 8);
        let action = grant(&g, walker, way);
        assert!(g.legal_actions().contains(&action));
        let way_ref = g.object_ref(way);
        g.change_controller(way_ref, p.other());
        reject(&mut g, action.clone());
        g.change_controller(way_ref, p);
        act(&mut g, action);
        pay(&mut g);
        g.change_controller(way_ref, p.other());
        let saved = g.snapshot();
        super::super::contracts::branch_contract(g.clone());
        for replay in 0..2 {
            if replay == 1 {
                g.restore(&saved).unwrap();
            }
            settle(&mut g);
            assert_eq!(beast_count(&g, p), 1);
            assert_eq!(beast_count(&g, p.other()), 0);
        }
    }
}
