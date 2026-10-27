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
    for p in [Player::P0, Player::P1] {
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
fn esper_flashback_returns_transformed_with_lore_and_finality() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        empty_library(&mut g, p);
        let source = put(
            &mut g,
            p,
            "Esper Origins // Summon: Esper Maduin",
            Zone::Graveyard,
        );
        let before = g.snapshot();
        checked(&mut g, Action::AnnounceFlashback { card: source });
        let pay = g
            .legal_actions()
            .into_iter()
            .find(|a| matches!(a, Action::PayCosts { .. }))
            .unwrap();
        checked(&mut g, pay);
        settle(&mut g);
        assert_eq!(g.object_view(source).zone, Zone::Battlefield);
        assert_eq!(g.object_view(source).card.name(), "Summon: Esper Maduin");
        assert_eq!((g.stats(source).power, g.stats(source).toughness), (4, 4));
        assert_eq!(g.counter_count(source, CounterKind::Finality), 1);
        assert_eq!(g.counter_count(source, CounterKind::Lore), 1);
        assert!(g.has_subtype(source, "Saga"));
        let after = g.snapshot();
        let mut branch = g.clone();
        branch.restore(&before).unwrap();
        checked(&mut branch, Action::AnnounceFlashback { card: source });
        let pay = branch
            .legal_actions()
            .into_iter()
            .find(|a| matches!(a, Action::PayCosts { .. }))
            .unwrap();
        checked(&mut branch, pay);
        settle(&mut branch);
        assert_eq!(branch.snapshot(), after);
        g.move_object(source, Zone::Graveyard);
        assert_eq!(g.object_view(source).zone, Zone::Exile);
        assert_eq!(g.object_view(source).card.name(), "Esper Origins");
    }
}

#[test]
fn esper_saga_reveals_without_drawing_and_grants_other_creatures() {
    for p in [Player::P0, Player::P1] {
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
fn esper_surveil_two_is_private_and_the_player_may_mill_one() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        empty_library(&mut g, p);
        let a = put(&mut g, p, "Forest", Zone::Library);
        let b = put(&mut g, p, "Shock", Zone::Library);
        let source = put(
            &mut g,
            p,
            "Esper Origins // Summon: Esper Maduin",
            Zone::Hand,
        );
        big_helpers::cast(&mut g, source, None, &[]);
        for _ in 0..30 {
            if matches!(g.state.pending, Pending::EffectLibrary { .. }) {
                break;
            }
            checked(&mut g, Action::Pass);
        }
        assert!(g.observe(p).library_choice.is_some());
        assert!(g.observe(p.other()).library_choice.is_none());
        checked(
            &mut g,
            Action::ArrangeLibrary {
                card: b,
                destination: LibraryDestination::Graveyard,
            },
        );
        checked(
            &mut g,
            Action::ArrangeLibrary {
                card: a,
                destination: LibraryDestination::Top,
            },
        );
        settle(&mut g);
        assert_eq!(g.object_view(b).zone, Zone::Graveyard);
        assert_eq!(g.object_view(a).zone, Zone::Library);
        assert_eq!(g.object_view(source).zone, Zone::Graveyard);
    }
}

#[test]
fn esper_transformed_entry_is_seen_as_a_creature_by_tap_replacement() {
    for p in [Player::P0, Player::P1] {
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
    for p in [Player::P0, Player::P1] {
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
    for p in [Player::P0, Player::P1] {
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
