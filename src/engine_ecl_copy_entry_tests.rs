//! Paid Mirrorform scenarios plus exact physical-copy/DFC boundary regressions.
use super::ability_timeline_tests::{cast, checked, named, settle, setup};
use super::*;
fn mirror(g: &mut Game, p: Player, target: CardId) {
    let spell = named(g, p, "Mirrorform", Zone::Hand);
    g.state.players[p.index()].mana = [0, 2, 0, 0, 0, 4];
    cast(g, spell, None, &[Target::Card(target)]);
    settle(g);
    assert_eq!(g.state.objects[spell as usize].zone, Zone::Graveyard);
}
#[test]
fn ecl_copy_entry_mirrorform_paid_copies_every_controlled_nonland_without_entering() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let target = named(&mut g, p.other(), "Serra Angel", Zone::Battlefield);
        let bear = named(&mut g, p, "Bear Cub", Zone::Battlefield);
        let relic = named(&mut g, p, "Springleaf Drum", Zone::Battlefield);
        let land = named(&mut g, p, "Forest", Zone::Battlefield);
        let enemy = named(&mut g, p.other(), "Bear Cub", Zone::Battlefield);
        let before = g.object_ref(bear);
        g.state.objects[bear as usize].tapped = true;
        g.state.objects[bear as usize].damage = 1;
        g.effect_counter(
            bear,
            CounterKind::PlusOne,
            2,
            replacements::ResumeMode::Pending,
        );
        settle(&mut g);
        let hand = g.hand_count(p);
        mirror(&mut g, p, target);
        assert_eq!(g.object_ref(bear), before);
        assert_eq!(g.state.objects[bear as usize].card.name(), "Serra Angel");
        assert_eq!(
            g.stats(bear),
            Stats {
                power: 6,
                toughness: 6
            }
        );
        assert!(g.state.objects[bear as usize].tapped);
        assert_eq!(g.state.objects[bear as usize].damage, 1);
        assert!(
            g.keywords(bear)
                .contains(Keywords::FLYING.union(Keywords::VIGILANCE))
        );
        assert_eq!(g.state.objects[relic as usize].card.name(), "Serra Angel");
        assert_eq!(g.state.objects[land as usize].card.name(), "Forest");
        assert_eq!(g.state.objects[enemy as usize].card.name(), "Bear Cub");
        assert_eq!(g.hand_count(p), hand);
        assert_eq!(g.state.players[p.index()].mana, [0; 6]);
        assert!(g.state.pending_triggers.is_empty());
        assert!(!g.state.static_sources.contains(&relic));
        assert!(!g.state.objects[bear as usize].is_token());
        assert_eq!(
            g.observe(p)
                .copy_origins
                .iter()
                .find(|row| row.object == before)
                .unwrap()
                .native
                .definition
                .name(),
            "Bear Cub"
        );
    }
}
#[test]
fn ecl_copy_entry_mirrorform_target_aura_is_illegal_and_departed_target_fizzles() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let bear = named(&mut g, p, "Bear Cub", Zone::Battlefield);
        let aura = named(&mut g, p, "Gilt-Leaf's Embrace", Zone::Hand);
        cast(&mut g, aura, None, &[Target::Card(bear)]);
        settle(&mut g);
        let spell = named(&mut g, p, "Mirrorform", Zone::Hand);
        checked(&mut g, Action::AnnounceCast { card: spell });
        assert!(!g.legal_actions().contains(&Action::ChooseTarget {
            target: Target::Card(aura)
        }));
        checked(
            &mut g,
            Action::ChooseTarget {
                target: Target::Card(bear),
            },
        );
        while !matches!(g.state.pending, Pending::Priority) {
            let action = g
                .legal_actions()
                .into_iter()
                .find(|a| !matches!(a, Action::Concede))
                .unwrap();
            checked(&mut g, action);
        }
        g.move_object(bear, Zone::Exile);
        settle(&mut g);
        assert!(g.state.copy_recipes.is_none());
        assert_eq!(g.state.objects[spell as usize].zone, Zone::Graveyard);
    }
}
#[test]
fn ecl_copy_entry_mirrorform_land_copies_detach_former_equipment_and_aura_keep_token_identity() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let bear = named(&mut g, p, "Bear Cub", Zone::Battlefield);
        let boots = named(&mut g, p, "Swiftfoot Boots", Zone::Battlefield);
        assert!(g.attach_equipment(g.object_ref(boots), g.object_ref(bear)));
        let aura = named(&mut g, p, "Gilt-Leaf's Embrace", Zone::Hand);
        cast(&mut g, aura, None, &[Target::Card(bear)]);
        settle(&mut g);
        g.stage_copy_creation(p, CopyRecipe::new(Card::from_name("Bear Cub").unwrap()), 1);
        settle(&mut g);
        let token = g
            .state
            .copy_recipes
            .as_ref()
            .unwrap()
            .last()
            .unwrap()
            .object
            .card;
        let target = named(&mut g, p.other(), "Steam Vents", Zone::Battlefield);
        mirror(&mut g, p, target);
        for id in [bear, boots, aura, token] {
            assert_eq!(g.state.objects[id as usize].card.name(), "Steam Vents");
            assert!(g.state.objects[id as usize].attached_to.is_none());
            assert!(!g.state.objects[id as usize].tapped, "copying is not entry");
            assert_eq!(g.current_kinds(id), 2);
        }
        assert!(!g.state.objects[bear as usize].is_token());
        assert!(g.state.objects[token as usize].is_token());
        let save = g.snapshot();
        g.move_object(aura, Zone::Graveyard);
        assert_eq!(
            g.state.objects[aura as usize].card.name(),
            "Gilt-Leaf's Embrace"
        );
        g.move_object(token, Zone::Graveyard);
        assert!(g.state.objects[token as usize].is_token());
        g.state_based_actions();
        assert_eq!(g.state.objects[token as usize].zone, Zone::Outside);
        g.restore(&save).unwrap();
        assert_eq!(g.state.objects[aura as usize].card.name(), "Steam Vents");
    }
}
#[test]
fn ecl_copy_entry_mirrorform_copies_token_exceptions_and_restores_owned_physical_cards_after_repeated_copies()
 {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let source = named(&mut g, p, "Forest", Zone::Battlefield);
        let mut recipe = CopyRecipe::new(Card::from_name("Bear Cub").unwrap());
        recipe.exceptions = CopyExceptions::HASTE_AND_SACRIFICE.union(CopyExceptions::NIGHTMARE);
        g.stage_copy_creation(p.other(), recipe, 1);
        settle(&mut g);
        let target = g.state.copy_recipes.as_ref().unwrap()[0].object.card;
        let body = named(&mut g, p, "Springleaf Drum", Zone::Battlefield);
        mirror(&mut g, p, target);
        assert!(!g.state.objects[body as usize].is_token());
        assert!(g.keywords(body).contains(Keywords::HASTE));
        assert!(g.has_subtype(body, "Nightmare"));
        let copy_recipe = g.copiable_recipe(g.object_ref(body));
        g.stage_copy_creation(p, copy_recipe, 1);
        settle(&mut g);
        let new_token = g
            .state
            .copy_recipes
            .as_ref()
            .unwrap()
            .iter()
            .find(|r| r.object.card != target && r.object.card != body)
            .unwrap()
            .object
            .card;
        assert!(g.state.objects[new_token as usize].is_token());
        let checkpoint = g.snapshot();
        let mut branch = g.clone();
        mirror(&mut g, p, source);
        mirror(&mut branch, p, source);
        assert_eq!(g.snapshot(), branch.snapshot());
        g.move_object(body, Zone::Hand);
        assert_eq!(
            g.state.objects[body as usize].card.name(),
            "Springleaf Drum"
        );
        assert!(!g.state.objects[body as usize].is_token());
        g.restore(&checkpoint).unwrap();
        g.begin_step(Step::End);
        settle(&mut g);
        assert_eq!(g.state.objects[body as usize].zone, Zone::Graveyard);
        assert_eq!(
            g.state.objects[body as usize].card.name(),
            "Springleaf Drum"
        );
    }
}
#[test]
fn ecl_copy_entry_physical_copy_keeps_native_double_face_shape_and_back_face_mana_value() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let dfc = named(&mut g, p, "Brigid, Clachan's Heart", Zone::Battlefield);
        let single = named(&mut g, p, "Bear Cub", Zone::Battlefield);
        // Typed boundary: isolate native transformation and overlay from Brigid's upkeep payment.
        g.transform_permanent(dfc);
        let back = g.state.objects[dfc as usize].card;
        assert_eq!(g.object_mana_value(dfc), 3);
        let snapshot = g.copiable_recipe(g.object_ref(dfc));
        g.copy_permanents_exact(&[g.object_ref(single)], snapshot);
        assert_eq!(
            g.object_mana_value(single),
            0,
            "single face copying a back face has no mana cost"
        );
        g.transform_permanent(single);
        assert_eq!(g.state.objects[single as usize].card, back);
        assert_eq!(
            g.copiable_recipe(g.object_ref(single)).faces(),
            CopyFaces::Single
        );
        let bear = CopyRecipe::new(Card::from_name("Bear Cub").unwrap());
        g.copy_permanents_exact(&[g.object_ref(dfc)], bear);
        g.transform_permanent(dfc);
        assert_eq!(
            g.state.objects[dfc as usize].card.name(),
            "Bear Cub",
            "native transform does not end a copy overlay"
        );
        g.stage_copy_creation(p, g.copiable_recipe(g.object_ref(dfc)), 1);
        settle(&mut g);
        let token = g
            .state
            .copy_recipes
            .as_ref()
            .unwrap()
            .iter()
            .find(|r| r.native.is_none())
            .unwrap()
            .object
            .card;
        assert!(matches!(
            g.copiable_recipe(g.object_ref(token)).faces(),
            CopyFaces::Double { .. }
        ));
        g.transform_permanent(token);
        assert_eq!(g.state.objects[token as usize].card.name(), "Bear Cub");
        g.move_object(dfc, Zone::Exile);
        g.move_object(single, Zone::Graveyard);
        assert_eq!(
            g.state.objects[dfc as usize].card.name(),
            "Brigid, Clachan's Heart"
        );
        assert_eq!(g.state.objects[single as usize].card.name(), "Bear Cub");
    }
}

