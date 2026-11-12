//! Expend is based on payment order and actual mana paid for spells.
mod support;

use rustmage as _;
use rustmage_core::cards::authored::{CardExtras, CardRules};
use rustmage_core::{cards::*, engine::*};
use support::*;

const EXPEND_FOUR: TriggeredAbility = TriggeredAbility {
    condition: TriggerCondition::Expend(4),
    targets: TargetPlan::NONE,
    program: &[Instruction::GainLife(PlayerSet::Controller, 1)],
};
const WATCHER: CardRules = CardRules {
    effect: Effect::Creature(2, 2),
    triggers: &[EXPEND_FOUR],
    ..CardRules::EMPTY
};
const TWO_THRESHOLDS: CardRules = CardRules {
    triggers: &[
        EXPEND_FOUR,
        TriggeredAbility {
            condition: TriggerCondition::Expend(8),
            program: &[Instruction::GainLife(PlayerSet::Controller, 2)],
            ..EXPEND_FOUR
        },
    ],
    ..WATCHER
};
const CONVOKE_SACRIFICE: CardRules = CardRules {
    effect: Effect::Creature(3, 3),
    extras: &CardExtras {
        has_convoke: true,
        additional_cost: Some(AdditionalCost {
            kind: EffectChoiceKind::Sacrifice,
            filter: SacrificeFilter {
                keywords: Keywords::NONE,
                source_only: false,
                kinds: 1,
                nonland: false,
                other: false,
                tokens: None,
                artifact_enchantment_or_token: false,
                subtype: None,
            },
        }),
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
const ENCHANTMENT_WATCHER: CardRules = CardRules {
    effect: Effect::Permanent,
    ..WATCHER
};

fn named(name: &str) -> Card {
    Card::from_name(name).unwrap()
}

fn settle_triggers(game: &mut Game) {
    for _ in 0..32 {
        if game.state.pending == Pending::Priority && game.state.stack.is_empty() {
            return;
        }
        choose(game, |action| {
            matches!(action, Action::Pass | Action::PutTrigger { .. })
        });
    }
    panic!("trigger resolution exceeded scenario limit");
}

fn select_cost(game: &mut Game, cost: CastCostClause) {
    let clause = game
        .observe(game.actor().unwrap())
        .cast_costs
        .unwrap()
        .clauses
        .iter()
        .position(|row| row.clause == cost && !row.paid)
        .unwrap() as u16;
    act(game, Action::ChooseCastCost { clause });
}

fn mana_spent(game: &Game, player: Player) -> u64 {
    game.state.players[player.index()].spell_mana_spent_this_turn
}

#[test]
fn expend_crosses_each_threshold_once_using_spell_payments_from_both_turns() {
    let _rules = fixture_rules::activate(&[(named("Bear Cub"), &TWO_THRESHOLDS)]);
    for player in [Player::P0, Player::P1] {
        let mut game = scenario(player);
        let opponent = player.other();
        let early = put(&mut game, player, "Quick Study", Zone::Hand);
        cast(&mut game, early, None);
        settle(&mut game);
        assert_eq!(mana_spent(&game, player), 3);

        put(&mut game, player, "Bear Cub", Zone::Battlefield);
        put(&mut game, opponent, "Bear Cub", Zone::Battlefield);
        let tome = put(&mut game, player, "Mazemind Tome", Zone::Battlefield);
        choose(
            &mut game,
            |action| matches!(action, Action::Activate { card, ability: 1, .. } if *card == tome),
        );
        settle(&mut game);
        assert_eq!(mana_spent(&game, player), 3);
        assert_eq!(game.state.players[player.index()].life, 20);

        let boar = put(&mut game, player, "Quaketusk Boar", Zone::Hand);
        cast(&mut game, boar, None);
        settle_triggers(&mut game);
        assert_eq!(mana_spent(&game, player), 8);
        assert_eq!(game.state.players[player.index()].life, 23);
        assert_eq!(game.state.players[opponent.index()].life, 20);

        let extra = put(&mut game, player, "Giant Growth", Zone::Hand);
        cast(&mut game, extra, Some(Target::Card(boar)));
        settle(&mut game);
        assert_eq!(game.state.players[player.index()].life, 23);

        game.finish_cleanup();
        assert_eq!(game.state.active, opponent);
        assert_eq!(mana_spent(&game, player), 0);
        assert_eq!(mana_spent(&game, opponent), 0);
        // Spend real mana during the other player's turn.
        game.state.players[player.index()].mana = [0, 0, 0, 1, 0, 3];
        act(&mut game, Action::Pass);
        let lash = put(&mut game, player, "Flame Lash", Zone::Hand);
        cast(&mut game, lash, Some(Target::Player(opponent)));
        settle(&mut game);
        assert_eq!(game.state.players[player.index()].life, 24);
        assert_eq!(game.state.players[opponent.index()].life, 16);
    }
}

#[test]
fn expend_samples_observers_at_payment_and_convoke_is_not_spent_mana() {
    let _rules = fixture_rules::activate(&[
        (named("Bear Cub"), &WATCHER),
        (named("Hill Giant"), &CONVOKE_SACRIFICE),
    ]);
    let player = Player::P0;
    let mut root = scenario(player);
    let observer = put(&mut root, player, "Bear Cub", Zone::Battlefield);
    let convoker = put(&mut root, player, "Llanowar Elves", Zone::Battlefield);
    let spell = put(&mut root, player, "Hill Giant", Zone::Hand);
    root.state.players[player.index()].mana = [0, 0, 0, 1, 0, 3];
    act(&mut root, Action::AnnounceCast { card: spell });
    act(&mut root, Action::BeginConvoke);
    let start = root.snapshot();
    let sacrifice = Action::ChooseAdditionalCost {
        object: root.object_ref(observer),
    };
    let mut mana_first = root.clone();
    select_cost(&mut mana_first, CastCostClause::Mana);
    pay(&mut mana_first);
    assert_eq!(mana_spent(&mana_first, player), 4);
    select_cost(&mut mana_first, CastCostClause::Additional);
    let paid = mana_first.snapshot();
    let revision = mana_first.revision();
    let index = mana_first
        .legal_actions()
        .iter()
        .position(|action| *action == sacrifice)
        .unwrap();
    let mut raw = mana_first.clone();
    let mut indexed = mana_first.clone();
    drop(mana_first);
    act(&mut raw, sacrifice.clone());
    indexed.apply_index(player, revision, index).unwrap();
    assert_eq!(raw.snapshot(), indexed.snapshot());
    settle(&mut raw);
    assert_eq!(raw.object_view(observer).zone, Zone::Graveyard);
    assert_eq!(raw.state.players[player.index()].life, 21);
    indexed.restore(&paid).unwrap();
    act(&mut indexed, sacrifice.clone());
    settle(&mut indexed);
    assert_eq!(raw.snapshot(), indexed.snapshot());

    select_cost(&mut root, CastCostClause::Additional);
    act(&mut root, sacrifice);
    select_cost(&mut root, CastCostClause::Mana);
    pay(&mut root);
    settle(&mut root);
    assert_eq!(mana_spent(&root, player), 4);
    assert_eq!(root.state.players[player.index()].life, 20);

    root.restore(&start).unwrap();
    select_cost(&mut root, CastCostClause::Mana);
    let object = root.object_ref(convoker);
    act(
        &mut root,
        Action::ChooseConvoker {
            object,
            contribution: ConvokeContribution::Generic,
        },
    );
    pay(&mut root);
    assert_eq!(mana_spent(&root, player), 3);
    select_cost(&mut root, CastCostClause::Additional);
    let object = root.object_ref(observer);
    act(&mut root, Action::ChooseAdditionalCost { object });
    settle(&mut root);
    assert_eq!(root.state.players[player.index()].life, 20);
}

#[test]
fn failed_cast_restores_payment_threshold_and_unstacked_expend_trigger() {
    let _rules = fixture_rules::activate(&[(named("Impact Tremors"), &ENCHANTMENT_WATCHER)]);
    let player = Player::P0;
    let mut game = scenario(player);
    let silver = put(
        &mut game,
        player,
        "Silverquill, the Disputant",
        Zone::Battlefield,
    );
    game.state.objects[silver as usize].growth.power = -1;
    let enemy = put(&mut game, player.other(), "Bear Cub", Zone::Battlefield);
    let aura = put(
        &mut game,
        player.other(),
        "Witness Protection",
        Zone::Battlefield,
    );
    game.state.objects[aura as usize].attached_to = Some(game.object_ref(enemy));
    let prison = put(&mut game, player, "Banishing Light", Zone::Hand);
    cast(&mut game, prison, None);
    for _ in 0..8 {
        if game.state.pending != Pending::Priority {
            break;
        }
        act(&mut game, Action::Pass);
    }
    act(
        &mut game,
        Action::ChooseTarget {
            target: Target::Card(aura),
        },
    );
    settle_triggers(&mut game);
    assert_eq!(game.object_view(aura).zone, Zone::Exile);
    assert_eq!(mana_spent(&game, player), 3);

    put(&mut game, player, "Impact Tremors", Zone::Battlefield);
    let spell = put(&mut game, player, "Candy Grapple", Zone::Hand);
    game.state.players[player.index()].mana = [0, 0, 1, 1, 0, 1];
    act(&mut game, Action::AnnounceCast { card: spell });
    let sacrifice = Some(game.object_ref(prison));
    act(&mut game, Action::ChooseBargain { sacrifice });
    act(&mut game, Action::ChooseCasualty { pay: true });
    act(
        &mut game,
        Action::ChooseTarget {
            target: Target::Card(enemy),
        },
    );
    act(
        &mut game,
        Action::PayCosts {
            payment: [0, 0, 1, 0, 0, 1],
        },
    );
    assert_eq!(mana_spent(&game, player), 5);
    assert_eq!(game.state.pending_triggers.len(), 1);

    select_cost(&mut game, CastCostClause::Additional);
    // The opponent can attach their returning Aura to the only casualty payer.
    // If they do, the paid spell and its provisional threshold crossing unwind.
    let object = game.object_ref(silver);
    act(&mut game, Action::ChooseAuraAttachment { object });
    assert_eq!(game.object_view(spell).zone, Zone::Hand);
    assert_eq!(game.object_view(prison).zone, Zone::Battlefield);
    assert_eq!(mana_spent(&game, player), 3);
    assert!(game.state.pending_triggers.is_empty());
    assert_eq!(game.state.players[player.index()].life, 20);

    let shock = put(&mut game, player, "Shock", Zone::Hand);
    act(&mut game, Action::AnnounceCast { card: shock });
    act(&mut game, Action::ChooseCasualty { pay: false });
    act(
        &mut game,
        Action::ChooseTarget {
            target: Target::Player(player.other()),
        },
    );
    pay(&mut game);
    settle(&mut game);
    assert_eq!(mana_spent(&game, player), 4);
    assert_eq!(game.state.players[player.index()].life, 21);
}

#[test]
fn a_direct_cast_acquires_targets_before_spending_its_expend_mana() {
    const BOTH: CardRules = CardRules {
        triggers: &[
            TriggeredAbility {
                condition: TriggerCondition::FirstTargetedByController,
                ..EXPEND_FOUR
            },
            TriggeredAbility {
                condition: TriggerCondition::Expend(2),
                ..EXPEND_FOUR
            },
        ],
        ..WATCHER
    };
    let _rules = fixture_rules::activate(&[(Card::HillGiant, &BOTH)]);
    let player = Player::P0;
    let mut game = scenario(player);
    let creature = put(&mut game, player, "Hill Giant", Zone::Battlefield);
    let spell = put(&mut game, player, "Lightning Strike", Zone::Hand);
    choose(&mut game, |action| {
        matches!(action, Action::Cast {
            card, target: Some(Target::Card(target)), ..
        } if *card == spell && *target == creature)
    });

    let source = game.object_ref(creature);
    let mut triggers = game
        .state
        .pending_triggers
        .iter()
        .chain(&game.state.ordering_triggers)
        .chain(&game.state.stack)
        .filter_map(|item| {
            if let StackKind::TriggeredAbility { ability } = item.kind
                && item
                    .source
                    .is_some_and(|observer| observer.object == source)
            {
                return Some((item.id, ability));
            }
            None
        })
        .collect::<Vec<_>>();
    triggers.sort_unstable();
    assert_eq!(
        triggers
            .into_iter()
            .map(|(_, ability)| ability)
            .collect::<Vec<_>>(),
        [0, 1]
    );
    assert_eq!(mana_spent(&game, player), 2);
}
