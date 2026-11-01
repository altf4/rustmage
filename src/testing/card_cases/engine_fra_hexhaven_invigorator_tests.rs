//! Focused printed outcomes; shared engine invariants live in core tests.
use super::*;

fn hexhaven_optional(g: &mut Game) {
    for _ in 0..48 {
        if matches!(g.state.pending, Pending::EffectOptional { .. }) {
            return;
        }
        choose(g, |a| matches!(a, Action::Pass | Action::PutTrigger { .. }));
    }
    panic!("damage trigger did not reach optional decision");
}

#[test]
fn hexhaven_searches_up_to_damage_amount_and_puts_selected_lands_tapped() {
    {
        let p = Player::P0;
        let mut g = scenario(p);
        let source = put(&mut g, p, "Hexhaven Invigorator", Zone::Battlefield);
        let dealer = put(&mut g, p.other(), "Bear Cub", Zone::Battlefield);
        let a = put(&mut g, p, "Forest", Zone::Library);
        let b = put(&mut g, p, "Mountain", Zone::Library);
        let c = put(&mut g, p, "Island", Zone::Library);
        let invalid = put(&mut g, p, "Bear Cub", Zone::Library);

        let event = DamageEvent {
            preventable: true,
            source: g.damage_source(dealer),
            target: Target::Card(source),
            amount: 2,
            combat: false,
        };
        g.apply_damage_batch(&[event]);
        assert_eq!(g.state.pending_triggers.len(), 1);
        // The amount is frozen even if the original permanent leaves before resolution.
        g.move_object(source, Zone::Graveyard);
        hexhaven_optional(&mut g);
        act(&mut g, Action::ChooseOptional { accept: true });
        reject(&mut g, Action::ChooseSearch { card: invalid });
        act(&mut g, Action::ChooseSearch { card: a });
        reject(&mut g, Action::ChooseSearch { card: a });

        act(&mut g, Action::ChooseSearch { card: b });
        reject(&mut g, Action::ChooseSearch { card: c });
        act(&mut g, Action::FinishSearch);
        settle(&mut g);
        for id in [a, b] {
            assert_eq!(g.object_view(id).zone, Zone::Battlefield);
            assert!(g.state.objects[id as usize].tapped);
            assert_eq!(g.state.objects[id as usize].controller, p);
        }
        assert_eq!(g.object_view(c).zone, Zone::Library);
        assert_eq!(g.object_view(invalid).zone, Zone::Library);
    }
}

#[test]
fn hexhaven_decline_and_empty_search_are_distinct_choices() {
    {
        let p = Player::P0;
        for accept in [false, true] {
            let mut g = scenario(p);
            let original_library = g.state.players[p.index()].library.len();
            let source = put(&mut g, p, "Hexhaven Invigorator", Zone::Battlefield);
            let dealer = put(&mut g, p.other(), "Bear Cub", Zone::Battlefield);
            let event = DamageEvent {
                preventable: true,
                source: g.damage_source(dealer),
                target: Target::Card(source),
                amount: 1,
                combat: true,
            };
            g.apply_damage_batch(&[event]);
            hexhaven_optional(&mut g);
            act(&mut g, Action::ChooseOptional { accept });
            if accept {
                act(&mut g, Action::FinishSearch);
            }
            settle(&mut g);
            assert_eq!(g.state.players[p.index()].library.len(), original_library);
        }
    }
}
