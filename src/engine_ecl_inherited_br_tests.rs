//! Paid inherited-card audit matrices supplement existing adversarial rule suites.
use super::ability_timeline_tests::{cast, checked, named, settle, setup};
use super::*;
fn reject(g: &mut Game, action: Action) {
    let s = g.snapshot();
    let r = g.revision();
    assert_eq!(
        g.apply(g.actor().unwrap(), action),
        Err(Error::IllegalAction)
    );
    assert_eq!(g.snapshot(), s);
    assert_eq!(g.revision(), r);
}
fn token_ids(g: &Game, p: Player) -> Vec<CardId> {
    g.state
        .objects
        .iter()
        .enumerate()
        .filter(|(_, o)| o.zone == Zone::Battlefield && o.controller == p && o.is_token())
        .map(|(id, _)| id as CardId)
        .collect()
}
#[test]
fn ecl_inherited_br_paid_creature_bodies_and_entry_obligations_both_seats() {
    for p in [Player::P0, Player::P1] {
        for (name, power, toughness) in [
            ("Bile-Vial Boggart", 1, 1),
            ("Bitterbloom Bearer", 1, 1),
            ("Boldwyr Aggressor", 2, 5),
            ("Boneclub Berserker", 2, 4),
            ("Brambleback Brute", 2, 3),
            ("Elder Auntie", 2, 2),
            ("Explosive Prodigy", 1, 1),
            ("Flame-Chain Mauler", 2, 2),
            ("Flamekin Gildweaver", 4, 3),
            ("Graveshifter", 2, 2),
            ("Kulrath Zealot", 6, 5),
            ("Moonglove Extractor", 2, 1),
            ("Nightmare Sower", 2, 3),
            ("Scarblade Scout", 2, 2),
            ("Shimmercreep", 3, 5),
            ("Sizzling Changeling", 3, 2),
            ("Squawkroaster", 1, 4),
        ] {
            let mut g = setup(p);
            named(&mut g, p.other(), "Gigantosaurus", Zone::Battlefield);
            named(&mut g, p, "Bear Cub", Zone::Graveyard);
            named(&mut g, p, "Mountain", Zone::Library);
            named(&mut g, p, "Swamp", Zone::Library);
            let id = named(&mut g, p, name, Zone::Hand);
            if name == "Bitterbloom Bearer" {
                g.state.active = p.other();
                g.state.step = Step::Upkeep;
            }
            cast(&mut g, id, None, &[]);
            settle(&mut g);
            assert_eq!(g.object_view(id).zone, Zone::Battlefield, "{name}");
            assert_eq!(g.stats(id), Stats { power, toughness }, "{name}");
            if name == "Brambleback Brute" {
                assert_eq!(g.counter_count(id, CounterKind::MinusOne), 2);
            }
            if name == "Elder Auntie" {
                let ids = token_ids(&g, p);
                assert_eq!(ids.len(), 1);
                assert!(g.has_subtype(ids[0], "Goblin"));
                assert_eq!(g.object_view(ids[0]).colors, 4 | 8);
            }
            if name == "Flamekin Gildweaver" {
                let ids = token_ids(&g, p);
                assert_eq!(ids.len(), 1);
                let treasure = ids[0];
                assert!(g.has_subtype(treasure, "Treasure"));
                g.state.players[p.index()].mana = [0; 6];
                checked(
                    &mut g,
                    Action::ActivateMana {
                        card: treasure,
                        ability: 0,
                        production: [0, 0, 0, 1, 0, 0],
                        payment: [0; 6],
                        counter_payment: Counters::default(),
                    },
                );
                assert_eq!(g.state.players[p.index()].mana, [0, 0, 0, 1, 0, 0]);
                assert!(!g.on_battlefield(treasure));
            }
        }
    }
}
#[test]
fn ecl_inherited_br_giantfall_modes_preserve_damage_source_and_artifact_restriction() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let source = named(&mut g, p, "Scarblade Scout", Zone::Battlefield);
        let target = named(&mut g, p.other(), "Bear Cub", Zone::Battlefield);
        let spell = named(&mut g, p, "Giantfall", Zone::Hand);
        cast(
            &mut g,
            spell,
            Some(0),
            &[Target::Card(source), Target::Card(target)],
        );
        settle(&mut g);
        assert_eq!(g.object_view(target).zone, Zone::Graveyard);
        assert_eq!(g.state.players[p.index()].life, 22);
        let artifact = named(&mut g, p.other(), "Firdoch Core", Zone::Battlefield);
        let spell = named(&mut g, p, "Giantfall", Zone::Hand);
        cast(&mut g, spell, Some(1), &[Target::Card(artifact)]);
        settle(&mut g);
        assert_eq!(g.object_view(artifact).zone, Zone::Graveyard);
    }
}
#[test]
fn ecl_inherited_br_reckless_and_impolite_followups_require_a_surviving_target() {
    for p in [Player::P0, Player::P1] {
        for (name, valid) in [
            ("Reckless Ransacking", true),
            ("Reckless Ransacking", false),
            ("Impolite Entrance", true),
            ("Impolite Entrance", false),
        ] {
            let mut g = setup(p);
            let target = named(&mut g, p, "Bear Cub", Zone::Battlefield);
            named(&mut g, p, "Mountain", Zone::Library);
            let spell = named(&mut g, p, name, Zone::Hand);
            let hand = g.hand_count(p);
            cast(&mut g, spell, None, &[Target::Card(target)]);
            if !valid {
                g.move_object(target, Zone::Hand);
            }
            settle(&mut g);
            if name == "Reckless Ransacking" {
                assert_eq!(token_ids(&g, p).len(), usize::from(valid));
                if valid {
                    assert_eq!(
                        g.stats(target),
                        Stats {
                            power: 5,
                            toughness: 4
                        }
                    );
                }
            } else {
                assert_eq!(
                    g.hand_count(p),
                    hand - 1 + usize::from(!valid) + usize::from(valid)
                );
                if valid {
                    assert!(
                        g.keywords(target)
                            .contains(Keywords::HASTE.union(Keywords::TRAMPLE))
                    );
                }
            }
        }
    }
}
#[test]
fn ecl_inherited_br_tweeze_damage_then_optional_private_discard_draw() {
    for p in [Player::P0, Player::P1] {
        for discard in [false, true] {
            let mut g = setup(p);
            let hand = named(&mut g, p, "Mountain", Zone::Hand);
            let top = named(&mut g, p, "Swamp", Zone::Library);
            let spell = named(&mut g, p, "Tweeze", Zone::Hand);
            cast(&mut g, spell, None, &[Target::Player(p.other())]);
            for _ in 0..10 {
                if g.observe(p).effect_choice.is_some() {
                    break;
                }
                checked(&mut g, Action::Pass);
            }
            assert_eq!(g.state.players[p.other().index()].life, 17);
            assert!(g.observe(p.other()).objects.iter().all(|o| o.id != hand));
            if discard {
                checked(&mut g, Action::Discard { card: hand });
            } else {
                checked(&mut g, Action::DeclineEffectPayment);
            }
            settle(&mut g);
            assert_eq!(
                g.object_view(hand).zone,
                if discard { Zone::Graveyard } else { Zone::Hand }
            );
            assert_eq!(
                g.object_view(top).zone,
                if discard { Zone::Hand } else { Zone::Library }
            );
        }
    }
}
#[test]
fn ecl_inherited_br_sear_targets_only_creatures_and_walkers_and_applies_four_damage() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let victim = named(&mut g, p.other(), "Gigantosaurus", Zone::Battlefield);
        let spell = named(&mut g, p, "Sear", Zone::Hand);
        checked(&mut g, Action::AnnounceCast { card: spell });
        reject(
            &mut g,
            Action::ChooseTarget {
                target: Target::Player(p.other()),
            },
        );
        checked(
            &mut g,
            Action::ChooseTarget {
                target: Target::Card(victim),
            },
        );
        checked(
            &mut g,
            Action::PayCosts {
                payment: [0, 0, 0, 1, 0, 1],
            },
        );
        settle(&mut g);
        assert_eq!(g.object_view(victim).damage, 4);
    }
}
#[test]
fn ecl_inherited_br_paid_counter_spells_hit_actual_creatures_and_both_sides() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let own = named(&mut g, p, "Gigantosaurus", Zone::Battlefield);
        let enemy = named(&mut g, p.other(), "Gigantosaurus", Zone::Battlefield);
        let rot = named(&mut g, p, "Blight Rot", Zone::Hand);
        cast(&mut g, rot, None, &[Target::Card(enemy)]);
        settle(&mut g);
        assert_eq!(g.counter_count(enemy, CounterKind::MinusOne), 4);
        let dark = named(&mut g, p, "Darkness Descends", Zone::Hand);
        cast(&mut g, dark, None, &[]);
        settle(&mut g);
        assert_eq!(g.counter_count(own, CounterKind::MinusOne), 2);
        assert_eq!(g.counter_count(enemy, CounterKind::MinusOne), 6);
    }
}
#[test]
fn ecl_inherited_br_feed_the_flames_exiles_a_later_death_not_just_lethal_damage() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let enemy = named(&mut g, p.other(), "Gigantosaurus", Zone::Battlefield);
        let spell = named(&mut g, p, "Feed the Flames", Zone::Hand);
        cast(&mut g, spell, None, &[Target::Card(enemy)]);
        settle(&mut g);
        assert_eq!(g.object_view(enemy).damage, 5);
        assert!(g.on_battlefield(enemy));
        g.move_object(enemy, Zone::Graveyard);
        assert_eq!(g.object_view(enemy).zone, Zone::Exile);
    }
}
#[test]
fn ecl_inherited_br_moonglove_paid_attack_finishes_draw_and_life_loss_after_source_leaves() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let source = named(&mut g, p, "Moonglove Extractor", Zone::Hand);
        cast(&mut g, source, None, &[]);
        settle(&mut g);
        let top = named(&mut g, p, "Mountain", Zone::Library);
        let before = g.hand_count(p);
        g.state.objects[source as usize].summoning_sick = false;
        g.state.step = Step::Attackers;
        g.state.pending = Pending::Attackers;
        checked(&mut g, Action::Attack { card: source });
        checked(&mut g, Action::FinishAttackers);
        g.move_object(source, Zone::Graveyard);
        settle(&mut g);
        assert_eq!(g.object_view(top).zone, Zone::Hand);
        assert_eq!(g.hand_count(p), before + 1);
        assert_eq!(g.state.players[p.index()].life, 19);
    }
}
#[test]
fn ecl_inherited_br_mauler_repeated_paid_activations_grant_fixed_menace_and_cleanup() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let source = named(&mut g, p, "Flame-Chain Mauler", Zone::Hand);
        cast(&mut g, source, None, &[]);
        settle(&mut g);
        for _ in 0..2 {
            checked(
                &mut g,
                Action::AnnounceActivate {
                    card: source,
                    ability: 0,
                    counter_payment: Counters::default(),
                },
            );
            checked(
                &mut g,
                Action::PayCosts {
                    payment: [0, 0, 0, 1, 0, 1],
                },
            );
            settle(&mut g);
        }
        assert_eq!(
            g.stats(source),
            Stats {
                power: 4,
                toughness: 2
            }
        );
        assert!(g.keywords(source).contains(Keywords::MENACE));
        g.state.step = Step::End;
        g.give_priority();
        checked(&mut g, Action::Pass);
        checked(&mut g, Action::Pass);
        assert_eq!(
            g.stats(source),
            Stats {
                power: 2,
                toughness: 2
            }
        );
        assert!(!g.keywords(source).contains(Keywords::MENACE));
    }
}
#[test]
fn ecl_inherited_br_bile_optional_death_target_decline_accept_and_stale_incarnation() {
    for p in [Player::P0, Player::P1] {
        for case in 0..3 {
            let mut g = setup(p);
            let source = named(&mut g, p, "Bile-Vial Boggart", Zone::Hand);
            cast(&mut g, source, None, &[]);
            settle(&mut g);
            let victim = named(&mut g, p.other(), "Bear Cub", Zone::Battlefield);
            g.move_object(source, Zone::Graveyard);
            g.prepare_triggers();
            if case == 0 {
                checked(&mut g, Action::FinishTargets);
            } else {
                checked(
                    &mut g,
                    Action::ChooseTarget {
                        target: Target::Card(victim),
                    },
                );
                if case == 2 {
                    g.move_object(victim, Zone::Hand);
                    g.move_object(victim, Zone::Battlefield);
                }
            }
            settle(&mut g);
            assert_eq!(
                g.counter_count(victim, CounterKind::MinusOne),
                u64::from(case == 1)
            );
        }
    }
}
#[test]
fn ecl_inherited_br_paid_tribal_sources_count_kindred_and_only_own_recipients() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let source = named(&mut g, p, "Boneclub Berserker", Zone::Hand);
        cast(&mut g, source, None, &[]);
        settle(&mut g);
        let kindred = named(&mut g, p, "Boggart Mischief", Zone::Battlefield);
        let changeling = named(&mut g, p, "Chitinous Graspling", Zone::Battlefield);
        assert_eq!(g.stats(source).power, 6);
        g.move_object(kindred, Zone::Hand);
        assert_eq!(g.stats(source).power, 4);
        g.move_object(changeling, Zone::Hand);
        assert_eq!(g.stats(source).power, 2);
        let giant = named(&mut g, p, "Boldwyr Aggressor", Zone::Hand);
        cast(&mut g, giant, None, &[]);
        settle(&mut g);
        let friend = named(&mut g, p, "Chitinous Graspling", Zone::Battlefield);
        let enemy = named(&mut g, p.other(), "Chitinous Graspling", Zone::Battlefield);
        assert!(g.keywords(friend).contains(Keywords::DOUBLE_STRIKE));
        assert!(!g.keywords(enemy).contains(Keywords::DOUBLE_STRIKE));
        g.move_object(giant, Zone::Hand);
        assert!(!g.keywords(friend).contains(Keywords::DOUBLE_STRIKE));
    }
}
#[test]
fn ecl_inherited_br_paid_bitterbloom_flash_and_own_upkeep_finishes_token_before_lethal_sba() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        g.state.active = p.other();
        g.state.step = Step::Upkeep;
        let source = named(&mut g, p, "Bitterbloom Bearer", Zone::Hand);
        cast(&mut g, source, None, &[]);
        settle(&mut g);
        assert!(
            g.keywords(source)
                .contains(Keywords::FLASH.union(Keywords::FLYING))
        );
        g.state.active = p;
        g.state.players[p.index()].life = 1;
        g.begin_step(Step::Upkeep);
        g.give_priority();
        g.prepare_triggers();
        checked(&mut g, Action::Pass);
        checked(&mut g, Action::Pass);
        assert_eq!(g.state.players[p.index()].life, 0);
        let ids = token_ids(&g, p);
        assert_eq!(ids.len(), 1);
        assert_eq!(g.object_view(ids[0]).colors, 2 | 4);
        assert!(g.keywords(ids[0]).contains(Keywords::FLYING));
        assert_eq!(g.outcome(), Some(Outcome::Winner(p.other())));
    }
}
#[test]
fn ecl_inherited_br_tweeze_without_hand_does_not_draw_for_free() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let top = named(&mut g, p, "Mountain", Zone::Library);
        let spell = named(&mut g, p, "Tweeze", Zone::Hand);
        cast(&mut g, spell, None, &[Target::Player(p.other())]);
        settle(&mut g);
        assert_eq!(g.state.players[p.other().index()].life, 17);
        assert_eq!(g.object_view(top).zone, Zone::Library);
    }
}
