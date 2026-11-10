//! Common rule fixtures for the Spider-Man engine checkpoint.
mod support;

use rustmage as _;
use rustmage_core::cards::authored::CardRules;
use rustmage_core::cards::*;
use rustmage_core::engine::*;
use support::*;

const MODIFIED_TARGET: CardRules = CardRules {
    effect: Effect::Spell(
        TargetPlan::Single(TargetSpec::one(TargetRule::Filtered {
            base: &TargetRule::Creature,
            predicate: &TargetPredicate::Modified,
        })),
        &[Instruction::PutCounters {
            subject: Subject::Targets,
            kind: CounterKind::PlusOne,
            amount: 1,
        }],
    ),
    ..CardRules::EMPTY
};

const COUNT_MODIFIED: CardRules = CardRules {
    effect: Effect::Spell(
        TargetPlan::NONE,
        &[Instruction::GainLifeScaled(
            PlayerSet::Controller,
            AmountValue::Stat(StatValue::ControlledModifiedCreatures),
        )],
    ),
    ..CardRules::EMPTY
};

const MODIFIED_DEPARTURE: CardRules = CardRules {
    effect: Effect::Creature(3, 3),
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::BattlefieldDeparture(DepartureRule {
            kinds: 1,
            relation: CastRelation::You,
            modified: true,
            ..DepartureRule::ANY
        }),
        targets: TargetPlan::NONE,
        program: &[Instruction::GainLife(PlayerSet::Controller, 1)],
    }],
    ..CardRules::EMPTY
};

const UNCOUNTERABLE: CardRules = CardRules {
    effect: Effect::Creature(2, 2),
    static_groups: &[StaticAbilityGroup {
        line: 0,
        first_layer: AbilityLayer::Rule,
        fragments: &[StaticAbilityFragment::StackItemsCannotBeCountered],
    }],
    ..CardRules::EMPTY
};

const COUNTER_STACK: CardRules = CardRules {
    effect: Effect::Spell(
        TargetPlan::Single(TargetSpec::one(TargetRule::Union(&[
            TargetRule::Spell,
            TargetRule::TriggeredAbility { relation: -1 },
        ]))),
        &[Instruction::Counter],
    ),
    ..CardRules::EMPTY
};

fn card(name: &str) -> Card {
    Card::from_name(name).unwrap()
}

fn attach(game: &mut Game, owner: Player, name: &str, recipient: CardId) -> CardId {
    let attachment = put(game, owner, name, Zone::Battlefield);
    game.state.objects[attachment as usize].attached_to = Some(game.object_ref(recipient));
    attachment
}

#[test]
fn modified_choices_count_and_departures_use_the_creature_controller_and_actual_attachments() {
    let _rules = fixture_rules::activate(&[
        (card("Opt"), &MODIFIED_TARGET),
        (card("Divination"), &COUNT_MODIFIED),
        (Card::HillGiant, &MODIFIED_DEPARTURE),
    ]);
    for player in [Player::P0, Player::P1] {
        let mut game = scenario(player);
        let observer = put(&mut game, player, "Hill Giant", Zone::Battlefield);
        let marked = put(&mut game, player, "Grizzly Bears", Zone::Battlefield);
        let equipped = put(&mut game, player, "Grizzly Bears", Zone::Battlefield);
        let enchanted = put(&mut game, player, "Grizzly Bears", Zone::Battlefield);
        let hostile_aura_only = put(&mut game, player, "Grizzly Bears", Zone::Battlefield);
        let opposing_creature = put(&mut game, player.other(), "Bear Cub", Zone::Battlefield);
        let noncreature = put(&mut game, player, "Forest", Zone::Battlefield);
        game.add_counter(marked, CounterKind::Charge, 2);
        game.add_counter(opposing_creature, CounterKind::Charge, 1);
        game.add_counter(noncreature, CounterKind::Charge, 1);
        attach(&mut game, player.other(), "Quick-Draw Katana", equipped);
        attach(&mut game, player, "Pacifism", enchanted);
        attach(&mut game, player.other(), "Pacifism", hostile_aura_only);

        let count = put(&mut game, player, "Divination", Zone::Hand);
        cast(&mut game, count, None);
        settle(&mut game);
        assert_eq!(game.state.players[player.index()].life, 23);

        let spell = put(&mut game, player, "Opt", Zone::Hand);
        act(&mut game, Action::AnnounceCast { card: spell });
        let choose_target = |card| Action::ChooseTarget {
            target: Target::Card(card),
        };
        for target in [marked, equipped, enchanted, opposing_creature] {
            assert!(game.legal_actions().contains(&choose_target(target)));
        }
        reject(&mut game, choose_target(hostile_aura_only));
        reject(&mut game, choose_target(noncreature));
        act(&mut game, choose_target(marked));
        pay(&mut game);
        settle(&mut game);
        assert_eq!(game.counter_count(marked, CounterKind::PlusOne), 1);

        let destroy = put(&mut game, player, "Day of Judgment", Zone::Hand);
        cast(&mut game, destroy, None);
        pass_pair(&mut game);
        for _ in 0..2 {
            choose(&mut game, |action| {
                matches!(action, Action::PutTrigger { .. })
            });
        }
        settle(&mut game);
        assert_eq!(game.object_view(observer).zone, Zone::Graveyard);
        assert_eq!(game.state.players[player.index()].life, 26);
    }
}

fn pass_pair(game: &mut Game) {
    act(game, Action::Pass);
    act(game, Action::Pass);
}

#[test]
fn both_spells_and_triggered_abilities_remain_targets_while_countering_is_prohibited() {
    let _rules = fixture_rules::activate(&[
        (card("Opt"), &COUNTER_STACK),
        (Card::HillGiant, &UNCOUNTERABLE),
    ]);
    for ability in [false, true] {
        let player = Player::P0;
        let mut game = scenario(player);
        let provider = put(&mut game, player.other(), "Hill Giant", Zone::Battlefield);
        if ability {
            let hunter = put(&mut game, player, "Helpful Hunter", Zone::Hand);
            cast(&mut game, hunter, None);
            pass_pair(&mut game);
            assert!(matches!(
                game.state.stack.last().unwrap().kind,
                StackKind::TriggeredAbility { .. }
            ));
        } else {
            let shock = put(&mut game, player, "Shock", Zone::Hand);
            cast(&mut game, shock, Some(Target::Player(player.other())));
        }
        let victim = game.state.stack.last().unwrap();
        let target = if ability {
            Target::Stack(victim.id)
        } else {
            Target::Spell(victim.source.unwrap().id)
        };
        let counter = put(&mut game, player, "Opt", Zone::Hand);
        cast(&mut game, counter, Some(target));

        let mut removed = game.clone();
        let bounce = put(&mut removed, player, "Unsummon", Zone::Hand);
        cast(&mut removed, bounce, Some(Target::Card(provider)));
        settle(&mut removed);
        assert_eq!(removed.object_view(provider).zone, Zone::Hand);
        assert_eq!(
            removed.state.players[player.index()].cards_drawn_this_turn,
            0
        );
        assert_eq!(removed.state.players[player.other().index()].life, 20);

        settle(&mut game);
        if ability {
            assert_eq!(game.state.players[player.index()].cards_drawn_this_turn, 1);
        } else {
            assert_eq!(game.state.players[player.other().index()].life, 18);
        }
    }
}
