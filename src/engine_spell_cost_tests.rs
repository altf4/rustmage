use super::tests::{act, cast, scenario, spawn};
use super::*;
fn named(g: &mut Game, p: Player, name: &str, zone: Zone) -> CardId {
    let c = Card::from_name(name).unwrap();
    assert!(c.is_supported(), "{name}");
    spawn(g, p, c, zone)
}
fn replay(g: &mut Game, a: Action) {
    let saved = g.snapshot();
    let rev = g.revision();
    let actor = g.actor().unwrap();
    let mut fork = g.clone();
    act(&mut fork, a.clone());
    act(g, a.clone());
    let after = g.snapshot();
    assert_eq!(fork.snapshot(), after);
    g.restore(&saved).unwrap();
    assert_eq!(g.apply_index(actor, rev, 0), Err(Error::StaleRevision));
    act(g, a);
    assert_eq!(after, g.snapshot());
}
fn passes(g: &mut Game) {
    replay(g, Action::Pass);
    replay(g, Action::Pass);
}
fn reject(g: &mut Game, a: Action) {
    let saved = g.snapshot();
    let rev = g.revision();
    assert_eq!(g.apply(g.actor().unwrap(), a), Err(Error::IllegalAction));
    assert_eq!(saved, g.snapshot());
    assert_eq!(rev, g.revision());
}
fn base(g: &Game, id: CardId) -> ManaCost {
    let (colored, generic) = g.state.objects[id as usize].card.cost();
    ManaCost { colored, generic }
}
fn generic(g: &Game, id: CardId) -> u32 {
    g.reduced_spell_cost(g.state.objects[id as usize].owner, id, base(g, id))
        .generic
}
fn triggered(g: &mut Game, target: Option<Target>) {
    g.prepare_triggers();
    for _ in 0..20 {
        if g.state.pending == Pending::Priority {
            return;
        }
        let a = g
            .legal_actions()
            .into_iter()
            .find(|a| {
                matches!(a, Action::PutTrigger { .. } | Action::FinishTargets)
                    || target.is_some_and(|t| *a == Action::ChooseTarget { target: t })
            })
            .unwrap();
        replay(g, a);
    }
    panic!("trigger did not finish");
}
fn pay(g: &mut Game) {
    let a = g
        .legal_actions()
        .into_iter()
        .find(|a| matches!(a, Action::PayCosts { .. }))
        .unwrap();
    replay(g, a);
}

