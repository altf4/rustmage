//! Brigid's paid face changes, exact transformation history, and transforming token copies.
use super::ability_timeline_tests::{cast, checked, named, settle, setup};
use super::ability_world::CharacteristicWorld;
use super::*;
const FRONT: &str = "Brigid, Clachan's Heart";
const BACK: &str = "Brigid, Doun's Mind";
fn payment(g: &mut Game) {
    g.prepare_triggers();
    for _ in 0..40 {
        if matches!(g.state.pending, Pending::ResolutionMana { .. }) {
            return;
        }
        let action = if g.state.pending == Pending::Priority {
            Action::Pass
        } else {
            g.legal_actions()
                .into_iter()
                .find(|a| matches!(a, Action::PutTrigger { .. }))
                .unwrap()
        };
        checked(g, action);
    }
    panic!("no transform payment");
}
fn transform_paid(g: &mut Game, color: usize) {
    g.begin_step(Step::Main1);
    payment(g);
    let mut mana = [0; 6];
    mana[color] = 1;
    checked(g, Action::PayResolutionMana { payment: mana });
    settle(g);
}
fn kithkin_count(g: &Game) -> usize {
    g.state
        .objects
        .iter()
        .filter(|o| {
            o.card == Card::from_name("@token/c48e9e904360e4cee1deeed8").unwrap()
                && o.zone == Zone::Battlefield
        })
        .count()
}
#[test]
fn ecl_transform_brigid_paid_front_and_both_first_main_choices() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let id = named(&mut g, p, FRONT, Zone::Hand);
        assert_eq!(
            g.state.objects[id as usize].card.identity(),
            "Brigid, Clachan's Heart // Brigid, Doun's Mind"
        );
        g.state.players[p.index()].mana = [1, 0, 0, 0, 0, 2];
        cast(&mut g, id, None, &[]);
        settle(&mut g);
        assert_eq!(kithkin_count(&g), 1);
        assert_eq!(
            g.stats(id),
            Stats {
                power: 3,
                toughness: 2
            }
        );
        assert_eq!(g.colors(id), 1);
        let reference = g.object_ref(id);
        g.state.players[p.index()].mana = [0, 0, 0, 0, 1, 0];
        transform_paid(&mut g, 4);
        assert_eq!(g.object_ref(id), reference);
        assert_eq!(g.state.objects[id as usize].card.name(), BACK);
        assert_eq!(g.colors(id), 16);
        assert_eq!(kithkin_count(&g), 1, "transforming does not enter");
        assert!(g.state.objects[id as usize].summoning_sick);
        assert_eq!(g.state.objects[id as usize].card.fixed_mana_value(), 3);
        g.state.players[p.index()].mana = [1, 0, 0, 0, 0, 0];
        transform_paid(&mut g, 0);
        assert_eq!(g.state.objects[id as usize].card.name(), FRONT);
        assert_eq!(kithkin_count(&g), 2, "front transform trigger");
        g.begin_step(Step::Main1);
        payment(&mut g);
        checked(&mut g, Action::DeclineResolutionMana);
        settle(&mut g);
        assert_eq!(g.state.objects[id as usize].card.name(), FRONT);
    }
}
#[test]
fn ecl_transform_brigid_mana_is_one_color_from_other_current_controlled_creatures() {
    for p in [Player::P0, Player::P1] {
        for color in [0, 4] {
            let mut g = setup(p);
            let id = named(&mut g, p, FRONT, Zone::Battlefield);
            named(&mut g, p, "Bear Cub", Zone::Battlefield);
            named(&mut g, p, "Chomping Changeling", Zone::Battlefield);
            named(&mut g, p.other(), "Bear Cub", Zone::Battlefield);
            named(&mut g, p, "Bear Cub", Zone::Hand);
            transform_paid(&mut g, 4);
            g.state.players[p.index()].mana = [0; 6];
            let mut production = [0; 6];
            production[color] = 2;
            let action = Action::AnnounceMana {
                card: id,
                ability: 0,
                counter_payment: Counters::default(),
                production,
            };
            checked(&mut g, action);
            checked(&mut g, Action::PayCosts { payment: [0; 6] });
            assert!(g.state.objects[id as usize].tapped);
            assert_eq!(g.state.players[p.index()].mana, production);
            assert!(
                !g.legal_actions()
                    .iter()
                    .any(|a| matches!(a,Action::AnnounceMana{card,..} if *card==id))
            );
        }
    }
}
#[test]
fn ecl_transform_brigid_preserves_counters_damage_growth_and_attachments() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let id = named(&mut g, p, FRONT, Zone::Battlefield);
        // Use a paid Aura with no ability removal so both face changes remain legal.
        let aura = named(&mut g, p, "Gilt-Leaf's Embrace", Zone::Hand);
        cast(&mut g, aura, None, &[Target::Card(id)]);
        settle(&mut g);
        g.add_counter(id, CounterKind::PlusOne, 1);
        g.state.objects[id as usize].damage = 1;
        g.state.objects[id as usize].growth = Stats {
            power: 2,
            toughness: 2,
        };
        let reference = g.object_ref(id);
        let stats = g.stats(id);
        transform_paid(&mut g, 4);
        assert_eq!(g.stats(id), stats);
        assert_eq!(g.state.objects[id as usize].damage, 1);
        assert_eq!(g.counter_count(id, CounterKind::PlusOne), 1);
        assert_eq!(g.state.objects[aura as usize].attached_to, Some(reference));
        assert_eq!(g.object_ref(id), reference);
    }
}
#[test]
fn ecl_transform_brigid_blink_resets_front_without_rebinding_old_trigger() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let id = named(&mut g, p, FRONT, Zone::Battlefield);
        transform_paid(&mut g, 4);
        let old = g.object_ref(id);
        g.begin_step(Step::Main1);
        g.prepare_triggers();
        let blink = named(&mut g, p, "Personify", Zone::Hand);
        cast(&mut g, blink, None, &[Target::Card(id)]);
        payment(&mut g);
        checked(
            &mut g,
            Action::PayResolutionMana {
                payment: [1, 0, 0, 0, 0, 0],
            },
        );
        settle(&mut g);
        assert_ne!(g.object_ref(id), old);
        assert_eq!(g.state.objects[id as usize].card.name(), FRONT);
        assert_eq!(kithkin_count(&g), 1);
        assert!(g.observe(p).transforms.is_empty());
        let zone = g.move_object_raw(id, Zone::Graveyard);
        g.finish_zone_events(&[zone], &[]);
        assert_eq!(g.state.objects[id as usize].card.name(), FRONT);
    }
}
#[test]
fn ecl_transform_brigid_only_own_first_main_and_available_trigger() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let id = named(&mut g, p, FRONT, Zone::Battlefield);
        g.state.active = p.other();
        g.begin_step(Step::Main1);
        assert!(g.state.pending_triggers.is_empty());
        g.state.active = p;
        g.begin_step(Step::Main2);
        assert!(g.state.pending_triggers.is_empty());
        let stop = named(&mut g, p, "Stop Cold", Zone::Hand);
        cast(&mut g, stop, None, &[Target::Card(id)]);
        settle(&mut g);
        g.begin_step(Step::Main1);
        g.prepare_triggers();
        assert!(g.state.stack.is_empty());
        assert_eq!(g.state.objects[id as usize].card.name(), FRONT);
    }
}
#[test]
fn ecl_transform_history_rejects_older_trigger_even_after_two_face_changes() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let id = named(&mut g, p, FRONT, Zone::Battlefield);
        // Explicit duplicate-trigger fixture isolates CR701.27f before a trigger doubler is admitted.
        let view = g.object_view(id);
        assert!(g.queue_trigger(&view, 1));
        assert!(g.queue_trigger(&view, 1));
        payment(&mut g);
        checked(
            &mut g,
            Action::PayResolutionMana {
                payment: [0, 0, 0, 0, 1, 0],
            },
        );
        settle_until_priority(&mut g);
        assert_eq!(g.state.objects[id as usize].card.name(), BACK);
        g.transform_permanent(id);
        assert_eq!(g.state.objects[id as usize].card.name(), FRONT);
        payment(&mut g); // the new token trigger resolves before the older transform ability
        checked(
            &mut g,
            Action::PayResolutionMana {
                payment: [0, 0, 0, 0, 1, 0],
            },
        );
        settle(&mut g);
        assert_eq!(g.state.objects[id as usize].card.name(), FRONT);
    }
}
fn settle_until_priority(g: &mut Game) {
    assert_eq!(g.state.pending, Pending::Priority);
    assert!(g.state.resolving.is_none());
}
#[test]
fn ecl_transform_copy_of_back_face_has_both_faces_and_its_front_mana_value() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let id = named(&mut g, p, FRONT, Zone::Battlefield);
        transform_paid(&mut g, 4);
        let recipe = g.copiable_recipe(g.object_ref(id));
        g.stage_copy_creation(p.other(), recipe, 1);
        settle(&mut g);
        let copy = g.state.copy_recipes.as_ref().unwrap()[0].object.card;
        assert!(g.state.objects[copy as usize].is_copy_token());
        assert_eq!(g.state.objects[copy as usize].card.name(), BACK);
        assert_eq!(g.state.objects[copy as usize].card.fixed_mana_value(), 3);
        g.state.active = p.other();
        g.state.priority = p.other();
        transform_paid(&mut g, 0);
        assert_eq!(g.state.objects[copy as usize].card.name(), FRONT);
        assert_eq!(
            g.copiable_recipe(g.object_ref(copy)).definition.name(),
            FRONT
        );
        assert_eq!(kithkin_count(&g), 1);
    }
}
#[test]
fn ecl_transform_dynamic_mana_pays_a_real_spell_and_zero_still_can_tap() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let id = named(&mut g, p, FRONT, Zone::Battlefield);
        transform_paid(&mut g, 4);
        g.state.players[p.index()].mana = [0; 6];
        checked(
            &mut g,
            Action::AnnounceMana {
                card: id,
                ability: 0,
                counter_payment: Counters::default(),
                production: [0; 6],
            },
        );
        checked(&mut g, Action::PayCosts { payment: [0; 6] });
        assert!(g.object_view(id).tapped);
        g.state.objects[id as usize].tapped = false;
        named(&mut g, p, "Bear Cub", Zone::Battlefield);
        named(&mut g, p, "Bear Cub", Zone::Battlefield);
        let draw = named(&mut g, p, "Divination", Zone::Hand);
        g.state.players[p.index()].mana = [0, 1, 0, 0, 0, 0];
        checked(&mut g, Action::AnnounceCast { card: draw });
        let mixed = Action::ActivateMana {
            card: id,
            ability: 0,
            payment: [0; 6],
            counter_payment: Counters::default(),
            production: [1, 0, 0, 0, 1, 0],
        };
        let before = g.snapshot();
        assert!(g.apply(p, mixed).is_err());
        assert_eq!(g.snapshot(), before);
        checked(
            &mut g,
            Action::ActivateMana {
                card: id,
                ability: 0,
                payment: [0; 6],
                counter_payment: Counters::default(),
                production: [2, 0, 0, 0, 0, 0],
            },
        );
        checked(
            &mut g,
            Action::PayCosts {
                payment: [2, 1, 0, 0, 0, 0],
            },
        );
        settle(&mut g);
        assert_eq!(g.state.objects[draw as usize].zone, Zone::Graveyard);
        assert!(g.object_view(id).tapped);
    }
}
#[test]
fn ecl_transform_back_faces_are_never_separate_deck_cards() {
    let back = Card::from_name(BACK).unwrap();
    assert!(!back.is_standard_legal());
    assert!(!Card::ALL.contains(&back));
    assert!(matches!(
        Game::new(Config {
            decks: [vec![back; 60], vec![Card::Forest; 60]],
            seed: 0,
            starting_player: Player::P0,
            deck_rules: DeckRules::Freeform
        }),
        Err(Error::InvalidDeck(_))
    ));
}
