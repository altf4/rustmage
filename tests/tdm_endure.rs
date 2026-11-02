//! CR701.63: the enduring permanent's controller chooses; prohibited counters
//! force a Spirit. These scenarios use actual Anafenza death triggers.
mod support;
use rustmage as _;
use rustmage_core::cards::authored::{CardExtras, CardRules};
use rustmage_core::cards::*;
use rustmage_core::engine::*;
use support::*;

const STEAL: CardRules = CardRules {
    effect: Effect::Spell(
        TargetPlan::Single(TargetSpec::one(TargetRule::Creature)),
        &[Instruction::GainControl {
            subject: Subject::Targets,
            controller: PlayerSet::Controller,
        }],
    ),
    ..CardRules::EMPTY
};
const NO_COUNTERS: &[StaticAbilityGroup] = &[StaticAbilityGroup {
    line: 0,
    first_layer: AbilityLayer::Rule,
    fragments: &[StaticAbilityFragment::CounterProhibition(1)],
}];
const BARRIER: CardRules = CardRules {
    effect: Effect::Permanent,
    static_groups: NO_COUNTERS,
    extras: &CardExtras {
        static_ability_groups: NO_COUNTERS,
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};

fn wait(g: &mut Game, ready: impl Fn(&Game) -> bool) {
    for _ in 0..32 {
        if ready(g) {
            return;
        }
        assert_eq!(g.state.pending, Pending::Priority);
        act(g, Action::Pass);
    }
    panic!("32-action Endure setup budget exhausted");
}

fn trigger_root(p: Player) -> (Game, CardId) {
    let mut g = scenario(p);
    let anafenza = put(&mut g, p, "Anafenza, Unyielding Lineage", Zone::Battlefield);
    let victim = put(&mut g, p, "Bear Cub", Zone::Battlefield);
    let shock = put(&mut g, p, "Shock", Zone::Hand);
    cast(&mut g, shock, Some(Target::Card(victim)));
    wait(&mut g, |g| {
        g.object_view(victim).zone == Zone::Graveyard
            && g.state.stack.len() == 1
            && matches!(g.state.stack[0].kind, StackKind::TriggeredAbility { .. })
    });
    (g, anafenza)
}

fn steal_source(g: &mut Game, source: CardId, recipient: Player) {
    if g.actor() != Some(recipient) {
        act(g, Action::Pass);
    }
    let steal = put(g, recipient, "Opt", Zone::Hand);
    cast(g, steal, Some(Target::Card(source)));
    wait(g, |g| g.object_view(source).controller == recipient);
    assert_eq!(g.state.stack.len(), 1);
}

fn spirits(g: &Game, p: Player) -> Vec<CardId> {
    g.state
        .objects
        .iter()
        .enumerate()
        .filter_map(|(id, object)| {
            (object.zone == Zone::Battlefield
                && object.controller == p
                && object.is_token()
                && object.card.has_subtype("Spirit"))
            .then_some(id as CardId)
        })
        .collect()
}

#[test]
fn prohibited_endure_counters_create_the_spirit_without_an_impossible_choice() {
    let barrier = Card::from_name("Short Bow").unwrap();
    let _rules = fixture_rules::activate(&[(barrier, &BARRIER)]);
    for p in [Player::P0, Player::P1] {
        let (mut g, source) = trigger_root(p);
        put(&mut g, p.other(), "Short Bow", Zone::Battlefield);
        settle(&mut g);
        assert_eq!(g.counter_count(source, CounterKind::PlusOne), 0);
        let tokens = spirits(&g, p);
        assert_eq!(tokens.len(), 1);
        assert_eq!(stats(&g, tokens[0]), (2, 2));
        assert_eq!(g.object_view(tokens[0]).colors, 1);
        assert_eq!(g.object_view(tokens[0]).keywords, Keywords::NONE);
        assert!(g.observe(p).endure_choice.is_none());
        reject(&mut g, Action::ChooseEndure { put_counters: true });
    }
}

#[test]
fn endure_uses_the_live_permanent_controller_for_both_outcomes() {
    let _rules = fixture_rules::activate(&[(Card::from_name("Opt").unwrap(), &STEAL)]);
    for original in [Player::P0, Player::P1] {
        for counters in [false, true] {
            let (mut g, source) = trigger_root(original);
            let current = original.other();
            steal_source(&mut g, source, current);
            wait(&mut g, |g| {
                matches!(g.state.pending, Pending::Endure { .. })
            });
            assert_eq!(g.actor(), Some(current));
            assert!(g.observe(current).endure_choice.is_some());
            assert!(g.observe(original).endure_choice.is_none());
            act(
                &mut g,
                Action::ChooseEndure {
                    put_counters: counters,
                },
            );
            settle(&mut g);
            assert_eq!(
                g.counter_count(source, CounterKind::PlusOne),
                if counters { 2 } else { 0 }
            );
            assert_eq!(spirits(&g, current).len(), usize::from(!counters));
            assert!(spirits(&g, original).is_empty());
        }
    }
}

#[test]
fn forced_endure_spirit_uses_its_current_controllers_token_replacements() {
    let _rules = fixture_rules::activate(&[
        (Card::from_name("Opt").unwrap(), &STEAL),
        (Card::from_name("Short Bow").unwrap(), &BARRIER),
    ]);
    for original in [Player::P0, Player::P1] {
        for current_controls_doubler in [false, true] {
            let (mut g, source) = trigger_root(original);
            let current = original.other();
            steal_source(&mut g, source, current);
            put(&mut g, original, "Short Bow", Zone::Battlefield);
            let doubler = if current_controls_doubler {
                current
            } else {
                original
            };
            let elspeth = put(&mut g, doubler, "Elspeth, Storm Slayer", Zone::Battlefield);
            // Direct fixture entry does not apply starting loyalty counters.
            g.add_counter(elspeth, CounterKind::Loyalty, 5);
            settle(&mut g);
            assert_eq!(g.counter_count(source, CounterKind::PlusOne), 0);
            let tokens = spirits(&g, current);
            assert_eq!(tokens.len(), if current_controls_doubler { 2 } else { 1 });
            assert!(spirits(&g, original).is_empty());
            for token in tokens {
                assert_eq!(stats(&g, token), (2, 2));
                assert_eq!(g.object_view(token).owner, current);
            }
        }
    }
}

#[test]
fn departed_enduring_source_keeps_its_last_controller_and_never_rebinds() {
    let _rules = fixture_rules::activate(&[(Card::from_name("Opt").unwrap(), &STEAL)]);
    for original in [Player::P0, Player::P1] {
        let (mut g, source) = trigger_root(original);
        let current = original.other();
        steal_source(&mut g, source, current);
        if g.actor() != Some(current) {
            act(&mut g, Action::Pass);
        }
        let bounce = put(&mut g, current, "Unsummon", Zone::Hand);
        cast(&mut g, bounce, Some(Target::Card(source)));
        wait(&mut g, |g| g.object_view(source).zone == Zone::Hand);
        let old = g.state.stack[0].source.unwrap().object;
        if g.actor() != Some(original) {
            act(&mut g, Action::Pass);
        }
        cast(&mut g, source, None);
        wait(&mut g, |g| g.object_view(source).zone == Zone::Battlefield);
        assert_ne!(g.object_ref(source), old);
        assert_eq!(g.object_view(source).controller, original);
        settle(&mut g);
        assert_eq!(g.counter_count(source, CounterKind::PlusOne), 0);
        assert_eq!(spirits(&g, current).len(), 1);
        assert!(spirits(&g, original).is_empty());
    }
}

#[test]
fn changed_endure_actor_survives_owned_raw_index_and_restore_branches() {
    let _rules = fixture_rules::activate(&[(Card::from_name("Opt").unwrap(), &STEAL)]);
    let original = Player::P0;
    let current = original.other();
    let (mut parent, source) = trigger_root(original);
    steal_source(&mut parent, source, current);
    wait(&mut parent, |g| {
        matches!(g.state.pending, Pending::Endure { .. })
    });
    let saved = parent.snapshot();
    let rev = parent.revision();
    let token_action = Action::ChooseEndure {
        put_counters: false,
    };
    let index = parent
        .legal_actions()
        .iter()
        .position(|a| *a == token_action)
        .unwrap();
    let mut raw = parent.clone();
    let mut indexed = parent.clone();
    let mut sibling = parent.clone();
    drop(parent);
    assert_eq!(
        raw.apply(original, token_action.clone()),
        Err(Error::WrongActor)
    );
    assert_eq!(raw.snapshot(), saved);
    raw.apply(current, token_action.clone()).unwrap();
    indexed.apply_index(current, rev, index).unwrap();
    settle(&mut raw);
    settle(&mut indexed);
    assert_eq!(raw.snapshot(), indexed.snapshot());
    assert_eq!(spirits(&raw, current).len(), 1);
    act(&mut sibling, Action::ChooseEndure { put_counters: true });
    settle(&mut sibling);
    assert_eq!(sibling.counter_count(source, CounterKind::PlusOne), 2);
    assert!(spirits(&sibling, current).is_empty());
    indexed.restore(&saved).unwrap();
    assert_eq!(
        indexed.apply_index(current, rev, index),
        Err(Error::StaleRevision)
    );
    act(&mut indexed, token_action);
    settle(&mut indexed);
    assert_eq!(raw.snapshot(), indexed.snapshot());
}
