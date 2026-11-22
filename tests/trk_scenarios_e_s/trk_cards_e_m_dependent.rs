//! Star Trek E–M cards using reviewed shared engine primitives.
use crate::support::*;

use rustmage as _;
use rustmage::CardName;
use rustmage_core::cards::authored::CardRules;
use rustmage_core::{cards::*, engine::*};

fn owned(game: &mut Game, player: Player, name: &str, zone: Zone) -> CardId {
    spawn(game, player, Card::from_name(name).unwrap(), zone)
}
fn drain(game: &mut Game) {
    for _ in 0..48 {
        if game.state.pending == Pending::Priority
            && game.state.stack.is_empty()
            && game.state.pending_triggers.is_empty()
            && game.state.resolving.is_none()
        {
            return;
        }
        choose(game, |a| {
            matches!(a, Action::Pass | Action::PutTrigger { .. })
        });
    }
    panic!("resolution budget");
}

const NAMED_ACTIVATION: CardRules = CardRules {
    effect: Effect::Permanent,
    activated: &[ActivatedAbility {
        exhaust: false,
        guard: ActivationGuard::Always,
        origin: ActivationOrigin::Battlefield,
        cycling: false,
        cost: ActivationCost {
            tap: true,
            ..ActivationCost::ZERO
        },
        targets: TargetPlan::NONE,
        program: &[Instruction::GainLife(PlayerSet::Controller, 1)],
        sorcery: false,
        once_per_turn: false,
    }],
    ..CardRules::EMPTY
};

#[test]
fn chekov_selects_opponents_exiled_land_privately_and_grants_only_that_card_play() {
    let player = Player::P0;
    let mut game = scenario(player);
    for name in ["Forest", "Forest", "Island"] {
        put(&mut game, player, name, Zone::Battlefield);
    }
    let untouched = put(&mut game, player.other(), "Mountain", Zone::Library);
    let chosen = put(&mut game, player.other(), "Forest", Zone::Library);
    let unchosen = put(&mut game, player.other(), "Bear Cub", Zone::Library);
    let chekov = owned(&mut game, player, "Ensign Pavel Chekov", Zone::Hand);
    cast(&mut game, chekov, None);
    for _ in 0..12 {
        if matches!(game.state.pending, Pending::MilledSelection { .. }) {
            break;
        }
        choose(&mut game, |a| {
            matches!(a, Action::Pass | Action::PutTrigger { .. })
        });
    }
    assert_eq!(game.object_view(untouched).zone, Zone::Library);
    assert_eq!(game.object_view(unchosen).zone, Zone::Exile);
    assert_eq!(
        game.observe(player).milled_choice.unwrap().candidates,
        [game.object_ref(chosen)]
    );
    assert!(game.observe(player.other()).milled_choice.is_none());
    let unchosen_ref = game.object_ref(unchosen);
    reject(
        &mut game,
        Action::SelectMilled {
            object: unchosen_ref,
        },
    );
    let action = Action::SelectMilled {
        object: game.object_ref(chosen),
    };
    let checkpoint = game.snapshot();
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
    indexed.restore(&checkpoint).unwrap();
    assert_eq!(
        indexed.apply_index(player, revision, index),
        Err(Error::StaleRevision)
    );
    drain(&mut game);
    assert_eq!(
        game.observe(player).exile_permissions[0].object,
        game.object_ref(chosen)
    );
    reject(&mut game, Action::PlayLand { card: unchosen });
    act(&mut game, Action::PlayLand { card: chosen });
    assert_eq!(game.object_view(chosen).zone, Zone::Battlefield);
    assert_eq!(game.object_view(chosen).owner, player.other());
}

