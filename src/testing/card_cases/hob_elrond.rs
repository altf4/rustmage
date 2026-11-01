//! Focused printed outcomes; shared engine invariants live in core tests.
//! Actual activated-creature events and exact delayed blink continuations.
use super::*;
use crate::cards::{AnimationCreatureTypes, AnimationDuration, AnimationSpec};
fn hand(g: &Game, p: Player) -> usize {
    g.state
        .objects
        .iter()
        .filter(|o| o.zone == Zone::Hand && o.owner == p)
        .count()
}
const FIXTURES: &[&str] = &[
    "Elrond, Moon-Reader",
    "Forest",
    "Bear Cub",
    "Llanowar Elves",
    "Goldvein Pick",
    "Witness Protection",
    "Disenchant",
    "Banishing Light",
    "Adaptive Automaton",
    "Authority of the Consuls",
    "Territory Forge",
    "Prophetic Prism",
    "Rubble Rouser",
    "Candy Grapple",
    "Silverquill, the Disputant",
    "Chitin Gravestalker",
    "Troop of Ponies",
    "@token/hob-dwarf",
    "Lotus Ring",
    "Fíli the Pathfinder",
    "Axgard Cavalry",
];
fn fixture(g: &mut Game, p: Player, name: &str, zone: Zone) -> CardId {
    assert!(FIXTURES.contains(&name), "unlisted {name}");
    put(g, p, name, zone)
}
fn quiet(g: &Game) -> bool {
    g.state.pending == Pending::Priority
        && g.state.stack.is_empty()
        && g.state.pending_triggers.is_empty()
        && g.state.ordering_triggers.is_empty()
        && g.state.resolving.is_none()
}
fn until(g: &mut Game, done: impl Fn(&Game) -> bool) {
    for _ in 0..96 {
        if done(g) {
            return;
        }
        choose(g, |a| matches!(a, Action::Pass | Action::PutTrigger { .. }));
    }
    panic!("Elrond budget {:?}", g.state.pending)
}
pub(super) fn finish(g: &mut Game) {
    until(g, quiet)
}
fn paid(g: &mut Game, p: Player, name: &str, mana: Mana, target: Option<Target>) -> CardId {
    let id = fixture(g, p, name, Zone::Hand);
    g.state.players[p.index()].mana = mana;
    cast(g, id, target);
    assert_eq!(g.state.players[p.index()].mana, [0; 6]);
    id
}
fn elrond(g: &mut Game, p: Player) -> CardId {
    let id = paid(g, p, "Elrond, Moon-Reader", [0, 1, 0, 0, 0, 2], None);
    finish(g);
    assert_eq!(stats(g, id), (3, 3));
    assert!(g.has_subtype(id, "Elf"));
    assert!(g.has_subtype(id, "Noble"));
    id
}
fn activate(g: &mut Game, id: CardId, targets: &[CardId]) {
    let p = g.state.objects[id as usize].controller;
    g.state.players[p.index()].mana = [0, 2, 0, 0, 0, 5];
    act(
        g,
        Action::AnnounceActivate {
            card: id,
            ability: 0,
            counter_payment: Counters::default(),
        },
    );
    for &id in targets {
        act(
            g,
            Action::ChooseTarget {
                target: Target::Card(id),
            },
        );
    }
    if g.legal_actions().contains(&Action::FinishTargets) {
        act(g, Action::FinishTargets);
    }
    pay(g);
    assert_eq!(g.state.players[p.index()].mana, [0; 6]);
}
pub(super) fn mana(g: &mut Game, id: CardId) {
    choose(
        g,
        |a| matches!(a,Action::ActivateMana{card,..}|Action::TapMana{card}|Action::TapGrantedMana{card,..} if *card==id),
    );
}
fn end(g: &mut Game) {
    g.begin_step(Step::End);
    g.give_priority();
    g.prepare_triggers();
}
#[test]
fn elrond_paid_own_zero_target_activation_draws_once_even_if_countered_or_source_gone() {
    {
        let p = Player::P0;
        let mut g = scenario(p);
        let id = elrond(&mut g, p);
        let before = hand(&g, p);
        activate(&mut g, id, &[]);
        assert_eq!(g.state.triggered_this_turn.len(), 1);
        assert_eq!(g.state.stack.len(), 2);
        let ability = g
            .state
            .stack
            .iter()
            .position(|x| matches!(x.kind, StackKind::ActivatedAbility { .. }))
            .unwrap();
        assert!(g.counter_stack_item(g.state.stack[ability].id, false));
        g.move_object(id, Zone::Graveyard);
        finish(&mut g);
        assert_eq!(hand(&g, p), before + 1);
    }
}
#[test]
fn elrond_direct_mana_once_suppression_control_and_new_incarnation() {
    {
        let p = Player::P0;
        let mut g = scenario(p);
        let id = elrond(&mut g, p);
        let a = fixture(&mut g, p, "Llanowar Elves", Zone::Battlefield);
        let b = fixture(&mut g, p, "Llanowar Elves", Zone::Battlefield);
        let before = hand(&g, p);
        mana(&mut g, a);
        finish(&mut g);
        assert_eq!(hand(&g, p), before + 1);
        mana(&mut g, b);
        finish(&mut g);
        assert_eq!(hand(&g, p), before + 1);
        let aura = paid(
            &mut g,
            p,
            "Witness Protection",
            [0, 1, 0, 0, 0, 0],
            Some(Target::Card(id)),
        );
        finish(&mut g);
        g.state.objects[a as usize].tapped = false;
        mana(&mut g, a);
        finish(&mut g);
        g.move_object(aura, Zone::Graveyard);
        g.state.objects[a as usize].tapped = false;
        mana(&mut g, a);
        finish(&mut g);
        assert_eq!(hand(&g, p), before + 1); // the fixture adds and then casts its Aura
        let q = p.other();
        g.record_control(g.object_ref(id), q, None);
        g.refresh_controllers();
        g.state.priority = q;
        let c = fixture(&mut g, q, "Llanowar Elves", Zone::Battlefield);
        let theirs = hand(&g, q);
        mana(&mut g, c);
        finish(&mut g);
        assert_eq!(hand(&g, q), theirs);
        g.move_object(id, Zone::Exile);
        g.move_object(id, Zone::Battlefield);
        g.state.priority = p;
        g.state.objects[a as usize].tapped = false;
        mana(&mut g, a);
        finish(&mut g);
        assert_eq!(hand(&g, p), before + 2);
        g.begin_turn();
        g.state.active = p;
        g.state.priority = p;
        g.state.pending = Pending::Priority;
        g.state.step = Step::Main1;
        g.state.objects[a as usize].tapped = false;
        g.state.objects[a as usize].summoning_sick = false;
        let n = hand(&g, p);
        mana(&mut g, a);
        finish(&mut g);
        assert_eq!(hand(&g, p), n + 1);
    }
}