#[test]
fn each_global_reducer_matches_only_its_filter_and_controller() {
    for (source, matching, other) in [
        ("Dragonlord's Servant", "Shivan Dragon", "Bear Cub"),
        ("Voyager Quickwelder", "Goldvein Pick", "Bear Cub"),
        ("Geist of Saint Thalia", "Divination", "Bear Cub"),
        ("Mocking Sprite", "Divination", "Bear Cub"),
        ("Archmage of Runes", "Divination", "Bear Cub"),
        ("Stormcatch Mentor", "Divination", "Bear Cub"),
        ("Honest Rutstein", "Bear Cub", "Divination"),
        ("Ballyrush Banneret", "Oltec Cloud Guard", "Bear Cub"),
    ] {
        for owner in [Player::P0, Player::P1] {
            let mut g = scenario();
            named(&mut g, owner, source, Zone::Battlefield);
            let yes = named(&mut g, Player::P0, matching, Zone::Hand);
            let no = named(&mut g, Player::P0, other, Zone::Hand);
            assert_eq!(
                generic(&g, yes),
                base(&g, yes).generic - u32::from(owner == Player::P0),
                "{source}"
            );
            assert_eq!(generic(&g, no), base(&g, no).generic);
            assert_eq!(
                g.reduced_spell_cost(Player::P0, yes, base(&g, yes)).colored,
                base(&g, yes).colored
            );
        }
    }
}
#[test]
fn tribal_union_applies_once_to_changeling_that_has_both_subtypes() {
    let mut g = scenario();
    named(&mut g, Player::P0, "Ballyrush Banneret", Zone::Battlefield);
    let spell = named(&mut g, Player::P0, "Three Tree Mascot", Zone::Hand);
    assert_eq!(generic(&g, spell), 1);
    g.state.players[0].mana[1] = 1;
    cast(&mut g, spell, None);
    passes(&mut g);
    assert_eq!(g.object_view(spell).zone, Zone::Battlefield);
}
#[test]
fn multiple_sources_stack_to_zero_without_reducing_colored_requirements() {
    let mut g = scenario();
    for _ in 0..4 {
        named(&mut g, Player::P0, "Mocking Sprite", Zone::Battlefield);
    }
    let spell = named(&mut g, Player::P0, "Divination", Zone::Hand);
    assert_eq!(generic(&g, spell), 0);
    reject(
        &mut g,
        Action::Cast {
            card: spell,
            target: None,
            payment: [0; 6],
        },
    );
    g.state.players[0].mana[1] = 2;
    reject(
        &mut g,
        Action::Cast {
            card: spell,
            target: None,
            payment: [0, 2, 0, 0, 0, 0],
        },
    );
    cast(&mut g, spell, None);
    assert_eq!(g.state.players[0].mana[1], 1);
    passes(&mut g);
    assert_eq!(g.state.players[0].library.len(), 18);
}
#[test]
fn cost_source_index_tracks_departure_reentry_and_snapshot_restore() {
    let mut g = scenario();
    let source = named(&mut g, Player::P0, "Voyager Quickwelder", Zone::Battlefield);
    let spell = named(&mut g, Player::P0, "Goldvein Pick", Zone::Hand);
    let saved = g.snapshot();
    assert_eq!(generic(&g, spell), 1);
    g.move_object(source, Zone::Hand);
    assert_eq!(generic(&g, spell), 2);
    assert!(!g.state.static_sources.contains(&source));
    g.move_object(source, Zone::Battlefield);
    assert_eq!(generic(&g, spell), 1);
    assert_eq!(
        g.state
            .static_sources
            .iter()
            .filter(|&&id| id == source)
            .count(),
        1
    );
    g.restore(&saved).unwrap();
    assert_eq!(generic(&g, spell), 1);
    assert_eq!(
        g.state
            .static_sources
            .iter()
            .filter(|&&id| id == source)
            .count(),
        1
    );
}
#[test]
fn second_spell_reducers_have_exact_ordinal_boundaries_for_both_players() {
    for (source, amount) in [("Uthros Psionicist", 2), ("Highspire Bell-Ringer", 1)] {
        for active in [Player::P0, Player::P1] {
            for count in [0, 1, 2, 3] {
                let mut g = scenario();
                g.state.active = active;
                g.state.players[0].spells_cast_this_turn = count;
                named(&mut g, Player::P0, source, Zone::Battlefield);
                let spell = named(&mut g, Player::P0, "Divination", Zone::Hand);
                assert_eq!(generic(&g, spell), 2 - if count == 1 { amount } else { 0 });
            }
        }
    }
}
#[test]
fn countered_first_spell_counts_for_the_second_spell_discount() {
    let mut g = scenario();
    named(&mut g, Player::P0, "Uthros Psionicist", Zone::Battlefield);
    let first = named(&mut g, Player::P0, "Bear Cub", Zone::Hand);
    g.state.players[0].mana[4] = 2;
    cast(&mut g, first, None);
    assert_eq!(g.state.players[0].spells_cast_this_turn, 1);
    replay(&mut g, Action::Pass);
    let counter = named(&mut g, Player::P1, "Counterspell", Zone::Hand);
    g.state.players[1].mana[1] = 2;
    cast(&mut g, counter, Some(Target::Spell(first)));
    passes(&mut g);
    let second = named(&mut g, Player::P0, "Divination", Zone::Hand);
    g.state.players[0].mana[1] = 1;
    cast(&mut g, second, None);
    assert_eq!(g.state.players[0].spells_cast_this_turn, 2);
    passes(&mut g);
    let third = named(&mut g, Player::P0, "Divination", Zone::Hand);
    g.state.players[0].mana[1] = 1;
    reject(
        &mut g,
        Action::Cast {
            card: third,
            target: None,
            payment: [0, 1, 0, 0, 0, 0],
        },
    );
}
#[test]
fn geyser_drake_reduces_only_during_turns_other_than_its_controllers() {
    let mut g = scenario();
    let source = named(&mut g, Player::P0, "Geyser Drake", Zone::Battlefield);
    assert!(g.keywords(source).contains(Keywords::FLYING));
    let spell = named(&mut g, Player::P0, "Murder", Zone::Hand);
    assert_eq!(generic(&g, spell), 1);
    g.state.active = Player::P1;
    assert_eq!(generic(&g, spell), 0);
    let target = named(&mut g, Player::P1, "Bear Cub", Zone::Battlefield);
    g.state.priority = Player::P0;
    g.state.players[0].mana[2] = 2;
    cast(&mut g, spell, Some(Target::Card(target)));
    passes(&mut g);
    assert_eq!(g.object_view(target).zone, Zone::Graveyard);
}
#[test]
fn self_subtype_conditions_are_owned_battlefield_facts_and_accept_changeling() {
    for (name, amount) in [
        ("Arcane Epiphany", 1),
        ("Mistmeadow Council", 1),
        ("Pearl of Wisdom", 1),
        ("Saved by the Shell", 1),
        ("Visions of Villainy", 1),
        ("Venom's Hunger", 2),
    ] {
        for (owner, zone) in [
            (Player::P0, Zone::Battlefield),
            (Player::P1, Zone::Battlefield),
            (Player::P0, Zone::Hand),
            (Player::P0, Zone::Graveyard),
        ] {
            let mut g = scenario();
            named(&mut g, owner, "Three Tree Mascot", zone);
            let spell = named(&mut g, Player::P0, name, Zone::Hand);
            assert_eq!(
                generic(&g, spell),
                base(&g, spell).generic
                    - if owner == Player::P0 && zone == Zone::Battlefield {
                        amount
                    } else {
                        0
                    },
                "{name}/{owner:?}/{zone:?}"
            );
        }
    }
}
#[test]
fn earlier_spell_self_discounts_apply_after_one_or_more_completed_casts() {
    for (name, amount) in [("Gigastorm Titan", 3), ("Focus the Mind", 2)] {
        for count in [0, 1, 2] {
            let mut g = scenario();
            g.state.players[0].spells_cast_this_turn = count;
            let id = named(&mut g, Player::P0, name, Zone::Hand);
            assert_eq!(
                generic(&g, id),
                base(&g, id).generic - if count > 0 { amount } else { 0 }
            );
        }
    }
}
#[test]
fn orysa_uses_live_total_toughness_before_entry_and_draws_two() {
    let mut g = scenario();
    let turtle = named(&mut g, Player::P0, "Aegis Turtle", Zone::Battlefield);
    named(&mut g, Player::P0, "Serra Angel", Zone::Battlefield);
    let orysa = named(&mut g, Player::P0, "Orysa, Tide Choreographer", Zone::Hand);
    assert_eq!(generic(&g, orysa), 4);
    let shell = named(&mut g, Player::P0, "Saved by the Shell", Zone::Hand);
    g.state.players[0].mana[4] = 1;
    cast(&mut g, shell, Some(Target::Card(turtle)));
    passes(&mut g);
    assert_eq!(generic(&g, orysa), 1);
    g.state.players[0].mana = [0, 1, 0, 0, 0, 1];
    cast(&mut g, orysa, None);
    passes(&mut g);
    triggered(&mut g, None);
    passes(&mut g);
    assert_eq!(g.state.players[0].library.len(), 18);
}
#[test]
fn archmage_reduces_and_draws_for_a_completed_cast_even_if_the_spell_is_countered() {
    let mut g = scenario();
    named(&mut g, Player::P0, "Archmage of Runes", Zone::Battlefield);
    named(&mut g, Player::P0, "Mocking Sprite", Zone::Battlefield);
    let spell = named(&mut g, Player::P0, "Divination", Zone::Hand);
    g.state.players[0].mana[1] = 1;
    cast(&mut g, spell, None);
    triggered(&mut g, None);
    replay(&mut g, Action::Pass);
    let counter = named(&mut g, Player::P1, "Counterspell", Zone::Hand);
    g.state.players[1].mana[1] = 2;
    cast(&mut g, counter, Some(Target::Spell(spell)));
    passes(&mut g);
    passes(&mut g);
    assert_eq!(g.object_view(spell).zone, Zone::Graveyard);
    assert_eq!(g.state.players[0].library.len(), 19);
    assert_eq!(g.state.players[0].spells_cast_this_turn, 1);
}
#[test]
fn stormcatch_reduction_preserves_haste_prowess_and_source_identity() {
    let mut g = scenario();
    let source = named(&mut g, Player::P0, "Stormcatch Mentor", Zone::Battlefield);
    g.state.objects[source as usize].summoning_sick = true;
    assert!(g.keywords(source).contains(Keywords::HASTE));
    let spell = named(&mut g, Player::P0, "Arcane Epiphany", Zone::Hand);
    assert_eq!(generic(&g, spell), 1);
    g.state.players[0].mana = [0, 2, 0, 0, 0, 1];
    cast(&mut g, spell, None);
    triggered(&mut g, None);
    passes(&mut g);
    assert_eq!((g.stats(source).power, g.stats(source).toughness), (2, 2));
    passes(&mut g);
    assert_eq!(g.state.players[0].library.len(), 17);
}
#[test]
fn honest_rutsteins_return_trigger_and_creature_discount_both_execute() {
    let mut g = scenario();
    let bear = named(&mut g, Player::P0, "Bear Cub", Zone::Graveyard);
    let source = named(&mut g, Player::P0, "Honest Rutstein", Zone::Hand);
    g.state.players[0].mana = [0, 0, 1, 0, 1, 1];
    cast(&mut g, source, None);
    passes(&mut g);
    triggered(&mut g, Some(Target::Card(bear)));
    passes(&mut g);
    assert_eq!(g.object_view(bear).zone, Zone::Hand);
    assert_eq!(generic(&g, bear), 0);
    g.state.players[0].mana[4] = 1;
    cast(&mut g, bear, None);
    passes(&mut g);
    assert_eq!(g.object_view(bear).zone, Zone::Battlefield);
}
#[test]
fn saved_by_shell_reduced_cast_still_enforces_owned_target_and_full_keywords() {
    let mut g = scenario();
    let target = named(&mut g, Player::P0, "Bear Cub", Zone::Battlefield);
    let opponent = named(&mut g, Player::P1, "Bear Cub", Zone::Battlefield);
    named(&mut g, Player::P0, "Aegis Turtle", Zone::Battlefield);
    let spell = named(&mut g, Player::P0, "Saved by the Shell", Zone::Hand);
    g.state.players[0].mana[4] = 1;
    reject(
        &mut g,
        Action::Cast {
            card: spell,
            target: Some(Target::Card(opponent)),
            payment: [0, 0, 0, 0, 1, 0],
        },
    );
    cast(&mut g, spell, Some(Target::Card(target)));
    passes(&mut g);
    let view = g.object_view(target);
    assert_eq!((view.power, view.toughness), (3, 3));
    assert_eq!(view.counters.plus_one, 1);
    assert!(
        view.keywords.contains(
            Keywords::TRAMPLE
                .union(Keywords::HEXPROOF)
                .union(Keywords::INDESTRUCTIBLE)
        )
    );
}
#[test]
fn reduced_hybrid_allocations_keep_colored_requirements_and_deduplicate_payments() {
    for name in ["Abandon Attachments", "Kin-Tree Severance"] {
        let card = Card::from_name(name).unwrap();
        for pool in [[1, 1, 1, 1, 1, 1], [0, 5, 0, 0, 0, 0], [2, 0, 2, 0, 2, 4]] {
            let mut actual = Vec::new();
            card.reduced_spell_payments(pool, 1, |p| actual.push(p));
            let mut independent = Vec::new();
            for cost in card.cost_options() {
                payments(pool, cost.colored, cost.generic.saturating_sub(1), |p| {
                    if !independent.contains(&p) {
                        independent.push(p);
                    }
                });
            }
            actual.sort();
            independent.sort();
            assert_eq!(actual, independent);
            let n = actual.len();
            actual.dedup();
            assert_eq!(actual.len(), n);
        }
    }
}
#[test]
fn staged_two_brid_total_is_locked_after_targets_and_survives_reducer_departure() {
    let mut g = scenario();
    let source = named(&mut g, Player::P0, "Mocking Sprite", Zone::Battlefield);
    let victim = named(&mut g, Player::P1, "Shivan Dragon", Zone::Battlefield);
    let spell = named(&mut g, Player::P0, "Kin-Tree Severance", Zone::Hand);
    for _ in 0..5 {
        named(&mut g, Player::P0, "Island", Zone::Battlefield);
    }
    replay(&mut g, Action::AnnounceCast { card: spell });
    assert!(
        g.observe(Player::P0)
            .cost_choices
            .iter()
            .any(|c| c.index == 0)
    );
    replay(&mut g, Action::ChooseSpellCost { cost: 0 });
    assert!(g.state.announcing.as_ref().unwrap().total_cost.is_none());
    replay(
        &mut g,
        Action::ChooseTarget {
            target: Target::Card(victim),
        },
    );
    assert_eq!(
        g.state.announcing.as_ref().unwrap().cost.unwrap().generic,
        6
    );
    assert_eq!(
        g.state
            .announcing
            .as_ref()
            .unwrap()
            .total_cost
            .unwrap()
            .generic,
        5
    );
    // Isolate a source change after declaration. No spell/ability is allowed to
    // interrupt payment; a locked total must nevertheless remain self-contained.
    g.move_object(source, Zone::Hand);
    assert_eq!(generic(&g, spell), 6);
    assert_eq!(g.observe(Player::P0).payment_request.unwrap().generic, 5);
    g.state.players[0].mana[1] = 6;
    reject(
        &mut g,
        Action::PayCosts {
            payment: [0, 6, 0, 0, 0, 0],
        },
    );
    replay(
        &mut g,
        Action::PayCosts {
            payment: [0, 5, 0, 0, 0, 0],
        },
    );
    assert_eq!(g.state.players[0].mana[1], 1);
    passes(&mut g);
    assert_eq!(g.object_view(victim).zone, Zone::Exile);
}
#[test]
fn additional_sacrifice_can_remove_the_reducer_without_repricing_its_spell() {
    let mut g = scenario();
    let source = named(
        &mut g,
        Player::P0,
        "Geist of Saint Thalia",
        Zone::Battlefield,
    );
    let recipient = named(&mut g, Player::P0, "Bear Cub", Zone::Battlefield);
    let spell = named(&mut g, Player::P0, "Duty Beyond Death", Zone::Hand);
    g.state.players[0].mana[0] = 1;
    replay(&mut g, Action::AnnounceCast { card: spell });
    assert_eq!(
        g.state
            .announcing
            .as_ref()
            .unwrap()
            .total_cost
            .unwrap()
            .generic,
        0
    );
    let reference = g.object_ref(source);
    replay(&mut g, Action::ChooseAdditionalCost { object: reference });
    pay(&mut g);
    assert_eq!(g.object_view(source).zone, Zone::Graveyard);
    assert_eq!(g.state.players[0].mana[0], 0);
    passes(&mut g);
    assert_eq!(g.object_view(recipient).counters.plus_one, 1);
    assert!(g.keywords(recipient).contains(Keywords::INDESTRUCTIBLE));
    let next = named(&mut g, Player::P0, "Duty Beyond Death", Zone::Hand);
    assert_eq!(generic(&g, next), 1);
}
#[test]
fn modal_declaration_locks_reduced_cost_after_mode_selection() {
    let mut g = scenario();
    named(
        &mut g,
        Player::P0,
        "Geist of Saint Thalia",
        Zone::Battlefield,
    );
    let spell = named(&mut g, Player::P0, "Pawpatch Formation", Zone::Hand);
    g.state.players[0].mana[4] = 1;
    replay(&mut g, Action::AnnounceCast { card: spell });
    assert!(g.state.announcing.as_ref().unwrap().total_cost.is_none());
    replay(&mut g, Action::ChooseMode { mode: 2 });
    assert_eq!(g.observe(Player::P0).payment_request.unwrap().generic, 0);
    pay(&mut g);
    passes(&mut g);
    assert_eq!(g.state.players[0].life, 20);
    assert_eq!(
        g.state
            .objects
            .iter()
            .filter(|o| o.zone == Zone::Battlefield && o.card.has_subtype("Food"))
            .count(),
        1
    );
    assert_eq!(g.state.players[0].library.len(), 19);
}

