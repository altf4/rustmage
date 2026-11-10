mod support;
use rustmage as _;
use rustmage_core::cards::authored::{CardExtras, CardRules};
use rustmage_core::{cards::*, engine::*};
use support::*;

const DARKNESS: TriggeredAbility = TriggeredAbility {
    condition: TriggerCondition::SourceDealtDamage,
    targets: TargetPlan::NONE,
    program: &[
        Instruction::LoseLifeScaled(PlayerSet::Controller, AmountValue::DamageEventAmount),
        Instruction::If {
            guard: StepGuard::Expression {
                value: StatValue::ControllerLifeAtMostHalfStarting,
                minimum: Some(1),
                maximum: None,
            },
            skip: 2,
        },
        Instruction::UntapSource,
        Instruction::TransformSource,
    ],
};
const BODY: CardRules = CardRules {
    effect: Effect::Creature(2, 3),
    keywords: Keywords::DEATHTOUCH,
    triggers: &[DARKNESS],
    ..CardRules::EMPTY
};
const FRONT: CardRules = CardRules {
    extras: &CardExtras {
        transforming_double_face: true,
        ..CardExtras::EMPTY
    },
    ..BODY
};
const WITHER: CardRules = CardRules {
    keywords: Keywords::WITHER.union(Keywords::LIFELINK),
    ..BODY
};
const MULTI: CardRules = CardRules {
    effect: Effect::Spell(
        TargetPlan::Single(TargetSpec {
            rule: TargetRule::Any,
            min: 3,
            max: 3,
        }),
        &[Instruction::PermanentsDealPowerDamage {
            filters: &[BattlefieldFilter {
                kinds: 1,
                relation: 0,
                nonland: false,
                tapped: None,
                without: Keywords::NONE,
                subtype: None,
                max_mana: u32::MAX,
            }],
            exclude_source: false,
        }],
    ),
    ..CardRules::EMPTY
};

fn pass_priority(game: &mut Game) {
    act(game, Action::Pass);
    act(game, Action::Pass);
}

fn receipt(item: &SpellView) -> (ObjectRef, u128) {
    let event = item.event_object.as_deref().unwrap();
    let EventObjectFacts::DamageDealt { amount } = &event.facts else {
        panic!("missing source damage amount");
    };
    (event.object, *amount)
}

#[test]
fn one_receipt_per_exact_source_counts_players_and_permanents_after_prevention() {
    let _rules = fixture_rules::activate(&[
        (Card::from_name("Bear Cub").unwrap(), &BODY),
        (Card::Shock, &MULTI),
    ]);
    for player in [Player::P0, Player::P1] {
        let opponent = player.other();
        let mut game = scenario(player);
        let first = put(&mut game, player, "Bear Cub", Zone::Battlefield);
        let second = put(&mut game, player, "Bear Cub", Zone::Battlefield);
        game.state.objects[second as usize].growth.power = 1;
        game.grant_keyword_effects(&[first], game.object_ref(first), Keywords::LIFELINK);
        let shielded = put(&mut game, opponent, "Hill Giant", Zone::Battlefield);
        game.add_counter(shielded, CounterKind::Shield, 1);
        let walker = put(
            &mut game,
            opponent,
            "Elspeth, Storm Slayer",
            Zone::Battlefield,
        );
        game.add_counter(walker, CounterKind::Loyalty, 9);
        let spell = put(&mut game, player, "Shock", Zone::Hand);
        announce(
            &mut game,
            spell,
            &[
                Target::Player(opponent),
                Target::Card(shielded),
                Target::Card(walker),
            ],
        );
        pass_priority(&mut game);
        let mut receipts: Vec<_> = game.state.ordering_triggers.iter().map(receipt).collect();
        receipts.sort_unstable();
        assert_eq!(
            receipts,
            vec![(game.object_ref(first), 4), (game.object_ref(second), 6)]
        );
        assert_eq!(game.state.players[player.index()].life, 24);
        assert_eq!(game.state.players[opponent.index()].life, 15);
        assert_eq!(game.counter_count(walker, CounterKind::Loyalty), 4);
        assert_eq!(game.counter_count(shielded, CounterKind::Shield), 0);
        assert_eq!(game.object_view(shielded).damage, 0);
        choose(&mut game, |action| {
            matches!(action, Action::PutTrigger { .. })
        });
        settle(&mut game);
        assert_eq!(game.state.players[player.index()].life, 14);
    }
}

