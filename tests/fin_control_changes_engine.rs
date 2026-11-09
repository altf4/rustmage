//! Shared control-loss events, including the simultaneous cleanup action.
mod support;
use rustmage as _;
use rustmage_core::cards::authored::CardRules;
use rustmage_core::{cards::*, engine::*};
use support::*;

const TREASURE: Card = Card(5302);
const WATCH: CardRules = CardRules {
    effect: Effect::Creature(3, 3),
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::OpponentTakesPermanent,
        targets: TargetPlan::NONE,
        program: &[Instruction::CreateTokens {
            token: TREASURE,
            amount: 1,
            tapped: false,
        }],
    }],
    ..CardRules::EMPTY
};
const TAKE_ARTIFACTS: CardRules = CardRules {
    effect: Effect::Spell(
        TargetPlan::NONE,
        &[Instruction::TakeOpposingArtifactsUntilEndTurn],
    ),
    ..CardRules::EMPTY
};

const EQUIPMENT: TargetRule = TargetRule::Filtered {
    base: &TargetRule::Object {
        zone: TargetZone::Battlefield,
        kinds: 16,
        relation: -1,
        other: false,
        nonland: false,
        tapped: false,
        min_mana: 0,
        max_mana: u32::MAX,
    },
    predicate: &TargetPredicate::Subtype("Equipment"),
};

const WATCH_EQUIPMENT: CardRules = CardRules {
    effect: Effect::Spell(
        TargetPlan::Groups(&[
            TargetSpec::one(TargetRule::Object {
                zone: TargetZone::Battlefield,
                kinds: 1,
                relation: 0,
                other: false,
                nonland: false,
                tapped: false,
                min_mana: 0,
                max_mana: u32::MAX,
            }),
            TargetSpec::one(EQUIPMENT),
        ]),
        &[
            Instruction::SelectGroup(1),
            Instruction::GainControlUntilEndTurn,
            Instruction::AttachEquipmentByGroups {
                equipment: 1,
                recipient: 0,
            },
            Instruction::DelayTargetEvent {
                event: DelayedTargetEvent::ControlLostWhileAttached,
                ability: 0,
            },
        ],
    ),
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::DelayedTargetControlLostWhileAttached,
        targets: TargetPlan::NONE,
        program: &[Instruction::UnattachEventObject],
    }],
    ..CardRules::EMPTY
};

const PASS_CONTROL: CardRules = CardRules {
    effect: Effect::Creature(2, 4),
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::PlayerDamaged {
            subject: DamageSubject::Source,
            combat_only: true,
            opponent_only: false,
            one_or_more: false,
        },
        targets: TargetPlan::NONE,
        program: &[
            Instruction::GainControl {
                subject: Subject::Source,
                controller: PlayerSet::DamagedPlayer,
            },
            Instruction::IfSubjectControlledBy {
                subject: Subject::Source,
                controller: PlayerSet::DamagedPlayer,
                skip: 3,
            },
            Instruction::DrawScaled(PlayerSet::Controller, AmountValue::DamageEventAmount),
            Instruction::CreateTokensScaled {
                token: TREASURE,
                amount: AmountValue::DamageEventAmount,
                tapped: true,
            },
            Instruction::LoseLifeScaled(PlayerSet::Controller, AmountValue::DamageEventAmount),
        ],
    }],
    ..CardRules::EMPTY
};

fn settle_with_triggers(game: &mut Game) {
    for _ in 0..36 {
        if game.state.pending == Pending::Priority
            && game.state.stack.is_empty()
            && game.state.pending_triggers.is_empty()
        {
            return;
        }
        choose(game, |action| {
            matches!(action, Action::Pass | Action::PutTrigger { .. })
        });
    }
    panic!("control-loss trigger budget exhausted");
}

fn treasures(game: &Game, player: Player) -> Vec<ObjectView> {
    game.observe(player)
        .objects
        .into_iter()
        .filter(|view| {
            view.zone == Zone::Battlefield && view.card == TREASURE && view.controller == player
        })
        .collect()
}

fn pass_at_end(game: &mut Game) {
    game.state.step = Step::End;
    act(game, Action::Pass);
    act(game, Action::Pass);
}

#[test]
fn a_control_batch_uses_the_former_controller_of_each_permanent_and_observer() {
    let _rules = fixture_rules::activate(&[
        (Card::from_name("Opt").unwrap(), &TAKE_ARTIFACTS),
        (Card::from_name("Interface Ace").unwrap(), &WATCH),
    ]);
    let player = Player::P0;
    let former = player.other();
    let mut game = scenario(player);
    let observer = put(&mut game, former, "Interface Ace", Zone::Battlefield);
    let equipment = put(&mut game, former, "Swiftfoot Boots", Zone::Battlefield);
    let spell = put(&mut game, player, "Opt", Zone::Hand);
    cast(&mut game, spell, None);
    act(&mut game, Action::Pass);
    let before = game.snapshot();
    let index = game
        .legal_actions()
        .iter()
        .position(|action| *action == Action::Pass)
        .unwrap();
    let revision = game.revision();
    let mut indexed = game.clone();
    indexed.apply_index(former, revision, index).unwrap();
    act(&mut game, Action::Pass);
    assert_eq!(indexed.snapshot(), game.snapshot());
    assert_eq!(
        game.state.pending,
        Pending::OrderTriggers { player: former }
    );
    assert_eq!(game.object_view(observer).controller, player);
    assert_eq!(game.object_view(equipment).controller, player);
    assert_eq!(game.state.ordering_triggers.len(), 2);
    indexed.restore(&before).unwrap();
    assert_eq!(
        indexed.apply_index(former, revision, index),
        Err(Error::StaleRevision)
    );
    act(&mut indexed, Action::Pass);
    assert_eq!(indexed.snapshot(), game.snapshot());
    settle_with_triggers(&mut game);
    assert_eq!(treasures(&game, former).len(), 2);
    assert!(treasures(&game, player).is_empty());

    let turn = game.state.turn;
    pass_at_end(&mut game);
    assert_eq!(game.state.step, Step::Cleanup);
    assert_eq!(game.state.pending, Pending::OrderTriggers { player });
    assert_eq!(game.object_view(observer).controller, former);
    assert_eq!(game.object_view(equipment).controller, former);
    settle_with_triggers(&mut game);
    assert_eq!(treasures(&game, player).len(), 2);
    act(&mut game, Action::Pass);
    act(&mut game, Action::Pass);
    assert_eq!(game.state.turn, turn + 1);
    assert_eq!(treasures(&game, player).len(), 2);
}

