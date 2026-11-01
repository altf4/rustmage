//! Focused printed outcomes; shared engine invariants live in core tests.
//! BIG tokens exercise paid legal actions, exact incarnations and owned continuations.
use super::*;
use crate::engine::testing::card_cases::big_helpers::{cast, checked, named, settle, setup};

fn enter(g: &mut Game, p: Player, name: &str) -> CardId {
    let id = named(g, p, name, Zone::Hand);
    cast_creature(g, id);
    settle(g);
    id
}
fn cast_creature(g: &mut Game, id: CardId) {
    checked(g, Action::AnnounceCast { card: id });
    while g.state.announcing.as_ref().is_some_and(|a| {
        a.item.source.is_some_and(|source| source.id == id) && a.item.kind == StackKind::Spell
    }) {
        let a = g
            .legal_actions()
            .into_iter()
            .find(|a| matches!(a, Action::PayCosts { .. }))
            .unwrap();
        checked(g, a);
    }
}
fn factory(name: &str) -> Card {
    Card::from_name(name).unwrap()
}
fn tokens(g: &Game, p: Player, subtype: &str) -> Vec<CardId> {
    g.state
        .objects
        .iter()
        .enumerate()
        .filter_map(|(i, o)| {
            let i = i as CardId;
            (o.zone == Zone::Battlefield
                && o.controller == p
                && o.is_token()
                && g.has_subtype(i, subtype))
            .then_some(i)
        })
        .collect()
}
fn activate(g: &mut Game, source: CardId, target: Option<CardId>) {
    checked(
        g,
        Action::AnnounceActivate {
            card: source,
            ability: 0,
            counter_payment: Counters::default(),
        },
    );
    if let Some(target) = target {
        checked(
            g,
            Action::ChooseTarget {
                target: Target::Card(target),
            },
        );
    }
    while g.state.announcing.is_some() {
        let a = g
            .legal_actions()
            .into_iter()
            .find(|a| matches!(a, Action::PayCosts { .. }))
            .unwrap();
        checked(g, a);
    }
}
fn reject(g: &mut Game, action: Action) {
    assert_eq!(
        g.apply(g.actor().unwrap(), action),
        Err(Error::IllegalAction)
    );
}

#[test]
fn salvager_enters_then_counts_only_current_controlled_creature_tokens() {
    {
        let p = Player::P0;
        let mut g = setup(p);
        let source = enter(&mut g, p, "Sandstorm Salvager");
        let golem = tokens(&g, p, "Golem")[0];
        assert_eq!((g.stats(golem).power, g.stats(golem).toughness), (3, 3));
        assert_eq!(g.current_kinds(golem), 17);
        reject(
            &mut g,
            Action::AnnounceActivate {
                card: source,
                ability: 0,
                counter_payment: Counters::default(),
            },
        );
        g.create_tokens(
            p.other(),
            factory("@token/639d226bebb0fc69fa4bd180"),
            1,
            false,
        );
        g.create_tokens(p, factory("@token/7a77609ddf2619e2bd02e81a"), 1, false);
        let enemy = tokens(&g, p.other(), "Golem")[0];
        g.state.objects[source as usize].summoning_sick = false;
        activate(&mut g, source, None);
        // The ability remains independent of its source after payment.
        g.move_object(source, Zone::Graveyard);
        settle(&mut g);
        assert_eq!(g.counter_count(golem, CounterKind::PlusOne), 1);
        assert!(g.keywords(golem).contains(Keywords::TRAMPLE));
        assert_eq!(g.counter_count(enemy, CounterKind::PlusOne), 0);
        g.finish_cleanup();
        assert!(!g.keywords(golem).contains(Keywords::TRAMPLE));
        assert_eq!(g.stats(golem).power, 4);
    }
}

#[test]
fn oltec_modes_copy_only_owned_artifact_tokens_and_copy_keeps_name() {
    {
        let p = Player::P0;
        let mut g = setup(p);
        named(&mut g, p, "Oltec Matterweaver", Zone::Battlefield);
        enter(&mut g, p, "Bear Cub");
        let gnome = tokens(&g, p, "Gnome")[0];
        assert_eq!(g.current_kinds(gnome), 17);
        let bear = named(&mut g, p, "Bear Cub", Zone::Hand);
        cast_creature(&mut g, bear);
        g.prepare_triggers();
        checked(&mut g, Action::ChooseMode { mode: 1 });
        reject(
            &mut g,
            Action::ChooseTarget {
                target: Target::Card(bear),
            },
        );
        checked(
            &mut g,
            Action::ChooseTarget {
                target: Target::Card(gnome),
            },
        );
        settle(&mut g);
        let copies = tokens(&g, p, "Gnome");
        assert_eq!(copies.len(), 2);
        for id in copies {
            assert_eq!(g.object_view(id).name(), "Gnome Token");
        }
        // A spell of another type never queues the creature cast trigger.
        let shock = named(&mut g, p, "Shock", Zone::Hand);
        cast(&mut g, shock, None, &[Target::Player(p.other())]);
        assert!(g.state.pending_triggers.is_empty());
        settle(&mut g);
    }
}

