//! Untargeted damage-dealer blink and exact turn-duration subscriptions.
mod support;

use rustmage as _;
use rustmage_core::cards::authored::CardRules;
use rustmage_core::{cards::*, engine::*};
use support::*;

const OPTIONAL_BLINK: &[Instruction] = &[
    Instruction::MayIfCombatDamageDealerOnBattlefield { skip: 1 },
    Instruction::BlinkCombatDamageDealer,
];

const DELAYED_PROVIDER: CardRules = CardRules {
    effect: Effect::Creature(2, 3),
    triggers: &[
        TriggeredAbility {
            condition: TriggerCondition::SelfEnters,
            targets: TargetPlan::Single(TargetSpec::one(TargetRule::Object {
                zone: TargetZone::Battlefield,
                kinds: 1,
                relation: 0,
                other: false,
                nonland: false,
                tapped: false,
                min_mana: 0,
                max_mana: u32::MAX,
            })),
            program: &[
                Instruction::GrantKeywords {
                    subject: Subject::Targets,
                    keywords: Keywords::CANT_BE_BLOCKED,
                },
                Instruction::DelayTargetEvent {
                    event: DelayedTargetEvent::CombatDamage,
                    ability: 1,
                },
            ],
        },
        TriggeredAbility {
            condition: TriggerCondition::DelayedTargetEvent,
            targets: TargetPlan::NONE,
            program: OPTIONAL_BLINK,
        },
    ],
    ..CardRules::EMPTY
};

const PLAYER_DAMAGE: TriggerCondition = TriggerCondition::PlayerDamaged {
    subject: DamageSubject::Creatures {
        filter: CreatureFilter {
            relation: 0,
            other: false,
            subtype: None,
            keyword: Keywords::NONE,
            attacking: false,
            tokens: None,
            artifact: false,
        },
        qualification: DamageQualification::Any,
    },
    combat_only: true,
    opponent_only: false,
    one_or_more: false,
};

const PLAYER_DAMAGE_PROVIDER: CardRules = CardRules {
    effect: Effect::Permanent,
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::When {
            guard: StepGuard::ClassLevelAtLeast(3),
            event: &PLAYER_DAMAGE,
        },
        targets: TargetPlan::NONE,
        program: OPTIONAL_BLINK,
    }],
    ..CardRules::EMPTY
};

fn named(name: &str) -> Card {
    Card::from_name(name).unwrap()
}

fn pass_pair(game: &mut Game) {
    act(game, Action::Pass);
    act(game, Action::Pass);
}

fn place_triggers(game: &mut Game) {
    while let Some(action) = game
        .legal_actions()
        .into_iter()
        .find(|action| matches!(action, Action::PutTrigger { .. }))
    {
        act(game, action);
    }
}

fn first_strike(game: &mut Game, attackers: &[CardId]) {
    for _ in 0..8 {
        if game.state.pending == Pending::Attackers {
            break;
        }
        act(game, Action::Pass);
    }
    assert_eq!(game.state.pending, Pending::Attackers);
    for &card in attackers {
        act(game, Action::Attack { card });
    }
    act(game, Action::FinishAttackers);
    pass_pair(game);
    act(game, Action::FinishBlockers);
    pass_pair(game);
    assert_eq!(game.state.step, Step::FirstStrikeDamage);
}

fn schedule(game: &mut Game, player: Player, watched: CardId) -> CardId {
    let source = put(game, player, "Hill Giant", Zone::Hand);
    cast(game, source, None);
    pass_pair(game);
    act(
        game,
        Action::ChooseTarget {
            target: Target::Card(watched),
        },
    );
    settle(game);
    source
}

#[test]
fn a_delayed_exact_watch_repeats_after_declining_and_returns_under_owner_control() {
    let _rules = fixture_rules::activate(&[(Card::HillGiant, &DELAYED_PROVIDER)]);
    for player in [Player::P0, Player::P1] {
        let mut game = scenario(player);
        let watched = put(
            &mut game,
            player.other(),
            "Grizzly Bears",
            Zone::Battlefield,
        );
        let original = game.object_ref(watched);
        game.change_controller(original, player);
        game.grant_keyword_effects(
            &[watched],
            original,
            Keywords::HASTE.union(Keywords::DOUBLE_STRIKE),
        );
        let source = schedule(&mut game, player, watched);
        assert!(game.keywords(watched).contains(Keywords::CANT_BE_BLOCKED));
        game.move_object(source, Zone::Graveyard);
        assert_eq!(
            game.observe(player).delayed_target_events.unwrap()[0].subject,
            original
        );

        first_strike(&mut game, &[watched]);
        place_triggers(&mut game);
        let trigger = game.state.stack.last().unwrap();
        assert!(trigger.targets.is_empty());
        assert_eq!(
            trigger.creature_combat_event.as_ref().unwrap().dealer,
            original
        );
        pass_pair(&mut game);
        act(&mut game, Action::ChooseOptional { accept: false });
        assert_eq!(game.object_ref(watched), original);

        pass_pair(&mut game);
        place_triggers(&mut game);
        assert_eq!(game.state.step, Step::CombatDamage);
        pass_pair(&mut game);
        act(&mut game, Action::ChooseOptional { accept: true });
        assert_eq!(game.state.players[player.other().index()].life, 16);
        assert_ne!(game.object_ref(watched), original);
        assert_eq!(game.object_view(watched).controller, player.other());
        assert_eq!(
            game.observe(player).delayed_target_events.unwrap()[0].subject,
            original
        );
        game.finish_cleanup();
        assert!(game.observe(player).delayed_target_events.is_none());
    }
}

