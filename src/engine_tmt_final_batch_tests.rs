//! Complete final-seven paid rules interactions, privacy and owned branches.
use super::ability_timeline_tests::{cast, checked, named, next_turn, settle, settle_on, setup};
use super::*;

fn reject(g: &mut Game, action: Action) {
    let before = g.snapshot();
    let rev = g.revision();
    assert!(g.apply(g.actor().unwrap(), action).is_err());
    assert_eq!(g.snapshot(), before);
    assert_eq!(g.revision(), rev);
}
fn finish(g: &mut Game) {
    let initial = g.state.announcing.as_ref().map(|a| a.item.id);
    for _ in 0..12 {
        if g.state.announcing.is_none() || g.state.announcing.as_ref().map(|a| a.item.id) != initial
        {
            return;
        }
        let a = g
            .legal_actions()
            .into_iter()
            .find(|a| {
                matches!(
                    a,
                    Action::PayCosts { .. }
                        | Action::FinishTargets
                        | Action::ChooseSpellCost { .. }
                        | Action::ChooseX { .. }
                )
            })
            .expect("payment");
        checked(g, a);
    }
    panic!("payment");
}
fn paid(g: &mut Game, p: Player, name: &str) -> CardId {
    let id = named(g, p, name, Zone::Hand);
    checked(g, Action::AnnounceCast { card: id });
    finish(g);
    id
}
fn refill(g: &mut Game) {
    for state in &mut g.state.players {
        state.mana = [30; 6];
    }
}
fn to_step(g: &mut Game, wanted: Step) {
    for _ in 0..100 {
        if g.state.step == wanted {
            return;
        }
        let actions = g.legal_actions();
        let a = actions
            .iter()
            .find(|a| {
                matches!(
                    a,
                    Action::Pass | Action::FinishAttackers | Action::FinishBlockers
                )
            })
            .or_else(|| actions.iter().find(|a| !matches!(a, Action::Concede)))
            .unwrap()
            .clone();
        checked(g, a);
    }
    panic!("step {wanted:?}");
}
fn mature(g: &mut Game) {
    next_turn(g);
    next_turn(g);
    to_step(g, Step::Main1);
    refill(g);
}
fn until(g: &mut Game, predicate: impl Fn(&Game) -> bool) {
    g.prepare_triggers();
    for _ in 0..80 {
        if predicate(g) {
            return;
        }
        let a = if g.state.pending == Pending::Priority {
            Action::Pass
        } else {
            g.legal_actions()
                .into_iter()
                .find(|a| !matches!(a, Action::Concede))
                .unwrap()
        };
        checked(g, a);
    }
    panic!("phase {:?}", g.state.pending);
}
fn activate(g: &mut Game, id: CardId) {
    checked(
        g,
        Action::AnnounceActivate {
            card: id,
            ability: 0,
            counter_payment: Counters::default(),
        },
    );
}
fn attack(g: &mut Game, id: CardId) {
    to_step(g, Step::Attackers);
    checked(g, Action::Attack { card: id });
    checked(g, Action::FinishAttackers);
}
fn lore(g: &mut Game, id: CardId) {
    g.effect_counter(id, CounterKind::Lore, 1, replacements::ResumeMode::Pending);
    g.prepare_triggers();
}
fn copied(g: &Game, definition: Card) -> Vec<CardId> {
    g.state
        .objects
        .iter()
        .enumerate()
        .filter(|(_, o)| o.zone == Zone::Battlefield && o.card == definition && o.is_token())
        .map(|(i, _)| i as CardId)
        .collect()
}
fn all_types(g: &mut Game, id: CardId) {
    let timestamp = g.next_role_timestamp();
    let recipient = g.object_ref(id);
    g.state
        .roles
        .get_or_insert_with(Default::default)
        .creature_types
        .push(subtypes::CreatureTypesEffectView {
            recipient,
            source: recipient,
            types: &[],
            additional: false,
            all_creature_types: true,
            expires_turn: None,
            timestamp,
        });
}
#[test]
fn tmt_final_don_paid_hybrids_attack_grants_independent_additive_and_consumed_only_by_noncreature()
{
    for p in [Player::P0, Player::P1] {
        for alternative in 0..3 {
            let mut g = setup(p);
            let don = named(&mut g, p, "Don & Raph, Hard Science", Zone::Hand);
            g.state.players[p.index()].mana = match alternative {
                0 => [0, 0, 0, 2, 0, 1],
                1 => [0, 1, 0, 1, 0, 1],
                _ => [0, 2, 0, 0, 0, 1],
            };
            checked(&mut g, Action::AnnounceCast { card: don });
            if matches!(g.state.pending, Pending::SpellCost { .. }) {
                let action = g
                    .legal_actions()
                    .into_iter()
                    .find(|a| matches!(a, Action::ChooseSpellCost { .. }))
                    .unwrap();
                checked(&mut g, action);
            }
            finish(&mut g);
            settle(&mut g);
            mature(&mut g);
            assert!(g.keywords(don).contains(Keywords::MENACE));
            named(
                &mut g,
                p,
                "@token/283fd83a9aec83a51b736926",
                Zone::Battlefield,
            );
            named(
                &mut g,
                p,
                "@token/283fd83a9aec83a51b736926",
                Zone::Battlefield,
            );
            attack(&mut g, don);
            settle(&mut g);
            assert_eq!(g.observe(p).next_spell_affinities.len(), 1);
            // A second resolved attack packet is an independent affinity instance.
            let view = g.object_view(don);
            assert!(g.queue_trigger(&view, 0));
            settle(&mut g);
            assert_eq!(g.observe(p).next_spell_affinities.len(), 2);
            g.move_object(don, Zone::Exile);
            to_step(&mut g, Step::Main2);
            refill(&mut g);
            paid(&mut g, p, "Grizzly Bears");
            settle(&mut g);
            assert_eq!(g.observe(p).next_spell_affinities.len(), 2);
            let archive = named(&mut g, p, "Hedron Archive", Zone::Hand);
            g.state.players[p.index()].mana = [0; 6];
            let snapshot = g.snapshot();
            assert!(
                g.legal_actions()
                    .contains(&Action::AnnounceCast { card: archive })
            );
            assert_eq!(snapshot, g.snapshot());
            checked(&mut g, Action::AnnounceCast { card: archive });
            assert_eq!(g.observe(p).next_spell_affinities.len(), 2);
            finish(&mut g);
            assert!(g.observe(p).next_spell_affinities.is_empty());
            settle(&mut g);
            assert!(g.on_battlefield(archive));
        }
    }
}
#[test]
fn tmt_final_don_grants_expire_and_copies_do_not_consume() {
    let p = Player::P0;
    let mut g = setup(p);
    let don = paid(&mut g, p, "Don & Raph, Hard Science");
    settle(&mut g);
    mature(&mut g);
    attack(&mut g, don);
    settle(&mut g);
    let source = g.object_view(don);
    let before = g.observe(p).next_spell_affinities.len();
    // A copied stack object is not a cast event and never consumes a grant.
    g.stage_copy_creation(p, CopyRecipe::new(source.card), 1);
    settle(&mut g);
    assert_eq!(before, g.observe(p).next_spell_affinities.len());
    next_turn(&mut g);
    assert!(g.observe(p).next_spell_affinities.is_empty());
}
#[test]
fn tmt_final_krang_intrinsic_affinity_does_not_reduce_later_spells() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        for _ in 0..6 {
            named(&mut g, p, "Hedron Archive", Zone::Battlefield);
        }
        g.state.players[p.index()].mana = [0, 2, 0, 0, 0, 0];
        let krang = paid(&mut g, p, "Krang, Master Mind");
        settle(&mut g);
        let archive = named(&mut g, p, "Hedron Archive", Zone::Hand);
        assert_eq!(g.spell_reduction(p, archive), 0);
        g.state.players[p.index()].mana = [4, 0, 0, 0, 0, 0];
        checked(&mut g, Action::AnnounceCast { card: archive });
        finish(&mut g);
        assert_eq!(g.state.players[p.index()].mana, [0; 6]);
        settle(&mut g);
        assert!(g.on_battlefield(archive));
        assert_eq!(g.stats(krang).power, 8);
    }
}
#[test]
fn tmt_final_krang_affinity_other_artifact_power_and_hand_intervening_if() {
    for p in [Player::P0, Player::P1] {
        for hand in [0, 2, 4, 6] {
            let mut g = setup(p);
            for _ in 0..6 {
                named(
                    &mut g,
                    p,
                    "@token/283fd83a9aec83a51b736926",
                    Zone::Battlefield,
                );
            }
            for _ in 0..hand {
                named(&mut g, p, "Plains", Zone::Hand);
            }
            g.state.players[p.index()].mana = [0, 2, 0, 0, 0, 0];
            let krang = paid(&mut g, p, "Krang, Master Mind");
            settle(&mut g);
            assert_eq!(g.hand_count(p), hand.max(4));
            assert_eq!(
                g.stats(krang),
                Stats {
                    power: 7,
                    toughness: 4
                }
            );
            let artifact = named(&mut g, p.other(), "Hedron Archive", Zone::Battlefield);
            assert_eq!(g.stats(krang).power, 7);
            let own = named(&mut g, p, "Hedron Archive", Zone::Battlefield);
            assert_eq!(g.stats(krang).power, 8);
            g.move_object(own, Zone::Graveyard);
            g.move_object(artifact, Zone::Graveyard);
            assert_eq!(g.stats(krang).power, 7);
        }
    }
}
#[test]
fn tmt_final_krang_entry_guard_rechecks_and_draws_current_difference() {
    for added in [1, 4] {
        let p = Player::P0;
        let mut g = setup(p);
        let krang = paid(&mut g, p, "Krang, Master Mind");
        until(&mut g, |g| {
            g.on_battlefield(krang) && g.state.stack.iter().any(|s| s.card == krang)
        });
        for _ in 0..added {
            named(&mut g, p, "Island", Zone::Hand);
        }
        settle(&mut g);
        assert_eq!(g.hand_count(p), 4);
    }
    let p = Player::P0;
    let mut g = setup(p);
    for _ in 0..4 {
        named(&mut g, p, "Plains", Zone::Hand);
    }
    let krang = paid(&mut g, p, "Krang, Master Mind");
    until(&mut g, |g| g.on_battlefield(krang));
    assert!(!g.state.stack.iter().any(|s| s.card == krang));
}
#[test]
fn tmt_final_rat_revolt_owned_end_only_and_source_departure() {
    for leave in [false, true] {
        let p = Player::P0;
        let mut g = setup(p);
        let king = paid(&mut g, p, "Rat King, Verminister");
        settle(&mut g);
        if leave {
            let land = named(&mut g, p, "Plains", Zone::Battlefield);
            g.move_object(land, Zone::Hand);
        }
        to_step(&mut g, Step::End);
        settle(&mut g);
        assert_eq!(
            g.counter_count(king, CounterKind::PlusOne),
            u64::from(leave)
        );
        let rats = g
            .state
            .objects
            .iter()
            .filter(|o| o.zone == Zone::Battlefield && o.card.has_subtype("Rat"))
            .count();
        assert_eq!(rats, if leave { 2 } else { 1 });
    }
    let p = Player::P0;
    let mut g = setup(p);
    let king = paid(&mut g, p, "Rat King, Verminister");
    settle(&mut g);
    let land = named(&mut g, p, "Plains", Zone::Battlefield);
    g.move_object(land, Zone::Hand);
    to_step(&mut g, Step::End);
    until(&mut g, |g| g.state.stack.iter().any(|s| s.card == king));
    g.move_object(king, Zone::Exile);
    settle(&mut g);
    assert_eq!(
        g.state
            .objects
            .iter()
            .filter(|o| o.zone == Zone::Battlefield && o.card.has_subtype("Rat"))
            .count(),
        1
    );
}
fn rat(g: &mut Game, p: Player) -> CardId {
    named(g, p, "@token/5bdb8e6c4e82e2817e0da137", Zone::Battlefield)
}
#[test]
fn tmt_final_rat_three_simultaneous_sacrifices_include_source_return_all_names_tapped() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let king = paid(&mut g, p, "Rat King, Verminister");
        settle(&mut g);
        mature(&mut g);
        let a = rat(&mut g, p);
        let b = rat(&mut g, p);
        let bad = rat(&mut g, p.other());
        let target = named(&mut g, p, "Grizzly Bears", Zone::Graveyard);
        let twin = named(&mut g, p, "Grizzly Bears", Zone::Graveyard);
        let foreign = named(&mut g, p.other(), "Grizzly Bears", Zone::Graveyard);
        let other = named(&mut g, p, "Hill Giant", Zone::Graveyard);
        activate(&mut g, king);
        checked(
            &mut g,
            Action::ChooseTarget {
                target: Target::Card(target),
            },
        );
        if g.legal_actions().contains(&Action::FinishTargets) {
            checked(&mut g, Action::FinishTargets);
        }
        let badref = g.object_ref(bad);
        reject(&mut g, Action::ChooseAdditionalCost { object: badref });
        for id in [king, a, b] {
            let object = g.object_ref(id);
            checked(&mut g, Action::ChooseAdditionalCost { object });
            if id != b {
                reject(&mut g, Action::ChooseAdditionalCost { object });
            }
        }
        finish(&mut g);
        assert_eq!(g.state.objects[king as usize].zone, Zone::Graveyard);
        assert_eq!(g.state.objects[a as usize].zone, Zone::Outside);
        assert_eq!(g.state.objects[b as usize].zone, Zone::Outside);
        settle(&mut g);
        for id in [target, twin] {
            assert!(g.on_battlefield(id));
            assert!(g.state.objects[id as usize].tapped);
        }
        for id in [foreign, other] {
            assert_eq!(g.state.objects[id as usize].zone, Zone::Graveyard);
        }
    }
}
#[test]
fn tmt_final_rat_insufficient_group_target_and_new_incarnation_rejected() {
    let p = Player::P0;
    let mut g = setup(p);
    let king = paid(&mut g, p, "Rat King, Verminister");
    settle(&mut g);
    mature(&mut g);
    rat(&mut g, p);
    let target = named(&mut g, p, "Grizzly Bears", Zone::Graveyard);
    reject(
        &mut g,
        Action::AnnounceActivate {
            card: king,
            ability: 0,
            counter_payment: Counters::default(),
        },
    );
    rat(&mut g, p);
    activate(&mut g, king);
    checked(
        &mut g,
        Action::ChooseTarget {
            target: Target::Card(target),
        },
    );
    if g.legal_actions().contains(&Action::FinishTargets) {
        checked(&mut g, Action::FinishTargets);
    }
    for _ in 0..3 {
        let action = g
            .legal_actions()
            .into_iter()
            .find(|a| matches!(a, Action::ChooseAdditionalCost { .. }))
            .unwrap();
        checked(&mut g, action);
    }
    finish(&mut g);
    g.move_object(target, Zone::Exile);
    g.move_object(target, Zone::Graveyard);
    settle(&mut g);
    assert_eq!(g.state.objects[target as usize].zone, Zone::Graveyard);
}
#[test]
fn tmt_final_cloning_all_chapters_nonlegendary_mutant_copies_and_copy_of_copy() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let original = named(&mut g, p, "Don & Raph, Hard Science", Zone::Graveyard);
        let definition = g.state.objects[original as usize].card;
        let saga = paid(&mut g, p, "The Cloning of Shredder");
        settle_on(&mut g, Some(Target::Card(original)));
        assert_eq!(g.state.objects[original as usize].zone, Zone::Exile);
        assert_eq!(copied(&g, definition).len(), 1);
        for n in 2..=3 {
            lore(&mut g, saga);
            settle(&mut g);
            assert_eq!(copied(&g, definition).len(), n);
        }
        assert_eq!(g.state.objects[saga as usize].zone, Zone::Graveyard);
        for id in copied(&g, definition) {
            let view = g.object_view(id);
            assert!(!view.legendary);
            assert!(g.has_subtype(id, "Mutant"));
            assert!(g.has_subtype(id, "Turtle"));
            assert!(g.keywords(id).contains(Keywords::MENACE));
        }
        let source = copied(&g, definition)[0];
        let aura = named(&mut g, p, "Witness Protection", Zone::Hand);
        cast(&mut g, aura, None, &[Target::Card(source)]);
        settle(&mut g);
        assert!(!g.object_view(source).legendary);
        assert!(!g.has_subtype(source, "Mutant"));
        assert!(!g.keywords(source).contains(Keywords::MENACE));
        let reflection = named(&mut g, p, "Self-Reflection", Zone::Hand);
        cast(&mut g, reflection, None, &[Target::Card(source)]);
        settle(&mut g);
        assert_eq!(copied(&g, definition).len(), 4);
        let copy = *copied(&g, definition).last().unwrap();
        assert!(!g.object_view(copy).legendary);
        assert!(g.has_subtype(copy, "Mutant"));
    }
}
#[test]
fn tmt_final_cloning_departed_source_keeps_link_but_returned_source_cannot_reuse() {
    let p = Player::P0;
    let mut g = setup(p);
    let original = named(&mut g, p, "Grizzly Bears", Zone::Graveyard);
    let def = g.state.objects[original as usize].card;
    let saga = paid(&mut g, p, "The Cloning of Shredder");
    settle_on(&mut g, Some(Target::Card(original)));
    lore(&mut g, saga);
    g.move_object(saga, Zone::Exile);
    settle(&mut g);
    assert_eq!(copied(&g, def).len(), 2);
    g.move_object(saga, Zone::Battlefield);
    settle(&mut g);
    lore(&mut g, saga);
    settle(&mut g);
    assert_eq!(copied(&g, def).len(), 2);
}
#[test]
fn tmt_final_cloning_departed_exile_card_cannot_be_copied_by_later_chapter() {
    let p = Player::P0;
    let mut g = setup(p);
    let original = named(&mut g, p, "Grizzly Bears", Zone::Graveyard);
    let def = g.state.objects[original as usize].card;
    let saga = paid(&mut g, p, "The Cloning of Shredder");
    settle_on(&mut g, Some(Target::Card(original)));
    g.move_object(original, Zone::Hand);
    g.move_object(original, Zone::Exile);
    lore(&mut g, saga);
    settle(&mut g);
    assert_eq!(copied(&g, def).len(), 1);
}
#[test]
fn tmt_final_cloning_multiple_links_choose_exact_card_and_reject_foreign() {
    let p = Player::P0;
    let mut g = setup(p);
    let first = named(&mut g, p, "Grizzly Bears", Zone::Graveyard);
    let second = named(&mut g, p, "Hill Giant", Zone::Graveyard);
    let bad = named(&mut g, p.other(), "Aegis Turtle", Zone::Exile);
    let saga = paid(&mut g, p, "The Cloning of Shredder");
    settle_on(&mut g, Some(Target::Card(first)));
    g.state.objects[saga as usize].counters.lore = 0;
    lore(&mut g, saga);
    settle_on(&mut g, Some(Target::Card(second)));
    lore(&mut g, saga);
    until(&mut g, |g| {
        g.state.resolving.as_ref().is_some_and(|r| {
            r.choice
                .as_ref()
                .is_some_and(|c| matches!(c.kind, EffectChoiceKind::CopyLinkedExile(_)))
        })
    });
    let badref = g.object_ref(bad);
    reject(&mut g, Action::ChooseCreature { object: badref });
    let object = g.object_ref(first);
    checked(&mut g, Action::ChooseCreature { object });
    settle(&mut g);
    assert_eq!(copied(&g, g.state.objects[first as usize].card).len(), 2);
    assert_eq!(copied(&g, g.state.objects[second as usize].card).len(), 1);
}
#[test]
fn tmt_final_ronin_destroy_all_preserves_indestructible_then_mill_reflexive_new_targets() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let own = named(&mut g, p, "Grizzly Bears", Zone::Battlefield);
        let enemy = named(&mut g, p.other(), "Hill Giant", Zone::Battlefield);
        let survivor = named(&mut g, p.other(), "Aegis Turtle", Zone::Battlefield);
        let r = g.object_ref(survivor);
        g.grant_keyword_effects(&[survivor], r, Keywords::INDESTRUCTIBLE);
        let milled = named(&mut g, p, "Gigantosaurus", Zone::Library);
        let saga = paid(&mut g, p, "The Last Ronin");
        settle(&mut g);
        for id in [own, enemy] {
            assert_eq!(g.state.objects[id as usize].zone, Zone::Graveyard);
        }
        assert!(g.on_battlefield(survivor));
        let before = g.state.players[p.index()].library.len();
        lore(&mut g, saga);
        until(&mut g, |g| {
            g.legal_actions().contains(&Action::ChooseTarget {
                target: Target::Card(milled),
            })
        });
        assert_eq!(g.state.players[p.index()].library.len(), before - 4);
        assert_eq!(g.state.objects[milled as usize].zone, Zone::Graveyard);
        let bad = named(&mut g, p.other(), "Gigantosaurus", Zone::Graveyard);
        reject(
            &mut g,
            Action::ChooseTarget {
                target: Target::Card(bad),
            },
        );
        checked(
            &mut g,
            Action::ChooseTarget {
                target: Target::Card(milled),
            },
        );
        if g.legal_actions().contains(&Action::FinishTargets) {
            checked(&mut g, Action::FinishTargets);
        }
        settle(&mut g);
        assert_eq!(g.state.objects[milled as usize].zone, Zone::Hand);
    }
}
fn ronin_observer(g: &mut Game, p: Player) -> CardId {
    let saga = paid(g, p, "The Last Ronin");
    settle(g);
    lore(g, saga);
    settle(g);
    lore(g, saga);
    settle(g);
    assert_eq!(g.state.objects[saga as usize].zone, Zone::Graveyard);
    saga
}
#[test]
fn tmt_final_ronin_delayed_attack_alone_outlives_saga_repeats_and_expires() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        ronin_observer(&mut g, p);
        let bear = named(&mut g, p, "Grizzly Bears", Zone::Battlefield);
        attack(&mut g, bear);
        settle(&mut g);
        assert_eq!(g.counter_count(bear, CounterKind::PlusOne), 3);
        assert!(
            g.keywords(bear).contains(
                Keywords::TRAMPLE
                    .union(Keywords::LIFELINK)
                    .union(Keywords::INDESTRUCTIBLE)
            )
        );
        // A later combat still observes independently; creature-action dispatch receives the complete declaration.
        g.creature_action_triggers(p, &[bear], CreatureActionKind::Attack);
        settle(&mut g);
        assert_eq!(g.counter_count(bear, CounterKind::PlusOne), 6);
        let other = named(&mut g, p, "Hill Giant", Zone::Battlefield);
        g.creature_action_triggers(p, &[bear, other], CreatureActionKind::Attack);
        settle(&mut g);
        assert_eq!(g.counter_count(bear, CounterKind::PlusOne), 6);
        g.creature_action_triggers(p.other(), &[bear], CreatureActionKind::Attack);
        settle(&mut g);
        assert_eq!(g.counter_count(bear, CounterKind::PlusOne), 6);
        next_turn(&mut g);
        assert!(g.observe(p).delayed_attack_alone.is_empty());
        assert!(!g.keywords(bear).contains(Keywords::INDESTRUCTIBLE));
        assert_eq!(g.counter_count(bear, CounterKind::PlusOne), 6);
    }
}
#[test]
fn tmt_final_ronin_delayed_exact_attacker_does_not_rebind_returned_card() {
    let p = Player::P0;
    let mut g = setup(p);
    ronin_observer(&mut g, p);
    let bear = named(&mut g, p, "Grizzly Bears", Zone::Battlefield);
    attack(&mut g, bear);
    g.move_object(bear, Zone::Exile);
    g.move_object(bear, Zone::Battlefield);
    settle(&mut g);
    assert_eq!(g.counter_count(bear, CounterKind::PlusOne), 0);
    assert!(!g.keywords(bear).contains(Keywords::INDESTRUCTIBLE));
}
fn crew(g: &mut Game, van: CardId, crewer: CardId) {
    activate(g, van);
    let object = g.object_ref(crewer);
    checked(g, Action::ChooseCrewer { object });
    checked(g, Action::FinishCrew);
    settle(g);
}
#[test]
fn tmt_final_van_crew_one_attack_only_exact_crewer_double_plus_one_only() {
    for p in [Player::P0, Player::P1] {
        for mutant in [false, true] {
            let mut g = setup(p);
            let van = paid(&mut g, p, "Turtle Van");
            settle(&mut g);
            mature(&mut g);
            let bear = named(&mut g, p, "Grizzly Bears", Zone::Battlefield);
            if mutant {
                all_types(&mut g, bear);
            }
            let bad = named(&mut g, p, "Hill Giant", Zone::Battlefield);
            g.add_counter(bear, CounterKind::PlusOne, 2);
            g.add_counter(bear, CounterKind::Stun, 1);
            crew(&mut g, van, bear);
            assert!(g.is_creature(van));
            assert_eq!(
                g.stats(van),
                Stats {
                    power: 4,
                    toughness: 4
                }
            );
            attack(&mut g, van);
            until(&mut g, |g| {
                g.legal_actions().contains(&Action::ChooseTarget {
                    target: Target::Card(bear),
                })
            });
            reject(
                &mut g,
                Action::ChooseTarget {
                    target: Target::Card(bad),
                },
            );
            checked(
                &mut g,
                Action::ChooseTarget {
                    target: Target::Card(bear),
                },
            );
            if g.legal_actions().contains(&Action::FinishTargets) {
                checked(&mut g, Action::FinishTargets);
            }
            settle(&mut g);
            assert_eq!(
                g.counter_count(bear, CounterKind::PlusOne),
                if mutant { 6 } else { 3 }
            );
            assert_eq!(g.counter_count(bear, CounterKind::Stun), 1);
        }
    }
}
#[test]
fn tmt_final_van_crewer_control_change_is_legal_but_new_incarnation_is_not() {
    let p = Player::P0;
    let mut g = setup(p);
    let van = paid(&mut g, p, "Turtle Van");
    settle(&mut g);
    mature(&mut g);
    let bear = named(&mut g, p, "Grizzly Bears", Zone::Battlefield);
    crew(&mut g, van, bear);
    let transfer = g.activation_item(p, van, 0);
    g.gain_control(&transfer, Subject::Targets, PlayerSet::Opponent);
    g.state.objects[bear as usize].controller = p.other();
    attack(&mut g, van);
    until(&mut g, |g| {
        g.legal_actions().contains(&Action::ChooseTarget {
            target: Target::Card(bear),
        })
    });
    checked(
        &mut g,
        Action::ChooseTarget {
            target: Target::Card(bear),
        },
    );
    if g.legal_actions().contains(&Action::FinishTargets) {
        checked(&mut g, Action::FinishTargets);
    }
    g.move_object(bear, Zone::Exile);
    g.move_object(bear, Zone::Battlefield);
    settle(&mut g);
    assert_eq!(g.counter_count(bear, CounterKind::PlusOne), 0);
}
fn mixed_setup(p: Player) -> Game {
    let names = [
        "Don & Raph, Hard Science",
        "Krang, Master Mind",
        "Rat King, Verminister",
        "Splinter, Radical Rat",
    ];
    let config = Config {
        decks: [vec![Card::Forest; 60], vec![Card::Forest; 60]],
        seed: 83,
        starting_player: p,
        deck_rules: DeckRules::Freeform,
    };
    let mut sides = [Vec::new(), Vec::new()];
    sides[p.index()] = names
        .into_iter()
        .map(|s| Card::from_name(s).unwrap())
        .collect();
    sides[p.other().index()] = vec![Card::from_name("Savanti Romero, Time's Exile").unwrap()];
    let mut g = Game::new_with_sideboards(config, sides).unwrap();
    for ps in &mut g.state.players {
        ps.kept = true;
        ps.mana = [30; 6];
    }
    g.state.active = p;
    g.state.priority = p;
    g.state.step = Step::Main1;
    g.state.pending = Pending::Priority;
    g
}
#[test]
fn tmt_final_forever_partial_zero_through_four_mixed_search_private_public_and_exact_hand() {
    for p in [Player::P0, Player::P1] {
        for found_count in 0..=4 {
            let mut g = mixed_setup(p);
            let pool: Vec<_> = g.outside_game_cards(p).collect();
            let library = pool[0];
            g.move_object(library.card, Zone::Library);
            let library = g.object_ref(library.card);
            let mut candidates = vec![library];
            candidates.extend(pool.iter().skip(1).copied());
            let duplicate = named(&mut g, p, "Don & Raph, Hard Science", Zone::Library);
            let bad = named(&mut g, p, "Grizzly Bears", Zone::Library);
            let foreign = g.outside_game_cards(p.other()).next().unwrap();
            let unregistered = named(&mut g, p, "Savanti Romero, Time's Exile", Zone::Outside);
            paid(&mut g, p, "Turtles Forever");
            until(&mut g, |g| {
                matches!(g.state.pending, Pending::LegendarySplitSearch { .. })
            });
            assert!(g.observe(p.other()).legendary_split_search.is_none());
            assert!(
                g.observe(p)
                    .legendary_split_search
                    .as_ref()
                    .unwrap()
                    .outside
                    .iter()
                    .all(|o| o.owner == p)
            );
            reject(&mut g, Action::ChooseSearch { card: bad });
            reject(&mut g, Action::ChooseSearch { card: foreign.card });
            reject(&mut g, Action::ChooseSearch { card: unregistered });
            for (i, object) in candidates.iter().take(found_count).enumerate() {
                checked(&mut g, Action::ChooseSearch { card: object.card });
                if i == 0 {
                    reject(&mut g, Action::ChooseSearch { card: duplicate });
                }
                assert!(g.observe(p.other()).legendary_split_search.is_none());
            }
            checked(&mut g, Action::FinishSearch);
            if found_count > 0 {
                assert_eq!(g.actor(), Some(p.other()));
                let view = g.observe(p.other()).legendary_split_search.unwrap();
                assert!(view.revealed);
                assert_eq!(view.found.len(), found_count);
                assert!(view.library.is_empty() && view.outside.is_empty());
                reject(&mut g, Action::FinishSearch);
                for object in candidates.iter().take(found_count.min(2)) {
                    checked(&mut g, Action::ChooseSearch { card: object.card });
                }
                checked(&mut g, Action::FinishSearch);
            }
            settle(&mut g);
            for (i, object) in candidates.iter().take(found_count).enumerate() {
                let expected = if i < 2 { Zone::Hand } else { Zone::Library };
                assert_eq!(g.state.objects[object.card as usize].zone, expected);
                if expected == Zone::Hand {
                    assert!(
                        g.observe(p.other())
                            .objects
                            .iter()
                            .any(|o| o.id == object.card && o.zone == Zone::Hand)
                    );
                } else {
                    assert!(
                        !g.observe(p.other())
                            .objects
                            .iter()
                            .any(|o| o.id == object.card && o.zone == Zone::Library)
                    );
                }
                assert!(!g.outside_game_card_matches(p, *object));
            }
            assert!(g.outside_game_card_matches(p.other(), foreign));
        }
    }
}

