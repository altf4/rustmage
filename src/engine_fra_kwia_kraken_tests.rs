use super::*;

fn tokens(g: &Game, card: u16, owner: Player) -> usize {
    g.state
        .objects
        .iter()
        .filter(|o| {
            o.is_token()
                && o.zone == Zone::Battlefield
                && o.controller == owner
                && o.card == Card::test_index(card)
        })
        .count()
}

#[test]
fn kwia_uses_one_life_gain_trigger_per_turn_and_queued_trigger_survives() {
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        let kwia = put(&mut g, p, "Kwia Vigorbloom", Zone::Battlefield);
        assert_eq!(stats(&g, kwia), (6, 6));
        for keyword in [Keywords::FLYING, Keywords::VIGILANCE, Keywords::LIFELINK] {
            assert!(g.keywords(kwia).contains(keyword));
        }
        g.gain_life(p.other(), 1);
        g.gain_life(p, 0);
        assert_eq!(tokens(&g, 5453, p), 0);
        g.gain_life(p, 1);
        g.gain_life(p, 3);
        g.prepare_triggers();
        let saved = g.snapshot();
        super::super::contracts::branch_contract(g.clone());
        for replay in 0..2 {
            if replay == 1 {
                g.restore(&saved).unwrap();
            }
            g.move_object(kwia, Zone::Graveyard);
            settle(&mut g);
            assert_eq!(tokens(&g, 5453, p), 1);
        }
    }
}

#[test]
fn kwia_life_trigger_resets_next_turn_and_obeys_token_replacement() {
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        put(&mut g, p, "Doubling Season", Zone::Battlefield);
        put(&mut g, p, "Kwia Vigorbloom", Zone::Battlefield);
        g.gain_life(p, 1);
        g.prepare_triggers();
        settle(&mut g);
        assert_eq!(tokens(&g, 5453, p), 2);
        g.finish_cleanup();
        g.gain_life(p, 1);
        g.prepare_triggers();
        settle(&mut g);
        assert_eq!(tokens(&g, 5453, p), 4);
    }
}

#[test]
fn kwia_ward_counters_or_allows_opponents_targeting_spell() {
    for p in [Player::P0, Player::P1] {
        for paid in [false, true] {
            let mut g = scenario(p.other());
            let kwia = put(&mut g, p, "Kwia Vigorbloom", Zone::Battlefield);
            let spell = put(&mut g, p.other(), "Shock", Zone::Hand);
            cast(&mut g, spell, Some(Target::Card(kwia)));
            for _ in 0..24 {
                if g.state.pending != Pending::Priority {
                    break;
                }
                act(&mut g, Action::Pass);
            }
            if paid {
                choose(&mut g, |a| matches!(a, Action::PayResolutionMana { .. }));
            } else {
                act(&mut g, Action::DeclineResolutionMana);
            }
            settle(&mut g);
            assert_eq!(g.object_view(kwia).damage, if paid { 2 } else { 0 });
        }
    }
}

#[test]
fn kraken_triggers_each_players_upkeep_and_token_belongs_to_trigger_controller() {
    for p in [Player::P0, Player::P1] {
        for active in [p, p.other()] {
            let mut g = scenario(active);
            let kraken = put(&mut g, p, "Verdant Kraken", Zone::Battlefield);
            assert_eq!(stats(&g, kraken), (6, 6));
            g.begin_step(Step::Upkeep);
            g.prepare_triggers();
            let saved = g.snapshot();
            super::super::contracts::branch_contract(g.clone());
            for replay in 0..2 {
                if replay == 1 {
                    g.restore(&saved).unwrap();
                }
                let reference = g.object_ref(kraken);
                g.change_controller(reference, p.other());
                settle(&mut g);
                assert_eq!(tokens(&g, 5454, p), 1);
                assert_eq!(tokens(&g, 5454, p.other()), 0);
            }
        }
    }
}