#[test]
fn ecl_copy_entry_mirrorform_freezes_copiable_values_and_rebuilds_intrinsic_providers() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let target = named(&mut g, p.other(), "Serra Angel", Zone::Battlefield);
        let lord = named(&mut g, p, "Imperious Perfect", Zone::Battlefield);
        let elf = named(&mut g, p, "Llanowar Elves", Zone::Battlefield);
        assert_eq!(g.stats(elf).power, 2);
        let buff = named(&mut g, p, "Giant Growth", Zone::Hand);
        cast(&mut g, buff, None, &[Target::Card(target)]);
        settle(&mut g);
        assert_eq!(g.stats(target).power, 7);
        let hostile = named(&mut g, p.other(), "Noggle the Mind", Zone::Hand);
        g.state.priority = p.other();
        cast(&mut g, hostile, None, &[Target::Card(elf)]);
        settle(&mut g);
        g.state.priority = p;
        let increment = named(&mut g, p, "Giant Growth", Zone::Hand);
        cast(&mut g, increment, None, &[Target::Card(lord)]);
        settle(&mut g);
        mirror(&mut g, p, target);
        assert_eq!(
            g.stats(lord),
            Stats {
                power: 7,
                toughness: 7
            },
            "recipient's external growth persists, target's growth isn't copied"
        );
        assert_eq!(
            g.stats(elf),
            Stats {
                power: 1,
                toughness: 1
            }
        );
        assert!(!g.keywords(elf).contains(Keywords::FLYING));
        assert!(g.has_subtype(elf, "Noggle"));
        assert!(!g.has_subtype(elf, "Angel"));
        assert_eq!(
            g.state.objects[hostile as usize].attached_to,
            Some(g.object_ref(elf))
        );
        assert!(!g.state.static_sources.contains(&lord));
        g.move_object(hostile, Zone::Graveyard);
        assert_eq!(
            g.stats(elf),
            Stats {
                power: 4,
                toughness: 4
            }
        );
        assert!(g.keywords(elf).contains(Keywords::FLYING));
    }
}
#[test]
fn ecl_copy_entry_copiable_changeling_exists_in_layer_four_even_when_its_ability_is_removed() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let mut recipe = CopyRecipe::new(Card::from_name("Bear Cub").unwrap());
        recipe.exceptions = CopyExceptions::CHANGELING;
        g.stage_copy_creation(p.other(), recipe, 1);
        settle(&mut g);
        let target = g.state.copy_recipes.as_ref().unwrap()[0].object.card;
        let body = named(&mut g, p, "Springleaf Drum", Zone::Battlefield);
        mirror(&mut g, p, target);
        assert!(g.has_subtype(body, "Elf"));
        assert!(g.keywords(body).contains(Keywords::CHANGELING));
        g.remove_abilities_exact(
            &[g.object_ref(body)],
            crate::cards::AnimationDuration::Permanent,
        );
        assert!(!g.keywords(body).contains(Keywords::CHANGELING));
        assert!(
            g.has_subtype(body, "Elf"),
            "CDA applied before the later ability layer"
        );
        g.state.players[p.index()].mana = [30; 6];
        let aura = named(&mut g, p, "Noggle the Mind", Zone::Hand);
        cast(&mut g, aura, None, &[Target::Card(body)]);
        settle(&mut g);
        assert!(g.has_subtype(body, "Noggle"));
        assert!(!g.has_subtype(body, "Elf"));
        let recipe = g.copiable_recipe(g.object_ref(body));
        assert!(recipe.exceptions.changeling);
        g.stage_copy_creation(p, recipe, 1);
        settle(&mut g);
        let new = g
            .state
            .copy_recipes
            .as_ref()
            .unwrap()
            .last()
            .unwrap()
            .object
            .card;
        assert!(g.has_subtype(new, "Elf"));
        assert!(g.keywords(new).contains(Keywords::CHANGELING));
    }
}
#[test]
fn ecl_copy_entry_mirrorform_planeswalkers_keep_loyalty_usage_and_do_not_gain_entry_loyalty() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let walker = named(&mut g, p, "Liliana, Dreadhorde General", Zone::Hand);
        cast(&mut g, walker, None, &[]);
        settle(&mut g);
        let action = g
            .legal_actions()
            .into_iter()
            .find(|a| matches!(a, Action::Activate { card, ability: 0, .. } if *card == walker))
            .unwrap();
        checked(&mut g, action);
        settle(&mut g);
        assert_eq!(g.counter_count(walker, CounterKind::Loyalty), 7);
        let target = named(
            &mut g,
            p.other(),
            "Liliana, Dreadhorde General",
            Zone::Battlefield,
        );
        g.state.objects[target as usize].counters.loyalty = 6;
        mirror(&mut g, p, target);
        assert_eq!(g.counter_count(walker, CounterKind::Loyalty), 7);
        assert!(g.on_battlefield(walker));
        assert!(!g.legal_actions().iter().any(|a| matches!(a, Action::Activate {card,..} | Action::AnnounceActivate {card,..} if *card == walker)));
        let bear = named(&mut g, p.other(), "Bear Cub", Zone::Battlefield);
        mirror(&mut g, p, bear);
        assert_eq!(g.counter_count(walker, CounterKind::Loyalty), 7);
        mirror(&mut g, p, target);
        assert!(!g.legal_actions().iter().any(|a| matches!(a, Action::Activate {card,..} | Action::AnnounceActivate {card,..} if *card == walker)), "temporary loss of planeswalker type does not reset loyalty allowance");
        let fresh = named(&mut g, p, "Springleaf Drum", Zone::Battlefield);
        mirror(&mut g, p, target);
        assert_eq!(
            g.state.objects[fresh as usize].zone,
            Zone::Graveyard,
            "copying does not add starting loyalty"
        );
        assert_eq!(
            g.state.objects[fresh as usize].card.name(),
            "Springleaf Drum"
        );
    }
}

