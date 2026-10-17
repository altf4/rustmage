//! Complete paid counter, entry, graveyard and sacrifice obligations, both seats.
use super::ability_timeline_tests::{cast, checked, named, settle};
fn setup(p: Player) -> Game {
    let mut g = super::ability_timeline_tests::setup(p);
    for player in &mut g.state.players {
        player.mana = [5; 6];
    }
    g
}
use super::*;

fn reject(g: &mut Game, action: Action) {
    let saved = g.snapshot();
    let revision = g.revision();
    assert!(g.apply(g.actor().unwrap(), action).is_err());
    assert_eq!(saved, g.snapshot());
    assert_eq!(revision, g.revision());
}
fn pay(g: &mut Game) {
    for _ in 0..8 {
        if g.state.announcing.is_none() {
            return;
        }
        let action = g
            .legal_actions()
            .into_iter()
            .find(|a| matches!(a, Action::FinishTargets | Action::PayCosts { .. }))
            .unwrap();
        checked(g, action);
    }
    panic!("unpaid");
}
fn activate(
    g: &mut Game,
    id: CardId,
    ability: u16,
    target: Option<Target>,
    sacrifice: Option<CardId>,
) {
    checked(
        g,
        Action::AnnounceActivate {
            card: id,
            ability,
            counter_payment: Counters::default(),
        },
    );
    if let Some(target) = target {
        checked(g, Action::ChooseTarget { target });
    }
    if let Some(id) = sacrifice {
        while !matches!(g.state.pending, Pending::AdditionalCost { .. }) {
            checked(g, Action::FinishTargets);
        }
        let source = g.object_ref(id);
        let own_source = g.object_ref(match g.state.announcing.as_ref() {
            Some(a) => a.item.card,
            None => panic!("announcement"),
        });
        reject(g, Action::ChooseAdditionalCost { object: own_source });
        let object = source;
        checked(g, Action::ChooseAdditionalCost { object });
    }
    pay(g);
}
fn until(g: &mut Game, stop: impl Fn(&Game) -> bool) {
    g.prepare_triggers();
    for _ in 0..100 {
        if stop(g) {
            return;
        }
        let a = if g.state.pending == Pending::Priority {
            Action::Pass
        } else {
            g.legal_actions()
                .into_iter()
                .find(|a| {
                    matches!(
                        a,
                        Action::PutTrigger { .. } | Action::FinishTargets | Action::PayCosts { .. }
                    )
                })
                .unwrap_or_else(|| panic!("{:?}", g.state.pending))
        };
        checked(g, a);
    }
    panic!("stalled");
}
fn cast_x(g: &mut Game, id: CardId, x: u32) {
    checked(g, Action::AnnounceCast { card: id });
    checked(g, Action::ChooseX { value: x });
    pay(g);
}
fn counter(g: &mut Game, id: CardId, kind: CounterKind, amount: u64, by: Player) {
    let view = g.object_view(id);
    let mut counters = Counters::default();
    counters.add(kind, amount);
    g.stage_counter_placements(
        vec![replacements::CounterPlacement {
            object: view.reference,
            placed_by: by,
            card: view.card,
            owner: view.owner,
            controller: view.controller,
            subtypes: view.granted_subtypes,
            kind: view.kinds,
            amount: counters,
            effect: true,
            entry_index: None,
            private: false,
        }],
        replacements::ResumeMode::Pending,
    );
}