#[test]
fn tmt_final_forever_all_library_partial_zero_through_four_remainder_is_shuffled_in_place() {
    for p in [Player::P0, Player::P1] {
        for amount in 0..=4 {
            let mut g = mixed_setup(p);
            let pool: Vec<_> = g.outside_game_cards(p).collect();
            for object in &pool {
                g.move_object(object.card, Zone::Library);
            }
            paid(&mut g, p, "Turtles Forever");
            until(&mut g, |g| {
                matches!(g.state.pending, Pending::LegendarySplitSearch { .. })
            });
            for object in pool.iter().take(amount) {
                checked(&mut g, Action::ChooseSearch { card: object.card });
            }
            checked(&mut g, Action::FinishSearch);
            if amount > 0 {
                for object in pool.iter().take(amount.min(2)) {
                    checked(&mut g, Action::ChooseSearch { card: object.card });
                }
                checked(&mut g, Action::FinishSearch);
            }
            settle(&mut g);
            for (i, object) in pool.iter().take(amount).enumerate() {
                assert_eq!(
                    g.state.objects[object.card as usize].zone,
                    if i < 2 { Zone::Hand } else { Zone::Library }
                );
            }
            assert!(g.outside_game_cards(p).next().is_none());
        }
    }
}
#[test]
fn tmt_final_forever_ceased_tokens_and_unregistered_outside_are_never_found() {
    let p = Player::P0;
    let mut g = mixed_setup(p);
    let card = Card::from_name("Don & Raph, Hard Science").unwrap();
    g.stage_copy_creation(p, CopyRecipe::new(card), 1);
    settle(&mut g);
    let token = copied(&g, card)[0];
    g.move_object(token, Zone::Graveyard);
    checked(&mut g, Action::Pass);
    settle(&mut g);
    assert_eq!(g.state.objects[token as usize].zone, Zone::Outside);
    while g.actor() != Some(p) {
        checked(&mut g, Action::Pass);
    }
    refill(&mut g);
    paid(&mut g, p, "Turtles Forever");
    until(&mut g, |g| {
        matches!(g.state.pending, Pending::LegendarySplitSearch { .. })
    });
    reject(&mut g, Action::ChooseSearch { card: token });
    checked(&mut g, Action::FinishSearch);
    settle(&mut g);
}
#[test]
fn tmt_final_rat_group_funding_cannot_spend_the_third_rat_for_mana() {
    for extra in [false, true] {
        let p = Player::P0;
        let mut g = setup(p);
        let king = paid(&mut g, p, "Rat King, Verminister");
        settle(&mut g);
        mature(&mut g);
        rat(&mut g, p);
        let treasure = named(
            &mut g,
            p,
            "@token/b60f6cd3b44b22bd289fe4e2",
            Zone::Battlefield,
        );
        let item = g.activation_item(p, treasure, 0);
        g.animate_source(
            &item,
            crate::cards::AnimationSpec {
                artifact: true,
                creature_types: crate::cards::AnimationCreatureTypes::All,
                base: Some((1, 1)),
                keywords: Keywords::NONE,
                duration: crate::cards::AnimationDuration::Permanent,
            },
        );
        assert!(g.has_subtype(treasure, "Rat"));
        if extra {
            rat(&mut g, p);
        }
        g.state.players[p.index()].mana = [0; 6];
        let goal = FundingGoal {
            actor: p,
            required: [0; 6],
            generic: 1,
            reserve: None,
            source: Some((
                g.object_ref(king),
                0,
                Counters::default(),
                g.state.objects[king as usize].card,
            )),
        };
        let snap = g.snapshot();
        assert_eq!(g.can_fund(goal), extra);
        assert_eq!(g.snapshot(), snap);
    }
}
#[test]
fn tmt_final_don_successful_cast_consumes_grant_even_if_later_countered() {
    let p = Player::P0;
    let mut g = setup(p);
    let don = paid(&mut g, p, "Don & Raph, Hard Science");
    settle(&mut g);
    mature(&mut g);
    attack(&mut g, don);
    settle(&mut g);
    to_step(&mut g, Step::Main2);
    refill(&mut g);
    let archive = paid(&mut g, p, "Hedron Archive");
    assert!(g.observe(p).next_spell_affinities.is_empty());
    // Countering an already committed spell never refunds its cast-time grant.
    let packet = g.state.stack.pop().unwrap();
    assert_eq!(packet.card, archive);
    g.move_object(archive, Zone::Graveyard);
    settle(&mut g);
    assert!(g.observe(p).next_spell_affinities.is_empty());
}
#[test]
fn tmt_final_cloning_illegal_first_target_does_not_exile_or_create_and_other_chapters_do_nothing() {
    let p = Player::P0;
    let mut g = setup(p);
    let first = named(&mut g, p, "Grizzly Bears", Zone::Graveyard);
    let def = g.state.objects[first as usize].card;
    let saga = paid(&mut g, p, "The Cloning of Shredder");
    until(&mut g, |g| {
        g.legal_actions().contains(&Action::ChooseTarget {
            target: Target::Card(first),
        })
    });
    checked(
        &mut g,
        Action::ChooseTarget {
            target: Target::Card(first),
        },
    );
    if g.legal_actions().contains(&Action::FinishTargets) {
        checked(&mut g, Action::FinishTargets);
    }
    g.move_object(first, Zone::Hand);
    settle(&mut g);
    assert!(copied(&g, def).is_empty());
    lore(&mut g, saga);
    settle(&mut g);
    assert!(copied(&g, def).is_empty());
}
#[test]
fn tmt_final_ronin_reflexive_target_can_be_removed_before_resolution() {
    let p = Player::P0;
    let mut g = setup(p);
    let target = named(&mut g, p, "Grizzly Bears", Zone::Graveyard);
    let saga = paid(&mut g, p, "The Last Ronin");
    settle(&mut g);
    lore(&mut g, saga);
    until(&mut g, |g| {
        g.legal_actions().contains(&Action::ChooseTarget {
            target: Target::Card(target),
        })
    });
    checked(
        &mut g,
        Action::ChooseTarget {
            target: Target::Card(target),
        },
    );
    if g.legal_actions().contains(&Action::FinishTargets) {
        checked(&mut g, Action::FinishTargets);
    }
    assert!(!g.state.stack.is_empty());
    g.move_object(target, Zone::Exile);
    settle(&mut g);
    assert_eq!(g.state.objects[target as usize].zone, Zone::Exile);
}
#[test]
fn tmt_final_van_countered_crew_still_records_exact_payment_and_history_expires() {
    let p = Player::P0;
    let mut g = setup(p);
    let van = paid(&mut g, p, "Turtle Van");
    settle(&mut g);
    mature(&mut g);
    let first = named(&mut g, p, "Grizzly Bears", Zone::Battlefield);
    activate(&mut g, van);
    let object = g.object_ref(first);
    checked(&mut g, Action::ChooseCrewer { object });
    checked(&mut g, Action::FinishCrew);
    g.state.stack.pop().unwrap();
    settle(&mut g);
    assert!(!g.is_creature(van));
    let other = named(&mut g, p, "Hill Giant", Zone::Battlefield);
    crew(&mut g, van, other);
    attack(&mut g, van);
    until(&mut g, |g| {
        g.legal_actions().contains(&Action::ChooseTarget {
            target: Target::Card(first),
        })
    });
    checked(
        &mut g,
        Action::ChooseTarget {
            target: Target::Card(first),
        },
    );
    if g.legal_actions().contains(&Action::FinishTargets) {
        checked(&mut g, Action::FinishTargets);
    }
    settle(&mut g);
    assert_eq!(g.counter_count(first, CounterKind::PlusOne), 1);
    next_turn(&mut g);
    assert!(g.state.crew.as_ref().is_none_or(|s| s.history.is_empty()));
}

