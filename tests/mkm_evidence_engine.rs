//! Casting, resolving, and branching evidence costs with exact graveyard objects.
mod support;

use rustmage::PaidCastOrigin;
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
    Card::from_name(name).unwrap_or_else(|| panic!("unknown fixture card {name}"))
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

const EVIDENCE_ALTERNATIVE: CardRules = CardRules {
    effect: Effect::Permanent,
    static_groups: &[StaticAbilityGroup {
        line: 0,
        first_layer: AbilityLayer::Rule,
        fragments: &[StaticAbilityFragment::SpellAlternativeEvidenceCost(10)],
    }],
    ..CardRules::EMPTY
};

const OPTIONAL_EVIDENCE: CardRules = CardRules {
    effect: Effect::Spell(
        TargetPlan::NONE,
        &[
            Instruction::IfEvidence { skip: 1 },
            Instruction::Draw(PlayerSet::Controller, 1),
        ],
    ),
    extras: &CardExtras {
        collect_evidence: Some((6, 0)),
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};

const EVIDENCE_WATCHER: CardRules = CardRules {
    effect: Effect::Permanent,
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::EvidenceCollected,
        targets: TargetPlan::NONE,
        program: &[Instruction::Draw(PlayerSet::Controller, 1)],
    }],
    ..CardRules::EMPTY
};

