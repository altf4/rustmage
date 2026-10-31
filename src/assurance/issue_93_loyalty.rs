//! Counter prohibition against real loyalty abilities. Blossombind is gated;
//! the explicitly injected Aura isolates its already-typed prohibition effect.
use super::*;

#[test]
fn positive_loyalty_cost_cannot_be_paid_when_counters_are_prohibited() {
    for player in [Player::P0, Player::P1] {
        let mut g = scenario(player);
        let walker = put(
            &mut g,
            player,
            "Chandra, Chill of Compliance",
            Zone::Battlefield,
        );
        g.state.objects[walker as usize].counters.loyalty = 7;
        let object = g.object_ref(walker);
        let timestamp = g.next_role_timestamp();
        g.state
            .roles
            .get_or_insert_with(Default::default)
            .animations
            .push(AnimationView {
                object,
                timestamp,
                artifact: false,
                land: false,
                preserve_card_types: false,
                food_only: false,
                artifact_noncreature: false,
                colors: None,
                creature_types: None,
                all_creature_types: false,
                base: Some((3, 3)),
                keywords: Keywords::NONE,
                expires_turn: None,
            });
        assert_eq!(g.current_kinds(walker) & (1 | 64), 1 | 64);
        let aura = put(&mut g, player, "Pacifism", Zone::Battlefield);
        g.state.objects[aura as usize].card = Card::from_name("Blossombind").unwrap();
        g.state.objects[aura as usize].attached_to = Some(object);
        assert!(replacements::counters_prohibited(&g, object));
        let lurker = put(&mut g, player, "Bear Cub", Zone::Battlefield);
        g.state.objects[lurker as usize].card = Card::from_name("Sanctum Lurker").unwrap();
        g.register_battlefield_sources(lurker);
        let granted = Action::AnnouncePlaneswalkerGrant {
            source: object,
            grantor: g.object_ref(lurker),
            ability: 0,
        };
        put(&mut g, player.other(), "Bear Cub", Zone::Battlefield);
        let positive = Action::AnnounceActivate {
            card: walker,
            ability: 0,
            counter_payment: Counters::default(),
        };
        assert!(!g.legal_actions().contains(&positive));
        reject(&mut g, positive.clone());
        assert!(!g.legal_actions().contains(&granted));
        reject(&mut g, granted.clone());
        assert!(g.legal_actions().contains(&Action::AnnounceActivate {
            card: walker,
            ability: 3,
            counter_payment: Counters::default(),
        }));
        assert!(g.legal_actions().contains(&Action::AnnounceActivate {
            card: walker,
            ability: 2,
            counter_payment: Counters::default(),
        }));
        assert!(!g.legal_actions().iter().any(|action| matches!(action,
            Action::Activate { card, ability: 0, .. }
            | Action::BeginActivate { card, ability: 0, .. } if *card == walker)));

        let snapshot = g.snapshot();
        // Suppressing the Aura's abilities removes its prohibition without
        // changing either object's incarnation or the stored loyalty history.
        let mut blank = g.activation_item(player, walker, 0);
        blank.targets = vec![BoundTarget {
            group: 0,
            target: Target::Card(aura),
            object: Some(g.object_ref(aura)),
        }];
        g.remove_target_abilities_until_end_of_turn(&blank);
        assert!(!replacements::counters_prohibited(&g, object));
        assert!(g.legal_actions().contains(&positive));
        assert!(g.legal_actions().contains(&granted));
        g.restore(&snapshot).unwrap();
        assert!(replacements::counters_prohibited(&g, object));
        reject(&mut g, positive.clone());
        for _ in 0..2 {
            g.move_object(aura, Zone::Graveyard);
            assert!(g.legal_actions().contains(&positive));
            assert!(g.legal_actions().contains(&granted));
            let revision = g.revision();
            let index = g
                .legal_actions()
                .iter()
                .position(|action| *action == positive)
                .unwrap();
            let mut indexed = g.clone();
            act(&mut g, positive.clone());
            indexed.apply_index(player, revision, index).unwrap();
            assert_eq!(g.snapshot(), indexed.snapshot());
            drop(indexed);
            pay(&mut g);
            assert_eq!(g.counter_count(walker, CounterKind::Loyalty), 8);
            assert!(g.object_view(walker).loyalty_activated_this_turn);
            g.restore(&snapshot).unwrap();
        }
        g.move_object(aura, Zone::Graveyard);
        act(&mut g, granted);
        pay(&mut g);
        assert_eq!(g.counter_count(walker, CounterKind::Loyalty), 9);
        assert!(g.object_view(walker).loyalty_activated_this_turn);
    }
}
