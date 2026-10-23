use super::ability_timeline_tests::{cast, checked, named, settle, setup};
use super::tests::spawn;
use super::*;

const SAVED_DAMAGE: &[Instruction] = &[Instruction::PutCountersScaled {
    subject: Subject::Source,
    kind: CounterKind::PlusOne,
    amount: AmountValue::DamageEventAmount,
}];
const COUNTER_LOSS: &[Instruction] = &[Instruction::LoseLifeScaled(
    PlayerSet::Opponent,
    AmountValue::SourceCounterQuantity(CounterKind::PlusOne),
)];

const SOUL_LOSS: &[Instruction] = &[Instruction::LoseLifeScaled(
    PlayerSet::Opponent,
    AmountValue::SourceCounterQuantity(CounterKind::Soul),
)];
const STASH_LOSS: &[Instruction] = &[Instruction::LoseLifeScaled(
    PlayerSet::Opponent,
    AmountValue::SourceCounterQuantity(CounterKind::Stash),
)];

// Immutable generic programs exercise core composition without changing any
// printed definition or claiming that a gated consumer is supported.
fn resolve(g: &mut Game, item: SpellView, program: &'static [Instruction]) {
    g.state.resolving = Some(Resolution {
        wheel: None,
        continuation: None,
        chosen_creature_type: None,
        chosen_color: None,
        explore: None,
        evidence: None,
        connive: None,
        item,
        program,
        pc: 0,
        target_group: 0,
        moved_count: None,
        discards: [0; 2],
        mana_payment: None,
        forage: None,
        selected_discards: [Vec::new(), Vec::new()],
        milled: Vec::new(),
        milled_selection: None,
        library: None,
        selection: None,
        target_library: None,
        search: None,
        legendary_search: None,
        optional_searches: None,
        choice: None,
    });
    g.run_program();
}
fn selected(g: &mut Game, p: Player, source: CardId, program: &[Instruction]) -> SpellView {
    let mut item = g.activation_item(p, source, 0);
    item.source_counter_lki =
        card_quantities::counter_quantity_facts(program, |kind| g.counter_count(source, kind));
    item
}

#[test]
fn saved_actual_combat_amount_drives_counters_and_does_not_follow_a_new_source() {
    for p in [Player::P0, Player::P1] {
        for departed in [false, true] {
            let mut g = setup(p);
            let source = named(&mut g, p, "Stromkirk Noble", Zone::Hand);
            cast(&mut g, source, None, &[]);
            settle(&mut g);
            // Begin a later combat fixture after the paid creature has resolved.
            g.state.objects[source as usize].summoning_sick = false;
            g.state.objects[source as usize].counters.plus_one = 2;
            g.state.attackers.push(Attacker {
                defender: Some(CombatDefender::Player(p.other())),
                card: source,
                blocked: false,
            });
            g.state.step = Step::Blockers;
            g.give_priority();
            checked(&mut g, Action::Pass);
            checked(&mut g, Action::Pass);
            let item = g.state.stack.pop().unwrap();
            assert_eq!(item.damage_event.unwrap().amount, 3);
            assert!(item.source_counter_lki.is_none());
            if departed {
                g.move_object(source, Zone::Hand);
                g.move_object(source, Zone::Battlefield);
                assert_ne!(g.object_ref(source), item.object);
            } else {
                // The saved dealt amount is independent of power at resolution.
                g.state.objects[source as usize].growth.power = 100;
            }
            let before = g.snapshot();
            let mut branch = {
                let parent = g.clone();
                parent.clone()
            };
            resolve(&mut branch, item.clone(), SAVED_DAMAGE);
            resolve(&mut g, item.clone(), SAVED_DAMAGE);
            assert_eq!(g.snapshot(), branch.snapshot());
            assert_eq!(
                g.state.objects[source as usize].counters.plus_one,
                if departed { 0 } else { 5 }
            );
            let after = g.snapshot();
            g.restore(&before).unwrap();
            resolve(&mut g, item, SAVED_DAMAGE);
            assert_eq!(g.snapshot(), after);
        }
    }
}

