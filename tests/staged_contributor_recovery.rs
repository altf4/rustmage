//! Current Crew/Saddle/Station choices remain editable and explicitly recoverable.
mod support;
use rustmage::Card;
use rustmage_core::cards::authored::{CardExtras, CardRules};
use rustmage_core::{cards::*, engine::*};
use support::*;

const ZERO: Mana = [0; 6];
const ONE: Mana = [0, 0, 0, 0, 0, 1];
const BASE: ActivatedAbility = ActivatedAbility {
    exhaust: false,
    guard: ActivationGuard::Always,
    origin: ActivationOrigin::Battlefield,
    cycling: false,
    cost: ActivationCost::ZERO,
    targets: TargetPlan::NONE,
    program: &[],
    sorcery: false,
    once_per_turn: false,
};
const CREW: CardRules = CardRules {
    effect: Effect::Permanent,
    extras: &CardExtras {
        vehicle_stats: Some(Stats {
            power: 5,
            toughness: 5,
        }),
        ..CardExtras::EMPTY
    },
    activated: &[ActivatedAbility {
        program: &[Instruction::Crew { minimum: 3 }],
        ..BASE
    }],
    ..CardRules::EMPTY
};
const SADDLE: CardRules = CardRules {
    effect: Effect::Creature(2, 2),
    activated: &[ActivatedAbility {
        program: &[Instruction::BecomeSaddled { minimum: 3 }],
        ..BASE
    }],
    ..CardRules::EMPTY
};
const PAID_CREW: CardRules = CardRules {
    activated: &[ActivatedAbility {
        cost: ActivationCost {
            generic: 1,
            mana_component: true,
            ..ActivationCost::ZERO
        },
        ..CREW.activated[0]
    }],
    ..CREW
};
const PAID_SADDLE: CardRules = CardRules {
    activated: &[ActivatedAbility {
        cost: PAID_CREW.activated[0].cost,
        ..SADDLE.activated[0]
    }],
    ..SADDLE
};
const NEGATIVE: CardRules = CardRules {
    effect: Effect::Creature(-4, 8),
    ..CardRules::EMPTY
};
const CREATURE_MANA: CardRules = CardRules {
    effect: Effect::Creature(3, 3),
    activated: &[ActivatedAbility {
        cost: ActivationCost {
            tap: true,
            ..ActivationCost::ZERO
        },
        program: &[Instruction::AddMana(ManaProduction::Fixed(ONE))],
        ..BASE
    }],
    ..CardRules::EMPTY
};
const MATERIAL_MANA: CardRules = CardRules {
    effect: Effect::Permanent,
    activated: &[ActivatedAbility {
        cost: ActivationCost {
            tap: true,
            sacrifice: true,
            ..ActivationCost::ZERO
        },
        program: &[Instruction::AddMana(ManaProduction::Fixed(ONE))],
        ..BASE
    }],
    ..CardRules::EMPTY
};
const CHILD_MANA: ActivatedAbility = ActivatedAbility {
    cost: ActivationCost {
        generic: 1,
        mana_component: true,
        ..ActivationCost::ZERO
    },
    program: &[Instruction::AddMana(ManaProduction::Fixed(ONE))],
    ..BASE
};
const NESTED_CREW: CardRules = CardRules {
    activated: &[PAID_CREW.activated[0], CHILD_MANA],
    ..CREW
};
const NESTED_SADDLE: CardRules = CardRules {
    activated: &[PAID_SADDLE.activated[0], CHILD_MANA],
    ..SADDLE
};

