//! Pre-admission proofs for static creature grants used by persist and ward.
mod support;
use rustmage as _;
use rustmage_core::cards::authored::CardRules;
use rustmage_core::cards::*;
use rustmage_core::engine::*;
use support::*;

const PERSIST: CardRules = CardRules {
    effect: Effect::Creature(5, 5),
    static_groups: &[StaticAbilityGroup {
        line: 0,
        first_layer: AbilityLayer::Ability,
        fragments: &[StaticAbilityFragment::ControlledCreatureTrigger {
            ability: 0,
            other: true,
            nontoken: true,
        }],
    }],
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::Granted(&TriggerCondition::ReturnDeath(
            ReturnDeathFilter::SourceWithoutCounter(CounterKind::MinusOne),
        )),
        targets: TargetPlan::NONE,
        program: &[Instruction::ReturnDeathCard(ReturnKind::Persist)],
    }],
    ..CardRules::EMPTY
};
const WARD: CardRules = CardRules {
    effect: Effect::Creature(2, 2),
    static_groups: &[StaticAbilityGroup {
        line: 1,
        first_layer: AbilityLayer::Ability,
        fragments: &[StaticAbilityFragment::ControlledCreatureTrigger {
            ability: 1,
            other: true,
            nontoken: false,
        }],
    }],
    triggers: &[
        TriggeredAbility {
            condition: TriggerCondition::Ward,
            targets: TargetPlan::NONE,
            program: &[Instruction::CounterRelatedUnlessPayLife(2)],
        },
        TriggeredAbility {
            condition: TriggerCondition::Granted(&TriggerCondition::Ward),
            targets: TargetPlan::NONE,
            program: &[Instruction::CounterRelatedUnlessPayLife(2)],
        },
    ],
    ..CardRules::EMPTY
};
fn fixture_card() -> Card {
    Card::from_name("Grizzly Bears").unwrap()
}
fn murder(g: &mut Game, caster: Player, target: CardId) {
    let spell = put(g, caster, "Murder", Zone::Hand);
    cast(g, spell, Some(Target::Card(target)));
    settle(g);
}
fn spirit(g: &mut Game, controller: Player) -> CardId {
    let token = Card::from_name("@token/spirit_1_1_white_flying").unwrap();
    g.create_tokens(controller, token, 1, false);
    assert!(!g.advance_replacements(false));
    g.advance_entries();
    g.state
        .objects
        .iter()
        .enumerate()
        .find_map(|(id, object)| {
            (object.zone == Zone::Battlefield
                && object.card == token
                && object.controller == controller)
                .then_some(id as CardId)
        })
        .expect("fixed token factory entered")
}
fn reach_life(g: &mut Game) {
    for _ in 0..24 {
        if g.legal_actions().contains(&Action::DeclineResolutionLife) {
            return;
        }
        if matches!(g.state.pending, Pending::OrderTriggers { .. }) {
            choose(g, |action| matches!(action, Action::PutTrigger { .. }));
        } else {
            assert_eq!(g.state.pending, Pending::Priority);
            assert!(!g.state.stack.is_empty(), "missing independent life ward");
            act(g, Action::Pass);
        }
    }
    panic!("missing ward life payment");
}
fn remove(g: &mut Game, target: CardId) {
    let reference = g.object_ref(target);
    g.remove_abilities_exact(
        &[reference],
        reference,
        g.state.objects[target as usize].controller,
        AnimationDuration::Permanent,
    );
}
fn resolve_with_wards(g: &mut Game, pay: bool) -> usize {
    let mut paid = 0;
    for _ in 0..64 {
        if g.state.stack.is_empty()
            && g.state.resolving.is_none()
            && g.state.pending == Pending::Priority
        {
            return paid;
        }
        if g.legal_actions().contains(&Action::DeclineResolutionLife) {
            paid += 1;
            act(
                g,
                if pay {
                    Action::PayResolutionLife
                } else {
                    Action::DeclineResolutionLife
                },
            );
        } else if matches!(g.state.pending, Pending::OrderTriggers { .. }) {
            choose(g, |action| matches!(action, Action::PutTrigger { .. }));
        } else {
            assert_eq!(g.state.pending, Pending::Priority);
            act(g, Action::Pass);
        }
    }
    panic!("ward resolution exceeded its action bound");
}

