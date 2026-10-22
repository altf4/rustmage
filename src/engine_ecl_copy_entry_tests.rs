//! Paid Mirrorform scenarios plus exact physical-copy/DFC boundary regressions.
use super::ability_timeline_tests::{cast, checked, named, settle, setup};
use super::*;
use crate::cards::CreatureType;
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

// Candidate spawning allows the complete body to be tested before catalogue admission.
fn omni(g: &mut Game, p: Player, zone: Zone) -> CardId {
    super::tests::spawn(g, p, Card::from_name("Omni-Changeling").unwrap(), zone)
}
fn to_omni_copy(g: &mut Game) {
    for _ in 0..30 {
        if matches!(g.state.pending, Pending::EntryCopy { .. }) {
            return;
        }
        let action = g
            .legal_actions()
            .into_iter()
            .find(|a| !matches!(a, Action::Concede))
            .unwrap();
        checked(g, action);
    }
    panic!("missing Omni copy choice: {:?}", g.state.pending);
}
fn paid_omni(g: &mut Game, p: Player) -> CardId {
    let id = omni(g, p, Zone::Hand);
    g.state.players[p.index()].mana = [0, 2, 0, 0, 0, 3];
    cast(g, id, None, &[]);
    to_omni_copy(g);
    id
}
fn select_omni(g: &mut Game, target: Option<CardId>) {
    let object = target.map(|id| g.object_ref(id));
    checked(g, Action::ChooseEntryCopy { object });
}
#[test]
fn ecl_copy_entry_omni_paid_opposing_hexproof_creature_is_nontargeted_copy_and_restores_native_card()
 {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let target = named(&mut g, p.other(), "Serra Angel", Zone::Battlefield);
        let boots = named(&mut g, p.other(), "Swiftfoot Boots", Zone::Battlefield);
        assert!(g.attach_equipment(g.object_ref(boots), g.object_ref(target)));
        g.state.objects[target as usize].tapped = true;
        g.state.objects[target as usize].damage = 1;
        g.effect_counter(
            target,
            CounterKind::PlusOne,
            2,
            replacements::ResumeMode::Pending,
        );
        settle(&mut g);
        let id = paid_omni(&mut g, p);
        assert!(
            g.observe(p)
                .entry_copy
                .as_ref()
                .unwrap()
                .candidates
                .contains(&g.object_ref(target))
        );
        select_omni(&mut g, Some(target));
        settle(&mut g);
        assert_eq!(g.state.players[p.index()].mana, [0; 6]);
        assert_eq!(g.state.objects[id as usize].card.name(), "Serra Angel");
        assert_eq!(
            g.stats(id),
            Stats {
                power: 4,
                toughness: 4
            }
        );
        assert!(!g.state.objects[id as usize].tapped);
        assert_eq!(g.state.objects[id as usize].damage, 0);
        assert!(!g.counter_amounts(id).has_any());
        assert!(
            g.keywords(id).contains(
                Keywords::FLYING
                    .union(Keywords::VIGILANCE)
                    .union(Keywords::CHANGELING)
            )
        );
        assert!(!g.keywords(id).contains(Keywords::HEXPROOF));
        assert!(!g.state.objects[id as usize].card.has_convoke());
        assert!(g.has_subtype(id, "Elf") && g.has_subtype(id, "Elemental"));
        g.move_object(id, Zone::Graveyard);
        assert_eq!(g.state.objects[id as usize].card.name(), "Omni-Changeling");
        assert!(!g.state.objects[id as usize].is_token());
    }
}
#[test]
fn ecl_copy_entry_omni_decline_and_repeated_new_copy_replacements() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let id = paid_omni(&mut g, p);
        select_omni(&mut g, None);
        settle(&mut g);
        assert_eq!(g.state.objects[id as usize].zone, Zone::Graveyard);
        let lord = named(&mut g, p, "Imperious Perfect", Zone::Battlefield);
        let original = paid_omni(&mut g, p);
        select_omni(&mut g, None);
        settle(&mut g);
        assert_eq!(
            g.stats(original),
            Stats {
                power: 1,
                toughness: 1
            }
        );
        let copied = paid_omni(&mut g, p);
        for _ in 0..3 {
            select_omni(&mut g, Some(original));
            assert!(matches!(g.state.pending, Pending::EntryCopy { .. }));
            assert_eq!(g.state.objects[copied as usize].zone, Zone::Stack);
        }
        select_omni(&mut g, None);
        settle(&mut g);
        assert_eq!(
            g.stats(copied),
            Stats {
                power: 1,
                toughness: 1
            }
        );
        assert!(g.state.objects[copied as usize].card.has_convoke());
        assert!(g.on_battlefield(lord));
        assert!(
            g.copy_recipe(g.object_ref(copied))
                .unwrap()
                .exceptions
                .changeling
        );
    }
}
#[test]
fn ecl_copy_entry_omni_copied_etb_and_fresh_chosen_type_apply_after_copy() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let target = named(&mut g, p.other(), "Inspiring Overseer", Zone::Battlefield);
        let life = g.state.players[p.index()].life;
        let hand = g.hand_count(p);
        let copied = paid_omni(&mut g, p);
        select_omni(&mut g, Some(target));
        settle(&mut g);
        assert_eq!(g.state.players[p.index()].life, life + 1);
        assert_eq!(g.hand_count(p), hand + 1);
        assert_eq!(
            g.stats(copied),
            Stats {
                power: 2,
                toughness: 1
            }
        );
        let target = named(&mut g, p.other(), "Adaptive Automaton", Zone::Battlefield);
        let old = CreatureType::from_name("Elf").unwrap();
        g.set_entry_value(g.object_ref(target), EntryChoiceValue::CreatureType(old));
        let bear = named(&mut g, p, "Bear Cub", Zone::Battlefield);
        let copied = paid_omni(&mut g, p);
        select_omni(&mut g, Some(target));
        assert!(matches!(g.state.pending, Pending::EntryValue { .. }));
        let chosen = CreatureType::from_name("Bear").unwrap();
        checked(
            &mut g,
            Action::ChooseEntryValue {
                value: EntryChoiceValue::CreatureType(chosen),
            },
        );
        settle(&mut g);
        assert_eq!(
            g.entry_value(g.object_ref(copied)),
            Some(EntryChoiceValue::CreatureType(chosen))
        );
        assert_eq!(
            g.stats(bear),
            Stats {
                power: 3,
                toughness: 3
            }
        );
    }
}

