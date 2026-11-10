//! A mana X declaration admits only funded values with exact legal targets.
mod support;
use rustmage as _;
use rustmage_core::cards::authored::{CardExtras, CardRules};
use rustmage_core::{cards::*, engine::*};
use support::*;

const SAGA: TargetRule = TargetRule::Filtered {
    base: &TargetRule::Object {
        zone: TargetZone::Graveyard,
        kinds: 0,
        relation: 0,
        other: false,
        nonland: false,
        tapped: false,
        min_mana: 0,
        max_mana: u32::MAX,
    },
    predicate: &TargetPredicate::Subtype("Saga"),
};
const SUMMON: CardRules = CardRules {
    effect: Effect::Creature(1, 2),
    activated: &[ActivatedAbility {
        exhaust: false,
        guard: ActivationGuard::Always,
        cycling: false,
        origin: ActivationOrigin::Battlefield,
        cost: ActivationCost {
            mana_component: true,
            tap: true,
            ..ActivationCost::ZERO
        },
        targets: TargetPlan::Single(TargetSpec::one(TargetRule::ManaValueExactly {
            base: &SAGA,
            amount: AmountValue::ChosenX,
            value: None,
        })),
        program: &[
            Instruction::ActivationX,
            Instruction::CaptureEntryResult,
            Instruction::ReturnToBattlefieldWithCounter(CounterKind::Finality),
            Instruction::GrantEntryResultKeywords {
                keywords: Keywords::HASTE,
                permanent: false,
            },
        ],
        sorcery: true,
        once_per_turn: false,
    }],
    ..CardRules::EMPTY
};

fn rydia() -> Card {
    Card::from_name("Rydia, Summoner of Mist").unwrap()
}

