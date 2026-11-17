//! Current Crew/Saddle/Station choices remain editable and explicitly recoverable.
mod support;
use rustmage::Card;
use rustmage_core::cards::authored::CardRules;
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
            ..ActivationCost::ZERO
        },
        program: &[
            Instruction::AddMana(ManaProduction::Fixed(ONE)),
            Instruction::CreateTokens {
                token: Card(5304),
                amount: 3,
                tapped: false,
            },
        ],
        ..BASE
    }],
    ..CardRules::EMPTY
};

#[derive(Clone, Copy)]
enum Kind {
    Crew,
    Saddle,
}
impl Kind {
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
        .unwrap();
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
        let _rules = fixture_rules::activate(&[(Card::GrizzlyBears, kind.rules(false))]);
        for player in [Player::P0, Player::P1] {
            let mut game = scenario(player);
            let source = put(&mut game, player, "Grizzly Bears", Zone::Battlefield);
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
            (Card::GrizzlyBears, kind.rules(false)),
            (Card::LlanowarElves, &NEGATIVE),
        ]);
        for player in [Player::P0, Player::P1] {
            let mut game = scenario(player);
            let source = put(&mut game, player, "Grizzly Bears", Zone::Battlefield);
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
            same_step(&mut game, Action::SubmitAttempt);
            assert_eq!(game.state.pending, kind.pending(player));
            assert_eq!(
                game.last_attempt_outcome_view(player).unwrap().outcome,
                AttemptOutcome::Incomplete
            );
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
        let _rules = fixture_rules::activate(&[
            (Card::GrizzlyBears, kind.rules(true)),
            (producer, &MATERIAL_MANA),
        ]);
        for player in [Player::P0, Player::P1] {
            let mut game = scenario(player);
            let source = put(&mut game, player, "Grizzly Bears", Zone::Battlefield);
            let mana_source = spawn(&mut game, player, producer, Zone::Battlefield);
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
            let tokens: Vec<_> = game
                .controlled_token_ids(player)
                .map(|id| game.object_ref(id))
                .collect();
            assert_eq!(tokens.len(), 3);
            assert!(game.object_view(mana_source).tapped);
            assert_eq!(game.state.players[player.index()].mana, ONE);
            same_step(&mut game, kind.choose(tokens[0]));
            reject(&mut game, kind.finish());
            same_step(&mut game, Action::EndManaWindow);
            same_step(&mut game, Action::ResetCostSelection);
            assert!(game.attempt_view(player).unwrap().mana_window_closed);
            assert!(kind.selected(&game, player).is_empty());
            assert!(game.object_view(mana_source).tapped);
            assert_eq!(game.state.players[player.index()].mana, ONE);
            assert_eq!(game.controlled_token_ids(player).count(), 3);
            reject(&mut game, mana);
            for &object in &tokens {
                same_step(&mut game, kind.choose(object));
            }
            same_step(&mut game, kind.finish());
            assert_eq!(game.state.pending, Pending::PayCosts { player });
            assert!(
                tokens
                    .iter()
                    .all(|object| !game.object_view(object.card).tapped)
            );
            // Finish selected the group but has not paid the taxed activation.
            same_step(&mut game, Action::ResetCostSelection);
            assert_eq!(game.state.pending, kind.pending(player));
            for &object in &tokens {
                act(&mut game, kind.choose(object));
            }
            act(&mut game, kind.finish());
            same_step(&mut game, Action::PayCosts { payment: ONE });
            assert!(
                tokens
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
            (Card::GrizzlyBears, kind.rules(true)),
            (Card::LlanowarElves, &CREATURE_MANA),
        ]);
        for player in [Player::P0, Player::P1] {
            let mut game = scenario(player);
            let source = put(&mut game, player, "Grizzly Bears", Zone::Battlefield);
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
