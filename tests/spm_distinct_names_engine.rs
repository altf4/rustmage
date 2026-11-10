//! A target group compares the rules names of exact graveyard creature cards.
mod support;
use rustmage as _;
use rustmage_core::cards::authored::CardRules;
use rustmage_core::cards::*;
use rustmage_core::engine::*;
use support::*;

const RETURN: CardRules = CardRules {
    effect: Effect::Spell(
        TargetPlan::Single(TargetSpec {
            rule: TargetRule::Grouped {
                base: &TargetRule::Object {
                    zone: TargetZone::Graveyard,
                    kinds: 1,
                    relation: 0,
                    other: false,
                    nonland: false,
                    tapped: false,
                    min_mana: 0,
                    max_mana: u32::MAX,
                },
                constraint: TargetGroupConstraint::DistinctNames,
            },
            min: 0,
            max: 6,
        }),
        &[Instruction::ReturnToBattlefield],
    ),
    ..CardRules::EMPTY
};

const FORK: CardRules = CardRules {
    effect: Effect::Spell(
        TargetPlan::Single(TargetSpec::one(TargetRule::Spell)),
        &[Instruction::CopyTargetSpell {
            group: 0,
            retarget: true,
            exceptions: CopyExceptions::NONE,
        }],
    ),
    ..CardRules::EMPTY
};

fn choice(card: CardId) -> Action {
    Action::ChooseTarget {
        target: Target::Card(card),
    }
}

fn announce(game: &mut Game) -> CardId {
    let card = put(game, Player::P0, "Zombify", Zone::Hand);
    act(game, Action::AnnounceCast { card });
    card
}

#[test]
fn zero_to_six_distinct_names_are_legal_and_stale_targets_drop_individually() {
    let _rules = fixture_rules::activate(&[(Card::from_name("Zombify").unwrap(), &RETURN)]);
    let mut game = scenario(Player::P0);
    let names = [
        "Grizzly Bears",
        "Hill Giant",
        "Bear Cub",
        "Savannah Lions",
        "Goblin Piker",
        "Llanowar Elves",
        "Druid of the Cowl",
    ];
    let cards = names.map(|name| put(&mut game, Player::P0, name, Zone::Graveyard));
    let duplicate = put(&mut game, Player::P0, "Grizzly Bears", Zone::Graveyard);
    let other = put(&mut game, Player::P1, "Grizzly Bears", Zone::Graveyard);
    let noncreature = put(&mut game, Player::P0, "Forest", Zone::Graveyard);
    announce(&mut game);
    let open = game.snapshot();

    act(&mut game, Action::FinishTargets);
    pay(&mut game);
    settle(&mut game);
    assert!(
        cards
            .iter()
            .all(|id| game.object_view(*id).zone == Zone::Graveyard)
    );

    game.restore(&open).unwrap();
    reject(&mut game, choice(other));
    reject(&mut game, choice(noncreature));
    let action = choice(cards[0]);
    let mut indexed = game.clone();
    let index = indexed
        .legal_actions()
        .iter()
        .position(|a| *a == action)
        .unwrap();
    indexed
        .apply_index(Player::P0, indexed.revision(), index)
        .unwrap();
    act(&mut game, action);
    assert_eq!(game.snapshot(), indexed.snapshot());
    reject(&mut game, choice(cards[0]));
    reject(&mut game, choice(duplicate));
    for card in &cards[1..6] {
        act(&mut game, choice(*card));
    }
    if game.legal_actions().contains(&Action::FinishTargets) {
        reject(&mut game, choice(cards[6]));
        act(&mut game, Action::FinishTargets);
    }
    reject(&mut game, choice(cards[6]));
    pay(&mut game);
    let old = game.object_ref(cards[0]);
    game.move_object(cards[0], Zone::Exile);
    game.move_object(cards[0], Zone::Graveyard);
    assert_ne!(game.object_ref(cards[0]), old);
    settle(&mut game);
    assert_eq!(game.object_view(cards[0]).zone, Zone::Graveyard);
    assert_eq!(game.object_view(cards[6]).zone, Zone::Graveyard);
    assert_eq!(game.object_view(duplicate).zone, Zone::Graveyard);
    assert!(
        cards[1..6]
            .iter()
            .all(|id| game.object_view(*id).zone == Zone::Battlefield)
    );
}

#[test]
fn a_spell_copy_checks_distinct_names_when_retargeting_is_finished() {
    let _rules = fixture_rules::activate(&[
        (Card::from_name("Zombify").unwrap(), &RETURN),
        (Card::Shock, &FORK),
    ]);
    let mut game = scenario(Player::P0);
    let first = put(&mut game, Player::P0, "Grizzly Bears", Zone::Graveyard);
    let second = put(&mut game, Player::P0, "Hill Giant", Zone::Graveyard);
    let duplicate = put(&mut game, Player::P0, "Grizzly Bears", Zone::Graveyard);
    let unique = put(&mut game, Player::P0, "Llanowar Elves", Zone::Graveyard);
    let spell = announce(&mut game);
    for card in [first, second] {
        act(&mut game, choice(card));
    }
    act(&mut game, Action::FinishTargets);
    pay(&mut game);

    let fork = put(&mut game, Player::P0, "Shock", Zone::Hand);
    cast(&mut game, fork, Some(Target::Spell(spell)));
    act(&mut game, Action::Pass);
    act(&mut game, Action::Pass);
    assert!(matches!(game.state.pending, Pending::Retarget { .. }));
    let saved = game.snapshot();

    act(
        &mut game,
        Action::SetStackTarget {
            index: 1,
            target: Target::Card(duplicate),
        },
    );
    reject(&mut game, Action::FinishStackTargets);
    game.restore(&saved).unwrap();
    act(
        &mut game,
        Action::SetStackTarget {
            index: 1,
            target: Target::Card(unique),
        },
    );
    act(&mut game, Action::FinishStackTargets);
    settle(&mut game);
    for card in [first, second, unique] {
        assert_eq!(game.object_view(card).zone, Zone::Battlefield);
    }
    assert_eq!(game.object_view(duplicate).zone, Zone::Graveyard);
}