#[test]
fn gowron_grows_for_other_deaths_and_klingon_sacrifice_pays_before_resolution() {
    for (sacrifice, qualifies) in [("General Chang, Cold Warrior", true), ("Bear Cub", false)] {
        let player = Player::P0;
        let mut game = scenario(player);
        let gowron = owned(
            &mut game,
            player,
            "Gowron, Eager for Battle",
            Zone::Battlefield,
        );
        let material = owned(&mut game, player, sacrifice, Zone::Battlefield);
        put(&mut game, player, "Island", Zone::Library);
        act(
            &mut game,
            Action::AnnounceActivate {
                card: gowron,
                ability: 0,
                counter_payment: Counters::default(),
            },
        );
        let gowron_ref = game.object_ref(gowron);
        let material_ref = game.object_ref(material);
        reject(
            &mut game,
            Action::ChooseAdditionalCost { object: gowron_ref },
        );
        act(
            &mut game,
            Action::ChooseAdditionalCost {
                object: material_ref,
            },
        );
        pay(&mut game);
        drain(&mut game);
        assert_eq!(game.object_view(material).zone, Zone::Graveyard);
        assert_eq!(game.counter_count(gowron, CounterKind::PlusOne), 1);
        assert!(game.keywords(gowron).contains(Keywords::TRAMPLE));
        assert_eq!(
            game.state.players[player.index()].cards_drawn_this_turn,
            u64::from(qualifies)
        );
        assert_eq!(
            game.state.players[player.index()].life,
            20 - i128::from(qualifies)
        );
    }
}

#[test]
fn kai_choice_observes_opponent_spell_and_named_source_activation() {
    let _fixture =
        fixture_rules::activate(&[(Card::from_name("Short Bow").unwrap(), &NAMED_ACTIVATION)]);
    for (name, activate) in [("Opt", false), ("Short Bow", true)] {
        let player = Player::P0;
        let mut game = scenario(player);
        let kai = owned(&mut game, player, "Kai Winn, Faithless Leader", Zone::Hand);
        cast(&mut game, kai, None);
        for _ in 0..8 {
            if matches!(game.state.pending, Pending::EntryValue { .. }) {
                break;
            }
            choose(&mut game, |a| {
                matches!(a, Action::Pass | Action::PutTrigger { .. })
            });
        }
        reject(
            &mut game,
            Action::ChooseEntryValue {
                value: EntryChoiceValue::CardName(CardName::from_name("Forest").unwrap()),
            },
        );
        act(
            &mut game,
            Action::ChooseEntryValue {
                value: EntryChoiceValue::CardName(CardName::from_name(name).unwrap()),
            },
        );
        drain(&mut game);
        let source = if activate {
            put(&mut game, player.other(), "Short Bow", Zone::Battlefield)
        } else {
            put(&mut game, player.other(), "Opt", Zone::Hand)
        };
        act(&mut game, Action::Pass);
        if activate {
            act(
                &mut game,
                Action::AnnounceActivate {
                    card: source,
                    ability: 0,
                    counter_payment: Counters::default(),
                },
            );
            pay(&mut game);
        } else {
            cast(&mut game, source, None);
        }
        for _ in 0..16 {
            if game.state.players[player.other().index()].life == 17 {
                break;
            }
            choose(&mut game, |a| {
                matches!(a, Action::Pass | Action::PutTrigger { .. })
            });
        }
        assert_eq!(game.state.players[player.other().index()].life, 17);
    }
}

#[test]
fn miles_optional_etb_return_and_each_end_step_history_are_exact() {
    let player = Player::P0;
    let mut game = scenario(player);
    let other = put(&mut game, player, "Bear Cub", Zone::Battlefield);
    let miles = owned(
        &mut game,
        player,
        "Miles O'Brien, Transporter Ops",
        Zone::Hand,
    );
    cast(&mut game, miles, None);
    let choice = Action::ChooseTarget {
        target: Target::Card(other),
    };
    for _ in 0..12 {
        if game.legal_actions().contains(&choice) {
            break;
        }
        choose(&mut game, |a| {
            matches!(a, Action::Pass | Action::PutTrigger { .. })
        });
    }
    reject(
        &mut game,
        Action::ChooseTarget {
            target: Target::Card(miles),
        },
    );
    act(&mut game, choice);
    if game.legal_actions().contains(&Action::FinishTargets) {
        act(&mut game, Action::FinishTargets);
    }
    drain(&mut game);
    assert_eq!(game.object_view(other).zone, Zone::Hand);
    assert!(
        game.state.players[player.index()]
            .turn_facts
            .permanent_returned_to_your_hand
    );
    game.state.step = Step::Main2;
    game.give_priority();
    act(&mut game, Action::Pass);
    act(&mut game, Action::Pass);
    drain(&mut game);
    assert_eq!(game.state.players[player.index()].cards_drawn_this_turn, 1);
    let checkpoint = game.snapshot();
    let mut branch = game.clone();
    branch.restore(&checkpoint).unwrap();
    assert_eq!(branch.snapshot(), checkpoint);
}