#[test]
fn persist_grant_returns_under_owner_control_and_checks_predeath_minus_counter_both_seats() {
    let _scope = fixture_rules::activate(&[(fixture_card(), &PERSIST)]);
    for controller in [Player::P0, Player::P1] {
        let mut g = scenario(controller);
        put(&mut g, controller, "Grizzly Bears", Zone::Battlefield);
        let recipient = put(&mut g, controller.other(), "Hill Giant", Zone::Battlefield);
        g.state.objects[recipient as usize].controller = controller;
        let original = g.object_ref(recipient);
        murder(&mut g, controller, recipient);
        assert_eq!(g.object_view(recipient).zone, Zone::Battlefield);
        assert_ne!(g.object_ref(recipient), original);
        assert_eq!(g.object_view(recipient).controller, controller.other());
        assert!(!g.object_view(recipient).tapped);
        assert_eq!(g.counter_count(recipient, CounterKind::MinusOne), 1);
        assert_eq!(stats(&g, recipient), (2, 2));
        g.state.objects[recipient as usize].controller = controller;
        murder(&mut g, controller, recipient);
        assert_eq!(g.object_view(recipient).zone, Zone::Graveyard);
    }
}

#[test]
fn persist_grant_captures_simultaneous_deaths_and_excludes_self_token_opponent_and_noncreature() {
    let _scope = fixture_rules::activate(&[(fixture_card(), &PERSIST)]);
    for controller in [Player::P0, Player::P1] {
        let mut g = scenario(controller);
        let provider = put(&mut g, controller, "Grizzly Bears", Zone::Battlefield);
        let yours = put(&mut g, controller, "Hill Giant", Zone::Battlefield);
        let theirs = put(&mut g, controller.other(), "Hill Giant", Zone::Battlefield);
        let token = spirit(&mut g, controller);
        let artifact = put(&mut g, controller, "Swiftfoot Boots", Zone::Battlefield);
        g.sacrifice_batch(&[(controller, artifact)]);
        assert!(g.state.pending_triggers.is_empty());
        let spell = put(&mut g, controller, "Day of Judgment", Zone::Hand);
        cast(&mut g, spell, None);
        settle(&mut g);
        assert_eq!(g.object_view(provider).zone, Zone::Graveyard);
        assert_eq!(g.object_view(yours).zone, Zone::Battlefield);
        assert_eq!(g.counter_count(yours, CounterKind::MinusOne), 1);
        assert_eq!(g.object_view(theirs).zone, Zone::Graveyard);
        assert_eq!(g.object_view(token).zone, Zone::Outside);
    }
}

#[test]
fn persist_grant_obeys_recipient_timestamps_and_source_removal_dependency() {
    let _scope = fixture_rules::activate(&[(fixture_card(), &PERSIST)]);
    for controller in [Player::P0, Player::P1] {
        for blank_provider in [false, true] {
            for removal_first in [false, true] {
                let mut g = scenario(controller);
                let recipient = put(&mut g, controller, "Hill Giant", Zone::Battlefield);
                if removal_first && !blank_provider {
                    remove(&mut g, recipient);
                }
                let provider = put(&mut g, controller, "Grizzly Bears", Zone::Battlefield);
                if blank_provider {
                    remove(&mut g, provider);
                    if removal_first {
                        // A later static-effect timestamp cannot defeat a dependency
                        // on loss of the ability which generates that effect.
                        g.assign_static_source_timestamp(g.object_ref(provider));
                    }
                } else if !removal_first {
                    remove(&mut g, recipient);
                }
                murder(&mut g, controller, recipient);
                assert_eq!(
                    g.object_view(recipient).zone,
                    if !blank_provider && removal_first {
                        Zone::Battlefield
                    } else {
                        Zone::Graveyard
                    },
                    "controller={controller:?}, blank_provider={blank_provider}, removal_first={removal_first}"
                );
            }
        }
    }
}

