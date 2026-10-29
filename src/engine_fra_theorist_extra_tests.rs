use super::*;

#[test]
fn theorist_bounces_stolen_artifact_to_its_owner_after_source_departure() {
    for player in [Player::P0, Player::P1] {
        let mut g = scenario(player);
        // Use a supported placeholder so the draft definition remains gated.
        let source = put(&mut g, player, "Bear Cub", Zone::Battlefield);
        g.state.objects[source as usize].card =
            Card::from_name("The Theorist, Jace Beleren").unwrap();
        g.state.objects[source as usize].counters.loyalty = 3;
        g.register_battlefield_sources(source);
        let artifact = put(&mut g, player, "Goldvein Pick", Zone::Battlefield);
        g.change_controller(g.object_ref(artifact), player.other());
        act(
            &mut g,
            Action::AnnounceActivate {
                card: source,
                ability: 1,
                counter_payment: Counters::default(),
            },
        );
        act(
            &mut g,
            Action::ChooseTarget {
                target: Target::Card(artifact),
            },
        );
        if g.legal_actions().contains(&Action::FinishTargets) {
            act(&mut g, Action::FinishTargets);
        }
        pay(&mut g);
        g.move_object(source, Zone::Graveyard);
        let checkpoint = g.snapshot();
        for _ in 0..2 {
            settle(&mut g);
            assert_eq!(g.object_view(artifact).zone, Zone::Hand);
            assert_eq!(g.object_view(artifact).owner, player);
            assert_eq!(g.object_view(source).zone, Zone::Graveyard);
            g.restore(&checkpoint).unwrap();
        }
    }
}
