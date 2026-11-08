//! Wither damage remains a damage event while counter replacements suspend it.
mod support;
use rustmage as _;
use rustmage_core::cards::authored::{CardExtras, CardRules};
use rustmage_core::{cards::*, engine::*};
use support::*;

const BODY: CardRules = CardRules {
    effect: Effect::Creature(3, 3),
    keywords: Keywords::WITHER.union(Keywords::LIFELINK),
    ..CardRules::EMPTY
};
const TANK: CardRules = CardRules {
    effect: Effect::Creature(10, 10),
    ..CardRules::EMPTY
};
const CREATURE: TargetRule = TargetRule::Object {
    zone: TargetZone::Battlefield,
    kinds: 1,
    relation: -1,
    other: false,
    nonland: false,
    tapped: false,
    min_mana: 0,
    max_mana: u32::MAX,
};
const DEATH: CardRules = CardRules {
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::SelfDies,
        targets: TargetPlan::Single(TargetSpec::one(CREATURE)),
        program: &[Instruction::Damage(1)],
    }],
    ..BODY
};
const KILL: CardRules = CardRules {
    effect: Effect::Spell(
        TargetPlan::Single(TargetSpec::one(CREATURE)),
        &[Instruction::Destroy],
    ),
    ..CardRules::EMPTY
};
const BARRIER: CardRules = CardRules {
    effect: Effect::Permanent,
    static_groups: &[StaticAbilityGroup {
        line: 0,
        first_layer: AbilityLayer::Rule,
        fragments: &[StaticAbilityFragment::CounterProhibition(1)],
    }],
    extras: &CardExtras::EMPTY,
    ..CardRules::EMPTY
};
fn card(name: &str) -> Card {
    Card::from_name(name).unwrap()
}
fn ready(g: &mut Game, condition: impl Fn(&Game) -> bool) {
    for _ in 0..64 {
        if condition(g) {
            return;
        }
        choose(g, |a| {
            matches!(
                a,
                Action::Pass | Action::PutTrigger { .. } | Action::ChooseReplacement { .. }
            )
        });
    }
    panic!("bounded wither continuation");
}
fn drain(g: &mut Game) {
    ready(g, |g| {
        g.state.pending == Pending::Priority
            && g.state.stack.is_empty()
            && g.state.resolving.is_none()
            && g.state.pending_triggers.is_empty()
    });
}
fn bite(g: &mut Game, p: Player, source: CardId, target: CardId) {
    let spell = put(g, p, "Bite Down", Zone::Hand);
    announce(g, spell, &[Target::Card(source), Target::Card(target)]);
}
#[test]
fn wither_noncombat_counter_replacements_do_not_multiply_lifelink() {
    let _scope =
        fixture_rules::activate(&[(card("Bear Cub"), &BODY), (card("Gigantosaurus"), &TANK)]);
    for p in [Player::P0, Player::P1] {
        for doubled in [false, true] {
            let mut g = scenario(p);
            let source = put(&mut g, p, "Bear Cub", Zone::Battlefield);
            g.state.objects[source as usize].owner = p.other();
            let foe = put(&mut g, p.other(), "Gigantosaurus", Zone::Battlefield);
            if doubled {
                put(&mut g, p.other(), "Doubling Season", Zone::Battlefield);
            }
            bite(&mut g, p, source, foe);
            drain(&mut g);
            assert_eq!(
                g.counter_count(foe, CounterKind::MinusOne),
                if doubled { 6 } else { 3 }
            );
            assert_eq!(g.object_view(foe).damage, 0);
            assert!(g.state.objects[foe as usize].damaged_this_turn);
            assert_eq!(g.state.players[p.index()].life, 23);
            assert_eq!(g.state.players[p.other().index()].life, 20);
        }
    }
}
#[test]
fn wither_combat_counters_are_not_effects_and_retaliation_is_simultaneous() {
    let _scope =
        fixture_rules::activate(&[(card("Bear Cub"), &BODY), (card("Gigantosaurus"), &TANK)]);
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        let source = put(&mut g, p, "Bear Cub", Zone::Battlefield);
        let foe = put(&mut g, p.other(), "Gigantosaurus", Zone::Battlefield);
        put(&mut g, p.other(), "Doubling Season", Zone::Battlefield);
        g.state.attackers.push(Attacker {
            card: source,
            defender: Some(CombatDefender::Player(p.other())),
            blocked: true,
        });
        g.state.blocks.push((foe, source));
        g.state.blocking.push(foe);
        g.state.step = Step::Blockers;
        g.give_priority();
        act(&mut g, Action::Pass);
        act(&mut g, Action::Pass);
        assert_eq!(g.counter_count(foe, CounterKind::MinusOne), 3);
        assert_eq!(g.object_view(foe).damage, 0);
        assert_eq!(g.object_view(source).zone, Zone::Graveyard);
        assert_eq!(g.state.players[p.index()].life, 23);
    }
}
#[test]
fn wither_prevention_and_counter_prohibition_have_different_damage_results() {
    let _scope = fixture_rules::activate(&[
        (card("Bear Cub"), &BODY),
        (card("Gigantosaurus"), &TANK),
        (card("Short Bow"), &BARRIER),
    ]);
    for shield in [false, true] {
        let p = Player::P0;
        let mut g = scenario(p);
        let source = put(&mut g, p, "Bear Cub", Zone::Battlefield);
        let foe = put(&mut g, p.other(), "Gigantosaurus", Zone::Battlefield);
        if shield {
            g.add_counter(foe, CounterKind::Shield, 1);
        } else {
            put(&mut g, p.other(), "Short Bow", Zone::Battlefield);
        }
        bite(&mut g, p, source, foe);
        drain(&mut g);
        assert_eq!(g.counter_count(foe, CounterKind::MinusOne), 0);
        assert_eq!(g.object_view(foe).damage, 0);
        assert_eq!(g.state.objects[foe as usize].damaged_this_turn, !shield);
        assert_eq!(
            g.state.players[p.index()].life,
            if shield { 20 } else { 23 }
        );
    }
}
#[test]
fn wither_zero_toughness_and_deathtouch_kill_without_marked_damage() {
    const TOUCH: CardRules = CardRules {
        keywords: BODY.keywords.union(Keywords::DEATHTOUCH),
        ..BODY
    };
    const IMMORTAL: CardRules = CardRules {
        effect: Effect::Creature(3, 3),
        keywords: Keywords::INDESTRUCTIBLE,
        ..TANK
    };
    for (source_rules, target_rules) in [(&BODY, &IMMORTAL), (&TOUCH, &TANK)] {
        let _scope = fixture_rules::activate(&[
            (card("Bear Cub"), source_rules),
            (card("Gigantosaurus"), target_rules),
        ]);
        let mut g = scenario(Player::P0);
        let source = put(&mut g, Player::P0, "Bear Cub", Zone::Battlefield);
        let foe = put(&mut g, Player::P1, "Gigantosaurus", Zone::Battlefield);
        bite(&mut g, Player::P0, source, foe);
        drain(&mut g);
        assert_eq!(g.object_view(foe).zone, Zone::Graveyard);
        assert_eq!(g.state.players[0].life, 23);
    }
}
#[test]
fn wither_counter_replacement_completion_is_owned_across_raw_index_restore() {
    let _scope =
        fixture_rules::activate(&[(card("Bear Cub"), &BODY), (card("Gigantosaurus"), &TANK)]);
    let mut parent = scenario(Player::P0);
    let source = put(&mut parent, Player::P0, "Bear Cub", Zone::Battlefield);
    let foe = put(&mut parent, Player::P1, "Gigantosaurus", Zone::Battlefield);
    parent.state.objects[foe as usize].growth.toughness = 20;
    for _ in 0..2 {
        put(
            &mut parent,
            Player::P1,
            "Doubling Season",
            Zone::Battlefield,
        );
    }
    bite(&mut parent, Player::P0, source, foe);
    ready(&mut parent, |g| {
        matches!(g.state.pending, Pending::Replacement { .. })
    });
    assert_eq!(parent.counter_count(foe, CounterKind::MinusOne), 0);
    assert_eq!(parent.state.players[0].life, 20);
    let action = parent
        .legal_actions()
        .into_iter()
        .find(|a| matches!(a, Action::ChooseReplacement { .. }))
        .unwrap();
    let index = parent
        .legal_actions()
        .iter()
        .position(|a| a == &action)
        .unwrap();
    let revision = parent.revision();
    let saved = parent.snapshot();
    let mut raw = parent.clone();
    let mut indexed = parent.clone();
    let mut sibling = parent.clone();
    drop(parent);
    assert_eq!(
        raw.apply(Player::P0, action.clone()),
        Err(Error::WrongActor)
    );
    assert_eq!(raw.snapshot(), saved);
    raw.apply(Player::P1, action.clone()).unwrap();
    indexed.apply_index(Player::P1, revision, index).unwrap();
    drain(&mut raw);
    drain(&mut indexed);
    assert_eq!(raw.snapshot(), indexed.snapshot());
    assert_eq!(sibling.counter_count(foe, CounterKind::MinusOne), 0);
    assert_eq!(raw.counter_count(foe, CounterKind::MinusOne), 12);
    assert_eq!(raw.state.players[0].life, 23);
    sibling.restore(&saved).unwrap();
    assert_eq!(
        sibling.apply_index(Player::P1, revision, index),
        Err(Error::StaleRevision)
    );
    act(&mut sibling, action);
    drain(&mut sibling);
    assert_eq!(raw.snapshot(), sibling.snapshot());
}
#[test]
fn wither_uses_departed_sources_last_known_keywords() {
    let _scope = fixture_rules::activate(&[
        (card("Bear Cub"), &DEATH),
        (card("Gigantosaurus"), &TANK),
        (card("Shock"), &KILL),
    ]);
    let mut g = scenario(Player::P0);
    let source = put(&mut g, Player::P0, "Bear Cub", Zone::Battlefield);
    let foe = put(&mut g, Player::P1, "Gigantosaurus", Zone::Battlefield);
    let kill = put(&mut g, Player::P0, "Shock", Zone::Hand);
    cast(&mut g, kill, Some(Target::Card(source)));
    ready(&mut g, |g| {
        matches!(g.state.pending, Pending::Targets { .. })
    });
    act(
        &mut g,
        Action::ChooseTarget {
            target: Target::Card(foe),
        },
    );
    drain(&mut g);
    assert_eq!(g.counter_count(foe, CounterKind::MinusOne), 1);
    assert_eq!(g.object_view(foe).damage, 0);
    assert_eq!(g.state.players[0].life, 21);
}
