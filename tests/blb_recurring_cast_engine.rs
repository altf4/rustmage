//! Independent source and duration for recurring cast-triggered abilities.
mod support;

use rustmage as _;
use rustmage_core::cards::authored::CardRules;
use rustmage_core::{cards::*, engine::*};
use support::*;

const RECURRING: CardRules = CardRules {
    effect: Effect::Spell(
        TargetPlan::NONE,
        &[Instruction::ScheduleCastTriggerUntilEndOfNextTurn { ability: 0 }],
    ),
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::Reflexive,
        targets: TargetPlan::Single(TargetSpec {
            rule: TargetRule::Creature,
            min: 0,
            max: 1,
        }),
        program: &[Instruction::Damage(2)],
    }],
    ..CardRules::EMPTY
};

fn named(name: &str) -> Card {
    Card::from_name(name).unwrap()
}

fn install(game: &mut Game, player: Player) -> (CardId, ObjectRef) {
    let id = put(game, player, "Dragon Fodder", Zone::Hand);
    cast(game, id, None);
    let source = game.object_ref(id);
    assert!(game.observe(player).turn_callbacks.is_none());
    settle(game);
    assert_eq!(game.object_view(id).zone, Zone::Graveyard);
    assert_ne!(game.object_ref(id), source);
    let rows = game.observe(player).turn_callbacks.unwrap();
    assert_eq!(rows.len(), 1);
    assert_eq!(rows[0].event, TurnCallbackEvent::SpellCast);
    assert_eq!(rows[0].expiry, Some(TurnCallbackExpiry::EndOfNextTurn));
    assert_eq!(rows[0].item.source.unwrap().object, source);
    assert_eq!(game.observe(player.other()).turn_callbacks.unwrap(), rows);
    (id, source)
}

fn finish_targets(game: &mut Game) {
    if game.legal_actions().contains(&Action::FinishTargets) {
        act(game, Action::FinishTargets);
    }
}

fn finish_choices(game: &mut Game, target: CardId) {
    for _ in 0..32 {
        if game.state.pending == Pending::Priority && game.state.stack.is_empty() {
            return;
        }
        let pick = Action::ChooseTarget {
            target: Target::Card(target),
        };
        if game.legal_actions().contains(&pick) {
            act(game, pick);
        } else {
            choose(game, |action| {
                matches!(
                    action,
                    Action::Pass
                        | Action::PutTrigger { .. }
                        | Action::FinishTargets
                        | Action::FinishStackTargets
                )
            });
        }
    }
    panic!("recurring cast choices exceeded scenario limit");
}

#[test]
fn recurring_cast_trigger_keeps_its_source_and_targets_when_the_spell_is_countered() {
    let _rules = fixture_rules::activate(&[(named("Dragon Fodder"), &RECURRING)]);
    for player in [Player::P0, Player::P1] {
        let opponent = player.other();
        let mut root = scenario(player);
        let (registration, source) = install(&mut root, player);
        let creature = put(&mut root, opponent, "Gigantosaurus", Zone::Battlefield);
        let spell = put(&mut root, player, "Shock", Zone::Hand);
        cast(&mut root, spell, Some(Target::Player(opponent)));
        assert!(matches!(root.state.pending, Pending::Targets { .. }));
        let opponent_view = root.observe(opponent);
        let announcement = opponent_view.announcing.unwrap();
        assert_eq!(announcement.controller, player);
        assert!(announcement.targets.is_empty());
        assert!(opponent_view.targeting_group.is_none());
        assert!(root.observe(player).targeting_group.is_some());
        let pick = Action::ChooseTarget {
            target: Target::Card(creature),
        };
        reject(
            &mut root,
            Action::ChooseTarget {
                target: Target::Player(opponent),
            },
        );
        let checkpoint = root.snapshot();
        let index = root
            .legal_actions()
            .iter()
            .position(|action| *action == pick)
            .unwrap();
        let revision = root.revision();
        let mut raw = root.clone();
        let mut indexed = root.clone();
        drop(root);
        act(&mut raw, pick);
        indexed.apply_index(player, revision, index).unwrap();
        assert_eq!(raw.snapshot(), indexed.snapshot());
        for branch in [&mut raw, &mut indexed] {
            finish_targets(branch);
            if branch.actor() != Some(opponent) {
                act(branch, Action::Pass);
            }
            let cancel = put(branch, opponent, "Cancel", Zone::Hand);
            cast(branch, cancel, Some(Target::Spell(spell)));
            settle(branch);
            assert_eq!(branch.object_view(spell).zone, Zone::Graveyard);
            assert_eq!(branch.object_view(registration).zone, Zone::Graveyard);
            assert_eq!(branch.state.players[opponent.index()].life, 20);
            assert_eq!(branch.object_view(creature).damage, 2);
            let damage = branch.observe(player).permanent_damage_history;
            assert_eq!(damage.len(), 1);
            assert_eq!(damage[0].source, source);
            assert_eq!(damage[0].recipient, branch.object_ref(creature));
        }
        assert_eq!(raw.snapshot(), indexed.snapshot());

        indexed.restore(&checkpoint).unwrap();
        act(&mut indexed, Action::FinishTargets);
        settle(&mut indexed);
        assert_eq!(indexed.object_view(creature).damage, 0);
        assert_eq!(indexed.state.players[opponent.index()].life, 18);
    }
}

