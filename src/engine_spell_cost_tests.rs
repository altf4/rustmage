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
