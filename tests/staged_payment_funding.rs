//! Payment menus describe current choices; callers own future payment planning.
mod support;
use rustmage::Card;
use rustmage_core::cards::authored::CardRules;
use rustmage_core::cards::*;
use rustmage_core::engine::*;
use support::*;

const CONVERTER: CardRules = CardRules {
    effect: Effect::Creature(2, 2),
    activated: &[ActivatedAbility {
        cost: ActivationCost {
            generic: 2,
            mana_component: true,
            ..ActivationCost::ZERO
        },
        targets: TargetPlan::NONE,
        program: &[Instruction::AddMana(ManaProduction::Fixed([
            0, 0, 0, 0, 1, 0,
        ]))],
        guard: ActivationGuard::Always,
        origin: ActivationOrigin::Battlefield,
        exhaust: false,
        cycling: false,
        sorcery: false,
        once_per_turn: false,
    }],
    ..CardRules::EMPTY
};

fn no_projection_work() {
    let work = Game::payment_planning_work();
    assert_eq!(work.projected_moves, 0);
    assert_eq!(work.projected_steps, 0);
    assert_eq!(work.continuation_plans, 0);
}

#[test]
fn a_concrete_child_payment_may_strand_its_parent_without_projection() {
    let _rules = fixture_rules::activate(&[(Card::GrizzlyBears, &CONVERTER)]);
    for player in [Player::P0, Player::P1] {
        let mut raw = scenario(player);
        let source = put(&mut raw, player, "Grizzly Bears", Zone::Battlefield);
        let spell = put(&mut raw, player, "Divination", Zone::Hand);
        raw.state.players[player.index()].mana = [0, 1, 0, 0, 2, 0];
        Game::reset_payment_planning_work();
        act(&mut raw, Action::AnnounceCast { card: spell });
        let child = Action::AnnounceMana {
            card: source,
            ability: 0,
            counter_payment: Counters::default(),
            production: [0, 0, 0, 0, 1, 0],
        };
        assert!(raw.legal_actions().contains(&child));
        raw.observe(player);
        raw.observe(player.other());
        let mut indexed = raw.clone();
        let index = indexed
            .legal_actions()
            .iter()
            .position(|action| action == &child)
            .unwrap();
        indexed
            .apply_index(player, indexed.revision(), index)
            .unwrap();
        act(&mut raw, child);
        assert_eq!(raw.snapshot(), indexed.snapshot());
        let payment = Action::PayCosts {
            payment: [0, 1, 0, 0, 1, 0],
        };
        let index = indexed
            .legal_actions()
            .iter()
            .position(|action| action == &payment)
            .unwrap();
        indexed
            .apply_index(player, indexed.revision(), index)
            .unwrap();
        act(&mut raw, payment);
        assert_eq!(raw.snapshot(), indexed.snapshot());
        assert_eq!(raw.state.players[player.index()].mana, [0, 0, 0, 0, 2, 0]);
        assert_eq!(
            raw.state
                .announcing
                .as_ref()
                .unwrap()
                .item
                .source
                .unwrap()
                .id,
            spell
        );
        assert!(raw.mana_payment_domain().is_none());
        reject(
            &mut raw,
            Action::PayCosts {
                payment: [0, 1, 0, 0, 1, 0],
            },
        );
        no_projection_work();
    }
}

#[test]
fn large_x_domain_and_invalid_concrete_payment_do_no_projection_work() {
    for player in [Player::P0, Player::P1] {
        let mut game = scenario(player);
        let spell = put(&mut game, player, "Exsanguinate", Zone::Hand);
        game.state.players[player.index()].mana = [0; 6];
        Game::reset_payment_planning_work();
        act(&mut game, Action::AnnounceCast { card: spell });
        let choice = game.x_choice().expect("parameterized X declaration");
        assert!(choice.contains(1 << 20));
        game.observe(player);
        act(&mut game, Action::ChooseX { value: 1 << 20 });
        assert!(game.mana_payment_domain().is_none());
        reject(&mut game, Action::PayCosts { payment: [0; 6] });
        no_projection_work();
    }
}

#[test]
fn closing_the_mana_window_preserves_payment_but_hides_new_abilities() {
    let mut game = scenario(Player::P0);
    let forest = put(&mut game, Player::P0, "Forest", Zone::Battlefield);
    let spell = put(&mut game, Player::P0, "Divination", Zone::Hand);
    game.state.players[0].mana = [0, 1, 0, 0, 2, 0];
    Game::reset_payment_planning_work();
    act(&mut game, Action::AnnounceCast { card: spell });
    assert!(
        game.legal_actions()
            .contains(&Action::TapMana { card: forest })
    );
    act(&mut game, Action::EndManaWindow);
    assert!(
        !game
            .legal_actions()
            .contains(&Action::TapMana { card: forest })
    );
    reject(&mut game, Action::TapMana { card: forest });
    act(
        &mut game,
        Action::PayCosts {
            payment: [0, 1, 0, 0, 2, 0],
        },
    );
    assert!(game.state.announcing.is_none());
    no_projection_work();
}

