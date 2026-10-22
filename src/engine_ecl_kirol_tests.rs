//! Paid external tap costs and owned copies of actual controlled trigger packets.
use super::ability_timeline_tests::{cast, checked, named, settle, setup};
use super::*;
fn reject(g: &mut Game, a: Action) {
    let before = g.snapshot();
    let r = g.revision();
    assert_eq!(g.apply(g.actor().unwrap(), a), Err(Error::IllegalAction));
    assert_eq!(g.snapshot(), before);
    assert_eq!(g.revision(), r);
}
fn paid(g: &mut Game, p: Player, white: u32) -> CardId {
    let id = named(g, p, "Kirol, Attentive First-Year", Zone::Hand);
    let payment = [white, 0, 0, 2 - white, 0, 1];
    g.state.players[p.index()].mana = payment;
    checked(g, Action::AnnounceCast { card: id });
    checked(g, Action::ChooseSpellCost { cost: white as u16 });
    checked(g, Action::PayCosts { payment });
    settle(g);
    id
}
fn to_trigger(g: &mut Game, source: CardId, target: Option<Target>) -> SpellView {
    for _ in 0..30 {
        if let Some(item) = g
            .state
            .stack
            .last()
            .filter(|s| s.kind.is_triggered() && s.card == source)
        {
            return item.clone();
        }
        let actions = g.legal_actions();
        let a = target
            .and_then(|t| {
                actions
                    .iter()
                    .find(|a| **a == Action::ChooseTarget { target: t })
            })
            .or_else(|| {
                actions.iter().find(|a| {
                    matches!(
                        a,
                        Action::FinishTargets | Action::PutTrigger { .. } | Action::Pass
                    )
                })
            })
            .unwrap()
            .clone();
        checked(g, a);
    }
    panic!("trigger window");
}
fn lieutenant(g: &mut Game, p: Player, target: CardId) -> SpellView {
    g.state.players[p.index()].mana = [30; 6];
    let source = named(g, p, "Thoughtweft Lieutenant", Zone::Hand);
    cast(g, source, None, &[]);
    to_trigger(g, source, Some(Target::Card(target)))
}
fn activate(g: &mut Game, id: CardId, trigger: u64, witnesses: [CardId; 2]) {
    checked(
        g,
        Action::AnnounceActivate {
            card: id,
            ability: 0,
            counter_payment: Counters::default(),
        },
    );
    checked(
        g,
        Action::ChooseTarget {
            target: Target::Stack(trigger),
        },
    );
    for card in witnesses {
        let object = g.object_ref(card);
        checked(g, Action::ChooseAdditionalCost { object });
    }
    checked(g, Action::PayCosts { payment: [0; 6] });
}
fn to_retarget(g: &mut Game) {
    for _ in 0..10 {
        if matches!(g.state.pending, Pending::Retarget { .. }) {
            return;
        }
        checked(g, Action::Pass);
    }
    panic!("no retarget");
}
#[test]
fn ecl_kirol_paid_hybrids_copy_and_retarget_without_allocating_a_source_and_once_each_turn() {
    for p in [Player::P0, Player::P1] {
        for white in 0..=2 {
            let mut g = setup(p);
            let id = paid(&mut g, p, white);
            let a = named(&mut g, p, "Bear Cub", Zone::Battlefield);
            let b = named(&mut g, p, "Bear Cub", Zone::Battlefield);
            let trigger = lieutenant(&mut g, p, a);
            let object_count = g.state.objects.len();
            activate(&mut g, id, trigger.id, [id, b]);
            assert!(g.object_view(id).tapped && g.object_view(b).tapped);
            g.move_object(id, Zone::Graveyard);
            to_retarget(&mut g);
            let view = g.retarget_view().unwrap();
            assert!(view.item.is_copy);
            assert_eq!(view.item.kind, trigger.kind);
            assert_eq!(view.item.object, trigger.object);
            assert_eq!(view.item.source_card, trigger.source_card);
            assert_eq!(g.state.objects.len(), object_count);
            checked(
                &mut g,
                Action::SetStackTarget {
                    index: 0,
                    target: Target::Card(b),
                },
            );
            checked(&mut g, Action::FinishStackTargets);
            settle(&mut g);
            assert_eq!(g.stats(a).power, 3);
            assert_eq!(g.stats(b).power, 3);
            assert!(
                g.keywords(a).contains(Keywords::TRAMPLE)
                    && g.keywords(b).contains(Keywords::TRAMPLE)
            );
        }
    }
}
#[test]
fn ecl_kirol_fresh_self_and_fresh_creatures_pay_external_taps_but_not_duplicate_or_noncreatures() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let id = paid(&mut g, p, 0);
        let bear = named(&mut g, p, "Bear Cub", Zone::Battlefield);
        g.state.objects[bear as usize].summoning_sick = true;
        let artifact = named(&mut g, p, "Firdoch Core", Zone::Battlefield);
        let enemy = named(&mut g, p.other(), "Bear Cub", Zone::Battlefield);
        let trigger = lieutenant(&mut g, p, bear);
        checked(
            &mut g,
            Action::AnnounceActivate {
                card: id,
                ability: 0,
                counter_payment: Counters::default(),
            },
        );
        checked(
            &mut g,
            Action::ChooseTarget {
                target: Target::Stack(trigger.id),
            },
        );
        let object = g.object_ref(id);
        checked(&mut g, Action::ChooseAdditionalCost { object });
        reject(&mut g, Action::ChooseAdditionalCost { object });
        for card in [artifact, enemy] {
            let object = g.object_ref(card);
            reject(&mut g, Action::ChooseAdditionalCost { object });
        }
        reject(&mut g, Action::PayCosts { payment: [0; 6] });
        let object = g.object_ref(bear);
        checked(&mut g, Action::ChooseAdditionalCost { object });
        checked(&mut g, Action::PayCosts { payment: [0; 6] });
        assert!(g.object_view(id).summoning_sick && g.object_view(bear).summoning_sick);
        to_retarget(&mut g);
        checked(&mut g, Action::FinishStackTargets);
        g.state.objects[id as usize].tapped = false;
        g.state.objects[bear as usize].tapped = false;
        assert!(
            !g.legal_actions()
                .iter()
                .any(|a| matches!(a,Action::AnnounceActivate{card,..}if *card==id)),
            "once applies despite untapping and a remaining legal trigger"
        );
        settle(&mut g);
    }
}
#[test]
fn ecl_kirol_rejects_foreign_triggers_spells_and_activated_abilities() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let id = paid(&mut g, p, 1);
        let bear = named(&mut g, p, "Bear Cub", Zone::Battlefield);
        let trigger = lieutenant(&mut g, p, bear);
        let enemy = named(&mut g, p.other(), "Deepway Navigator", Zone::Hand);
        g.move_object(enemy, Zone::Battlefield);
        g.prepare_triggers();
        let enemy_trigger = g
            .state
            .stack
            .iter()
            .find(|s| s.kind.is_triggered() && s.card == enemy)
            .unwrap()
            .id;
        g.state.priority = p;
        g.state.players[p.index()].mana = [30; 6];
        let spell = named(&mut g, p, "Shock", Zone::Hand);
        cast(&mut g, spell, None, &[Target::Player(p.other())]);
        let spell_id = g.state.stack.last().unwrap().id;
        let extractor = named(&mut g, p, "Scavenging Ooze", Zone::Battlefield);
        let grave = named(&mut g, p, "Bear Cub", Zone::Graveyard);
        checked(
            &mut g,
            Action::AnnounceActivate {
                card: extractor,
                ability: 0,
                counter_payment: Counters::default(),
            },
        );
        checked(
            &mut g,
            Action::ChooseTarget {
                target: Target::Card(grave),
            },
        );
        checked(
            &mut g,
            Action::PayCosts {
                payment: [0, 0, 0, 0, 1, 0],
            },
        );
        let activated = g.state.stack.last().unwrap().id;
        checked(
            &mut g,
            Action::AnnounceActivate {
                card: id,
                ability: 0,
                counter_payment: Counters::default(),
            },
        );
        for bad in [enemy_trigger, spell_id, activated] {
            reject(
                &mut g,
                Action::ChooseTarget {
                    target: Target::Stack(bad),
                },
            );
        }
        reject(
            &mut g,
            Action::ChooseTarget {
                target: Target::Spell(spell),
            },
        );
        checked(
            &mut g,
            Action::ChooseTarget {
                target: Target::Stack(trigger.id),
            },
        );
        for card in [id, bear] {
            let object = g.object_ref(card);
            checked(&mut g, Action::ChooseAdditionalCost { object });
        }
        checked(&mut g, Action::PayCosts { payment: [0; 6] });
        to_retarget(&mut g);
        checked(&mut g, Action::FinishStackTargets);
        settle(&mut g);
    }
}
#[test]
fn ecl_kirol_preserves_cast_event_parameters_after_trigger_source_departs() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let id = paid(&mut g, p, 0);
        let source = named(&mut g, p, "Enraged Flamecaster", Zone::Battlefield);
        let bear = named(&mut g, p, "Bear Cub", Zone::Battlefield);
        let angel = named(&mut g, p, "Serra Angel", Zone::Hand);
        g.state.players[p.index()].mana = [30; 6];
        cast(&mut g, angel, None, &[]);
        let trigger = to_trigger(&mut g, source, None);
        activate(&mut g, id, trigger.id, [id, bear]);
        g.move_object(source, Zone::Graveyard);
        for _ in 0..8 {
            if g.state.stack.last().is_some_and(|s| s.is_copy) {
                break;
            }
            checked(&mut g, Action::Pass);
        }
        let copy = g.state.stack.last().unwrap();
        assert!(copy.is_copy);
        assert_eq!(copy.cast_event, trigger.cast_event);
        assert_eq!(copy.source_facts, trigger.source_facts);
        assert_eq!(copy.object, trigger.object);
        settle(&mut g);
        assert_eq!(g.state.players[p.other().index()].life, 16);
        assert_eq!(g.state.objects[angel as usize].zone, Zone::Battlefield);
    }
}
#[test]
fn ecl_kirol_copies_intrinsic_and_delayed_sacrifice_trigger_packets() {
    for p in [Player::P0, Player::P1] {
        for intrinsic in [false, true] {
            let mut g = setup(p);
            let id = paid(&mut g, p, 2);
            let bear = named(&mut g, p, "Bear Cub", Zone::Battlefield);
            let victim = if intrinsic {
                let recipe = CopyRecipe {
                    definition: Card::from_name("Bear Cub").unwrap(),
                    exceptions: CopyExceptions::SACRIFICE_ONLY,
                };
                g.stage_copy_creation(p, recipe, 1);
                settle(&mut g);
                g.state
                    .copy_recipes
                    .as_ref()
                    .unwrap()
                    .iter()
                    .find(|row| row.recipe == recipe)
                    .unwrap()
                    .object
                    .card
            } else {
                let victim = named(&mut g, p, "Bear Cub", Zone::Battlefield);
                let source = named(&mut g, p, "Meek Attack", Zone::Battlefield);
                let item = g.activation_item(p, source, 0);
                let object = g.object_ref(victim);
                g.record_mobilize_tokens(&item, vec![object]);
                victim
            };
            g.begin_step(Step::End);
            g.prepare_triggers();
            let trigger = g.state.stack.last().unwrap().clone();
            assert_eq!(
                trigger.kind,
                if intrinsic {
                    StackKind::CopyEndStepSacrifice
                } else {
                    StackKind::DelayedSacrifice
                }
            );
            activate(&mut g, id, trigger.id, [id, bear]);
            for _ in 0..8 {
                if g.state.stack.last().is_some_and(|s| s.is_copy) {
                    break;
                }
                checked(&mut g, Action::Pass);
            }
            let copy = g.state.stack.last().unwrap();
            assert!(copy.is_copy);
            assert_eq!(copy.kind, trigger.kind);
            assert_eq!(copy.delayed_sacrifice, trigger.delayed_sacrifice);
            settle(&mut g);
            assert!(!g.on_battlefield(victim));
        }
    }
}
#[test]
fn ecl_kirol_missing_target_cannot_copy_and_countered_activation_keeps_paid_costs() {
    for p in [Player::P0, Player::P1] {
        for counter_activation in [false, true] {
            let mut g = setup(p);
            let id = paid(&mut g, p, 0);
            let bear = named(&mut g, p, "Bear Cub", Zone::Battlefield);
            let trigger = lieutenant(&mut g, p, bear);
            activate(&mut g, id, trigger.id, [id, bear]);
            let activation = g.state.stack.last().unwrap().id;
            if counter_activation {
                assert!(g.counter_stack_item(activation, false));
            } else {
                assert!(g.counter_stack_item(trigger.id, false));
            }
            settle(&mut g);
            assert!(g.object_view(id).tapped && g.object_view(bear).tapped);
            assert_eq!(g.stats(bear).power, if counter_activation { 3 } else { 2 });
            assert!(!g.state.stack.iter().any(|s| s.is_copy));
        }
    }
}
#[test]
fn ecl_kirol_reuse_resets_on_opponent_turn_with_frozen_trigger_controller() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let id = paid(&mut g, p, 2);
        let bear = named(&mut g, p, "Bear Cub", Zone::Battlefield);
        let first = lieutenant(&mut g, p, bear);
        activate(&mut g, id, first.id, [id, bear]);
        to_retarget(&mut g);
        checked(&mut g, Action::FinishStackTargets);
        settle(&mut g);
        assert_ne!(g.state.objects[id as usize].used_abilities, 0);
        g.state.active = p.other();
        g.state.turn += 1;
        g.begin_turn();
        assert_eq!(g.state.objects[id as usize].used_abilities, 0);
        // A real entry event in the opponent's upkeep supplies a controlled trigger.
        let source = named(&mut g, p, "Thoughtweft Lieutenant", Zone::Hand);
        g.move_object(source, Zone::Battlefield);
        let trigger = to_trigger(&mut g, source, Some(Target::Card(bear)));
        g.state.objects[id as usize].tapped = false;
        g.state.objects[bear as usize].tapped = false;
        g.state.priority = p;
        activate(&mut g, id, trigger.id, [id, bear]);
        to_retarget(&mut g);
        assert_eq!(g.retarget_view().unwrap().item.controller, p);
        checked(&mut g, Action::FinishStackTargets);
        settle(&mut g);
        assert_eq!(g.state.active, p.other());
        assert_ne!(g.state.objects[id as usize].used_abilities, 0);
    }
}
