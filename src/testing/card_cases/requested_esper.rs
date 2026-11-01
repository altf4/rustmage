//! Focused printed outcomes; shared engine invariants live in core tests.
//! Esper Origins' flashback and its transformed Saga.
use super::big_helpers::{checked, settle, setup};
use super::*;

fn empty_library(g: &mut Game, p: Player) {
    for id in std::mem::take(&mut g.state.players[p.index()].library) {
        g.state.objects[id as usize].zone = Zone::Outside;
    }
}

#[test]
fn esper_origins_normal_cast_resolves_to_graveyard_and_cannot_cast_back() {
    {
        let p = Player::P0;
        let mut g = setup(p);
        empty_library(&mut g, p);
        let source = put(
            &mut g,
            p,
            "Esper Origins // Summon: Esper Maduin",
            Zone::Hand,
        );
        assert!(
            !g.legal_actions()
                .contains(&Action::AnnounceBack { card: source })
        );
        let life = g.state.players[p.index()].life;
        big_helpers::cast(&mut g, source, None, &[]);
        settle(&mut g);
        assert_eq!(g.state.players[p.index()].life, life + 2);
        assert_eq!(g.object_view(source).zone, Zone::Graveyard);
        assert!(
            g.legal_actions()
                .contains(&Action::AnnounceFlashback { card: source })
        );
    }
}

#[test]
fn esper_saga_reveals_without_drawing_and_grants_other_creatures() {
    {
        let p = Player::P0;
        for (name, expected) in [("Forest", Zone::Hand), ("Shock", Zone::Library)] {
            let mut g = setup(p);
            empty_library(&mut g, p);
            let source = put(
                &mut g,
                p,
                "Esper Origins // Summon: Esper Maduin",
                Zone::Graveyard,
            );
            let ally = put(&mut g, p, "Bear Cub", Zone::Battlefield);
            let enemy = put(&mut g, p.other(), "Bear Cub", Zone::Battlefield);
            checked(&mut g, Action::AnnounceFlashback { card: source });
            let pay = g
                .legal_actions()
                .into_iter()
                .find(|a| matches!(a, Action::PayCosts { .. }))
                .unwrap();
            checked(&mut g, pay);
            settle(&mut g);
            let top = put(&mut g, p, name, Zone::Library);
            let draws = g.state.players[p.index()].cards_drawn_this_turn;
            assert!(g.queue_trigger(&g.object_view(source), 0));
            settle(&mut g);
            assert_eq!(g.object_view(top).zone, expected);
            assert_eq!(g.state.players[p.index()].cards_drawn_this_turn, draws);
            assert_eq!(
                g.observe(p.other())
                    .last_library_reveal
                    .as_ref()
                    .unwrap()
                    .revealed
                    .card
                    .name(),
                name
            );
            let mana = g.state.players[p.index()].mana[4];
            g.begin_step(Step::Main1);
            g.give_priority();
            g.prepare_triggers();
            settle(&mut g);
            assert_eq!(g.state.players[p.index()].mana[4], mana + 2);
            g.begin_step(Step::Main1);
            g.give_priority();
            g.prepare_triggers();
            settle(&mut g);
            assert_eq!(g.stats(ally).power, 4);
            assert!(g.keywords(ally).contains(Keywords::TRAMPLE));
            assert_eq!(g.stats(enemy).power, 2);
            assert!(!g.keywords(enemy).contains(Keywords::TRAMPLE));
            assert_eq!(g.object_view(source).zone, Zone::Exile);
        }
    }
}

