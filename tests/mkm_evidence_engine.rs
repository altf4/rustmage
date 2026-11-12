//! Casting, resolving, and branching evidence costs with exact graveyard objects.
mod support;

use rustmage as _;
use rustmage_core::cards::authored::{CardExtras, CardRules};
use rustmage_core::{cards::*, engine::*};
use support::*;

const RED: Mana = [0, 0, 0, 1, 0, 0];
const NO_FILTER: SacrificeFilter = SacrificeFilter {
    keywords: Keywords::NONE,
    source_only: false,
    kinds: 0,
    nonland: false,
    other: false,
    tokens: None,
    artifact_enchantment_or_token: false,
    subtype: None,
};
const fn up_to_one(kinds: u8) -> TargetSpec {
    TargetSpec {
        rule: TargetRule::Object {
            zone: TargetZone::Battlefield,
            kinds,
            relation: -1,
            other: false,
            nonland: false,
            tapped: false,
            min_mana: 0,
            max_mana: u32::MAX,
        },
        min: 0,
        max: 1,
    }
}
const TARGET_EVIDENCE_SPELL: CardRules = CardRules {
    effect: Effect::Spell(
        TargetPlan::Groups(&[up_to_one(16), up_to_one(1)]),
        &[Instruction::DestroyAllTargets],
    ),
    extras: &CardExtras {
        additional_cost: Some(AdditionalCost {
            kind: EffectChoiceKind::CollectEvidenceForTargetManaValue,
            filter: NO_FILTER,
        }),
        flashback_cost: Some(ManaCost {
            colored: RED,
            generic: 1,
        }),
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
const SACRIFICE_MANA: CardRules = CardRules {
    effect: Effect::Permanent,
    activated: &[ActivatedAbility {
        exhaust: false,
        guard: ActivationGuard::Always,
        cycling: false,
        origin: ActivationOrigin::Battlefield,
        cost: ActivationCost {
            sacrifice: true,
            ..ActivationCost::ZERO
        },
        targets: TargetPlan::NONE,
        program: &[Instruction::AddMana(ManaProduction::Fixed(RED))],
        sorcery: false,
        once_per_turn: false,
    }],
    ..CardRules::EMPTY
};
const NO_EXILE: CardRules = CardRules {
    effect: Effect::Permanent,
    static_groups: &[StaticAbilityGroup {
        line: 0,
        first_layer: AbilityLayer::Rule,
        fragments: &[StaticAbilityFragment::GraveyardCardsCannotBeExiled],
    }],
    ..CardRules::EMPTY
};
const COLLECT_THREE: CardRules = CardRules {
    effect: Effect::Creature(2, 2),
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::SelfEnters,
        targets: TargetPlan::NONE,
        program: &[
            Instruction::CollectEvidence {
                threshold: 3,
                skip: 1,
            },
            Instruction::Draw(PlayerSet::Controller, 1),
        ],
    }],
    ..CardRules::EMPTY
};
const COLLECT_ZERO: CardRules = CardRules {
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::SelfEnters,
        targets: TargetPlan::NONE,
        program: &[
            Instruction::CollectEvidence {
                threshold: 0,
                skip: 1,
            },
            Instruction::Draw(PlayerSet::Controller, 1),
        ],
    }],
    ..COLLECT_THREE
};

fn named(name: &str) -> Card {
    Card::from_name(name).unwrap()
}
fn choose_evidence(g: &Game, card: CardId) -> Action {
    Action::ChooseEvidence {
        object: g.object_ref(card),
    }
}
fn choose_cost(g: &mut Game, kind: CastCostClause) {
    let index = g
        .state
        .announcing
        .as_ref()
        .unwrap()
        .cast_costs
        .as_ref()
        .unwrap()
        .clauses
        .iter()
        .position(|row| row.clause == kind)
        .unwrap();
    act(
        g,
        Action::ChooseCastCost {
            clause: index.try_into().unwrap(),
        },
    );
}
fn begin_evidence_clause(g: &mut Game) {
    act(g, Action::BeginOrderedCosts);
    choose_cost(g, CastCostClause::Additional);
}

fn wait_for_evidence(g: &mut Game) {
    for _ in 0..8 {
        if matches!(g.state.pending, Pending::Evidence { .. }) {
            return;
        }
        assert_eq!(g.state.pending, Pending::Priority);
        act(g, Action::Pass);
    }
    panic!("evidence choice did not become pending");
}

