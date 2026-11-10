//! An additional untargeted choice owns one exact live or last-known creature.
mod support;
use rustmage as _;
use rustmage_core::cards::authored::{CardExtras, CardRules, one};
use rustmage_core::{cards::*, engine::*};
use support::*;

const CHOICE: CardRules = CardRules {
    effect: Effect::Spell(
        one(TargetRule::Creature),
        &[Instruction::DamageScaled(
            AmountValue::ChosenCostCreaturePower,
        )],
    ),
    extras: &CardExtras {
        additional_cost: Some(AdditionalCost {
            kind: EffectChoiceKind::ChooseControlledOrWarpedCreature,
            filter: SacrificeFilter {
                keywords: Keywords::NONE,
                source_only: false,
                kinds: 0,
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
const RED: Mana = [0, 0, 0, 1, 0, 0];
const FUNDER: CardRules = CardRules {
    effect: Effect::Creature(2, 2),
    activated: &[ActivatedAbility {
        cost: ActivationCost {
            sacrifice: true,
            ..ActivationCost::ZERO
        },
        targets: TargetPlan::NONE,
        program: &[Instruction::AddMana(ManaProduction::Fixed(RED))],
        guard: ActivationGuard::Always,
        origin: ActivationOrigin::Battlefield,
        exhaust: false,
        cycling: false,
        sorcery: false,
        once_per_turn: false,
    }],
    ..CardRules::EMPTY
};
const EXILE_POWER: DynamicStats = DynamicStats {
    power: Some(StatExpr {
        value: StatValue::Count(CountFilter {
            zone: CountZone::Exile,
            relation: 0,
            kind: 0,
            exclude: 0,
            subtype: None,
            basic: false,
            other: false,
            cards_only: true,
        }),
        multiplier: 1,
        offset: 0,
    }),
    toughness: None,
};
const WARPED: CardRules = CardRules {
    effect: Effect::Creature(0, 4),
    static_groups: &[StaticAbilityGroup {
        line: 0,
        first_layer: AbilityLayer::Cda,
        fragments: &[StaticAbilityFragment::DynamicStats(0, EXILE_POWER)],
    }],
    extras: &CardExtras {
        dynamic_stats: Some(EXILE_POWER),
        warp: Some(&WarpSpec::fixed(ManaCost {
            colored: [0, 1, 0, 0, 0, 0],
            generic: 0,
        })),
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};

fn clause(game: &mut Game, kind: CastCostClause) {
    let index = game
        .state
        .announcing
        .as_ref()
        .unwrap()
        .cast_costs
        .as_ref()
        .unwrap()
        .clauses
        .iter()
        .position(|row| row.clause == kind && !row.paid)
        .unwrap();
    act(
        game,
        Action::ChooseCastCost {
            clause: index as u16,
        },
    );
}

fn choose_cost(game: &Game, card: CardId) -> Action {
    Action::ChooseAdditionalCost {
        object: game.object_ref(card),
    }
}

fn target(game: &mut Game, card: CardId) {
    act(
        game,
        Action::ChooseTarget {
            target: Target::Card(card),
        },
    );
}

#[test]
fn choosing_after_mana_is_non_targeting_and_tracks_live_or_departed_power() {
    let bear = Card::from_name("Bear Cub").unwrap();
    let _rules = fixture_rules::activate(&[(Card::Shock, &CHOICE), (bear, &FUNDER)]);
    let player = Player::P0;
    let mut game = scenario(player);
    let funder = put(&mut game, player, "Bear Cub", Zone::Battlefield);
    let victim = put(&mut game, player.other(), "Hill Giant", Zone::Battlefield);
    game.state.objects[victim as usize].counters.plus_one = 4;
    let spell = put(&mut game, player, "Shock", Zone::Hand);
    game.state.players[player.index()].mana = [0; 6];
    reject(&mut game, Action::AnnounceCast { card: spell });

    let creature = put(&mut game, player, "Goblin Piker", Zone::Battlefield);
    game.state.objects[creature as usize].gained_keywords = Keywords::PROTECTION_RED;
    let reference = game.object_ref(creature);
    act(&mut game, Action::AnnounceCast { card: spell });
    reject(
        &mut game,
        Action::ChooseTarget {
            target: Target::Card(creature),
        },
    );
    target(&mut game, victim);
    act(
        &mut game,
        Action::ActivateMana {
            card: funder,
            ability: 0,
            payment: [0; 6],
            counter_payment: Counters::default(),
            production: RED,
        },
    );
    pay(&mut game);
    clause(&mut game, CastCostClause::Additional);
    let choice = choose_cost(&game, creature);
    let opponent_choice = choose_cost(&game, victim);
    reject(&mut game, opponent_choice);
    assert!(game.observe(player.other()).additional_cost.is_none());
    let before = game.snapshot();
    let index = game
        .legal_actions()
        .iter()
        .position(|action| *action == choice)
        .unwrap();
    let mut live = game.clone();
    let mut departed = game.clone();
    act(&mut live, choice);
    departed
        .apply_index(player, departed.revision(), index)
        .unwrap();
    assert_eq!(live.snapshot(), departed.snapshot());
    assert_eq!(game.snapshot(), before);
    assert_eq!(live.object_ref(creature), reference);
    assert_eq!(live.object_view(creature).zone, Zone::Battlefield);
    assert_eq!(live.state.stack.last().unwrap().targets.len(), 1);

    live.state.objects[creature as usize].counters.plus_one = 2;
    settle(&mut live);
    assert_eq!(live.object_view(victim).damage, 4);

    departed.state.objects[creature as usize].counters.plus_one = 3;
    departed.move_object(creature, Zone::Graveyard);
    departed.move_object(creature, Zone::Battlefield);
    assert_ne!(departed.object_ref(creature), reference);
    settle(&mut departed);
    assert_eq!(departed.object_view(victim).damage, 5);
}

#[test]
fn only_the_exact_warped_exile_is_eligible_and_its_last_power_is_kept() {
    let bear = Card::from_name("Bear Cub").unwrap();
    let _rules = fixture_rules::activate(&[(Card::Shock, &CHOICE), (bear, &WARPED)]);
    let player = Player::P1;
    let mut game = scenario(player);
    let warped = put(&mut game, player, "Bear Cub", Zone::Hand);
    let victim = put(&mut game, player.other(), "Hill Giant", Zone::Battlefield);
    game.state.objects[victim as usize].counters.plus_one = 4;
    let ordinary = put(&mut game, player, "Bear Cub", Zone::Exile);
    let spell = put(&mut game, player, "Shock", Zone::Hand);
    reject(&mut game, Action::AnnounceCast { card: spell });

    act(&mut game, Action::AnnounceWarp { card: warped });
    pay(&mut game);
    settle(&mut game);
    for _ in 0..40 {
        if game.object_view(warped).zone == Zone::Exile {
            break;
        }
        choose(&mut game, |action| {
            matches!(
                action,
                Action::Pass
                    | Action::FinishAttackers
                    | Action::FinishBlockers
                    | Action::PutTrigger { .. }
            )
        });
    }
    let exact = game.object_ref(warped);
    assert_eq!(game.observe(player).warped_cards[0].object, exact);
    assert_eq!(game.state.turn, 3);
    game.state.players[player.index()].mana = RED;
    act(&mut game, Action::AnnounceCast { card: spell });
    target(&mut game, victim);
    pay(&mut game);
    clause(&mut game, CastCostClause::Additional);
    let unqualified = choose_cost(&game, ordinary);
    reject(&mut game, unqualified);
    let choice = choose_cost(&game, warped);
    act(&mut game, choice);
    assert_eq!(game.object_ref(warped), exact);
    assert_eq!(game.observe(player).warped_cards[0].object, exact);
    assert_eq!(stats(&game, warped).0, 2);

    put(&mut game, player, "Island", Zone::Exile);
    let mut live = game.clone();
    settle(&mut live);
    assert_eq!(live.object_view(victim).damage, 3);
    game.move_object(warped, Zone::Hand);
    game.move_object(warped, Zone::Exile);
    put(&mut game, player, "Forest", Zone::Exile);
    assert!(game.observe(player).warped_cards.is_empty());
    settle(&mut game);
    assert_eq!(game.object_view(victim).damage, 3);
}