#[test]
fn fdn_counter_eight_all_printed_spell_costs_are_paid_including_scourge_green() {
    for p in [Player::P0, Player::P1] {
        for (name, total) in [
            ("Mossborn Hydra", 3),
            ("Wildwood Scourge", 4),
            ("Prime Speaker Zegana", 6),
            ("Lunar Insight", 3),
            ("Scavenging Ooze", 2),
            ("Soul-Shackled Zombie", 4),
            ("Ayli, Eternal Pilgrim", 2),
            ("Tribute to Hunger", 3),
        ] {
            let mut g = setup(p);
            let id = named(&mut g, p, name, Zone::Hand);
            let before = g.state.players[p.index()].mana.iter().sum::<u32>();
            if name == "Wildwood Scourge" {
                cast_x(&mut g, id, 3);
            } else {
                let targets = if name == "Tribute to Hunger" {
                    vec![Target::Player(p.other())]
                } else {
                    vec![]
                };
                cast(&mut g, id, None, &targets);
            }
            assert_eq!(
                before - g.state.players[p.index()].mana.iter().sum::<u32>(),
                total,
                "{name}"
            );
            settle(&mut g);
            if name == "Wildwood Scourge" {
                assert_eq!(g.object_view(id).counters.plus_one, 3);
            }
        }
        let mut g = setup(p);
        g.state.players[p.index()].mana = [0, 0, 0, 0, 0, 4];
        let id = named(&mut g, p, "Wildwood Scourge", Zone::Hand);
        assert!(
            !g.legal_actions()
                .contains(&Action::AnnounceCast { card: id })
        );
    }
}
#[test]
fn fdn_counter_eight_mossborn_only_plus_one_landfall_replacements_and_exact_source() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let hydra = named(&mut g, p, "Mossborn Hydra", Zone::Hand);
        cast(&mut g, hydra, None, &[]);
        settle(&mut g);
        assert_eq!(g.stats(hydra).power, 1);
        assert!(g.keywords(hydra).contains(Keywords::TRAMPLE));
        counter(&mut g, hydra, CounterKind::Stun, 3, p);
        let land = named(&mut g, p.other(), "Forest", Zone::Hand);
        g.move_object(land, Zone::Battlefield);
        settle(&mut g);
        assert_eq!(g.object_view(hydra).counters.plus_one, 1);
        let land = named(&mut g, p, "Forest", Zone::Hand);
        g.move_object(land, Zone::Battlefield);
        settle(&mut g);
        assert_eq!(g.object_view(hydra).counters.plus_one, 2);
        assert_eq!(g.object_view(hydra).counters.stun, 3);
        let land = named(&mut g, p, "Forest", Zone::Hand);
        g.move_object(land, Zone::Battlefield);
        until(&mut g, |g| !g.state.stack.is_empty());
        g.move_object(hydra, Zone::Graveyard);
        g.move_object(hydra, Zone::Battlefield);
        settle(&mut g);
        assert_eq!(g.object_view(hydra).counters.plus_one, 1);
    }
}
#[test]
fn fdn_counter_eight_scourge_observes_each_other_nonhydra_counter_event_and_entry() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let scourge = named(&mut g, p, "Wildwood Scourge", Zone::Hand);
        cast_x(&mut g, scourge, 2);
        settle(&mut g);
        let bear = named(&mut g, p, "Bear Cub", Zone::Battlefield);
        let foe = named(&mut g, p.other(), "Bear Cub", Zone::Battlefield);
        counter(&mut g, bear, CounterKind::PlusOne, 4, p.other());
        settle(&mut g);
        assert_eq!(g.object_view(scourge).counters.plus_one, 3);
        counter(&mut g, bear, CounterKind::Stun, 1, p);
        counter(&mut g, foe, CounterKind::PlusOne, 1, p);
        counter(&mut g, scourge, CounterKind::PlusOne, 2, p);
        settle(&mut g);
        assert_eq!(g.object_view(scourge).counters.plus_one, 5);
        let hydra = named(&mut g, p, "Mossborn Hydra", Zone::Hand);
        g.move_object(hydra, Zone::Battlefield);
        settle(&mut g);
        assert_eq!(g.object_view(scourge).counters.plus_one, 5);
        let zegana = named(&mut g, p, "Prime Speaker Zegana", Zone::Hand);
        g.move_object(zegana, Zone::Battlefield);
        settle(&mut g);
        assert_eq!(g.object_view(scourge).counters.plus_one, 6);
        let stop = named(&mut g, p.other(), "Stop Cold", Zone::Hand);
        g.state.priority = p.other();
        cast(&mut g, stop, None, &[Target::Card(scourge)]);
        settle(&mut g);
        counter(&mut g, bear, CounterKind::PlusOne, 1, p);
        settle(&mut g);
        assert_eq!(g.object_view(scourge).counters.plus_one, 6);
    }
}
#[test]
fn fdn_counter_eight_zegana_entry_greatest_other_power_is_frozen_but_draw_is_live_lki() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let bear = named(&mut g, p, "Bear Cub", Zone::Battlefield);
        g.state.objects[bear as usize].growth.power = 5;
        named(&mut g, p.other(), "Gigantosaurus", Zone::Battlefield);
        let id = named(&mut g, p, "Prime Speaker Zegana", Zone::Hand);
        cast(&mut g, id, None, &[]);
        until(&mut g, |g| {
            g.on_battlefield(id) && !g.state.stack.is_empty()
        });
        assert_eq!(g.object_view(id).counters.plus_one, 7);
        g.state.objects[id as usize].growth.power = 3;
        let hand = g.hand_count(p);
        g.move_object(id, Zone::Graveyard);
        settle(&mut g);
        assert_eq!(g.hand_count(p), hand + 11);
        let mut g = setup(p);
        let id = named(&mut g, p, "Prime Speaker Zegana", Zone::Hand);
        cast(&mut g, id, None, &[]);
        settle(&mut g);
        assert_eq!(g.object_view(id).counters.plus_one, 0);
        assert_eq!(g.hand_count(p), 1);
    }
}
#[test]
fn fdn_counter_eight_lunar_counts_distinct_nonland_mana_values_including_zero_tokens() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        named(&mut g, p, "Bear Cub", Zone::Battlefield);
        named(&mut g, p, "Scavenging Ooze", Zone::Battlefield);
        let hydra = named(&mut g, p, "Mossborn Hydra", Zone::Hand);
        g.move_object(hydra, Zone::Battlefield);
        named(&mut g, p.other(), "Serra Angel", Zone::Battlefield);
        named(&mut g, p, "Forest", Zone::Battlefield);
        let token = Card::catalogue()
            .find(|c| c.token_info().is_some_and(|t| t.subtypes == "Treasure"))
            .unwrap();
        super::tests::spawn(&mut g, p, token, Zone::Battlefield);
        let id = named(&mut g, p, "Lunar Insight", Zone::Hand);
        cast(&mut g, id, None, &[]);
        settle(&mut g);
        assert_eq!(g.hand_count(p), 3);
        assert!(
            g.observe(p.other())
                .objects
                .iter()
                .all(|o| o.zone != Zone::Hand || o.owner != p)
        );
    }
}
#[test]
fn fdn_counter_eight_ooze_exiles_cards_not_tokens_creature_benefit_and_departed_source() {
    for p in [Player::P0, Player::P1] {
        for creature in [false, true] {
            for departed in [false, true] {
                let mut g = setup(p);
                let ooze = named(&mut g, p, "Scavenging Ooze", Zone::Battlefield);
                let target = named(
                    &mut g,
                    p.other(),
                    if creature { "Bear Cub" } else { "Forest" },
                    Zone::Graveyard,
                );
                let before = g.state.players[p.index()].mana[4];
                activate(&mut g, ooze, 0, Some(Target::Card(target)), None);
                assert_eq!(g.state.players[p.index()].mana[4], before - 1);
                if departed {
                    g.move_object(ooze, Zone::Graveyard);
                    g.move_object(ooze, Zone::Battlefield);
                }
                settle(&mut g);
                assert_eq!(g.object_view(target).zone, Zone::Exile);
                assert_eq!(g.state.players[p.index()].life, 20 + i128::from(creature));
                assert_eq!(
                    g.object_view(ooze).counters.plus_one,
                    u64::from(creature && !departed)
                );
            }
        }
    }
}
#[test]
fn fdn_counter_eight_zombie_same_grave_optional_partial_targets_actual_creature_only() {
    for p in [Player::P0, Player::P1] {
        for invalidate_creature in [false, true] {
            let mut g = setup(p);
            let creature = named(&mut g, p.other(), "Bear Cub", Zone::Graveyard);
            let land = named(&mut g, p.other(), "Forest", Zone::Graveyard);
            let own = named(&mut g, p, "Plains", Zone::Graveyard);
            let zombie = named(&mut g, p, "Soul-Shackled Zombie", Zone::Hand);
            cast(&mut g, zombie, None, &[]);
            until(&mut g, |g| {
                matches!(g.state.pending, Pending::Targets { .. })
            });
            checked(
                &mut g,
                Action::ChooseTarget {
                    target: Target::Card(creature),
                },
            );
            reject(
                &mut g,
                Action::ChooseTarget {
                    target: Target::Card(own),
                },
            );
            checked(
                &mut g,
                Action::ChooseTarget {
                    target: Target::Card(land),
                },
            );
            pay(&mut g);
            if invalidate_creature {
                g.move_object(creature, Zone::Hand);
            }
            settle(&mut g);
            assert_eq!(g.object_view(land).zone, Zone::Exile);
            assert_eq!(
                g.state.players[p.index()].life,
                if invalidate_creature { 20 } else { 22 }
            );
            assert_eq!(
                g.state.players[p.other().index()].life,
                if invalidate_creature { 20 } else { 18 }
            );
        }
    }
}
#[test]
fn fdn_counter_eight_ayli_sacrifice_lki_and_life_guard_are_announcement_only() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let ayli = named(&mut g, p, "Ayli, Eternal Pilgrim", Zone::Battlefield);
        let bear = named(&mut g, p, "Bear Cub", Zone::Battlefield);
        g.state.objects[bear as usize].growth.toughness = 4;
        assert!(g.keywords(ayli).contains(Keywords::DEATHTOUCH));
        let before = g.state.players[p.index()].mana.iter().sum::<u32>();
        activate(&mut g, ayli, 0, None, Some(bear));
        assert_eq!(
            before - g.state.players[p.index()].mana.iter().sum::<u32>(),
            1
        );
        assert_eq!(g.object_view(bear).zone, Zone::Graveyard);
        g.move_object(bear, Zone::Battlefield);
        settle(&mut g);
        assert_eq!(g.state.players[p.index()].life, 26);
        reject(
            &mut g,
            Action::AnnounceActivate {
                card: ayli,
                ability: 1,
                counter_payment: Counters::default(),
            },
        );
        g.state.players[p.index()].life = 30;
        let target = named(&mut g, p.other(), "Serra Angel", Zone::Battlefield);
        let before = g.state.players[p.index()].mana.iter().sum::<u32>();
        activate(&mut g, ayli, 1, Some(Target::Card(target)), Some(bear));
        assert_eq!(
            before - g.state.players[p.index()].mana.iter().sum::<u32>(),
            3
        );
        g.state.players[p.index()].life = 2;
        settle(&mut g);
        assert_eq!(g.object_view(target).zone, Zone::Exile);
        assert_eq!(g.state.players[p.index()].life, 2);
    }
}
#[test]
fn fdn_counter_eight_tribute_opponent_chooses_and_gains_frozen_toughness_or_zero() {
    for p in [Player::P0, Player::P1] {
        for exists in [false, true] {
            let mut g = setup(p);
            let own = named(&mut g, p, "Bear Cub", Zone::Battlefield);
            let foe = if exists {
                let id = named(&mut g, p.other(), "Bear Cub", Zone::Battlefield);
                g.state.objects[id as usize].growth.toughness = 5;
                Some(id)
            } else {
                None
            };
            let spell = named(&mut g, p, "Tribute to Hunger", Zone::Hand);
            cast(&mut g, spell, None, &[Target::Player(p.other())]);
            if let Some(foe) = foe {
                until(&mut g, |g| {
                    matches!(g.state.pending, Pending::EffectChoice { .. })
                });
                assert_eq!(g.actor(), Some(p.other()));
                let object = g.object_ref(own);
                reject(&mut g, Action::Sacrifice { object });
                let object = g.object_ref(foe);
                checked(&mut g, Action::Sacrifice { object });
            }
            settle(&mut g);
            assert_eq!(
                g.state.players[p.index()].life,
                if exists { 27 } else { 20 }
            );
            assert_eq!(g.object_view(own).zone, Zone::Battlefield);
        }
    }
}

