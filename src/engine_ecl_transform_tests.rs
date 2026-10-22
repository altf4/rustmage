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
const ASHLING: &str = "Ashling, Rekindled";
fn until_effect(g: &mut Game) {
    g.prepare_triggers();
    for _ in 0..60 {
        if matches!(
            g.state.pending,
            Pending::EffectChoice { .. } | Pending::EffectMana { .. }
        ) {
            return;
        }
        let a = if g.state.pending == Pending::Priority {
            Action::Pass
        } else {
            g.legal_actions()
                .into_iter()
                .find(|a| matches!(a, Action::PutTrigger { .. }))
                .unwrap()
        };
        checked(g, a);
    }
    panic!("missing effect decision");
}
fn ashling_back_mana(g: &mut Game, color: usize) {
    g.begin_step(Step::Main1);
    payment(g);
    checked(
        g,
        Action::PayResolutionMana {
            payment: [0, 1, 0, 0, 0, 0],
        },
    );
    until_effect(g);
    let mut production = [0; 6];
    production[color] = 2;
    assert!(!g.legal_actions().contains(&Action::ChooseMana {
        production: [1, 1, 0, 0, 0, 0]
    }));
    assert!(!g.legal_actions().contains(&Action::ChooseMana {
        production: [0, 0, 0, 0, 0, 2]
    }));
    checked(g, Action::ChooseMana { production });
    settle(g);
}
#[test]
fn ecl_transform_ashling_paid_entry_optional_discard_is_private_and_draws_only_after_payment() {
    for p in [Player::P0, Player::P1] {
        for pay in [false, true] {
            let mut g = setup(p);
            let id = named(&mut g, p, ASHLING, Zone::Hand);
            let fodder = named(&mut g, p, "Island", Zone::Hand);
            let top = named(&mut g, p, "Forest", Zone::Library);
            g.state.players[p.index()].mana = [0, 0, 0, 1, 0, 1];
            cast(&mut g, id, None, &[]);
            until_effect(&mut g);
            assert!(g.observe(p).effect_choice.is_some());
            assert!(g.observe(p.other()).effect_choice.is_none());
            if pay {
                checked(&mut g, Action::Discard { card: fodder });
            } else {
                checked(&mut g, Action::DeclineEffectPayment);
            }
            settle(&mut g);
            assert_eq!(
                g.state.objects[fodder as usize].zone,
                if pay { Zone::Graveyard } else { Zone::Hand }
            );
            assert_eq!(
                g.state.objects[top as usize].zone,
                if pay { Zone::Hand } else { Zone::Library }
            );
            assert_eq!(
                g.stats(id),
                Stats {
                    power: 1,
                    toughness: 3
                }
            );
        }
    }
}
#[test]
fn ecl_transform_ashling_back_generates_two_of_each_single_color_with_owned_restriction() {
    for p in [Player::P0, Player::P1] {
        for color in 0..5 {
            let mut g = setup(p);
            let id = named(&mut g, p, ASHLING, Zone::Battlefield);
            g.state.players[p.index()].mana = [0, 1, 0, 0, 0, 0];
            ashling_back_mana(&mut g, color);
            assert_eq!(
                g.state.objects[id as usize].card.name(),
                "Ashling, Rimebound"
            );
            assert_eq!(g.state.objects[id as usize].card.fixed_mana_value(), 2);
            let mut expected = [0; 6];
            expected[color] = 2;
            assert_eq!(g.state.players[p.index()].mana, expected);
            let lot = &g.observe(p).mana_lots[0];
            assert_eq!(
                lot.restriction,
                ManaRestrictionView::SpellManaValueAtLeast(4)
            );
            assert_eq!(lot.amounts, expected);
            assert_eq!(g.colors(id), 2);
            // The resulting restriction cannot pay another resolution-time transform cost.
            g.begin_step(Step::Main1);
            payment(&mut g);
            assert!(
                !g.legal_actions()
                    .iter()
                    .any(|a| matches!(a, Action::PayResolutionMana { .. }))
            );
            checked(&mut g, Action::DeclineResolutionMana);
            until_effect(&mut g);
            checked(
                &mut g,
                Action::ChooseMana {
                    production: expected,
                },
            );
            settle(&mut g);
            assert_eq!(g.state.players[p.index()].mana[color], 4);
            assert_eq!(
                g.state.objects[id as usize].card.name(),
                "Ashling, Rimebound"
            );
        }
    }
}
#[test]
fn ecl_transform_ashling_mana_survives_source_departure_but_only_casts_mv_four_spells() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let id = named(&mut g, p, ASHLING, Zone::Battlefield);
        g.state.players[p.index()].mana = [0, 1, 0, 0, 0, 0];
        ashling_back_mana(&mut g, 3);
        let event = g.move_object_raw(id, Zone::Graveyard);
        g.finish_zone_events(&[event], &[]);
        assert_eq!(g.state.objects[id as usize].card.name(), ASHLING);
        g.state.players[p.index()].mana[5] = 2;
        let low = named(&mut g, p, "Goblin Piker", Zone::Hand);
        let four = named(&mut g, p, "Hill Giant", Zone::Hand);
        assert!(
            !g.legal_actions()
                .contains(&Action::AnnounceCast { card: low })
        );
        let before = g.snapshot();
        assert!(g.apply(p, Action::AnnounceCast { card: low }).is_err());
        assert_eq!(g.snapshot(), before);
        cast(&mut g, four, None, &[]);
        settle(&mut g);
        assert!(g.on_battlefield(four));
        assert_eq!(g.state.players[p.index()].mana, [0; 6]);
    }
}
#[test]
fn ecl_transform_ashling_x_spell_interval_crosses_mana_value_threshold_exactly() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        named(&mut g, p, ASHLING, Zone::Battlefield);
        g.state.players[p.index()].mana = [0, 1, 0, 0, 0, 0];
        ashling_back_mana(&mut g, 4);
        // Ordinary green + generic pay X0..1; restricted green unlocks X3, leaving X2 illegal.
        g.state.players[p.index()].mana[5] = 1;
        g.state.players[p.index()].mana[4] += 1;
        let scourge = named(&mut g, p, "Wildwood Scourge", Zone::Hand);
        checked(&mut g, Action::AnnounceCast { card: scourge });
        assert_eq!(
            g.x_choice().unwrap().intervals,
            vec![
                XInterval {
                    minimum: 0,
                    maximum: 1
                },
                XInterval {
                    minimum: 3,
                    maximum: 3
                }
            ]
        );
        checked(&mut g, Action::ChooseX { value: 3 });
        for _ in 0..10 {
            if g.state.announcing.is_none() {
                break;
            }
            let actions = g.legal_actions();
            let a = if actions.contains(&Action::ChooseTarget {
                target: Target::Player(p.other()),
            }) {
                Action::ChooseTarget {
                    target: Target::Player(p.other()),
                }
            } else {
                actions
                    .into_iter()
                    .find(|a| matches!(a, Action::FinishTargets | Action::PayCosts { .. }))
                    .unwrap()
            };
            checked(&mut g, a);
        }
        assert_eq!(g.state.stack.last().unwrap().mana_value(), 4);
        settle(&mut g);
        assert_eq!(g.counter_count(scourge, CounterKind::PlusOne), 3);
        assert_eq!(
            g.stats(scourge),
            Stats {
                power: 3,
                toughness: 3
            }
        );
    }
}
#[test]
fn ecl_transform_ashling_return_to_front_loots_and_mana_trigger_works_after_face_changes() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let id = named(&mut g, p, ASHLING, Zone::Battlefield);
        g.state.players[p.index()].mana = [0, 1, 0, 0, 0, 0];
        ashling_back_mana(&mut g, 4);
        let fodder = named(&mut g, p, "Island", Zone::Hand);
        let top = named(&mut g, p, "Forest", Zone::Library);
        g.state.players[p.index()].mana[3] = 1;
        g.begin_step(Step::Main1);
        payment(&mut g);
        checked(
            &mut g,
            Action::PayResolutionMana {
                payment: [0, 0, 0, 1, 0, 0],
            },
        );
        until_effect(&mut g);
        checked(&mut g, Action::Discard { card: fodder });
        until_effect(&mut g);
        checked(
            &mut g,
            Action::ChooseMana {
                production: [0, 0, 0, 0, 2, 0],
            },
        );
        settle(&mut g);
        assert_eq!(g.state.objects[id as usize].card.name(), ASHLING);
        assert_eq!(g.state.objects[top as usize].zone, Zone::Hand);
        assert_eq!(g.state.players[p.index()].mana[4], 4);
    }
}
const TRYSTAN: &str = "Trystan, Callous Cultivator";
fn mill_three(g: &mut Game, p: Player, elf: bool) -> [CardId; 3] {
    [
        named(
            g,
            p,
            if elf { "Nameless Inversion" } else { "Island" },
            Zone::Library,
        ),
        named(g, p, "Forest", Zone::Library),
        named(g, p, "Bear Cub", Zone::Library),
    ]
}
#[test]
fn ecl_transform_trystan_paid_front_mills_before_testing_owned_elf_cards() {
    for p in [Player::P0, Player::P1] {
        for elf in [false, true] {
            let mut g = setup(p);
            let milled = mill_three(&mut g, p, elf);
            named(&mut g, p.other(), "Llanowar Elves", Zone::Graveyard);
            named(&mut g, p, "Llanowar Elves", Zone::Hand);
            let id = named(&mut g, p, TRYSTAN, Zone::Hand);
            g.state.players[p.index()].mana = [0, 0, 0, 0, 1, 2];
            cast(&mut g, id, None, &[]);
            settle(&mut g);
            for card in milled {
                assert_eq!(g.state.objects[card as usize].zone, Zone::Graveyard);
            }
            assert_eq!(g.state.players[p.index()].life, if elf { 22 } else { 20 });
            assert_eq!(
                g.stats(id),
                Stats {
                    power: 3,
                    toughness: 4
                }
            );
            assert!(g.keywords(id).contains(Keywords::DEATHTOUCH));
            assert_eq!(g.state.players[p.index()].mana, [0; 6]);
        }
    }
}
#[test]
fn ecl_transform_trystan_back_optional_non_target_elf_card_exile_follows_mill() {
    for p in [Player::P0, Player::P1] {
        for pay in [false, true] {
            let mut g = setup(p);
            let id = named(&mut g, p, TRYSTAN, Zone::Battlefield);
            let milled = mill_three(&mut g, p, true);
            let prior = named(&mut g, p, "Llanowar Elves", Zone::Graveyard);
            let foe = named(&mut g, p.other(), "Llanowar Elves", Zone::Graveyard);
            g.begin_step(Step::Main1);
            payment(&mut g);
            checked(
                &mut g,
                Action::PayResolutionMana {
                    payment: [0, 0, 1, 0, 0, 0],
                },
            );
            until_effect(&mut g);
            assert_eq!(
                g.state.objects[id as usize].card.name(),
                "Trystan, Penitent Culler"
            );
            assert!(
                g.observe(p.other()).effect_choice.is_some(),
                "graveyard cost choices are public"
            );
            for card in milled {
                assert_eq!(g.state.objects[card as usize].zone, Zone::Graveyard);
            }
            assert!(
                g.legal_actions().contains(&Action::ExileGraveyardCard {
                    object: g.object_ref(prior)
                }),
                "not limited to the newly milled batch"
            );
            let before = g.snapshot();
            assert!(
                g.apply(
                    p,
                    Action::ExileGraveyardCard {
                        object: g.object_ref(foe)
                    }
                )
                .is_err()
            );
            assert_eq!(g.snapshot(), before);
            assert!(!g.legal_actions().contains(&Action::ExileGraveyardCard {
                object: g.object_ref(milled[1])
            }));
            if pay {
                let object = g.object_ref(milled[0]);
                checked(&mut g, Action::ExileGraveyardCard { object });
            } else {
                checked(&mut g, Action::DeclineEffectPayment);
            }
            settle(&mut g);
            assert_eq!(
                g.state.objects[milled[0] as usize].zone,
                if pay { Zone::Exile } else { Zone::Graveyard }
            );
            assert_eq!(
                g.state.players[p.other().index()].life,
                if pay { 18 } else { 20 }
            );
            assert_eq!(g.state.players[p.index()].life, 20);
            assert!(g.keywords(id).contains(Keywords::DEATHTOUCH));
        }
    }
}
#[test]
fn ecl_transform_trystan_back_without_elf_does_not_drain_and_front_transformation_mills_again() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let id = named(&mut g, p, TRYSTAN, Zone::Battlefield);
        let first = mill_three(&mut g, p, false);
        transform_paid(&mut g, 2);
        assert_eq!(g.state.players[p.other().index()].life, 20);
        assert_eq!(
            g.state.objects[id as usize].card.name(),
            "Trystan, Penitent Culler"
        );
        for card in first {
            assert_eq!(g.state.objects[card as usize].zone, Zone::Graveyard);
        }
        let second = mill_three(&mut g, p, true);
        let reference = g.object_ref(id);
        transform_paid(&mut g, 4);
        assert_eq!(g.object_ref(id), reference);
        assert_eq!(g.state.objects[id as usize].card.name(), TRYSTAN);
        assert_eq!(g.state.players[p.index()].life, 22);
        for card in second {
            assert_eq!(g.state.objects[card as usize].zone, Zone::Graveyard);
        }
    }
}
#[test]
fn ecl_transform_trystan_back_trigger_survives_paid_bounce_and_exile_cost_rejects_tokens() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let id = named(&mut g, p, TRYSTAN, Zone::Battlefield);
        let milled = mill_three(&mut g, p, true);
        g.begin_step(Step::Main1);
        payment(&mut g);
        checked(
            &mut g,
            Action::PayResolutionMana {
                payment: [0, 0, 1, 0, 0, 0],
            },
        );
        let bounce = named(&mut g, p, "Unsummon", Zone::Hand);
        cast(&mut g, bounce, None, &[Target::Card(id)]);
        until_effect(&mut g);
        assert_eq!(g.state.objects[id as usize].zone, Zone::Hand);
        assert_eq!(g.state.objects[id as usize].card.name(), TRYSTAN);
        let token = Card::catalogue()
            .find(|c| c.is_token() && c.has_subtype("Elf"))
            .unwrap();
        // Explicit pre-SBA fixture isolates the cost's card-only restriction.
        let token = super::tests::spawn(&mut g, p, token, Zone::Graveyard);
        assert!(!g.legal_actions().contains(&Action::ExileGraveyardCard {
            object: g.object_ref(token)
        }));
        let object = g.object_ref(milled[0]);
        checked(&mut g, Action::ExileGraveyardCard { object });
        settle(&mut g);
        assert_eq!(g.state.players[p.other().index()].life, 18);
        assert_eq!(g.state.objects[id as usize].zone, Zone::Hand);
    }
}
#[test]
fn ecl_transform_trystan_copy_enters_back_without_transform_mill_or_exile_trigger() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let id = named(&mut g, p, TRYSTAN, Zone::Battlefield);
        transform_paid(&mut g, 2);
        let library = mill_three(&mut g, p.other(), true);
        let recipe = g.copiable_recipe(g.object_ref(id));
        g.stage_copy_creation(p.other(), recipe, 1);
        settle(&mut g);
        for card in library {
            assert_eq!(g.state.objects[card as usize].zone, Zone::Library);
        }
        assert_eq!(g.state.players[p.index()].life, 20);
        let copy = g.state.copy_recipes.as_ref().unwrap()[0].object.card;
        assert_eq!(
            g.state.objects[copy as usize].card.name(),
            "Trystan, Penitent Culler"
        );
        assert_eq!(g.state.objects[copy as usize].card.fixed_mana_value(), 3);
        assert!(g.keywords(copy).contains(Keywords::DEATHTOUCH));
    }
}
const EIRDU: &str = "Eirdu, Carrier of Dawn";
const ISILU: &str = "Isilu, Carrier of Twilight";
fn murder(g: &mut Game, p: Player, victim: CardId) {
    let spell = named(g, p, "Murder", Zone::Hand);
    cast(g, spell, None, &[Target::Card(victim)]);
    settle(g);
}
#[test]
fn ecl_transform_eirdu_paid_faces_and_creature_convoke_from_summoning_sick_sources() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let god = named(&mut g, p, EIRDU, Zone::Hand);
        g.state.players[p.index()].mana = [2, 0, 0, 0, 0, 3];
        cast(&mut g, god, None, &[]);
        settle(&mut g);
        assert_eq!(
            g.stats(god),
            Stats {
                power: 5,
                toughness: 5
            }
        );
        assert!(
            g.keywords(god)
                .contains(Keywords::FLYING.union(Keywords::LIFELINK))
        );
        let elf = named(&mut g, p, "Llanowar Elves", Zone::Battlefield);
        let hill = named(&mut g, p, "Hill Giant", Zone::Battlefield);
        let spell = named(&mut g, p, "Great Forest Druid", Zone::Hand);
        g.state.players[p.index()].mana = [0; 6];
        checked(&mut g, Action::AnnounceCast { card: spell });
        checked(&mut g, Action::BeginConvoke);
        for (id, contribution) in [
            (god, ConvokeContribution::Generic),
            (elf, ConvokeContribution::Green),
        ] {
            let object = g.object_ref(id);
            checked(
                &mut g,
                Action::ChooseConvoker {
                    object,
                    contribution,
                },
            );
        }
        let pay = g
            .legal_actions()
            .into_iter()
            .find(|a| matches!(a, Action::PayCosts { .. }))
            .unwrap();
        checked(&mut g, pay);
        settle(&mut g);
        assert_eq!(g.object_view(spell).zone, Zone::Battlefield);
        assert!(
            g.object_view(god).tapped && g.object_view(elf).tapped && !g.object_view(hill).tapped
        );
        g.state.players[p.index()].mana = [0, 0, 1, 0, 0, 0];
        transform_paid(&mut g, 2);
        assert_eq!(g.state.objects[god as usize].card.name(), ISILU);
        assert_eq!(g.colors(god), 4);
        assert_eq!(g.state.objects[god as usize].card.fixed_mana_value(), 5);
        g.state.players[p.index()].mana = [1, 0, 0, 0, 0, 0];
        transform_paid(&mut g, 0);
        assert_eq!(g.state.objects[god as usize].card.name(), EIRDU);
    }
}
#[test]
fn ecl_transform_eirdu_grant_excludes_noncreatures_opponents_and_blanked_provider() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let god = named(&mut g, p, EIRDU, Zone::Battlefield);
        assert!(g.spell_has_convoke(p, Card::from_name("Great Forest Druid").unwrap()));
        assert!(!g.spell_has_convoke(p.other(), Card::from_name("Great Forest Druid").unwrap()));
        assert!(!g.spell_has_convoke(p, Card::from_name("Divination").unwrap()));
        let aura = named(&mut g, p, "Noggle the Mind", Zone::Hand);
        cast(&mut g, aura, None, &[Target::Card(god)]);
        settle(&mut g);
        assert!(!g.spell_has_convoke(p, Card::from_name("Great Forest Druid").unwrap()));
        let creature = named(&mut g, p, "Great Forest Druid", Zone::Hand);
        g.state.players[p.index()].mana = [0; 6];
        assert!(
            !g.legal_actions()
                .contains(&Action::AnnounceCast { card: creature })
        );
        assert!(
            g.spell_has_convoke(p, Card::from_name("Harmonized Crescendo").unwrap()),
            "intrinsic convoke remains"
        );
    }
}
#[test]
fn ecl_transform_isilu_persist_uses_owner_and_minus_counter_prevents_second_return() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        named(&mut g, p, ISILU, Zone::Battlefield);
        let creature = named(&mut g, p.other(), "Hill Giant", Zone::Battlefield);
        g.state.objects[creature as usize].controller = p;
        let old = g.object_ref(creature);
        murder(&mut g, p, creature);
        assert_eq!(g.object_view(creature).zone, Zone::Battlefield);
        assert_ne!(g.object_ref(creature), old);
        assert_eq!(g.object_view(creature).controller, p.other());
        assert_eq!(g.object_view(creature).counters.minus_one, 1);
        assert!(!g.object_view(creature).tapped);
        assert_eq!(
            g.stats(creature),
            Stats {
                power: 2,
                toughness: 2
            }
        );
        g.state.objects[creature as usize].controller = p;
        murder(&mut g, p, creature);
        assert_eq!(g.object_view(creature).zone, Zone::Graveyard);
    }
}
#[test]
fn ecl_transform_isilu_simultaneous_deaths_preserve_grant_and_exclude_self_tokens_opponents() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let god = named(&mut g, p, ISILU, Zone::Battlefield);
        let yours = named(&mut g, p, "Hill Giant", Zone::Battlefield);
        let theirs = named(&mut g, p.other(), "Hill Giant", Zone::Battlefield);
        let token = super::tests::spawn(
            &mut g,
            p,
            Card::from_name("@token/c48e9e904360e4cee1deeed8").unwrap(),
            Zone::Battlefield,
        );
        let wrath = named(&mut g, p, "Day of Judgment", Zone::Hand);
        cast(&mut g, wrath, None, &[]);
        settle(&mut g);
        assert_eq!(g.object_view(yours).zone, Zone::Battlefield);
        assert_eq!(g.object_view(yours).counters.minus_one, 1);
        assert_eq!(g.object_view(god).zone, Zone::Graveyard);
        assert_eq!(g.state.objects[god as usize].card.name(), EIRDU);
        assert_eq!(g.object_view(theirs).zone, Zone::Graveyard);
        assert_eq!(g.object_view(token).zone, Zone::Outside);
    }
}
#[test]
fn ecl_transform_isilu_blanked_recipient_or_grantor_cannot_persist() {
    for blank_grantor in [false, true] {
        let mut g = setup(Player::P0);
        let god = named(&mut g, Player::P0, ISILU, Zone::Battlefield);
        let creature = named(&mut g, Player::P0, "Hill Giant", Zone::Battlefield);
        let aura = named(&mut g, Player::P0, "Noggle the Mind", Zone::Hand);
        cast(
            &mut g,
            aura,
            None,
            &[Target::Card(if blank_grantor { god } else { creature })],
        );
        settle(&mut g);
        murder(&mut g, Player::P0, creature);
        assert_eq!(g.object_view(creature).zone, Zone::Graveyard);
    }
}
#[test]
fn ecl_transform_isilu_persist_departure_lki_retains_minus_before_sba_counter_cancellation() {
    let mut g = setup(Player::P0);
    named(&mut g, Player::P0, ISILU, Zone::Battlefield);
    let creature = named(&mut g, Player::P0, "Hill Giant", Zone::Battlefield);
    g.state.objects[creature as usize].counters.plus_one = 1;
    g.state.objects[creature as usize].counters.minus_one = 1;
    g.state.objects[creature as usize].damage = 10;
    g.state_based_actions();
    settle(&mut g);
    assert_eq!(g.object_view(creature).zone, Zone::Graveyard);
}
#[test]
fn ecl_transform_eirdu_behold_cost_witness_can_also_convoke_before_reveal_payment() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let god = named(&mut g, p, EIRDU, Zone::Battlefield);
        let elf = named(&mut g, p, "Llanowar Elves", Zone::Battlefield);
        let spell = named(&mut g, p, "Lys Alana Dignitary", Zone::Hand);
        g.state.players[p.index()].mana = [0; 6];
        checked(&mut g, Action::AnnounceCast { card: spell });
        checked(&mut g, Action::ChooseSpellCost { cost: 0 });
        let object = g.object_ref(elf);
        checked(&mut g, Action::ChooseAdditionalCost { object });
        checked(&mut g, Action::BeginConvoke);
        for (id, contribution) in [
            (god, ConvokeContribution::Generic),
            (elf, ConvokeContribution::Green),
        ] {
            let object = g.object_ref(id);
            checked(
                &mut g,
                Action::ChooseConvoker {
                    object,
                    contribution,
                },
            );
        }
        checked(&mut g, Action::PayCosts { payment: [0; 6] });
        settle(&mut g);
        assert_eq!(g.object_view(spell).zone, Zone::Battlefield);
        assert!(g.object_view(elf).tapped);
        assert_eq!(g.state.players[p.index()].mana, [0; 6]);
    }
}
