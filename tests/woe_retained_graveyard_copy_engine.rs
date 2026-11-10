//! Exact paid graveyard copying with a copiable activation template.
mod support;

use rustmage as _;
use rustmage_core::cards::authored::CardRules;
use rustmage_core::{cards::*, engine::*};
use support::*;

const SOURCE: &str = "Dreadwing Scavenger";
const ZERO: Mana = [0; 6];
const BASE: ActivatedAbility = ActivatedAbility {
    exhaust: false,
    guard: ActivationGuard::Always,
    cycling: false,
    origin: ActivationOrigin::Battlefield,
    cost: ActivationCost::ZERO,
    targets: TargetPlan::NONE,
    program: &[],
    sorcery: false,
    once_per_turn: false,
};
const LOOTER: CardRules = CardRules {
    effect: Effect::Creature(1, 1),
    keywords: Keywords::FLYING,
    activated: &[
        ActivatedAbility {
            cost: ActivationCost {
                tap: true,
                ..ActivationCost::ZERO
            },
            program: &[
                Instruction::Draw(PlayerSet::Controller, 1),
                Instruction::Discard(PlayerSet::Controller, 1),
            ],
            ..BASE
        },
        ActivatedAbility {
            cost: ActivationCost {
                mana_component: true,
                ..ActivationCost::ZERO
            },
            targets: TargetPlan::Single(TargetSpec::one(TargetRule::ManaValueExactly {
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
                amount: AmountValue::ChosenX,
                value: None,
            })),
            program: &[
                Instruction::ActivationX,
                Instruction::CopySourceFromGraveyardRetainingActivation {
                    blueprint_group: 0,
                    keywords: Keywords::FLYING,
                },
            ],
            sorcery: true,
            ..BASE
        },
    ],
    ..CardRules::EMPTY
};
const COPY_TOKEN: CardRules = CardRules {
    effect: Effect::Spell(
        TargetPlan::Single(TargetSpec::one(TargetRule::Creature)),
        &[Instruction::CreateCopyToken {
            subject: Subject::Targets,
            amount: 1,
        }],
    ),
    ..CardRules::EMPTY
};

fn original(card: CardId) -> Action {
    Action::AnnounceActivate {
        card,
        ability: 1,
        counter_payment: Counters::default(),
    }
}

fn loot(card: CardId) -> Action {
    Action::AnnounceActivate {
        card,
        ability: 0,
        counter_payment: Counters::default(),
    }
}

fn target(card: CardId) -> Action {
    Action::ChooseTarget {
        target: Target::Card(card),
    }
}

fn select(game: &mut Game, blueprint: CardId) {
    act(game, target(blueprint));
    if game.legal_actions().contains(&Action::FinishTargets) {
        act(game, Action::FinishTargets);
    }
}

fn retained(source: ObjectRef) -> Action {
    Action::AnnounceRetainedCopyAbility { source }
}