#[test]
fn target_evidence_locks_before_mana_sacrifices_and_rejects_unfundable_targets() {
    let _rules = fixture_rules::activate(&[
        (named("Lightning Strike"), &TARGET_EVIDENCE_SPELL),
        (named("Candy Trail"), &SACRIFICE_MANA),
    ]);
    let player = Player::P0;
    let mut game = scenario(player);
    game.state.players[player.index()].mana = [0, 0, 0, 0, 0, 1];
    let artifact = put(&mut game, player, "Candy Trail", Zone::Battlefield);
    let creature = put(
        &mut game,
        player.other(),
        "Grizzly Bears",
        Zone::Battlefield,
    );
    let expensive = put(&mut game, player.other(), "Hill Giant", Zone::Battlefield);
    let fuel = put(&mut game, player, "Grizzly Bears", Zone::Graveyard);
    let foreign = put(&mut game, player.other(), "Hill Giant", Zone::Graveyard);
    let spell = put(&mut game, player, "Lightning Strike", Zone::Hand);

    act(&mut game, Action::AnnounceCast { card: spell });
    act(
        &mut game,
        Action::ChooseTarget {
            target: Target::Card(artifact),
        },
    );
    reject(
        &mut game,
        Action::ChooseTarget {
            target: Target::Card(expensive),
        },
    );
    act(
        &mut game,
        Action::ChooseTarget {
            target: Target::Card(creature),
        },
    );
    choose(
        &mut game,
        |action| matches!(action, Action::ActivateMana { card, .. } if *card == artifact),
    );
    assert_eq!(game.object_view(artifact).zone, Zone::Graveyard);

    begin_evidence_clause(&mut game);
    let evidence = game.observe(player).evidence.unwrap();
    assert_eq!(evidence.threshold, 3);
    assert!(game.observe(player.other()).evidence.is_none());
    let wrong_owner = choose_evidence(&game, foreign);
    reject(&mut game, wrong_owner);
    let first = choose_evidence(&game, fuel);
    act(&mut game, first);
    reject(&mut game, Action::FinishEvidence);
    let sacrificed_target = choose_evidence(&game, artifact);
    act(&mut game, sacrificed_target);
    act(&mut game, Action::FinishEvidence);
    choose_cost(&mut game, CastCostClause::Mana);
    pay(&mut game);
    assert_eq!(
        game.observe(player.other())
            .last_evidence
            .unwrap()
            .threshold,
        3
    );
    settle(&mut game);
    assert_eq!(game.object_view(creature).zone, Zone::Graveyard);
}

#[test]
fn mandatory_zero_evidence_is_paid_and_the_casting_card_cannot_be_its_own_fuel() {
    let _rules = fixture_rules::activate(&[(named("Lightning Strike"), &TARGET_EVIDENCE_SPELL)]);
    let player = Player::P1;
    let mut game = scenario(player);
    let artifact = put(&mut game, player.other(), "Candy Trail", Zone::Battlefield);
    let source = put(&mut game, player, "Lightning Strike", Zone::Graveyard);

    act(&mut game, Action::AnnounceFlashback { card: source });
    reject(
        &mut game,
        Action::ChooseTarget {
            target: Target::Card(artifact),
        },
    );
    act(&mut game, Action::FinishTargets);
    begin_evidence_clause(&mut game);
    let evidence = game.observe(player).evidence.unwrap();
    assert_eq!(evidence.threshold, 0);
    assert!(evidence.candidates.is_empty());
    reject(&mut game, Action::DeclineEvidence);
    act(&mut game, Action::FinishEvidence);
    choose_cost(&mut game, CastCostClause::Mana);
    pay(&mut game);
    let event = game.observe(player.other()).last_evidence.unwrap();
    assert_eq!(event.sequence, 1);
    assert_eq!(event.threshold, 0);
    assert!(event.exiled.is_empty());
    settle(&mut game);
    assert_eq!(game.object_view(artifact).zone, Zone::Battlefield);
    assert_eq!(game.object_view(source).zone, Zone::Exile);
}

