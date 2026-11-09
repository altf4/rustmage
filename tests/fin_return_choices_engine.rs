//! Non-targeted exact object choices that return cards to their owners' hands.
mod support;
use rustmage as _;
use rustmage_core::cards::authored::CardRules;
use rustmage_core::{cards::*, engine::*};
use support::*;

const ANY: SacrificeFilter = SacrificeFilter {
    keywords: Keywords::NONE,
    source_only: false,
    kinds: 0,
    nonland: false,
    other: false,
    tokens: None,
    artifact_enchantment_or_token: false,
    subtype: None,
};
const PERMANENT_KINDS: u16 = 1 | 2 | 16 | 32 | 64 | 256;

const RETURN_ANOTHER: CardRules = CardRules {
    effect: Effect::Creature(2, 2),
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::SelfEnters,
        targets: TargetPlan::NONE,
        program: &[Instruction::ChooseObjects {
            kind: EffectChoiceKind::ReturnToHand,
            players: PlayerSet::Controller,
            filter: SacrificeFilter { other: true, ..ANY },
            amount: 1,
            optional_skip: Some(0),
        }],
    }],
    ..CardRules::EMPTY
};

const RETURN_LAND: CardRules = CardRules {
    effect: Effect::Creature(2, 2),
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::AtStep {
            step: TriggerStep::End,
            relation: CastRelation::You,
            guard: StepGuard::Always,
        },
        targets: TargetPlan::NONE,
        program: &[Instruction::ChooseObjects {
            kind: EffectChoiceKind::ReturnToHand,
            players: PlayerSet::Controller,
            filter: SacrificeFilter { kinds: 2, ..ANY },
            amount: 1,
            optional_skip: None,
        }],
    }],
    ..CardRules::EMPTY
};

const MILL_AND_RETURN: CardRules = CardRules {
    effect: Effect::Creature(2, 2),
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::SelfEnters,
        targets: TargetPlan::NONE,
        program: &[
            Instruction::Mill(PlayerSet::Controller, MillAmount::Fixed(2)),
            Instruction::ChooseObjects {
                kind: EffectChoiceKind::ReturnGraveyardToHand {
                    kinds: PERMANENT_KINDS,
                },
                players: PlayerSet::Controller,
                filter: ANY,
                amount: 1,
                optional_skip: None,
            },
        ],
    }],
    ..CardRules::EMPTY
};

fn await_choice(game: &mut Game, player: Player) {
    for _ in 0..12 {
        if matches!(game.state.pending, Pending::EffectChoice { .. }) {
            assert_eq!(game.actor(), Some(player));
            assert!(
                game.state
                    .resolving
                    .as_ref()
                    .unwrap()
                    .item
                    .targets
                    .is_empty()
            );
            return;
        }
        assert_eq!(game.state.pending, Pending::Priority);
        act(game, Action::Pass);
    }
    panic!("no return choice");
}

#[test]
fn optional_bounce_chooses_another_controlled_incarnation_and_returns_to_owner() {
    let source_name = "Bear Cub";
    let player = Player::P0;
    let opponent = player.other();
    let _rules =
        fixture_rules::activate(&[(Card::from_name(source_name).unwrap(), &RETURN_ANOTHER)]);
    let mut parent = scenario(player);
    let source = put(&mut parent, player, source_name, Zone::Hand);
    let stolen = put(&mut parent, opponent, "Llanowar Elves", Zone::Battlefield);
    parent.state.objects[stolen as usize].controller = player;
    parent.state.objects[stolen as usize].gained_keywords = Keywords::PROTECTION_EVERYTHING;
    let opposing = put(&mut parent, opponent, "Grizzly Bears", Zone::Battlefield);
    let land = put(&mut parent, player, "Forest", Zone::Battlefield);
    let stale = parent.object_ref(land);
    parent.move_object(land, Zone::Exile);
    parent.move_object(land, Zone::Battlefield);
    assert_ne!(stale, parent.object_ref(land));
    cast(&mut parent, source, None);
    await_choice(&mut parent, player);

    let selected = parent.object_ref(stolen);
    let bounce = Action::ChooseEffectObject { object: selected };
    let decline = Action::ChooseOptional { accept: false };
    for viewer in [player, opponent] {
        let view = parent.observe(viewer).effect_choice.unwrap();
        assert!(view.optional);
        assert_eq!(view.kind, EffectChoiceKind::ReturnToHand);
        assert_eq!(view.candidates, vec![selected, parent.object_ref(land)]);
        assert!(view.inspected_cards.is_none());
    }
    let actions = parent.legal_actions();
    assert_eq!(actions.len(), 4);
    assert!(actions.contains(&decline));
    for invalid in [
        stale,
        parent.object_ref(source),
        parent.object_ref(opposing),
    ] {
        reject(&mut parent, Action::ChooseEffectObject { object: invalid });
    }
    reject(&mut parent, Action::Sacrifice { object: selected });
    reject(&mut parent, Action::ChooseOptional { accept: true });
    let wire = serde_json::to_value(&bounce).unwrap();
    assert_eq!(wire["kind"], "ChooseEffectObject");
    assert_eq!(serde_json::from_value::<Action>(wire).unwrap(), bounce);

    let saved = parent.snapshot();
    let revision = parent.revision();
    let index = actions.iter().position(|action| *action == bounce).unwrap();
    let mut raw = parent.clone();
    let mut indexed = parent.clone();
    drop(parent);
    assert_eq!(raw.apply(opponent, bounce.clone()), Err(Error::WrongActor));
    assert_eq!(raw.snapshot(), saved);
    act(&mut raw, bounce);
    indexed.apply_index(player, revision, index).unwrap();
    settle(&mut raw);
    settle(&mut indexed);
    assert_eq!(raw.snapshot(), indexed.snapshot());
    let returned = raw.object_view(stolen);
    assert_eq!((returned.zone, returned.owner), (Zone::Hand, opponent));
    assert_ne!(returned.reference, selected);
    assert_eq!(raw.object_view(source).zone, Zone::Battlefield);

    raw.restore(&saved).unwrap();
    act(&mut raw, decline);
    settle(&mut raw);
    assert_eq!(raw.object_view(stolen).zone, Zone::Battlefield);
    assert_eq!(raw.object_view(stolen).controller, player);
    assert_eq!(indexed.object_view(stolen).zone, Zone::Hand);
}