#[test]
fn persist_grant_preserves_minus_counter_lki_before_simultaneous_sba_cancellation() {
    let _scope = fixture_rules::activate(&[(fixture_card(), &PERSIST)]);
    let mut g = scenario(Player::P0);
    put(&mut g, Player::P0, "Grizzly Bears", Zone::Battlefield);
    let recipient = put(&mut g, Player::P0, "Hill Giant", Zone::Battlefield);
    let counters = Counters::of(CounterKind::PlusOne, 1).with(CounterKind::MinusOne, 1);
    g.set_counters(recipient, counters);
    g.state.objects[recipient as usize].damage = 10;
    g.state_based_actions();
    assert_eq!(g.object_view(recipient).zone, Zone::Graveyard);
    assert!(g.state.pending_triggers.is_empty());
    assert!(g.state.stack.is_empty());
}

#[test]
fn independent_persist_grants_freeze_departed_providers_and_keep_exact_owned_return_links() {
    let _scope = fixture_rules::activate(&[(fixture_card(), &PERSIST)]);
    let mut g = scenario(Player::P0);
    let first = put(&mut g, Player::P0, "Grizzly Bears", Zone::Battlefield);
    let second = put(&mut g, Player::P0, "Grizzly Bears", Zone::Battlefield);
    let recipient = put(&mut g, Player::P0, "Hill Giant", Zone::Battlefield);
    g.sacrifice_batch(&[(Player::P0, recipient)]);
    assert_eq!(g.state.pending_triggers.len(), 2);
    assert_ne!(
        g.state.pending_triggers[0].attachment_trigger,
        g.state.pending_triggers[1].attachment_trigger
    );
    g.move_object(first, Zone::Exile);
    g.move_object(second, Zone::Exile);
    g.state.stack.append(&mut g.state.pending_triggers);
    let snapshot = g.snapshot();
    let mut indexed = g.clone();
    let pass = indexed
        .legal_actions()
        .iter()
        .position(|a| *a == Action::Pass)
        .unwrap();
    indexed
        .apply_index(indexed.actor().unwrap(), indexed.revision(), pass)
        .unwrap();
    act(&mut g, Action::Pass);
    assert_eq!(g.snapshot(), indexed.snapshot());
    settle(&mut g);
    assert_eq!(g.object_view(recipient).zone, Zone::Battlefield);
    assert_eq!(g.counter_count(recipient, CounterKind::MinusOne), 1);
    let returned = g.object_ref(recipient);
    // The second instance follows the old graveyard card, not the new permanent.
    assert_eq!(returned.generation, 2);
    g.restore(&snapshot).unwrap();
    let mut stale = g.clone();
    stale.move_object(recipient, Zone::Exile);
    stale.move_object(recipient, Zone::Graveyard);
    settle(&mut stale);
    assert_eq!(stale.object_view(recipient).zone, Zone::Graveyard);
    settle(&mut g);
    assert_eq!(g.object_ref(recipient), returned);
}

#[test]
fn static_ward_grant_distinguishes_own_printed_ward_and_includes_other_creature_tokens() {
    let _scope = fixture_rules::activate(&[(fixture_card(), &WARD)]);
    for controller in [Player::P0, Player::P1] {
        for (which, pay_ward, expected) in [(0, true, 1), (1, false, 1), (2, true, 1), (3, true, 0)]
        {
            let caster = controller.other();
            let mut g = scenario(caster);
            let provider = put(&mut g, controller, "Grizzly Bears", Zone::Battlefield);
            let recipient = match which {
                0 => provider,
                2 => spirit(&mut g, controller),
                3 => put(&mut g, caster, "Hill Giant", Zone::Battlefield),
                _ => put(&mut g, controller, "Hill Giant", Zone::Battlefield),
            };
            let spell = put(&mut g, caster, "Murder", Zone::Hand);
            cast(&mut g, spell, Some(Target::Card(recipient)));
            assert_eq!(resolve_with_wards(&mut g, pay_ward), expected);
            assert_eq!(
                g.state.players[caster.index()].life,
                if pay_ward {
                    20 - 2 * expected as i128
                } else {
                    20
                }
            );
            assert_eq!(
                g.object_view(recipient).zone,
                if pay_ward {
                    if which == 2 {
                        Zone::Outside
                    } else {
                        Zone::Graveyard
                    }
                } else {
                    Zone::Battlefield
                }
            );
        }
    }
}

