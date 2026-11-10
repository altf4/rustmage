//! Counter-bound land characteristics and mana event receipts.
mod support;
use rustmage as _;
use rustmage_core::cards::authored::CardRules;
use rustmage_core::{cards::*, engine::*};
use support::*;

const COLORLESS: Mana = [0, 0, 0, 0, 0, 1];
const BLUE_AND_COLORLESS: Mana = [0, 1, 0, 0, 0, 1];
const MANA_BONUS: &[StaticAbilityGroup] = &[StaticAbilityGroup {
    line: 0,
    first_layer: AbilityLayer::Ability,
    fragments: &[StaticAbilityFragment::ControlledLandColorlessTapManaBonus],
}];
const PROVIDER: CardRules = CardRules {
    effect: Effect::Creature(4, 4),
    static_groups: MANA_BONUS,
    ..CardRules::EMPTY
};
const BASE_TAP: ActivatedAbility = ActivatedAbility {
    exhaust: false,
    guard: ActivationGuard::Always,
    cycling: false,
    origin: ActivationOrigin::Battlefield,
    cost: ActivationCost {
        tap: true,
        ..ActivationCost::ZERO
    },
    targets: TargetPlan::NONE,
    program: &[Instruction::AddMana(ManaProduction::Fixed(COLORLESS))],
    sorcery: false,
    once_per_turn: false,
};

fn tap(card: CardId, production: Mana) -> Action {
    Action::ActivateMana {
        card,
        ability: 0,
        counter_payment: Counters::default(),
        payment: [0; 6],
        production,
    }
}

#[test]
fn colorless_tap_bonus_reads_only_the_base_event_and_only_the_controllers_land() {
    const UNCONDITIONAL: CardRules = CardRules {
        effect: Effect::Permanent,
        static_groups: &[StaticAbilityGroup {
            line: 0,
            first_layer: AbilityLayer::Ability,
            fragments: &[StaticAbilityFragment::ControlledLandTapManaBonus(COLORLESS)],
        }],
        ..CardRules::EMPTY
    };
    const ROCK: CardRules = CardRules {
        effect: Effect::Permanent,
        activated: &[BASE_TAP],
        ..CardRules::EMPTY
    };
    let unconditional = Card::from_name("Swiftfoot Boots").unwrap();
    let rock = Card::from_name("Soul-Guide Lantern").unwrap();
    let wastes = Card::from_name("Wastes").unwrap();
    let _rules = fixture_rules::activate(&[
        (Card::HillGiant, &PROVIDER),
        (unconditional, &UNCONDITIONAL),
        (rock, &ROCK),
    ]);
    let player = Player::P1;
    let mut game = scenario(player);
    put(&mut game, player, "Hill Giant", Zone::Battlefield);
    let extra = spawn(&mut game, player, unconditional, Zone::Battlefield);
    let forest = put(&mut game, player, "Forest", Zone::Battlefield);
    let rock = spawn(&mut game, player, rock, Zone::Battlefield);
    let land = spawn(&mut game, player, wastes, Zone::Battlefield);
    let enemy_land = spawn(&mut game, player.other(), wastes, Zone::Battlefield);
    game.state.players[player.index()].mana = [0; 6];
    act(&mut game, Action::TapMana { card: forest });
    // The other trigger's {C} cannot make the original {G} event eligible.
    assert_eq!(game.state.players[player.index()].mana, [0, 0, 0, 0, 1, 1]);
    game.move_object(extra, Zone::Graveyard);
    act(&mut game, tap(rock, COLORLESS));
    assert_eq!(game.state.players[player.index()].mana[5], 2);
    act(&mut game, tap(land, COLORLESS));
    assert_eq!(game.state.players[player.index()].mana[5], 4);
    assert!(game.state.stack.is_empty());
    act(&mut game, Action::Pass);
    game.state.players[player.other().index()].mana = [0; 6];
    act(&mut game, tap(enemy_land, COLORLESS));
    assert_eq!(game.state.players[player.other().index()].mana, COLORLESS);
}

