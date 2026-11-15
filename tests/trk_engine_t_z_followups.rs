//! Shared typed rules needed by the final Star Trek T–Z card cohort.
mod support;

use rustmage as _;
use rustmage_core::cards::authored::CardRules;
use rustmage_core::{cards::*, engine::*};
use support::*;

const CONQUEROR: CardRules = CardRules {
    effect: Effect::Spell(
        TargetPlan::NONE,
        &[
            Instruction::Mill(PlayerSet::Controller, MillAmount::Fixed(3)),
            Instruction::ChooseObjects {
                kind: EffectChoiceKind::ReturnGraveyardToHandMatching(SearchFilter::Land),
                players: PlayerSet::Controller,
                filter: SacrificeFilter {
                    keywords: Keywords::NONE,
                    source_only: false,
                    kinds: 0,
                    nonland: false,
                    other: false,
                    tokens: None,
                    artifact_enchantment_or_token: false,
                    subtype: None,
                },
                amount: 1,
                optional_skip: None,
            },
            Instruction::IfAmountAtLeast {
                value: AmountValue::MovedCount,
                minimum: 1,
                skip: 1,
            },
            Instruction::Skip(1),
            Instruction::CreateTokens {
                token: Card(5302),
                amount: 1,
                tapped: false,
            },
        ],
    ),
    ..CardRules::EMPTY
};
const CARETAKER: CardRules = CardRules {
    effect: Effect::Spell(
        TargetPlan::Single(TargetSpec::one(TargetRule::Player {
            opponent_only: true,
        })),
        &[Instruction::ExileTargetLibraryUntilNonlandCastFree],
    ),
    ..CardRules::EMPTY
};
const NEEDS: CardRules = CardRules {
    effect: Effect::Spell(
        TargetPlan::NONE,
        &[Instruction::PreventPlayerDamageThisTurn],
    ),
    ..CardRules::EMPTY
};
const WRATH: CardRules = CardRules {
    effect: Effect::Spell(
        TargetPlan::Single(TargetSpec::one(TargetRule::Creature)),
        &[
            Instruction::Damage(1),
            Instruction::ScheduleStepWithTargetThisTurn {
                step: TriggerStep::BeginCombat,
                relation: CastRelation::Any,
                ability: 0,
            },
        ],
    ),
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::AtStep {
            step: TriggerStep::BeginCombat,
            relation: CastRelation::Any,
            guard: StepGuard::Always,
        },
        targets: TargetPlan::NONE,
        program: &[Instruction::TakeDelayedBattlefieldObjectsUntilEndTurn],
    }],
    ..CardRules::EMPTY
};
const RIKER: CardRules = CardRules {
    effect: Effect::Spell(
        TargetPlan::NONE,
        &[Instruction::SelectLibraryToSacrificeAtNextOwnEndStep(
            LibrarySelectionEffect {
                spec: LibrarySelectionSpec {
                    look: 0,
                    count: LibrarySelectionCount::All,
                    filter: LibrarySelectionFilter::Kinds(2),
                    optional: false,
                    reveal: true,
                    remainder: LibraryRemainder::BottomRandom,
                },
                look: LibraryLook::UntilKindsCount { kinds: 2, count: 2 },
                destination: LibraryPickDestination::Battlefield,
                public: true,
                maximum_mana_value: None,
            },
        )],
    ),
    ..CardRules::EMPTY
};
const COFFEE: CardRules = CardRules {
    effect: Effect::Spell(
        TargetPlan::NONE,
        &[Instruction::SelectLibraryTo(LibrarySelectionEffect {
            spec: LibrarySelectionSpec {
                look: 5,
                count: LibrarySelectionCount::Fixed(1),
                filter: LibrarySelectionFilter::AnyOf(&[
                    LibrarySelectionFilter::Subtype("Plains"),
                    LibrarySelectionFilter::AllOf(&[
                        LibrarySelectionFilter::NonlandPermanent,
                        LibrarySelectionFilter::ManaValueAtMost(3),
                    ]),
                ]),
                optional: true,
                reveal: true,
                remainder: LibraryRemainder::BottomRandom,
            },
            look: LibraryLook::Fixed(5),
            destination: LibraryPickDestination::Hand,
            public: false,
            maximum_mana_value: None,
        })],
    ),
    ..CardRules::EMPTY
};
const FIRST_CHARGE: CardRules = CardRules {
    effect: Effect::Creature(2, 2),
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::FirstSelfCountersAddedThisTurn {
            kind: CounterKind::Charge,
        },
        targets: TargetPlan::NONE,
        program: &[Instruction::Draw(PlayerSet::Controller, 1)],
    }],
    ..CardRules::EMPTY
};
const PRIVATE_LOOK: CardRules = CardRules {
    effect: Effect::Spell(
        TargetPlan::Single(TargetSpec::one(TargetRule::Player {
            opponent_only: true,
        })),
        &[
            Instruction::LookTargetHand,
            Instruction::Draw(PlayerSet::Controller, 1),
        ],
    ),
    ..CardRules::EMPTY
};
const DISTINCT_LANDS: StatValue = StatValue::DistinctNames(CountFilter {
    zone: CountZone::Battlefield,
    relation: 0,
    kind: 2,
    exclude: 0,
    subtype: None,
    basic: false,
    other: false,
    cards_only: false,
});
const PRAXIS: CardRules = CardRules {
    effect: Effect::Spell(
        TargetPlan::NONE,
        &[Instruction::ModifyPlayerCreaturesScaled {
            players: PlayerSet::Each,
            value: DISTINCT_LANDS,
            power_per_count: -1,
            toughness_per_count: -1,
            create_if_at_least: Some((6, Card(6010), 2)),
        }],
    ),
    ..CardRules::EMPTY
};
const NEEDS_CREATURES: CardRules = CardRules {
    effect: Effect::Spell(
        TargetPlan::NONE,
        &[
            Instruction::GrantControlledCreaturesThisTurn(
                Keywords::HEXPROOF.union(Keywords::INDESTRUCTIBLE),
            ),
            Instruction::PlayerHexproofThisTurn,
            Instruction::PreventPlayerDamageThisTurn,
        ],
    ),
    ..CardRules::EMPTY
};
const OPTIONAL_HAND_EXILE: CardRules = CardRules {
    effect: Effect::Spell(
        TargetPlan::NONE,
        &[
            Instruction::ChooseObjects {
                kind: EffectChoiceKind::ExileFromHand,
                players: PlayerSet::Controller,
                filter: SacrificeFilter {
                    keywords: Keywords::NONE,
                    source_only: false,
                    kinds: 0,
                    nonland: false,
                    other: false,
                    tokens: None,
                    artifact_enchantment_or_token: false,
                    subtype: None,
                },
                amount: 1,
                optional_skip: Some(2),
            },
            Instruction::IfAmountAtLeast {
                value: AmountValue::MovedCount,
                minimum: 1,
                skip: 1,
            },
            Instruction::Draw(PlayerSet::Controller, 1),
        ],
    ),
    ..CardRules::EMPTY
};

