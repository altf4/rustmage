//! A mandatory graveyard land return and a live land-creature stat bonus.
mod support;

use rustmage as _;
use rustmage_core::cards::authored::CardRules;
use rustmage_core::cards::*;
use rustmage_core::engine::characteristic_layers::AnimationTypeMode;
use rustmage_core::engine::*;
use support::*;

const BONUSES: &[StaticAbilityGroup] = &[StaticAbilityGroup {
    line: 0,
    first_layer: AbilityLayer::Modify,
    fragments: &[StaticAbilityFragment::ControlledCreatureKindBonus {
        kinds: 2,
        stats: Stats {
            power: 1,
            toughness: 1,
        },
    }],
}];
const TURTLE: CardRules = CardRules {
    effect: Effect::Creature(3, 3),
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::SelfEntersOrAttacks,
        targets: TargetPlan::NONE,
        program: &[
            Instruction::Mill(PlayerSet::Controller, MillAmount::Fixed(3)),
            Instruction::ChooseOwnGraveyardLandReturn { tapped: true },
        ],
    }],
    static_groups: BONUSES,
    ..CardRules::EMPTY
};
const POWER_LAND: CardRules = CardRules {
    effect: Effect::Land,
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
        program: &[Instruction::AddMana(ManaProduction::SourcePower {
            color: 5,
        })],
        sorcery: false,
        once_per_turn: false,
    }],
    ..CardRules::EMPTY
};

fn card(name: &str) -> Card {
    Card::from_name(name).unwrap()
}

fn clear_library(game: &mut Game, player: Player) {
    for id in game.state.players[player.index()].library.clone() {
        game.move_object(id, Zone::Exile);
    }
}

fn resolve_top(game: &mut Game) {
    act(game, Action::Pass);
    act(game, Action::Pass);
}

fn select(game: &Game, id: CardId) -> Action {
    Action::SelectLand {
        object: game.object_ref(id),
    }
}

#[test]
fn entry_mills_before_one_mandatory_choice_from_the_whole_owned_graveyard() {
    let _rules = fixture_rules::activate(&[(card("Hill Giant"), &TURTLE)]);
    let player = Player::P0;
    let mut parent = scenario(player);
    clear_library(&mut parent, player);
    let source = put(&mut parent, player, "Hill Giant", Zone::Hand);
    let old = put(&mut parent, player, "Plains", Zone::Graveyard);
    let fresh = put(&mut parent, player, "Forest", Zone::Library);
    let one = put(&mut parent, player, "Opt", Zone::Library);
    let two = put(&mut parent, player, "Shock", Zone::Library);
    let hand = put(&mut parent, player, "Mountain", Zone::Hand);
    let foreign = put(&mut parent, player.other(), "Island", Zone::Graveyard);
    let fresh_before = parent.object_ref(fresh);

    cast(&mut parent, source, None);
    resolve_top(&mut parent);
    assert_eq!(parent.object_view(source).zone, Zone::Battlefield);
    resolve_top(&mut parent);
    assert_eq!(parent.object_view(one).zone, Zone::Graveyard);
    assert_eq!(parent.object_view(two).zone, Zone::Graveyard);
    let fresh_after = parent.object_ref(fresh);
    assert_ne!(fresh_before, fresh_after);
    for viewer in [player, player.other()] {
        let view = parent.observe(viewer).effect_choice.unwrap();
        assert!(!view.optional);
        assert_eq!(view.required[player.index()], 1);
        assert_eq!(view.candidates, [parent.object_ref(old), fresh_after]);
    }
    reject(&mut parent, Action::FinishLandSelection);
    reject(
        &mut parent,
        Action::SelectLand {
            object: fresh_before,
        },
    );
    for invalid in [hand, foreign, one] {
        let action = select(&parent, invalid);
        reject(&mut parent, action);
    }

    let action = select(&parent, fresh);
    let index = parent
        .legal_actions()
        .iter()
        .position(|candidate| *candidate == action)
        .unwrap();
    let saved = parent.snapshot();
    let mut raw = parent.clone();
    let mut indexed = parent.clone();
    act(&mut raw, action);
    indexed
        .apply_index(player, parent.revision(), index)
        .unwrap();
    for branch in [&raw, &indexed] {
        assert_eq!(branch.object_view(fresh).zone, Zone::Battlefield);
        assert!(branch.object_view(fresh).tapped);
        assert_eq!(branch.object_view(old).zone, Zone::Graveyard);
    }
    assert_eq!(raw.snapshot(), indexed.snapshot());

    raw.restore(&saved).unwrap();
    let action = select(&raw, old);
    act(&mut raw, action);
    assert_eq!(raw.object_view(old).zone, Zone::Battlefield);
    assert!(raw.object_view(old).tapped);
    assert_eq!(raw.object_view(fresh).zone, Zone::Graveyard);
    assert_eq!(parent.snapshot(), saved);
}