#[test]
fn mixed_mana_adds_colorless_once_alongside_an_owned_bonus_choice() {
    const LAND: CardRules = CardRules {
        effect: Effect::Land,
        activated: &[ActivatedAbility {
            program: &[Instruction::AddMana(ManaProduction::Fixed(
                BLUE_AND_COLORLESS,
            ))],
            ..BASE_TAP
        }],
        ..CardRules::EMPTY
    };
    const ADD_ONE_TYPE: CardRules = CardRules {
        effect: Effect::Permanent,
        static_groups: &[StaticAbilityGroup {
            line: 0,
            first_layer: AbilityLayer::Ability,
            fragments: &[StaticAbilityFragment::BasicLandTapManaBonus(1)],
        }],
        ..CardRules::EMPTY
    };
    let choose_type = Card::from_name("Swiftfoot Boots").unwrap();
    let _rules = fixture_rules::activate(&[
        (Card::HillGiant, &PROVIDER),
        (Card::Forest, &LAND),
        (choose_type, &ADD_ONE_TYPE),
    ]);
    let player = Player::P0;
    let mut game = scenario(player);
    let land = put(&mut game, player, "Forest", Zone::Battlefield);
    put(&mut game, player, "Hill Giant", Zone::Battlefield);
    spawn(&mut game, player.other(), choose_type, Zone::Battlefield);
    game.state.players[player.index()].mana = [0; 6];
    act(&mut game, tap(land, BLUE_AND_COLORLESS));
    assert!(matches!(game.state.pending, Pending::EffectMana { .. }));
    let saved = game.snapshot();
    let action = Action::ChooseMana {
        production: [0, 1, 0, 0, 0, 0],
    };
    let revision = game.revision();
    let index = game
        .legal_actions()
        .iter()
        .position(|a| *a == action)
        .unwrap();
    let mut indexed = game.clone();
    indexed.apply_index(player, revision, index).unwrap();
    act(&mut game, action);
    assert_eq!(game.snapshot(), indexed.snapshot());
    assert_eq!(game.state.players[player.index()].mana, [0, 2, 0, 0, 0, 2]);
    indexed.restore(&saved).unwrap();
    assert_eq!(
        indexed.apply_index(player, revision, index),
        Err(Error::StaleRevision)
    );
    act(
        &mut indexed,
        Action::ChooseMana {
            production: COLORLESS,
        },
    );
    assert_eq!(
        indexed.state.players[player.index()].mana,
        [0, 1, 0, 0, 0, 3]
    );
}

#[test]
fn the_conditional_land_bonus_funds_a_cast_and_disappears_with_its_provider() {
    let _rules = fixture_rules::activate(&[(Card::HillGiant, &PROVIDER)]);
    let player = Player::P0;
    let mut game = scenario(player);
    let provider = put(&mut game, player, "Hill Giant", Zone::Battlefield);
    let land = spawn(
        &mut game,
        player,
        Card::from_name("Wastes").unwrap(),
        Zone::Battlefield,
    );
    let spell = put(&mut game, player, "Swiftfoot Boots", Zone::Hand);
    game.state.players[player.index()].mana = [0; 6];
    let announce = Action::AnnounceCast { card: spell };
    let before = game.snapshot();
    let exact = rustmage_core::engine::funding::FundingGoal {
        actor: player,
        required: [0, 0, 0, 0, 0, 2],
        generic: 0,
        reserve: None,
        source: None,
    };
    assert!(game.can_fund(exact.clone()));
    assert!(game.legal_actions().contains(&announce));
    let mut without_provider = game.clone();
    without_provider.move_object(provider, Zone::Graveyard);
    assert!(!without_provider.can_fund(exact));
    reject(&mut without_provider, announce.clone());
    assert_eq!(game.snapshot(), before);

    act(&mut game, announce);
    act(&mut game, tap(land, COLORLESS));
    pay(&mut game);
    assert_eq!(game.object_view(spell).zone, Zone::Stack);
    assert_eq!(game.state.players[player.index()].mana, [0; 6]);
}

const BLIGHT_ATTACKER: CardRules = CardRules {
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::SelfAttacks,
        targets: TargetPlan::Single(TargetSpec::one(TargetRule::Object {
            zone: TargetZone::Battlefield,
            kinds: 2,
            relation: -1,
            other: false,
            nonland: false,
            tapped: false,
            min_mana: 0,
            max_mana: u32::MAX,
        })),
        program: &[
            Instruction::PutCounters {
                subject: Subject::Targets,
                kind: CounterKind::Blight,
                amount: 1,
            },
            Instruction::BlankTargetLandWhileBlighted,
        ],
    }],
    ..PROVIDER
};

