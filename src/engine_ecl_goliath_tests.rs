//! Resolution-only replacements, exact dream counters and one free cast during attack resolution.
use super::ability_timeline_tests::{cast, checked, named, settle, setup};
use super::*;
const NAME: &str = "Goliath Daydreamer";
fn paid(g: &mut Game, p: Player) -> CardId {
    let id = named(g, p, NAME, Zone::Hand);
    g.state.players[p.index()].mana = [0, 0, 0, 2, 0, 2];
    cast(g, id, None, &[]);
    settle(g);
    id
}
fn reach(g: &mut Game, stop: impl Fn(&Game) -> bool) {
    g.prepare_triggers();
    for _ in 0..100 {
        if stop(g) {
            return;
        }
        let actions = g.legal_actions();
        let a = actions
            .iter()
            .find(|a| {
                matches!(
                    a,
                    Action::Pass | Action::FinishTargets | Action::PayCosts { .. }
                )
            })
            .or_else(|| actions.iter().find(|a| !matches!(a, Action::Concede)))
            .unwrap()
            .clone();
        checked(g, a);
    }
    panic!("missing Goliath choice {:?}", g.state.pending);
}
fn reject(g: &mut Game, a: Action) {
    let before = g.snapshot();
    let revision = g.revision();
    assert_eq!(g.apply(g.actor().unwrap(), a), Err(Error::IllegalAction));
    assert_eq!(g.snapshot(), before);
    assert_eq!(g.revision(), revision);
}
fn announce_exiled(g: &mut Game, id: CardId, target: Target) {
    checked(g, Action::AnnounceExile { card: id });
    checked(g, Action::ChooseTarget { target });
    let a = g
        .legal_actions()
        .into_iter()
        .find(|a| matches!(a, Action::PayCosts { .. }))
        .unwrap();
    checked(g, a);
}
fn attack(g: &mut Game, p: Player, source: CardId) {
    g.state.active = p;
    g.state.priority = p;
    g.state.step = Step::Attackers;
    g.state.pending = Pending::Attackers;
    g.state.objects[source as usize].summoning_sick = false;
    g.state.objects[source as usize].tapped = false;
    checked(g, Action::Attack { card: source });
    checked(g, Action::FinishAttackers);
}
fn dream(g: &mut Game, p: Player, name: &str) -> CardId {
    let id = named(g, p, name, Zone::Exile);
    g.set_counters(id, Counters::of(CounterKind::Dream, 1));
    id
}
fn finish_free(g: &mut Game) {
    reach(g, |g| g.state.announcing.is_none());
    checked(g, Action::FinishResolutionCasts);
    settle(g);
}
#[test]
fn ecl_goliath_paid_body_marks_actual_hand_spell_and_source_departure_preserves_resolution() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let source = paid(&mut g, p);
        assert_eq!(
            g.stats(source),
            Stats {
                power: 4,
                toughness: 4
            }
        );
        let spell = named(&mut g, p, "Shock", Zone::Hand);
        g.state.players[p.index()].mana = [30; 6];
        cast(&mut g, spell, None, &[Target::Player(p.other())]);
        reach(&mut g, |g| !g.observe(p).dream_exiles.is_empty());
        assert_eq!(g.observe(p).dream_exiles[0].object, g.object_ref(spell));
        g.move_object(source, Zone::Graveyard);
        settle(&mut g);
        assert_eq!(g.state.players[p.other().index()].life, 18);
        assert_eq!(g.state.objects[spell as usize].zone, Zone::Exile);
        assert_eq!(g.counter_count(spell, CounterKind::Dream), 1);
        assert!(g.observe(p).dream_exiles.is_empty());
        // Another instance uses the same owned dream-card pool, without a link to the first.
        let source = named(&mut g, p, NAME, Zone::Battlefield);
        attack(&mut g, p, source);
        reach(&mut g, |g| {
            matches!(g.state.pending, Pending::ResolutionCasts { .. })
        });
        let object = g.object_ref(spell);
        checked(
            &mut g,
            Action::CastExiledDuringResolution {
                object,
                face: CastFace::Normal,
            },
        );
        checked(
            &mut g,
            Action::ChooseTarget {
                target: Target::Player(p.other()),
            },
        );
        finish_free(&mut g);
        assert_eq!(g.state.players[p.other().index()].life, 16);
        assert_eq!(g.state.objects[spell as usize].zone, Zone::Graveyard);
        assert_eq!(g.counter_count(spell, CounterKind::Dream), 0);
    }
}
#[test]
fn ecl_goliath_countered_fizzled_and_countered_trigger_do_not_create_dream_exile() {
    for p in [Player::P0, Player::P1] {
        for case in 0..3 {
            let mut g = setup(p);
            paid(&mut g, p);
            let victim = named(&mut g, p.other(), "Bear Cub", Zone::Battlefield);
            let spell = named(&mut g, p, "Shock", Zone::Hand);
            g.state.players[p.index()].mana = [30; 6];
            cast(&mut g, spell, None, &[Target::Card(victim)]);
            if case == 2 {
                reach(&mut g, |g| {
                    g.state.stack.last().is_some_and(|s| s.kind.is_triggered())
                });
                let trigger = g.state.stack.last().unwrap().id;
                assert!(g.counter_stack_item(trigger, false));
            } else {
                reach(&mut g, |g| !g.observe(p).dream_exiles.is_empty());
                if case == 0 {
                    let id = g
                        .state
                        .stack
                        .iter()
                        .find(|s| s.card == spell && s.kind == StackKind::Spell)
                        .unwrap()
                        .id;
                    assert!(g.counter_stack_item(id, false));
                } else {
                    g.move_object(victim, Zone::Hand);
                }
            }
            settle(&mut g);
            assert_eq!(g.state.objects[spell as usize].zone, Zone::Graveyard);
            assert_eq!(g.counter_count(spell, CounterKind::Dream), 0);
            assert!(g.observe(p).dream_exiles.is_empty());
        }
    }
}
#[test]
fn ecl_goliath_origin_filter_excludes_exile_and_foreign_casts_but_includes_free_hand_cast() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        paid(&mut g, p);
        let exiled = named(&mut g, p, "Shock", Zone::Exile);
        let object = g.object_ref(exiled);
        g.grant_exile_play_permission(p, object, false);
        g.state.players[p.index()].mana = [30; 6];
        announce_exiled(&mut g, exiled, Target::Player(p.other()));
        settle(&mut g);
        assert_eq!(g.state.objects[exiled as usize].zone, Zone::Graveyard);
        named(&mut g, p, "Omniscience", Zone::Battlefield);
        let hand = named(&mut g, p, "Shock", Zone::Hand);
        g.state.players[p.index()].mana = [0; 6];
        checked(
            &mut g,
            Action::AnnounceFree {
                card: hand,
                face: CastFace::Normal,
                flash_surcharge: false,
            },
        );
        checked(
            &mut g,
            Action::ChooseTarget {
                target: Target::Player(p.other()),
            },
        );
        checked(&mut g, Action::PayCosts { payment: [0; 6] });
        settle(&mut g);
        assert_eq!(g.state.objects[hand as usize].zone, Zone::Exile);
        assert_eq!(g.counter_count(hand, CounterKind::Dream), 1);
        g.state.priority = p.other();
        g.state.players[p.other().index()].mana = [30; 6];
        let foreign = named(&mut g, p.other(), "Shock", Zone::Hand);
        cast(&mut g, foreign, None, &[Target::Player(p)]);
        settle(&mut g);
        assert_eq!(g.state.objects[foreign as usize].zone, Zone::Graveyard);
        assert_eq!(g.counter_count(foreign, CounterKind::Dream), 0);
    }
}
#[test]
fn ecl_goliath_attack_cast_is_optional_one_spell_only_and_source_independent() {
    for p in [Player::P0, Player::P1] {
        for decline in [false, true] {
            let mut g = setup(p);
            let source = paid(&mut g, p);
            let first = dream(&mut g, p, "Bear Cub");
            let second = dream(&mut g, p, "Shock");
            let foreign = dream(&mut g, p.other(), "Shock");
            let land = dream(&mut g, p, "Forest");
            attack(&mut g, p, source);
            g.move_object(source, Zone::Graveyard);
            reach(&mut g, |g| {
                matches!(g.state.pending, Pending::ResolutionCasts { .. })
            });
            for id in [foreign, land] {
                let object = g.object_ref(id);
                reject(
                    &mut g,
                    Action::CastExiledDuringResolution {
                        object,
                        face: CastFace::Normal,
                    },
                );
            }
            if decline {
                checked(&mut g, Action::FinishResolutionCasts);
                settle(&mut g);
                assert_eq!(g.counter_count(first, CounterKind::Dream), 1);
            } else {
                let object = g.object_ref(first);
                checked(
                    &mut g,
                    Action::CastExiledDuringResolution {
                        object,
                        face: CastFace::Normal,
                    },
                );
                checked(&mut g, Action::PayCosts { payment: [0; 6] });
                let object = g.object_ref(second);
                reject(
                    &mut g,
                    Action::CastExiledDuringResolution {
                        object,
                        face: CastFace::Normal,
                    },
                );
                checked(&mut g, Action::FinishResolutionCasts);
                settle(&mut g);
                assert_eq!(g.state.objects[first as usize].zone, Zone::Battlefield);
                assert_eq!(g.counter_count(first, CounterKind::Dream), 0);
            }
            assert_eq!(g.state.objects[second as usize].zone, Zone::Exile);
        }
    }
}
#[test]
fn ecl_goliath_attack_cast_pays_additional_cost_and_free_x_is_zero() {
    for p in [Player::P0, Player::P1] {
        for x_spell in [false, true] {
            let mut g = setup(p);
            let source = paid(&mut g, p);
            let spell = dream(
                &mut g,
                p,
                if x_spell {
                    "Wildwood Scourge"
                } else {
                    "Thrill of Possibility"
                },
            );
            let donor = named(&mut g, p, "Forest", Zone::Hand);
            g.state.players[p.index()].mana = [0; 6];
            attack(&mut g, p, source);
            reach(&mut g, |g| {
                matches!(g.state.pending, Pending::ResolutionCasts { .. })
            });
            let object = g.object_ref(spell);
            checked(
                &mut g,
                Action::CastExiledDuringResolution {
                    object,
                    face: CastFace::Normal,
                },
            );
            if x_spell {
                assert_eq!(g.state.announcing.as_ref().unwrap().item.x, Some(0));
                reject(&mut g, Action::ChooseX { value: 1 });
            } else {
                reject(&mut g, Action::PayCosts { payment: [0; 6] });
                let object = g.object_ref(donor);
                checked(&mut g, Action::ChooseAdditionalCost { object });
            }
            finish_free(&mut g);
            assert_eq!(g.state.objects[spell as usize].zone, Zone::Graveyard);
            if !x_spell {
                assert_eq!(g.state.objects[donor as usize].zone, Zone::Graveyard);
            }
        }
    }
}
#[test]
fn ecl_goliath_adventure_and_omen_choose_between_distinct_resolution_replacements() {
    for p in [Player::P0, Player::P1] {
        for omen in [false, true] {
            for choose_dream in [false, true] {
                let mut g = setup(p);
                paid(&mut g, p);
                let name = if omen {
                    "Whirlwing Stormbrood // Dynamic Soar"
                } else {
                    "Bilbo Baggins, Burglar // Take a Glance"
                };
                let spell = named(&mut g, p, name, Zone::Hand);
                g.state.players[p.index()].mana = [30; 6];
                if omen {
                    let bear = named(&mut g, p, "Bear Cub", Zone::Battlefield);
                    checked(&mut g, Action::AnnounceOmen { card: spell });
                    checked(
                        &mut g,
                        Action::ChooseTarget {
                            target: Target::Card(bear),
                        },
                    );
                } else {
                    checked(&mut g, Action::AnnounceAdventure { card: spell });
                }
                reach(&mut g, |g| {
                    matches!(g.state.pending, Pending::ZoneReplacement { .. })
                });
                let choice = g.zone_choice_view(p).unwrap();
                assert!(choice.choices.contains(&ZoneReplacementKind::DreamExile));
                let other = if omen {
                    ZoneReplacementKind::OmenShuffle
                } else {
                    ZoneReplacementKind::AdventureExile
                };
                assert!(choice.choices.contains(&other));
                assert!(g.zone_choice_view(p.other()).is_none());
                checked(
                    &mut g,
                    Action::ChooseZoneReplacement {
                        effect: if choose_dream {
                            ZoneReplacementKind::DreamExile
                        } else {
                            other
                        },
                    },
                );
                settle(&mut g);
                assert_eq!(
                    g.counter_count(spell, CounterKind::Dream),
                    u64::from(choose_dream)
                );
                assert_eq!(
                    g.state.objects[spell as usize].zone,
                    if choose_dream || !omen {
                        Zone::Exile
                    } else {
                        Zone::Library
                    }
                );
                assert_eq!(g.can_play_adventure(p, spell), !omen && !choose_dream);
                assert_eq!(
                    g.state.objects[spell as usize].card,
                    Card::from_name(name).unwrap()
                );
            }
        }
    }
}
#[test]
fn ecl_goliath_other_exile_replacement_and_resolving_self_exile_do_not_gain_dream_counters() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        paid(&mut g, p);
        named(&mut g, p, "Dryad Militant", Zone::Battlefield);
        let spell = named(&mut g, p, "Shock", Zone::Hand);
        g.state.players[p.index()].mana = [30; 6];
        cast(&mut g, spell, None, &[Target::Player(p.other())]);
        reach(&mut g, |g| {
            matches!(g.state.pending, Pending::ZoneReplacement { .. })
        });
        assert!(
            g.zone_choice_view(p)
                .unwrap()
                .choices
                .contains(&ZoneReplacementKind::Exile)
        );
        checked(
            &mut g,
            Action::ChooseZoneReplacement {
                effect: ZoneReplacementKind::Exile,
            },
        );
        settle(&mut g);
        assert_eq!(g.state.objects[spell as usize].zone, Zone::Exile);
        assert_eq!(g.counter_count(spell, CounterKind::Dream), 0);
        let light = named(&mut g, p, "Morningtide's Light", Zone::Hand);
        g.state.players[p.index()].mana = [30; 6];
        cast(&mut g, light, None, &[]);
        settle(&mut g);
        assert_eq!(g.state.objects[light as usize].zone, Zone::Exile);
        assert_eq!(g.counter_count(light, CounterKind::Dream), 0);
        assert!(g.observe(p).dream_exiles.is_empty());
    }
}
#[test]
fn ecl_goliath_multiple_observers_and_copied_mark_trigger_place_only_one_dream_counter() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        paid(&mut g, p);
        named(&mut g, p, NAME, Zone::Battlefield);
        let kirol = named(&mut g, p, "Kirol, Attentive First-Year", Zone::Battlefield);
        let witness = named(&mut g, p, "Bear Cub", Zone::Battlefield);
        let spell = named(&mut g, p, "Shock", Zone::Hand);
        g.state.players[p.index()].mana = [30; 6];
        cast(&mut g, spell, None, &[Target::Player(p.other())]);
        reach(&mut g, |g| {
            g.state.stack.last().is_some_and(|s| s.kind.is_triggered())
        });
        let trigger = g.state.stack.last().unwrap().id;
        checked(
            &mut g,
            Action::AnnounceActivate {
                card: kirol,
                ability: 0,
                counter_payment: Counters::default(),
            },
        );
        checked(
            &mut g,
            Action::ChooseTarget {
                target: Target::Stack(trigger),
            },
        );
        for id in [kirol, witness] {
            let object = g.object_ref(id);
            checked(&mut g, Action::ChooseAdditionalCost { object });
        }
        checked(&mut g, Action::PayCosts { payment: [0; 6] });
        reach(&mut g, |g| {
            g.state
                .stack
                .iter()
                .any(|s| s.is_copy && s.kind.is_triggered())
        });
        assert!(
            g.state
                .stack
                .iter()
                .any(|s| s.is_copy && s.cast_event.is_some())
        );
        settle(&mut g);
        assert_eq!(g.counter_count(spell, CounterKind::Dream), 1);
        assert_eq!(g.observe(p).dream_exiles, g.observe(p.other()).dream_exiles);
    }
}