#[test]
fn elrond_borrowed_ability_uses_actual_source_current_creature() {
    {
        let p = Player::P0;
        for borrowed_name in ["Prophetic Prism", "Llanowar Elves"] {
            for creature in [false, true] {
                let mut g = scenario(p);
                elrond(&mut g, p);
                let blueprint = fixture(&mut g, p.other(), borrowed_name, Zone::Battlefield);
                if borrowed_name == "Llanowar Elves" {
                    // Forge can exile artifacts/lands. The temporary artifact type
                    // makes this creature blueprint a legal actual paid ETB target.
                    g.change_object_form(
                        g.object_ref(blueprint),
                        AnimationSpec {
                            artifact: true,
                            base: None,
                            creature_types: AnimationCreatureTypes::Unchanged,
                            keywords: Keywords::NONE,
                            duration: AnimationDuration::EndOfTurn,
                        },
                        characteristic_layers::AnimationTypeMode::PreserveCardTypes,
                    );
                }
                g.state.players[p.index()].mana = [8; 6];
                let forge = crate::engine::testing::card_cases::territory_forge_tests::forge(
                    &mut g, p, blueprint,
                );
                let acquired = g
                    .borrowed_ability_views()
                    .into_iter()
                    .find(|a| a.source.card == forge)
                    .unwrap();
                if creature {
                    g.change_object_form(
                        g.object_ref(forge),
                        AnimationSpec {
                            artifact: false,
                            base: Some((3, 3)),
                            creature_types: AnimationCreatureTypes::Unchanged,
                            keywords: Keywords::NONE,
                            duration: AnimationDuration::EndOfTurn,
                        },
                        characteristic_layers::AnimationTypeMode::AddCreature,
                    );
                }
                g.state.objects[forge as usize].summoning_sick = false;
                let before = hand(&g, p);
                g.state.players[p.index()].mana = [0, 0, 0, 0, 0, 1];
                let production = if borrowed_name == "Prophetic Prism" {
                    [0, 1, 0, 0, 0, 0]
                } else {
                    [0, 0, 0, 0, 1, 0]
                };
                act(
                    &mut g,
                    Action::AnnounceBorrowedMana {
                        card: forge,
                        exiled: acquired.exiled,
                        ability: acquired.ability,
                        counter_payment: Counters::default(),
                        production,
                    },
                );
                if g.state.announcing.is_some() {
                    pay(&mut g);
                }
                finish(&mut g);
                assert_eq!(hand(&g, p), before + usize::from(creature));
            }
        }
    }
}
#[test]
fn elrond_return_replacement_and_exact_exile_generation() {
    {
        let p = Player::P0;
        let mut g = scenario(p);
        let source = elrond(&mut g, p);
        fixture(
            &mut g,
            p.other(),
            "Authority of the Consuls",
            Zone::Battlefield,
        );
        let a = fixture(&mut g, p, "Bear Cub", Zone::Battlefield);
        let b = fixture(&mut g, p, "Bear Cub", Zone::Battlefield);
        activate(&mut g, source, &[a, b]);
        finish(&mut g);
        g.move_object(a, Zone::Hand);
        g.move_object(a, Zone::Exile);
        end(&mut g);
        finish(&mut g);
        assert_eq!(g.object_view(a).zone, Zone::Exile);
        assert_eq!(g.object_view(b).zone, Zone::Battlefield);
        assert!(g.object_view(b).tapped);
    }
}

