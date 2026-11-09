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