#[test]
fn needs_grants_only_current_own_creatures_and_player_protection() {
    let _rules =
        fixture_rules::activate(&[(Card::from_name("Unsummon").unwrap(), &NEEDS_CREATURES)]);
    let mut game = scenario(Player::P0);
    let own = put(&mut game, Player::P0, "Grizzly Bears", Zone::Battlefield);
    let enemy = put(&mut game, Player::P1, "Grizzly Bears", Zone::Battlefield);
    let artifact = put(&mut game, Player::P0, "Short Bow", Zone::Battlefield);
    let land = put(&mut game, Player::P0, "Forest", Zone::Battlefield);
    let spell = put(&mut game, Player::P0, "Unsummon", Zone::Hand);
    cast(&mut game, spell, None);
    drain_choices(&mut game);
    for keyword in [Keywords::HEXPROOF, Keywords::INDESTRUCTIBLE] {
        assert!(game.object_view(own).keywords.contains(keyword));
        for id in [enemy, artifact, land] {
            assert!(!game.object_view(id).keywords.contains(keyword));
        }
    }
    assert!(game.state.players[0].hexproof_until_end_turn);
    assert_eq!(game.observe(Player::P0).player_damage_shields.len(), 1);
}

#[test]
fn optional_hand_exile_draws_only_after_actual_exile() {
    struct Reset;
    impl Drop for Reset {
        fn drop(&mut self) {
            zone_replacements::TEST_PREVENT_ZONE_MOVE.with(|cell| cell.set(None));
        }
    }
    let _reset = Reset;
    let _rules =
        fixture_rules::activate(&[(Card::from_name("Unsummon").unwrap(), &OPTIONAL_HAND_EXILE)]);
    for (choose_card, prevent, expected_draws) in
        [(false, false, 0), (true, false, 1), (true, true, 0)]
    {
        let mut game = scenario(Player::P0);
        let card = put(&mut game, Player::P0, "Shock", Zone::Hand);
        let spell = put(&mut game, Player::P0, "Unsummon", Zone::Hand);
        cast(&mut game, spell, None);
        while !matches!(game.state.pending, Pending::EffectChoice { .. }) {
            choose(&mut game, |a| {
                matches!(a, Action::Pass | Action::PutTrigger { .. })
            });
        }
        let exact = game.object_ref(card);
        if prevent {
            zone_replacements::TEST_PREVENT_ZONE_MOVE.with(|cell| cell.set(Some(exact)));
            game.state.zone_rules.get_or_insert_default();
        }
        if choose_card {
            act(&mut game, Action::SelectResolutionExile { object: exact });
        } else {
            act(&mut game, Action::DeclineEffectPayment);
        }
        drain_choices(&mut game);
        assert_eq!(game.state.players[0].cards_drawn_this_turn, expected_draws);
        assert_eq!(
            game.object_view(card).zone,
            if prevent || !choose_card {
                Zone::Hand
            } else {
                Zone::Exile
            }
        );
        zone_replacements::TEST_PREVENT_ZONE_MOVE.with(|cell| cell.set(None));
    }
}