#[test]
fn reduced_draw_and_removal_spells_keep_all_unconditional_effects() {
    for (name, payment, drawn, life) in [
        ("Pearl of Wisdom", [0, 1, 0, 0, 0, 1], 2, 20),
        ("Visions of Villainy", [0, 0, 1, 0, 0, 1], 2, 18),
        ("Venom's Hunger", [0, 0, 1, 0, 0, 2], 0, 22),
    ] {
        let mut g = scenario();
        named(&mut g, Player::P0, "Three Tree Mascot", Zone::Battlefield);
        let victim = named(&mut g, Player::P1, "Bear Cub", Zone::Battlefield);
        let spell = named(&mut g, Player::P0, name, Zone::Hand);
        g.state.players[0].mana = payment;
        cast(
            &mut g,
            spell,
            if name == "Venom's Hunger" {
                Some(Target::Card(victim))
            } else {
                None
            },
        );
        passes(&mut g);
        assert_eq!(g.state.players[0].library.len(), 20 - drawn);
        assert_eq!(g.state.players[0].life, life);
        assert_eq!(g.state.players[0].mana, [0; 6]);
        assert_eq!(
            g.object_view(victim).zone,
            if name == "Venom's Hunger" {
                Zone::Graveyard
            } else {
                Zone::Battlefield
            }
        );
    }
}
#[test]
fn focus_mind_and_gigastorm_discount_after_a_real_first_spell() {
    for name in ["Focus the Mind", "Gigastorm Titan"] {
        let mut g = scenario();
        let first = named(&mut g, Player::P0, "Bear Cub", Zone::Hand);
        let spell = named(&mut g, Player::P0, name, Zone::Hand);
        let spare = named(&mut g, Player::P0, "Shock", Zone::Hand);
        g.state.players[0].mana = [0, 1, 0, 0, 2, if name == "Focus the Mind" { 2 } else { 1 }];
        cast(&mut g, first, None);
        passes(&mut g);
        assert_eq!(g.state.players[0].spells_cast_this_turn, 1);
        cast(&mut g, spell, None);
        passes(&mut g);
        assert_eq!(g.state.players[0].spells_cast_this_turn, 2);
        if name == "Focus the Mind" {
            assert!(matches!(g.state.pending, Pending::EffectDiscard { .. }));
            assert_eq!(g.state.players[0].library.len(), 17);
            assert!(g.observe(Player::P1).effect_discards.is_empty());
            reject(&mut g, Action::Pass);
            replay(&mut g, Action::Discard { card: spare });
            assert_eq!(g.state.players[0].library.len(), 17);
        } else {
            assert_eq!(
                (g.object_view(spell).power, g.object_view(spell).toughness),
                (4, 4)
            );
        }
        assert_eq!(g.state.players[0].mana, [0; 6]);
    }
}
#[test]
fn mistmeadow_council_discount_and_entry_draw_both_execute() {
    let mut g = scenario();
    named(&mut g, Player::P0, "Three Tree Mascot", Zone::Battlefield);
    let spell = named(&mut g, Player::P0, "Mistmeadow Council", Zone::Hand);
    g.state.players[0].mana = [0, 0, 0, 0, 1, 3];
    cast(&mut g, spell, None);
    passes(&mut g);
    triggered(&mut g, None);
    passes(&mut g);
    assert_eq!(g.state.players[0].library.len(), 19);
    assert_eq!(
        (g.object_view(spell).power, g.object_view(spell).toughness),
        (4, 3)
    );
    for name in [
        "Geist of Saint Thalia",
        "Geyser Drake",
        "Highspire Bell-Ringer",
        "Mocking Sprite",
    ] {
        let id = named(&mut g, Player::P1, name, Zone::Battlefield);
        assert!(g.keywords(id).contains(Keywords::FLYING));
    }
}

