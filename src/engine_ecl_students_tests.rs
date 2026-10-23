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

const ABIGALE_KINDS: [CounterKind; 3] = [
    CounterKind::Flying,
    CounterKind::FirstStrike,
    CounterKind::Lifelink,
];
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
            for kind in ABIGALE_KINDS {
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
        for kind in ABIGALE_KINDS {
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
        for kind in ABIGALE_KINDS {
            assert_eq!(amounts.get(kind), 1);
        }
        assert_eq!(view.effects.len(), 2);
        settle(&mut g);
        for kind in ABIGALE_KINDS {
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

fn sanar_library(g: &mut Game, p: Player, names: &[&str]) -> Vec<CardId> {
    while let Some(&id) = g.state.players[p.index()].library.last() {
        g.move_object(id, Zone::Exile);
    }
    let ids = names
        .iter()
        .map(|name| named(g, p, name, Zone::Library))
        .collect::<Vec<_>>();
    g.state.players[p.index()].library.reverse();
    ids
}
fn start_sanar(g: &mut Game, source: CardId, leave: bool) {
    let p = g.state.active;
    g.begin_step(Step::Main1);
    g.prepare_triggers();
    assert_eq!(g.state.stack.len(), 1);
    if leave {
        g.move_object(source, Zone::Exile);
    }
    checked(g, Action::Pass);
    checked(g, Action::Pass);
    assert_eq!(g.state.active, p);
}

#[test]
fn ecl_student_sanar_three_hybrid_routes_body_and_only_own_first_main_trigger() {
    for p in [Player::P0, Player::P1] {
        for cost in [0, 1, 2] {
            let mut g = setup(p);
            let source = named(&mut g, p, "Sanar, Innovative First-Year", Zone::Hand);
            g.state.players[p.index()].mana = match cost {
                0 => [0, 0, 0, 2, 0, 2],
                1 => [0, 1, 0, 1, 0, 2],
                _ => [0, 2, 0, 0, 0, 2],
            };
            checked(&mut g, Action::AnnounceCast { card: source });
            checked(&mut g, Action::ChooseSpellCost { cost });
            pay(&mut g);
            settle(&mut g);
            assert_eq!(g.state.players[p.index()].mana, [0; 6]);
            assert_eq!(
                g.stats(source),
                Stats {
                    power: 2,
                    toughness: 4
                }
            );
            g.begin_step(Step::Main2);
            assert!(g.state.pending_triggers.is_empty());
            g.state.active = p.other();
            g.begin_step(Step::Main1);
            assert!(g.state.pending_triggers.is_empty());
        }
    }
}
#[test]
fn ecl_student_sanar_reveals_through_lands_and_freezes_colors_for_nongreedy_assignment() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let source = named(&mut g, p, "Sanar, Innovative First-Year", Zone::Battlefield);
        let ids = sanar_library(
            &mut g,
            p,
            &[
                "Forest",
                "Sanar, Innovative First-Year",
                "Island",
                "Opt",
                "Shock",
            ],
        );
        start_sanar(&mut g, source, false);
        for viewer in [p, p.other()] {
            let view = g.observe(viewer).library_selection.unwrap();
            assert!(view.public && view.reveal);
            assert_eq!(view.available_colors, Some(10));
            assert_eq!(view.maximum, 2);
            assert_eq!(
                view.remaining.iter().map(|v| v.id).collect::<Vec<_>>(),
                ids[..4]
            );
            assert_eq!(
                view.candidates.iter().map(|r| r.card).collect::<Vec<_>>(),
                vec![ids[1], ids[3]]
            );
        }
        reject(&mut g, Action::SelectLibrary { card: ids[0] });
        reject(&mut g, Action::SelectLibrary { card: ids[4] });
        checked(&mut g, Action::SelectLibrary { card: ids[1] });
        reject(&mut g, Action::SelectLibrary { card: ids[1] });
        // The first multicolored card must be assignable to red, leaving blue
        // for Opt; choosing its first color greedily would reject a legal set.
        g.move_object(source, Zone::Exile);
        g.move_object(source, Zone::Battlefield);
        g.set_color_effects(&[source], 0, AnimationDuration::Permanent);
        assert_eq!(
            g.observe(p).library_selection.unwrap().available_colors,
            Some(10)
        );
        checked(&mut g, Action::SelectLibrary { card: ids[3] });
        checked(&mut g, Action::FinishLibrarySelection);
        settle(&mut g);
        for &id in &[ids[1], ids[3]] {
            assert_eq!(g.object_view(id).zone, Zone::Exile);
            assert!(g.can_cast_exiled(p, id));
            assert!(!g.can_cast_exiled(p.other(), id));
            assert!(!g.can_play_exiled(p, id));
        }
        assert_eq!(g.state.players[p.index()].library.len(), 3);
    }
}
#[test]
fn ecl_student_sanar_color_matching_rejects_hall_deficiency_not_just_excess_count() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let source = named(&mut g, p, "Sanar, Innovative First-Year", Zone::Battlefield);
        named(&mut g, p, "Savannah Lions", Zone::Battlefield);
        let ids = sanar_library(
            &mut g,
            p,
            &[
                "Sanar, Innovative First-Year",
                "Sanar, Innovative First-Year",
                "Opt",
            ],
        );
        start_sanar(&mut g, source, false);
        assert_eq!(g.observe(p).library_selection.unwrap().maximum, 3);
        checked(&mut g, Action::SelectLibrary { card: ids[0] });
        checked(&mut g, Action::SelectLibrary { card: ids[1] });
        // Three cards whose available colors are only blue/red cannot consume
        // the white slot, even though all three individually have allowed colors.
        reject(&mut g, Action::SelectLibrary { card: ids[2] });
        assert!(
            g.observe(p.other())
                .library_selection
                .unwrap()
                .candidates
                .is_empty()
        );
        checked(&mut g, Action::FinishLibrarySelection);
        settle(&mut g);
        assert_eq!(g.object_view(ids[2]).zone, Zone::Library);
    }
}
#[test]
fn ecl_student_sanar_wrong_color_and_duplicate_monocolored_choices_are_atomic() {
    for p in [Player::P0, Player::P1] {
        for names in [
            ["Opt", "Negate"],
            ["Shock", "Shock"],
            ["Bear Cub", "Springleaf Drum"],
        ] {
            let mut g = setup(p);
            let source = named(&mut g, p, "Sanar, Innovative First-Year", Zone::Battlefield);
            let ids = sanar_library(&mut g, p, &names);
            start_sanar(&mut g, source, false);
            if names[0] == "Bear Cub" {
                for id in ids {
                    reject(&mut g, Action::SelectLibrary { card: id });
                }
            } else {
                checked(&mut g, Action::SelectLibrary { card: ids[0] });
                reject(&mut g, Action::SelectLibrary { card: ids[1] });
            }
            checked(&mut g, Action::FinishLibrarySelection);
            settle(&mut g);
        }
    }
}
#[test]
fn ecl_student_sanar_resolution_color_changes_distinct_all_permanents_and_zero_still_shuffle() {
    for p in [Player::P0, Player::P1] {
        for zero in [false, true] {
            let mut g = setup(p);
            let source = named(&mut g, p, "Sanar, Innovative First-Year", Zone::Battlefield);
            let ids = sanar_library(&mut g, p, &["Opt", "Shock", "Bear Cub", "Swamp"]);
            g.begin_step(Step::Main1);
            g.prepare_triggers();
            if zero {
                g.move_object(source, Zone::Exile);
            } else {
                let enchantment = named(&mut g, p, "Morcant's Eyes", Zone::Battlefield);
                g.set_color_effects(&[enchantment], 16, AnimationDuration::Permanent);
                named(&mut g, p, "Bear Cub", Zone::Battlefield);
                named(&mut g, p, "Bear Cub", Zone::Battlefield);
            }
            g.remember_library_top(p, g.object_ref(ids[0]));
            let mut rng = g.state.rng;
            let mut expected = g.state.players[p.index()].library.clone();
            rng.shuffle(&mut expected);
            checked(&mut g, Action::Pass);
            checked(&mut g, Action::Pass);
            if !zero {
                let view = g.observe(p).library_selection.unwrap();
                assert_eq!(view.available_colors, Some(26));
                assert_eq!(view.maximum, 3);
                assert_eq!(view.remaining.len(), 3);
                checked(&mut g, Action::FinishLibrarySelection);
                settle(&mut g);
            }
            assert_eq!(g.state.rng, rng);
            assert_eq!(g.state.players[p.index()].library, expected);
            assert!(g.observe(p).library_selection.is_none());
            assert!(g.observe(p).exile_permissions.is_empty());
        }
    }
}
#[test]
fn ecl_student_sanar_all_five_colors_short_library_and_optional_decline() {
    for p in [Player::P0, Player::P1] {
        for short in [false, true] {
            let mut g = setup(p);
            let source = named(&mut g, p, "Sanar, Innovative First-Year", Zone::Battlefield);
            g.set_color_effects(&[source], 31, AnimationDuration::Permanent);
            let names = if short {
                vec!["Opt", "Shock"]
            } else {
                vec![
                    "Forest",
                    "Savannah Lions",
                    "Opt",
                    "Duress",
                    "Shock",
                    "Bear Cub",
                    "Island",
                ]
            };
            let ids = sanar_library(&mut g, p, &names);
            start_sanar(&mut g, source, false);
            let view = g.observe(p).library_selection.unwrap();
            assert_eq!(view.available_colors, Some(31));
            assert_eq!(view.remaining.len(), if short { 2 } else { 6 });
            assert_eq!(view.maximum, if short { 2 } else { 5 });
            if !short {
                for &id in &ids[1..6] {
                    checked(&mut g, Action::SelectLibrary { card: id });
                }
            }
            checked(&mut g, Action::FinishLibrarySelection);
            settle(&mut g);
            assert_eq!(g.state.exile_permissions.len(), if short { 0 } else { 5 });
            assert_eq!(g.state.players[p.index()].library.len(), 2);
        }
    }
}
#[test]
fn ecl_student_sanar_casts_require_mana_obey_timing_and_expire_with_exact_exile_identity() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let source = named(&mut g, p, "Sanar, Innovative First-Year", Zone::Battlefield);
        let ids = sanar_library(&mut g, p, &["Opt", "Shock", "Forest"]);
        start_sanar(&mut g, source, false);
        checked(&mut g, Action::SelectLibrary { card: ids[0] });
        checked(&mut g, Action::FinishLibrarySelection);
        settle(&mut g);
        g.state.players[p.index()].mana = [0; 6];
        reject(&mut g, Action::AnnounceExile { card: ids[0] });
        g.state.players[p.index()].mana = [0, 1, 0, 0, 0, 0];
        checked(&mut g, Action::AnnounceExile { card: ids[0] });
        pay(&mut g);
        settle(&mut g);
        assert_eq!(g.state.players[p.index()].mana, [0; 6]);
        assert_eq!(g.object_view(ids[0]).zone, Zone::Graveyard);
        assert!(!g.can_cast_exiled(p, ids[0]));
        g.move_object(ids[0], Zone::Exile);
        assert!(!g.can_cast_exiled(p, ids[0]));
        let creature = named(&mut g, p, "Bear Cub", Zone::Exile);
        g.grant_exile_cast_permission(p, g.object_ref(creature));
        g.state.priority = p.other();
        reject(&mut g, Action::AnnounceExile { card: creature });
        g.state.priority = p;
        g.state.step = Step::End;
        reject(&mut g, Action::AnnounceExile { card: creature });
        super::ability_timeline_tests::next_turn(&mut g);
        assert!(!g.can_cast_exiled(p, creature));
        assert!(g.state.exile_permissions.is_empty());
    }
}
#[test]
fn ecl_student_sanar_cast_only_permissions_never_allow_land_plays_but_ordinary_permissions_still_do()
 {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let land = named(&mut g, p, "Forest", Zone::Exile);
        g.grant_exile_cast_permission(p, g.object_ref(land));
        assert!(g.can_cast_exiled(p, land));
        assert!(!g.can_play_exiled(p, land));
        reject(&mut g, Action::PlayLand { card: land });
        reject(&mut g, Action::AnnounceExile { card: land });
        // The independent ordinary permission remains usable if also granted.
        g.grant_exile_play_permission(p, g.object_ref(land), false);
        assert!(g.can_play_exiled(p, land));
        checked(&mut g, Action::PlayLand { card: land });
        assert_eq!(g.object_view(land).zone, Zone::Battlefield);
    }
}