#[test]
fn private_hand_look_is_actor_only_and_continues_to_draw_with_owned_restore() {
    let _rules = fixture_rules::activate(&[(Card::from_name("Unsummon").unwrap(), &PRIVATE_LOOK)]);
    for player in [Player::P0, Player::P1] {
        let mut game = scenario(player);
        let secret = put(&mut game, player.other(), "Shock", Zone::Hand);
        let spell = put(&mut game, player, "Unsummon", Zone::Hand);
        cast(&mut game, spell, Some(Target::Player(player.other())));
        while !matches!(game.state.pending, Pending::HandCardChoice { .. }) {
            choose(&mut game, |a| {
                matches!(a, Action::Pass | Action::PutTrigger { .. })
            });
        }
        let actor_view = game.observe(player).hand_choice.unwrap();
        assert_eq!(actor_view.kind, HandChoiceKind::LookOnly);
        assert!(
            actor_view
                .inspected_cards
                .unwrap()
                .iter()
                .any(|o| o.id == secret)
        );
        assert!(actor_view.candidates.is_empty());
        assert!(game.observe(player.other()).hand_choice.is_none());
        let action = Action::ChooseHandCard { object: None };
        let legal = game.legal_actions();
        assert!(
            legal
                .iter()
                .all(|a| matches!(a, Action::Concede | Action::ChooseHandCard { object: None }))
        );
        let index = legal.iter().position(|a| *a == action).unwrap();
        let checkpoint = game.snapshot();
        let revision = game.revision();
        let actor = game.actor().unwrap();
        let mut indexed = game.clone();
        act(&mut game, action.clone());
        indexed.apply_index(actor, revision, index).unwrap();
        assert_eq!(game.snapshot(), indexed.snapshot());
        indexed.restore(&checkpoint).unwrap();
        assert_eq!(
            indexed.apply_index(actor, revision, index),
            Err(Error::StaleRevision)
        );
        act(&mut indexed, action);
        assert_eq!(game.snapshot(), indexed.snapshot());
        drain_choices(&mut game);
        assert_eq!(game.state.players[player.index()].cards_drawn_this_turn, 1);
        assert_eq!(game.object_view(secret).zone, Zone::Hand);
    }
}