#[test]
fn selected_counter_quantity_reads_live_any_zone_then_exact_departure_lki() {
    for p in [Player::P0, Player::P1] {
        for (kind, program) in [
            (CounterKind::PlusOne, COUNTER_LOSS),
            (CounterKind::Stash, STASH_LOSS),
        ] {
            let mut g = setup(p);
            let source = named(&mut g, p, "Fanatical Firebrand", Zone::Exile);
            g.add_counter(source, kind, 3);
            let item = selected(&mut g, p, source, program);
            assert_eq!(item.source_counter_lki.as_ref().unwrap().len(), 1);
            assert_eq!(
                g.item_amount(AmountValue::SourceCounterQuantity(kind), &item),
                3
            );
            g.state.stack.push(item);
            // An ordinary public priority action copies the selected packet through
            // owned raw/index branches and compares both seats' observations.
            checked(&mut g, Action::Pass);
            g.add_counter(source, kind, 4);
            // The packet must refresh at departure, rather than use activation-time 3.
            g.move_object(source, Zone::Hand);
            g.move_object(source, Zone::Exile);
            g.add_counter(source, kind, 99);
            let item = g.state.stack.pop().unwrap();
            assert_eq!(
                g.item_amount(AmountValue::SourceCounterQuantity(kind), &item),
                7
            );
            let life = g.state.players[p.other().index()].life;
            let before = g.snapshot();
            let mut owned = {
                let parent = g.clone();
                parent.clone()
            };
            resolve(&mut owned, item.clone(), program);
            resolve(&mut g, item.clone(), program);
            assert_eq!(g.snapshot(), owned.snapshot());
            assert_eq!(g.state.players[p.other().index()].life, life - 7);
            let after = g.snapshot();
            g.restore(&before).unwrap();
            resolve(&mut g, item, program);
            assert_eq!(g.snapshot(), after);
        }
    }
}

#[test]
fn ordered_shared_counter_removal_then_sacrifice_captures_post_payment_lki() {
    for p in [Player::P0, Player::P1] {
        for (kind, program) in [
            (CounterKind::PlusOne, COUNTER_LOSS),
            (CounterKind::Soul, SOUL_LOSS),
        ] {
            let mut g = setup(p);
            let source = named(&mut g, p, "Fanatical Firebrand", Zone::Battlefield);
            g.add_counter(source, kind, 3);
            g.add_counter(source, CounterKind::Stash, 9);
            let mut item = selected(&mut g, p, source, program);
            // A typed cost-composition fixture calls the exact shared production
            // payment routine. It is deliberately not installed on a printed card.
            let mut definition = g.state.objects[source as usize].card.activations()[0];
            definition.cost.remove = Some((Some(kind), 1));
            definition.program = program;
            let paid = Counters::of(kind, 1);
            let original_item = item.clone();
            let before = g.snapshot();
            let mut branch = {
                let parent = g.clone();
                parent.clone()
            };
            let mut branch_item = item.clone();
            branch.pay_activation_definition(
                &mut branch_item,
                [0; 6],
                paid.clone(),
                0,
                &definition,
            );
            g.pay_activation_definition(&mut item, [0; 6], paid.clone(), 0, &definition);
            assert_eq!(g.snapshot(), branch.snapshot());
            assert_eq!(item, branch_item);
            assert_eq!(g.state.objects[source as usize].zone, Zone::Graveyard);
            assert_eq!(item.source_counter_lki.as_ref().unwrap()[0].count, 2);
            let after = g.snapshot();
            g.restore(&before).unwrap();
            let mut replay = original_item;
            g.pay_activation_definition(&mut replay, [0; 6], paid, 0, &definition);
            assert_eq!(g.state.objects[source as usize].zone, Zone::Graveyard);
            assert_eq!(replay.source_counter_lki, item.source_counter_lki);
            assert_eq!(g.snapshot(), after);
            let life = g.state.players[p.other().index()].life;
            resolve(&mut g, item, program);
            assert_eq!(g.state.players[p.other().index()].life, life - 2);
        }
    }
}

#[test]
fn ordinary_packets_have_no_counter_storage_and_only_referenced_kinds_are_saved() {
    let mut g = setup(Player::P0);
    let source = named(&mut g, Player::P0, "Fanatical Firebrand", Zone::Battlefield);
    let ordinary = g.activation_item(Player::P0, source, 0);
    assert!(ordinary.source_counter_lki.is_none());
    assert!(
        serde_json::to_value(&ordinary)
            .unwrap()
            .get("source_counter_lki")
            .is_none()
    );
    let facts = card_quantities::counter_quantity_facts(
        &[
            Instruction::LoseLifeScaled(
                PlayerSet::Opponent,
                AmountValue::SourceCounterQuantity(CounterKind::PlusOne),
            ),
            Instruction::GainLifeScaled(
                PlayerSet::Controller,
                AmountValue::SourceCounterQuantity(CounterKind::PlusOne),
            ),
        ],
        |_| 12,
    )
    .unwrap();
    assert_eq!(facts.len(), 1);
    assert_eq!(
        facts[0],
        SourceCounterQuantityView {
            kind: CounterKind::PlusOne,
            count: 12
        }
    );
}