#[test]
fn attack_returns_an_old_land_after_an_empty_mill_and_cannot_return_exiled_mill() {
    let _rules = fixture_rules::activate(&[(card("Hill Giant"), &TURTLE)]);
    let player = Player::P1;
    for exile_mill in [false, true] {
        let mut game = scenario(player);
        clear_library(&mut game, player);
        let turtle = put(&mut game, player, "Hill Giant", Zone::Battlefield);
        let candidate = if exile_mill {
            put(
                &mut game,
                player.other(),
                "Rest in Peace",
                Zone::Battlefield,
            );
            put(&mut game, player, "Plains", Zone::Library)
        } else {
            put(&mut game, player, "Plains", Zone::Graveyard)
        };
        game.state.step = Step::Attackers;
        game.state.pending = Pending::Attackers;
        act(&mut game, Action::Attack { card: turtle });
        act(&mut game, Action::FinishAttackers);
        resolve_top(&mut game);
        if exile_mill {
            assert!(game.observe(player).effect_choice.is_none());
            assert_eq!(game.object_view(candidate).zone, Zone::Exile);
        } else {
            reject(&mut game, Action::FinishLandSelection);
            let action = select(&game, candidate);
            act(&mut game, action);
            assert_eq!(game.object_view(candidate).zone, Zone::Battlefield);
            assert!(game.object_view(candidate).tapped);
        }
    }
}

fn animate(game: &mut Game, land: CardId) {
    game.change_object_form(
        game.object_ref(land),
        AnimationSpec {
            artifact: false,
            creature_types: AnimationCreatureTypes::Named(&["Elemental"]),
            base: Some((2, 2)),
            keywords: Keywords::NONE,
            duration: AnimationDuration::EndOfTurn,
        },
        AnimationTypeMode::AddCreature,
    );
}

#[test]
fn the_live_bonus_tracks_type_control_and_abilities_in_play_and_funding() {
    let _rules = fixture_rules::activate(&[
        (card("Hill Giant"), &TURTLE),
        (card("Crystal Grotto"), &POWER_LAND),
    ]);
    let player = Player::P0;
    let mut game = scenario(player);
    let land = put(&mut game, player, "Crystal Grotto", Zone::Battlefield);
    let plain_land = put(&mut game, player, "Crystal Grotto", Zone::Battlefield);
    let foreign = put(
        &mut game,
        player.other(),
        "Crystal Grotto",
        Zone::Battlefield,
    );
    let plain_creature = put(&mut game, player, "Grizzly Bears", Zone::Battlefield);
    let spell = put(&mut game, player, "Adaptive Automaton", Zone::Hand);
    game.state.objects[plain_land as usize].tapped = true;
    game.state.players[player.index()].mana = [0; 6];
    let cast = Action::AnnounceCast { card: spell };
    animate(&mut game, land);
    animate(&mut game, foreign);
    reject(&mut game, cast.clone());
    let turtle = put(&mut game, player, "Hill Giant", Zone::Battlefield);
    let expected = Stats {
        power: 3,
        toughness: 3,
    };
    assert_eq!(game.stats(land), expected);
    assert_eq!(game.stats(foreign).power, 2);
    assert_eq!(game.stats(plain_creature).power, 2);
    assert_eq!(game.stats(plain_land).power, 0);

    let mut transferred = game.clone();
    transferred.change_controller(transferred.object_ref(turtle), player.other());
    assert_eq!(transferred.stats(land).power, 2);
    assert_eq!(transferred.stats(foreign), expected);
    reject(&mut transferred, cast.clone());

    let mut stripped = game.clone();
    stripped.remove_abilities_exact(
        &[stripped.object_ref(turtle)],
        stripped.object_ref(foreign),
        player.other(),
        AnimationDuration::EndOfTurn,
    );
    assert_eq!(stripped.stats(land).power, 2);
    reject(&mut stripped, cast.clone());

    act(&mut game, cast);
    choose(
        &mut game,
        |action| matches!(action, Action::ActivateMana { card, .. } if *card == land),
    );
    assert_eq!(game.state.players[player.index()].mana, [0, 0, 0, 0, 0, 3]);
    pay(&mut game);
    resolve_top(&mut game);
    choose(&mut game, |action| {
        matches!(action, Action::ChooseEntryValue { .. })
    });
    assert_eq!(game.object_view(spell).zone, Zone::Battlefield);
}