#[test]
fn praxis_scales_both_players_and_uses_same_six_name_count_for_tokens() {
    let _rules = fixture_rules::activate(&[(Card::from_name("Unsummon").unwrap(), &PRAXIS)]);
    for (lands, expected) in [
        (&["Forest", "Island", "Mountain", "Plains", "Swamp"][..], 5),
        (
            &["Forest", "Island", "Mountain", "Plains", "Swamp", "Wastes"][..],
            6,
        ),
    ] {
        let mut game = scenario(Player::P0);
        for &name in lands {
            put(&mut game, Player::P0, name, Zone::Battlefield);
        }
        put(&mut game, Player::P0, "Forest", Zone::Battlefield);
        let own = put(&mut game, Player::P0, "Grizzly Bears", Zone::Battlefield);
        let enemy = put(&mut game, Player::P1, "Grizzly Bears", Zone::Battlefield);
        for id in [own, enemy] {
            game.state.objects[id as usize].growth = Stats {
                power: 10,
                toughness: 10,
            };
        }
        let spell = put(&mut game, Player::P0, "Unsummon", Zone::Hand);
        cast(&mut game, spell, None);
        drain_choices(&mut game);
        for id in [own, enemy] {
            assert_eq!(stats(&game, id), (12 - expected, 12 - expected));
        }
        assert_eq!(
            game.state
                .objects
                .iter()
                .filter(|o| o.zone == Zone::Battlefield && o.card == Card(6010))
                .count(),
            if expected == 6 { 2 } else { 0 }
        );
    }
}

fn drain_choices(game: &mut Game) {
    for _ in 0..80 {
        if game.state.pending == Pending::Priority
            && game.state.stack.is_empty()
            && game.state.pending_triggers.is_empty()
            && game.state.resolving.is_none()
        {
            return;
        }
        let actions = game.legal_actions();
        let action = actions
            .into_iter()
            .find(|a| {
                matches!(
                    a,
                    Action::Pass
                        | Action::PutTrigger { .. }
                        | Action::SelectLibrary { .. }
                        | Action::FinishLibrarySelection
                        | Action::ChooseOptional { accept: true }
                        | Action::FinishResolutionCasts
                )
            })
            .unwrap_or_else(|| panic!("unhandled {:?}", game.state.pending));
        act(game, action);
    }
    panic!("resolution did not finish");
}