const TAP_AND_WATERBEND: CardRules = CardRules {
    effect: Effect::Creature(2, 2),
    activated: &[ActivatedAbility {
        cost: ActivationCost {
            tap: true,
            generic: 1,
            mana_component: true,
            ..ActivationCost::ZERO
        },
        program: &[
            Instruction::WaterbendCost(1),
            Instruction::GainLife(PlayerSet::Controller, 1),
        ],
        ..CONVERTER.activated[0]
    }],
    ..CardRules::EMPTY
};

#[test]
fn concrete_waterbend_cannot_pay_the_same_source_tap_twice() {
    let _rules = fixture_rules::activate(&[(Card::GrizzlyBears, &TAP_AND_WATERBEND)]);
    let mut game = scenario(Player::P0);
    let source = put(&mut game, Player::P0, "Grizzly Bears", Zone::Battlefield);
    let helper = put(&mut game, Player::P0, "Goblin Piker", Zone::Battlefield);
    game.state.players[0].mana = [0; 6];
    act(
        &mut game,
        Action::AnnounceActivate {
            card: source,
            ability: 0,
            counter_payment: Counters::default(),
        },
    );
    act(&mut game, Action::BeginWaterbend);
    let mut invalid = game.clone();
    // Supply a complete conflicting clause, independently of prefix filtering.
    let reference = invalid.object_ref(source);
    let frame = invalid
        .state
        .announcing
        .as_mut()
        .unwrap()
        .convoke
        .as_mut()
        .unwrap();
    frame.substituted.push(reference);
    frame.remaining.generic = 0;
    assert!(invalid.mana_payment_domain().is_none());
    assert!(
        !invalid
            .legal_actions()
            .contains(&Action::PayCosts { payment: [0; 6] })
    );
    reject(&mut invalid, Action::PayCosts { payment: [0; 6] });
    assert!(!invalid.object_view(source).tapped);
    let helper = game.object_ref(helper);
    act(&mut game, Action::ChooseWaterbender { object: helper });
    act(&mut game, Action::PayCosts { payment: [0; 6] });
    assert!(game.object_view(source).tapped);
    assert!(game.object_view(helper.card).tapped);
    assert!(game.state.announcing.is_none());
}

const SHARED_COUNTERS: CardRules = CardRules {
    effect: Effect::Creature(2, 2),
    activated: &[
        ActivatedAbility {
            cost: ActivationCost {
                generic: 1,
                mana_component: true,
                remove: Some((Some(CounterKind::PlusOne), 1)),
                additional: Some(AdditionalCost {
                    kind: EffectChoiceKind::RemoveCreatureCounter(CounterKind::PlusOne),
                    filter: SacrificeFilter {
                        kinds: 1,
                        source_only: false,
                        other: false,
                        nonland: false,
                        tokens: None,
                        subtype: None,
                        keywords: Keywords::NONE,
                        artifact_enchantment_or_token: false,
                    },
                }),
                ..ActivationCost::ZERO
            },
            program: &[Instruction::GainLife(PlayerSet::Controller, 1)],
            ..CONVERTER.activated[0]
        },
        ActivatedAbility {
            cost: ActivationCost {
                remove: Some((Some(CounterKind::PlusOne), 1)),
                ..ActivationCost::ZERO
            },
            program: &[Instruction::AddMana(ManaProduction::Fixed([
                0, 0, 0, 0, 0, 1,
            ]))],
            ..CONVERTER.activated[0]
        },
    ],
    ..CardRules::EMPTY
};

#[test]
fn a_nested_payment_cannot_make_one_counter_pay_two_parent_costs() {
    let _rules = fixture_rules::activate(&[(Card::GrizzlyBears, &SHARED_COUNTERS)]);
    let mut game = scenario(Player::P0);
    let source = put(&mut game, Player::P0, "Grizzly Bears", Zone::Battlefield);
    game.state.objects[source as usize].counters.plus_one = 2;
    game.state.players[0].mana = [0, 0, 0, 0, 0, 1];
    act(
        &mut game,
        Action::AnnounceActivate {
            card: source,
            ability: 0,
            counter_payment: Counters::of(CounterKind::PlusOne, 1),
        },
    );
    let object = game.object_ref(source);
    act(
        &mut game,
        Action::ChooseCreatureCounterCost {
            object,
            counter: CounterKind::PlusOne,
        },
    );
    let mut valid = game.clone();
    act(
        &mut valid,
        Action::PayCosts {
            payment: [0, 0, 0, 0, 0, 1],
        },
    );
    assert_eq!(valid.state.objects[source as usize].counters.plus_one, 0);
    assert!(valid.state.announcing.is_none());

    act(
        &mut game,
        Action::AnnounceMana {
            card: source,
            ability: 1,
            counter_payment: Counters::of(CounterKind::PlusOne, 1),
            production: [0, 0, 0, 0, 0, 1],
        },
    );
    act(&mut game, Action::PayCosts { payment: [0; 6] });
    assert_eq!(game.state.objects[source as usize].counters.plus_one, 1);
    assert!(game.mana_payment_domain().is_none());
    assert!(!game.legal_actions().contains(&Action::PayCosts {
        payment: [0, 0, 0, 0, 0, 1]
    }));
    reject(
        &mut game,
        Action::PayCosts {
            payment: [0, 0, 0, 0, 0, 1],
        },
    );
}