#[test]
fn artifact_affinity_uses_owned_battlefield_count_and_preserves_colored_payment() {
    for staged in [false, true] {
        let mut g = scenario();
        let spell = named(&mut g, Player::P0, "Memory Guardian", Zone::Hand);
        named(&mut g, Player::P0, "Memory Guardian", Zone::Battlefield);
        named(&mut g, Player::P0, "Goldvein Pick", Zone::Battlefield);
        named(&mut g, Player::P1, "Goldvein Pick", Zone::Battlefield);
        named(&mut g, Player::P0, "Goldvein Pick", Zone::Hand);
        named(&mut g, Player::P0, "Bear Cub", Zone::Battlefield);
        assert_eq!(generic(&g, spell), 2);
        g.state.players[0].mana = [0, 1, 0, 0, 0, 1];
        reject(&mut g, Action::AnnounceCast { card: spell });
        g.state.players[0].mana = [0, 0, 0, 0, 0, 5];
        reject(&mut g, Action::AnnounceCast { card: spell });
        g.state.players[0].mana = [0, 1, 0, 0, 0, 2];
        if staged {
            replay(&mut g, Action::AnnounceCast { card: spell });
            assert_eq!(g.observe(Player::P0).payment_request.unwrap().generic, 2);
            pay(&mut g);
        } else {
            cast(&mut g, spell, None);
        }
        passes(&mut g);
        assert_eq!(g.object_view(spell).zone, Zone::Battlefield);
        assert_eq!((g.stats(spell).power, g.stats(spell).toughness), (3, 4));
        assert!(g.keywords(spell).contains(Keywords::FLYING));
    }
}