#[test]
fn first_charge_event_counts_even_while_ability_is_suppressed_and_restores() {
    let _rules =
        fixture_rules::activate(&[(Card::from_name("Grizzly Bears").unwrap(), &FIRST_CHARGE)]);
    let mut game = scenario(Player::P0);
    let ship = put(&mut game, Player::P0, "Grizzly Bears", Zone::Battlefield);
    let exact = game.object_ref(ship);
    game.remove_abilities_exact(&[exact], exact, Player::P1, AnimationDuration::Permanent);
    game.add_counter(ship, CounterKind::Charge, 1);
    game.self_counters_added(exact, CounterKind::Charge, 1, Player::P1);
    drain_choices(&mut game);
    assert_eq!(game.state.players[0].cards_drawn_this_turn, 0);

    // Model the suppression effect ending without changing the permanent's incarnation.
    game.state
        .roles
        .as_mut()
        .unwrap()
        .resolved_ability_modifications
        .clear();
    game.add_counter(ship, CounterKind::Charge, 1);
    game.self_counters_added(exact, CounterKind::Charge, 1, Player::P0);
    drain_choices(&mut game);
    assert_eq!(game.state.players[0].cards_drawn_this_turn, 0);

    let checkpoint = game.snapshot();
    let mut restored = game.clone();
    restored.restore(&checkpoint).unwrap();
    restored.add_counter(ship, CounterKind::Charge, 1);
    restored.self_counters_added(exact, CounterKind::Charge, 1, Player::P0);
    drain_choices(&mut restored);
    assert_eq!(restored.state.players[0].cards_drawn_this_turn, 0);

    game.state.step = Step::Cleanup;
    game.state.priority = Player::P0;
    act(&mut game, Action::Pass);
    act(&mut game, Action::Pass);
    game.add_counter(ship, CounterKind::Charge, 1);
    game.self_counters_added(exact, CounterKind::Charge, 1, Player::P1);
    drain_choices(&mut game);
    assert_eq!(game.state.players[0].cards_drawn_this_turn, 1);
}

#[test]
fn conqueror_uses_the_whole_graveyard_and_actual_hand_move_for_treasure_fallback() {
    let _rules = fixture_rules::activate(&[(Card::from_name("Unsummon").unwrap(), &CONQUEROR)]);
    for player in [Player::P0, Player::P1] {
        let mut game = scenario(player);
        let old_land = put(&mut game, player, "Island", Zone::Graveyard);
        let spell = put(&mut game, player, "Unsummon", Zone::Hand);
        cast(&mut game, spell, None);
        while !matches!(game.state.pending, Pending::EffectChoice { player: p } if p == player) {
            choose(&mut game, |a| {
                matches!(a, Action::Pass | Action::PutTrigger { .. })
            });
        }
        let land_ref = game.object_ref(old_land);
        act(
            &mut game,
            Action::SelectGraveyardReturn { object: land_ref },
        );
        drain_choices(&mut game);
        assert_eq!(game.object_view(old_land).zone, Zone::Hand);
        assert_eq!(
            game.state
                .objects
                .iter()
                .filter(|o| o.zone == Zone::Battlefield && o.card == Card(5302))
                .count(),
            0
        );
    }
}

#[test]
fn conqueror_treasure_fallback_uses_actual_failed_return() {
    struct Reset;
    impl Drop for Reset {
        fn drop(&mut self) {
            zone_replacements::TEST_PREVENT_ZONE_MOVE.with(|cell| cell.set(None));
        }
    }
    let _reset = Reset;
    let _rules = fixture_rules::activate(&[(Card::from_name("Unsummon").unwrap(), &CONQUEROR)]);
    let mut game = scenario(Player::P0);
    let land = put(&mut game, Player::P0, "Island", Zone::Graveyard);
    let spell = put(&mut game, Player::P0, "Unsummon", Zone::Hand);
    cast(&mut game, spell, None);
    while !matches!(game.state.pending, Pending::EffectChoice { .. }) {
        choose(&mut game, |a| {
            matches!(a, Action::Pass | Action::PutTrigger { .. })
        });
    }
    let exact = game.object_ref(land);
    zone_replacements::TEST_PREVENT_ZONE_MOVE.with(|cell| cell.set(Some(exact)));
    game.state.zone_rules.get_or_insert_default();
    act(&mut game, Action::SelectGraveyardReturn { object: exact });
    drain_choices(&mut game);
    assert_eq!(game.object_view(land).zone, Zone::Graveyard);
    assert_eq!(
        game.state
            .objects
            .iter()
            .filter(|o| o.zone == Zone::Battlefield && o.card == Card(5302))
            .count(),
        1
    );
}