fn attack(game: &mut Game, source: CardId) {
    game.state.step = Step::Attackers;
    game.state.pending = Pending::Attackers;
    act(game, Action::Attack { card: source });
    act(game, Action::FinishAttackers);
}

fn target(card: CardId) -> Action {
    Action::ChooseTarget {
        target: Target::Card(card),
    }
}

fn finish_target(game: &mut Game) {
    if game.legal_actions().contains(&Action::FinishTargets) {
        act(game, Action::FinishTargets);
    }
}

fn blighted_tap(game: &Game, land: CardId) -> Action {
    Action::TapGrantedMana {
        card: land,
        grantor: game.object_ref(land),
        production: COLORLESS,
    }
}

#[test]
fn the_exact_land_loses_earlier_land_types_and_abilities_until_its_last_counter() {
    use rustmage_core::engine::subtypes::BasicLandTypesEffectView;

    let _rules = fixture_rules::activate(&[(Card::HillGiant, &BLIGHT_ATTACKER)]);
    let player = Player::P0;
    let mut game = scenario(player);
    let source = put(&mut game, player, "Hill Giant", Zone::Battlefield);
    let town = put(&mut game, player, "Starting Town", Zone::Battlefield);
    let unrelated = put(&mut game, player, "Bear Cub", Zone::Battlefield);
    let exact = game.object_ref(town);
    let town_card = game.object_view(town).card;
    game.set_copy_recipe(
        exact,
        CopyRecipe::new(town_card).with_exceptions(CopyExceptions {
            artifact: true,
            ..CopyExceptions::NONE
        }),
    );
    let addition = BasicLandTypesEffectView {
        recipient: exact,
        source: game.object_ref(source),
        kind: BasicLandType::Forest,
        timestamp: game.next_role_timestamp(),
    };
    game.state
        .roles
        .as_mut()
        .unwrap()
        .basic_land_types
        .push(addition);
    game.add_counter(town, CounterKind::Blight, 1);
    assert!(game.object_view(town).has_subtype("Town"));
    assert!(game.object_view(town).has_subtype("Forest"));
    assert!(game.printed_abilities_available(town));

    attack(&mut game, source);
    reject(&mut game, target(unrelated));
    let chosen = target(town);
    let saved = game.snapshot();
    let revision = game.revision();
    let index = game
        .legal_actions()
        .iter()
        .position(|a| *a == chosen)
        .unwrap();
    let mut indexed = game.clone();
    indexed.apply_index(player, revision, index).unwrap();
    act(&mut game, chosen);
    assert_eq!(game.snapshot(), indexed.snapshot());
    indexed.restore(&saved).unwrap();
    assert_eq!(
        indexed.apply_index(player, revision, index),
        Err(Error::StaleRevision)
    );
    act(&mut indexed, target(town));
    assert_eq!(game.snapshot(), indexed.snapshot());
    finish_target(&mut game);

    // The attacking object can be gone when its already-stacked trigger resolves.
    game.move_object(source, Zone::Graveyard);
    settle(&mut game);
    assert_eq!(game.counter_count(town, CounterKind::Blight), 2);
    assert_eq!(game.object_view(town).kinds & (16 | 2), 16 | 2);
    assert!(!game.object_view(town).has_subtype("Town"));
    assert!(!game.object_view(town).has_subtype("Forest"));
    assert!(!game.printed_abilities_available(town));

    let marked = game.snapshot();
    game.pay_counters(town, &Counters::of(CounterKind::Blight, 1));
    assert!(!game.printed_abilities_available(town));
    let grant = blighted_tap(&game, town);
    game.state.players[player.index()].mana = [0; 6];
    act(&mut game, grant.clone());
    assert_eq!(game.state.players[player.index()].mana, COLORLESS);
    game.pay_counters(town, &Counters::of(CounterKind::Blight, 1));
    game.add_counter(town, CounterKind::Blight, 1);
    game.state.objects[town as usize].tapped = false;
    assert!(game.object_view(town).has_subtype("Town"));
    assert!(game.object_view(town).has_subtype("Forest"));
    assert!(game.printed_abilities_available(town));
    reject(&mut game, grant);

    game.restore(&marked).unwrap();
    game.move_object(town, Zone::Hand);
    game.move_object(town, Zone::Battlefield);
    game.add_counter(town, CounterKind::Blight, 1);
    assert_ne!(game.object_ref(town), exact);
    assert!(game.object_view(town).has_subtype("Town"));
    assert!(game.printed_abilities_available(town));
}

