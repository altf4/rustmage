//! Paid first-year student interactions with owned raw/index branching and restore.
use super::ability_timeline_tests::{cast, checked, named, settle, setup};
use super::ability_world::CharacteristicWorld;
use super::*;
use crate::cards::AnimationDuration;
fn reject(g: &mut Game, action: Action) {
    let before = g.snapshot();
    let revision = g.revision();
    assert_eq!(
        g.apply(g.actor().unwrap(), action),
        Err(Error::IllegalAction)
    );
    assert_eq!(g.snapshot(), before);
    assert_eq!(g.revision(), revision);
}
fn pay(g: &mut Game) {
    let action = g
        .legal_actions()
        .into_iter()
        .find(|a| matches!(a, Action::PayCosts { .. }))
        .unwrap();
    checked(g, action);
}
fn activate(g: &mut Game, source: CardId, target: CardId) {
    checked(
        g,
        Action::AnnounceActivate {
            card: source,
            ability: 0,
            counter_payment: Counters::default(),
        },
    );
    checked(
        g,
        Action::ChooseTarget {
            target: Target::Card(target),
        },
    );
    pay(g);
}
#[test]
fn ecl_student_tam_both_hybrid_payments_tap_cost_self_target_and_source_departure() {
    for p in [Player::P0, Player::P1] {
        for cost in [0, 1] {
            let mut g = setup(p);
            let tam = named(&mut g, p, "Tam, Mindful First-Year", Zone::Hand);
            let mut mana = [0; 6];
            mana[if cost == 0 { 4 } else { 1 }] = 1;
            mana[5] = 1;
            g.state.players[p.index()].mana = mana;
            checked(&mut g, Action::AnnounceCast { card: tam });
            checked(&mut g, Action::ChooseSpellCost { cost });
            pay(&mut g);
            settle(&mut g);
            assert_eq!(g.state.players[p.index()].mana, [0; 6]);
            assert_eq!(
                g.stats(tam),
                Stats {
                    power: 2,
                    toughness: 2
                }
            );
            assert_eq!(g.colors(tam), 18);
            let action = Action::AnnounceActivate {
                card: tam,
                ability: 0,
                counter_payment: Counters::default(),
            };
            reject(&mut g, action);
            g.state.objects[tam as usize].summoning_sick = false;
            activate(&mut g, tam, tam);
            assert!(g.object_view(tam).tapped);
            settle(&mut g);
            assert_eq!(g.colors(tam), 31);
            assert!(!g.keywords(tam).contains(Keywords::HEXPROOF_OWN_COLORS));
            g.state.objects[tam as usize].tapped = false;
            let bear = named(&mut g, p, "Bear Cub", Zone::Battlefield);
            let foe = named(&mut g, p.other(), "Bear Cub", Zone::Battlefield);
            checked(
                &mut g,
                Action::AnnounceActivate {
                    card: tam,
                    ability: 0,
                    counter_payment: Counters::default(),
                },
            );
            reject(
                &mut g,
                Action::ChooseTarget {
                    target: Target::Card(foe),
                },
            );
            checked(
                &mut g,
                Action::ChooseTarget {
                    target: Target::Card(bear),
                },
            );
            pay(&mut g);
            g.move_object(tam, Zone::Exile);
            settle(&mut g);
            assert_eq!(g.colors(bear), 31);
            assert!(!g.keywords(bear).contains(Keywords::HEXPROOF_OWN_COLORS));
            assert!(g.expire_color_effects());
            assert_eq!(g.colors(bear), 16);
        }
    }
}
#[test]
fn ecl_student_tam_dynamic_own_color_hexproof_filters_opponent_sources_only() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let tam = named(&mut g, p, "Tam, Mindful First-Year", Zone::Battlefield);
        let bear = named(&mut g, p, "Bear Cub", Zone::Battlefield);
        let foreign = named(&mut g, p.other(), "Bear Cub", Zone::Battlefield);
        let sources = ["Pacifism", "Opt", "Duress", "Shock", "Snakeskin Veil"]
            .map(|name| named(&mut g, p.other(), name, Zone::Hand));
        assert!(g.keywords(bear).contains(Keywords::HEXPROOF_OWN_COLORS));
        assert!(!g.keywords(tam).contains(Keywords::HEXPROOF_OWN_COLORS));
        assert!(!g.keywords(foreign).contains(Keywords::HEXPROOF_OWN_COLORS));
        for colors in [0, 1, 2, 4, 8, 16, 31] {
            g.set_color_effects(&[bear], colors, AnimationDuration::Permanent);
            for source in sources {
                let reference = g.object_ref(source);
                let source_colors = g.state.objects[source as usize].card.colors();
                assert_eq!(
                    g.cast_target_matches(
                        TargetRule::Creature,
                        p.other(),
                        reference,
                        Target::Card(bear)
                    ),
                    colors & source_colors == 0
                );
                assert!(g.cast_target_matches(
                    TargetRule::Creature,
                    p,
                    reference,
                    Target::Card(bear)
                ));
                assert!(g.cast_target_matches(
                    TargetRule::Creature,
                    p.other(),
                    reference,
                    Target::Card(tam)
                ));
            }
        }
        checked(&mut g, Action::Pass);
    }
}
#[test]
fn ecl_student_tam_paid_response_makes_previously_legal_shock_target_illegal() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let tam = named(&mut g, p, "Tam, Mindful First-Year", Zone::Battlefield);
        g.state.objects[tam as usize].summoning_sick = false;
        let bear = named(&mut g, p, "Bear Cub", Zone::Battlefield);
        let shock = named(&mut g, p.other(), "Shock", Zone::Hand);
        g.state.priority = p.other();
        cast(&mut g, shock, None, &[Target::Card(bear)]);
        checked(&mut g, Action::Pass);
        activate(&mut g, tam, bear);
        settle(&mut g);
        assert_eq!(g.colors(bear), 31);
        assert!(g.on_battlefield(bear));
        assert_eq!(g.object_view(bear).damage, 0);
        assert_eq!(g.object_view(shock).zone, Zone::Graveyard);
    }
}
#[test]
fn ecl_student_tam_source_and_recipient_ability_removal_and_control_changes_use_timeline() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let tam = named(&mut g, p, "Tam, Mindful First-Year", Zone::Battlefield);
        let bear = named(&mut g, p, "Bear Cub", Zone::Battlefield);
        let foreign = named(&mut g, p.other(), "Bear Cub", Zone::Battlefield);
        let stop = named(&mut g, p, "Stop Cold", Zone::Hand);
        cast(&mut g, stop, None, &[Target::Card(tam)]);
        settle(&mut g);
        assert!(!g.keywords(bear).contains(Keywords::HEXPROOF_OWN_COLORS));
        g.move_object(stop, Zone::Graveyard);
        assert!(g.keywords(bear).contains(Keywords::HEXPROOF_OWN_COLORS));
        let witness = named(&mut g, p, "Witness Protection", Zone::Hand);
        cast(&mut g, witness, None, &[Target::Card(bear)]);
        settle(&mut g);
        assert!(!g.keywords(bear).contains(Keywords::HEXPROOF_OWN_COLORS));
        g.move_object(tam, Zone::Exile);
        g.move_object(tam, Zone::Battlefield);
        assert!(g.keywords(bear).contains(Keywords::HEXPROOF_OWN_COLORS));
        let offering = named(&mut g, p, "Harmless Offering", Zone::Hand);
        cast(
            &mut g,
            offering,
            None,
            &[Target::Player(p.other()), Target::Card(tam)],
        );
        settle(&mut g);
        assert!(!g.keywords(bear).contains(Keywords::HEXPROOF_OWN_COLORS));
        assert!(g.keywords(foreign).contains(Keywords::HEXPROOF_OWN_COLORS));
        assert!(!g.keywords(tam).contains(Keywords::HEXPROOF_OWN_COLORS));
    }
}
#[test]
fn ecl_student_tam_departed_ability_source_uses_frozen_colors_without_rebinding_new_incarnation() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let tam = named(&mut g, p, "Tam, Mindful First-Year", Zone::Battlefield);
        g.state.objects[tam as usize].summoning_sick = false;
        let bear = named(&mut g, p, "Bear Cub", Zone::Battlefield);
        let firebrand = named(&mut g, p.other(), "Fanatical Firebrand", Zone::Battlefield);
        let old = g.object_ref(firebrand);
        g.state.priority = p.other();
        activate(&mut g, firebrand, bear);
        assert_eq!(g.object_view(firebrand).zone, Zone::Graveyard);
        g.move_object(firebrand, Zone::Battlefield);
        g.set_color_effects(&[firebrand], 0, AnimationDuration::Permanent);
        assert_ne!(g.object_ref(firebrand), old);
        checked(&mut g, Action::Pass);
        activate(&mut g, tam, bear);
        settle(&mut g);
        assert_eq!(g.colors(firebrand), 0);
        assert_eq!(g.colors(bear), 31);
        assert_eq!(g.object_view(bear).damage, 0);
    }
}