const DISCARD_SPELL: CardRules = CardRules {
    effect: Effect::Spell(TargetPlan::NONE, &[]),
    extras: &CardExtras {
        additional_cost: Some(AdditionalCost {
            kind: EffectChoiceKind::Discard,
            filter: NO_FILTER,
        }),
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};

const GRAVEYARD_TAX: CardRules = CardRules {
    effect: Effect::Permanent,
    static_groups: &[StaticAbilityGroup {
        line: 0,
        first_layer: AbilityLayer::Rule,
        fragments: &[StaticAbilityFragment::OpponentGraveyardOrExileSpellTax(2)],
    }],
    ..CardRules::EMPTY
};

fn alternative(game: &Game, spell: CardId, provider: ObjectRef, origin: PaidCastOrigin) -> Action {
    Action::AnnounceCastWithEvidenceAlternative {
        object: game.object_ref(spell),
        provider,
        face: CastFace::Normal,
        threshold: 10,
        flash_surcharge: false,
        origin,
    }
}

fn select_evidence(game: &mut Game, cards: &[CardId]) {
    for &card in cards {
        let choice = choose_evidence(game, card);
        act(game, choice);
    }
    act(game, Action::FinishEvidence);
}

fn ten_mana_witness(game: &mut Game, player: Player) -> CardId {
    spawn(game, player, named("Omniscience"), Zone::Graveyard)
}

#[test]
fn the_hand_alternative_pays_zero_mana_and_fixes_printed_x_at_zero() {
    let _rules = fixture_rules::activate(&[(named("Anthem of Champions"), &EVIDENCE_ALTERNATIVE)]);
    let player = Player::P0;
    let mut game = scenario(player);
    let spell = put(&mut game, player, "Wildwood Scourge", Zone::Hand);
    let source = put(&mut game, player, "Anthem of Champions", Zone::Battlefield);
    let provider = game.object_ref(source);
    let choice = alternative(&game, spell, provider, PaidCastOrigin::Ordinary);
    game.state.players[player.index()].mana = [0; 6];
    reject(&mut game, choice.clone());
    let fuel = ten_mana_witness(&mut game, player);

    act(&mut game, choice);
    reject(&mut game, Action::ChooseX { value: 1 });
    act(&mut game, Action::BeginOrderedCosts);
    choose_cost(&mut game, CastCostClause::OriginEvidence(10));
    assert!(game.observe(player.other()).evidence.is_none());
    select_evidence(&mut game, &[fuel]);
    choose_cost(&mut game, CastCostClause::Mana);
    pay(&mut game);

    let item = game.state.stack.last().unwrap();
    assert_eq!(item.x, Some(0));
    assert_eq!(item.mana_value(), 1);
    assert_eq!(item.cast.unwrap().payment, [0; 6]);
    let event = game.observe(player.other()).last_evidence.unwrap();
    assert_eq!(event.source, game.object_ref(spell));
    assert_eq!(event.threshold, 10);
    assert_eq!(event.exiled, vec![game.object_ref(fuel)]);
}

#[test]
fn the_alternative_requires_independent_graveyard_and_exile_permissions_and_pays_taxes() {
    let _rules = fixture_rules::activate(&[
        (named("Anthem of Champions"), &EVIDENCE_ALTERNATIVE),
        (named("Short Bow"), &GRAVEYARD_TAX),
    ]);
    let player = Player::P1;
    for origin in [Zone::Graveyard, Zone::Exile] {
        let mut game = scenario(player);
        let provider = put(&mut game, player, "Anthem of Champions", Zone::Battlefield);
        put(&mut game, player.other(), "Short Bow", Zone::Battlefield);
        let owner = if origin == Zone::Exile {
            player.other()
        } else {
            player
        };
        let spell = put(&mut game, owner, "Bear Cub", origin);
        let reference = game.object_ref(spell);
        let choice = alternative(
            &game,
            spell,
            game.object_ref(provider),
            PaidCastOrigin::Ordinary,
        );
        let fuel = [
            put(&mut game, player, "Hill Giant", Zone::Graveyard),
            put(&mut game, player, "Hill Giant", Zone::Graveyard),
            put(&mut game, player, "Lightning Bolt", Zone::Graveyard),
        ];
        game.state.players[player.index()].mana = [0, 0, 0, 0, 0, 2];
        reject(&mut game, choice.clone());
        if origin == Zone::Graveyard {
            game.grant_graveyard_permission(player, reference, GraveyardCastKind::Cast);
        } else {
            game.grant_exile_play_permission(player, reference, false);
        }
        reject(&mut game, choice.clone());

        let last_fuel = put(&mut game, player, "Lightning Bolt", Zone::Graveyard);
        game.state.players[player.index()].mana = [0; 6];
        reject(&mut game, choice.clone());
        game.state.players[player.index()].mana = [0, 0, 0, 0, 0, 2];
        act(&mut game, choice);
        act(&mut game, Action::BeginOrderedCosts);
        choose_cost(&mut game, CastCostClause::OriginEvidence(10));
        reject(&mut game, Action::ChooseEvidence { object: reference });
        select_evidence(&mut game, &[fuel[0], fuel[1], fuel[2], last_fuel]);
        choose_cost(&mut game, CastCostClause::Mana);
        pay(&mut game);

        let cast = game.state.stack.last().unwrap().cast.unwrap();
        assert_eq!(cast.origin, origin);
        assert_eq!(cast.caster, player);
        assert_eq!(cast.payment, [0, 0, 0, 0, 0, 2]);
        assert_eq!(game.object_view(spell).owner, owner);
    }
}

#[test]
fn the_printed_discard_can_fund_the_later_alternative_evidence_clause() {
    let _rules = fixture_rules::activate(&[
        (named("Anthem of Champions"), &EVIDENCE_ALTERNATIVE),
        (named("Quick Study"), &DISCARD_SPELL),
    ]);
    let player = Player::P0;
    let mut game = scenario(player);
    let provider = put(&mut game, player, "Anthem of Champions", Zone::Battlefield);
    let spell = put(&mut game, player, "Quick Study", Zone::Hand);
    let choice = alternative(
        &game,
        spell,
        game.object_ref(provider),
        PaidCastOrigin::Ordinary,
    );
    let low = put(&mut game, player, "Grizzly Bears", Zone::Hand);
    reject(&mut game, choice.clone());
    let discarded = spawn(&mut game, player, named("Omniscience"), Zone::Hand);

    act(&mut game, choice);
    act(&mut game, Action::BeginOrderedCosts);
    assert!(!game.legal_actions().iter().any(|action| {
        let Action::ChooseCastCost { clause } = action else {
            return false;
        };
        game.state
            .announcing
            .as_ref()
            .unwrap()
            .cast_costs
            .as_ref()
            .unwrap()
            .clauses[usize::from(*clause)]
        .clause
            == CastCostClause::OriginEvidence(10)
    }));
    choose_cost(&mut game, CastCostClause::Additional);
    let object = game.object_ref(low);
    reject(&mut game, Action::ChooseAdditionalCost { object });
    let object = game.object_ref(discarded);
    act(&mut game, Action::ChooseAdditionalCost { object });
    assert_eq!(game.object_view(discarded).zone, Zone::Graveyard);
    choose_cost(&mut game, CastCostClause::OriginEvidence(10));
    select_evidence(&mut game, &[discarded]);
    choose_cost(&mut game, CastCostClause::Mana);
    pay(&mut game);
    assert_eq!(game.object_view(discarded).zone, Zone::Exile);
    assert_eq!(game.object_view(low).zone, Zone::Hand);
}

#[test]
fn printed_and_alternative_evidence_use_disjoint_cards_and_commit_two_ordered_events() {
    let _rules = fixture_rules::activate(&[
        (named("Anthem of Champions"), &EVIDENCE_ALTERNATIVE),
        (named("Quick Study"), &OPTIONAL_EVIDENCE),
        (named("Short Bow"), &EVIDENCE_WATCHER),
    ]);
    let player = Player::P0;
    let mut game = scenario(player);
    let provider = put(&mut game, player, "Anthem of Champions", Zone::Battlefield);
    put(&mut game, player, "Short Bow", Zone::Battlefield);
    let spell = put(&mut game, player, "Quick Study", Zone::Hand);
    let choice = alternative(
        &game,
        spell,
        game.object_ref(provider),
        PaidCastOrigin::Ordinary,
    );
    let large = ten_mana_witness(&mut game, player);
    let first = put(&mut game, player, "Hill Giant", Zone::Graveyard);
    let mut insufficient = game.clone();
    act(&mut insufficient, choice.clone());
    reject(
        &mut insufficient,
        Action::ChooseCollectEvidence { collect: true },
    );
    let second = put(&mut game, player, "Hill Giant", Zone::Graveyard);
    act(&mut game, choice);
    let mut one_event = game.clone();
    act(
        &mut one_event,
        Action::ChooseCollectEvidence { collect: false },
    );
    act(&mut one_event, Action::BeginOrderedCosts);
    choose_cost(&mut one_event, CastCostClause::OriginEvidence(10));
    select_evidence(&mut one_event, &[large]);
    choose_cost(&mut one_event, CastCostClause::Mana);
    pay(&mut one_event);
    settle(&mut one_event);
    assert_eq!(
        one_event.state.players[player.index()].cards_drawn_this_turn,
        1
    );

    act(&mut game, Action::ChooseCollectEvidence { collect: true });
    act(&mut game, Action::BeginOrderedCosts);
    let mut reverse = game.clone();
    let reference = game.object_ref(large);

    choose_cost(&mut game, CastCostClause::OriginEvidence(10));
    let first_choice = choose_evidence(&game, first);
    reject(&mut game, first_choice);
    let large_choice = Action::ChooseEvidence { object: reference };
    let index = game
        .legal_actions()
        .iter()
        .position(|a| *a == large_choice)
        .unwrap();
    let revision = game.revision();
    let saved = game.snapshot();
    let mut indexed = game.clone();
    act(&mut game, large_choice.clone());
    indexed.apply_index(player, revision, index).unwrap();
    assert_eq!(indexed.snapshot(), game.snapshot());
    indexed.restore(&saved).unwrap();
    assert_eq!(
        indexed.apply_index(player, revision, index),
        Err(Error::StaleRevision)
    );
    act(&mut indexed, large_choice);
    assert_eq!(indexed.snapshot(), game.snapshot());

    act(&mut game, Action::FinishEvidence);
    let event = game.observe(player.other()).last_evidence.unwrap();
    assert_eq!((event.sequence, event.threshold), (1, 10));
    assert_eq!(event.exiled, vec![game.object_ref(large)]);
    choose_cost(&mut game, CastCostClause::Additional);
    assert!(!game.observe(player).evidence.unwrap().paid);
    reject(&mut game, Action::ChooseEvidence { object: reference });
    select_evidence(&mut game, &[first, second]);
    let event = game.observe(player.other()).last_evidence.unwrap();
    assert_eq!((event.sequence, event.threshold), (2, 6));
    assert_eq!(
        event.exiled,
        vec![game.object_ref(first), game.object_ref(second)]
    );

    choose_cost(&mut reverse, CastCostClause::Additional);
    reject(&mut reverse, Action::ChooseEvidence { object: reference });
    select_evidence(&mut reverse, &[first, second]);
    let event = reverse.observe(player.other()).last_evidence.unwrap();
    assert_eq!((event.sequence, event.threshold), (1, 6));
    choose_cost(&mut reverse, CastCostClause::OriginEvidence(10));
    assert!(!reverse.observe(player).evidence.unwrap().paid);
    select_evidence(&mut reverse, &[large]);
    let event = reverse.observe(player.other()).last_evidence.unwrap();
    assert_eq!((event.sequence, event.threshold), (2, 10));

    for branch in [&mut game, &mut reverse] {
        choose_cost(branch, CastCostClause::Mana);
        pay(branch);
        assert_eq!(branch.state.pending, Pending::OrderTriggers { player });
        choose(branch, |action| matches!(action, Action::PutTrigger { .. }));
        settle(branch);
        assert_eq!(
            branch.state.players[player.index()].cards_drawn_this_turn,
            3
        );
    }
}

#[test]
fn a_paid_cast_during_resolution_accepts_the_alternative() {
    const PAID_OFFER: CardRules = CardRules {
        effect: Effect::Spell(TargetPlan::NONE, &[Instruction::ExileTopMayCastPaid]),
        ..CardRules::EMPTY
    };
    let _rules = fixture_rules::activate(&[
        (named("Anthem of Champions"), &EVIDENCE_ALTERNATIVE),
        (named("Opt"), &PAID_OFFER),
    ]);
    let player = Player::P0;
    let mut game = scenario(player);
    let provider = put(&mut game, player, "Anthem of Champions", Zone::Battlefield);
    let spell = put(&mut game, player, "Quick Study", Zone::Library);
    let fuel = ten_mana_witness(&mut game, player);
    let offer = put(&mut game, player, "Opt", Zone::Hand);
    cast(&mut game, offer, None);
    act(&mut game, Action::Pass);
    act(&mut game, Action::Pass);
    assert_eq!(game.state.pending, Pending::ResolutionCasts { player });
    game.state.players[player.index()].mana = [0; 6];

    let provider = game.object_ref(provider);
    let ordinary = alternative(&game, spell, provider, PaidCastOrigin::Ordinary);
    reject(&mut game, ordinary);
    let choice = alternative(&game, spell, provider, PaidCastOrigin::PaidResolution);
    act(&mut game, choice);
    act(&mut game, Action::BeginOrderedCosts);
    choose_cost(&mut game, CastCostClause::OriginEvidence(10));
    select_evidence(&mut game, &[fuel]);
    choose_cost(&mut game, CastCostClause::Mana);
    pay(&mut game);
    if matches!(game.state.pending, Pending::ResolutionCasts { .. }) {
        act(&mut game, Action::FinishResolutionCasts);
    }
    settle(&mut game);
    assert_eq!(game.object_view(spell).zone, Zone::Graveyard);
    assert_eq!(game.state.players[player.index()].cards_drawn_this_turn, 2);
}

const BLUE: Mana = [0, 1, 0, 0, 0, 0];
const GREEN: Mana = [0, 0, 0, 0, 1, 0];
const CRYPTEX_MANA: CardRules = CardRules {
    effect: Effect::Permanent,
    activated: &[ActivatedAbility {
        exhaust: false,
        guard: ActivationGuard::Always,
        cycling: false,
        origin: ActivationOrigin::Battlefield,
        cost: ActivationCost {
            tap: true,
            additional: Some(AdditionalCost {
                kind: EffectChoiceKind::CollectEvidence(3),
                filter: NO_FILTER,
            }),
            ..ActivationCost::ZERO
        },
        targets: TargetPlan::NONE,
        program: &[
            Instruction::AddMana(ManaProduction::Choice {
                mask: 31,
                amount: 1,
            }),
            Instruction::PutCounters {
                subject: Subject::Source,
                kind: CounterKind::Unlock,
                amount: 1,
            },
        ],
        sorcery: false,
        once_per_turn: false,
    }],
    ..CardRules::EMPTY
};
const PARENT_EVIDENCE: CardRules = CardRules {
    extras: &CardExtras {
        collect_evidence: Some((3, 0)),
        ..CardExtras::EMPTY
    },
    ..OPTIONAL_EVIDENCE
};

fn announce_evidence_mana(source: CardId) -> Action {
    Action::AnnounceMana {
        card: source,
        ability: 0,
        counter_payment: Counters::default(),
        production: [0; 6],
    }
}

#[test]
fn evidence_mana_commits_exiles_before_color_and_unlock_replacement_choices() {
    let _rules = fixture_rules::activate(&[
        (named("Soul-Guide Lantern"), &CRYPTEX_MANA),
        (named("Short Bow"), &NO_EXILE),
        (named("Candy Trail"), &EVIDENCE_WATCHER),
    ]);
    let player = Player::P1;
    let mut game = scenario(player);
    game.state.players[player.index()].mana = [0; 6];
    let source = put(&mut game, player, "Soul-Guide Lantern", Zone::Battlefield);
    let choice = announce_evidence_mana(source);
    reject(&mut game, choice.clone());

    let fuel = put(&mut game, player, "Hill Giant", Zone::Graveyard);
    let mut prohibited = game.clone();
    put(
        &mut prohibited,
        player.other(),
        "Short Bow",
        Zone::Battlefield,
    );
    reject(&mut prohibited, choice.clone());

    put(&mut game, player, "Candy Trail", Zone::Battlefield);
    put(&mut game, player, "Doubling Season", Zone::Battlefield);
    put(&mut game, player, "Doubling Season", Zone::Battlefield);
    act(&mut game, choice);
    assert!(!game.object_view(source).tapped);
    reject(&mut game, Action::DeclineEvidence);
    reject(&mut game, Action::FinishEvidence);
    select_evidence(&mut game, &[fuel]);
    assert!(game.observe(player).last_evidence.is_none());
    pay(&mut game);
    assert!(matches!(game.state.pending, Pending::EffectMana { .. }));
    assert!(game.object_view(source).tapped);
    assert_eq!(game.object_view(fuel).zone, Zone::Exile);
    assert_eq!(game.state.players[player.index()].mana, [0; 6]);
    assert_eq!(game.counter_count(source, CounterKind::Unlock), 0);
    assert!(game.state.stack.is_empty());
    let event = game.observe(player.other()).last_evidence.unwrap();
    assert_eq!(event.sequence, 1);
    assert_eq!(event.source, game.object_ref(source));
    assert_eq!(event.exiled, [game.object_ref(fuel)]);

    act(&mut game, Action::ChooseMana { production: GREEN });
    assert_eq!(game.state.players[player.index()].mana, GREEN);
    assert_eq!(game.state.pending, Pending::Replacement { player });
    assert_eq!(game.counter_count(source, CounterKind::Unlock), 0);
    choose(&mut game, |action| {
        matches!(action, Action::ChooseReplacement { .. })
    });
    assert_eq!(game.counter_count(source, CounterKind::Unlock), 4);
    settle(&mut game);
    assert_eq!(game.state.players[player.index()].cards_drawn_this_turn, 1);
    assert_eq!(game.observe(player.other()).last_evidence.unwrap(), event);
}

#[test]
fn a_parent_cast_keeps_its_exact_evidence_while_mana_evidence_branches_and_replays() {
    let _rules = fixture_rules::activate(&[
        (named("Soul-Guide Lantern"), &CRYPTEX_MANA),
        (named("Quick Study"), &PARENT_EVIDENCE),
    ]);
    let player = Player::P0;
    let mut game = scenario(player);
    let source = put(&mut game, player, "Soul-Guide Lantern", Zone::Battlefield);
    let spell = put(&mut game, player, "Quick Study", Zone::Hand);
    let reserved = put(&mut game, player, "Hill Giant", Zone::Graveyard);
    let first = put(&mut game, player, "Grizzly Bears", Zone::Graveyard);
    let other = put(&mut game, player.other(), "Hill Giant", Zone::Graveyard);
    let initial_payment = [0, 0, 0, 0, 0, 2];
    game.state.players[player.index()].mana = initial_payment;
    let mut short = game.clone();
    act(&mut short, Action::AnnounceCast { card: spell });
    reject(&mut short, Action::ChooseCollectEvidence { collect: true });

    let second = put(&mut game, player, "Lightning Bolt", Zone::Graveyard);
    game.begin_combat_rollback(player);
    act(&mut game, Action::AnnounceCast { card: spell });
    act(&mut game, Action::ChooseCollectEvidence { collect: true });
    select_evidence(&mut game, &[reserved]);
    let parent_source = game.object_ref(spell);
    act(&mut game, announce_evidence_mana(source));
    assert!(game.observe(player.other()).evidence.is_none());
    for unavailable in [reserved, other] {
        let choice = choose_evidence(&game, unavailable);
        reject(&mut game, choice);
    }

    let choice = choose_evidence(&game, first);
    let index = game
        .legal_actions()
        .iter()
        .position(|action| *action == choice)
        .unwrap();
    let revision = game.revision();
    let saved = game.snapshot();
    let mut indexed = game.clone();
    act(&mut game, choice.clone());
    indexed.apply_index(player, revision, index).unwrap();
    assert_eq!(game.snapshot(), indexed.snapshot());
    indexed.restore(&saved).unwrap();
    assert_eq!(
        indexed.apply_index(player, revision, index),
        Err(Error::StaleRevision)
    );
    act(&mut indexed, choice.clone());
    assert_eq!(game.snapshot(), indexed.snapshot());
    reject(&mut game, choice);
    reject(&mut game, Action::FinishEvidence);

    select_evidence(&mut game, &[second]);
    pay(&mut game);
    assert_eq!(game.object_view(reserved).zone, Zone::Graveyard);
    reject(&mut game, Action::ChooseMana { production: GREEN });
    act(&mut game, Action::ChooseMana { production: BLUE });
    assert_eq!(game.state.pending, Pending::PayCosts { player });
    assert_eq!(game.counter_count(source, CounterKind::Unlock), 1);
    assert_eq!(game.state.players[player.index()].mana, [0, 1, 0, 0, 0, 2]);
    let first_event = game.observe(player.other()).last_evidence.unwrap();
    assert_eq!(first_event.sequence, 1);
    assert_eq!(
        first_event.exiled,
        [game.object_ref(first), game.object_ref(second)]
    );

    let mut failed = game.clone();
    failed.finish_cast_rollback(false);
    assert_eq!(failed.state.pending, Pending::ManaRollback { player });
    let mut stale = failed.clone();
    stale.state.objects[first as usize].generation += 1;
    reject(&mut stale, Action::ChooseManaRollback { keep: true });
    for keep in [false, true] {
        let mut replay = failed.clone();
        act(&mut replay, Action::ChooseManaRollback { keep });
        assert_eq!(replay.object_view(spell).zone, Zone::Hand);
        assert_eq!(replay.object_view(reserved).zone, Zone::Graveyard);
        let zone = if keep { Zone::Exile } else { Zone::Graveyard };
        assert_eq!(replay.object_view(first).zone, zone);
        assert_eq!(replay.object_view(second).zone, zone);
        assert_eq!(replay.object_view(source).tapped, keep);
        assert_eq!(
            replay.counter_count(source, CounterKind::Unlock),
            u64::from(keep)
        );
        assert_eq!(
            replay.state.players[player.index()].mana,
            [0, u32::from(keep), 0, 0, 0, 2]
        );
        assert_eq!(
            replay.observe(player.other()).last_evidence,
            keep.then(|| first_event.clone())
        );
    }

    pay(&mut game);
    let second_event = game.observe(player.other()).last_evidence.unwrap();
    assert_eq!(second_event.sequence, 2);
    assert_eq!(second_event.source, parent_source);
    assert_eq!(second_event.exiled, [game.object_ref(reserved)]);
    settle(&mut game);
    assert_eq!(game.state.players[player.index()].cards_drawn_this_turn, 1);
}

const COMBAT_EVIDENCE: CardRules = CardRules {
    effect: Effect::Creature(6, 6),
    keywords: Keywords::FLYING.union(Keywords::LIFELINK),
    triggers: &[
        TriggeredAbility {
            condition: TriggerCondition::PlayerDamaged {
                subject: DamageSubject::Source,
                combat_only: true,
                opponent_only: false,
                one_or_more: false,
            },
            targets: TargetPlan::NONE,
            program: &[
                Instruction::May { skip: 2 },
                Instruction::CollectEvidenceX,
                Instruction::Reflexive { ability: 1 },
            ],
        },
        TriggeredAbility {
            condition: TriggerCondition::Reflexive,
            targets: TargetPlan::NONE,
            program: &[Instruction::DamagePlayerPermanents {
                players: PlayerSet::DamagedPlayer,
                kinds: 1 | 64,
                amount: AmountValue::ChosenX,
            }],
        },
    ],
    ..CardRules::EMPTY
};

fn pass_pair(game: &mut Game) {
    act(game, Action::Pass);
    act(game, Action::Pass);
}

#[test]
fn announced_evidence_x_is_optional_and_carries_the_declared_amount_to_a_new_damage_trigger() {
    let _rules = fixture_rules::activate(&[(named("Hill Giant"), &COMBAT_EVIDENCE)]);
    for player in [Player::P0, Player::P1] {
        let opponent = player.other();
        let mut game = scenario(player);
        let source = put(&mut game, player, "Hill Giant", Zone::Battlefield);
        let witness = ten_mana_witness(&mut game, player);
        let foreign = put(&mut game, opponent, "Hill Giant", Zone::Graveyard);
        let creature = put(&mut game, opponent, "Grizzly Bears", Zone::Battlefield);
        let walker = put(
            &mut game,
            opponent,
            "Chandra, Flameshaper",
            Zone::Battlefield,
        );
        game.add_counter(walker, CounterKind::Loyalty, 5);
        let artifact = put(&mut game, opponent, "Short Bow", Zone::Battlefield);
        let friendly = put(&mut game, player, "Bear Cub", Zone::Battlefield);
        let murder = put(&mut game, opponent, "Murder", Zone::Hand);
        let newcomer = put(&mut game, opponent, "Valley Floodcaller", Zone::Hand);

        let mut no_player_hit = game.clone();
        no_player_hit.apply_damage_batch(&[DamageEvent {
            preventable: true,
            source: no_player_hit.damage_source(source),
            target: Target::Card(walker),
            amount: 1,
            combat: true,
        }]);
        assert!(no_player_hit.state.pending_triggers.is_empty());
        no_player_hit.apply_damage_batch(&[DamageEvent {
            preventable: true,
            source: no_player_hit.damage_source(source),
            target: Target::Player(opponent),
            amount: 1,
            combat: false,
        }]);
        assert!(no_player_hit.state.pending_triggers.is_empty());

        game.state.step = Step::Attackers;
        game.state.pending = Pending::Attackers;
        act(&mut game, Action::Attack { card: source });
        act(&mut game, Action::FinishAttackers);
        pass_pair(&mut game);
        act(&mut game, Action::FinishBlockers);
        pass_pair(&mut game);
        assert_eq!(game.state.stack.len(), 1);
        assert_eq!(game.state.players[opponent.index()].life, 14);
        assert_eq!(game.state.players[player.index()].life, 26);
        let original = game.state.stack.last().unwrap().id;
        pass_pair(&mut game);

        let mut declined = game.clone();
        act(&mut declined, Action::ChooseOptional { accept: false });
        assert!(declined.state.stack.is_empty());
        assert!(declined.observe(opponent).last_evidence.is_none());

        act(&mut game, Action::ChooseOptional { accept: true });
        let domain = game.observe(player).x_choice.unwrap();
        assert_eq!(
            domain.intervals,
            [XInterval {
                minimum: 0,
                maximum: 10
            }]
        );
        assert!(game.observe(opponent).x_choice.is_none());
        reject(&mut game, Action::ChooseX { value: 11 });

        let mut zero = game.clone();
        act(&mut zero, Action::ChooseX { value: 0 });
        act(&mut zero, Action::FinishEvidence);
        let event = zero.observe(opponent).last_evidence.unwrap();
        assert_eq!((event.threshold, event.total_mana), (0, 0));
        assert!(event.exiled.is_empty());
        assert_eq!(zero.state.stack.last().unwrap().x, Some(0));
        settle(&mut zero);
        assert_eq!(zero.object_view(creature).zone, Zone::Battlefield);
        assert_eq!(zero.counter_count(walker, CounterKind::Loyalty), 5);

        let choice = Action::ChooseX { value: 2 };
        let index = game
            .legal_actions()
            .iter()
            .position(|action| *action == choice)
            .unwrap();
        let revision = game.revision();
        let mut indexed = game.clone();
        act(&mut game, choice);
        indexed.apply_index(player, revision, index).unwrap();
        assert_eq!(indexed.snapshot(), game.snapshot());
        assert_eq!(game.observe(player).evidence.unwrap().threshold, 2);
        reject(&mut game, Action::DeclineEvidence);
        let wrong_owner = choose_evidence(&game, foreign);
        reject(&mut game, wrong_owner);
        select_evidence(&mut game, &[witness]);
        let receipt = game.observe(opponent).last_evidence.unwrap();
        assert_eq!((receipt.threshold, receipt.total_mana), (2, 10));
        assert_eq!(game.state.stack.len(), 1);
        let packet = game.state.stack.last().unwrap();
        assert_ne!(packet.id, original);
        assert_eq!(packet.kind, StackKind::TriggeredAbility { ability: 1 });
        assert_eq!(packet.x, Some(2));
        assert_eq!(packet.damage_event.unwrap().player, opponent);
        assert!(packet.targets.is_empty());
        assert_eq!(game.object_view(creature).zone, Zone::Battlefield);

        game.state.players[opponent.index()].mana = [0, 1, 2, 0, 0, 3];
        if game.actor() != Some(opponent) {
            act(&mut game, Action::Pass);
        }
        cast(&mut game, murder, Some(Target::Card(source)));
        pass_pair(&mut game);
        assert_eq!(game.object_view(source).zone, Zone::Graveyard);
        if game.actor() != Some(opponent) {
            act(&mut game, Action::Pass);
        }
        cast(&mut game, newcomer, None);
        pass_pair(&mut game);
        assert_eq!(game.object_view(newcomer).zone, Zone::Battlefield);
        settle(&mut game);
        assert_eq!(game.object_view(creature).zone, Zone::Graveyard);
        assert_eq!(game.object_view(newcomer).zone, Zone::Graveyard);
        assert_eq!(game.counter_count(walker, CounterKind::Loyalty), 3);
        assert_eq!(game.object_view(friendly).zone, Zone::Battlefield);
        assert!(!game.object_view(artifact).damaged_this_turn);
        assert_eq!(game.state.players[opponent.index()].life, 14);
        assert_eq!(game.state.players[player.index()].life, 32);
    }
}

const EVIDENCE_RETURN: CardRules = CardRules {
    effect: Effect::Creature(3, 3),
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::SelfDies,
        targets: TargetPlan::NONE,
        program: &[Instruction::ExileDepartedSourceCollectEvidenceReturnTapped(
            4,
        )],
    }],
    ..CardRules::EMPTY
};