#[test]
fn graveyard_affinity_unions_count_each_owned_card_once() {
    let mut g = scenario();
    let chitin = named(&mut g, Player::P0, "Chitin Gravestalker", Zone::Hand);
    let terror = named(&mut g, Player::P0, "Tolarian Terror", Zone::Hand);
    for name in [
        "Memory Guardian",
        "Bear Cub",
        "Goldvein Pick",
        "Divination",
        "Counterspell",
    ] {
        named(&mut g, Player::P0, name, Zone::Graveyard);
    }
    named(&mut g, Player::P1, "Memory Guardian", Zone::Graveyard);
    named(&mut g, Player::P0, "Memory Guardian", Zone::Battlefield);
    assert_eq!(generic(&g, chitin), 2); // artifact creature counts once, not twice
    assert_eq!(generic(&g, terror), 4);
    g.state.players[0].mana = [0, 0, 1, 0, 0, 2];
    cast(&mut g, chitin, None);
    passes(&mut g);
    assert_eq!((g.stats(chitin).power, g.stats(chitin).toughness), (5, 4));
    g.state.players[0].mana = [0, 1, 0, 0, 0, 4];
    cast(&mut g, terror, None);
    passes(&mut g);
    assert_eq!((g.stats(terror).power, g.stats(terror).toughness), (5, 5));
    assert!(
        g.state.objects[terror as usize]
            .card
            .triggers()
            .iter()
            .any(|t| t.condition == crate::cards::TriggerCondition::Ward)
    );
}

#[test]
fn affinity_does_not_discount_cycling_activation() {
    let mut g = scenario();
    let chitin = named(&mut g, Player::P0, "Chitin Gravestalker", Zone::Hand);
    for _ in 0..6 {
        named(&mut g, Player::P0, "Memory Guardian", Zone::Graveyard);
    }
    assert_eq!(generic(&g, chitin), 0);
    g.state.players[0].mana[5] = 1;
    assert!(
        !g.legal_actions()
            .iter()
            .any(|a| matches!(a,Action::Activate{card,..} if *card==chitin))
    );
    g.state.players[0].mana[5] = 2;
    let a = g
        .legal_actions()
        .into_iter()
        .find(|a| matches!(a,Action::Activate{card,..} if *card==chitin))
        .unwrap();
    replay(&mut g, a);
    assert_eq!(g.object_view(chitin).zone, Zone::Graveyard);
    passes(&mut g);
    assert_eq!(g.state.players[0].library.len(), 19);
    assert_eq!(g.state.players[0].spells_cast_this_turn, 0);
}

#[test]
fn creature_affinity_entry_draw_and_global_spell_scope_are_exact() {
    let mut g = scenario();
    let source = named(&mut g, Player::P0, "Witherbloom, the Balancer", Zone::Hand);
    named(&mut g, Player::P0, "Bear Cub", Zone::Battlefield);
    named(&mut g, Player::P1, "Bear Cub", Zone::Battlefield);
    assert_eq!(generic(&g, source), 5);
    g.state.players[0].mana = [0, 0, 1, 0, 1, 5];
    cast(&mut g, source, None);
    passes(&mut g);
    assert!(
        g.keywords(source).contains(Keywords::FLYING)
            && g.keywords(source).contains(Keywords::DEATHTOUCH)
    );
    let draw = named(&mut g, Player::P0, "Divination", Zone::Hand);
    let pack = named(&mut g, Player::P0, "Salt Road Packbeast", Zone::Hand);
    assert_eq!(generic(&g, draw), 0);
    assert_eq!(generic(&g, pack), 3);
    g.state.players[0].mana = [1, 0, 0, 0, 0, 3];
    cast(&mut g, pack, None);
    passes(&mut g);
    triggered(&mut g, None);
    passes(&mut g);
    assert_eq!(g.state.players[0].library.len(), 19);
    assert_eq!((g.stats(pack).power, g.stats(pack).toughness), (4, 3));
    g.state.players[0].mana[1] = 1;
    cast(&mut g, draw, None);
    passes(&mut g);
    assert_eq!(g.state.players[0].library.len(), 17);
}

#[test]
fn cave_reductions_sum_real_owned_battlefield_and_graveyard_lands() {
    let mut g = scenario();
    let leech = named(&mut g, Player::P0, "Gargantuan Leech", Zone::Hand);
    // Captivating Cave is an executable Cave; graveyard lands count as cards.
    named(&mut g, Player::P0, "Captivating Cave", Zone::Battlefield);
    named(&mut g, Player::P0, "Captivating Cave", Zone::Graveyard);
    named(&mut g, Player::P1, "Captivating Cave", Zone::Graveyard);
    named(&mut g, Player::P0, "Captivating Cave", Zone::Hand);
    named(&mut g, Player::P0, "Captivating Cave", Zone::Exile);
    assert_eq!(generic(&g, leech), 5);
    g.state.players[0].mana = [0, 0, 1, 0, 0, 5];
    cast(&mut g, leech, None);
    passes(&mut g);
    assert!(g.keywords(leech).contains(Keywords::LIFELINK));
    assert_eq!((g.stats(leech).power, g.stats(leech).toughness), (5, 5));
}

#[test]
fn exile_and_graveyard_creature_counts_exclude_other_zones_and_opponent() {
    let mut g = scenario();
    let swarm = named(&mut g, Player::P0, "Huskburster Swarm", Zone::Hand);
    named(&mut g, Player::P0, "Bear Cub", Zone::Graveyard);
    named(&mut g, Player::P0, "Memory Guardian", Zone::Exile);
    named(&mut g, Player::P1, "Bear Cub", Zone::Exile);
    named(&mut g, Player::P0, "Goldvein Pick", Zone::Graveyard);
    named(&mut g, Player::P0, "Bear Cub", Zone::Battlefield);
    assert_eq!(generic(&g, swarm), 5);
    g.state.players[0].mana = [0, 0, 1, 0, 0, 5];
    cast(&mut g, swarm, None);
    passes(&mut g);
    assert!(
        g.keywords(swarm).contains(Keywords::MENACE)
            && g.keywords(swarm).contains(Keywords::DEATHTOUCH)
    );
    assert_eq!((g.stats(swarm).power, g.stats(swarm).toughness), (6, 6));
}

