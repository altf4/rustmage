//! Frozen departure counters, permanent animation, and their complete consumers.
use super::ability_timeline_tests::{checked, named, settle, setup};
use super::*;
pub(super) fn candidate(g: &mut Game, p: Player, name: &str, zone: Zone) -> CardId {
    let definition = Card::from_name(name).expect("pinned candidate");
    if zone == Zone::Battlefield && definition.is_supported() {
        let id = super::tests::spawn(g, p, definition, Zone::Hand);
        cast_candidate(g, id, &[]);
        settle(g);
        // Battlefield fixtures model a permanent controlled since this turn
        // began. Reach that state through real turns after the paid cast.
        if definition.is_creature() {
            super::ability_timeline_tests::next_turn(g);
            super::ability_timeline_tests::next_turn(g);
            for _ in 0..8 {
                if g.state.step == Step::Main1 {
                    break;
                }
                checked(g, Action::Pass);
            }
            assert_eq!(g.state.step, Step::Main1);
        }
        for player in &mut g.state.players {
            player.mana = [30; 6];
        }
        id
    } else {
        super::tests::spawn(g, p, definition, zone)
    }
}
pub(super) fn reject(g: &mut Game, action: Action) {
    let before = g.snapshot();
    let revision = g.revision();
    assert!(g.apply(g.actor().unwrap(), action).is_err());
    assert_eq!(before, g.snapshot());
    assert_eq!(revision, g.revision());
}
fn finish(g: &mut Game) {
    let original = g.state.announcing.as_ref().map(|a| a.item.id);
    for _ in 0..12 {
        if g.state.announcing.as_ref().map(|a| a.item.id) != original
            || g.state.announcing.is_none()
        {
            return;
        }
        let action = g
            .legal_actions()
            .into_iter()
            .find(|a| {
                matches!(
                    a,
                    Action::PayCosts { .. }
                        | Action::FinishTargets
                        | Action::ChooseSpellCost { .. }
                        | Action::ChooseActivationCost { .. }
                        | Action::ChooseX { value: 0 }
                )
            })
            .unwrap_or_else(|| panic!("payment at {:?}", g.state.pending));
        checked(g, action);
    }
    panic!("unfinished payment");
}
pub(super) fn cast_candidate(g: &mut Game, id: CardId, targets: &[Target]) {
    if g.state.objects[id as usize].card.is_supported() {
        checked(g, Action::AnnounceCast { card: id });
    } else {
        // A gated definition cannot enter a user's deck. Seed only its proposal;
        // targets, mana, effects and all continuations still use checked actions.
        let item = g.cast_item(g.actor().unwrap(), id, [0; 6]);
        g.open_payment_announcement(item, Counters::default(), None);
    }
    for &target in targets {
        checked(g, Action::ChooseTarget { target });
    }
    finish(g);
}
pub(super) fn activate(g: &mut Game, id: CardId, targets: &[Target]) {
    checked(
        g,
        Action::AnnounceActivate {
            card: id,
            ability: 0,
            counter_payment: Counters::default(),
        },
    );
    for &target in targets {
        checked(g, Action::ChooseTarget { target });
    }
    finish(g);
}
pub(super) fn food(g: &mut Game, p: Player) -> CardId {
    named(g, p, "@token/283fd83a9aec83a51b736926", Zone::Battlefield)
}
pub(super) fn mutagens(g: &Game, p: Player) -> usize {
    g.state
        .objects
        .iter()
        .filter(|o| {
            o.zone == Zone::Battlefield && o.controller == p && o.card.has_subtype("Mutagen")
        })
        .count()
}
#[test]
fn tmt_departure_donatello_paid_sorcery_tap_targets_only_owned_artifacts_and_animates_after_counters()
 {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let don = candidate(&mut g, p, "Donatello, Mutant Mechanic", Zone::Battlefield);
        let artifact = food(&mut g, p);
        let enemy = food(&mut g, p.other());
        let bear = named(&mut g, p, "Grizzly Bears", Zone::Battlefield);
        checked(
            &mut g,
            Action::AnnounceActivate {
                card: don,
                ability: 0,
                counter_payment: Counters::default(),
            },
        );
        reject(
            &mut g,
            Action::ChooseTarget {
                target: Target::Card(enemy),
            },
        );
        reject(
            &mut g,
            Action::ChooseTarget {
                target: Target::Card(bear),
            },
        );
        checked(
            &mut g,
            Action::ChooseTarget {
                target: Target::Card(artifact),
            },
        );
        finish(&mut g);
        assert!(g.state.objects[don as usize].tapped);
        settle(&mut g);
        assert_eq!(g.counter_count(artifact, CounterKind::PlusOne), 3);
        assert_eq!(
            (g.stats(artifact).power, g.stats(artifact).toughness),
            (3, 3)
        );
        assert!(g.has_subtype(artifact, "Robot"));
        assert!(g.has_subtype(artifact, "Food"));
        assert!(g.current_kinds(artifact) & 17 == 17);
        reject(
            &mut g,
            Action::AnnounceActivate {
                card: don,
                ability: 0,
                counter_payment: Counters::default(),
            },
        );
    }
}
#[test]
fn tmt_departure_donatello_full_named_counter_snapshot_is_owned_and_doubled_normally() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let don = candidate(&mut g, p, "Donatello, Mutant Mechanic", Zone::Battlefield);
        let token = food(&mut g, p);
        let target = named(&mut g, p, "Grizzly Bears", Zone::Battlefield);
        named(&mut g, p, "Doubling Season", Zone::Battlefield);
        let mut counters = Counters::of(CounterKind::PlusOne, 2);
        for (kind, amount) in [
            (CounterKind::Stun, 1),
            (CounterKind::Charge, 3),
            (CounterKind::Lore, 1),
            (CounterKind::Loyalty, 2),
            (CounterKind::Flying, 1),
        ] {
            counters.add(kind, amount);
        }
        g.set_counters(token, counters.clone());
        activate(&mut g, token, &[]);
        assert!(matches!(g.state.pending, Pending::Targets { .. }));
        let packet = &g.state.announcing.as_ref().unwrap().item;
        assert_eq!(
            packet
                .death_event
                .as_ref()
                .unwrap()
                .counter_snapshot
                .as_deref(),
            Some(&counters)
        );
        assert!(g.state.objects[token as usize].zone != Zone::Battlefield);
        checked(
            &mut g,
            Action::ChooseTarget {
                target: Target::Card(target),
            },
        );
        finish(&mut g);
        g.move_object(don, Zone::Hand);
        settle(&mut g);
        for kind in CounterKind::ALL {
            assert_eq!(
                g.counter_count(target, kind),
                counters.get(kind) * 2,
                "{kind:?}"
            );
        }
        assert!(g.keywords(target).contains(Keywords::FLYING));
    }
}
#[test]
fn tmt_departure_ooze_counts_only_plus_one_on_owned_creatures_for_bounce_and_exile() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        candidate(&mut g, p, "The Ooze", Zone::Battlefield);
        let a = named(&mut g, p, "Grizzly Bears", Zone::Battlefield);
        let b = named(&mut g, p.other(), "Grizzly Bears", Zone::Battlefield);
        let artifact = food(&mut g, p);
        g.set_counters(a, Counters::of(CounterKind::PlusOne, 3));
        g.set_counters(b, Counters::of(CounterKind::PlusOne, 4));
        g.set_counters(artifact, Counters::of(CounterKind::PlusOne, 5));
        g.effect_counter(a, CounterKind::Charge, 7, replacements::ResumeMode::Program);
        let bounce = named(&mut g, p, "Into the Roil", Zone::Hand);
        checked(&mut g, Action::AnnounceCast { card: bounce });
        checked(&mut g, Action::ChooseKicker { pay: false });
        checked(
            &mut g,
            Action::ChooseTarget {
                target: Target::Card(a),
            },
        );
        finish(&mut g);
        settle(&mut g);
        assert_eq!(mutagens(&g, p), 3);
        g.move_object(b, Zone::Exile);
        g.move_object(artifact, Zone::Exile);
        settle(&mut g);
        assert_eq!(mutagens(&g, p), 3);
    }
}
#[test]
fn tmt_departure_ooze_paid_exile_from_either_graveyard_creates_mutagen_but_invalid_target_does_not()
{
    for p in [Player::P0, Player::P1] {
        for invalidate in [false, true] {
            let mut g = setup(p);
            let ooze = candidate(&mut g, p, "The Ooze", Zone::Battlefield);
            let card = named(&mut g, p.other(), "Opt", Zone::Graveyard);
            activate(&mut g, ooze, &[Target::Card(card)]);
            if invalidate {
                g.move_object(card, Zone::Hand);
            }
            g.move_object(ooze, Zone::Hand);
            settle(&mut g);
            assert_eq!(mutagens(&g, p), usize::from(!invalidate));
            assert_eq!(
                g.state.objects[card as usize].zone,
                if invalidate { Zone::Hand } else { Zone::Exile }
            );
        }
    }
}
#[test]
fn tmt_departure_super_shredder_sees_other_permanents_of_both_players_and_never_rebinds() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let shred = candidate(&mut g, p, "Super Shredder", Zone::Battlefield);
        let own = food(&mut g, p);
        let enemy = named(&mut g, p.other(), "Forest", Zone::Battlefield);
        g.move_object(own, Zone::Hand);
        g.move_object(enemy, Zone::Exile);
        settle(&mut g);
        assert_eq!(g.counter_count(shred, CounterKind::PlusOne), 2);
        assert!(g.keywords(shred).contains(Keywords::MENACE));
        let own = food(&mut g, p);
        g.move_object(own, Zone::Graveyard);
        g.prepare_triggers();
        let old = g.object_ref(shred);
        g.move_object(shred, Zone::Hand);
        g.move_object(shred, Zone::Battlefield);
        assert_ne!(old, g.object_ref(shred));
        settle(&mut g);
        assert_eq!(g.counter_count(shred, CounterKind::PlusOne), 0);
    }
}
#[test]
fn tmt_departure_mind_transfer_paid_instant_preserves_types_and_abilities_and_expires() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let creature = named(&mut g, p, "Serra Angel", Zone::Battlefield);
        let spell = candidate(&mut g, p, "Mind Transfer Protocol", Zone::Hand);
        let hand = g
            .state
            .objects
            .iter()
            .filter(|o| o.zone == Zone::Hand && o.owner == p)
            .count();
        cast_candidate(&mut g, spell, &[Target::Card(creature)]);
        settle(&mut g);
        assert_eq!(
            (g.stats(creature).power, g.stats(creature).toughness),
            (4, 5)
        );
        assert_eq!(g.current_kinds(creature) & 17, 17);
        assert!(g.has_subtype(creature, "Angel"));
        assert!(g.keywords(creature).contains(Keywords::FLYING));
        assert!(g.keywords(creature).contains(Keywords::VIGILANCE));
        assert_eq!(
            g.state
                .objects
                .iter()
                .filter(|o| o.zone == Zone::Hand && o.owner == p)
                .count(),
            hand
        );
        super::ability_timeline_tests::next_turn(&mut g);
        assert_eq!(
            (g.stats(creature).power, g.stats(creature).toughness),
            (4, 4)
        );
        assert_eq!(g.current_kinds(creature) & 16, 0);
    }
}
#[test]
fn tmt_departure_mind_transfer_one_invalid_target_never_draws() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let target = food(&mut g, p);
        let spell = candidate(&mut g, p, "Mind Transfer Protocol", Zone::Hand);
        cast_candidate(&mut g, spell, &[Target::Card(target)]);
        let before = g.state.players[p.index()].library.len();
        g.move_object(target, Zone::Hand);
        settle(&mut g);
        assert_eq!(g.state.players[p.index()].library.len(), before);
    }
}
#[test]
fn tmt_departure_pizza_paid_entry_creates_food_and_disappear_can_target_an_opponents_artifact_or_creature()
 {
    for p in [Player::P0, Player::P1] {
        for creature in [false, true] {
            let mut g = setup(p);
            let pizza = candidate(&mut g, p, "Pizza Face, Gastromancer", Zone::Hand);
            cast_candidate(&mut g, pizza, &[]);
            settle(&mut g);
            let own = g
                .state
                .objects
                .iter()
                .enumerate()
                .find(|(_, o)| {
                    o.zone == Zone::Battlefield
                        && o.controller == p
                        && o.is_token()
                        && o.card.has_subtype("Food")
                })
                .map(|(i, _)| i as CardId)
                .unwrap();
            let target = if creature {
                named(&mut g, p.other(), "Serra Angel", Zone::Battlefield)
            } else {
                food(&mut g, p.other())
            };
            activate(&mut g, own, &[]);
            settle(&mut g);
            g.state.step = Step::Main2;
            checked(&mut g, Action::Pass);
            checked(&mut g, Action::Pass);
            assert_eq!(g.state.step, Step::End);
            reject(
                &mut g,
                Action::ChooseTarget {
                    target: Target::Card(pizza),
                },
            );
            checked(
                &mut g,
                Action::ChooseTarget {
                    target: Target::Card(target),
                },
            );
            finish(&mut g);
            g.move_object(pizza, Zone::Hand);
            settle(&mut g);
            assert_eq!(g.counter_count(target, CounterKind::PlusOne), 3);
            assert_eq!(g.stats(target).power, if creature { 7 } else { 3 });
            assert_eq!(g.stats(target).toughness, if creature { 7 } else { 3 });
            assert_eq!(g.has_subtype(target, "Mutant"), !creature);
            if creature {
                assert!(g.has_subtype(target, "Angel"));
                assert!(g.keywords(target).contains(Keywords::FLYING));
            } else {
                assert!(g.has_subtype(target, "Food"));
                assert_eq!(g.current_kinds(target) & 17, 17);
            }
            super::ability_timeline_tests::next_turn(&mut g);
            assert_eq!(g.has_subtype(target, "Mutant"), !creature);
        }
    }
}
#[test]
fn tmt_departure_pizza_sacrifice_cost_is_ten_tap_and_sacrifice_for_fifteen_life() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let pizza = candidate(&mut g, p, "Pizza Face, Gastromancer", Zone::Hand);
        cast_candidate(&mut g, pizza, &[]);
        settle(&mut g);
        reject(
            &mut g,
            Action::AnnounceActivate {
                card: pizza,
                ability: 0,
                counter_payment: Counters::default(),
            },
        );
        super::ability_timeline_tests::next_turn(&mut g);
        super::ability_timeline_tests::next_turn(&mut g);
        g.state.players[p.index()].mana = [0, 0, 0, 0, 0, 9];
        reject(
            &mut g,
            Action::AnnounceActivate {
                card: pizza,
                ability: 0,
                counter_payment: Counters::default(),
            },
        );
        g.state.players[p.index()].mana = [0, 0, 0, 0, 0, 10];
        let before = g.state.players[p.index()].life;
        activate(&mut g, pizza, &[]);
        assert_eq!(g.state.objects[pizza as usize].zone, Zone::Graveyard);
        assert_eq!(g.state.players[p.index()].mana, [0; 6]);
        settle(&mut g);
        assert_eq!(g.state.players[p.index()].life, before + 15);
    }
}
#[test]
fn tmt_departure_pizza_optional_zero_target_adds_no_counters_or_animation() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let pizza = candidate(&mut g, p, "Pizza Face, Gastromancer", Zone::Battlefield);
        named(&mut g, p, "Grizzly Bears", Zone::Battlefield);
        let token = food(&mut g, p);
        activate(&mut g, token, &[]);
        settle(&mut g);
        g.state.step = Step::Main2;
        checked(&mut g, Action::Pass);
        checked(&mut g, Action::Pass);
        checked(&mut g, Action::FinishTargets);
        settle(&mut g);
        assert_eq!(g.counter_count(pizza, CounterKind::PlusOne), 0);
        assert_eq!((g.stats(pizza).power, g.stats(pizza).toughness), (2, 4));
    }
}
#[test]
fn tmt_departure_ooze_mutagen_is_sorcery_paid_one_tap_sacrifice_and_can_target_enemy_creature() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let ooze = candidate(&mut g, p, "The Ooze", Zone::Battlefield);
        let grave = named(&mut g, p, "Opt", Zone::Graveyard);
        let enemy = named(&mut g, p.other(), "Grizzly Bears", Zone::Battlefield);
        activate(&mut g, ooze, &[Target::Card(grave)]);
        settle(&mut g);
        let token = g
            .state
            .objects
            .iter()
            .enumerate()
            .find(|(_, o)| {
                o.zone == Zone::Battlefield && o.controller == p && o.card.has_subtype("Mutagen")
            })
            .map(|(i, _)| i as CardId)
            .unwrap();
        g.state.step = Step::BeginCombat;
        reject(
            &mut g,
            Action::AnnounceActivate {
                card: token,
                ability: 0,
                counter_payment: Counters::default(),
            },
        );
        g.state.step = Step::Main1;
        g.state.players[p.index()].mana = [0; 6];
        reject(
            &mut g,
            Action::AnnounceActivate {
                card: token,
                ability: 0,
                counter_payment: Counters::default(),
            },
        );
        g.state.players[p.index()].mana = [0, 0, 0, 0, 0, 1];
        activate(&mut g, token, &[Target::Card(enemy)]);
        settle(&mut g);
        assert_eq!(g.state.objects[token as usize].zone, Zone::Outside);
        assert_eq!(g.counter_count(enemy, CounterKind::PlusOne), 1);
        assert_eq!(g.state.players[p.index()].mana, [0; 6]);
    }
}
#[test]
fn tmt_departure_ooze_simultaneous_animated_source_and_creature_leave_freeze_individual_counts() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let ooze = candidate(&mut g, p, "The Ooze", Zone::Battlefield);
        let creature = named(&mut g, p, "Grizzly Bears", Zone::Battlefield);
        named(&mut g, p, "Doubling Season", Zone::Battlefield);
        let spell = candidate(&mut g, p, "Mind Transfer Protocol", Zone::Hand);
        cast_candidate(&mut g, spell, &[Target::Card(ooze)]);
        settle(&mut g);
        g.set_counters(ooze, Counters::of(CounterKind::PlusOne, 2));
        g.set_counters(creature, Counters::of(CounterKind::PlusOne, 1));
        let death = g.death_trigger_sources();
        let events = g.move_objects_raw(&[ooze, creature], Zone::Hand);
        g.finish_zone_events(&events, &death);
        settle(&mut g);
        assert_eq!(mutagens(&g, p), 6);
    }
}
#[test]
fn tmt_departure_donatello_simultaneous_artifact_source_death_preserves_each_snapshot() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let don = candidate(&mut g, p, "Donatello, Mutant Mechanic", Zone::Battlefield);
        let token = food(&mut g, p);
        let recipient = named(&mut g, p, "Aegis Turtle", Zone::Battlefield);
        let spell = candidate(&mut g, p, "Mind Transfer Protocol", Zone::Hand);
        cast_candidate(&mut g, spell, &[Target::Card(don)]);
        settle(&mut g);
        g.set_counters(don, Counters::of(CounterKind::PlusOne, 2));
        g.set_counters(token, Counters::of(CounterKind::Charge, 3));
        let death = g.death_trigger_sources();
        let events = g.move_objects_raw(&[don, token], Zone::Graveyard);
        g.finish_zone_events(&events, &death);
        super::ability_timeline_tests::settle_on(&mut g, Some(Target::Card(recipient)));
        assert_eq!(g.counter_count(recipient, CounterKind::PlusOne), 2);
        assert_eq!(g.counter_count(recipient, CounterKind::Charge), 3);
    }
}
#[test]
fn tmt_departure_donatello_counter_placement_overflow_rolls_back_the_entire_resolution() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        candidate(&mut g, p, "Donatello, Mutant Mechanic", Zone::Battlefield);
        let token = food(&mut g, p);
        let recipient = named(&mut g, p, "Aegis Turtle", Zone::Battlefield);
        g.set_counters(token, Counters::of(CounterKind::Charge, 1));
        g.set_counters(recipient, Counters::of(CounterKind::Charge, u64::MAX));
        activate(&mut g, token, &[]);
        checked(
            &mut g,
            Action::ChooseTarget {
                target: Target::Card(recipient),
            },
        );
        finish(&mut g);
        checked(&mut g, Action::Pass);
        let before = g.snapshot();
        let revision = g.revision();
        assert!(matches!(
            g.apply(g.actor().unwrap(), Action::Pass),
            Err(Error::CapacityExceeded(_))
        ));
        assert_eq!(before, g.snapshot());
        assert_eq!(revision, g.revision());
        assert_eq!(g.counter_count(recipient, CounterKind::Charge), u64::MAX);
    }
}
#[test]
fn tmt_departure_ooze_plus_one_finality_exile_triggers_but_donatello_requires_actual_graveyard() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let ooze = candidate(&mut g, p, "The Ooze", Zone::Battlefield);
        candidate(&mut g, p, "Donatello, Mutant Mechanic", Zone::Battlefield);
        let artifact = food(&mut g, p);
        let spell = candidate(&mut g, p, "Mind Transfer Protocol", Zone::Hand);
        cast_candidate(&mut g, spell, &[Target::Card(artifact)]);
        settle(&mut g);
        let mut counters = Counters::of(CounterKind::PlusOne, 2);
        counters.add(CounterKind::Finality, 1);
        g.set_counters(artifact, counters);
        activate(&mut g, artifact, &[]);
        assert!(
            g.state
                .pending_triggers
                .iter()
                .chain(&g.state.stack)
                .chain(&g.state.ordering_triggers)
                .chain(g.state.announcing.iter().map(|a| &a.item))
                .all(|i| i.source_card.name() != "Donatello, Mutant Mechanic")
        );
        settle(&mut g);
        assert_eq!(mutagens(&g, p), 2);
        assert!(g.on_battlefield(ooze));
    }
}
#[test]
fn tmt_departure_mind_transfer_animates_equipment_unattaches_and_preserves_existing_counters() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let bearer = named(&mut g, p, "Grizzly Bears", Zone::Battlefield);
        let equipment = named(&mut g, p, "Skateboard", Zone::Battlefield);
        activate(&mut g, equipment, &[Target::Card(bearer)]);
        settle(&mut g);
        assert_eq!(
            g.state.objects[equipment as usize].attached_to,
            Some(g.object_ref(bearer))
        );
        g.set_counters(equipment, Counters::of(CounterKind::PlusOne, 1));
        let spell = candidate(&mut g, p, "Mind Transfer Protocol", Zone::Hand);
        cast_candidate(&mut g, spell, &[Target::Card(equipment)]);
        settle(&mut g);
        assert_eq!(
            (g.stats(equipment).power, g.stats(equipment).toughness),
            (5, 6)
        );
        assert!(g.has_subtype(equipment, "Equipment"));
        assert!(g.state.objects[equipment as usize].attached_to.is_none());
        super::ability_timeline_tests::next_turn(&mut g);
        assert!(!g.is_creature(equipment));
        assert_eq!(g.counter_count(equipment, CounterKind::PlusOne), 1);
    }
}
#[test]
fn tmt_departure_donatello_existing_vehicle_crewed_in_response_keeps_printed_base_and_no_robot() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let don = candidate(&mut g, p, "Donatello, Mutant Mechanic", Zone::Battlefield);
        let vehicle = named(&mut g, p, "Turtle Blimp", Zone::Battlefield);
        let bear = named(&mut g, p, "Grizzly Bears", Zone::Battlefield);
        g.state.step = Step::BeginCombat;
        reject(
            &mut g,
            Action::AnnounceActivate {
                card: don,
                ability: 0,
                counter_payment: Counters::default(),
            },
        );
        g.state.step = Step::Main1;
        activate(&mut g, don, &[Target::Card(vehicle)]);
        checked(
            &mut g,
            Action::AnnounceActivate {
                card: vehicle,
                ability: 0,
                counter_payment: Counters::default(),
            },
        );
        let object = g.object_ref(bear);
        checked(&mut g, Action::ChooseCrewer { object });
        checked(&mut g, Action::FinishCrew);
        settle(&mut g);
        assert_eq!((g.stats(vehicle).power, g.stats(vehicle).toughness), (6, 7));
        assert!(!g.has_subtype(vehicle, "Robot"));
        assert!(g.has_subtype(vehicle, "Vehicle"));
        super::ability_timeline_tests::next_turn(&mut g);
        assert!(!g.is_creature(vehicle));
        assert_eq!(g.counter_count(vehicle, CounterKind::PlusOne), 3);
    }
}
#[test]
fn tmt_departure_donatello_can_choose_zero_and_does_not_follow_a_blinked_target() {
    for p in [Player::P0, Player::P1] {
        for zero in [false, true] {
            let mut g = setup(p);
            candidate(&mut g, p, "Donatello, Mutant Mechanic", Zone::Battlefield);
            let token = food(&mut g, p);
            let target = named(&mut g, p, "Grizzly Bears", Zone::Battlefield);
            g.set_counters(token, Counters::of(CounterKind::Charge, 2));
            activate(&mut g, token, &[]);
            if zero {
                checked(&mut g, Action::FinishTargets);
            } else {
                checked(
                    &mut g,
                    Action::ChooseTarget {
                        target: Target::Card(target),
                    },
                );
                finish(&mut g);
                g.move_object(target, Zone::Hand);
                g.move_object(target, Zone::Battlefield);
            }
            settle(&mut g);
            assert_eq!(g.counter_count(target, CounterKind::Charge), 0);
        }
    }
}
#[test]
fn tmt_departure_donatello_uses_controller_before_death_even_for_foreign_owned_token() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        candidate(&mut g, p, "Donatello, Mutant Mechanic", Zone::Battlefield);
        let token = food(&mut g, p.other());
        g.state.objects[token as usize].controller = p;
        g.set_counters(token, Counters::of(CounterKind::Charge, 2));
        let target = named(&mut g, p, "Grizzly Bears", Zone::Battlefield);
        activate(&mut g, token, &[]);
        checked(
            &mut g,
            Action::ChooseTarget {
                target: Target::Card(target),
            },
        );
        finish(&mut g);
        settle(&mut g);
        assert_eq!(g.counter_count(target, CounterKind::Charge), 2);
        assert_eq!(g.state.objects[token as usize].owner, p.other());
    }
}
#[test]
fn tmt_departure_super_shredder_has_actual_menace_blocking_requirements() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let shred = candidate(&mut g, p, "Super Shredder", Zone::Battlefield);
        let a = named(&mut g, p.other(), "Hill Giant", Zone::Battlefield);
        g.state.step = Step::Attackers;
        g.state.pending = Pending::Attackers;
        checked(&mut g, Action::Attack { card: shred });
        checked(&mut g, Action::FinishAttackers);
        checked(&mut g, Action::Pass);
        checked(&mut g, Action::Pass);
        reject(
            &mut g,
            Action::Block {
                blocker: a,
                attacker: shred,
            },
        );
        let b = named(&mut g, p.other(), "Hill Giant", Zone::Battlefield);
        checked(
            &mut g,
            Action::Block {
                blocker: a,
                attacker: shred,
            },
        );
        reject(&mut g, Action::FinishBlockers);
        checked(
            &mut g,
            Action::Block {
                blocker: b,
                attacker: shred,
            },
        );
        checked(&mut g, Action::FinishBlockers);
    }
}
#[test]
fn tmt_departure_donatello_copies_every_transferable_counter_kind_without_inventing_abilities() {
    for p in [Player::P0, Player::P1] {
        for kind in CounterKind::ALL {
            // A finality counter would replace this graveyard move with exile;
            // that actual exclusion is verified in the combined Ooze scenario.
            if kind == CounterKind::Finality {
                continue;
            }
            let mut g = setup(p);
            candidate(&mut g, p, "Donatello, Mutant Mechanic", Zone::Battlefield);
            let token = food(&mut g, p);
            let target = named(&mut g, p, "Aegis Turtle", Zone::Battlefield);
            g.set_counters(token, Counters::of(kind, 2));
            activate(&mut g, token, &[]);
            checked(
                &mut g,
                Action::ChooseTarget {
                    target: Target::Card(target),
                },
            );
            finish(&mut g);
            settle(&mut g);
            assert_eq!(g.counter_count(target, kind), 2, "{kind:?}");
        }
    }
}