#[test]
fn fdn_counter_eight_replacements_change_added_amount_and_observers_fire_once_per_object() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        named(&mut g, p, "Doubling Season", Zone::Battlefield);
        let hydra = named(&mut g, p, "Mossborn Hydra", Zone::Hand);
        cast(&mut g, hydra, None, &[]);
        settle(&mut g);
        assert_eq!(g.object_view(hydra).counters.plus_one, 2);
        let land = named(&mut g, p, "Forest", Zone::Hand);
        g.move_object(land, Zone::Battlefield);
        settle(&mut g);
        assert_eq!(g.object_view(hydra).counters.plus_one, 6);
        let scourge = named(&mut g, p, "Wildwood Scourge", Zone::Hand);
        g.state.players[p.index()].mana = [5; 6];
        cast_x(&mut g, scourge, 1);
        settle(&mut g);
        assert_eq!(g.object_view(scourge).counters.plus_one, 2);
        let bear = named(&mut g, p, "Bear Cub", Zone::Battlefield);
        counter(&mut g, bear, CounterKind::PlusOne, 4, p.other());
        if g.state.replacements.is_some() {
            g.advance_replacements(false);
        }
        settle(&mut g);
        assert_eq!(g.object_view(bear).counters.plus_one, 8);
        assert_eq!(g.object_view(scourge).counters.plus_one, 4);
    }
}
#[test]
fn fdn_counter_eight_ooze_stale_target_fizzles_and_capacity_failure_rolls_back_exile_counter_life()
{
    for p in [Player::P0, Player::P1] {
        for overflow in [false, true] {
            let mut g = setup(p);
            let ooze = named(&mut g, p, "Scavenging Ooze", Zone::Battlefield);
            let bear = named(&mut g, p.other(), "Bear Cub", Zone::Graveyard);
            activate(&mut g, ooze, 0, Some(Target::Card(bear)), None);
            if overflow {
                g.state.players[p.index()].life = i128::MAX;
                checked(&mut g, Action::Pass);
                let saved = g.snapshot();
                let revision = g.revision();
                assert!(matches!(
                    g.apply(g.actor().unwrap(), Action::Pass),
                    Err(Error::CapacityExceeded(_))
                ));
                assert_eq!(saved, g.snapshot());
                assert_eq!(revision, g.revision());
                assert_eq!(g.object_view(bear).zone, Zone::Graveyard);
                assert_eq!(g.object_view(ooze).counters.plus_one, 0);
            } else {
                g.move_object(bear, Zone::Hand);
                g.move_object(bear, Zone::Graveyard);
                settle(&mut g);
                assert_eq!(g.object_view(bear).zone, Zone::Graveyard);
                assert_eq!(g.object_view(ooze).counters.plus_one, 0);
                assert_eq!(g.state.players[p.index()].life, 20);
            }
        }
    }
}
#[test]
fn fdn_counter_eight_zombie_zero_targets_and_scourge_zero_x_are_legal() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let bear = named(&mut g, p.other(), "Bear Cub", Zone::Graveyard);
        let zombie = named(&mut g, p, "Soul-Shackled Zombie", Zone::Hand);
        cast(&mut g, zombie, None, &[]);
        settle(&mut g);
        assert_eq!(g.object_view(bear).zone, Zone::Graveyard);
        assert_eq!(g.state.players[p.index()].life, 20);
        let scourge = named(&mut g, p, "Wildwood Scourge", Zone::Hand);
        cast_x(&mut g, scourge, 0);
        settle(&mut g);
        assert_eq!(g.object_view(scourge).zone, Zone::Graveyard);
    }
}
#[test]
fn fdn_counter_eight_simultaneous_rise_freezes_entry_power_and_dispatches_counters_once() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let scourge = named(&mut g, p, "Wildwood Scourge", Zone::Hand);
        cast_x(&mut g, scourge, 3);
        settle(&mut g);
        let zegana = named(&mut g, p, "Prime Speaker Zegana", Zone::Graveyard);
        let giant = named(&mut g, p.other(), "Gigantosaurus", Zone::Graveyard);
        let rise = named(&mut g, p, "Rise of the Dark Realms", Zone::Hand);
        g.state.players[p.index()].mana = [5; 6];
        cast(&mut g, rise, None, &[]);
        settle(&mut g);
        assert_eq!(g.object_view(giant).controller, p);
        assert_eq!(
            g.object_view(zegana).counters.plus_one,
            3,
            "simultaneous ten-power creature is outside the pre-entry world"
        );
        assert_eq!(
            g.object_view(scourge).counters.plus_one,
            4,
            "one counter-placement event for the Zegana entry"
        );
        assert_eq!(g.hand_count(p), 4);
    }
}