#[derive(Clone, Copy)]
enum Kind {
    Crew,
    Saddle,
}
impl Kind {
    fn card(self) -> Card {
        match self {
            Self::Crew => Card::from_name("Hedge Shredder").unwrap(),
            Self::Saddle => Card::GrizzlyBears,
        }
    }
    fn rules(self, paid: bool) -> &'static CardRules {
        match (self, paid) {
            (Self::Crew, false) => &CREW,
            (Self::Crew, true) => &PAID_CREW,
            (Self::Saddle, false) => &SADDLE,
            (Self::Saddle, true) => &PAID_SADDLE,
        }
    }
    fn choose(self, object: ObjectRef) -> Action {
        match self {
            Self::Crew => Action::ChooseCrewer { object },
            Self::Saddle => Action::ChooseSaddler { object },
        }
    }
    fn finish(self) -> Action {
        match self {
            Self::Crew => Action::FinishCrew,
            Self::Saddle => Action::FinishSaddle,
        }
    }
    fn pending(self, player: Player) -> Pending {
        match self {
            Self::Crew => Pending::Crew { player },
            Self::Saddle => Pending::Saddle { player },
        }
    }
    fn selected(self, game: &Game, player: Player) -> Vec<ObjectRef> {
        match self {
            Self::Crew => game.crew_choice(player).unwrap().selected,
            Self::Saddle => game.saddle_choice().unwrap().selected,
        }
    }
}
fn announce(source: CardId) -> Action {
    Action::AnnounceActivate {
        card: source,
        ability: 0,
        counter_payment: Counters::default(),
    }
}
fn same_step(game: &mut Game, action: Action) {
    let actor = game.actor().unwrap();
    let index = game
        .legal_actions()
        .iter()
        .position(|a| *a == action)
        .unwrap_or_else(|| {
            panic!(
                "missing {action:?} in {:?}: {:?}",
                game.state.pending,
                game.legal_actions()
            )
        });
    let revision = game.revision();
    let mut indexed = game.clone();
    indexed.apply_index(actor, revision, index).unwrap();
    act(game, action);
    assert_eq!(game.snapshot(), indexed.snapshot());
}
fn assert_failed_material(game: &Game, player: Player) {
    let outcome = game.last_attempt_outcome_view(player).unwrap();
    assert_eq!(outcome.outcome, AttemptOutcome::Failed);
    assert_eq!(
        outcome.reason,
        Some(AttemptFailureReason::UnsatisfiedMaterial)
    );
    assert_eq!(game.state.pending, Pending::Priority);
    assert!(game.state.announcing.is_none());
    assert!(game.state.stack.is_empty());
}

#[test]
fn empty_crew_and_saddle_attempts_fail_explicitly_for_either_player() {
    for kind in [Kind::Crew, Kind::Saddle] {
        let _rules = fixture_rules::activate(&[(kind.card(), kind.rules(false))]);
        for player in [Player::P0, Player::P1] {
            let mut game = scenario(player);
            let source = spawn(&mut game, player, kind.card(), Zone::Battlefield);
            let source_object = game.object_ref(source);
            game.state.players[player.index()].mana = ZERO;
            Game::reset_payment_planning_work();
            same_step(&mut game, announce(source));
            assert_eq!(game.state.pending, kind.pending(player));
            reject(&mut game, kind.finish());
            reject(&mut game, Action::SubmitAttempt);
            same_step(&mut game, Action::EndManaWindow);
            same_step(&mut game, Action::SubmitAttempt);
            assert_failed_material(&game, player);
            assert_eq!(game.object_ref(source), source_object);
            assert!(!game.object_view(source).tapped);
            assert!(game.legal_actions().contains(&announce(source)));
            let work = Game::payment_planning_work();
            assert_eq!(work.projected_moves, 0);
            assert_eq!(work.projected_steps, 0);
            assert_eq!(work.continuation_plans, 0);
        }
    }
}