#[test]
fn tmt_final_don_announced_total_locks_before_artifacts_are_sacrificed_for_mana() {
    let p = Player::P0;
    let mut g = setup(p);
    let don = paid(&mut g, p, "Don & Raph, Hard Science");
    settle(&mut g);
    mature(&mut g);
    attack(&mut g, don);
    settle(&mut g);
    to_step(&mut g, Step::Main2);
    let factory = Card::catalogue()
        .find(|c| c.token_info().is_some_and(|t| t.name == "Treasure Token"))
        .unwrap();
    let first = super::tests::spawn(&mut g, p, factory, Zone::Battlefield);
    let second = super::tests::spawn(&mut g, p, factory, Zone::Battlefield);
    g.state.players[p.index()].mana = [0; 6];
    let archive = named(&mut g, p, "Hedron Archive", Zone::Hand);
    checked(&mut g, Action::AnnounceCast { card: archive });
    assert_eq!(g.observe(p).payment_request.unwrap().generic, 2);
    for id in [first, second] {
        let action = g
            .legal_actions()
            .into_iter()
            .find(|a| matches!(a,Action::ActivateMana{card,..}if *card==id))
            .unwrap();
        checked(&mut g, action);
        assert_eq!(g.observe(p).payment_request.unwrap().generic, 2);
    }
    finish(&mut g);
    assert!(g.observe(p).next_spell_affinities.is_empty());
    settle(&mut g);
    assert!(g.on_battlefield(archive));
}
#[test]
fn tmt_final_ronin_delayed_observer_is_not_a_ninja_permanent_trigger() {
    let p = Player::P0;
    let mut g = setup(p);
    ronin_observer(&mut g, p);
    let splinter = named(&mut g, p, "Splinter, Radical Rat", Zone::Battlefield);
    let bear = named(&mut g, p, "Grizzly Bears", Zone::Battlefield);
    all_types(&mut g, bear);
    attack(&mut g, bear);
    settle(&mut g);
    assert_eq!(g.counter_count(bear, CounterKind::PlusOne), 3);
    assert!(g.on_battlefield(splinter));
}