#[test]
fn fdn_counter_eight_unrestricted_targets_include_both_players_graveyards_and_permanents() {
    for p in [Player::P0, Player::P1] {
        for target_owner in [p, p.other()] {
            let mut g = setup(p);
            let ooze = named(&mut g, p, "Scavenging Ooze", Zone::Battlefield);
            let victim = named(&mut g, target_owner, "Bear Cub", Zone::Graveyard);
            activate(&mut g, ooze, 0, Some(Target::Card(victim)), None);
            settle(&mut g);
            assert_eq!(g.object_view(victim).zone, Zone::Exile);
            assert_eq!(g.object_view(ooze).counters.plus_one, 1);
            assert_eq!(g.state.players[p.index()].life, 21);

            let mut g = setup(p);
            let victim = named(&mut g, target_owner, "Bear Cub", Zone::Graveyard);
            let land = named(&mut g, target_owner, "Forest", Zone::Graveyard);
            let wrong_grave = named(&mut g, target_owner.other(), "Plains", Zone::Graveyard);
            let zombie = named(&mut g, p, "Soul-Shackled Zombie", Zone::Hand);
            cast(&mut g, zombie, None, &[]);
            until(&mut g, |g| {
                matches!(g.state.pending, Pending::Targets { .. })
            });
            checked(
                &mut g,
                Action::ChooseTarget {
                    target: Target::Card(victim),
                },
            );
            reject(
                &mut g,
                Action::ChooseTarget {
                    target: Target::Card(wrong_grave),
                },
            );
            checked(
                &mut g,
                Action::ChooseTarget {
                    target: Target::Card(land),
                },
            );
            pay(&mut g);
            settle(&mut g);
            assert_eq!(g.object_view(victim).zone, Zone::Exile);
            assert_eq!(g.object_view(land).zone, Zone::Exile);
            assert_eq!(g.object_view(wrong_grave).zone, Zone::Graveyard);
            assert_eq!(g.state.players[p.index()].life, 22);
            assert_eq!(g.state.players[p.other().index()].life, 18);

            let mut g = setup(p);
            g.state.players[p.index()].life = 30;
            let ayli = named(&mut g, p, "Ayli, Eternal Pilgrim", Zone::Battlefield);
            let cost = named(&mut g, p, "Bear Cub", Zone::Battlefield);
            let victim = named(&mut g, target_owner, "Serra Angel", Zone::Battlefield);
            activate(&mut g, ayli, 1, Some(Target::Card(victim)), Some(cost));
            settle(&mut g);
            assert_eq!(g.object_view(cost).zone, Zone::Graveyard);
            assert_eq!(g.object_view(victim).zone, Zone::Exile);
        }
        for sacrifice_target in [false, true] {
            let mut g = setup(p);
            g.state.players[p.index()].life = 30;
            let ayli = named(&mut g, p, "Ayli, Eternal Pilgrim", Zone::Battlefield);
            let cost = named(&mut g, p, "Bear Cub", Zone::Battlefield);
            // Both are legal targets. Paying the cost can invalidate its own target;
            // "another" restricts the sacrificed creature, not the exile target.
            let target = if sacrifice_target { cost } else { ayli };
            activate(&mut g, ayli, 1, Some(Target::Card(target)), Some(cost));
            settle(&mut g);
            assert_eq!(g.object_view(cost).zone, Zone::Graveyard);
            assert_eq!(
                g.object_view(ayli).zone,
                if sacrifice_target {
                    Zone::Battlefield
                } else {
                    Zone::Exile
                }
            );
            assert_eq!(g.state.players[p.index()].life, 30);
        }
    }
}
