//! Positive X costs and independently retargeted, exact ability copies.
mod support;
use rustmage as _;
use rustmage_core::cards::authored::{CardExtras, CardRules};
use rustmage_core::{cards::*, engine::*};
use support::*;

const fn activated(
    cost: ActivationCost,
    targets: TargetPlan,
    program: &'static [Instruction],
) -> ActivatedAbility {
    ActivatedAbility {
        exhaust: false,
        guard: ActivationGuard::Always,
        cycling: false,
        origin: ActivationOrigin::Battlefield,
        cost,
        targets,
        program,
        sorcery: false,
        once_per_turn: false,
    }
}

const MIMIC: CardRules = CardRules {
    effect: Effect::Creature(2, 4),
    activated: &[activated(
        ActivationCost {
            mana_component: true,
            tap: true,
            ..ActivationCost::ZERO
        },
        TargetPlan::Single(TargetSpec::one(TargetRule::YourAbility)),
        &[
            Instruction::ActivationXWithMinimum {
                symbols: 2,
                minimum: 1,
            },
            Instruction::ActivationCannotBeCopied,
            Instruction::CopyTargetAbilityX { group: 0 },
        ],
    )],
    ..CardRules::EMPTY
};
const TARGETED: CardRules = CardRules {
    effect: Effect::Creature(2, 1),
    activated: &[activated(
        ActivationCost {
            mana_component: true,
            generic: 1,
            tap: true,
            ..ActivationCost::ZERO
        },
        TargetPlan::Single(TargetSpec::one(TargetRule::Creature)),
        &[
            Instruction::ActivationX,
            Instruction::PutCountersScaled {
                subject: Subject::Targets,
                kind: CounterKind::PlusOne,
                amount: AmountValue::ChosenX,
            },
        ],
    )],
    ..CardRules::EMPTY
};
const GAIN_LIFE: CardRules = CardRules {
    effect: Effect::Creature(3, 3),
    activated: &[activated(
        ActivationCost::ZERO,
        TargetPlan::NONE,
        &[Instruction::GainLife(PlayerSet::Controller, 1)],
    )],
    ..CardRules::EMPTY
};

fn announce_activation(card: CardId, ability: u16) -> Action {
    Action::AnnounceActivate {
        card,
        ability,
        counter_payment: Counters::default(),
    }
}

fn choose_target(game: &mut Game, target: Target) {
    act(game, Action::ChooseTarget { target });
    if game.legal_actions().contains(&Action::FinishTargets) {
        act(game, Action::FinishTargets);
    }
}

fn mimic(game: &mut Game, card: CardId, value: u32, target: u64) {
    act(game, announce_activation(card, 0));
    act(game, Action::ChooseX { value });
    choose_target(game, Target::Stack(target));
    pay(game);
}

fn direct(game: &mut Game, id: CardId) {
    choose(
        game,
        |action| matches!(action, Action::Activate { card, ability: 0, .. } if *card == id),
    );
}

fn pass_pair(game: &mut Game) {
    act(game, Action::Pass);
    act(game, Action::Pass);
}