#[test]
fn an_underpowered_prefix_can_reset_or_fail_without_tapping_its_contributors() {
    for kind in [Kind::Crew, Kind::Saddle] {
        let _rules = fixture_rules::activate(&[
            (kind.card(), kind.rules(false)),
            (Card::LlanowarElves, &NEGATIVE),
        ]);
        for player in [Player::P0, Player::P1] {
            let mut game = scenario(player);
            let source = spawn(&mut game, player, kind.card(), Zone::Battlefield);
            let negative = put(&mut game, player, "Llanowar Elves", Zone::Battlefield);
            let enough = put(&mut game, player, "Hill Giant", Zone::Battlefield);
            let spare = put(&mut game, player, "Bear Cub", Zone::Battlefield);
            let exact = [source, negative, enough, spare].map(|id| game.object_ref(id));
            act(&mut game, announce(source));
            // This supplied creature is eligible even though its negative power
            // makes every remaining current extension insufficient.
            same_step(&mut game, kind.choose(exact[1]));
            assert_eq!(kind.selected(&game, player), [exact[1]]);
            same_step(&mut game, Action::EndManaWindow);
            let mut failed = game.clone();
            same_step(&mut failed, Action::SubmitAttempt);
            assert_failed_material(&failed, player);
            for object in exact {
                assert_eq!(failed.object_ref(object.card), object);
                assert!(!failed.object_view(object.card).tapped);
            }

            same_step(&mut game, Action::ResetCostSelection);
            assert!(kind.selected(&game, player).is_empty());
            assert!(game.attempt_view(player).unwrap().mana_window_closed);
            // No contributors have been selected and a valid completion still
            // exists. Submitting here would only rewrite an incomplete receipt.
            assert!(!game.attempt_view(player).unwrap().can_submit);
            reject(&mut game, Action::SubmitAttempt);
            assert_eq!(game.state.pending, kind.pending(player));
            same_step(&mut game, kind.choose(exact[2]));
            same_step(&mut game, kind.finish());
            assert!(game.object_view(enough).tapped);
            assert!(!game.object_view(negative).tapped);
            assert!(!game.object_view(spare).tapped);
            assert!(!game.legal_actions().contains(&Action::ResetCostSelection));
            assert_eq!(
                game.last_attempt_outcome_view(player).unwrap().outcome,
                AttemptOutcome::Completed
            );
        }
    }
}

#[test]
fn paid_contributor_costs_validate_exact_power_and_reset_only_unpaid_choices() {
    let producer = Card::from_name("Short Bow").unwrap();
    for kind in [Kind::Crew, Kind::Saddle] {
        let _rules =
            fixture_rules::activate(&[(kind.card(), kind.rules(true)), (producer, &MATERIAL_MANA)]);
        for player in [Player::P0, Player::P1] {
            let mut game = scenario(player);
            let source = spawn(&mut game, player, kind.card(), Zone::Battlefield);
            let mana_source = spawn(&mut game, player, producer, Zone::Battlefield);
            let material_source = game.object_ref(mana_source);
            let exiled: Vec<_> = (0..3)
                .map(|_| {
                    let creature = put(&mut game, player, "Llanowar Elves", Zone::Exile);
                    let object = game.object_ref(creature);
                    game.state.exile_links.push(ExileLink {
                        ability_origin: None,
                        source: material_source,
                        exiled: object,
                        return_zone: Zone::Battlefield,
                    });
                    object
                })
                .collect();
            game.state.players[player.index()].mana = ZERO;
            act(&mut game, announce(source));
            assert_eq!(game.state.pending, kind.pending(player));
            reject(&mut game, kind.finish());
            let mana = Action::ActivateMana {
                card: mana_source,
                ability: 0,
                payment: ZERO,
                counter_payment: Counters::default(),
                production: ONE,
            };
            same_step(&mut game, mana.clone());
            assert_eq!(game.state.pending, kind.pending(player));
            let contributors: Vec<_> = exiled
                .iter()
                .map(|old| {
                    assert_eq!(game.object_view(old.card).zone, Zone::Battlefield);
                    let current = game.object_ref(old.card);
                    assert_ne!(current, *old);
                    current
                })
                .collect();
            assert_eq!(game.object_view(mana_source).zone, Zone::Graveyard);
            assert_eq!(game.state.players[player.index()].mana, ONE);
            same_step(&mut game, kind.choose(contributors[0]));
            reject(&mut game, kind.finish());
            same_step(&mut game, Action::EndManaWindow);
            same_step(&mut game, Action::ResetCostSelection);
            assert!(game.attempt_view(player).unwrap().mana_window_closed);
            assert!(kind.selected(&game, player).is_empty());
            assert_eq!(game.object_view(mana_source).zone, Zone::Graveyard);
            assert_ne!(game.object_ref(mana_source), material_source);
            assert_eq!(game.state.players[player.index()].mana, ONE);
            for object in &contributors {
                assert_eq!(game.object_ref(object.card), *object);
                assert_eq!(game.object_view(object.card).zone, Zone::Battlefield);
            }
            reject(&mut game, mana);
            for &object in &contributors {
                same_step(&mut game, kind.choose(object));
            }
            same_step(&mut game, kind.finish());
            assert_eq!(game.state.pending, Pending::PayCosts { player });
            assert!(
                contributors
                    .iter()
                    .all(|object| !game.object_view(object.card).tapped)
            );
            // Finish selected the group but has not paid the taxed activation.
            same_step(&mut game, Action::ResetCostSelection);
            assert_eq!(game.state.pending, kind.pending(player));
            for &object in &contributors {
                act(&mut game, kind.choose(object));
            }
            act(&mut game, kind.finish());
            same_step(&mut game, Action::PayCosts { payment: ONE });
            assert!(
                contributors
                    .iter()
                    .all(|object| game.object_view(object.card).tapped)
            );
            assert_eq!(game.state.players[player.index()].mana, ZERO);
            assert!(!game.legal_actions().contains(&Action::ResetCostSelection));
        }
    }
}