const ABIGALE_COUNTERS: Keywords = Keywords::FLYING
    .union(Keywords::FIRST_STRIKE)
    .union(Keywords::LIFELINK);
fn abigale_trigger(g: &mut Game, cost: u16) -> CardId {
    let p = g.state.active;
    let abigale = named(g, p, "Abigale, Eloquent First-Year", Zone::Hand);
    checked(g, Action::AnnounceCast { card: abigale });
    checked(g, Action::ChooseSpellCost { cost });
    pay(g);
    for _ in 0..12 {
        if g.legal_actions().contains(&Action::FinishTargets) {
            reject(
                g,
                Action::ChooseTarget {
                    target: Target::Card(abigale),
                },
            );
            return abigale;
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
    panic!("missing Abigale entry trigger");
}
fn abigale_on(g: &mut Game, target: CardId) -> CardId {
    let abigale = abigale_trigger(g, 0);
    checked(
        g,
        Action::ChooseTarget {
            target: Target::Card(target),
        },
    );
    settle(g);
    abigale
}
#[test]
fn ecl_student_abigale_three_hybrid_routes_intrinsic_keywords_and_optional_other_target() {
    for p in [Player::P0, Player::P1] {
        for cost in [0, 1, 2] {
            let mut g = setup(p);
            g.state.players[p.index()].mana = match cost {
                0 => [0, 0, 2, 0, 0, 0],
                1 => [1, 0, 1, 0, 0, 0],
                _ => [2, 0, 0, 0, 0, 0],
            };
            let spider = named(&mut g, p.other(), "Skybeast Tracker", Zone::Battlefield);
            let abigale = abigale_trigger(&mut g, cost);
            checked(&mut g, Action::FinishTargets);
            settle(&mut g);
            assert_eq!(g.state.players[p.index()].mana, [0; 6]);
            assert_eq!(g.keywords(abigale), ABIGALE_COUNTERS);
            assert_eq!(
                g.stats(abigale),
                Stats {
                    power: 1,
                    toughness: 1
                }
            );
            assert!(g.keywords(spider).contains(Keywords::REACH));
            assert_eq!(g.counter_count(spider, CounterKind::Flying), 0);
        }
    }
}
#[test]
fn ecl_student_abigale_removes_intrinsic_and_activated_abilities_but_counters_survive_source() {
    for p in [Player::P0, Player::P1] {
        for controller in [p, p.other()] {
            let mut g = setup(p);
            let source = named(&mut g, controller, "Fanatical Firebrand", Zone::Battlefield);
            let abigale = abigale_on(&mut g, source);
            assert_eq!(g.keywords(source), ABIGALE_COUNTERS);
            for kind in keyword_counters::KINDS {
                assert_eq!(g.counter_count(source, kind), 1);
            }
            g.state.priority = controller;
            reject(
                &mut g,
                Action::AnnounceActivate {
                    card: source,
                    ability: 0,
                    counter_payment: Counters::default(),
                },
            );
            g.move_object(abigale, Zone::Exile);
            assert_eq!(g.keywords(source), ABIGALE_COUNTERS);
            assert!(!g.printed_abilities_available(source));
            checked(&mut g, Action::Pass);
            let old = g.object_ref(source);
            g.move_object(source, Zone::Exile);
            g.move_object(source, Zone::Battlefield);
            assert_ne!(old, g.object_ref(source));
            assert_eq!(g.counter_count(source, CounterKind::Flying), 0);
            assert_eq!(g.keywords(source), Keywords::HASTE);
            assert!(g.printed_abilities_available(source));
            checked(&mut g, Action::Pass);
        }
    }
}
#[test]
fn ecl_student_abigale_partial_counter_removal_preserves_timestamp_addition_renews_kind_only() {
    // CR 613.7c: adding another of the same kind renews all of that kind.
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        named(&mut g, p, "Doubling Season", Zone::Battlefield);
        let spider = named(&mut g, p, "Skybeast Tracker", Zone::Battlefield);
        abigale_on(&mut g, spider);
        for kind in keyword_counters::KINDS {
            assert_eq!(g.counter_count(spider, kind), 2);
        }
        let witness = named(&mut g, p, "Witness Protection", Zone::Hand);
        cast(&mut g, witness, None, &[Target::Card(spider)]);
        settle(&mut g);
        assert_eq!(g.keywords(spider), Keywords::NONE);
        let timestamp = |g: &Game| {
            g.state
                .roles
                .as_ref()
                .unwrap()
                .keyword_effects
                .iter()
                .find(|row| {
                    row.recipient == g.object_ref(spider)
                        && row.origin
                            == KeywordEffectOrigin::Counter {
                                kind: CounterKind::Flying,
                            }
                })
                .unwrap()
                .timestamp
        };
        let old = timestamp(&g);
        g.pay_counters(spider, &Counters::of(CounterKind::Flying, 1));
        assert_eq!(timestamp(&g), old);
        assert_eq!(g.keywords(spider), Keywords::NONE);
        g.add_counter(spider, CounterKind::Flying, 1);
        assert!(timestamp(&g) > old);
        assert_eq!(g.keywords(spider), Keywords::FLYING);
        let renewed = timestamp(&g);
        g.pay_counters(spider, &Counters::of(CounterKind::Flying, 1));
        assert_eq!(timestamp(&g), renewed);
        assert_eq!(g.keywords(spider), Keywords::FLYING);
        g.pay_counters(spider, &Counters::of(CounterKind::Flying, 1));
        assert_eq!(g.keywords(spider), Keywords::NONE);
        assert!(
            !g.state
                .roles
                .as_ref()
                .unwrap()
                .keyword_effects
                .iter()
                .any(|row| row.recipient == g.object_ref(spider)
                    && row.origin
                        == KeywordEffectOrigin::Counter {
                            kind: CounterKind::Flying
                        })
        );
        g.move_object(witness, Zone::Graveyard);
        assert_eq!(
            g.keywords(spider),
            Keywords::FIRST_STRIKE.union(Keywords::LIFELINK)
        );
        checked(&mut g, Action::Pass);
    }
}
#[test]
fn ecl_student_abigale_counter_prohibition_still_removes_original_abilities() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let spider = named(&mut g, p, "Skybeast Tracker", Zone::Battlefield);
        let aura = named(&mut g, p, "Blossombind", Zone::Hand);
        cast(&mut g, aura, None, &[Target::Card(spider)]);
        settle(&mut g);
        abigale_on(&mut g, spider);
        assert_eq!(g.keywords(spider), Keywords::NONE);
        assert!(!g.printed_abilities_available(spider));
        for kind in keyword_counters::KINDS {
            assert_eq!(g.counter_count(spider, kind), 0);
        }
        checked(&mut g, Action::Pass);
    }
}
#[test]
fn ecl_student_abigale_frozen_target_incarnation_and_departed_trigger_source() {
    for p in [Player::P0, Player::P1] {
        for blink_target in [false, true] {
            let mut g = setup(p);
            let spider = named(&mut g, p.other(), "Skybeast Tracker", Zone::Battlefield);
            let abigale = abigale_trigger(&mut g, 0);
            checked(
                &mut g,
                Action::ChooseTarget {
                    target: Target::Card(spider),
                },
            );
            // Finish target announcement before either exact object departs.
            if g.legal_actions().contains(&Action::FinishTargets) {
                checked(&mut g, Action::FinishTargets);
            }
            g.move_object(abigale, Zone::Exile);
            if blink_target {
                g.move_object(spider, Zone::Exile);
                g.move_object(spider, Zone::Battlefield);
            }
            settle(&mut g);
            assert_eq!(
                g.keywords(spider),
                if blink_target {
                    Keywords::REACH
                } else {
                    ABIGALE_COUNTERS
                }
            );
        }
    }
}
#[test]
fn ecl_student_keyword_counters_private_zone_grants_do_not_publish_private_references() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let hand = named(&mut g, p, "Skybeast Tracker", Zone::Hand);
        g.add_counter(hand, CounterKind::Flying, 2);
        assert_eq!(g.keywords(hand), Keywords::REACH.union(Keywords::FLYING));
        assert!(
            !g.observe(p.other())
                .keyword_effects
                .iter()
                .any(|row| row.recipient.card == hand)
        );
        let before = g.snapshot();
        let mut branch = g.clone();
        branch.pay_counters(hand, &Counters::of(CounterKind::Flying, 2));
        assert_eq!(branch.keywords(hand), Keywords::REACH);
        assert!(g.keywords(hand).contains(Keywords::FLYING));
        branch.restore(&before).unwrap();
        assert_eq!(branch.snapshot(), before);
        g.move_object(hand, Zone::Battlefield);
        assert_eq!(g.counter_count(hand, CounterKind::Flying), 0);
        assert_eq!(g.keywords(hand), Keywords::REACH);
        checked(&mut g, Action::Pass);
    }
}
#[test]
fn ecl_student_abigale_counter_overflow_rolls_back_ability_removal_and_observations() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let spider = named(&mut g, p, "Skybeast Tracker", Zone::Battlefield);
        g.set_counters(spider, Counters::of(CounterKind::Flying, u64::MAX));
        abigale_trigger(&mut g, 0);
        checked(
            &mut g,
            Action::ChooseTarget {
                target: Target::Card(spider),
            },
        );
        if g.legal_actions().contains(&Action::FinishTargets) {
            checked(&mut g, Action::FinishTargets);
        }
        checked(&mut g, Action::Pass);
        let before = g.snapshot();
        let revision = g.revision();
        let observations =
            [g.observe(p), g.observe(p.other())].map(|view| serde_json::to_value(view).unwrap());
        assert!(matches!(
            g.apply(g.actor().unwrap(), Action::Pass),
            Err(Error::CapacityExceeded(_))
        ));
        assert_eq!(g.snapshot(), before);
        assert_eq!(g.revision(), revision);
        assert_eq!(
            [g.observe(p), g.observe(p.other())].map(|view| serde_json::to_value(view).unwrap()),
            observations
        );
    }
}