fn uniform_setup(player: Player) -> (Game, CardId, CardId) {
    let mut game = scenario(player);
    let creature = put(&mut game, player, "Bear Cub", Zone::Battlefield);
    let equipment = put(
        &mut game,
        player.other(),
        "Swiftfoot Boots",
        Zone::Battlefield,
    );
    let spell = put(&mut game, player, "Opt", Zone::Hand);
    act(&mut game, Action::AnnounceCast { card: spell });
    for card in [creature, equipment] {
        let target = Action::ChooseTarget {
            target: Target::Card(card),
        };
        if !game.legal_actions().contains(&target) {
            act(&mut game, Action::FinishTargets);
        }
        act(&mut game, target);
    }
    if game.legal_actions().contains(&Action::FinishTargets) {
        act(&mut game, Action::FinishTargets);
    }
    pay(&mut game);
    settle(&mut game);
    assert_eq!(game.object_view(equipment).controller, player);
    assert_eq!(
        game.object_view(equipment).attached_to,
        Some(game.object_ref(creature))
    );
    (game, equipment, creature)
}

#[test]
fn cleanup_control_loss_opens_priority_before_detaching_the_exact_equipment() {
    let _rules = fixture_rules::activate(&[(Card::from_name("Opt").unwrap(), &WATCH_EQUIPMENT)]);
    let player = Player::P1;
    for blink in [false, true] {
        let (mut game, equipment, creature) = uniform_setup(player);
        pass_at_end(&mut game);
        assert_eq!(game.state.step, Step::Cleanup);
        assert_eq!(game.object_view(equipment).controller, player.other());
        assert_eq!(
            game.object_view(equipment).attached_to,
            Some(game.object_ref(creature))
        );
        assert_eq!(game.state.stack.len(), 1);
        assert_eq!(game.state.stack[0].controller, player);
        if blink {
            game.move_object(equipment, Zone::Exile);
            game.move_object(equipment, Zone::Battlefield);
            assert!(game.attach_equipment(game.object_ref(equipment), game.object_ref(creature)));
        }
        settle(&mut game);
        assert_eq!(
            game.object_view(equipment).attached_to,
            blink.then(|| game.object_ref(creature))
        );
    }
}

#[test]
fn control_loss_watches_this_turn_again_and_rechecks_the_live_creature_on_resolution() {
    let _rules = fixture_rules::activate(&[(Card::from_name("Opt").unwrap(), &WATCH_EQUIPMENT)]);
    let player = Player::P0;
    let (mut game, equipment, creature) = uniform_setup(player);
    game.record_control(game.object_ref(equipment), player.other(), None);
    assert_eq!(game.state.pending_triggers.len(), 1);
    game.record_control(game.object_ref(creature), player.other(), None);
    settle_with_triggers(&mut game);
    assert_eq!(
        game.object_view(equipment).attached_to,
        Some(game.object_ref(creature))
    );

    game.record_control(game.object_ref(equipment), player, None);
    game.record_control(game.object_ref(creature), player, None);
    game.record_control(game.object_ref(equipment), player.other(), None);
    assert_eq!(game.state.pending_triggers.len(), 1);
    settle_with_triggers(&mut game);
    assert_eq!(game.object_view(equipment).attached_to, None);
}

#[test]
fn control_of_the_exact_combat_source_gates_the_frozen_damage_consequence() {
    let _rules = fixture_rules::activate(&[(Card::GrizzlyBears, &PASS_CONTROL)]);
    let player = Player::P0;
    for blink in [false, true] {
        let mut game = scenario(player);
        let dealer = put(&mut game, player, "Grizzly Bears", Zone::Battlefield);
        for _ in 0..10 {
            if game.state.pending == Pending::Attackers {
                break;
            }
            act(&mut game, Action::Pass);
        }
        act(&mut game, Action::Attack { card: dealer });
        act(&mut game, Action::FinishAttackers);
        for _ in 0..14 {
            if !game.state.stack.is_empty() {
                break;
            }
            choose(&mut game, |action| {
                matches!(action, Action::Pass | Action::FinishBlockers)
            });
        }
        assert_eq!(game.state.stack.len(), 1);
        assert_eq!(game.state.players[player.other().index()].life, 18);
        if blink {
            game.move_object(dealer, Zone::Exile);
            game.move_object(dealer, Zone::Battlefield);
        }
        settle(&mut game);
        assert_eq!(
            game.object_view(dealer).controller,
            if blink { player } else { player.other() }
        );
        assert_eq!(game.hand_count(player), if blink { 0 } else { 2 });
        assert_eq!(
            game.state.players[player.index()].life,
            if blink { 20 } else { 18 }
        );
        let tokens = treasures(&game, player);
        assert_eq!(tokens.len(), if blink { 0 } else { 2 });
        assert!(tokens.iter().all(|view| view.tapped));
    }
}
