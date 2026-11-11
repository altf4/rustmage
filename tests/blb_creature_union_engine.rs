//! One simultaneous placement on the union of creatures that qualify.
mod support;

use rustmage as _;
use rustmage_core::cards::authored::CardRules;
use rustmage_core::{cards::*, engine::*};
use support::*;

const OTHER_OWN: CreatureFilter = CreatureFilter {
    relation: 0,
    other: true,
    subtype: None,
    keyword: Keywords::NONE,
    attacking: false,
    tokens: None,
    artifact: false,
};

const COUNTER_SOURCE: CardRules = CardRules {
    effect: Effect::Creature(2, 2),
    activated: &[ActivatedAbility {
        cost: ActivationCost::ZERO,
        guard: ActivationGuard::Always,
        targets: TargetPlan::NONE,
        program: &[Instruction::PutCreatureGroupCounters {
            filters: &[
                CreatureFilter {
                    tokens: Some(true),
                    ..OTHER_OWN
                },
                CreatureFilter {
                    subtype: Some("Rabbit"),
                    ..OTHER_OWN
                },
            ],
            kind: CounterKind::PlusOne,
            amount: 1,
        }],
        origin: ActivationOrigin::Battlefield,
        sorcery: false,
        cycling: false,
        exhaust: false,
        once_per_turn: false,
    }],
    ..CardRules::EMPTY
};

#[test]
fn creature_union_excludes_its_source_and_counts_overlap_once() {
    let definition = Card::from_name("Head of the Homestead").unwrap();
    let _rules = fixture_rules::activate(&[(definition, &COUNTER_SOURCE)]);
    let player = Player::P1;
    let mut game = scenario(player);
    let source = put(
        &mut game,
        player,
        "Head of the Homestead",
        Zone::Battlefield,
    );
    let rabbit = put(
        &mut game,
        player,
        "Head of the Homestead",
        Zone::Battlefield,
    );
    let own_nonmatching = put(&mut game, player, "Bear Cub", Zone::Battlefield);
    let opponent = put(
        &mut game,
        player.other(),
        "Head of the Homestead",
        Zone::Battlefield,
    );
    let tokens = game.state.objects.len() as CardId;
    game.create_tokens(player, Card::DrakeToken, 1, false);
    let drake = tokens;
    let rabbit_token = game.state.objects.len() as CardId;
    game.create_tokens(
        player,
        Card::from_name("@token/7f8a13153fafb109da3a6381").unwrap(),
        1,
        false,
    );
    assert!(game.has_subtype(rabbit_token, "Rabbit"));

    choose(
        &mut game,
        |action| matches!(action, Action::Activate { card, ability: 0, .. } | Action::AnnounceActivate { card, ability: 0, .. } if *card == source),
    );
    if matches!(game.state.pending, Pending::PayCosts { .. }) {
        pay(&mut game);
    }
    settle(&mut game);
    for creature in [rabbit, drake, rabbit_token] {
        assert_eq!(game.counter_count(creature, CounterKind::PlusOne), 1);
    }
    for excluded in [source, own_nonmatching, opponent] {
        assert_eq!(game.counter_count(excluded, CounterKind::PlusOne), 0);
    }
}