#[test]
fn ecl_student_abigale_flying_blocks_and_first_strike_lifelink_deal_only_once() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let spider = named(&mut g, p, "Skybeast Tracker", Zone::Battlefield);
        let blocker = named(&mut g, p.other(), "Bear Cub", Zone::Battlefield);
        abigale_on(&mut g, spider);
        let life = g.state.players[p.index()].life;
        let opponent_life = g.state.players[p.other().index()].life;
        g.state.objects[spider as usize].summoning_sick = false;
        g.state.step = Step::Attackers;
        g.state.pending = Pending::Attackers;
        checked(&mut g, Action::Attack { card: spider });
        checked(&mut g, Action::FinishAttackers);
        checked(&mut g, Action::Pass);
        checked(&mut g, Action::Pass);
        assert_eq!(g.state.pending, Pending::Blockers);
        reject(
            &mut g,
            Action::Block {
                blocker,
                attacker: spider,
            },
        );
        checked(&mut g, Action::FinishBlockers);
        checked(&mut g, Action::Pass);
        checked(&mut g, Action::Pass);
        assert_eq!(g.state.step, Step::FirstStrikeDamage);
        assert_eq!(g.state.players[p.index()].life, life + 2);
        assert_eq!(g.state.players[p.other().index()].life, opponent_life - 2);
        checked(&mut g, Action::Pass);
        checked(&mut g, Action::Pass);
        assert_eq!(g.state.step, Step::CombatDamage);
        assert_eq!(g.state.players[p.index()].life, life + 2);
        assert_eq!(g.state.players[p.other().index()].life, opponent_life - 2);
    }
}
#[test]
fn ecl_student_keyword_counter_grant_consults_borrowed_counter_overlay() {
    struct CounterOverlay<'a> {
        game: &'a Game,
        id: CardId,
        counters: Counters,
    }
    impl CharacteristicWorld for CounterOverlay<'_> {
        fn game(&self) -> &Game {
            self.game
        }
        fn object(&self, id: CardId) -> &Object {
            &self.game.state.objects[id as usize]
        }
        fn counter_amounts(&self, id: CardId) -> CounterAmounts<'_> {
            if id == self.id {
                CounterAmounts {
                    hot: self.counters.hot(),
                    named: self.counters.named.as_deref(),
                }
            } else {
                self.game.counter_amounts(id)
            }
        }
        fn life(&self, player: Player) -> i128 {
            self.game.state.players[player.index()].life
        }
        fn gained_life(&self, player: Player) -> bool {
            self.game.state.players[player.index()].life_gained_this_turn > 0
        }
        fn attacking(&self, id: CardId) -> bool {
            self.game.state.attackers.iter().any(|a| a.card == id)
        }
    }
    let mut g = setup(Player::P0);
    let spider = named(&mut g, Player::P0, "Skybeast Tracker", Zone::Battlefield);
    abigale_on(&mut g, spider);
    let before = g.snapshot();
    let overlay = CounterOverlay {
        game: &g,
        id: spider,
        counters: Counters::of(CounterKind::FirstStrike, 1).with(CounterKind::Lifelink, 1),
    };
    assert_eq!(
        ability_evaluation::evaluate_abilities(&overlay)
            .result
            .objects[spider as usize]
            .keywords,
        Keywords::FIRST_STRIKE.union(Keywords::LIFELINK)
    );
    assert_eq!(g.keywords(spider), ABIGALE_COUNTERS);
    assert_eq!(g.snapshot(), before);
}