#[test]
fn helm_adds_one_map_per_artifact_creation_event_and_copies_maps() {
    {
        let p = Player::P0;
        let mut g = setup(p);
        let helm = named(&mut g, p, "Worldwalker Helm", Zone::Battlefield);
        g.create_tokens(p, factory("@token/639d226bebb0fc69fa4bd180"), 3, false);
        g.advance_replacements(false);
        settle(&mut g);
        assert_eq!(tokens(&g, p, "Golem").len(), 3);
        assert_eq!(tokens(&g, p, "Map").len(), 1);
        g.create_tokens(
            p.other(),
            factory("@token/639d226bebb0fc69fa4bd180"),
            1,
            false,
        );
        g.create_tokens(p, factory("@token/bat_2_1_black_flying"), 1, false);
        g.advance_replacements(false);
        settle(&mut g);
        assert_eq!(tokens(&g, p, "Map").len(), 1);
        let map = tokens(&g, p, "Map")[0];
        activate(&mut g, helm, Some(map));
        settle(&mut g);
        assert_eq!(tokens(&g, p, "Map").len(), 3);
        let copy = tokens(&g, p, "Map")
            .into_iter()
            .find(|&i| g.state.objects[i as usize].is_copy_token())
            .unwrap();
        assert_eq!(g.state.objects[copy as usize].card.activations().len(), 1);
    }
}

#[test]
fn molten_copies_added_artifact_type_but_temporary_haste_and_delayed_sacrifice_are_not_copiable() {
    {
        let p = Player::P0;
        let mut g = setup(p);
        let bear = named(&mut g, p, "Bear Cub", Zone::Battlefield);
        let spell = named(&mut g, p, "Molten Duplication", Zone::Hand);
        cast(&mut g, spell, None, &[Target::Card(bear)]);
        settle(&mut g);
        let copy = tokens(&g, p, "Bear")[0];
        assert_eq!(g.current_kinds(copy), 17);
        assert!(g.keywords(copy).contains(Keywords::HASTE));
        let recipe = g.copiable_recipe(g.object_ref(copy));
        assert!(recipe.exceptions.artifact);
        assert_eq!(recipe.exceptions.sacrifice_abilities, 0);
        g.stage_copy_creation(p, recipe, 1);
        settle(&mut g);
        let permanent = tokens(&g, p, "Bear")
            .into_iter()
            .find(|&i| i != copy)
            .unwrap();
        assert!(!g.keywords(permanent).contains(Keywords::HASTE));
        g.move_object(spell, Zone::Exile);
        g.begin_step(Step::End);
        settle(&mut g);
        assert!(!g.on_battlefield(copy));
        assert!(g.on_battlefield(permanent));
    }
}

#[test]
fn tyrant_observes_self_and_four_power_entries_then_copies_only_nontoken_death() {
    {
        let p = Player::P0;
        let mut g = setup(p);
        let tyrant = enter(&mut g, p, "Vaultborn Tyrant");
        assert_eq!(g.state.players[p.index()].life, 23);
        enter(&mut g, p, "Bear Cub");
        assert_eq!(g.state.players[p.index()].life, 23);
        let big = named(&mut g, p, "Apothecary Stomper", Zone::Hand);
        g.move_object(big, Zone::Battlefield);
        settle(&mut g);
        assert!(g.state.players[p.index()].life >= 26);
        let before = g.state.players[p.index()].life;
        g.move_object(tyrant, Zone::Graveyard);
        settle(&mut g);
        let copy = tokens(&g, p, "Dinosaur")[0];
        assert_eq!(g.current_kinds(copy), 17);
        assert!(g.keywords(copy).contains(Keywords::TRAMPLE));
        assert_eq!(g.state.players[p.index()].life, before + 3);
        g.move_object(copy, Zone::Graveyard);
        settle(&mut g);
        assert!(tokens(&g, p, "Dinosaur").is_empty());
    }
}

