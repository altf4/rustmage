//! Exact hand exile links and the shared-type test for a Saga's later chapter.
use crate::support::*;

use rustmage as _;
use rustmage_core::cards::authored::CardRules;
use rustmage_core::{cards::*, engine::*};

const HAND_CARD: SacrificeFilter = SacrificeFilter {
    keywords: Keywords::NONE,
    source_only: false,
    kinds: 0,
    nonland: false,
    other: false,
    tokens: Some(false),
    artifact_enchantment_or_token: false,
    subtype: None,
};
const DRAW_AND_LINK: &[Instruction] = &[
    Instruction::Draw(PlayerSet::Controller, 2),
    Instruction::ChooseObjects {
        kind: EffectChoiceKind::ExileFromHandLinked,
        players: PlayerSet::Controller,
        filter: HAND_CARD,
        amount: 1,
        optional_skip: None,
    },
];
const SAGA: CardRules = CardRules {
    effect: Effect::Permanent,
    triggers: &[
        TriggeredAbility {
            condition: TriggerCondition::Chapter { number: 1 },
            targets: TargetPlan::NONE,
            program: DRAW_AND_LINK,
        },
        TriggeredAbility {
            condition: TriggerCondition::Chapter { number: 2 },
            targets: TargetPlan::NONE,
            program: DRAW_AND_LINK,
        },
        TriggeredAbility {
            condition: TriggerCondition::Chapter { number: 3 },
            targets: TargetPlan::NONE,
            program: DRAW_AND_LINK,
        },
        TriggeredAbility {
            condition: TriggerCondition::Chapter { number: 4 },
            targets: TargetPlan::NONE,
            program: &[
                Instruction::IfLinkedExileSharesCardType { skip: 1 },
                Instruction::GainLife(PlayerSet::Controller, 10),
            ],
        },
    ],
    ..CardRules::EMPTY
};

fn setup() -> (fixture_rules::Guard, Game, CardId) {
    let saga_card = Card::from_name("Amok Time").unwrap();
    let rules = fixture_rules::activate(&[(saga_card, &SAGA)]);
    let mut game = scenario(Player::P0);
    let saga = put(&mut game, Player::P0, "Amok Time", Zone::Battlefield);
    (rules, game, saga)
}

fn drive(game: &mut Game) {
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
    panic!("Saga resolution did not settle: {:?}", game.state.pending);
}

fn reach_hand_exile(game: &mut Game) {
    game.begin_step(Step::Main1);
    game.prepare_triggers();
    for _ in 0..24 {
        if game.state.pending == (Pending::EffectChoice { player: Player::P0 }) {
            return;
        }
        choose(game, |a| {
            matches!(a, Action::Pass | Action::PutTrigger { .. })
        });
    }
    panic!(
        "Saga chapter did not request a hand card: {:?}",
        game.state.pending
    );
}

fn exile_chapter(game: &mut Game, card: CardId) -> ObjectRef {
    reach_hand_exile(game);
    assert!(game.observe(Player::P1).effect_choice.is_none());
    let before = game.object_ref(card);
    act(game, Action::SelectResolutionExile { object: before });
    drive(game);
    assert_eq!(game.object_view(card).zone, Zone::Exile);
    game.object_ref(card)
}

fn final_chapter(game: &mut Game, saga: CardId, depart_before_resolution: bool) {
    game.state.objects[saga as usize].counters.lore = 3;
    game.begin_step(Step::Main1);
    game.prepare_triggers();
    if depart_before_resolution {
        // The fourth chapter keeps its exact source when the Saga leaves after
        // triggering but before that packet resolves.
        if game
            .legal_actions()
            .iter()
            .any(|a| matches!(a, Action::PutTrigger { .. }))
        {
            choose(game, |a| matches!(a, Action::PutTrigger { .. }));
        }
        game.move_object(saga, Zone::Graveyard);
    }
    drive(game);
}

#[test]
fn chosen_hand_card_links_exactly_and_the_choice_stays_private_and_owned() {
    let (_rules, mut game, saga) = setup();
    let chosen = put(&mut game, Player::P0, "Short Bow", Zone::Hand);
    let other = put(&mut game, Player::P0, "Grizzly Bears", Zone::Hand);
    let theirs = put(&mut game, Player::P1, "Shock", Zone::Hand);
    let source = game.object_ref(saga);
    reach_hand_exile(&mut game);
    let action = Action::SelectResolutionExile {
        object: game.object_ref(chosen),
    };
    let theirs_ref = game.object_ref(theirs);
    reject(
        &mut game,
        Action::SelectResolutionExile { object: theirs_ref },
    );
    assert!(game.observe(Player::P1).effect_choice.is_none());
    let before = game.snapshot();
    let index = game
        .legal_actions()
        .iter()
        .position(|a| *a == action)
        .unwrap();
    let mut indexed = game.clone();
    indexed
        .apply_index(Player::P0, game.revision(), index)
        .unwrap();
    act(&mut game, action);
    assert_eq!(game.snapshot(), indexed.snapshot());
    drive(&mut game);
    assert_eq!(game.object_view(chosen).zone, Zone::Exile);
    assert_eq!(game.object_view(other).zone, Zone::Hand);
    assert_eq!(
        game.linked_exile_for_origin(source, None),
        [game.object_ref(chosen)]
    );
    indexed.restore(&before).unwrap();
    assert_eq!(indexed.snapshot(), before);
    assert!(indexed.linked_exile_for_origin(source, None).is_empty());
}

#[test]
fn fourth_chapter_uses_intersection_of_only_still_exiled_card_types() {
    for (names, departing, should_gain) in [
        (&[][..], false, true),
        (&["Short Bow"][..], false, true),
        (&["Short Bow", "Adaptive Automaton"][..], false, true),
        (&["Short Bow", "Grizzly Bears"][..], false, false),
        (
            &["Short Bow", "Adaptive Automaton", "Enduring Courage"][..],
            false,
            false,
        ),
        (&["Short Bow", "Grizzly Bears"][..], true, true),
    ] {
        let (_rules, mut game, saga) = setup();
        let source = game.object_ref(saga);
        let mut exiled = Vec::new();
        for &name in names {
            let card = put(&mut game, Player::P0, name, Zone::Hand);
            exiled.push((card, exile_chapter(&mut game, card)));
        }
        if departing {
            game.move_object(exiled[1].0, Zone::Hand);
            assert_eq!(game.linked_exile_for_origin(source, None), [exiled[0].1]);
        }
        let life = game.state.players[0].life;
        final_chapter(&mut game, saga, false);
        assert_eq!(
            game.state.players[0].life,
            life + if should_gain { 10 } else { 0 },
            "{names:?}, departing={departing}"
        );
    }
}

#[test]
fn fourth_chapter_keeps_exact_links_after_saga_departs_and_reentry_resets_them() {
    let (_rules, mut game, saga) = setup();
    let source = game.object_ref(saga);
    let artifact = put(&mut game, Player::P0, "Short Bow", Zone::Hand);
    let exiled = exile_chapter(&mut game, artifact);
    assert_eq!(game.linked_exile_for_origin(source, None), [exiled]);
    let before = game.snapshot();
    let life = game.state.players[0].life;
    final_chapter(&mut game, saga, true);
    assert_eq!(game.state.players[0].life, life + 10);
    game.move_object(saga, Zone::Battlefield);
    let returned = game.object_ref(saga);
    assert_ne!(returned, source);
    assert!(game.linked_exile_for_origin(returned, None).is_empty());
    game.restore(&before).unwrap();
    assert_eq!(game.snapshot(), before);
}