#[test]
fn exact_funded_x_copies_and_recopies_without_resetting_the_source() {
    let definition = Card::from_name(SOURCE).unwrap();
    let _rules = fixture_rules::activate(&[(definition, &LOOTER)]);
    for player in [Player::P0, Player::P1] {
        let mut game = scenario(player);
        let source = put(&mut game, player, SOURCE, Zone::Battlefield);
        let elf = put(&mut game, player, "Llanowar Elves", Zone::Graveyard);
        let bear = put(&mut game, player, "Grizzly Bears", Zone::Graveyard);
        let walker = put(&mut game, player, "Marketback Walker", Zone::Graveyard);
        let instant = put(&mut game, player, "Opt", Zone::Graveyard);
        let foreign = put(&mut game, player.other(), "Llanowar Elves", Zone::Graveyard);
        let forest = put(&mut game, player, "Forest", Zone::Battlefield);
        let exact_source = game.object_ref(source);
        game.state.players[player.index()].mana = ZERO;
        game.set_counters(source, Counters::of(CounterKind::PlusOne, 1));
        assert!(game.legal_actions().contains(&loot(source)));

        act(&mut game, original(source));
        assert_eq!(
            game.observe(player).x_choice.unwrap().intervals,
            [XInterval {
                minimum: 0,
                maximum: 1
            }]
        );
        assert!(game.observe(player.other()).x_choice.is_none());
        reject(&mut game, Action::ChooseX { value: 2 });

        let mut zero = game.clone();
        act(&mut zero, Action::ChooseX { value: 0 });
        select(&mut zero, walker);
        pay(&mut zero);
        settle(&mut zero);
        assert_eq!(zero.object_view(source).name(), "Marketback Walker");
        assert_eq!(stats(&zero, source), (1, 1));
        assert!(zero.object_view(source).keywords.contains(Keywords::FLYING));
        zero.state.players[player.index()].mana = [0, 0, 0, 0, 0, 4];
        act(
            &mut zero,
            Action::Activate {
                card: source,
                ability: 0,
                target: None,
                payment: [0, 0, 0, 0, 0, 4],
                counter_payment: Counters::default(),
            },
        );
        settle(&mut zero);
        assert_eq!(zero.counter_count(source, CounterKind::PlusOne), 2);
        assert!(!zero.object_view(source).tapped);
        assert!(zero.legal_actions().contains(&retained(exact_source)));

        act(&mut game, Action::ChooseX { value: 1 });
        for illegal in [bear, instant, foreign] {
            reject(&mut game, target(illegal));
        }
        select(&mut game, elf);
        act(&mut game, Action::TapMana { card: forest });
        pay(&mut game);
        settle(&mut game);
        assert_eq!(game.object_ref(source), exact_source);
        assert_eq!(game.object_view(source).name(), "Llanowar Elves");
        assert_eq!(stats(&game, source), (2, 2));
        reject(&mut game, loot(source));

        let second_forest = put(&mut game, player, "Forest", Zone::Battlefield);
        let mut off_turn = game.clone();
        off_turn.state.active = player.other();
        reject(&mut off_turn, retained(exact_source));

        let action = retained(exact_source);
        let index = game
            .legal_actions()
            .iter()
            .position(|candidate| *candidate == action)
            .unwrap();
        let revision = game.revision();
        let checkpoint = game.snapshot();
        let mut indexed = game.clone();
        indexed.apply_index(player, revision, index).unwrap();
        act(&mut game, action.clone());
        assert_eq!(indexed.snapshot(), game.snapshot());
        indexed.restore(&checkpoint).unwrap();
        assert_eq!(
            indexed.apply_index(player, revision, index),
            Err(Error::StaleRevision)
        );
        act(&mut indexed, action);
        assert_eq!(indexed.snapshot(), game.snapshot());
        assert!(game.observe(player).x_choice.unwrap().contains(2));
        act(&mut game, Action::ChooseX { value: 2 });
        select(&mut game, bear);
        act(
            &mut game,
            Action::TapMana {
                card: second_forest,
            },
        );
        act(&mut game, Action::TapMana { card: source });
        pay(&mut game);

        let mut stale = game.clone();
        let old = stale.object_ref(bear);
        stale.move_object(bear, Zone::Exile);
        stale.move_object(bear, Zone::Graveyard);
        assert_ne!(stale.object_ref(bear), old);
        settle(&mut stale);
        assert_eq!(stale.object_view(source).name(), "Llanowar Elves");

        let mut departed = game.clone();
        departed.move_object(source, Zone::Exile);
        departed.move_object(source, Zone::Battlefield);
        settle(&mut departed);
        assert_ne!(departed.object_ref(source), exact_source);
        assert_eq!(departed.object_view(source).name(), SOURCE);
        assert!(departed.legal_actions().contains(&original(source)));
        reject(&mut departed, retained(exact_source));

        settle(&mut game);
        let copied = game.object_view(source);
        assert_eq!(copied.reference, exact_source);
        assert_eq!(copied.name(), "Grizzly Bears");
        assert!(copied.tapped);
        assert!(copied.keywords.contains(Keywords::FLYING));
        assert_eq!(stats(&game, source), (3, 3));
        assert_eq!(game.state.players[player.index()].mana, ZERO);
        assert_eq!(
            game.copiable_recipe(exact_source)
                .exceptions
                .retained_activation,
            Some(RetainedCopyActivation {
                definition,
                ability: 1
            })
        );
        game.state.step = Step::End;
        game.give_priority();
        act(&mut game, Action::Pass);
        act(&mut game, Action::Pass);
        assert_eq!(game.object_view(source).name(), "Grizzly Bears");
    }
}

#[test]
fn copies_of_the_blueprint_inherit_flying_and_their_own_retained_activation() {
    let _rules = fixture_rules::activate(&[
        (Card::from_name(SOURCE).unwrap(), &LOOTER),
        (Card::from_name("Think Twice").unwrap(), &COPY_TOKEN),
    ]);
    let player = Player::P0;
    let mut game = scenario(player);
    let source = put(&mut game, player, SOURCE, Zone::Battlefield);
    let elf = put(&mut game, player, "Llanowar Elves", Zone::Graveyard);
    let bear = put(&mut game, player, "Grizzly Bears", Zone::Graveyard);
    let copy = put(&mut game, player, "Think Twice", Zone::Hand);
    act(&mut game, original(source));
    act(&mut game, Action::ChooseX { value: 1 });
    select(&mut game, elf);
    pay(&mut game);
    settle(&mut game);

    let token = game.state.objects.len() as CardId;
    cast(&mut game, copy, Some(Target::Card(source)));
    settle(&mut game);
    assert!(game.object_view(token).is_token());
    assert_eq!(game.object_view(token).name(), "Llanowar Elves");
    assert!(game.object_view(token).keywords.contains(Keywords::FLYING));
    let object = game.object_ref(token);
    act(&mut game, retained(object));
    act(&mut game, Action::ChooseX { value: 2 });
    select(&mut game, bear);
    pay(&mut game);
    settle(&mut game);
    assert_eq!(game.object_view(token).name(), "Grizzly Bears");
    assert_eq!(game.object_ref(token), object);
    assert!(game.object_view(token).is_token());
    assert!(game.object_view(token).keywords.contains(Keywords::FLYING));
    assert_eq!(game.object_view(source).name(), "Llanowar Elves");
}
