//! Focused printed outcomes; shared engine invariants live in core tests.
use super::*;

fn tokens(g: &Game, card: Card, owner: Player) -> usize {
    g.state
        .objects
        .iter()
        .filter(|o| {
            o.is_token() && o.zone == Zone::Battlefield && o.controller == owner && o.card == card
        })
        .count()
}

#[test]
fn kwia_life_trigger_resets_next_turn_and_obeys_token_replacement() {
    {
        let p = Player::P0;
        let mut g = scenario(p);
        put(&mut g, p, "Doubling Season", Zone::Battlefield);
        put(&mut g, p, "Kwia Vigorbloom", Zone::Battlefield);
        g.gain_life(p, 1);
        g.prepare_triggers();
        settle(&mut g);
        assert_eq!(
            tokens(&g, Card::from_name("@token/fra-lotus").unwrap(), p),
            2
        );
        g.finish_cleanup();
        g.gain_life(p, 1);
        g.prepare_triggers();
        settle(&mut g);
        assert_eq!(
            tokens(&g, Card::from_name("@token/fra-lotus").unwrap(), p),
            4
        );
    }
}

#[test]
fn kwia_ward_counters_or_allows_opponents_targeting_spell() {
    {
        let p = Player::P0;
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
