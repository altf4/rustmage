//! Hideout lots and source subtypes in the existing speculative payment world.
use super::super::assurance::big_helpers::{checked, named, settle, setup};
use super::super::msh_hideout_falcon_tests::{finish, hideout, mana, types};
use super::*;

#[test]
fn msh_hideout_funding_kindred_live_types_virtual_departure_and_owned_keys() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let h = hideout(&mut g, p);
        let source = named(&mut g, p, "Clachan Festival", Zone::Battlefield);
        types(&mut g, source, &["Villain"]);
        let secret = named(&mut g, p, "Shock", Zone::Hand);
        g.state.players[p.index()].mana = [0, 0, 0, 0, 0, 4];
        checked(&mut g, mana(h, 1, 0));
        let reference = g.object_ref(source);
        let definition = g.state.objects[source as usize].card;
        let goal = FundingGoal {
            actor: p,
            required: [1, 0, 0, 0, 0, 0],
            generic: 4,
            reserve: None,
            source: Some((reference, 0, Counters::default(), definition, None)),
        };
        let before = g.snapshot();
        assert!(g.can_fund(goal.clone()));
        let mut keys = Vec::new();
        for villain in [false, true] {
            let mut world = FundingState::from_game(&g, &goal);
            world.prepare_entry_projection();
            let rows = &mut world
                .entry_projection
                .as_mut()
                .unwrap()
                .roles
                .creature_types;
            rows.retain(|row| row.recipient != reference);
            rows.push(subtypes::CreatureTypesEffectView {
                recipient: reference,
                source: reference,
                types: if villain { &["Villain"] } else { &["Kithkin"] },
                additional: false,
                timestamp: 100,
            });
            assert_eq!(world.reachable(&goal, &mut HashSet::new()), villain);
            let mut visits = 0;
            world.cost_departures(&[reference], Zone::Graveyard, &mut |departed| {
                visits += 1;
                assert_ne!(departed.reference(source), reference);
                assert_eq!(departed.reachable(&goal, &mut HashSet::new()), villain);
                keys.push(departed.key(&goal));
                false
            });
            assert_eq!(visits, 1);
        }
        assert!(keys[0] != keys[1]);
        assert_eq!(g.snapshot(), before);
        assert!(
            !g.observe(p.other())
                .objects
                .iter()
                .any(|o| o.reference.card == secret)
        );
        types(&mut g, source, &["Kithkin"]);
        assert!(!g.can_fund(goal.clone()));
        assert!(!g.legal_actions().contains(&Action::AnnounceActivate {
            card: source,
            ability: 0,
            counter_payment: Counters::default()
        }));
        types(&mut g, source, &["Villain"]);
        checked(
            &mut g,
            Action::AnnounceActivate {
                card: source,
                ability: 0,
                counter_payment: Counters::default(),
            },
        );
        finish(&mut g);
        settle(&mut g);
        assert!(g.mana_lots(p).is_empty());
        assert_eq!(g.state.players[p.index()].mana, [0; 6]);
    }
}
