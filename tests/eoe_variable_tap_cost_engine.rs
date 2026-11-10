//! Nonmana X is declared against complete projected mana and tap payments.
mod support;
use rustmage as _;
use rustmage_core::cards::authored::{CardRules, one};
use rustmage_core::{cards::*, engine::*};
use support::*;

const STARFORGE: ActivatedAbility = ActivatedAbility {
    cost: ActivationCost {
        generic: 2,
        mana_component: true,
        tap: true,
        additional: Some(AdditionalCost {
            kind: EffectChoiceKind::TapXPermanents,
            filter: SacrificeFilter {
                keywords: Keywords::NONE,
                source_only: false,
                kinds: 16,
                nonland: false,
                other: false,
                tokens: None,
                artifact_enchantment_or_token: false,
                subtype: None,
            },
        }),
        ..ActivationCost::ZERO
    },
    targets: one(TargetRule::Creature),
    program: &[Instruction::GrowScaled {
        subject: Subject::Targets,
        amount: AmountValue::ChosenX,
        power: 1,
        toughness: 0,
    }],
    guard: ActivationGuard::Always,
    origin: ActivationOrigin::Battlefield,
    exhaust: false,
    cycling: false,
    sorcery: true,
    once_per_turn: false,
};
const COLORLESS: Mana = [0, 0, 0, 0, 0, 1];
const TAP_MANA: ActivatedAbility = ActivatedAbility {
    cost: ActivationCost {
        tap: true,
        ..ActivationCost::ZERO
    },
    targets: TargetPlan::NONE,
    program: &[Instruction::AddMana(ManaProduction::Fixed(COLORLESS))],
    sorcery: false,
    ..STARFORGE
};
const SOURCE: CardRules = CardRules {
    effect: Effect::Permanent,
    activated: &[STARFORGE, TAP_MANA],
    ..CardRules::EMPTY
};
const FUNDER: CardRules = CardRules {
    activated: &[TAP_MANA],
    ..SOURCE
};
const ARTIFACT_CREATURE: CardRules = CardRules {
    effect: Effect::Creature(1, 1),
    ..CardRules::EMPTY
};

fn activate(card: CardId) -> Action {
    Action::AnnounceActivate {
        card,
        ability: 0,
        counter_payment: Counters::default(),
    }
}

fn tap_cost(game: &Game, card: CardId) -> Action {
    Action::ChooseAdditionalCost {
        object: game.object_ref(card),
    }
}

fn target(game: &mut Game, card: CardId) {
    act(
        game,
        Action::ChooseTarget {
            target: Target::Card(card),
        },
    );
}

fn open_group(game: &mut Game) {
    let index = game
        .state
        .announcing
        .as_ref()
        .unwrap()
        .cast_costs
        .as_ref()
        .unwrap()
        .clauses
        .iter()
        .position(|row| row.clause == CastCostClause::Additional && !row.paid)
        .unwrap();
    act(
        game,
        Action::ChooseCastCost {
            clause: index as u16,
        },
    );
}

#[test]
fn x_projects_artifact_mana_and_reserves_the_activated_lands_own_tap() {
    let land = Card::from_name("Scene of the Crime").unwrap();
    let bow = Card::from_name("Short Bow").unwrap();
    let _rules = fixture_rules::activate(&[(land, &SOURCE), (bow, &FUNDER)]);
    let player = Player::P0;
    let mut game = scenario(player);
    let source = spawn(&mut game, player, land, Zone::Battlefield);
    let funder = put(&mut game, player, "Short Bow", Zone::Battlefield);
    let artifact = put(&mut game, player, "Candy Trail", Zone::Battlefield);
    let tapped = put(&mut game, player, "Candy Trail", Zone::Battlefield);
    let foreign = put(&mut game, player.other(), "Candy Trail", Zone::Battlefield);
    let creature = put(&mut game, player, "Goblin Piker", Zone::Battlefield);
    game.state.objects[tapped as usize].tapped = true;
    game.state.players[player.index()].mana = COLORLESS;
    game.state.step = Step::Upkeep;
    reject(&mut game, activate(source));
    game.state.step = Step::Main1;

    act(&mut game, activate(source));
    assert_eq!(
        game.x_choice().unwrap().intervals,
        vec![XInterval {
            minimum: 0,
            maximum: 1
        }]
    );
    assert!(game.observe(player.other()).x_choice.is_none());
    reject(&mut game, Action::ChooseX { value: 2 });
    let initial = game.snapshot();
    let mut zero = game.clone();
    let mut one = game.clone();
    let one_action = Action::ChooseX { value: 1 };
    let index = game
        .legal_actions()
        .iter()
        .position(|action| *action == one_action)
        .unwrap();
    one.apply_index(player, one.revision(), index).unwrap();
    act(&mut zero, Action::ChooseX { value: 0 });
    assert_eq!(game.snapshot(), initial);

    for branch in [&mut zero, &mut one] {
        target(branch, creature);
        assert!(!branch.legal_actions().iter().any(|action| match action {
            Action::TapMana { card }
            | Action::ActivateMana { card, .. }
            | Action::AnnounceMana { card, .. } => *card == source,
            _ => false,
        }));
        act(
            branch,
            Action::ActivateMana {
                card: funder,
                ability: 0,
                payment: [0; 6],
                counter_payment: Counters::default(),
                production: COLORLESS,
            },
        );
        pay(branch);
    }
    assert!(zero.state.announcing.is_none());
    open_group(&mut one);
    for invalid in [source, funder, tapped, foreign, creature] {
        let choice = tap_cost(&one, invalid);
        reject(&mut one, choice);
    }
    let choice = tap_cost(&one, artifact);
    act(&mut one, choice);
    for branch in [&mut zero, &mut one] {
        assert!(branch.object_view(source).tapped);
        assert!(branch.object_view(funder).tapped);
        settle(branch);
    }
    assert!(!zero.object_view(artifact).tapped);
    assert!(one.object_view(artifact).tapped);
    assert_eq!(stats(&zero, creature), (2, 1));
    assert_eq!(stats(&one, creature), (3, 1));
}

#[test]
fn the_group_requires_distinct_artifacts_and_can_tap_a_sick_target() {
    let land = Card::from_name("Scene of the Crime").unwrap();
    let artifact_creature = Card::from_name("Gingerbrute").unwrap();
    let _rules =
        fixture_rules::activate(&[(land, &SOURCE), (artifact_creature, &ARTIFACT_CREATURE)]);
    let player = Player::P1;
    let mut game = scenario(player);
    let source = spawn(&mut game, player, land, Zone::Battlefield);
    let target_creature = spawn(&mut game, player, artifact_creature, Zone::Battlefield);
    let other = put(&mut game, player, "Candy Trail", Zone::Battlefield);
    game.state.objects[target_creature as usize].summoning_sick = true;
    game.state.players[player.index()].mana = [0, 0, 0, 0, 0, 2];
    act(&mut game, activate(source));
    assert_eq!(game.x_choice().unwrap().intervals[0].maximum, 2);
    act(&mut game, Action::ChooseX { value: 2 });
    target(&mut game, target_creature);
    pay(&mut game);
    open_group(&mut game);
    let first = tap_cost(&game, target_creature);
    act(&mut game, first.clone());
    assert!(!game.object_view(target_creature).tapped);
    reject(&mut game, first);
    let second = tap_cost(&game, other);
    act(&mut game, second);
    assert!(game.object_view(target_creature).tapped);
    assert!(game.object_view(other).tapped);
    assert!(game.object_view(source).tapped);
    settle(&mut game);
    assert_eq!(stats(&game, target_creature), (3, 1));
}
