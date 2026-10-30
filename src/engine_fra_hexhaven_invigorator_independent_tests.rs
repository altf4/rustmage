use super::*;

#[test]
fn invigorator_sums_simultaneous_damage_and_keeps_search_private() {
    for player in [Player::P0, Player::P1] {
        let mut g = scenario(player);
        let source = put(&mut g, player, "Hexhaven Invigorator", Zone::Battlefield);
        let a = put(&mut g, player.other(), "Bear Cub", Zone::Battlefield);
        let b = put(&mut g, player.other(), "Hill Giant", Zone::Battlefield);
        let lands = [
            put(&mut g, player, "Forest", Zone::Library),
            put(&mut g, player, "Mountain", Zone::Library),
            put(&mut g, player, "Island", Zone::Library),
        ];
        let events = [
            DamageEvent {
                preventable: true,
                source: g.damage_source(a),
                target: Target::Card(source),
                amount: 1,
                combat: true,
            },
            DamageEvent {
                preventable: true,
                source: g.damage_source(b),
                target: Target::Card(source),
                amount: 2,
                combat: true,
            },
        ];
        g.apply_damage_batch(&events);
        assert_eq!(g.state.pending_triggers.len(), 1);
        for _ in 0..48 {
            if matches!(g.state.pending, Pending::EffectOptional { .. }) {
                break;
            }
            choose(&mut g, |a| {
                matches!(a, Action::Pass | Action::PutTrigger { .. })
            });
        }
        act(&mut g, Action::ChooseOptional { accept: true });
        assert!(g.observe(player).search_choice.is_some());
        assert!(g.observe(player.other()).search_choice.is_none());
        let saved = g.snapshot();
        for replay in 0..2 {
            if replay == 1 {
                g.restore(&saved).unwrap();
            }
            for card in lands {
                act(&mut g, Action::ChooseSearch { card });
            }
            act(&mut g, Action::FinishSearch);
            settle(&mut g);
            for card in lands {
                assert_eq!(g.object_view(card).zone, Zone::Battlefield);
                assert!(g.object_view(card).tapped);
            }
        }
    }
}