#[test]
fn ecl_copy_entry_mirrorform_new_intrinsic_lords_apply_simultaneously_and_new_links_do_not_find_old_pairs()
 {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let lord = named(&mut g, p.other(), "Imperious Perfect", Zone::Battlefield);
        let a = named(&mut g, p, "Springleaf Drum", Zone::Battlefield);
        let b = named(&mut g, p, "Bear Cub", Zone::Battlefield);
        mirror(&mut g, p, lord);
        for id in [a, b] {
            assert_eq!(
                g.stats(id),
                Stats {
                    power: 3,
                    toughness: 3
                }
            );
        }
        assert_eq!(
            g.stats(lord),
            Stats {
                power: 2,
                toughness: 2
            }
        );
        let ward = named(&mut g, p, "Hexing Squelcher", Zone::Battlefield);
        mirror(&mut g, p, ward);
        let times: Vec<_> = g
            .observe(p)
            .ability_source_timestamps
            .into_iter()
            .filter(|row| [a, b, ward].contains(&row.source.card))
            .map(|row| row.timestamp)
            .collect();
        assert_eq!(times.len(), 3);
        assert!(times.iter().all(|&time| time == times[0]));
        let mut g = setup(p);
        let source = named(&mut g, p, "Champion of the Path", Zone::Battlefield);
        // Typed initial linked-cost witness; the actual copy and departure use ordinary paid/public actions.
        let old = named(&mut g, p, "Firdoch Core", Zone::Exile);
        g.remember_linked_exile_for(g.object_ref(source), &[old]);
        mirror(&mut g, p, source);
        assert_ne!(g.copy_ability_origin(g.object_ref(source)), 0);
        g.move_object(source, Zone::Graveyard);
        settle(&mut g);
        assert_eq!(
            g.state.objects[old as usize].zone,
            Zone::Exile,
            "newly acquired linked pair cannot return old pair's exile"
        );
        assert_eq!(
            g.state.objects[source as usize].card.name(),
            "Champion of the Path"
        );
    }
}
#[test]
fn ecl_copy_entry_back_face_mana_purpose_is_frozen_and_empty_copy_batches_do_not_create_effects() {
    let p = Player::P0;
    let mut g = setup(p);
    let native = named(
        &mut g,
        p.other(),
        "Brigid, Clachan's Heart",
        Zone::Battlefield,
    );
    g.transform_permanent(native);
    let single = named(&mut g, p, "Bear Cub", Zone::Battlefield);
    g.copy_permanents_exact(
        &[g.object_ref(single)],
        g.copiable_recipe(g.object_ref(native)),
    );
    let item = g.activation_item(p, single, 0);
    let mana_lots::Purpose::Activation(facts) = g.purpose_for_item(&item) else {
        panic!("activation purpose");
    };
    assert_eq!(facts.mana_value, 0);
    g.move_object(single, Zone::Exile);
    let mana_lots::Purpose::Activation(facts) = g.purpose_for_item(&item) else {
        panic!("activation purpose");
    };
    assert_eq!(facts.mana_value, 0);
    assert_eq!(g.object_mana_value(native), 3);
    let saved = g.snapshot();
    g.copy_permanents_exact(&[], CopyRecipe::new(Card::Forest));
    assert_eq!(g.snapshot(), saved);
}
fn typed_permanent_spell_copy(
    g: &mut Game,
    p: Player,
    name: &str,
    target: Option<Target>,
    evoked: bool,
) -> CardId {
    // Typed allocator boundary; Raiding's real conspire consumer is authored in the integrator's worktree.
    let id = named(g, p, name, Zone::Hand);
    let mut item = g.cast_item(p, id, [0; 6]);
    g.state.objects[id as usize].copy_kind = CopyKind::Spell;
    item.is_copy = true;
    item.cast.as_mut().unwrap().evoked = evoked;
    if let Some(target) = target {
        item.targets.push(g.bind_target(target));
    }
    g.state.stack.push(item);
    id
}
#[test]
fn ecl_copy_entry_permanent_spell_copies_enter_tokens_without_consuming_token_creation_replacements()
 {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let bear = named(&mut g, p, "Bear Cub", Zone::Battlefield);
        let crown = named(&mut g, p, "Mirrormind Crown", Zone::Battlefield);
        assert!(g.attach_equipment(g.object_ref(crown), g.object_ref(bear)));
        let hand = g.hand_count(p);
        let copy = typed_permanent_spell_copy(&mut g, p, "Inspiring Overseer", None, false);
        settle(&mut g);
        assert!(g.on_battlefield(copy));
        assert!(g.state.objects[copy as usize].is_token());
        assert_eq!(
            g.state.objects[copy as usize].card.name(),
            "Inspiring Overseer"
        );
        assert_eq!(g.hand_count(p), hand + 1);
        assert!(!g.state.players[p.index()].token_creation_attempted_this_turn);
        assert_eq!(
            g.copiable_recipe(g.object_ref(copy)).definition.name(),
            "Inspiring Overseer"
        );
        g.move_object(copy, Zone::Hand);
        g.state_based_actions();
        assert_eq!(g.state.objects[copy as usize].zone, Zone::Outside);
        assert!(g.state.objects[copy as usize].is_token());
        assert!(g.copy_recipe(g.object_ref(copy)).is_none());
    }
}
#[test]
fn ecl_copy_entry_aura_spell_copy_attaches_normally_and_illegal_copy_ceases_without_entry() {
    for p in [Player::P0, Player::P1] {
        for legal in [false, true] {
            let mut g = setup(p);
            let bear = named(&mut g, p, "Bear Cub", Zone::Battlefield);
            let copy = typed_permanent_spell_copy(
                &mut g,
                p,
                "Gilt-Leaf's Embrace",
                Some(Target::Card(bear)),
                false,
            );
            if !legal {
                g.move_object(bear, Zone::Exile);
            }
            settle(&mut g);
            if legal {
                assert!(g.on_battlefield(copy));
                assert!(g.state.objects[copy as usize].is_token());
                assert_eq!(
                    g.state.objects[copy as usize].attached_to,
                    Some(g.object_ref(bear))
                );
                assert_eq!(
                    g.stats(bear),
                    Stats {
                        power: 4,
                        toughness: 2
                    }
                );
                g.move_object(copy, Zone::Graveyard);
                g.state_based_actions();
                assert_eq!(
                    g.stats(bear),
                    Stats {
                        power: 2,
                        toughness: 2
                    }
                );
            }
            assert_eq!(g.state.objects[copy as usize].zone, Zone::Outside);
            assert!(!g.state.players[p.index()].token_creation_attempted_this_turn);
        }
    }
}
#[test]
fn ecl_copy_entry_permanent_spell_copy_inherits_evoke_decision_but_not_mana_spent_or_cast_entry() {
    for p in [Player::P0, Player::P1] {
        for evoked in [false, true] {
            let mut g = setup(p);
            let hand = g.hand_count(p);
            let copy = typed_permanent_spell_copy(&mut g, p, "Wistfulness", None, evoked);
            settle(&mut g);
            assert_eq!(
                g.hand_count(p),
                hand,
                "zero spent mana does not copy original payment effects"
            );
            assert_eq!(
                g.state.objects[copy as usize].zone,
                if evoked {
                    Zone::Outside
                } else {
                    Zone::Battlefield
                }
            );
            assert!(g.state.objects[copy as usize].is_token());
        }
        let mut g = setup(p);
        let bear = named(&mut g, p, "Bear Cub", Zone::Battlefield);
        let copy = typed_permanent_spell_copy(&mut g, p, "Sunderflock", None, false);
        settle(&mut g);
        assert!(g.on_battlefield(copy));
        assert!(
            g.on_battlefield(bear),
            "a permanent spell copy was never cast"
        );
    }
}
#[test]
fn ecl_copy_entry_double_faced_and_planeswalker_spell_copies_get_real_entry_characteristics() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let copy =
            typed_permanent_spell_copy(&mut g, p, "Liliana, Dreadhorde General", None, false);
        settle(&mut g);
        assert!(g.on_battlefield(copy));
        assert_eq!(g.counter_count(copy, CounterKind::Loyalty), 6);
        assert!(g.state.objects[copy as usize].is_token());
        let mut g = setup(p);
        let copy = typed_permanent_spell_copy(&mut g, p, "Brigid, Clachan's Heart", None, false);
        settle(&mut g);
        assert!(g.on_battlefield(copy));
        assert!(matches!(
            g.copiable_recipe(g.object_ref(copy)).faces(),
            CopyFaces::Double { .. }
        ));
        assert_eq!(g.object_mana_value(copy), 3);
        g.transform_permanent(copy);
        settle(&mut g);
        assert_eq!(
            g.state.objects[copy as usize].card.name(),
            "Brigid, Doun's Mind"
        );
        assert_eq!(g.object_mana_value(copy), 3);
        assert!(g.state.objects[copy as usize].is_token());
    }
}
#[test]
fn ecl_copy_entry_mirrorform_does_not_copy_entry_choices_or_reuse_a_previous_linked_choice() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let own = named(&mut g, p, "Adaptive Automaton", Zone::Hand);
        cast(&mut g, own, None, &[]);
        checked(&mut g, Action::Pass);
        checked(&mut g, Action::Pass);
        let elf = crate::CreatureType::from_name("Elf").unwrap();
        checked(
            &mut g,
            Action::ChooseEntryValue {
                value: EntryChoiceValue::CreatureType(elf),
            },
        );
        settle(&mut g);
        let friend = named(&mut g, p, "Llanowar Elves", Zone::Battlefield);
        assert_eq!(g.stats(friend).power, 2);
        g.state.active = p.other();
        g.state.priority = p.other();
        let target = named(&mut g, p.other(), "Adaptive Automaton", Zone::Hand);
        cast(&mut g, target, None, &[]);
        checked(&mut g, Action::Pass);
        checked(&mut g, Action::Pass);
        let bear = crate::CreatureType::from_name("Bear").unwrap();
        checked(
            &mut g,
            Action::ChooseEntryValue {
                value: EntryChoiceValue::CreatureType(bear),
            },
        );
        settle(&mut g);
        g.state.active = p;
        g.state.priority = p;
        mirror(&mut g, p, target);
        for id in [own, friend] {
            assert_eq!(g.source_chosen_creature_type(g.object_ref(id)), None);
            assert!(g.has_subtype(id, "Construct"));
            assert!(!g.has_subtype(id, "Elf"));
            assert!(!g.has_subtype(id, "Bear"));
            assert_eq!(
                g.stats(id),
                Stats {
                    power: 2,
                    toughness: 2
                }
            );
        }
        assert_eq!(
            g.source_chosen_creature_type(g.object_ref(target)),
            Some(bear)
        );
    }
}
#[test]
fn ecl_copy_entry_mirrorform_realms_without_a_type_produces_frozen_unspendable_colored_mana() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let body = named(&mut g, p, "Springleaf Drum", Zone::Battlefield);
        let target = named(&mut g, p.other(), "Eclipsed Realms", Zone::Hand);
        g.state.active = p.other();
        g.state.priority = p.other();
        checked(&mut g, Action::PlayLand { card: target });
        checked(
            &mut g,
            Action::ChooseEntryValue {
                value: EntryChoiceValue::CreatureType(
                    crate::CreatureType::from_name("Elf").unwrap(),
                ),
            },
        );
        settle(&mut g);
        g.state.active = p;
        g.state.priority = p;
        mirror(&mut g, p, target);
        assert_eq!(g.source_chosen_creature_type(g.object_ref(body)), None);
        let green = [0, 0, 0, 0, 1, 0];
        let action = g.legal_actions().into_iter().find(|a| matches!(a,
            Action::ActivateMana { card, ability: 1, production, .. } if *card == body && *production == green)).unwrap();
        checked(&mut g, action);
        assert_eq!(g.state.players[p.index()].mana, green);
        assert_eq!(g.available_mana(p, mana_lots::Purpose::Other), [0; 6]);
        assert_eq!(
            g.observe(p).mana_lots[0].restriction,
            ManaRestrictionView::Unspendable
        );
        let elf = named(&mut g, p, "Llanowar Elves", Zone::Hand);
        assert!(
            !g.legal_actions()
                .contains(&Action::AnnounceCast { card: elf })
        );
        g.move_object(body, Zone::Hand);
        assert_eq!(
            g.state.objects[body as usize].card.name(),
            "Springleaf Drum"
        );
        assert_eq!(g.available_mana(p, mana_lots::Purpose::Other), [0; 6]);
        assert!(
            !g.legal_actions()
                .contains(&Action::AnnounceCast { card: elf })
        );
    }
}