#[test]
fn caretaker_uses_opponent_library_and_leaves_declined_hit_exiled() {
    let _rules = fixture_rules::activate(&[(Card::from_name("Unsummon").unwrap(), &CARETAKER)]);
    for player in [Player::P0, Player::P1] {
        let mut game = scenario(player);
        let land = put(&mut game, player.other(), "Island", Zone::Library);
        let hit = put(&mut game, player.other(), "Shock", Zone::Library);
        game.state.players[player.other().index()]
            .library
            .retain(|&id| id != land && id != hit);
        game.state.players[player.other().index()]
            .library
            .extend([hit, land]);
        let spell = put(&mut game, player, "Unsummon", Zone::Hand);
        cast(&mut game, spell, Some(Target::Player(player.other())));
        while !matches!(game.state.pending, Pending::ResolutionCasts { .. }) {
            choose(&mut game, |a| {
                matches!(a, Action::Pass | Action::PutTrigger { .. })
            });
        }
        assert_eq!(game.object_view(hit).zone, Zone::Exile);
        assert_eq!(game.object_view(land).zone, Zone::Exile);
        choose(&mut game, |a| matches!(a, Action::FinishResolutionCasts));
        drain_choices(&mut game);
        assert_eq!(game.object_view(hit).zone, Zone::Exile);
    }
}

#[test]
fn needs_player_shield_expires_during_cleanup() {
    let _rules = fixture_rules::activate(&[(Card::from_name("Unsummon").unwrap(), &NEEDS)]);
    let mut game = scenario(Player::P0);
    let spell = put(&mut game, Player::P0, "Unsummon", Zone::Hand);
    cast(&mut game, spell, None);
    settle(&mut game);
    assert_eq!(game.observe(Player::P0).player_damage_shields.len(), 1);
    assert_eq!(
        game.observe(Player::P1).player_damage_shields[0].expires_turn,
        Some(game.state.turn)
    );
    let shock = put(&mut game, Player::P1, "Shock", Zone::Hand);
    game.state.priority = Player::P0;
    game.state.passes = 0;
    act(&mut game, Action::Pass);
    assert_eq!(game.actor(), Some(Player::P1));
    cast(&mut game, shock, Some(Target::Player(Player::P0)));
    settle(&mut game);
    assert_eq!(game.state.players[0].life, 20);
    game.state.step = Step::Cleanup;
    game.state.priority = Player::P0;
    act(&mut game, Action::Pass);
    act(&mut game, Action::Pass);
    assert!(game.observe(Player::P0).player_damage_shields.is_empty());
    let later = put(&mut game, Player::P1, "Shock", Zone::Hand);
    game.state.step = Step::Main1;
    game.state.pending = Pending::Priority;
    game.state.priority = Player::P0;
    game.state.passes = 0;
    game.state.players[Player::P1.index()].mana = [8; 6];
    act(&mut game, Action::Pass);
    assert_eq!(game.actor(), Some(Player::P1));
    cast(&mut game, later, Some(Target::Player(Player::P0)));
    settle(&mut game);
    assert_eq!(game.state.players[0].life, 18);
}

#[test]
fn wrath_delayed_control_tracks_exact_creature_and_grants_haste_at_next_combat() {
    let _rules = fixture_rules::activate(&[(Card::from_name("Unsummon").unwrap(), &WRATH)]);
    let mut game = scenario(Player::P0);
    let creature = put(&mut game, Player::P1, "Hill Giant", Zone::Battlefield);
    game.state.objects[creature as usize].tapped = true;
    let spell = put(&mut game, Player::P0, "Unsummon", Zone::Hand);
    cast(&mut game, spell, Some(Target::Card(creature)));
    settle(&mut game);
    assert_eq!(game.object_view(creature).controller, Player::P1);
    let checkpoint = game.snapshot();
    let mut returned = game.clone();
    returned.restore(&checkpoint).unwrap();
    returned.move_object(creature, Zone::Hand);
    returned.move_object(creature, Zone::Battlefield);
    returned.begin_step(Step::BeginCombat);
    returned.advance_replacements(false);
    returned.give_priority();
    drain_choices(&mut returned);
    assert_eq!(returned.object_view(creature).controller, Player::P1);
    game.begin_step(Step::BeginCombat);
    game.advance_replacements(false);
    game.give_priority();
    drain_choices(&mut game);
    assert_eq!(game.object_view(creature).controller, Player::P0);
    assert!(!game.object_view(creature).tapped);
    assert!(
        game.object_view(creature)
            .keywords
            .contains(Keywords::HASTE)
    );
}