#[test]
fn vivid_counts_distinct_live_colors_and_no_colorless_or_hand_color() {
    let mut g = scenario();
    let spell = named(&mut g, Player::P0, "Wildvine Pummeler", Zone::Hand);
    assert_eq!(generic(&g, spell), 6);
    named(
        &mut g,
        Player::P0,
        "Sami, Wildcat Captain",
        Zone::Battlefield,
    );
    named(
        &mut g,
        Player::P0,
        "Witherbloom, the Balancer",
        Zone::Battlefield,
    );
    named(&mut g, Player::P0, "Geyser Drake", Zone::Battlefield);
    named(&mut g, Player::P0, "Geyser Drake", Zone::Battlefield);
    assert_eq!(generic(&g, spell), 1);
    named(&mut g, Player::P0, "Three Tree Mascot", Zone::Battlefield);
    // Five colors reduce by five; Sami's separate artifact affinity adds one.
    assert_eq!(generic(&g, spell), 0);
    g.state.players[0].mana[4] = 1;
    cast(&mut g, spell, None);
    passes(&mut g);
    assert!(
        g.keywords(spell).contains(Keywords::REACH)
            && g.keywords(spell).contains(Keywords::TRAMPLE)
    );
    assert_eq!((g.stats(spell).power, g.stats(spell).toughness), (6, 5));
}

#[test]
fn multiple_affinity_instances_stack_without_discounting_colored_components() {
    let mut g = scenario();
    named(
        &mut g,
        Player::P0,
        "Sami, Wildcat Captain",
        Zone::Battlefield,
    );
    named(&mut g, Player::P0, "Memory Guardian", Zone::Battlefield);
    named(&mut g, Player::P0, "Goldvein Pick", Zone::Battlefield);
    let spell = named(&mut g, Player::P0, "Memory Guardian", Zone::Hand);
    assert_eq!(generic(&g, spell), 0); // two artifacts apply to self and granted affinity
    g.state.players[0].mana[5] = 5;
    reject(&mut g, Action::AnnounceCast { card: spell });
    g.state.players[0].mana = [0, 1, 0, 0, 0, 0];
    cast(&mut g, spell, None);
    passes(&mut g);
}

#[test]
fn affinity_total_stays_locked_when_additional_sacrifice_removes_counted_artifact() {
    let mut g = scenario();
    named(
        &mut g,
        Player::P0,
        "Sami, Wildcat Captain",
        Zone::Battlefield,
    );
    let victim = named(&mut g, Player::P0, "Memory Guardian", Zone::Battlefield);
    let receiver = named(&mut g, Player::P0, "Bear Cub", Zone::Battlefield);
    let spell = named(&mut g, Player::P0, "Duty Beyond Death", Zone::Hand);
    g.state.players[0].mana[0] = 1;
    replay(&mut g, Action::AnnounceCast { card: spell });
    let reference = g.object_ref(victim);
    replay(&mut g, Action::ChooseAdditionalCost { object: reference });
    assert_eq!(g.observe(Player::P0).payment_request.unwrap().generic, 0);
    pay(&mut g);
    assert_eq!(g.object_view(victim).zone, Zone::Graveyard);
    assert_eq!(generic(&g, spell), 1);
    passes(&mut g);
    assert_eq!(g.object_view(receiver).counters.plus_one, 1);
    assert!(g.keywords(receiver).contains(Keywords::INDESTRUCTIBLE));
}

#[test]
fn tribal_affinity_uses_changeling_and_executes_claws_and_scales_bodies() {
    let mut g = scenario();
    let mascot = named(&mut g, Player::P0, "Three Tree Mascot", Zone::Battlefield);
    named(&mut g, Player::P1, "Three Tree Mascot", Zone::Battlefield);
    let claws = named(&mut g, Player::P0, "Claws Out", Zone::Hand);
    let scales = named(&mut g, Player::P0, "Scales of Shale", Zone::Hand);
    assert_eq!(generic(&g, claws), 2);
    assert_eq!(generic(&g, scales), 1);
    g.state.players[0].mana = [2, 0, 0, 0, 0, 2];
    cast(&mut g, claws, None);
    passes(&mut g);
    assert_eq!((g.stats(mascot).power, g.stats(mascot).toughness), (4, 3));
    g.state.players[0].mana = [0, 0, 1, 0, 0, 1];
    cast(&mut g, scales, Some(Target::Card(mascot)));
    passes(&mut g);
    assert_eq!((g.stats(mascot).power, g.stats(mascot).toughness), (6, 3));
    assert!(
        g.keywords(mascot).contains(Keywords::LIFELINK)
            && g.keywords(mascot).contains(Keywords::INDESTRUCTIBLE)
    );
}

#[test]
fn town_affinity_draw_four_and_artifact_entry_surveil_replay() {
    let mut g = scenario();
    named(&mut g, Player::P0, "Gohn, Town of Ruin", Zone::Battlefield);
    named(&mut g, Player::P0, "Gohn, Town of Ruin", Zone::Graveyard);
    let travel = named(&mut g, Player::P0, "Travel the Overworld", Zone::Hand);
    assert_eq!(generic(&g, travel), 4);
    g.state.players[0].mana = [0, 2, 0, 0, 0, 4];
    cast(&mut g, travel, None);
    passes(&mut g);
    assert_eq!(g.state.players[0].library.len(), 16);
    named(&mut g, Player::P0, "Three Tree Mascot", Zone::Battlefield);
    let valk = named(&mut g, Player::P0, "Valkyrie Aerial Unit", Zone::Hand);
    assert_eq!(generic(&g, valk), 4);
    g.state.players[0].mana = [0, 2, 0, 0, 0, 4];
    cast(&mut g, valk, None);
    passes(&mut g);
    triggered(&mut g, None);
    passes(&mut g);
    assert_eq!(g.observe(Player::P1).actions.len(), 0);
    for _ in 0..10 {
        if g.state.pending == Pending::Priority {
            break;
        }
        let a = g
            .legal_actions()
            .into_iter()
            .find(|a| !matches!(a, Action::Concede))
            .unwrap();
        replay(&mut g, a);
    }
    assert_eq!(g.state.pending, Pending::Priority);
    assert!(g.keywords(valk).contains(Keywords::FLYING));
    assert_eq!((g.stats(valk).power, g.stats(valk).toughness), (5, 4));
}

