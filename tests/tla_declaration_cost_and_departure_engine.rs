mod support;

use rustmage as _;
use rustmage_core::cards::authored::{CardExtras, CardRules, one};
use rustmage_core::engine::characteristic_layers::AnimationTypeMode;
use rustmage_core::{cards::*, engine::*};
use support::*;

const ANY_OBJECT: SacrificeFilter = SacrificeFilter {
    artifact_enchantment_or_token: false,
    keywords: Keywords::NONE,
    source_only: false,
    kinds: 0,
    nonland: false,
    other: false,
    tokens: None,
    subtype: None,
};
const LIMIT: CardRules = CardRules {
    effect: Effect::Creature(3, 3),
    static_groups: &[StaticAbilityGroup {
        line: 0,
        first_layer: AbilityLayer::Rule,
        fragments: &[StaticAbilityFragment::ControlledCreaturesWithPowerAtLeastMaxOneBlocker(4)],
    }],
    ..CardRules::EMPTY
};
const DEATH: CardRules = CardRules {
    effect: Effect::Creature(3, 3),
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::BattlefieldDeparture(DepartureRule {
            kinds: 1,
            excluded_kinds: 2,
            relation: CastRelation::You,
            subject: DepartureSubject::Other,
            destinations: &[Zone::Graveyard],
            counters: DepartureCounters::Any,
            ..DepartureRule::ANY
        }),
        targets: TargetPlan::NONE,
        program: &[Instruction::PutEventCounters {
            subject: Subject::Source,
        }],
    }],
    ..CardRules::EMPTY
};
const TARGET_EMPTY: CardRules = CardRules {
    effect: Effect::Spell(
        one(TargetRule::Filtered {
            base: &TargetRule::Creature,
            predicate: &TargetPredicate::NoCounters,
        }),
        &[Instruction::Destroy],
    ),
    ..CardRules::EMPTY
};
const REDIRECT: CardRules = CardRules {
    effect: Effect::Spell(
        one(TargetRule::SingleTargetStackObject),
        &[Instruction::ChangeSingleStackTarget],
    ),
    extras: &CardExtras {
        additional_cost: Some(AdditionalCost {
            kind: EffectChoiceKind::PayLife(5),
            filter: ANY_OBJECT,
        }),
        additional_mana_alternative: Some(ManaCost {
            colored: [0; 6],
            generic: 2,
        }),
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};

#[test]
fn a_live_power_limit_constrains_declarations_and_keeps_existing_blocks() {
    let _rules = fixture_rules::activate(&[(Card::HillGiant, &LIMIT)]);
    let player = Player::P0;
    for (power, menace) in [(4, false), (4, true), (3, false)] {
        let mut game = scenario(player);
        put(&mut game, player, "Hill Giant", Zone::Battlefield);
        let attacker = put(&mut game, player, "Grizzly Bears", Zone::Battlefield);
        game.add_counter(attacker, CounterKind::PlusOne, power - 2);
        if menace {
            game.state.objects[attacker as usize].gained_keywords = Keywords::MENACE;
        }
        let first = put(&mut game, player.other(), "Bear Cub", Zone::Battlefield);
        let second = put(&mut game, player.other(), "Bear Cub", Zone::Battlefield);
        game.state.step = Step::Attackers;
        game.state.pending = Pending::Attackers;
        act(&mut game, Action::Attack { card: attacker });
        act(&mut game, Action::FinishAttackers);
        while game.state.pending != Pending::Blockers {
            act(&mut game, Action::Pass);
        }
        let first_block = Action::Block {
            blocker: first,
            attacker,
        };
        let second_block = Action::Block {
            blocker: second,
            attacker,
        };
        if menace {
            reject(&mut game, first_block);
            reject(&mut game, second_block);
            continue;
        }
        act(&mut game, first_block);
        if power == 4 {
            reject(&mut game, second_block);
        } else {
            act(&mut game, second_block);
        }
        act(&mut game, Action::FinishBlockers);
        game.add_counter(attacker, CounterKind::PlusOne, 1);
        assert_eq!(game.state.blocks.len(), if power == 4 { 1 } else { 2 });
    }
}

#[test]
fn a_nonland_death_copies_all_counters_and_an_empty_counter_target_is_live() {
    let shock = Card::from_name("Shock").unwrap();
    let _rules = fixture_rules::activate(&[(Card::HillGiant, &DEATH), (shock, &TARGET_EMPTY)]);
    let player = Player::P0;
    let mut game = scenario(player);
    let source = put(&mut game, player, "Hill Giant", Zone::Battlefield);
    let land = put(&mut game, player, "Forest", Zone::Battlefield);
    game.change_object_form(
        game.object_ref(land),
        AnimationSpec {
            artifact: false,
            creature_types: AnimationCreatureTypes::Unchanged,
            base: Some((2, 2)),
            keywords: Keywords::NONE,
            duration: AnimationDuration::EndOfTurn,
        },
        AnimationTypeMode::AddCreature,
    );
    game.add_counter(land, CounterKind::PlusOne, 5);
    game.move_object(land, Zone::Graveyard);
    game.prepare_triggers();
    assert!(game.state.stack.is_empty());
    let creature = put(&mut game, player, "Bear Cub", Zone::Battlefield);
    game.add_counter(creature, CounterKind::PlusOne, 2);
    game.add_counter(creature, CounterKind::Lore, 3);
    game.move_object(creature, Zone::Graveyard);
    game.prepare_triggers();
    settle(&mut game);
    assert_eq!(game.counter_count(source, CounterKind::PlusOne), 2);
    assert_eq!(game.counter_count(source, CounterKind::Lore), 3);

    let spell = put(&mut game, player, "Shock", Zone::Hand);
    let fresh = put(&mut game, player.other(), "Bear Cub", Zone::Battlefield);
    act(&mut game, Action::AnnounceCast { card: spell });
    reject(
        &mut game,
        Action::ChooseTarget {
            target: Target::Card(source),
        },
    );
    act(
        &mut game,
        Action::ChooseTarget {
            target: Target::Card(fresh),
        },
    );
    pay(&mut game);
    game.add_counter(fresh, CounterKind::Lore, 1);
    settle(&mut game);
    assert_eq!(game.object_view(fresh).zone, Zone::Battlefield);
}

fn pay_redirect(game: &mut Game, caster: Player, target: CardId, life: bool) {
    for _ in 0..20 {
        if game.state.pending == Pending::Priority && game.state.stack.len() == 2 {
            return;
        }
        let additional = game.observe(caster).cast_costs.and_then(|costs| {
            costs
                .clauses
                .iter()
                .position(|row| row.clause == CastCostClause::Additional && !row.paid)
        });
        choose(game, |action| match action {
            Action::ChooseSpellCost { cost } => *cost == u16::from(!life),
            Action::ChooseTarget {
                target: Target::Spell(id),
            } => *id == target,
            Action::ChooseCastCost { clause } => {
                additional.is_none_or(|index| usize::from(*clause) == index)
            }
            Action::ChooseAdditionalLife => life,
            Action::BeginOrderedCosts | Action::FinishTargets | Action::PayCosts { .. } => true,
            _ => false,
        });
    }
    panic!("redirect payment did not finish");
}

#[test]
fn life_and_mana_are_exclusive_additional_costs_for_retargeting() {
    let shock = Card::from_name("Shock").unwrap();
    let _rules = fixture_rules::activate(&[(shock, &REDIRECT)]);
    let player = Player::P0;
    for life in [true, false] {
        let mut game = scenario(player);
        let shot = put(&mut game, player.other(), "Lightning Strike", Zone::Hand);
        game.state.priority = player.other();
        cast(&mut game, shot, Some(Target::Player(player)));
        if game.actor() != Some(player) {
            act(&mut game, Action::Pass);
        }
        let spell = put(&mut game, player, "Shock", Zone::Hand);
        let hand = put(&mut game, player, "Forest", Zone::Hand);
        let action = Action::AnnounceCast { card: spell };
        if life {
            game.state.players[player.index()].mana = [0, 0, 0, 1, 0, 0];
            let mut short = game.clone();
            short.state.players[player.index()].life = 4;
            reject(&mut short, action.clone());
            short.state.players[player.index()].life = 5;
            assert!(short.legal_actions().contains(&action));
        } else {
            game.state.players[player.index()].life = 4;
            game.state.players[player.index()].mana = [0, 0, 0, 3, 0, 0];
        }
        let original_life = game.state.players[player.index()].life;
        act(&mut game, action);
        pay_redirect(&mut game, player, shot, life);
        assert_eq!(game.object_view(hand).zone, Zone::Hand);
        assert_eq!(
            game.state.players[player.index()].life,
            original_life - if life { 5 } else { 0 }
        );
        assert_eq!(game.state.players[player.index()].mana, [0; 6]);

        act(&mut game, Action::Pass);
        act(&mut game, Action::Pass);
        act(
            &mut game,
            Action::SetStackTarget {
                index: 0,
                target: Target::Player(player.other()),
            },
        );
        act(&mut game, Action::FinishStackTargets);
        settle(&mut game);
        assert_eq!(game.state.players[player.other().index()].life, 17);
    }
}

#[test]
fn an_attached_creature_death_can_mill_its_exact_predeparture_power() {
    const AURA: CardRules = CardRules {
        effect: Effect::Permanent,
        triggers: &[TriggeredAbility {
            condition: TriggerCondition::AttachedCreatureDies,
            targets: TargetPlan::NONE,
            program: &[Instruction::Mill(
                PlayerSet::Controller,
                MillAmount::Amount(AmountValue::DeathEventPower),
            )],
        }],
        ..CardRules::EMPTY
    };
    let aura = Card::from_name("Witness Protection").unwrap();
    let _rules = fixture_rules::activate(&[(aura, &AURA)]);
    let player = Player::P1;
    let mut game = scenario(player);
    let source = put(&mut game, player, "Witness Protection", Zone::Battlefield);
    let creature = put(&mut game, player, "Bear Cub", Zone::Battlefield);
    game.state.objects[source as usize].attached_to = Some(game.object_ref(creature));
    game.add_counter(creature, CounterKind::PlusOne, 3);
    let before = game.state.players[player.index()].library.len();
    game.move_object(creature, Zone::Graveyard);
    game.prepare_triggers();
    settle(&mut game);
    assert_eq!(game.state.players[player.index()].library.len(), before - 5);
}