#[test]
fn esper_transformed_entry_is_seen_as_a_creature_by_tap_replacement() {
    {
        let p = Player::P0;
        let mut g = setup(p);
        empty_library(&mut g, p);
        put(
            &mut g,
            p.other(),
            "Authority of the Consuls",
            Zone::Battlefield,
        );
        let source = put(
            &mut g,
            p,
            "Esper Origins // Summon: Esper Maduin",
            Zone::Graveyard,
        );
        checked(&mut g, Action::AnnounceFlashback { card: source });
        let pay = g
            .legal_actions()
            .into_iter()
            .find(|a| matches!(a, Action::PayCosts { .. }))
            .unwrap();
        checked(&mut g, pay);
        settle(&mut g);
        assert_eq!(g.object_view(source).zone, Zone::Battlefield);
        assert!(g.object_view(source).tapped);
        assert_eq!(g.counter_count(source, CounterKind::Lore), 1);
        assert_eq!(g.counter_count(source, CounterKind::Finality), 1);
    }
}

#[test]
fn transformed_saga_and_copy_have_distinct_mana_values_for_effects() {
    {
        let p = Player::P0;
        let mut g = setup(p);
        empty_library(&mut g, p);
        let source = put(
            &mut g,
            p,
            "Esper Origins // Summon: Esper Maduin",
            Zone::Graveyard,
        );
        let copy = put(&mut g, p, "Bear Cub", Zone::Battlefield);
        checked(&mut g, Action::AnnounceFlashback { card: source });
        let pay = g
            .legal_actions()
            .into_iter()
            .find(|a| matches!(a, Action::PayCosts { .. }))
            .unwrap();
        checked(&mut g, pay);
        settle(&mut g);
        let tokens = g.create_copy_groups_inner(
            p,
            &[TokenGroupView {
                token: Card::from_name("Summon: Esper Maduin").unwrap(),
                amount: 1,
                tapped: false,
            }],
            &[Some(CopyRecipe::new(
                Card::from_name("Summon: Esper Maduin").unwrap(),
            ))],
            None,
        );
        let token = tokens[0].card;
        settle(&mut g);
        assert_eq!(g.object_mana_value(token), 2);
        let recipe = CopyRecipe::new(Card::from_name("Summon: Esper Maduin").unwrap());
        g.apply_temporary_copy(g.object_ref(copy), recipe, CopyDeadline::EndOfTurn);
        assert_eq!(g.object_mana_value(source), 2);
        assert_eq!(g.object_mana_value(copy), 0);
        let sweep = put(&mut g, p, "Pest Control", Zone::Hand);
        big_helpers::cast(&mut g, sweep, None, &[]);
        settle(&mut g);
        assert_eq!(g.object_view(source).zone, Zone::Battlefield);
        assert_eq!(g.object_view(token).zone, Zone::Battlefield);
        assert_eq!(g.object_view(copy).zone, Zone::Graveyard);
    }
}

#[test]
fn transformed_saga_is_a_legal_two_mana_target() {
    {
        let p = Player::P0;
        let mut g = setup(p);
        empty_library(&mut g, p);
        let source = put(
            &mut g,
            p,
            "Esper Origins // Summon: Esper Maduin",
            Zone::Graveyard,
        );
        checked(&mut g, Action::AnnounceFlashback { card: source });
        let pay = g
            .legal_actions()
            .into_iter()
            .find(|a| matches!(a, Action::PayCosts { .. }))
            .unwrap();
        checked(&mut g, pay);
        settle(&mut g);
        g.state.active = p.other();
        g.state.priority = p.other();
        let seam = put(&mut g, p.other(), "Seam Rip", Zone::Hand);
        big_helpers::cast(&mut g, seam, None, &[]);
        for _ in 0..30 {
            if matches!(g.state.pending, Pending::Targets { .. }) {
                break;
            }
            let action = g
                .legal_actions()
                .into_iter()
                .find(|a| matches!(a, Action::Pass | Action::PutTrigger { .. }))
                .unwrap();
            checked(&mut g, action);
        }
        checked(
            &mut g,
            Action::ChooseTarget {
                target: Target::Card(source),
            },
        );
        settle(&mut g);
        assert_eq!(g.object_view(source).zone, Zone::Exile);
    }
}