#[test]
fn a_prevented_blight_addition_uses_only_counters_already_on_the_land() {
    const LOCK: CardRules = CardRules {
        effect: Effect::Permanent,
        static_groups: &[StaticAbilityGroup {
            line: 0,
            first_layer: AbilityLayer::Rule,
            fragments: &[StaticAbilityFragment::CounterProhibition(2)],
        }],
        ..CardRules::EMPTY
    };
    let lock = Card::from_name("Swiftfoot Boots").unwrap();
    let _rules = fixture_rules::activate(&[(Card::HillGiant, &BLIGHT_ATTACKER), (lock, &LOCK)]);
    let player = Player::P1;
    for prior in [0, 1] {
        let mut game = scenario(player);
        let source = put(&mut game, player, "Hill Giant", Zone::Battlefield);
        let land = put(&mut game, player, "Forest", Zone::Battlefield);
        game.add_counter(land, CounterKind::Blight, prior);
        spawn(&mut game, player.other(), lock, Zone::Battlefield);
        attack(&mut game, source);
        act(&mut game, target(land));
        finish_target(&mut game);
        settle(&mut game);
        assert_eq!(game.counter_count(land, CounterKind::Blight), prior);
        assert!(game.object_view(land).card.is_basic_land());
        assert_eq!(game.object_view(land).has_subtype("Forest"), prior == 0);
        assert_eq!(game.printed_abilities_available(land), prior == 0);
        if prior > 0 {
            let grant = blighted_tap(&game, land);
            game.state.players[player.index()].mana = [0; 6];
            act(&mut game, grant);
            assert_eq!(game.state.players[player.index()].mana, [0, 0, 0, 0, 0, 2]);
        }
    }
}

#[test]
fn funding_cannot_revive_an_expired_blight_effect_by_replacing_the_counter() {
    use rustmage_core::engine::ability_world::CharacteristicWorld;
    use rustmage_core::engine::characteristic_layers::AnimationTypeMode;
    use rustmage_core::engine::funding::{FundingGoal, FundingState};

    let _rules = fixture_rules::activate(&[(Card::HillGiant, &BLIGHT_ATTACKER)]);
    let player = Player::P0;
    let mut game = scenario(player);
    let source = put(&mut game, player, "Hill Giant", Zone::Battlefield);
    let land = put(&mut game, player, "Forest", Zone::Battlefield);
    let exact = game.object_ref(land);
    game.change_object_form(
        exact,
        AnimationSpec {
            artifact: false,
            creature_types: AnimationCreatureTypes::Unchanged,
            base: Some((2, 2)),
            keywords: Keywords::NONE,
            duration: AnimationDuration::Permanent,
        },
        AnimationTypeMode::AddCreature,
    );
    attack(&mut game, source);
    act(&mut game, target(land));
    finish_target(&mut game);
    settle(&mut game);
    game.state.players[player.index()].mana = [0; 6];
    let goal = FundingGoal {
        actor: player,
        required: [0, 0, 0, 0, 0, 2],
        generic: 0,
        reserve: None,
        source: None,
    };
    let saved = game.snapshot();
    assert!(game.can_fund(goal.clone()));
    let mut projected = FundingState::from_game(&game, &goal);
    assert!(projected.remove_creature_counter(exact, CounterKind::Blight));
    assert_eq!(projected.intrinsic_mana_color(land), Some(4));
    assert!(projected.visit_mana_source_counter_cost(
        exact,
        Some((CounterKind::Blight, 1)),
        &mut |funding| {
            assert_eq!(funding.counter_count(land, CounterKind::Blight), 1);
            assert_eq!(funding.intrinsic_mana_color(land), Some(4));
            assert!(funding.granted_mana(land).is_empty());
            true
        },
    ));
    assert_eq!(game.snapshot(), saved);
}