#[test]
fn a_death_trigger_pays_source_exile_with_separate_evidence_and_returns_it_tapped_to_its_owner() {
    struct ResetPrevention;
    impl Drop for ResetPrevention {
        fn drop(&mut self) {
            zone_replacements::TEST_PREVENT_ZONE_MOVE.with(|rule| rule.set(None));
        }
    }

    let _rules = fixture_rules::activate(&[
        (named("Hill Giant"), &EVIDENCE_RETURN),
        (named("Candy Trail"), &NO_EXILE),
    ]);
    let _prevention = ResetPrevention;
    for controller in [Player::P0, Player::P1] {
        for owner in [controller, controller.other()] {
            let mut game = scenario(controller);
            let source = put(&mut game, owner, "Hill Giant", Zone::Battlefield);
            let original = game.object_ref(source);
            if owner != controller {
                game.record_control(original, controller, None);
            }
            let first = put(&mut game, controller, "Grizzly Bears", Zone::Graveyard);
            let second = put(&mut game, controller, "Bear Cub", Zone::Graveyard);
            let foreign = put(&mut game, controller.other(), "Hill Giant", Zone::Graveyard);
            let removal = put(&mut game, controller, "Murder", Zone::Hand);
            cast(&mut game, removal, Some(Target::Card(source)));
            pass_pair(&mut game);
            let dead = game.object_ref(source);
            assert_eq!(game.object_view(source).zone, Zone::Graveyard);
            let trigger = game.state.stack.last().unwrap();
            assert_eq!(trigger.controller, controller);
            assert!(trigger.targets.is_empty());

            let mut insufficient = game.clone();
            insufficient.move_object(second, Zone::Hand);
            // Murder itself also entered the payer's graveyard. Remove it so
            // that the dying card's four mana value would be the only shortcut.
            insufficient.move_object(removal, Zone::Exile);
            settle(&mut insufficient);
            assert_eq!(insufficient.object_ref(source), dead);
            assert!(insufficient.observe(controller).last_evidence.is_none());
            assert_eq!(insufficient.object_view(first).zone, Zone::Graveyard);

            let mut stale = game.clone();
            stale.move_object(source, Zone::Exile);
            stale.move_object(source, Zone::Graveyard);
            let later = stale.object_ref(source);
            assert_ne!(dead, later);
            settle(&mut stale);
            assert_eq!(stale.object_ref(source), later);
            assert!(stale.observe(controller).last_evidence.is_none());
            assert_eq!(stale.object_view(first).zone, Zone::Graveyard);

            let mut prohibited = game.clone();
            put(
                &mut prohibited,
                controller.other(),
                "Candy Trail",
                Zone::Battlefield,
            );
            settle(&mut prohibited);
            assert_eq!(prohibited.object_ref(source), dead);
            assert!(prohibited.observe(controller).last_evidence.is_none());

            pass_pair(&mut game);
            assert_eq!(game.state.pending, Pending::Evidence { player: controller });
            let evidence = game.observe(controller).evidence.unwrap();
            assert_eq!(evidence.threshold, 4);
            assert!(evidence.optional);
            assert!(!evidence.candidates.contains(&dead));
            assert!(game.observe(controller.other()).evidence.is_none());
            let source_choice = choose_evidence(&game, source);
            let foreign_choice = choose_evidence(&game, foreign);
            reject(&mut game, source_choice);
            reject(&mut game, foreign_choice);
            let first_choice = choose_evidence(&game, first);
            act(&mut game, first_choice);
            reject(&mut game, Action::FinishEvidence);
            let mut declined = game.clone();
            act(&mut declined, Action::DeclineEvidence);
            assert_eq!(declined.object_ref(source), dead);
            assert_eq!(declined.object_view(first).zone, Zone::Graveyard);
            assert!(declined.observe(controller).last_evidence.is_none());

            let second_choice = choose_evidence(&game, second);
            act(&mut game, second_choice);
            let snapshot = game.snapshot();
            let mut indexed = game.clone();
            let mut restored = game.clone();
            let mut prevented = game.clone();
            let finish = game
                .legal_actions()
                .iter()
                .position(|action| *action == Action::FinishEvidence)
                .unwrap();
            act(&mut restored, Action::DeclineEvidence);
            restored.restore(&snapshot).unwrap();
            indexed
                .apply_index(controller, game.revision(), finish)
                .unwrap();
            act(&mut game, Action::FinishEvidence);
            act(&mut restored, Action::FinishEvidence);
            assert_eq!(game.snapshot(), indexed.snapshot());
            assert_eq!(game.snapshot(), restored.snapshot());

            let returned = game.object_view(source);
            assert_eq!(returned.reference.generation, dead.generation + 2);
            assert_eq!(returned.zone, Zone::Battlefield);
            assert_eq!(returned.controller, owner);
            assert!(returned.tapped);
            let receipt = game.observe(controller.other()).last_evidence.unwrap();
            assert_eq!((receipt.threshold, receipt.total_mana), (4, 4));
            assert_eq!(receipt.source, original);
            assert_eq!(receipt.player, controller);
            assert_eq!(
                receipt.exiled,
                [game.object_ref(first), game.object_ref(second)]
            );
            assert_eq!(game.object_view(foreign).zone, Zone::Graveyard);

            zone_replacements::TEST_PREVENT_ZONE_MOVE.with(|rule| rule.set(Some(dead)));
            act(&mut prevented, Action::FinishEvidence);
            zone_replacements::TEST_PREVENT_ZONE_MOVE.with(|rule| rule.set(None));
            assert_eq!(prevented.object_ref(source), dead);
            assert_eq!(prevented.object_view(source).zone, Zone::Graveyard);
            assert_eq!(prevented.object_view(first).zone, Zone::Exile);
            assert_eq!(prevented.object_view(second).zone, Zone::Exile);
            assert_eq!(prevented.observe(controller).last_evidence, Some(receipt));
        }
    }
}