#[test]
fn ecl_student_abigale_places_three_kinds_in_one_replaceable_event() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        named(&mut g, p, "Doubling Season", Zone::Battlefield);
        named(&mut g, p, "Doubling Season", Zone::Battlefield);
        let creature = named(&mut g, p, "Skybeast Tracker", Zone::Battlefield);
        abigale_trigger(&mut g, 0);
        checked(
            &mut g,
            Action::ChooseTarget {
                target: Target::Card(creature),
            },
        );
        if g.legal_actions().contains(&Action::FinishTargets) {
            checked(&mut g, Action::FinishTargets);
        }
        checked(&mut g, Action::Pass);
        checked(&mut g, Action::Pass);
        let view = g.observe(p).replacement_choice.unwrap();
        let amounts = view.counters.unwrap();
        for kind in keyword_counters::KINDS {
            assert_eq!(amounts.get(kind), 1);
        }
        assert_eq!(view.effects.len(), 2);
        settle(&mut g);
        for kind in keyword_counters::KINDS {
            assert_eq!(g.counter_count(creature, kind), 4);
        }
        assert_eq!(g.keywords(creature), ABIGALE_COUNTERS);
    }
}
#[test]
fn ecl_student_abigale_counters_and_resolution_removal_are_not_copiable_values() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let firebrand = named(&mut g, p, "Fanatical Firebrand", Zone::Battlefield);
        abigale_on(&mut g, firebrand);
        let recipe = g.copiable_recipe(g.object_ref(firebrand));
        g.stage_copy_creation(p, recipe, 1);
        settle(&mut g);
        let copy = g.state.copy_recipes.as_ref().unwrap()[0].object.card;
        assert_eq!(g.counter_count(copy, CounterKind::Flying), 0);
        assert_eq!(g.keywords(copy), Keywords::HASTE);
        assert!(g.printed_abilities_available(copy));
        assert_eq!(g.keywords(firebrand), ABIGALE_COUNTERS);
        assert!(!g.printed_abilities_available(firebrand));
        checked(&mut g, Action::Pass);
    }
}
