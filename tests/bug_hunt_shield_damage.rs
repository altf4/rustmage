mod support;
use rustmage as _;
use rustmage_core::cards::CounterKind;
use rustmage_core::engine::*;
use support::*;

#[test]
fn one_shield_prevents_the_simultaneous_combat_damage_from_both_blockers() {
    let attacker_player = Player::P0;
    let defender = attacker_player.other();
    let mut game = scenario(attacker_player);
    let captain = put(
        &mut game,
        attacker_player,
        "Captain America, Super-Soldier",
        Zone::Hand,
    );
    cast(&mut game, captain, None);
    settle(&mut game);
    // We start this combat on a later turn with the original entry counter intact.
    game.state.objects[captain as usize].summoning_sick = false;
    assert_eq!(game.counter_count(captain, CounterKind::Shield), 1);
    let first = put(&mut game, defender, "Skybeast Tracker", Zone::Battlefield);
    let second = put(&mut game, defender, "Treetop Sentries", Zone::Battlefield);
    assert_eq!(stats(&game, first), (2, 4));
    assert_eq!(stats(&game, second), (2, 4));
    while game.state.pending != Pending::Attackers {
        act(&mut game, Action::Pass);
    }
    act(&mut game, Action::Attack { card: captain });
    act(&mut game, Action::FinishAttackers);
    while game.state.pending != Pending::Blockers {
        act(&mut game, Action::Pass);
    }
    for blocker in [first, second] {
        act(
            &mut game,
            Action::Block {
                blocker,
                attacker: captain,
            },
        );
    }
    act(&mut game, Action::FinishBlockers);
    for _ in 0..16 {
        if game.state.step == Step::CombatDamage && game.state.pending == Pending::Priority {
            break;
        }
        let action = if let Some(range) = game.damage_assignment_range() {
            // Both 2/4 blockers survive Captain America's 3 first-strike damage.
            Action::AssignDamage {
                target: range.target,
                amount: range.maximum,
            }
        } else {
            Action::Pass
        };
        let index = game
            .legal_actions()
            .iter()
            .position(|candidate| *candidate == action)
            .expect("combat proceeds without choosing a single hit to shield");
        let mut indexed = game.clone();
        indexed
            .apply_index(game.actor().unwrap(), game.revision(), index)
            .unwrap();
        act(&mut game, action);
        assert_eq!(indexed.snapshot(), game.snapshot());
    }
    assert_eq!(game.state.step, Step::CombatDamage);
    assert_eq!(game.object_view(first).zone, Zone::Battlefield);
    assert_eq!(game.object_view(second).zone, Zone::Battlefield);
    assert_eq!(
        game.object_view(captain).zone,
        Zone::Battlefield,
        "both 2-power blockers deal damage at the same time and one shield prevents all four"
    );
    assert_eq!(game.counter_count(captain, CounterKind::Shield), 0);
    assert_eq!(game.object_view(captain).damage, 0);
}

#[test]
fn shield_is_spent_once_per_recipient_even_with_unpreventable_hits() {
    let p = Player::P0;
    let mut game = scenario(p);
    let first = put(&mut game, p.other(), "Bear Cub", Zone::Battlefield);
    let second = put(&mut game, p.other(), "Hill Giant", Zone::Battlefield);
    let a = put(&mut game, p, "Hill Giant", Zone::Battlefield);
    let b = put(&mut game, p, "Hill Giant", Zone::Battlefield);
    for recipient in [a, b] {
        game.add_counter(recipient, CounterKind::Shield, 2);
    }
    let hit = |source, target, preventable| DamageEvent {
        source,
        target: Target::Card(target),
        amount: 1,
        combat: false,
        preventable,
    };
    let first = game.damage_source(first);
    let second = game.damage_source(second);
    let mut hits = [
        hit(first, a, false),
        hit(first, b, true),
        hit(second, a, false),
        hit(second, a, true),
    ];
    for order in 0..2 {
        let mut branch = game.clone();
        if order == 1 {
            hits.reverse();
        }
        branch.apply_damage_batch(&hits);
        assert!(!branch.advance_replacements(false));
        assert_eq!(branch.counter_count(a, CounterKind::Shield), 1);
        assert_eq!(branch.counter_count(b, CounterKind::Shield), 1);
        assert_eq!(branch.object_view(a).damage, 2);
        assert_eq!(branch.object_view(b).damage, 0);

        branch.apply_damage_batch(&[hit(second, a, true)]);
        assert!(!branch.advance_replacements(false));
        assert_eq!(branch.counter_count(a, CounterKind::Shield), 0);
        assert_eq!(branch.object_view(a).damage, 2);
    }
}

#[test]
fn competing_combat_prevention_can_preserve_a_shield() {
    let p = Player::P0;
    let mut game = scenario(p);
    let bank = put(&mut game, p.other(), "Fog Bank", Zone::Battlefield);
    let first = put(&mut game, p, "Bear Cub", Zone::Battlefield);
    let second = put(&mut game, p, "Hill Giant", Zone::Battlefield);
    game.add_counter(bank, CounterKind::Shield, 1);
    game.apply_damage_batch(&[first, second].map(|id| DamageEvent {
        source: game.damage_source(id),
        target: Target::Card(bank),
        amount: 1,
        combat: true,
        preventable: true,
    }));
    assert!(game.advance_replacements(false));
    assert_eq!(game.actor(), Some(p.other()));
    let view = game.observe(p.other()).replacement_choice.unwrap();
    assert_eq!(view.damage_event_order.len(), 2);
    act(
        &mut game,
        Action::ChooseDamageEvent {
            index: view.damage_event_order[1].index,
        },
    );
    let prevent_combat = Action::ChooseDamagePrevention {
        source: game.object_ref(bank),
        prevention: DamagePreventionKind::CombatToRecipient,
    };
    act(&mut game, prevent_combat.clone());
    act(&mut game, prevent_combat);
    assert_eq!(game.counter_count(bank, CounterKind::Shield), 1);
    assert_eq!(game.object_view(bank).damage, 0);
}