#[test]
fn hivepool_affinity_grants_keywords_and_creates_exact_upkeep_slivers() {
    let mut g = scenario();
    let mascot = named(&mut g, Player::P0, "Three Tree Mascot", Zone::Battlefield);
    let hive = named(&mut g, Player::P0, "Thrumming Hivepool", Zone::Hand);
    assert_eq!(generic(&g, hive), 5);
    g.state.players[0].mana[5] = 5;
    cast(&mut g, hive, None);
    passes(&mut g);
    assert!(
        g.keywords(mascot).contains(Keywords::DOUBLE_STRIKE)
            && g.keywords(mascot).contains(Keywords::HASTE)
    );
    g.begin_step(Step::Upkeep);
    g.give_priority();
    g.prepare_triggers();
    triggered(&mut g, None);
    passes(&mut g);
    let slivers: Vec<_> = g
        .state
        .objects
        .iter()
        .enumerate()
        .filter(|(_, o)| {
            o.owner == Player::P0
                && o.zone == Zone::Battlefield
                && o.card.is_token()
                && o.card.has_subtype("Sliver")
        })
        .map(|(i, _)| i as CardId)
        .collect();
    assert_eq!(slivers.len(), 2);
    let token_card = g.state.objects[slivers[0] as usize].card;
    for id in slivers {
        assert_eq!((g.stats(id).power, g.stats(id).toughness), (1, 1));
        assert!(
            g.keywords(id).contains(Keywords::DOUBLE_STRIKE)
                && g.keywords(id).contains(Keywords::HASTE)
        );
    }
    let second = named(&mut g, Player::P0, "Thrumming Hivepool", Zone::Hand);
    assert_eq!(generic(&g, second), 3);
    spawn(&mut g, Player::P0, token_card, Zone::Graveyard);
    spawn(&mut g, Player::P0, token_card, Zone::Exile);
    let swarm = named(&mut g, Player::P0, "Huskburster Swarm", Zone::Hand);
    assert_eq!(generic(&g, swarm), 7); // token remnants are not creature cards
}

#[test]
fn distinct_land_names_discount_fungal_without_counting_duplicates_or_other_zones() {
    let mut g = scenario();
    let spell = named(&mut g, Player::P0, "Fungal Colossus", Zone::Hand);
    for name in ["Forest", "Forest", "Island"] {
        let id = named(&mut g, Player::P0, name, Zone::Battlefield);
        g.state.objects[id as usize].tapped = true;
    }
    named(&mut g, Player::P1, "Swamp", Zone::Battlefield);
    named(&mut g, Player::P0, "Mountain", Zone::Graveyard);
    named(&mut g, Player::P0, "Plains", Zone::Hand);
    assert_eq!(generic(&g, spell), 4);
    g.state.players[0].mana = [0, 0, 0, 0, 1, 3];
    reject(&mut g, Action::AnnounceCast { card: spell });
    g.state.players[0].mana[5] = 4;
    replay(&mut g, Action::AnnounceCast { card: spell });
    assert_eq!(g.observe(Player::P0).payment_request.unwrap().generic, 4);
    pay(&mut g);
    passes(&mut g);
    assert_eq!((g.stats(spell).power, g.stats(spell).toughness), (5, 5));
}

#[test]
fn ghalta_power_aggregates_keep_negative_power_and_use_live_effective_values() {
    for (name, expected) in [("Ghalta the Unstoppable", 3), ("Ghalta, Primal Hunger", 7)] {
        let mut g = scenario();
        let spell = named(&mut g, Player::P0, name, Zone::Hand);
        assert_eq!(
            generic(&g, spell),
            if name == "Ghalta the Unstoppable" {
                8
            } else {
                10
            }
        );
        let weak = named(&mut g, Player::P0, "Bear Cub", Zone::Battlefield);
        g.state.objects[weak as usize].growth.power = -4; // power -2, toughness remains 2
        assert_eq!(
            generic(&g, spell),
            if name == "Ghalta the Unstoppable" {
                8
            } else {
                10
            }
        );
        let strong = named(&mut g, Player::P0, "Fire Elemental", Zone::Battlefield);
        named(
            &mut g,
            Player::P1,
            "Ghalta, Primal Hunger",
            Zone::Battlefield,
        );
        assert_eq!(generic(&g, spell), expected); // max 5, signed sum 5 + -2 = 3
        g.state.objects[strong as usize].counters.plus_one = 1;
        assert_eq!(generic(&g, spell), expected - 1);
        g.state.players[0].mana = [0, 0, 0, 0, 2, expected - 1];
        cast(&mut g, spell, None);
        passes(&mut g);
        assert!(g.keywords(spell).contains(Keywords::TRAMPLE));
        let stats = if name == "Ghalta the Unstoppable" {
            (8, 8)
        } else {
            (12, 12)
        };
        assert_eq!((g.stats(spell).power, g.stats(spell).toughness), stats);
        assert_eq!(
            g.keywords(weak).contains(Keywords::TRAMPLE),
            name == "Ghalta the Unstoppable"
        );
    }
}

#[test]
fn eagle_discount_filters_live_flying_and_sums_signed_power() {
    let mut g = scenario();
    let eagle = named(&mut g, Player::P0, "The Lord of the Eagles", Zone::Hand);
    let memory = named(&mut g, Player::P0, "Memory Guardian", Zone::Battlefield);
    named(&mut g, Player::P0, "Geyser Drake", Zone::Battlefield);
    let ground = named(&mut g, Player::P0, "Fire Elemental", Zone::Battlefield);
    named(&mut g, Player::P1, "Serra Angel", Zone::Battlefield);
    assert_eq!(generic(&g, eagle), 2);
    g.state.objects[memory as usize].growth.power = -5;
    assert_eq!(generic(&g, eagle), 7); // -2 + 2
    g.state.objects[ground as usize].gained_keywords = Keywords::FLYING;
    assert_eq!(generic(&g, eagle), 2);
    g.state.players[0].mana = [0, 2, 0, 0, 0, 2];
    cast(&mut g, eagle, None);
    passes(&mut g);
    assert_eq!((g.stats(eagle).power, g.stats(eagle).toughness), (8, 8));
    assert!(
        g.keywords(eagle)
            .contains(Keywords::FLASH.union(Keywords::FLYING))
    );
}

#[test]
fn outlaw_affinity_counts_overlapping_types_and_changeling_once() {
    let mut g = scenario();
    let spell = named(&mut g, Player::P0, "Hellspur Brute", Zone::Hand);
    named(&mut g, Player::P0, "Three Tree Mascot", Zone::Battlefield);
    named(&mut g, Player::P0, "The Scarlet Witch", Zone::Battlefield); // Warlock
    named(
        &mut g,
        Player::P0,
        "Sami, Wildcat Captain",
        Zone::Battlefield,
    ); // Rogue
    named(&mut g, Player::P1, "Hellspur Brute", Zone::Battlefield);
    named(&mut g, Player::P0, "Three Tree Mascot", Zone::Graveyard);
    // Three outlaws reduce by three; Sami separately discounts by one artifact.
    assert_eq!(generic(&g, spell), 0);
    let sami = g
        .state
        .objects
        .iter()
        .position(|o| o.card.name() == "Sami, Wildcat Captain")
        .unwrap() as CardId;
    g.move_object(sami, Zone::Graveyard);
    assert_eq!(generic(&g, spell), 2);
    g.state.players[0].mana = [0, 0, 0, 1, 0, 2];
    cast(&mut g, spell, None);
    passes(&mut g);
    assert_eq!((g.stats(spell).power, g.stats(spell).toughness), (5, 4));
    assert!(g.keywords(spell).contains(Keywords::TRAMPLE));
}