#[test]
fn end_step_bounce_requires_a_controlled_land_when_available() {
    let source_name = "Bear Cub";
    let player = Player::P0;
    let _rules = fixture_rules::activate(&[(Card::from_name(source_name).unwrap(), &RETURN_LAND)]);
    for eligible in [true, false] {
        let mut game = scenario(player);
        let source = put(&mut game, player, source_name, Zone::Battlefield);
        let land = put(&mut game, player.other(), "Island", Zone::Battlefield);
        let other_land = put(&mut game, player.other(), "Forest", Zone::Battlefield);
        if eligible {
            game.state.objects[land as usize].controller = player;
        }
        let chosen = game.object_ref(land);
        game.state.step = Step::Main2;
        act(&mut game, Action::Pass);
        act(&mut game, Action::Pass);
        assert_eq!(game.state.step, Step::End);
        assert_eq!(game.state.stack.len(), 1);
        if eligible {
            await_choice(&mut game, player);
            let action = Action::ChooseEffectObject { object: chosen };
            assert_eq!(game.legal_actions(), vec![Action::Concede, action.clone()]);
            reject(&mut game, Action::ChooseOptional { accept: false });
            act(&mut game, action);
        }
        settle(&mut game);
        assert_eq!(game.object_view(source).zone, Zone::Battlefield);
        assert_eq!(game.object_view(other_land).zone, Zone::Battlefield);
        assert_eq!(
            game.object_view(land).zone,
            if eligible {
                Zone::Hand
            } else {
                Zone::Battlefield
            }
        );
        assert_eq!(game.object_view(land).owner, player.other());
    }
}

#[test]
fn mill_then_recover_chooses_any_owned_permanent_card_in_graveyard() {
    let source_name = "Bear Cub";
    let player = Player::P1;
    let opponent = player.other();
    let _rules =
        fixture_rules::activate(&[(Card::from_name(source_name).unwrap(), &MILL_AND_RETURN)]);
    let mut game = scenario(player);
    let source = put(&mut game, player, source_name, Zone::Hand);
    let existing = put(&mut game, player, "Grizzly Bears", Zone::Graveyard);
    let battle = spawn(
        &mut game,
        player,
        Card::from_name("@fixture/siege-front").unwrap(),
        Zone::Graveyard,
    );
    let opposing = put(&mut game, opponent, "Grizzly Bears", Zone::Graveyard);
    let hidden = put(&mut game, player, "Island", Zone::Hand);
    let instant = put(&mut game, player, "Shock", Zone::Library);
    let land = put(&mut game, player, "Forest", Zone::Library);
    let from_library = game.object_ref(land);
    cast(&mut game, source, None);
    await_choice(&mut game, player);
    assert_eq!(game.object_view(land).zone, Zone::Graveyard);
    assert_eq!(game.object_view(instant).zone, Zone::Graveyard);

    let candidates = [existing, battle, land].map(|id| game.object_ref(id));
    for viewer in [player, opponent] {
        let view = game.observe(viewer).effect_choice.unwrap();
        assert!(!view.optional);
        assert_eq!(
            view.kind,
            EffectChoiceKind::ReturnGraveyardToHand {
                kinds: PERMANENT_KINDS
            }
        );
        assert_eq!(view.candidates, candidates);
        assert!(view.inspected_cards.is_none());
    }
    let invalid = [opposing, instant, hidden, source].map(|id| game.object_ref(id));
    for object in invalid.into_iter().chain([from_library]) {
        reject(&mut game, Action::SelectGraveyardReturn { object });
    }
    reject(&mut game, Action::FinishGraveyardReturn);
    let actions = game.legal_actions();
    assert_eq!(actions.len(), candidates.len() + 1);

    let before = game.snapshot();
    for (id, object) in [existing, battle, land].into_iter().zip(candidates) {
        game.restore(&before).unwrap();
        let revision = game.revision();
        let action = Action::SelectGraveyardReturn { object };
        let index = actions.iter().position(|legal| *legal == action).unwrap();
        let mut indexed = game.clone();
        indexed.apply_index(player, revision, index).unwrap();
        act(&mut game, action);
        settle(&mut game);
        settle(&mut indexed);
        assert_eq!(game.snapshot(), indexed.snapshot());
        let returned = game.object_view(id);
        assert_eq!((returned.zone, returned.owner), (Zone::Hand, player));
        assert_ne!(returned.reference, object);
        for other in [existing, battle, land, opposing, instant] {
            if other != id {
                assert_eq!(game.object_view(other).zone, Zone::Graveyard);
            }
        }
    }
}