#[test]
fn double_x_funds_and_commits_exactly_then_copies_the_frozen_ability_independently() {
    let piker = Card::from_name("Goblin Piker").unwrap();
    let _rules = fixture_rules::activate(&[
        (Card::GrizzlyBears, &MIMIC),
        (piker, &TARGETED),
        (Card::HillGiant, &GAIN_LIFE),
    ]);
    let player = Player::P0;
    let opponent = player.other();
    let mut game = scenario(player);
    let copy_source = spawn(&mut game, player, Card::GrizzlyBears, Zone::Battlefield);
    let source = spawn(&mut game, player, piker, Zone::Battlefield);
    let exact = game.object_ref(source);
    let first = spawn(&mut game, player, piker, Zone::Battlefield);
    let second = spawn(&mut game, player, piker, Zone::Battlefield);
    let third = spawn(&mut game, player, piker, Zone::Battlefield);

    let announce = announce_activation(copy_source, 0);
    game.state.players[player.index()].mana = [0, 0, 0, 0, 0, 2];
    reject(&mut game, announce.clone()); // No own ability yet.
    act(&mut game, announce_activation(source, 0));
    act(&mut game, Action::ChooseX { value: 1 });
    choose_target(&mut game, Target::Card(first));
    pay(&mut game);
    let original = game.state.stack[0].id;
    assert_eq!(game.state.players[player.index()].mana, [0; 6]);
    assert!(game.object_view(source).tapped);
    let foreign = spawn(&mut game, opponent, Card::HillGiant, Zone::Battlefield);
    act(&mut game, Action::Pass);
    direct(&mut game, foreign);
    let foreign_ability = game.state.stack.last().unwrap().id;
    act(&mut game, Action::Pass);
    assert_eq!(game.actor(), Some(player));

    let land = put(&mut game, player, "Forest", Zone::Battlefield);
    reject(&mut game, announce.clone());
    let mut lands = vec![land];
    lands.extend((0..3).map(|_| put(&mut game, player, "Forest", Zone::Battlefield)));
    act(&mut game, announce);
    assert_eq!(
        game.observe(player).x_choice.unwrap().intervals,
        [XInterval {
            minimum: 1,
            maximum: 2
        }]
    );
    assert!(game.observe(opponent).x_choice.is_none());
    reject(&mut game, Action::ChooseX { value: 0 });
    reject(&mut game, Action::ChooseX { value: 3 });
    act(&mut game, Action::ChooseX { value: 2 });
    reject(
        &mut game,
        Action::ChooseTarget {
            target: Target::Stack(foreign_ability),
        },
    );
    choose_target(&mut game, Target::Stack(original));
    assert!(!game.object_view(copy_source).tapped);
    for land in lands {
        act(&mut game, Action::TapMana { card: land });
    }
    reject(
        &mut game,
        Action::PayCosts {
            payment: [0, 0, 0, 0, 2, 0],
        },
    );
    pay(&mut game);
    assert_eq!(game.state.players[player.index()].mana, [0; 6]);
    assert!(game.object_view(copy_source).tapped);
    assert_eq!(game.state.stack.last().unwrap().x, Some(2));
    game.move_object(source, Zone::Graveyard);
    assert_ne!(game.object_ref(source), exact);
    let object_count = game.state.objects.len();
    pass_pair(&mut game);

    let first_choice = game.observe(player).retarget.unwrap();
    assert_eq!(first_choice.item.x, Some(1));
    assert_eq!(first_choice.item.source().unwrap().object, exact);
    assert_eq!(first_choice.original[0].target, Target::Card(first));
    assert_eq!(first_choice.remaining_copies, 1);
    let choose_second = Action::SetStackTarget {
        index: 0,
        target: Target::Card(second),
    };
    let index = game
        .legal_actions()
        .iter()
        .position(|action| *action == choose_second)
        .unwrap();
    let saved = game.snapshot();
    let revision = game.revision();
    let mut raw = game.clone();
    let mut indexed = game;
    act(&mut raw, choose_second);
    indexed.apply_index(player, revision, index).unwrap();

    for branch in [&mut raw, &mut indexed] {
        act(branch, Action::FinishStackTargets);
        let second_choice = branch.observe(player).retarget.unwrap();
        assert_eq!(second_choice.remaining_copies, 0);
        assert_ne!(second_choice.item.id, first_choice.item.id);
        assert_eq!(second_choice.item.targets[0].target, Target::Card(first));
        act(
            branch,
            Action::SetStackTarget {
                index: 0,
                target: Target::Card(third),
            },
        );
        act(branch, Action::FinishStackTargets);
        assert_eq!(branch.state.stack.len(), 4);
        assert_eq!(branch.state.objects.len(), object_count);
        let stack = &branch.state.stack;
        assert_eq!(stack[0].id, original);
        assert_eq!(stack[0].targets[0].target, Target::Card(first));
        for copy in &stack[2..] {
            assert!(copy.is_copy);
            assert_eq!(copy.source().unwrap().object, exact);
            assert_eq!(copy.x, Some(1));
        }
        settle(branch);
        for target in [first, second, third] {
            assert_eq!(branch.counter_count(target, CounterKind::PlusOne), 1);
        }
    }
    assert_eq!(raw.snapshot(), indexed.snapshot());

    raw.restore(&saved).unwrap();
    raw.move_object(first, Zone::Graveyard);
    act(&mut raw, Action::FinishStackTargets); // Retaining an illegal inherited target is legal.
    act(
        &mut raw,
        Action::SetStackTarget {
            index: 0,
            target: Target::Card(third),
        },
    );
    act(&mut raw, Action::FinishStackTargets);
    settle(&mut raw);
    assert_eq!(raw.counter_count(first, CounterKind::PlusOne), 0);
    assert_eq!(raw.counter_count(second, CounterKind::PlusOne), 0);
    assert_eq!(raw.counter_count(third, CounterKind::PlusOne), 1);
}

#[test]
fn an_uncopiable_activation_is_a_legal_target_but_regular_and_copied_effects_cannot_copy_it() {
    let _rules =
        fixture_rules::activate(&[(Card::GrizzlyBears, &MIMIC), (Card::HillGiant, &GAIN_LIFE)]);
    let player = Player::P0;
    let mut game = scenario(player);
    let source = spawn(&mut game, player, Card::HillGiant, Zone::Battlefield);
    let first = spawn(&mut game, player, Card::GrizzlyBears, Zone::Battlefield);
    let second = spawn(&mut game, player, Card::GrizzlyBears, Zone::Battlefield);
    let echo = put(
        &mut game,
        player,
        "Echo, Perceptive Prodigy",
        Zone::Battlefield,
    );
    direct(&mut game, source);
    let original = game.state.stack[0].id;
    mimic(&mut game, first, 1, original);
    let blocked = game.state.stack[1].id;
    act(&mut game, announce_activation(echo, 0));
    choose_target(&mut game, Target::Stack(blocked));
    pay(&mut game);
    let copying_ability = game.state.stack[2].id;
    mimic(&mut game, second, 1, copying_ability);
    pass_pair(&mut game);
    assert_eq!(game.state.pending, Pending::Retarget { player });
    act(&mut game, Action::FinishStackTargets);
    assert!(game.state.stack[3].is_copy);
    assert_eq!(
        game.state.stack[3].targets[0].target,
        Target::Stack(blocked)
    );
    for remaining in [3, 2] {
        pass_pair(&mut game);
        assert_eq!(game.state.pending, Pending::Priority);
        assert_eq!(game.state.stack.len(), remaining);
        assert_eq!(game.state.stack[1].id, blocked);
    }
    settle(&mut game);
    assert_eq!(game.state.players[player.index()].life, 22);
}