#[test]
fn one_dealer_to_multiple_recipients_yields_one_delayed_packet_and_never_rebinds() {
    let _rules = fixture_rules::activate(&[(Card::HillGiant, &DELAYED_PROVIDER)]);
    let player = Player::P0;
    let mut game = scenario(player);
    let watched = put(&mut game, player, "Grizzly Bears", Zone::Battlefield);
    let blocker = put(&mut game, player.other(), "Bear Cub", Zone::Battlefield);
    let source = schedule(&mut game, player, watched);
    game.move_object(source, Zone::Graveyard);
    let original = game.object_ref(watched);
    game.change_controller(original, player.other());
    let event = DamageEvent {
        preventable: true,
        source: game.damage_source(watched),
        target: Target::Card(blocker),
        amount: 1,
        combat: true,
    };
    let mut newer = game.clone();
    newer.move_object(watched, Zone::Exile);
    newer.move_object(watched, Zone::Battlefield);
    newer.apply_damage_batch(&[DamageEvent {
        source: newer.damage_source(watched),
        ..event
    }]);
    newer.advance_replacements(false);
    assert!(newer.state.pending_triggers.is_empty() && newer.state.stack.is_empty());

    let mut noncombat = game.clone();
    noncombat.apply_damage_batch(&[DamageEvent {
        combat: false,
        ..event
    }]);
    noncombat.advance_replacements(false);
    assert!(noncombat.state.pending_triggers.is_empty() && noncombat.state.stack.is_empty());

    game.apply_damage_batch(&[
        event,
        DamageEvent {
            target: Target::Player(player),
            ..event
        },
    ]);
    game.advance_replacements(false);
    let packets: Vec<_> = game
        .state
        .pending_triggers
        .iter()
        .chain(game.state.stack.iter())
        .collect();
    assert_eq!(packets.len(), 1);
    assert_eq!(packets[0].controller, player);
    assert_eq!(
        packets[0].creature_combat_event.as_deref(),
        Some(&CreatureCombatEventView {
            dealer: original,
            amount: 2,
        })
    );
    assert!(packets[0].targets.is_empty());
}

#[test]
fn class_triggers_for_each_creature_that_hits_a_player_and_blink_ends_its_attack() {
    let class = named("Doubling Season");
    let _rules = fixture_rules::activate(&[(class, &PLAYER_DAMAGE_PROVIDER)]);
    let player = Player::P1;
    let mut game = scenario(player);
    let source = spawn(&mut game, player, class, Zone::Battlefield);
    let first = put(&mut game, player, "Grizzly Bears", Zone::Battlefield);
    let second = put(&mut game, player, "Bear Cub", Zone::Battlefield);
    game.state.objects[first as usize].gained_keywords = Keywords::DOUBLE_STRIKE;
    game.state.objects[second as usize].gained_keywords = Keywords::FIRST_STRIKE;
    let first_before = game.object_ref(first);
    let second_before = game.object_ref(second);

    let mut level_two = game.clone();
    level_two.state.objects[source as usize].class_level = 2;
    first_strike(&mut level_two, &[first, second]);
    assert!(level_two.state.stack.is_empty());
    assert!(level_two.state.pending_triggers.is_empty());

    game.state.objects[source as usize].class_level = 3;
    let mut permanent_hit = game.clone();
    let walker = put(
        &mut permanent_hit,
        player.other(),
        "Kaito, Cunning Infiltrator",
        Zone::Battlefield,
    );
    permanent_hit.add_counter(walker, CounterKind::Loyalty, 3);
    permanent_hit.apply_damage_batch(&[DamageEvent {
        preventable: true,
        source: permanent_hit.damage_source(first),
        target: Target::Card(walker),
        amount: 1,
        combat: true,
    }]);
    permanent_hit.advance_replacements(false);
    assert!(
        permanent_hit.state.pending_triggers.is_empty() && permanent_hit.state.stack.is_empty()
    );

    first_strike(&mut game, &[first, second]);
    place_triggers(&mut game);
    assert_eq!(game.state.stack.len(), 2);
    assert!(
        game.state
            .stack
            .iter()
            .all(|trigger| trigger.targets.is_empty())
    );
    game.move_object(source, Zone::Graveyard);
    for _ in 0..2 {
        let dealer = game
            .state
            .stack
            .last()
            .unwrap()
            .damage_event
            .unwrap()
            .dealer
            .unwrap();
        pass_pair(&mut game);
        let action = Action::ChooseOptional {
            accept: dealer == first_before,
        };
        let index = game
            .legal_actions()
            .iter()
            .position(|a| *a == action)
            .unwrap();
        let mut indexed = game.clone();
        indexed.apply_index(player, game.revision(), index).unwrap();
        act(&mut game, action);
        assert_eq!(game.snapshot(), indexed.snapshot());
    }
    assert_ne!(game.object_ref(first), first_before);
    assert_eq!(game.object_ref(second), second_before);
    assert!(
        !game
            .state
            .attackers
            .iter()
            .any(|attack| attack.card == first)
    );
    pass_pair(&mut game);
    assert_eq!(game.state.step, Step::CombatDamage);
    assert_eq!(game.state.players[player.other().index()].life, 16);
    assert!(game.state.stack.is_empty());
}
