mod support;
use rustmage as _;
use rustmage_core::cards::authored::CardRules;
use rustmage_core::{cards::*, engine::*};
use support::*;

const HALF_CREATURES: CardRules = CardRules {
    effect: Effect::Spell(
        TargetPlan::NONE,
        &[Instruction::ChooseHalf {
            kind: EffectChoiceKind::SacrificeCreaturesExceptSubtype("God"),
        }],
    ),
    ..CardRules::EMPTY
};

const UNSACRIFICABLE: CardRules = CardRules {
    effect: Effect::Creature(3, 3),
    keywords: Keywords::CANT_BE_SACRIFICED,
    ..CardRules::EMPTY
};

fn start_half_creature_choice(game: &mut Game, player: Player) {
    let spell = put(game, player, "Opt", Zone::Hand);
    cast(game, spell, None);
    act(game, Action::Pass);
    act(game, Action::Pass);
    assert_eq!(game.state.pending, Pending::EffectChoice { player });
}

#[test]
fn filtered_half_sacrifice_collects_each_players_quota_before_any_move() {
    let _rules = fixture_rules::activate(&[(Card::from_name("Opt").unwrap(), &HALF_CREATURES)]);
    let player = Player::P1;
    let mut game = scenario(player);
    let ours = (0..3)
        .map(|_| put(&mut game, player, "Bear Cub", Zone::Battlefield))
        .collect::<Vec<_>>();
    let theirs = (0..4)
        .map(|_| put(&mut game, player.other(), "Hill Giant", Zone::Battlefield))
        .collect::<Vec<_>>();
    let changeling = put(&mut game, player, "Three Tree Mascot", Zone::Battlefield);
    let land = put(&mut game, player.other(), "Forest", Zone::Battlefield);
    start_half_creature_choice(&mut game, player);

    let excluded = game.object_ref(changeling);
    reject(&mut game, Action::Sacrifice { object: excluded });
    let own_pick = game.object_ref(ours[0]);
    act(&mut game, Action::Sacrifice { object: own_pick });
    assert_eq!(game.actor(), Some(player.other()));
    assert_eq!(game.object_view(ours[0]).zone, Zone::Battlefield);
    let excluded = game.object_ref(land);
    reject(&mut game, Action::Sacrifice { object: excluded });

    let first = game.object_ref(theirs[0]);
    let second = game.object_ref(theirs[1]);
    act(&mut game, Action::Sacrifice { object: first });
    assert_eq!(game.object_view(theirs[0]).zone, Zone::Battlefield);
    act(&mut game, Action::Sacrifice { object: second });
    settle(&mut game);
    for id in [ours[0], theirs[0], theirs[1]] {
        assert_eq!(game.object_view(id).zone, Zone::Graveyard);
    }
    for id in [ours[1], ours[2], theirs[2], theirs[3], changeling, land] {
        assert_eq!(game.object_view(id).zone, Zone::Battlefield);
    }
}

#[test]
fn filtered_half_sacrifice_finishes_when_legal_choices_are_below_the_quota() {
    let _rules = fixture_rules::activate(&[
        (Card::from_name("Opt").unwrap(), &HALF_CREATURES),
        (Card::from_name("Hill Giant").unwrap(), &UNSACRIFICABLE),
    ]);
    let player = Player::P0;
    let mut game = scenario(player);
    let available = put(&mut game, player, "Bear Cub", Zone::Battlefield);
    for _ in 0..3 {
        put(&mut game, player, "Hill Giant", Zone::Battlefield);
    }
    start_half_creature_choice(&mut game, player);
    let object = game.object_ref(available);
    let choices: Vec<_> = game
        .legal_actions()
        .into_iter()
        .filter(|action| matches!(action, Action::Sacrifice { .. }))
        .collect();
    assert_eq!(choices, vec![Action::Sacrifice { object }]);
    act(&mut game, Action::Sacrifice { object });
    settle(&mut game);
    assert_eq!(game.object_view(available).zone, Zone::Graveyard);
}

#[test]
fn either_players_actual_sacrifice_qualifies_even_when_the_cards_are_exiled() {
    const OBSERVER: CardRules = CardRules {
        effect: Effect::Creature(2, 2),
        triggers: &[TriggeredAbility {
            condition: TriggerCondition::SacrificedByAny {
                filter: SacrificeFilter {
                    keywords: Keywords::NONE,
                    source_only: false,
                    kinds: 1,
                    nonland: false,
                    other: true,
                    tokens: None,
                    artifact_enchantment_or_token: false,
                    subtype: None,
                },
            },
            targets: TargetPlan::NONE,
            program: &[Instruction::PutCounters {
                subject: Subject::Source,
                kind: CounterKind::PlusOne,
                amount: 1,
            }],
        }],
        ..CardRules::EMPTY
    };
    let _rules = fixture_rules::activate(&[(Card::GrizzlyBears, &OBSERVER)]);
    let player = Player::P0;
    let mut game = scenario(player);
    let observer = put(&mut game, player, "Grizzly Bears", Zone::Battlefield);
    put(&mut game, player, "Rest in Peace", Zone::Battlefield);
    let own = put(&mut game, player, "Bear Cub", Zone::Battlefield);
    let other = put(&mut game, player.other(), "Hill Giant", Zone::Battlefield);
    let unrelated = put(&mut game, player.other(), "Hill Giant", Zone::Battlefield);
    game.move_object(unrelated, Zone::Graveyard);
    game.prepare_triggers();
    assert_eq!(game.object_view(unrelated).zone, Zone::Exile);
    assert!(game.state.stack.is_empty());

    game.sacrifice_batch(&[(player, own), (player.other(), other)]);
    game.prepare_triggers();
    assert_eq!(game.object_view(own).zone, Zone::Exile);
    assert_eq!(game.object_view(other).zone, Zone::Exile);
    assert_eq!(game.state.pending, Pending::OrderTriggers { player });
    assert_eq!(game.state.ordering_triggers.len(), 2);
    let action = Action::PutTrigger {
        id: game.state.ordering_triggers[0].id,
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
    settle(&mut game);
    assert_eq!(game.counter_count(observer, CounterKind::PlusOne), 2);

    game.sacrifice_batch(&[(player, observer)]);
    game.prepare_triggers();
    assert!(game.state.stack.is_empty());
    assert!(game.state.pending_triggers.is_empty());
}
