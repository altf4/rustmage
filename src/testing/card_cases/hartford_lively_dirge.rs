//! Focused printed outcomes; shared engine invariants live in core tests.
//! Spree payment, mandatory private search, and a public simultaneous return.
use super::big_helpers::{checked, settle};
use super::*;

#[test]
fn lively_dirge_spree_searches_then_selects_with_one_total_mana_value_budget() {
    {
        let p = Player::P0;
        for mode in 0..3 {
            let mut g = scenario(p);
            g.state.players[p.index()].mana = [0, 0, 6, 0, 0, 0];
            let searched = put(&mut g, p, "Bear Cub", Zone::Library);
            let a = put(&mut g, p, "Bear Cub", Zone::Graveyard);
            let b = put(&mut g, p, "Bear Cub", Zone::Graveyard);
            let three = put(&mut g, p, "Reclamation Sage", Zone::Graveyard);
            let five = put(&mut g, p, "Serra Angel", Zone::Graveyard);
            let other = put(&mut g, p.other(), "Bear Cub", Zone::Graveyard);
            let land = put(&mut g, p, "Forest", Zone::Graveyard);
            let spell = put(&mut g, p, "Lively Dirge", Zone::Hand);
            checked(&mut g, Action::AnnounceCast { card: spell });
            checked(&mut g, Action::ChooseMode { mode });
            pay(&mut g);
            assert_eq!(
                g.state.players[p.index()].mana[2],
                [3, 2, 1][usize::from(mode)]
            );
            for _ in 0..20 {
                if g.state.pending != Pending::Priority {
                    break;
                }
                checked(&mut g, Action::Pass);
            }
            if mode != 1 {
                assert!(matches!(g.state.pending, Pending::EffectSearch { .. }));
                assert!(g.observe(p.other()).search_choice.is_none());
                reject(&mut g, Action::FinishSearch);
                checked(&mut g, Action::ChooseSearch { card: searched });
                checked(&mut g, Action::FinishSearch);
                assert_eq!(g.object_view(searched).zone, Zone::Graveyard);
            }
            if mode != 0 {
                assert!(matches!(g.state.pending, Pending::EffectChoice { .. }));
                let a_ref = g.object_ref(a);
                let b_ref = g.object_ref(if mode == 2 { searched } else { b });
                for card in [five, other, land] {
                    let object = g.object_ref(card);
                    reject(&mut g, Action::SelectGraveyardReturn { object });
                }
                checked(&mut g, Action::SelectGraveyardReturn { object: a_ref });
                assert_eq!(g.object_view(a).zone, Zone::Graveyard);
                reject(&mut g, Action::SelectGraveyardReturn { object: a_ref });
                let object = g.object_ref(three);
                reject(&mut g, Action::SelectGraveyardReturn { object });
                let public = g.observe(p.other()).effect_choice.unwrap();
                assert!(public.selected[p.index()].contains(&a_ref));
                assert!(public.candidates.contains(&b_ref));
                checked(&mut g, Action::SelectGraveyardReturn { object: b_ref });
                settle(&mut g);
                for id in [a, b_ref.card] {
                    assert_eq!(g.object_view(id).zone, Zone::Battlefield);
                    assert_eq!(g.object_view(id).controller, p);
                }
            } else {
                settle(&mut g);
                assert_eq!(g.object_view(a).zone, Zone::Graveyard);
            }
        }
        // “Up to” permits declining the public selection entirely.
        let mut g = scenario(p);
        let creature = put(&mut g, p, "Bear Cub", Zone::Graveyard);
        let spell = put(&mut g, p, "Lively Dirge", Zone::Hand);
        checked(&mut g, Action::AnnounceCast { card: spell });
        checked(&mut g, Action::ChooseMode { mode: 1 });
        pay(&mut g);
        for _ in 0..20 {
            if matches!(g.state.pending, Pending::EffectChoice { .. }) {
                break;
            }
            checked(&mut g, Action::Pass);
        }
        checked(&mut g, Action::FinishGraveyardReturn);
        settle(&mut g);
        assert_eq!(g.object_view(creature).zone, Zone::Graveyard);
    }
}