#[test]
fn static_ward_grants_are_independent_and_obey_the_same_recipient_timestamp_order() {
    let _scope = fixture_rules::activate(&[(fixture_card(), &WARD)]);
    for removal in [0, 1, 2] {
        let mut g = scenario(Player::P1);
        let recipient = put(&mut g, Player::P0, "Hill Giant", Zone::Battlefield);
        if removal == 1 {
            remove(&mut g, recipient);
        }
        put(&mut g, Player::P0, "Grizzly Bears", Zone::Battlefield);
        let provider = put(&mut g, Player::P0, "Grizzly Bears", Zone::Battlefield);
        if removal == 2 {
            remove(&mut g, recipient);
        }
        let spell = put(&mut g, Player::P1, "Murder", Zone::Hand);
        cast(&mut g, spell, Some(Target::Card(recipient)));
        // Queued granted abilities outlive the live provider.
        g.move_object(provider, Zone::Exile);
        let expected = if removal == 2 { 0 } else { 2 };
        assert_eq!(resolve_with_wards(&mut g, true), expected);
        assert_eq!(g.state.players[1].life, 20 - 2 * expected as i128);
        assert_eq!(g.object_view(recipient).zone, Zone::Graveyard);
    }
}

#[test]
fn independent_life_wards_still_offer_payment_after_an_earlier_ward_counters_the_spell() {
    let _scope = fixture_rules::activate(&[(fixture_card(), &WARD)]);
    for controller in [Player::P0, Player::P1] {
        let caster = controller.other();
        let mut g = scenario(caster);
        put(&mut g, controller, "Grizzly Bears", Zone::Battlefield);
        put(&mut g, controller, "Grizzly Bears", Zone::Battlefield);
        let recipient = put(&mut g, controller, "Hill Giant", Zone::Battlefield);
        let spell = put(&mut g, caster, "Murder", Zone::Hand);
        cast(&mut g, spell, Some(Target::Card(recipient)));
        reach_life(&mut g);
        act(&mut g, Action::DeclineResolutionLife);
        assert_eq!(g.object_view(spell).zone, Zone::Graveyard);
        reach_life(&mut g);
        assert_eq!(g.actor(), Some(caster));
        let snapshot = g.snapshot();
        let mut indexed = g.clone();
        let pay = indexed
            .legal_actions()
            .iter()
            .position(|a| *a == Action::PayResolutionLife)
            .unwrap();
        indexed
            .apply_index(caster, indexed.revision(), pay)
            .unwrap();
        act(&mut g, Action::PayResolutionLife);
        assert_eq!(g.snapshot(), indexed.snapshot());
        settle(&mut g);
        assert_eq!(g.state.players[caster.index()].life, 18);
        assert_eq!(g.object_view(recipient).zone, Zone::Battlefield);
        g.restore(&snapshot).unwrap();
        // The option remains, but paying unaffordable life is still transactional.
        g.state.players[caster.index()].life = 1;
        reject(&mut g, Action::PayResolutionLife);
        act(&mut g, Action::DeclineResolutionLife);
        settle(&mut g);
        assert_eq!(g.state.players[caster.index()].life, 1);
        assert_eq!(g.object_view(recipient).zone, Zone::Battlefield);
    }
}

#[test]
fn life_ward_payment_uses_triggering_opponent_after_the_spell_changes_controller() {
    let _scope = fixture_rules::activate(&[(fixture_card(), &WARD)]);
    for controller in [Player::P0, Player::P1] {
        let caster = controller.other();
        let mut g = scenario(caster);
        put(&mut g, controller, "Grizzly Bears", Zone::Battlefield);
        let recipient = put(&mut g, controller, "Hill Giant", Zone::Battlefield);
        let spell = put(&mut g, caster, "Murder", Zone::Hand);
        cast(&mut g, spell, Some(Target::Card(recipient)));
        let packet = g
            .state
            .stack
            .iter_mut()
            .find(|item| item.source.is_some_and(|source| source.id == spell))
            .unwrap();
        packet.controller = controller;
        reach_life(&mut g);
        assert_eq!(g.actor(), Some(caster));
        act(&mut g, Action::PayResolutionLife);
        settle(&mut g);
        assert_eq!(g.state.players[caster.index()].life, 18);
        assert_eq!(g.state.players[controller.index()].life, 20);
        assert_eq!(g.object_view(recipient).zone, Zone::Graveyard);
    }
}