#[test]
fn elrond_granted_mana_sacrificed_creature_qualifies_not_artifact_grantor() {
    {
        let p = Player::P0;
        let mut g = scenario(p);
        elrond(&mut g, p);
        let bear = fixture(&mut g, p, "Bear Cub", Zone::Battlefield);
        let ring = fixture(&mut g, p, "Lotus Ring", Zone::Battlefield);
        g.state.players[p.index()].mana = [0, 0, 0, 0, 0, 3];
        choose(
            &mut g,
            |a| matches!(a,Action::Activate{card,target:Some(Target::Card(t)),..}if *card==ring&&*t==bear),
        );
        finish(&mut g);
        assert!(g.state.triggered_this_turn.is_empty());
        let before = hand(&g, p);
        let grantor = g.object_ref(ring);
        act(
            &mut g,
            Action::TapGrantedMana {
                card: bear,
                grantor,
                production: [0, 3, 0, 0, 0, 0],
            },
        );
        assert_eq!(g.object_view(bear).zone, Zone::Graveyard);
        finish(&mut g);
        assert_eq!(hand(&g, p), before + 1);
        assert_eq!(g.state.players[p.index()].mana, [0, 3, 0, 0, 0, 0]);
    }
}

#[test]
fn elrond_returned_observer_sees_other_simultaneous_returned_dwarf() {
    {
        let p = Player::P0;
        let mut g = scenario(p);
        let source = elrond(&mut g, p);
        // The other Dwarf is first: a serial return would enter it BEFORE Fíli
        // exists, losing that trigger. One batch must collect both Fíli entry events.
        let dwarf = fixture(&mut g, p, "Axgard Cavalry", Zone::Battlefield);
        let fili = fixture(&mut g, p, "Fíli the Pathfinder", Zone::Battlefield);
        activate(&mut g, source, &[dwarf, fili]);
        finish(&mut g);
        end(&mut g);
        assert_eq!(g.state.stack.len(), 1);
        assert_eq!(g.state.stack[0].kind, StackKind::DelayedReturn);
        finish(&mut g);
        let tokens = g
            .state
            .objects
            .iter()
            .filter(|o| {
                o.zone == Zone::Battlefield
                    && o.controller == p
                    && o.is_token()
                    && o.card == Card::from_name("@token/hob-dwarf").unwrap()
            })
            .count();
        assert_eq!(
            tokens, 2,
            "Fíli sees itself AND the simultaneously returned other nontoken Dwarf"
        );
    }
}