#[test]
fn tmt_final_van_counter_doubling_puts_additional_counters_with_replacements() {
    let p = Player::P0;
    let mut g = setup(p);
    let van = paid(&mut g, p, "Turtle Van");
    settle(&mut g);
    mature(&mut g);
    let bear = named(&mut g, p, "Grizzly Bears", Zone::Battlefield);
    all_types(&mut g, bear);
    g.add_counter(bear, CounterKind::PlusOne, 2);
    named(&mut g, p, "Doubling Season", Zone::Battlefield);
    crew(&mut g, van, bear);
    attack(&mut g, van);
    settle_on(&mut g, Some(Target::Card(bear)));
    assert_eq!(g.counter_count(bear, CounterKind::PlusOne), 12);
}
#[test]
fn tmt_final_rat_opponents_departures_and_end_step_never_qualify() {
    let p = Player::P0;
    let mut g = setup(p);
    let king = paid(&mut g, p, "Rat King, Verminister");
    settle(&mut g);
    let foreign = named(&mut g, p.other(), "Plains", Zone::Battlefield);
    g.move_object(foreign, Zone::Hand);
    to_step(&mut g, Step::End);
    settle(&mut g);
    assert_eq!(g.counter_count(king, CounterKind::PlusOne), 0);
    next_turn(&mut g);
    let own = named(&mut g, p, "Plains", Zone::Battlefield);
    g.move_object(own, Zone::Hand);
    to_step(&mut g, Step::End);
    settle(&mut g);
    assert_eq!(g.counter_count(king, CounterKind::PlusOne), 0);
}
