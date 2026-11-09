mod support;
use rustmage as _;
use rustmage_core::cards::authored::CardRules;
use rustmage_core::{cards::*, engine::*};
use support::*;

const CREATE: CardRules = CardRules {
    effect: Effect::Spell(
        TargetPlan::NONE,
        &[Instruction::CreateTokensThenCounters {
            token: Card(5328),
            amount: 1,
            kind: CounterKind::PlusOne,
            counters: AmountValue::Stat(StatValue::Count(CountFilter {
                zone: CountZone::Battlefield,
                relation: 0,
                kind: 2,
                exclude: 0,
                subtype: None,
                basic: false,
                other: false,
                cards_only: false,
            })),
        }],
    ),
    ..CardRules::EMPTY
};
const ANIMATE: CardRules = CardRules {
    effect: Effect::Spell(
        TargetPlan::Single(TargetSpec::one(TargetRule::Object {
            zone: TargetZone::Battlefield,
            kinds: 2,
            relation: 0,
            other: false,
            nonland: false,
            tapped: false,
            min_mana: 0,
            max_mana: u32::MAX,
        })),
        &[Instruction::AnimateTargets(AnimationSpec {
            artifact: false,
            creature_types: AnimationCreatureTypes::Unchanged,
            base: Some((4, 4)),
            keywords: Keywords::NONE,
            duration: AnimationDuration::EndOfTurn,
        })],
    ),
    ..CardRules::EMPTY
};

fn created(game: &Game) -> ObjectView {
    let mut tokens = game
        .state
        .objects
        .iter()
        .enumerate()
        .filter_map(|(id, object)| {
            (object.zone == Zone::Battlefield && object.is_token())
                .then(|| game.object_view(id as CardId))
        });
    let token = tokens.next().unwrap();
    assert!(tokens.next().is_none());
    token
}

#[test]
fn the_live_land_count_includes_an_actual_token_land_created_by_a_replacement() {
    let _rules = fixture_rules::activate(&[
        (Card::from_name("Opt").unwrap(), &CREATE),
        (Card::from_name("Shore Up").unwrap(), &ANIMATE),
    ]);
    let player = Player::P0;
    let mut game = scenario(player);
    let forest = put(&mut game, player, "Forest", Zone::Battlefield);
    put(&mut game, player, "Island", Zone::Battlefield);
    put(&mut game, player.other(), "Forest", Zone::Battlefield);
    let crown = put(&mut game, player, "Mirrormind Crown", Zone::Battlefield);
    let animate = put(&mut game, player, "Shore Up", Zone::Hand);
    cast(&mut game, animate, Some(Target::Card(forest)));
    settle(&mut game);
    assert_eq!(game.object_view(forest).kinds & 3, 3);
    choose(&mut game, |action| {
        matches!(action, Action::Activate { card, ability: 0, target: Some(Target::Card(target)), .. }
            if *card == crown && *target == forest)
    });
    settle(&mut game);
    assert_eq!(
        game.object_view(crown).attached_to,
        Some(game.object_ref(forest))
    );
    let spell = put(&mut game, player, "Opt", Zone::Hand);
    cast(&mut game, spell, None);
    act(&mut game, Action::Pass);
    act(&mut game, Action::Pass);
    assert!(matches!(game.state.pending, Pending::Replacement { .. }));

    let mut declined = game.clone();
    act(&mut declined, Action::ChooseOptional { accept: false });
    let hero = created(&declined);
    assert!(hero.card.has_subtype("Hero"));
    assert_eq!(declined.counter_count(hero.id, CounterKind::PlusOne), 2);

    let accept = Action::ChooseOptional { accept: true };
    let saved = game.snapshot();
    let revision = game.revision();
    let index = game
        .legal_actions()
        .iter()
        .position(|action| *action == accept)
        .unwrap();
    let mut indexed = game.clone();
    indexed.apply_index(player, revision, index).unwrap();
    act(&mut game, accept);
    assert_eq!(game.snapshot(), indexed.snapshot());
    indexed.restore(&saved).unwrap();
    assert_eq!(indexed.snapshot(), saved);
    let copy = created(&game);
    assert_eq!(copy.card, Card::Forest);
    assert_eq!(copy.kinds & 3, 2);
    assert_eq!(game.counter_count(copy.id, CounterKind::PlusOne), 3);
    assert_eq!(game.counter_count(forest, CounterKind::PlusOne), 0);
}