#[test]
fn wall_exclusion_uses_effective_changeling_types_and_removed_source_abilities() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let juggernaut = spawn(
            &mut g,
            p,
            Card::from_name("Juggernaut").unwrap(),
            Zone::Battlefield,
        );
        assert!(g.state.objects[juggernaut as usize].card.is_supported());
        let wall = named(&mut g, p.other(), "Chitinous Graspling", Zone::Battlefield);
        let bear = named(&mut g, p.other(), "Bear Cub", Zone::Battlefield);
        assert!(g.has_subtype(wall, "Wall"));
        assert!(!g.can_block_pair(wall, juggernaut));
        assert!(g.can_block_pair(bear, juggernaut));
        g.state.attackers.push(Attacker {
            defender: Some(CombatDefender::Player(p.other())),
            card: juggernaut,
            blocked: false,
        });
        g.state.step = Step::Blockers;
        g.state.pending = Pending::Blockers;
        let illegal = Action::Block {
            blocker: wall,
            attacker: juggernaut,
        };
        let before = g.snapshot();
        assert!(!g.legal_actions().contains(&illegal));
        assert!(g.apply(p.other(), illegal).is_err());
        assert_eq!(g.snapshot(), before);
        // Stop Cold is an actual paid response after attackers are declared.
        g.give_priority();
        let stop = named(&mut g, p, "Stop Cold", Zone::Hand);
        let target = Target::Card(juggernaut);
        cast(&mut g, stop, None, &[target]);
        settle(&mut g);
        assert!(g.can_block_pair(wall, juggernaut));
        g.state.pending = Pending::Blockers;
        checked(
            &mut g,
            Action::Block {
                blocker: wall,
                attacker: juggernaut,
            },
        );
        checked(&mut g, Action::FinishBlockers);
        assert!(g.state.blocks.contains(&(wall, juggernaut)));
    }
}

#[test]
fn wall_exclusion_observes_replaced_creature_types_until_the_exact_blocker_leaves() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let juggernaut = spawn(
            &mut g,
            p,
            Card::from_name("Juggernaut").unwrap(),
            Zone::Battlefield,
        );
        let blocker = named(&mut g, p.other(), "Chitinous Graspling", Zone::Battlefield);
        assert!(g.has_subtype(blocker, "Wall"));
        assert!(!g.can_block_pair(blocker, juggernaut));
        let source = spawn(&mut g, p, Card::Shock, Zone::Hand);
        let mut item = g.cast_item(p, source, [0; 6]);
        item.targets.push(BoundTarget {
            group: 0,
            target: Target::Card(blocker),
            object: Some(g.object_ref(blocker)),
        });
        resolve(
            &mut g,
            item,
            &[Instruction::SetCreatureTypes {
                subject: Subject::Targets,
                types: &["Werewolf"],
            }],
        );
        let frozen = g.object_view(blocker);
        assert!(frozen.has_subtype("Werewolf"));
        assert!(!frozen.has_subtype("Wall"));
        assert!(g.can_block_pair(blocker, juggernaut));
        g.state.attackers.push(Attacker {
            defender: Some(CombatDefender::Player(p.other())),
            card: juggernaut,
            blocked: false,
        });
        g.state.step = Step::Blockers;
        g.state.pending = Pending::Blockers;
        checked(
            &mut g,
            Action::Block {
                blocker,
                attacker: juggernaut,
            },
        );
        checked(&mut g, Action::FinishBlockers);
        g.move_object(blocker, Zone::Hand);
        g.move_object(blocker, Zone::Battlefield);
        assert_ne!(g.object_ref(blocker), frozen.reference);
        assert!(g.has_subtype(blocker, "Wall"));
        assert!(!g.can_block_pair(blocker, juggernaut));
        assert!(!frozen.has_subtype("Wall"));
    }
}