#[test]
fn wither_counter_choices_keep_the_original_damage_and_owned_trigger_receipt() {
    let _rules = fixture_rules::activate(&[(Card::from_name("Bear Cub").unwrap(), &WITHER)]);
    let player = Player::P0;
    let opponent = player.other();
    let mut raw = scenario(player);
    let dealer = put(&mut raw, player, "Bear Cub", Zone::Battlefield);
    let target = put(&mut raw, opponent, "Gigantosaurus", Zone::Battlefield);
    for _ in 0..2 {
        put(&mut raw, opponent, "Doubling Season", Zone::Battlefield);
    }
    let bite = put(&mut raw, player, "Bite Down", Zone::Hand);
    announce(
        &mut raw,
        bite,
        &[Target::Card(dealer), Target::Card(target)],
    );
    pass_priority(&mut raw);
    assert_eq!(raw.actor(), Some(opponent));
    assert_eq!(raw.state.pending_triggers.len(), 1);
    assert_eq!(
        receipt(&raw.state.pending_triggers[0]),
        (raw.object_ref(dealer), 2)
    );
    assert_eq!(raw.state.players[player.index()].life, 20);

    let actions = raw.legal_actions();
    let index = actions
        .iter()
        .position(|action| matches!(action, Action::ChooseReplacement { .. }))
        .unwrap();
    let action = actions[index].clone();
    let revision = raw.revision();
    let saved = raw.snapshot();
    let mut indexed = raw.clone();
    act(&mut raw, action.clone());
    indexed.apply_index(opponent, revision, index).unwrap();
    assert_eq!(raw.snapshot(), indexed.snapshot());
    assert_eq!(
        receipt(raw.state.stack.last().unwrap()),
        (raw.object_ref(dealer), 2)
    );
    assert_eq!(raw.counter_count(target, CounterKind::MinusOne), 8);
    assert_eq!(raw.state.players[player.index()].life, 22);
    settle(&mut raw);
    settle(&mut indexed);
    assert_eq!(raw.state.players[player.index()].life, 20);
    assert_eq!(raw.snapshot(), indexed.snapshot());

    indexed.restore(&saved).unwrap();
    act(&mut indexed, action);
    settle(&mut indexed);
    assert_eq!(raw.snapshot(), indexed.snapshot());
}

#[test]
fn combat_receipt_survives_lethal_damage_and_checks_the_threshold_after_life_loss() {
    let front = Card::from_name("Brigid, Clachan's Heart").unwrap();
    let _rules = fixture_rules::activate(&[(front, &FRONT)]);
    let player = Player::P0;
    let opponent = player.other();
    for (starting_life, blocked, prevented) in [
        (12, false, false),
        (13, false, false),
        (12, true, false),
        (12, true, true),
    ] {
        let mut game = scenario(player);
        game.state.players[player.index()].life = starting_life;
        let dealer = put(&mut game, player, front.name(), Zone::Battlefield);
        let original = game.object_ref(dealer);
        let blocker = put(&mut game, opponent, "Hill Giant", Zone::Battlefield);
        if prevented {
            game.add_counter(blocker, CounterKind::Shield, 1);
        }
        pass_priority(&mut game);
        pass_priority(&mut game);
        act(&mut game, Action::Attack { card: dealer });
        act(&mut game, Action::FinishAttackers);
        pass_priority(&mut game);
        if blocked {
            act(
                &mut game,
                Action::Block {
                    blocker,
                    attacker: dealer,
                },
            );
        }
        act(&mut game, Action::FinishBlockers);
        pass_priority(&mut game);
        if prevented {
            assert!(game.state.stack.is_empty());
        } else {
            let trigger = game.state.stack.last().unwrap();
            assert_eq!(receipt(trigger), (original, 2));
            assert_eq!(trigger.controller, player);
        }
        settle(&mut game);
        assert_eq!(
            game.state.players[player.index()].life,
            starting_life - if prevented { 0 } else { 2 }
        );
        let dealer = game.object_view(dealer);
        if blocked {
            assert_eq!(dealer.zone, Zone::Graveyard);
            assert_ne!(dealer.reference, original);
        } else {
            assert_eq!(dealer.reference, original);
            let transformed = starting_life == 12;
            assert_eq!(dealer.tapped, !transformed);
            assert_eq!(
                dealer.card,
                if transformed {
                    front.back_face().unwrap()
                } else {
                    front
                }
            );
        }
    }
}
