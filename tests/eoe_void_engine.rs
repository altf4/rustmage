//! Void follows actual public events across both players and the turn boundary.
mod support;
use rustmage as _;
use rustmage_core::cards::authored::{CardExtras, CardRules};
use rustmage_core::{cards::*, engine::*};
use support::*;

const DISCOUNT: SpellReduction = SpellReduction {
    source_only: true,
    filter: CastFilter::Any,
    amount: SpellReductionAmount::Fixed(1),
    turn: CostTurn::Any,
    guard: StepGuard::VoidActive,
};

const CONDITIONAL_SPELL: CardRules = CardRules {
    effect: Effect::Spell(
        TargetPlan::NONE,
        &[
            Instruction::If {
                guard: StepGuard::VoidActive,
                skip: 1,
            },
            Instruction::Draw(PlayerSet::Controller, 1),
        ],
    ),
    extras: &CardExtras {
        cost_reductions: &[DISCOUNT],
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};

const END_STEP_OBSERVER: CardRules = CardRules {
    effect: Effect::Creature(2, 2),
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::AtStep {
            step: TriggerStep::End,
            relation: CastRelation::You,
            guard: StepGuard::VoidActive,
        },
        targets: TargetPlan::NONE,
        program: &[Instruction::GainLife(PlayerSet::Controller, 1)],
    }],
    ..CardRules::EMPTY
};

fn assert_void(game: &Game, active: bool) {
    for player in [Player::P0, Player::P1] {
        assert_eq!(game.observe(player).void_active_this_turn, active);
    }
}

fn finish_turn(game: &mut Game) {
    let turn = game.state.turn;
    for _ in 0..48 {
        if game.state.turn != turn {
            return;
        }
        choose(game, |action| {
            matches!(
                action,
                Action::Pass
                    | Action::FinishAttackers
                    | Action::FinishBlockers
                    | Action::PutTrigger { .. }
            )
        });
    }
    panic!("turn did not end");
}

#[test]
fn void_requires_nonland_departure_and_resets_after_the_end_step() {
    let _rules = fixture_rules::activate(&[
        (Card::from_name("Pilfer").unwrap(), &CONDITIONAL_SPELL),
        (Card::from_name("Bear Cub").unwrap(), &END_STEP_OBSERVER),
    ]);
    for player in [Player::P0, Player::P1] {
        let mut parent = scenario(player);
        let land = put(&mut parent, player, "Forest", Zone::Battlefield);
        let artifact = put(&mut parent, player, "Chrome Companion", Zone::Battlefield);
        let observer = put(&mut parent, player, "Bear Cub", Zone::Battlefield);
        let scrapper = put(&mut parent, player, "Slagdrill Scrapper", Zone::Battlefield);
        let spell = put(&mut parent, player, "Pilfer", Zone::Hand);
        act(
            &mut parent,
            Action::AnnounceActivate {
                card: scrapper,
                ability: 0,
                counter_payment: Counters::default(),
            },
        );
        let checkpoint = parent.snapshot();
        let revision = parent.revision();
        let sacrifice_artifact = Action::ChooseAdditionalCost {
            object: parent.object_ref(artifact),
        };
        let index = parent
            .legal_actions()
            .iter()
            .position(|action| *action == sacrifice_artifact)
            .unwrap();
        let mut raw = parent.clone();
        let mut indexed = parent.clone();
        let mut land_only = parent;
        act(&mut raw, sacrifice_artifact);
        indexed.apply_index(player, revision, index).unwrap();
        let sacrifice_land = Action::ChooseAdditionalCost {
            object: land_only.object_ref(land),
        };
        act(&mut land_only, sacrifice_land);
        for game in [&mut raw, &mut indexed, &mut land_only] {
            assert_void(game, false);
            pay(game);
            settle(game);
            game.state.players[player.index()].mana = [0, 0, 1, 0, 0, 0];
        }
        assert_eq!(raw.snapshot(), indexed.snapshot());
        assert_eq!(raw.object_view(artifact).zone, Zone::Graveyard);
        assert_eq!(land_only.object_view(land).zone, Zone::Graveyard);
        assert_void(&raw, true);
        assert_void(&land_only, false);

        let announcement = Action::AnnounceCast { card: spell };
        reject(&mut land_only, announcement.clone());
        let library_before = raw.state.players[player.index()].library.len();
        act(&mut raw, announcement);
        act(
            &mut raw,
            Action::PayCosts {
                payment: [0, 0, 1, 0, 0, 0],
            },
        );
        settle(&mut raw);
        assert_eq!(
            raw.state.players[player.index()].library.len(),
            library_before - 1
        );
        for game in [&mut raw, &mut land_only] {
            finish_turn(game);
            assert_void(game, false);
            assert_eq!(game.object_view(observer).zone, Zone::Battlefield);
        }
        assert_eq!(raw.state.players[player.index()].life, STARTING_LIFE + 1);
        assert_eq!(land_only.state.players[player.index()].life, STARTING_LIFE);
        indexed.restore(&checkpoint).unwrap();
        assert_void(&indexed, false);
        assert_eq!(indexed.object_view(artifact).zone, Zone::Battlefield);
    }
}

#[test]
fn void_counts_a_committed_warp_even_when_the_spell_is_countered() {
    let player = Player::P0;
    let mut game = scenario(player);
    let warped = put(&mut game, player, "Bygone Colossus", Zone::Hand);
    let counter = put(&mut game, player.other(), "Cancel", Zone::Hand);
    let mut normal = game.clone();
    cast(&mut normal, warped, None);
    settle(&mut normal);
    assert_void(&normal, false);

    act(&mut game, Action::AnnounceWarp { card: warped });
    assert_void(&game, false);
    reject(&mut game, Action::PayCosts { payment: [0; 6] });
    assert_void(&game, false);
    pay(&mut game);
    assert_void(&game, true);
    act(&mut game, Action::Pass);
    cast(&mut game, counter, Some(Target::Spell(warped)));
    settle(&mut game);
    assert_eq!(game.object_view(warped).zone, Zone::Graveyard);
    assert_void(&game, true);
    finish_turn(&mut game);
    assert_void(&game, false);
}