#[test]
fn scarlet_source_power_filters_printed_mana_value_and_receiving_spell_type() {
    let mut g = scenario();
    let source = named(&mut g, Player::P0, "The Scarlet Witch", Zone::Battlefield);
    for (name, expected) in [
        ("Quick Study", 2),
        ("Murder", 1),
        ("Arcane Epiphany", 1),
        ("Kin-Tree Severance", 4),
        ("Memory Guardian", 4),
    ] {
        let spell = named(&mut g, Player::P0, name, Zone::Hand);
        assert_eq!(generic(&g, spell), expected, "{name}");
    }
    let draw = named(&mut g, Player::P0, "Arcane Epiphany", Zone::Hand);
    g.state.objects[source as usize].counters.plus_one = 1;
    assert_eq!(generic(&g, draw), 0);
    g.state.objects[source as usize].growth.power = -6;
    assert_eq!(generic(&g, draw), 3);
    g.state.objects[source as usize].growth.power = 0;
    g.state.players[0].mana[1] = 2;
    cast(&mut g, draw, None);
    passes(&mut g);
    assert_eq!(g.state.players[0].library.len(), 17);
    g.move_object(source, Zone::Graveyard);
    assert_eq!(generic(&g, draw), 3);
}

#[test]
fn derived_total_stays_locked_when_mana_sacrifice_changes_a_creature_cda() {
    let mut g = scenario();
    let bunny = named(&mut g, Player::P0, "Regal Bunnicorn", Zone::Battlefield);
    let treasure = Card::catalogue()
        .find(|c| c.token_info().is_some_and(|i| i.name == "Treasure Token"))
        .unwrap();
    let token = spawn(&mut g, Player::P0, treasure, Zone::Battlefield);
    let spell = named(&mut g, Player::P0, "Ghalta, Primal Hunger", Zone::Hand);
    assert_eq!(g.stats(bunny).power, 2);
    assert_eq!(generic(&g, spell), 8);
    g.state.players[0].mana = [0, 0, 0, 0, 2, 7];
    replay(&mut g, Action::AnnounceCast { card: spell });
    assert_eq!(g.observe(Player::P0).payment_request.unwrap().generic, 8);
    let a=g.legal_actions().into_iter().find(|a|matches!(a,Action::ActivateMana{card,production,..} if *card==token&&*production==[0,0,0,0,1,0])).unwrap();
    replay(&mut g, a);
    assert_eq!(g.stats(bunny).power, 1);
    assert_eq!(generic(&g, spell), 9);
    assert_eq!(g.observe(Player::P0).payment_request.unwrap().generic, 8);
    pay(&mut g);
    passes(&mut g);
    assert_eq!(g.object_view(spell).zone, Zone::Battlefield);
}

#[test]
fn scarlet_two_brid_uses_printed_value_and_keeps_total_after_source_departure() {
    let mut g = scenario();
    let source = named(&mut g, Player::P0, "The Scarlet Witch", Zone::Battlefield);
    let target = named(&mut g, Player::P1, "Shivan Dragon", Zone::Battlefield);
    let spell = named(&mut g, Player::P0, "Kin-Tree Severance", Zone::Hand);
    g.state.players[0].mana[5] = 4;
    replay(&mut g, Action::AnnounceCast { card: spell });
    replay(&mut g, Action::ChooseSpellCost { cost: 0 });
    replay(
        &mut g,
        Action::ChooseTarget {
            target: Target::Card(target),
        },
    );
    g.move_object(source, Zone::Graveyard);
    assert_eq!(generic(&g, spell), 6);
    assert_eq!(g.observe(Player::P0).payment_request.unwrap().generic, 4);
    pay(&mut g);
    passes(&mut g);
    assert_eq!(g.object_view(target).zone, Zone::Exile);
}

#[test]
fn all_fates_scroll_mana_and_sacrifice_draw_use_distinct_land_names() {
    let mut g = scenario();
    let source = named(&mut g, Player::P0, "All-Fates Scroll", Zone::Battlefield);
    let saved = g.snapshot();
    for color in 0..5 {
        g.restore(&saved).unwrap();
        let mut production = [0; 6];
        production[color] = 1;
        let a=g.legal_actions().into_iter().find(|a|matches!(a,Action::ActivateMana{card,production:p,..} if *card==source&&*p==production)).unwrap();
        replay(&mut g, a);
        assert_eq!(g.state.players[0].mana[color], 1);
        assert!(g.object_view(source).tapped);
    }
    g.restore(&saved).unwrap();
    for name in ["Forest", "Forest", "Island"] {
        named(&mut g, Player::P0, name, Zone::Battlefield);
    }
    named(&mut g, Player::P1, "Swamp", Zone::Battlefield);
    named(&mut g, Player::P0, "Mountain", Zone::Graveyard);
    g.state.players[0].mana[5] = 7;
    let a = g
        .legal_actions()
        .into_iter()
        .find(|a| matches!(a,Action::Activate{card,ability:1,..} if *card==source))
        .unwrap();
    replay(&mut g, a);
    assert_eq!(g.object_view(source).zone, Zone::Graveyard);
    passes(&mut g);
    assert_eq!(g.state.players[0].library.len(), 18);
}

#[test]
fn distinct_names_use_rules_names_rather_than_internal_token_factory_keys() {
    let mut g = scenario();
    let source = named(&mut g, Player::P0, "Fungal Colossus", Zone::Hand);
    let (left, right) = Card::catalogue()
        .filter(|c| c.is_token())
        .find_map(|left| {
            Card::catalogue()
                .find(|right| {
                    *right != left
                        && right
                            .token_info()
                            .is_some_and(|r| r.name == left.token_info().unwrap().name)
                })
                .map(|right| (left, right))
        })
        .expect("two factories with one rules name");
    assert_ne!(left.name(), right.name());
    spawn(&mut g, Player::P0, left, Zone::Battlefield);
    spawn(&mut g, Player::P0, right, Zone::Battlefield);
    named(&mut g, Player::P0, "Bear Cub", Zone::Battlefield);
    let filter = CountFilter {
        zone: CountZone::Battlefield,
        relation: 0,
        kind: 1,
        exclude: 0,
        subtype: None,
        basic: false,
        other: false,
        cards_only: false,
    };
    assert_eq!(
        g.stat_value(
            StatValue::DistinctNames(filter),
            g.object_ref(source),
            Player::P0
        ),
        2
    );
}
