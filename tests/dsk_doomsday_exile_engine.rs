//! Retain exact library bottoms while moving both opaque batches in one event.
mod support;
use rustmage as _;
use rustmage_core::cards::authored::CardRules;
use rustmage_core::cards::*;
use rustmage_core::engine::*;
use support::*;

const EXILE_TRIGGER_SOURCE: &str = "Hill Giant";
const RULES: CardRules = CardRules {
    effect: Effect::Creature(6, 6),
    keywords: Keywords::FLYING,
    triggers: &[
        TriggeredAbility {
            condition: TriggerCondition::SelfEntersIfCast,
            targets: TargetPlan::NONE,
            program: &[Instruction::ExileEachLibraryExceptBottomFaceDown(6)],
        },
        TriggeredAbility {
            condition: TriggerCondition::AtStep {
                step: TriggerStep::Upkeep,
                relation: CastRelation::You,
                guard: StepGuard::Always,
            },
            targets: TargetPlan::NONE,
            program: &[Instruction::Draw(PlayerSet::Controller, 1)],
        },
    ],
    ..CardRules::EMPTY
};

fn arrange_library(game: &mut Game, player: Player, count: usize) -> Vec<ObjectRef> {
    for id in game.state.players[player.index()].library.clone() {
        game.move_object(id, Zone::Outside);
    }
    let names = [
        "Island",
        "Bear Cub",
        "Hill Giant",
        "Plains",
        "Opt",
        "Swamp",
        "Forest",
        "Hill Giant",
        "Mountain",
    ];
    (0..count)
        .map(|position| {
            let card = put(game, player, names[position % names.len()], Zone::Library);
            game.object_ref(card)
        })
        .collect()
}

fn expect_library(game: &Game, player: Player, exact: &[ObjectRef]) {
    let actual: Vec<_> = game.state.players[player.index()]
        .library
        .iter()
        .map(|&id| game.object_ref(id))
        .collect();
    assert_eq!(
        actual.as_slice(),
        exact,
        "{player:?} library from bottom to top"
    );
}

fn expect_opaque_exile(game: &Game, departed: &[ObjectRef]) {
    for &before in departed {
        let view = game.object_view(before.card);
        assert_eq!(view.zone, Zone::Exile);
        assert_eq!(view.reference.generation, before.generation + 1);
        assert_eq!(view.kinds, 0);
        assert!(game.exiled_face_down(view.reference));
    }
    for viewer in [Player::P0, Player::P1] {
        let observed = game.observe(viewer);
        assert!(observed.known_library_top.iter().all(Option::is_none));
        assert!(observed.last_library_reveal.is_none());
        assert!(observed.zone_reveals.is_empty());
        for &before in departed {
            let exact = game.object_ref(before.card);
            assert!(!game.can_view_hideaway(viewer, exact));
            assert!(observed.objects.iter().all(|view| view.id != before.card));
            assert!(observed.exile_permissions.iter().all(|p| p.object != exact));
        }
    }
}

fn cast_until_exile_trigger(game: &mut Game, controller: Player, from: Zone) {
    let source = put(game, controller, EXILE_TRIGGER_SOURCE, from);
    if from == Zone::Exile {
        game.grant_exile_play_permission_with_expiry(
            controller,
            game.object_ref(source),
            None,
            ExilePlayExpiry::WhileExiled,
        )
        .unwrap();
        act(game, Action::AnnounceExile { card: source });
        pay(game);
    } else {
        cast(game, source, None);
    }
    act(game, Action::Pass);
    act(game, Action::Pass);
    if game.state.stack.is_empty() {
        choose(game, |action| matches!(action, Action::PutTrigger { .. }));
    }
    assert_eq!(game.state.stack.len(), 1);
    let trigger = game.state.stack.last().unwrap();
    assert_eq!(trigger.source.unwrap().id, source);
    assert!(matches!(trigger.kind, StackKind::TriggeredAbility { .. }));
    assert!(trigger.targets.is_empty());
}

