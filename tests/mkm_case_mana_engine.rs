//! Case spell mana flexibility preserves costs, restrictions, and actual mana.
mod support;

use rustmage as _;
use rustmage_core::cards::authored::{CardExtras, CardRules};
use rustmage_core::{cards::*, engine::*};
use support::*;

const AUDITOR: Card = Card::from_catalogue_id(660); // Case File Auditor
const CASE: Card = Card::from_catalogue_id(664); // Case of the Gateway Express, {1}{W}
const GREEN_TWO: Mana = [0, 0, 0, 0, 2, 0];
const ARTIFACT_MANA: Mana = [1, 0, 0, 0, 0, 1];
const AUDITOR_RULES: CardRules = CardRules {
    effect: Effect::Creature(1, 4),
    static_groups: &[StaticAbilityGroup {
        line: 0,
        first_layer: AbilityLayer::Rule,
        fragments: &[StaticAbilityFragment::SpendAnyColorForSpellSubtype("Case")],
    }],
    ..CardRules::EMPTY
};
const CASE_RULES: CardRules = CardRules {
    effect: Effect::Permanent,
    ..CardRules::EMPTY
};
const KICKED_CASE: CardRules = CardRules {
    extras: &CardExtras {
        kicker_cost: Some(ManaCost {
            colored: [0, 0, 0, 0, 0, 1],
            generic: 0,
        }),
        ..CardExtras::EMPTY
    },
    ..CASE_RULES
};
const SACRIFICE_MANA: ActivatedAbility = ActivatedAbility {
    exhaust: false,
    guard: ActivationGuard::Always,
    origin: ActivationOrigin::Battlefield,
    cycling: false,
    cost: ActivationCost {
        sacrifice: true,
        ..ActivationCost::ZERO
    },
    targets: TargetPlan::NONE,
    program: &[Instruction::AddMana(ManaProduction::Fixed(GREEN_TWO))],
    sorcery: false,
    once_per_turn: false,
};
const SACRIFICIAL_AUDITOR: CardRules = CardRules {
    activated: &[SACRIFICE_MANA],
    ..AUDITOR_RULES
};
const CASE_MANA_SOURCE: CardRules = CardRules {
    activated: &[ActivatedAbility {
        cost: ActivationCost {
            tap: true,
            ..ActivationCost::ZERO
        },
        program: &[Instruction::AddMana(ManaProduction::Tagged {
            production: &ManaProduction::Fixed(GREEN_TWO),
            quality: &ManaQuality {
                restriction: ManaRestriction::SpellSubtype("Case"),
                spell_bonus: ManaSpellBonus::None,
                retention: ManaRetention::Step,
            },
        })],
        ..SACRIFICE_MANA
    }],
    ..CASE_RULES
};
const ARTIFACT_MANA_SOURCE: CardRules = CardRules {
    activated: &[ActivatedAbility {
        cost: ActivationCost {
            tap: true,
            ..ActivationCost::ZERO
        },
        program: &[Instruction::AddMana(ManaProduction::Tagged {
            production: &ManaProduction::Fixed(ARTIFACT_MANA),
            quality: &ManaQuality {
                restriction: ManaRestriction::ArtifactSpell,
                spell_bonus: ManaSpellBonus::None,
                retention: ManaRetention::Step,
            },
        })],
        ..SACRIFICE_MANA
    }],
    ..CASE_RULES
};

fn cast_action(card: CardId, payment: Mana) -> Action {
    Action::Cast {
        card,
        target: None,
        payment,
    }
}

fn mana_action(card: CardId, production: Mana) -> Action {
    Action::ActivateMana {
        card,
        ability: 0,
        payment: [0; 6],
        counter_payment: Counters::default(),
        production,
    }
}

#[test]
fn cast_only_color_permission_follows_provider_control_and_records_actual_mana() {
    let _rules = fixture_rules::activate(&[(AUDITOR, &AUDITOR_RULES), (CASE, &CASE_RULES)]);
    for player in [Player::P0, Player::P1] {
        let mut game = scenario(player);
        let auditor = spawn(&mut game, player.other(), AUDITOR, Zone::Battlefield);
        let case = spawn(&mut game, player, CASE, Zone::Hand);
        let noncase = put(&mut game, player, "Savannah Lions", Zone::Hand);
        game.state.players[player.index()].mana = GREEN_TWO;

        reject(&mut game, Action::AnnounceCast { card: case });
        reject(&mut game, cast_action(case, GREEN_TWO));
        game.record_control(game.object_ref(auditor), player, None);
        reject(&mut game, Action::AnnounceCast { card: noncase });

        act(&mut game, cast_action(case, GREEN_TWO));
        let cast = game.state.stack.last().unwrap().cast.unwrap();
        assert_eq!(cast.payment, GREEN_TWO);
        assert!(!cast.spend_any_mana);
        settle(&mut game);
        assert_eq!(game.object_view(case).zone, Zone::Battlefield);
    }
}