#[test]
fn synthesizer_checks_artifact_mana_threshold_and_construct_bonus_tracks_artifact_world() {
    {
        let p = Player::P0;
        let mut g = setup(p);
        enter(&mut g, p, "Simulacrum Synthesizer");
        assert!(tokens(&g, p, "Construct").is_empty());
        enter(&mut g, p, "Worldwalker Helm");
        let construct = tokens(&g, p, "Construct")[0];
        // Synthesizer, Helm, Construct and the Helm's replacement Map.
        assert_eq!(g.stats(construct).power, 4);
        let map = tokens(&g, p, "Map")[0];
        g.move_object(map, Zone::Exile);
        settle(&mut g);
        assert_eq!(g.stats(construct).power, 3);
        enter(&mut g, p, "Bear Cub");
        assert_eq!(tokens(&g, p, "Construct").len(), 1);
        assert_eq!(g.stats(construct).toughness, 3);
    }
}

#[test]
fn gambit_leaves_from_any_destination_and_all_penalties_use_departing_controller() {
    for destination in [Zone::Hand, Zone::Exile, Zone::Graveyard] {
        {
            let p = Player::P0;
            let mut g = setup(p);
            let gambit = enter(&mut g, p, "Greed's Gambit");
            assert_eq!(g.state.players[p.index()].life, 26);
            let bats = tokens(&g, p, "Bat");
            assert_eq!(bats.len(), 3);
            for bat in bats {
                assert_eq!(g.stats(bat).power, 2);
                assert!(g.keywords(bat).contains(Keywords::FLYING));
            }
            g.begin_step(Step::End);
            settle(&mut g);
            assert_eq!(g.state.players[p.index()].life, 24);
            assert_eq!(tokens(&g, p, "Bat").len(), 2);
            // Leave only two creatures; sacrifice as many as possible and still lose six.
            g.move_object(gambit, destination);
            settle(&mut g);
            assert_eq!(g.state.players[p.index()].life, 18);
            assert!(tokens(&g, p, "Bat").is_empty());
            assert_eq!(g.state.players[p.other().index()].life, 20);
        }
    }
}

#[test]
fn fixed_golem_copy_removes_pt_cda_and_retains_other_subtypes_through_copying() {
    {
        let p = Player::P0;
        let mut g = setup(p);
        named(&mut g, p, "Bear Cub", Zone::Battlefield);
        let definition = factory("Crusader of Odric");
        g.stage_copy_creation(
            p,
            CopyRecipe {
                definition,
                exceptions: CopyExceptions::GOLEM,
                overrides: None,
                faces: None,
            },
            1,
        );
        settle(&mut g);
        let copy = tokens(&g, p, "Golem")[0];
        assert_eq!((g.stats(copy).power, g.stats(copy).toughness), (3, 3));
        assert_eq!(g.current_kinds(copy), 17);
        assert!(g.object_view(copy).has_subtype("Golem"));
        assert!(g.has_subtype(copy, "Human"));
        let recipe = g.copiable_recipe(g.object_ref(copy));
        g.stage_copy_creation(p, recipe, 1);
        settle(&mut g);
        for id in tokens(&g, p, "Golem") {
            assert_eq!(g.stats(id).power, 3);
        }
    }
}

#[test]
fn artifact_copy_exceptions_participate_in_dragon_replacement() {
    {
        let p = Player::P0;
        for golem in [false, true] {
            let mut g = setup(p);
            named(&mut g, p, "Draconic Visitor", Zone::Battlefield);
            if golem {
                g.stage_copy_creation(
                    p,
                    CopyRecipe {
                        definition: factory("Bear Cub"),
                        exceptions: CopyExceptions::GOLEM,
                        overrides: None,
                        faces: None,
                    },
                    1,
                );
            } else {
                let bear = named(&mut g, p, "Bear Cub", Zone::Battlefield);
                let molten = named(&mut g, p, "Molten Duplication", Zone::Hand);
                cast(&mut g, molten, None, &[Target::Card(bear)]);
            }
            if golem {
                g.advance_replacements(false);
            }
            settle(&mut g);
            let dragons = tokens(&g, p, "Dragon");
            assert_eq!(dragons.len(), 1, "golem={golem}");
            assert!(!g.state.objects[dragons[0] as usize].is_copy_token());
            assert_eq!(g.stats(dragons[0]).power, 5);
            assert!(g.keywords(dragons[0]).contains(Keywords::FLYING));
            if !golem {
                assert!(g.keywords(dragons[0]).contains(Keywords::HASTE));
                g.begin_step(Step::End);
                settle(&mut g);
                assert!(!g.on_battlefield(dragons[0]));
            }
        }
    }
}