const EXHAUST: CardRules = CardRules {
    effect: Effect::Creature(3, 3),
    activated: &[ActivatedAbility {
        exhaust: true,
        ..activated(
            ActivationCost::ZERO,
            TargetPlan::NONE,
            &[
                Instruction::ActivationCannotBeCopied,
                Instruction::GainLife(PlayerSet::Controller, 1),
            ],
        )
    }],
    extras: &CardExtras {
        exhaust_mask: 1,
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};

#[test]
fn a_captured_exhaust_activation_respects_the_same_copy_restriction() {
    let _rules = fixture_rules::activate(&[(Card::HillGiant, &EXHAUST)]);
    let player = Player::P0;
    let mut game = scenario(player);
    let pit = put(&mut game, player, "Pit Automaton", Zone::Battlefield);
    let source = spawn(&mut game, player, Card::HillGiant, Zone::Battlefield);
    act(&mut game, announce_activation(pit, 1));
    pay(&mut game);
    settle(&mut game);
    direct(&mut game, source);
    assert_eq!(game.state.stack.len(), 2);
    pass_pair(&mut game);
    assert_eq!(game.state.stack.len(), 1);
    assert!(!game.state.stack[0].is_copy);
    settle(&mut game);
    assert_eq!(game.state.players[player.index()].life, 21);
}

const HIDEAWAY: CardRules = CardRules {
    effect: Effect::Land,
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::SelfEnters,
        targets: TargetPlan::NONE,
        program: &[Instruction::SelectLibraryTo(LibrarySelectionEffect {
            spec: LibrarySelectionSpec {
                look: 4,
                count: LibrarySelectionCount::Fixed(1),
                filter: LibrarySelectionFilter::Any,
                optional: false,
                reveal: false,
                remainder: LibraryRemainder::BottomRandom,
            },
            look: LibraryLook::Fixed(4),
            destination: LibraryPickDestination::Hideaway,
            public: false,
            maximum_mana_value: None,
        })],
    }],
    ..CardRules::EMPTY
};

#[test]
fn targetless_trigger_copies_each_resolve_and_share_the_exact_private_hideaway_link() {
    let hideaway = Card::from_name("Clive's Hideaway").unwrap();
    let _rules = fixture_rules::activate(&[(Card::GrizzlyBears, &MIMIC), (hideaway, &HIDEAWAY)]);
    let player = Player::P0;
    let opponent = player.other();
    let mut game = scenario(player);
    let copier = spawn(&mut game, player, Card::GrizzlyBears, Zone::Battlefield);
    let land = spawn(&mut game, player, hideaway, Zone::Hand);
    act(&mut game, Action::PlayLand { card: land });
    let source = game.object_ref(land);
    let triggered = game.state.stack[0].id;
    mimic(&mut game, copier, 2, triggered);
    pass_pair(&mut game);
    assert_eq!(game.state.pending, Pending::Priority);
    assert_eq!(game.state.stack.len(), 3);
    for (index, ability) in game.state.stack.iter().enumerate() {
        assert!(matches!(ability.kind, StackKind::TriggeredAbility { .. }));
        assert_eq!(ability.source().unwrap().object, source);
        assert_eq!(ability.is_copy, index != 0);
    }
    let mut hidden = Vec::new();
    for _ in 0..3 {
        pass_pair(&mut game);
        let offer = game.observe(player).library_selection.unwrap();
        assert!(game.observe(opponent).library_selection.is_none());
        let card = offer.candidates[0].card;
        act(&mut game, Action::SelectLibrary { card });
        if game
            .legal_actions()
            .contains(&Action::FinishLibrarySelection)
        {
            act(&mut game, Action::FinishLibrarySelection);
        }
        assert_eq!(game.object_view(card).zone, Zone::Exile);
        hidden.push(game.object_ref(card));
    }
    assert!(game.state.stack.is_empty());
    let links = game.observe(player).linked_exile;
    assert_eq!(links.len(), 3);
    for (link, card) in links.iter().zip(hidden) {
        assert_eq!(link.source, source);
        assert_eq!(link.exiled, card);
    }
    assert!(game.observe(opponent).linked_exile.is_empty());
}