#[test]
fn a_colorless_kicker_keeps_its_colorless_requirement() {
    let _rules = fixture_rules::activate(&[(AUDITOR, &AUDITOR_RULES), (CASE, &KICKED_CASE)]);
    let player = Player::P0;
    let mut game = scenario(player);
    spawn(&mut game, player, AUDITOR, Zone::Battlefield);
    let case = spawn(&mut game, player, CASE, Zone::Hand);
    let all_green = [0, 0, 0, 0, 3, 0];
    game.state.players[player.index()].mana = all_green;
    let mut no_colorless = game.clone();
    act(&mut no_colorless, Action::AnnounceCast { card: case });
    reject(&mut no_colorless, Action::ChooseKicker { pay: true });
    act(&mut no_colorless, Action::ChooseKicker { pay: false });
    act(&mut no_colorless, Action::PayCosts { payment: GREEN_TWO });

    let payment = [0, 0, 0, 0, 2, 1];
    game.state.players[player.index()].mana = [0, 0, 0, 0, 3, 1];
    act(&mut game, Action::AnnounceCast { card: case });
    act(&mut game, Action::ChooseKicker { pay: true });
    reject(&mut game, Action::PayCosts { payment: all_green });
    act(&mut game, Action::PayCosts { payment });
    let cast = game.state.stack.last().unwrap().cast.unwrap();
    assert_eq!(cast.kicker, Some(true));
    assert_eq!(cast.payment, payment);

    let mut only_colorless = scenario(player);
    spawn(&mut only_colorless, player, AUDITOR, Zone::Battlefield);
    let case = spawn(&mut only_colorless, player, CASE, Zone::Hand);
    let payment = [0, 0, 0, 0, 0, 3];
    only_colorless.state.players[player.index()].mana = payment;
    act(&mut only_colorless, Action::AnnounceCast { card: case });
    act(&mut only_colorless, Action::ChooseKicker { pay: true });
    act(&mut only_colorless, Action::PayCosts { payment });
    assert_eq!(
        only_colorless
            .state
            .stack
            .last()
            .unwrap()
            .cast
            .unwrap()
            .payment,
        payment
    );
}

#[test]
fn projected_case_mana_funds_the_spell_but_artifact_only_mana_does_not() {
    let case_producer = Card::from_name("Thinking Cap").unwrap();
    let artifact_producer = Card::from_name("Magnifying Glass").unwrap();
    let _rules = fixture_rules::activate(&[
        (AUDITOR, &AUDITOR_RULES),
        (CASE, &CASE_RULES),
        (case_producer, &CASE_MANA_SOURCE),
        (artifact_producer, &ARTIFACT_MANA_SOURCE),
    ]);
    let player = Player::P1;
    let mut game = scenario(player);
    game.state.players[player.index()].mana = [0; 6];
    spawn(&mut game, player, AUDITOR, Zone::Battlefield);
    let case = spawn(&mut game, player, CASE, Zone::Hand);
    let artifact_source = spawn(&mut game, player, artifact_producer, Zone::Battlefield);
    act(&mut game, mana_action(artifact_source, ARTIFACT_MANA));
    reject(&mut game, Action::AnnounceCast { card: case });
    reject(&mut game, cast_action(case, ARTIFACT_MANA));

    let case_source = spawn(&mut game, player, case_producer, Zone::Battlefield);
    let mut direct = game.clone();
    act(&mut direct, mana_action(case_source, GREEN_TWO));
    reject(&mut direct, cast_action(case, ARTIFACT_MANA));
    act(&mut direct, cast_action(case, GREEN_TWO));

    act(&mut game, Action::AnnounceCast { card: case });
    act(&mut game, mana_action(case_source, GREEN_TWO));
    reject(
        &mut game,
        Action::PayCosts {
            payment: ARTIFACT_MANA,
        },
    );
    act(&mut game, Action::PayCosts { payment: GREEN_TWO });
    for branch in [&game, &direct] {
        assert_eq!(
            branch.state.stack.last().unwrap().cast.unwrap().payment,
            GREEN_TWO
        );
        assert_eq!(branch.state.players[player.index()].mana, ARTIFACT_MANA);
        let lots = branch.observe(player).mana_lots;
        assert_eq!(lots.len(), 1);
        assert_eq!(lots[0].restriction, ManaRestrictionView::ArtifactSpell);
        assert_eq!(lots[0].amounts, ARTIFACT_MANA);
    }
}

#[test]
fn sacrificing_the_permission_provider_cannot_fund_a_missing_colored_pip() {
    let _rules = fixture_rules::activate(&[(AUDITOR, &SACRIFICIAL_AUDITOR), (CASE, &CASE_RULES)]);
    let player = Player::P0;
    let mut game = scenario(player);
    game.state.players[player.index()].mana = [0; 6];
    let source = spawn(&mut game, player, AUDITOR, Zone::Battlefield);
    let case = spawn(&mut game, player, CASE, Zone::Hand);
    reject(&mut game, Action::AnnounceCast { card: case });

    game.state.players[player.index()].mana = [1, 0, 0, 0, 0, 0];
    act(&mut game, Action::AnnounceCast { card: case });
    let action = mana_action(source, GREEN_TWO);
    let index = game
        .legal_actions()
        .iter()
        .position(|a| *a == action)
        .unwrap();
    let saved = game.snapshot();
    let mut indexed = game.clone();
    indexed.apply_index(player, game.revision(), index).unwrap();
    act(&mut game, action);
    for branch in [&mut game, &mut indexed] {
        assert_eq!(branch.object_view(source).zone, Zone::Graveyard);
        reject(branch, Action::PayCosts { payment: GREEN_TWO });
        act(
            branch,
            Action::PayCosts {
                payment: [1, 0, 0, 0, 1, 0],
            },
        );
    }
    assert_eq!(game.snapshot(), indexed.snapshot());
    indexed.restore(&saved).unwrap();
    assert_eq!(indexed.object_view(source).zone, Zone::Battlefield);
    assert_eq!(indexed.snapshot(), saved);
}