#[test]
fn a_true_exile_prohibition_blocks_positive_evidence_but_zero_still_pays() {
    let _rules = fixture_rules::activate(&[
        (named("Grizzly Bears"), &COLLECT_THREE),
        (named("Hill Giant"), &COLLECT_ZERO),
        (named("Short Bow"), &NO_EXILE),
    ]);
    let player = Player::P0;
    let mut game = scenario(player);
    let witness = put(&mut game, player, "Hill Giant", Zone::Graveyard);
    put(&mut game, player.other(), "Short Bow", Zone::Battlefield);
    let positive = put(&mut game, player, "Grizzly Bears", Zone::Hand);
    cast(&mut game, positive, None);
    settle(&mut game);
    assert!(game.observe(player).last_evidence.is_none());
    assert_eq!(game.state.players[player.index()].cards_drawn_this_turn, 0);

    let zero = put(&mut game, player, "Hill Giant", Zone::Hand);
    cast(&mut game, zero, None);
    wait_for_evidence(&mut game);
    assert_eq!(game.observe(player).evidence.unwrap().threshold, 0);
    let forbidden = choose_evidence(&game, witness);
    reject(&mut game, forbidden);
    act(&mut game, Action::FinishEvidence);
    assert_eq!(game.state.players[player.index()].cards_drawn_this_turn, 1);
    assert_eq!(game.object_view(witness).zone, Zone::Graveyard);
    assert!(
        game.observe(player)
            .last_evidence
            .unwrap()
            .exiled
            .is_empty()
    );
}

#[test]
fn evidence_branch_owns_overpayment_and_receipts_only_include_actual_exiles() {
    struct ResetRedirect;
    impl Drop for ResetRedirect {
        fn drop(&mut self) {
            zone_replacements::TEST_REDIRECT_EXILE_TO_BATTLEFIELD.with(|value| value.set(None));
        }
    }

    let _rules = fixture_rules::activate(&[(named("Grizzly Bears"), &COLLECT_THREE)]);
    let _reset = ResetRedirect;
    let player = Player::P1;
    let mut game = scenario(player);
    let redirected = put(&mut game, player, "Hill Giant", Zone::Graveyard);
    let exiled = put(&mut game, player, "Lightning Bolt", Zone::Graveyard);
    let source = put(&mut game, player, "Grizzly Bears", Zone::Hand);
    zone_replacements::TEST_REDIRECT_EXILE_TO_BATTLEFIELD
        .with(|value| value.set(Some(game.object_ref(redirected))));
    cast(&mut game, source, None);
    wait_for_evidence(&mut game);
    assert!(game.observe(player.other()).evidence.is_none());

    let selection = choose_evidence(&game, redirected);
    let index = game
        .legal_actions()
        .iter()
        .position(|action| *action == selection)
        .unwrap();
    let revision = game.revision();
    let snapshot = game.snapshot();
    let mut raw = game.clone();
    let mut indexed = game.clone();
    let mut restored = game.clone();
    act(&mut restored, Action::DeclineEvidence);
    restored.restore(&snapshot).unwrap();
    drop(game);

    act(&mut raw, selection.clone());
    indexed.apply_index(player, revision, index).unwrap();
    act(&mut restored, selection.clone());
    assert_eq!(raw.snapshot(), indexed.snapshot());
    assert_eq!(raw.snapshot(), restored.snapshot());
    assert_eq!(raw.object_view(redirected).zone, Zone::Graveyard);
    reject(&mut raw, selection);

    let overpay = choose_evidence(&raw, exiled);
    act(&mut raw, overpay);
    act(&mut raw, Action::FinishEvidence);
    act(&mut indexed, Action::FinishEvidence);
    act(&mut restored, Action::DeclineEvidence);
    let receipt = raw.observe(player.other()).last_evidence.unwrap();
    assert_eq!(receipt.threshold, 3);
    assert_eq!(receipt.total_mana, 5);
    assert_eq!(receipt.exiled, vec![raw.object_ref(exiled)]);
    assert_eq!(raw.object_view(redirected).zone, Zone::Battlefield);
    assert_eq!(indexed.object_view(exiled).zone, Zone::Graveyard);
    assert!(
        indexed
            .observe(player)
            .last_evidence
            .unwrap()
            .exiled
            .is_empty()
    );
    assert!(restored.observe(player).last_evidence.is_none());
}