#[test]
fn station_without_a_current_tap_subject_uses_material_recovery() {
    for player in [Player::P0, Player::P1] {
        let mut game = scenario(player);
        let ship = put(&mut game, player, "Warmaker Gunship", Zone::Battlefield);
        let tapped = put(&mut game, player, "Bear Cub", Zone::Battlefield);
        let foreign = put(&mut game, player.other(), "Bear Cub", Zone::Battlefield);
        game.state.objects[tapped as usize].tapped = true;
        let exact = [ship, tapped, foreign].map(|id| game.object_ref(id));
        same_step(&mut game, announce(ship));
        assert_eq!(game.state.pending, Pending::AdditionalCost { player });
        for object in exact {
            reject(&mut game, Action::ChooseAdditionalCost { object });
        }
        same_step(&mut game, Action::EndManaWindow);
        same_step(&mut game, Action::SubmitAttempt);
        assert_failed_material(&game, player);
        for object in exact {
            assert_eq!(game.object_ref(object.card), object);
        }
        assert_eq!(game.counter_count(ship, CounterKind::Charge), 0);
        assert!(game.object_view(tapped).tapped);
        assert!(!game.object_view(foreign).tapped);
        assert!(game.legal_actions().contains(&announce(ship)));
    }
}

#[test]
fn resetting_a_contributor_tapped_for_mana_preserves_the_actual_payment() {
    for kind in [Kind::Crew, Kind::Saddle] {
        let _rules = fixture_rules::activate(&[
            (kind.card(), kind.rules(true)),
            (Card::LlanowarElves, &CREATURE_MANA),
        ]);
        for player in [Player::P0, Player::P1] {
            let mut game = scenario(player);
            let source = spawn(&mut game, player, kind.card(), Zone::Battlefield);
            let mana_source = put(&mut game, player, "Llanowar Elves", Zone::Battlefield);
            let other = put(&mut game, player, "Hill Giant", Zone::Battlefield);
            game.state.players[player.index()].mana = ZERO;
            let donor = game.object_ref(mana_source);
            let replacement = game.object_ref(other);
            act(&mut game, announce(source));
            act(&mut game, kind.choose(donor));
            same_step(
                &mut game,
                Action::ActivateMana {
                    card: mana_source,
                    ability: 0,
                    payment: ZERO,
                    counter_payment: Counters::default(),
                    production: ONE,
                },
            );
            assert!(game.object_view(mana_source).tapped);
            reject(&mut game, kind.finish());
            same_step(&mut game, Action::EndManaWindow);
            same_step(&mut game, Action::ResetCostSelection);
            assert_eq!(game.object_ref(mana_source), donor);
            assert!(game.object_view(mana_source).tapped);
            assert_eq!(game.state.players[player.index()].mana, ONE);
            reject(&mut game, kind.choose(donor));
            act(&mut game, kind.choose(replacement));
            act(&mut game, kind.finish());
            same_step(&mut game, Action::PayCosts { payment: ONE });
            assert!(game.object_view(mana_source).tapped);
            assert!(game.object_view(other).tapped);
        }
    }
}