#[test]
fn riker_sacrifices_only_two_exact_lands_at_own_next_end_step() {
    let _rules = fixture_rules::activate(&[(Card::from_name("Unsummon").unwrap(), &RIKER)]);
    let mut game = scenario(Player::P0);
    game.state.players[0].library.clear();
    let first = put(&mut game, Player::P0, "Island", Zone::Library);
    let middle = put(&mut game, Player::P0, "Shock", Zone::Library);
    let second = put(&mut game, Player::P0, "Forest", Zone::Library);
    let spell = put(&mut game, Player::P0, "Unsummon", Zone::Hand);
    cast(&mut game, spell, None);
    drain_choices(&mut game);
    assert_eq!(game.object_view(first).zone, Zone::Battlefield);
    assert_eq!(game.object_view(second).zone, Zone::Battlefield);
    assert_eq!(game.object_view(middle).zone, Zone::Library);
    let checkpoint = game.snapshot();
    let mut restored = game.clone();
    restored.restore(&checkpoint).unwrap();
    assert_eq!(restored.snapshot(), checkpoint);
    let unaffected = put(&mut game, Player::P0, "Mountain", Zone::Battlefield);
    game.begin_step(Step::End);
    game.advance_replacements(false);
    game.give_priority();
    drain_choices(&mut game);
    assert_eq!(game.object_view(first).zone, Zone::Graveyard);
    assert_eq!(game.object_view(second).zone, Zone::Graveyard);
    assert_eq!(game.object_view(unaffected).zone, Zone::Battlefield);
    assert_eq!(restored.object_view(first).zone, Zone::Battlefield);
}

#[test]
fn coffee_library_filter_allows_plains_or_small_permanent_only() {
    let _rules = fixture_rules::activate(&[(Card::from_name("Unsummon").unwrap(), &COFFEE)]);
    let mut game = scenario(Player::P0);
    game.state.players[0].library.clear();
    let plains = put(&mut game, Player::P0, "Temple Garden", Zone::Library);
    let small = put(&mut game, Player::P0, "Grizzly Bears", Zone::Library);
    let large = put(&mut game, Player::P0, "Serra Angel", Zone::Library);
    let instant = put(&mut game, Player::P0, "Shock", Zone::Library);
    let forest = put(&mut game, Player::P0, "Forest", Zone::Library);
    let spell = put(&mut game, Player::P0, "Unsummon", Zone::Hand);
    cast(&mut game, spell, None);
    while !matches!(game.state.pending, Pending::EffectLibrarySelection { .. }) {
        choose(&mut game, |a| {
            matches!(a, Action::Pass | Action::PutTrigger { .. })
        });
    }
    for allowed in [plains, small] {
        assert!(
            game.legal_actions()
                .contains(&Action::SelectLibrary { card: allowed })
        );
    }
    for disallowed in [large, instant, forest] {
        reject(&mut game, Action::SelectLibrary { card: disallowed });
    }
    act(&mut game, Action::SelectLibrary { card: plains });
    if game
        .legal_actions()
        .contains(&Action::FinishLibrarySelection)
    {
        act(&mut game, Action::FinishLibrarySelection);
    }
    drain_choices(&mut game);
    assert_eq!(game.object_view(plains).zone, Zone::Hand);
    assert_eq!(game.object_view(small).zone, Zone::Library);
}