#[test]
fn both_libraries_exile_exact_opaque_cards_in_one_resolution() {
    let _rules =
        fixture_rules::activate(&[(Card::from_name(EXILE_TRIGGER_SOURCE).unwrap(), &RULES)]);
    for controller in [Player::P0, Player::P1] {
        let opponent = controller.other();
        let mut parent = scenario(controller);
        let libraries = [
            arrange_library(&mut parent, Player::P0, 11),
            arrange_library(&mut parent, Player::P1, 9),
        ];
        let response = put(&mut parent, opponent, "Quick Study", Zone::Hand);
        let origin = if controller == Player::P0 {
            Zone::Hand
        } else {
            Zone::Exile
        };
        cast_until_exile_trigger(&mut parent, controller, origin);
        act(&mut parent, Action::Pass);
        assert_eq!(parent.actor(), Some(opponent));
        let finish = Action::Pass;
        let index = parent
            .legal_actions()
            .iter()
            .position(|action| *action == finish)
            .unwrap();
        let saved = parent.snapshot();
        let revision = parent.revision();
        let rng = parent.state.rng;
        let mut raw = parent.clone();
        let mut indexed = parent.clone();
        let mut reordered = parent.clone();
        drop(parent);
        for player in [Player::P0, Player::P1] {
            reordered.state.players[player.index()].library[6..].reverse();
        }

        assert_eq!(
            raw.apply(controller, finish.clone()),
            Err(Error::WrongActor)
        );
        assert_eq!(raw.snapshot(), saved);
        act(&mut raw, finish.clone());
        indexed.apply_index(opponent, revision, index).unwrap();
        act(&mut reordered, finish);
        assert_eq!(raw.snapshot(), indexed.snapshot());
        assert_eq!(raw.state.pending, Pending::Priority);
        assert!(raw.state.stack.is_empty());
        assert_eq!(raw.state.rng, rng);
        assert_eq!(reordered.state.rng, rng);

        for player in [Player::P0, Player::P1] {
            let library = &libraries[player.index()];
            expect_library(&raw, player, &library[..6]);
            expect_opaque_exile(&raw, &library[6..]);
            assert_eq!(
                raw.observe(player.other()).players[player.index()].library_count,
                6
            );
            assert_eq!(
                serde_json::to_value(raw.observe(player)).unwrap(),
                serde_json::to_value(reordered.observe(player)).unwrap()
            );
        }
        assert!(raw.observe(controller).cards_exiled_this_turn);
        reject(
            &mut raw,
            Action::AnnounceExile {
                card: libraries[controller.index()][7].card,
            },
        );
        reject(
            &mut raw,
            Action::PlayLand {
                card: libraries[controller.index()][6].card,
            },
        );

        indexed.restore(&saved).unwrap();
        assert_eq!(
            indexed.apply_index(opponent, revision, index),
            Err(Error::StaleRevision)
        );
        cast(&mut indexed, response, None);
        settle(&mut indexed);
        let opponent_cards = &libraries[opponent.index()];
        let drawn = &opponent_cards[opponent_cards.len() - 2..];
        for card in drawn {
            assert_eq!(indexed.object_view(card.card).zone, Zone::Hand);
            assert!(!indexed.exiled_face_down(indexed.object_ref(card.card)));
            assert_eq!(raw.object_view(card.card).zone, Zone::Exile);
        }
        expect_library(&indexed, opponent, &opponent_cards[..6]);
        expect_opaque_exile(&indexed, &opponent_cards[6..opponent_cards.len() - 2]);
    }
}

#[test]
fn short_libraries_and_prevented_moves_preserve_the_protected_bottom() {
    struct ClearPrevention;
    impl Drop for ClearPrevention {
        fn drop(&mut self) {
            zone_replacements::TEST_PREVENT_ZONE_MOVE.with(|slot| slot.set(None));
        }
    }
    let _clear = ClearPrevention;
    let _rules =
        fixture_rules::activate(&[(Card::from_name(EXILE_TRIGGER_SOURCE).unwrap(), &RULES)]);
    let player = Player::P0;
    for (short, long, prevent) in [(0, 6, false), (5, 9, true)] {
        let mut game = scenario(player);
        let smaller = arrange_library(&mut game, player, short);
        let larger = arrange_library(&mut game, player.other(), long);
        cast_until_exile_trigger(&mut game, player, Zone::Hand);
        let prevented = prevent.then(|| larger[6]);
        zone_replacements::TEST_PREVENT_ZONE_MOVE.with(|slot| slot.set(prevented));
        settle(&mut game);

        expect_library(&game, player, &smaller);
        let kept = if prevent { 7 } else { 6 };
        expect_library(&game, player.other(), &larger[..kept]);
        expect_opaque_exile(&game, &larger[kept..]);
        if let Some(exact) = prevented {
            assert!(!game.exiled_face_down(exact));
        }
        assert_eq!(game.observe(player).cards_exiled_this_turn, prevent);
    }
}

#[test]
fn noncast_entry_keeps_libraries_and_its_controller_still_draws_on_upkeep() {
    let _rules =
        fixture_rules::activate(&[(Card::from_name(EXILE_TRIGGER_SOURCE).unwrap(), &RULES)]);
    let player = Player::P1;
    let mut game = scenario(player);
    let own = arrange_library(&mut game, player, 8);
    let opponent = arrange_library(&mut game, player.other(), 9);
    let source = put(&mut game, player, EXILE_TRIGGER_SOURCE, Zone::Graveyard);
    let return_spell = put(&mut game, player, "Zombify", Zone::Hand);
    cast(&mut game, return_spell, Some(Target::Card(source)));
    settle(&mut game);
    assert_eq!(game.object_view(source).zone, Zone::Battlefield);
    expect_library(&game, player, &own);
    expect_library(&game, player.other(), &opponent);

    for _ in 0..64 {
        if game.state.active == player && game.state.step == Step::Upkeep {
            break;
        }
        choose(&mut game, |action| {
            matches!(
                action,
                Action::Pass | Action::FinishAttackers | Action::FinishBlockers
            )
        });
    }
    assert_eq!(game.state.active, player);
    assert_eq!(game.state.step, Step::Upkeep);
    settle(&mut game);
    expect_library(&game, player, &own[..7]);
    assert_eq!(game.object_view(own[7].card).zone, Zone::Hand);
    assert_eq!(game.state.players[player.index()].cards_drawn_this_turn, 1);
}