#[test]
fn ecl_copy_entry_omni_convoke_uses_summoning_sick_creatures_and_copied_entry_counters_are_new() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let target = named(&mut g, p.other(), "Serra Angel", Zone::Battlefield);
        let mut convokers = Vec::new();
        for name in [
            "Spectral Sailor",
            "Spectral Sailor",
            "Bear Cub",
            "Bear Cub",
            "Bear Cub",
        ] {
            let id = named(&mut g, p, name, Zone::Battlefield);
            g.state.objects[id as usize].summoning_sick = true;
            convokers.push(id);
        }
        let id = omni(&mut g, p, Zone::Hand);
        g.state.players[p.index()].mana = [0; 6];
        checked(&mut g, Action::AnnounceCast { card: id });
        checked(&mut g, Action::BeginConvoke);
        for (index, &card) in convokers.iter().enumerate() {
            let object = g.object_ref(card);
            checked(
                &mut g,
                Action::ChooseConvoker {
                    object,
                    contribution: if index < 2 {
                        ConvokeContribution::Blue
                    } else {
                        ConvokeContribution::Generic
                    },
                },
            );
        }
        let pay = g
            .legal_actions()
            .into_iter()
            .find(|a| matches!(a, Action::PayCosts { .. }))
            .unwrap();
        checked(&mut g, pay);
        to_omni_copy(&mut g);
        select_omni(&mut g, Some(target));
        settle(&mut g);
        assert!(
            convokers
                .iter()
                .all(|&id| g.state.objects[id as usize].tapped)
        );
        assert_eq!(
            g.stats(id),
            Stats {
                power: 4,
                toughness: 4
            }
        );
        assert_eq!(g.state.players[p.index()].mana, [0; 6]);
        let target = named(&mut g, p.other(), "Heroes' Bane", Zone::Battlefield);
        g.state.objects[target as usize].counters.plus_one = 9;
        named(&mut g, p, "Doubling Season", Zone::Battlefield);
        let id = paid_omni(&mut g, p);
        select_omni(&mut g, Some(target));
        settle(&mut g);
        assert_eq!(g.counter_amounts(id).get(CounterKind::PlusOne), 8);
        assert_eq!(
            g.stats(id),
            Stats {
                power: 8,
                toughness: 8
            }
        );
    }
}
#[test]
fn ecl_copy_entry_ability_resolution_counts_keep_old_packets_separate_from_new_copied_ability() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let source = named(&mut g, p, "Soulbright Seeker", Zone::Battlefield);
        let target = named(&mut g, p.other(), "Soulbright Seeker", Zone::Battlefield);
        let activation = |g: &mut Game| {
            checked(
                g,
                Action::Activate {
                    card: source,
                    ability: 0,
                    target: Some(Target::Card(source)),
                    payment: [0, 0, 0, 1, 0, 0],
                    counter_payment: Counters::default(),
                },
            );
        };
        for _ in 0..2 {
            activation(&mut g);
            settle(&mut g);
        }
        activation(&mut g); // Old third resolution waits below the instant copy effect.
        let spell = named(&mut g, p, "Mirrorform", Zone::Hand);
        g.state.players[p.index()].mana = [0, 2, 0, 0, 0, 4];
        cast(&mut g, spell, None, &[Target::Card(target)]);
        while g.state.objects[spell as usize].zone == Zone::Stack {
            checked(&mut g, Action::Pass);
        }
        let epoch = g.copy_ability_origin(g.object_ref(source));
        assert_ne!(epoch, 0);
        assert_eq!(g.state.stack.len(), 1);
        g.state.players[p.index()].mana = [0, 0, 0, 10, 0, 0];
        activation(&mut g);
        settle(&mut g);
        assert_eq!(
            g.state.players[p.index()].mana[3],
            13,
            "old third pays bonus, new first does not"
        );
        let rows = g.observe(p).ability_resolutions;
        assert_eq!(rows.len(), 2);
        assert!(rows.iter().any(|r| r.ability_origin == 0 && r.count == 3));
        assert!(
            rows.iter()
                .any(|r| r.ability_origin == epoch && r.count == 1)
        );
        activation(&mut g);
        settle(&mut g);
        assert_eq!(g.state.players[p.index()].mana[3], 12);
        activation(&mut g);
        settle(&mut g);
        assert_eq!(g.state.players[p.index()].mana[3], 15);
    }
}
#[test]
fn ecl_copy_entry_omni_cast_qualification_distinguishes_paid_free_and_token_spell_copy() {
    for p in [Player::P0, Player::P1] {
        for method in 0..3 {
            let mut g = setup(p);
            let target = named(&mut g, p.other(), "Sunderflock", Zone::Battlefield);
            let bear = named(&mut g, p, "Bear Cub", Zone::Battlefield);
            let id = if method == 0 {
                paid_omni(&mut g, p)
            } else {
                let id = omni(
                    &mut g,
                    p,
                    if method == 1 { Zone::Exile } else { Zone::Hand },
                );
                if method == 1 {
                    // Existing typed permission boundary; the actual free cast is public/legal.
                    g.grant_exile_free_cast_permission(p, g.object_ref(id));
                    g.state.players[p.index()].mana = [0; 6];
                    checked(
                        &mut g,
                        Action::AnnounceFree {
                            card: id,
                            face: CastFace::Normal,
                            flash_surcharge: false,
                        },
                    );
                    while g.state.announcing.is_some() {
                        let pay = g
                            .legal_actions()
                            .into_iter()
                            .find(|a| matches!(a, Action::PayCosts { .. }))
                            .unwrap();
                        checked(&mut g, pay);
                    }
                } else {
                    // Existing typed stack-copy allocator boundary; no cast event occurs.
                    let mut item = g.cast_item(p, id, [0; 6]);
                    item.is_copy = true;
                    g.state.objects[id as usize].copy_kind = CopyKind::Spell;
                    g.state.stack.push(item);
                }
                to_omni_copy(&mut g);
                id
            };
            select_omni(&mut g, Some(target));
            settle(&mut g);
            assert!(g.on_battlefield(id));
            assert!(g.has_subtype(id, "Elemental"));
            assert_eq!(
                g.on_battlefield(bear),
                method == 2,
                "only actual casts acquire cast-qualified ETB"
            );
            assert_eq!(g.state.objects[id as usize].is_token(), method == 2);
        }
    }
}
fn animate_copy_entry_boundary(g: &mut Game, id: CardId) {
    let item = g.activation_item(g.state.objects[id as usize].controller, id, 0);
    g.animate_source(
        &item,
        crate::cards::AnimationSpec {
            artifact: false,
            creature_types: crate::cards::AnimationCreatureTypes::Unchanged,
            base: Some((4, 4)),
            keywords: Keywords::NONE,
            duration: crate::cards::AnimationDuration::Permanent,
        },
    );
}
#[test]
fn ecl_copy_entry_omni_copies_animated_land_base_and_applies_its_fresh_life_choice() {
    for p in [Player::P0, Player::P1] {
        for paid in [false, true] {
            let mut g = setup(p);
            let target = named(&mut g, p.other(), "Steam Vents", Zone::Battlefield);
            // Typed animation boundary; the actual copy and entry choices remain public actions.
            animate_copy_entry_boundary(&mut g, target);
            let life = g.state.players[p.index()].life;
            let id = paid_omni(&mut g, p);
            select_omni(&mut g, Some(target));
            assert!(matches!(g.state.pending, Pending::EntryLife { .. }));
            checked(&mut g, Action::ChooseEntryLife { pay: paid });
            settle(&mut g);
            assert_eq!(g.state.objects[id as usize].card.name(), "Steam Vents");
            assert_eq!(g.current_kinds(id), 2);
            assert!(
                !g.has_subtype(id, "Elf"),
                "Changeling does not give noncreature lands creature types"
            );
            assert_eq!(g.state.objects[id as usize].tapped, !paid);
            assert_eq!(
                g.state.players[p.index()].life,
                life - if paid { 2 } else { 0 }
            );
            assert!(g.keywords(id).contains(Keywords::CHANGELING));
            g.move_object(id, Zone::Hand);
            assert_eq!(g.state.objects[id as usize].card.name(), "Omni-Changeling");
        }
    }
}
#[test]
fn ecl_copy_entry_omni_private_simultaneous_choices_are_owned_apnap_and_commit_together() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let target = named(&mut g, p, "Serra Angel", Zone::Battlefield);
        let land = named(&mut g, p, "Forest", Zone::Battlefield);
        let first = omni(&mut g, p, Zone::Hand);
        let second = omni(&mut g, p.other(), Zone::Hand);
        // Typed simultaneous hand-entry boundary; each raw public choice verifies owned branching.
        let moves = vec![
            g.entry_move(second, Zone::Battlefield, false),
            g.entry_move(first, Zone::Battlefield, false),
        ];
        g.stage_entries(moves, Some(p));
        assert!(g.advance_entries());
        assert_eq!(g.actor(), Some(p));
        assert!(g.observe(p.other()).entry_copy.is_none());
        let before = g.snapshot();
        let rev = g.revision();
        assert_eq!(
            g.apply(
                p,
                Action::ChooseEntryCopy {
                    object: Some(g.object_ref(land))
                }
            ),
            Err(Error::IllegalAction)
        );
        assert_eq!(g.snapshot(), before);
        assert_eq!(g.revision(), rev);
        select_omni(&mut g, Some(target));
        assert_eq!(g.actor(), Some(p.other()));
        assert_eq!(g.state.objects[first as usize].zone, Zone::Hand);
        assert_eq!(g.state.objects[second as usize].zone, Zone::Hand);
        assert!(g.observe(p).entry_copy.is_none());
        select_omni(&mut g, Some(target));
        settle(&mut g);
        assert!(g.on_battlefield(first) && g.on_battlefield(second));
        for id in [first, second] {
            assert_eq!(
                g.stats(id),
                Stats {
                    power: 4,
                    toughness: 4
                }
            );
        }
    }
}
#[test]
fn ecl_copy_entry_omni_copied_animated_aura_rechooses_attachment_or_fails_as_native_card() {
    for p in [Player::P0, Player::P1] {
        for legal in [false, true] {
            let mut g = setup(p);
            let enemy = named(&mut g, p.other(), "Bear Cub", Zone::Battlefield);
            let target = named(&mut g, p.other(), "Aquitect's Defenses", Zone::Battlefield);
            g.state.objects[target as usize].attached_to = Some(g.object_ref(enemy));
            g.retimestamp_live_static_source(g.object_ref(target));
            let recipient = legal.then(|| named(&mut g, p, "Bear Cub", Zone::Battlefield));
            let id = paid_omni(&mut g, p);
            // Typed mid-resolution animation boundary; SBAs wait until this whole entry finishes.
            animate_copy_entry_boundary(&mut g, target);
            select_omni(&mut g, Some(target));
            if let Some(recipient) = recipient {
                assert!(matches!(g.state.pending, Pending::AuraEntry { .. }));
                // A frozen prototype survives a typed source-departure boundary.
                g.move_object(target, Zone::Graveyard);
                let target = g.object_ref(recipient);
                checked(&mut g, Action::ChooseAuraAttachment { object: target });
                settle(&mut g);
                assert!(g.on_battlefield(id));
                assert_eq!(g.state.objects[id as usize].attached_to, Some(target));
                assert_eq!(
                    g.stats(recipient),
                    Stats {
                        power: 3,
                        toughness: 4
                    }
                );
                assert!(g.keywords(recipient).contains(Keywords::HEXPROOF));
                assert!(!g.is_creature(id));
                assert!(g.keywords(id).contains(Keywords::CHANGELING));
                g.move_object(id, Zone::Graveyard);
            } else {
                settle(&mut g);
                assert_eq!(
                    g.state.objects[target as usize].zone,
                    Zone::Graveyard,
                    "creature Auras cannot enchant"
                );
            }
            assert_eq!(g.state.objects[id as usize].zone, Zone::Graveyard);
            assert_eq!(g.state.objects[id as usize].card.name(), "Omni-Changeling");
        }
    }
}
#[test]
fn ecl_copy_entry_omni_token_entry_keeps_replacement_completion_and_uses_copied_etb() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let target = named(&mut g, p.other(), "Inspiring Overseer", Zone::Battlefield);
        let life = g.state.players[p.index()].life;
        let hand = g.hand_count(p);
        // Typed token-copy creation consumer enters the ordinary replacement continuation.
        g.stage_copy_creation(
            p,
            CopyRecipe::new(Card::from_name("Omni-Changeling").unwrap()),
            1,
        );
        to_omni_copy(&mut g);
        let reference = g.observe(p).entry_copy.unwrap().object;
        select_omni(&mut g, Some(target));
        settle(&mut g);
        assert!(g.state.replacements.is_none() && g.state.entry_batch.is_none());
        assert!(g.on_battlefield(reference.card));
        assert!(g.state.objects[reference.card as usize].is_token());
        assert_eq!(g.state.players[p.index()].life, life + 1);
        assert_eq!(g.hand_count(p), hand + 1);
        assert!(g.state.players[p.index()].token_creation_attempted_this_turn);
        g.move_object(reference.card, Zone::Graveyard);
        g.state_based_actions();
        assert_eq!(g.state.objects[reference.card as usize].zone, Zone::Outside);
    }
}
#[test]
fn ecl_copy_entry_omni_native_single_face_shape_remains_single_when_copying_a_transformed_creature()
{
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let target = named(
            &mut g,
            p.other(),
            "Brigid, Clachan's Heart",
            Zone::Battlefield,
        );
        // Typed native-transform boundary; the incoming Omni still uses an actual paid cast.
        g.transform_permanent(target);
        let back = g.state.objects[target as usize].card;
        let id = paid_omni(&mut g, p);
        select_omni(&mut g, Some(target));
        settle(&mut g);
        assert_eq!(g.state.objects[id as usize].card, back);
        assert_eq!(g.object_mana_value(id), 0);
        assert_eq!(
            g.copiable_recipe(g.object_ref(id)).faces(),
            CopyFaces::Single
        );
        g.transform_permanent(id);
        assert_eq!(g.state.objects[id as usize].card, back);
        g.stage_copy_creation(p, g.copiable_recipe(g.object_ref(id)), 1);
        settle(&mut g);
        let token = g
            .state
            .copy_recipes
            .as_ref()
            .unwrap()
            .iter()
            .find(|row| row.native.is_none() && row.recipe.definition == back)
            .unwrap()
            .object
            .card;
        assert_eq!(
            g.copiable_recipe(g.object_ref(token)).faces(),
            CopyFaces::Single
        );
        g.transform_permanent(token);
        assert_eq!(g.state.objects[token as usize].card, back);
        g.move_object(id, Zone::Graveyard);
        assert_eq!(g.state.objects[id as usize].card.name(), "Omni-Changeling");
        assert_eq!(g.state.objects[id as usize].card.fixed_mana_value(), 5);
    }
}
#[test]
fn ecl_copy_entry_omni_kindles_initial_copy_exceptions_are_overwritten_by_new_entry_copy() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        named(&mut g, p, "Imperious Perfect", Zone::Battlefield);
        let source = paid_omni(&mut g, p);
        select_omni(&mut g, None);
        settle(&mut g);
        let target = named(&mut g, p.other(), "Inspiring Overseer", Zone::Battlefield);
        let spell = named(&mut g, p, "Kindle the Inner Flame", Zone::Hand);
        g.state.players[p.index()].mana = [0, 0, 0, 1, 0, 3];
        cast(&mut g, spell, None, &[Target::Card(source)]);
        to_omni_copy(&mut g);
        let token = g.observe(p).entry_copy.unwrap().object.card;
        select_omni(&mut g, Some(target));
        settle(&mut g);
        assert_eq!(g.state.objects[spell as usize].zone, Zone::Graveyard);
        assert!(g.state.objects[token as usize].is_token());
        assert!(!g.keywords(token).contains(Keywords::HASTE));
        let exceptions = g.copiable_recipe(g.object_ref(token)).exceptions;
        assert!(exceptions.changeling);
        assert_eq!(exceptions.sacrifice_abilities, 0);
        super::ability_timeline_tests::next_turn(&mut g);
        assert!(
            g.on_battlefield(token),
            "overwritten sacrifice ability cannot trigger at end step"
        );
    }
}
