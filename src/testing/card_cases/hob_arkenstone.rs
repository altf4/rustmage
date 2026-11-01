//! Focused printed outcomes; shared engine invariants live in core tests.
//! Oracle-derived artifact/Adventure, library privacy and end-step outcomes.
use super::*;

fn resolve_to_choice(g: &mut Game) {
    for _ in 0..12 {
        if g.state.pending != Pending::Priority {
            return;
        }
        act(g, Action::Pass);
    }
    panic!("search did not request a choice");
}

#[test]
fn arkenstone_adventure_reveals_only_legendary_creature_then_can_cast_artifact() {
    {
        let p = Player::P0;
        let mut g = scenario(p);
        let wanted = put(&mut g, p, "Óin the Brave", Zone::Library);
        let ordinary = put(&mut g, p, "Grizzly Bears", Zone::Library);
        let legendary_artifact = put(&mut g, p, "My Precious", Zone::Library);
        let stone = put(&mut g, p, "The Arkenstone // Seek the Heart", Zone::Hand);
        g.state.players[p.index()].mana = [1, 0, 0, 0, 0, 2];
        act(&mut g, Action::AnnounceAdventure { card: stone });
        pay(&mut g);
        assert_eq!(g.state.players[p.index()].mana, [0; 6]);
        resolve_to_choice(&mut g);
        assert!(g.observe(p.other()).search_choice.is_none());
        for hidden in [wanted, ordinary, legendary_artifact] {
            assert!(g.observe(p.other()).objects.iter().all(|o| o.id != hidden));
        }
        reject(&mut g, Action::ChooseSearch { card: ordinary });
        reject(
            &mut g,
            Action::ChooseSearch {
                card: legendary_artifact,
            },
        );

        act(&mut g, Action::ChooseSearch { card: wanted });
        act(&mut g, Action::FinishSearch);
        settle(&mut g);
        assert_eq!(g.object_view(wanted).zone, Zone::Hand);
        assert!(
            g.observe(p.other())
                .known_hand
                .contains(&g.object_ref(wanted))
        );
        assert_eq!(g.object_view(stone).zone, Zone::Exile);
        assert!(g.can_play_adventure(p, stone));
        g.state.players[p.index()].mana = [0, 0, 0, 0, 0, 5];
        act(&mut g, Action::AnnounceExile { card: stone });
        pay(&mut g);
        settle(&mut g);
        assert_eq!(g.object_view(stone).zone, Zone::Battlefield);
        assert_eq!(g.state.players[p.index()].mana, [0; 6]);
    }
}

#[test]
fn arkenstone_anthem_includes_new_creatures_and_draws_only_on_own_end_step() {
    {
        let p = Player::P0;
        let mut g = scenario(p);
        let bearer = put(&mut g, p, "Grizzly Bears", Zone::Battlefield);
        let enemy = put(&mut g, p.other(), "Grizzly Bears", Zone::Battlefield);
        let stone = put(&mut g, p, "The Arkenstone", Zone::Hand);
        g.state.players[p.index()].mana = [0, 0, 0, 0, 0, 5];
        cast(&mut g, stone, None);
        settle(&mut g);
        assert_eq!(stats(&g, bearer), (3, 3));
        assert_eq!(stats(&g, enemy), (2, 2));
        let later = put(&mut g, p, "Grizzly Bears", Zone::Battlefield);
        assert_eq!(stats(&g, later), (3, 3));
        let before = g.hand_count(p);
        g.state.active = p.other();
        g.begin_step(Step::End);
        g.give_priority();
        g.prepare_triggers();
        settle(&mut g);
        assert_eq!(g.hand_count(p), before);
        g.state.active = p;
        g.begin_step(Step::End);
        g.give_priority();
        g.prepare_triggers();
        assert!(!g.state.stack.is_empty());
        // Once triggered, draw survives source departure and anthem stops immediately.
        g.move_object(stone, Zone::Graveyard);
        g.give_priority();
        assert_eq!(stats(&g, bearer), (2, 2));
        settle(&mut g);
        assert_eq!(g.hand_count(p), before + 1);
    }
}