fn announce(card: CardId) -> Action {
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

fn select(game: &mut Game, card: CardId) {
    act(game, target(card));
    if game.legal_actions().contains(&Action::FinishTargets) {
        act(game, Action::FinishTargets);
    }
}

#[test]
fn legal_x_is_the_intersection_of_exact_own_saga_values_and_projected_mana() {
    let _rules = fixture_rules::activate(&[(rydia(), &SUMMON)]);
    let player = Player::P1;
    let opponent = player.other();
    let mut game = scenario(player);
    let source = spawn(&mut game, player, rydia(), Zone::Battlefield);
    let three = put(&mut game, player, "Summon: Choco/Mog", Zone::Graveyard);
    let five = put(&mut game, player, "Summon: Fat Chocobo", Zone::Graveyard);
    let enemy_four = put(
        &mut game,
        opponent,
        "Summon: Primal Garuda",
        Zone::Graveyard,
    );
    let enemy_five = put(&mut game, opponent, "Summon: Shiva", Zone::Graveyard);
    let other_four = put(&mut game, player, "Hill Giant", Zone::Graveyard);
    let non_saga_front = put(
        &mut game,
        player,
        "Esper Origins // Summon: Esper Maduin",
        Zone::Graveyard,
    );
    let available = [0, 0, 0, 0, 0, 2];
    game.state.players[player.index()].mana = available;
    reject(&mut game, announce(source));
    let land = put(&mut game, player, "Forest", Zone::Battlefield);
    let mut only_three = game.clone();
    act(&mut only_three, announce(source));
    assert_eq!(
        only_three.observe(player).x_choice.unwrap().intervals,
        [XInterval {
            minimum: 3,
            maximum: 3
        }]
    );
    let lands = [
        land,
        put(&mut game, player, "Forest", Zone::Battlefield),
        put(&mut game, player, "Forest", Zone::Battlefield),
    ];
    let mut off_turn = game.clone();
    off_turn.state.active = opponent;
    reject(&mut off_turn, announce(source));

    act(&mut game, announce(source));
    assert_eq!(
        game.observe(player).x_choice.unwrap().intervals,
        [
            XInterval {
                minimum: 3,
                maximum: 3
            },
            XInterval {
                minimum: 5,
                maximum: 5
            },
        ]
    );
    assert!(game.observe(opponent).x_choice.is_none());
    for value in [0, 1, 2, 4, 6] {
        reject(&mut game, Action::ChooseX { value });
    }

    let action = Action::ChooseX { value: 5 };
    let index = game
        .legal_actions()
        .iter()
        .position(|candidate| *candidate == action)
        .unwrap();
    let revision = game.revision();
    let saved = game.snapshot();
    let mut raw = game.clone();
    act(&mut raw, action);
    game.apply_index(player, revision, index).unwrap();
    for branch in [&mut raw, &mut game] {
        assert_eq!(branch.state.pending, Pending::Targets { player });
        for illegal in [three, enemy_four, enemy_five, other_four, non_saga_front] {
            reject(branch, target(illegal));
        }
        select(branch, five);
        assert!(!branch.object_view(source).tapped);
        for land in lands {
            act(branch, Action::TapMana { card: land });
        }
        reject(
            branch,
            Action::PayCosts {
                payment: [0, 0, 0, 0, 2, 2],
            },
        );
        pay(branch);
        assert_eq!(branch.state.players[player.index()].mana, [0; 6]);
        assert!(branch.object_view(source).tapped);
        assert_eq!(branch.state.stack.last().unwrap().x, Some(5));
    }
    assert_eq!(raw.snapshot(), game.snapshot());

    let mut stale = raw.clone();
    stale.move_object(five, Zone::Exile);
    stale.move_object(five, Zone::Graveyard);
    settle(&mut stale);
    assert_eq!(stale.object_view(five).zone, Zone::Graveyard);

    raw.move_object(source, Zone::Exile);
    settle(&mut raw);
    assert_eq!(raw.object_view(five).zone, Zone::Battlefield);
    assert_eq!(raw.counter_count(five, CounterKind::Finality), 1);
    assert!(raw.keywords(five).contains(Keywords::HASTE));
    raw.finish_cleanup();
    assert!(!raw.keywords(five).contains(Keywords::HASTE));
    raw.move_object(five, Zone::Graveyard);
    assert_eq!(raw.object_view(five).zone, Zone::Exile);

    raw.restore(&saved).unwrap();
    act(&mut raw, Action::ChooseX { value: 3 });
    select(&mut raw, three);
    act(&mut raw, Action::TapMana { card: lands[0] });
    pay(&mut raw);
    settle(&mut raw);
    assert_eq!(raw.object_view(three).zone, Zone::Battlefield);
    assert_eq!(raw.object_view(five).zone, Zone::Graveyard);
}

const BORROWER: CardRules = CardRules {
    effect: Effect::Creature(2, 2),
    extras: &CardExtras {
        borrows_exiled_activations: true,
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};

#[test]
fn a_borrowed_mana_x_template_uses_the_borrowers_funding_and_exact_saga() {
    let _rules = fixture_rules::activate(&[(rydia(), &SUMMON), (Card::GrizzlyBears, &BORROWER)]);
    let player = Player::P0;
    let mut game = scenario(player);
    let recipient = spawn(&mut game, player, Card::GrizzlyBears, Zone::Battlefield);
    let donor = spawn(&mut game, player, rydia(), Zone::Exile);
    let saga = put(&mut game, player, "Summon: Choco/Mog", Zone::Graveyard);
    let source = game.object_ref(recipient);
    let exiled = game.object_ref(donor);
    game.state
        .borrowed_links
        .get_or_insert_default()
        .push(borrowed_activations::Link {
            source,
            origin: None,
            exiled,
            first_slot: 0,
            width: borrowed_activations::DONOR_SLOT_WIDTH,
            once_per_turn: false,
        });
    let action = Action::AnnounceBorrowedAbility {
        card: recipient,
        exiled,
        ability: 0,
        counter_payment: Counters::default(),
    };
    game.state.players[player.index()].mana = [0, 0, 0, 0, 0, 2];
    reject(&mut game, action.clone());
    game.state.players[player.index()].mana[5] = 3;
    act(&mut game, action);
    assert_eq!(
        game.observe(player).x_choice.unwrap().intervals,
        [XInterval {
            minimum: 3,
            maximum: 3
        }]
    );
    act(&mut game, Action::ChooseX { value: 3 });
    select(&mut game, saga);
    pay(&mut game);
    assert!(game.object_view(recipient).tapped);
    assert_eq!(
        game.state.stack.last().unwrap().source().unwrap().object,
        source
    );
    settle(&mut game);
    assert_eq!(game.object_ref(donor), exiled);
    assert_eq!(game.object_view(saga).zone, Zone::Battlefield);
    assert_eq!(game.counter_count(saga, CounterKind::Finality), 1);
    assert!(game.keywords(saga).contains(Keywords::HASTE));
}
