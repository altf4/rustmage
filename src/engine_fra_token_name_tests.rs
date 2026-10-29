use super::*;

#[test]
fn created_token_rules_names_are_public_and_respected_by_distinct_name_costs() {
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        let kraken = put(&mut g, p, "Verdant Kraken", Zone::Battlefield);
        let font = put(&mut g, p, "Transmutation Font", Zone::Battlefield);
        g.begin_step(Step::Upkeep);
        g.prepare_triggers();
        settle(&mut g);
        let first = g
            .state
            .objects
            .iter()
            .enumerate()
            .find_map(|(id, o)| {
                (o.is_token() && o.zone == Zone::Battlefield && o.card == Card::test_index(5454))
                    .then_some(id as CardId)
            })
            .unwrap();
        assert_eq!(g.object_view(first).name(), "Forest Tentacle Token");
        for viewer in [p, p.other()] {
            assert_eq!(
                g.observe(viewer)
                    .objects
                    .iter()
                    .find(|o| o.reference.card == first)
                    .unwrap()
                    .name(),
                "Forest Tentacle Token"
            );
        }
        g.create_tokens(p, Card::test_index(5454), 1, false);
        g.create_tokens(p, Card::test_index(5453), 1, false);
        g.create_tokens(p, Card::test_index(5455), 1, false);
        let second = g
            .state
            .objects
            .iter()
            .enumerate()
            .find_map(|(id, o)| {
                (id as CardId != first
                    && o.is_token()
                    && o.zone == Zone::Battlefield
                    && o.card == Card::test_index(5454))
                .then_some(id as CardId)
            })
            .unwrap();
        let lotus = g
            .state
            .objects
            .iter()
            .enumerate()
            .find_map(|(id, o)| {
                (o.is_token() && o.zone == Zone::Battlefield && o.card == Card::test_index(5453))
                    .then_some(id as CardId)
            })
            .unwrap();
        let leviathan = g
            .state
            .objects
            .iter()
            .enumerate()
            .find_map(|(id, o)| {
                (o.is_token() && o.zone == Zone::Battlefield && o.card == Card::test_index(5455))
                    .then_some(id as CardId)
            })
            .unwrap();
        assert_eq!(g.object_view(lotus).name(), "Lotus");
        assert_eq!(g.object_view(leviathan).name(), "Leviathan Token");
        // Font requires artifact tokens. Copy each creature token's own
        // characteristics with an artifact exception, preserving its name.
        for id in [first, second, leviathan] {
            let object = g.object_ref(id);
            let recipe = CopyRecipe::new(g.state.objects[id as usize].card)
                .with_exceptions(CopyExceptions::ARTIFACT);
            g.apply_temporary_copy(object, recipe, CopyDeadline::EndOfTurn);
            assert_ne!(g.object_view(id).kinds & 16, 0);
        }
        assert_eq!(g.object_view(first).name(), "Forest Tentacle Token");
        assert_eq!(g.object_view(second).name(), "Forest Tentacle Token");
        assert_eq!(g.object_view(leviathan).name(), "Leviathan Token");
        g.state.step = Step::Main1;
        g.state.priority = p;
        let [first_ref, second_ref, lotus_ref, leviathan_ref] =
            [first, second, lotus, leviathan].map(|id| g.object_ref(id));
        act(
            &mut g,
            Action::AnnounceActivate {
                card: font,
                ability: 1,
                counter_payment: Counters::default(),
            },
        );
        act(&mut g, Action::ChooseAdditionalCost { object: first_ref });
        reject(&mut g, Action::ChooseAdditionalCost { object: second_ref });
        act(&mut g, Action::ChooseAdditionalCost { object: lotus_ref });
        act(
            &mut g,
            Action::ChooseAdditionalCost {
                object: leviathan_ref,
            },
        );
        assert_eq!(g.object_view(kraken).zone, Zone::Battlefield);
    }
}