#[test]
fn recurring_cast_trigger_uses_actual_casts_until_the_end_of_the_next_own_turn() {
    let _rules = fixture_rules::activate(&[(named("Dragon Fodder"), &RECURRING)]);
    let player = Player::P0;
    let opponent = player.other();
    let mut game = scenario(player);
    let creature = put(&mut game, opponent, "Gigantosaurus", Zone::Battlefield);
    install(&mut game, player);

    let teach = put(&mut game, player, "Teach by Example", Zone::Hand);
    cast(&mut game, teach, None);
    act(&mut game, Action::FinishTargets);
    settle(&mut game);
    let shock = put(&mut game, player, "Shock", Zone::Hand);
    cast(&mut game, shock, Some(Target::Player(opponent)));
    finish_choices(&mut game, creature);
    assert_eq!(game.state.players[opponent.index()].life, 16);
    assert_eq!(game.object_view(creature).damage, 2);

    game.finish_cleanup();
    assert_eq!(game.state.active, opponent);
    assert_eq!(
        game.observe(player).turn_callbacks.unwrap()[0].expiry,
        Some(TurnCallbackExpiry::EndOfNextTurn)
    );
    game.state.players[opponent.index()].mana = [0, 0, 0, 1, 0, 0];
    let opposing_shock = put(&mut game, opponent, "Shock", Zone::Hand);
    cast(&mut game, opposing_shock, Some(Target::Player(player)));
    settle(&mut game);
    assert_eq!(game.object_view(creature).damage, 0);

    game.state.players[player.index()].mana = [0, 0, 0, 1, 0, 0];
    act(&mut game, Action::Pass);
    let off_turn_shock = put(&mut game, player, "Shock", Zone::Hand);
    cast(&mut game, off_turn_shock, Some(Target::Player(opponent)));
    finish_choices(&mut game, creature);
    assert_eq!(game.object_view(creature).damage, 2);

    game.finish_cleanup();
    assert_eq!(game.state.active, player);
    let expiry = Some(TurnCallbackExpiry::EndOfTurn(game.state.turn));
    assert_eq!(
        game.observe(player).turn_callbacks.unwrap()[0].expiry,
        expiry
    );
    game.state.step = Step::End;
    game.state.players[player.index()].mana = [0, 0, 0, 1, 0, 0];
    let next_shock = put(&mut game, player, "Shock", Zone::Hand);
    cast(&mut game, next_shock, Some(Target::Player(opponent)));
    finish_choices(&mut game, creature);
    assert_eq!(game.object_view(creature).damage, 2);

    game.finish_cleanup();
    assert!(game.observe(player).turn_callbacks.is_none());
    game.state.players[player.index()].mana = [0, 0, 0, 1, 0, 0];
    act(&mut game, Action::Pass);
    let expired = put(&mut game, player, "Shock", Zone::Hand);
    cast(&mut game, expired, Some(Target::Player(opponent)));
    settle(&mut game);
    assert_eq!(game.object_view(creature).damage, 0);
}