#[test]
fn same_source_mana_children_cannot_reset_suspended_contributor_selections() {
    for kind in [Kind::Crew, Kind::Saddle] {
        let rules = match kind {
            Kind::Crew => &NESTED_CREW,
            Kind::Saddle => &NESTED_SADDLE,
        };
        let _rules = fixture_rules::activate(&[(kind.card(), rules)]);
        for player in [Player::P0, Player::P1] {
            for complete_child in [false, true] {
                let mut game = scenario(player);
                let source = spawn(&mut game, player, kind.card(), Zone::Battlefield);
                let donor = put(&mut game, player, "Hill Giant", Zone::Battlefield);
                let donor = game.object_ref(donor);
                game.state.players[player.index()].mana = if complete_child { ONE } else { ZERO };
                act(&mut game, announce(source));
                act(&mut game, kind.choose(donor));
                let parent_context = game.state.announcing.as_ref().unwrap().item.id;
                let parent_attempt = game.attempt_view(player).unwrap().attempt_id;
                same_step(
                    &mut game,
                    Action::AnnounceMana {
                        card: source,
                        ability: 1,
                        counter_payment: Counters::default(),
                        production: ONE,
                    },
                );
                assert_eq!(game.state.pending, Pending::PayCosts { player });
                assert_ne!(
                    game.state.announcing.as_ref().unwrap().item.id,
                    parent_context
                );
                assert_eq!(
                    game.attempt_view(player).unwrap().parent_attempt_id,
                    Some(parent_attempt)
                );
                assert!(!game.attempt_view(player).unwrap().can_reset_selection);
                reject(&mut game, Action::ResetCostSelection);
                match kind {
                    Kind::Crew => {
                        assert!(game.crew_choice(player).is_none());
                        assert_eq!(
                            game.state
                                .crew
                                .as_ref()
                                .unwrap()
                                .payment
                                .as_ref()
                                .unwrap()
                                .selected,
                            [donor]
                        );
                    }
                    Kind::Saddle => {
                        assert!(game.saddle_choice().is_none());
                        assert_eq!(
                            game.state.saddle_payment.as_ref().unwrap().selected,
                            [donor]
                        );
                    }
                }
                // Observing the child must not construct a parent contributor
                // choice against the child's unrelated mana activation.
                assert!(
                    game.observe(player)
                        .attempt
                        .unwrap()
                        .parent_attempt_id
                        .is_some()
                );
                if complete_child {
                    same_step(&mut game, Action::PayCosts { payment: ONE });
                } else {
                    same_step(&mut game, Action::EndManaWindow);
                    same_step(&mut game, Action::SubmitAttempt);
                    assert_eq!(
                        game.last_attempt_outcome_view(player).unwrap().reason,
                        Some(AttemptFailureReason::UnsatisfiedMana)
                    );
                }
                assert_eq!(game.state.pending, kind.pending(player));
                assert_eq!(
                    game.state.announcing.as_ref().unwrap().item.id,
                    parent_context
                );
                assert_eq!(
                    game.attempt_view(player).unwrap().attempt_id,
                    parent_attempt
                );
                assert_eq!(kind.selected(&game, player), [donor]);
                assert!(!game.object_view(donor.card).tapped);
                assert!(game.attempt_view(player).unwrap().can_reset_selection);
                same_step(&mut game, Action::ResetCostSelection);
                assert!(kind.selected(&game, player).is_empty());
                assert!(!game.object_view(donor.card).tapped);
            }
        }
    }
}
