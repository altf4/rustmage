//! A resolution cost can consume an exact once-per-turn use without a second prompt.
mod support;

use rustmage as _;
use rustmage_core::cards::authored::CardRules;
use rustmage_core::{cards::*, engine::*};
use support::*;

const RUMMAGE: CardRules = CardRules {
    effect: Effect::Creature(2, 2),
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::OtherFilteredEntersYou(EntryPredicate::CreaturePowerAtMost(2)),
        targets: TargetPlan::NONE,
        program: &[
            Instruction::IfOptionalAbilityUnused { skip: 3 },
            Instruction::MayPayCosts {
                costs: &[ResolutionCost {
                    objects: Some(ResolutionObjectCost {
                        cost: AdditionalCost {
                            kind: EffectChoiceKind::Discard,
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
                        },
                        count: 1,
                    }),
                    ..ResolutionCost::FREE
                }],
                skip: 2,
            },
            Instruction::RecordOncePerTurn,
            Instruction::Draw(PlayerSet::Controller, 1),
        ],
    }],
    ..CardRules::EMPTY
};

const COPY: CardRules = CardRules {
    effect: Effect::Spell(
        TargetPlan::Single(TargetSpec::one(TargetRule::YourAbility)),
        &[Instruction::CopyTargetSpellOrAbility { group: 0 }],
    ),
    ..CardRules::EMPTY
};

fn pass_pair(game: &mut Game) {
    act(game, Action::Pass);
    act(game, Action::Pass);
}

fn enter(game: &mut Game, player: Player, name: &str) {
    let creature = put(game, player, name, Zone::Exile);
    game.move_object(creature, Zone::Battlefield);
    game.prepare_triggers();
}

fn discard_choice(game: &Game, card: CardId) -> Action {
    Action::ChooseAdditionalCost {
        object: game.object_ref(card),
    }
}

fn pay_discard(game: &mut Game) {
    choose(game, |action| {
        matches!(action, Action::PayResolutionMana { .. })
    });
}

#[test]
fn an_empty_hand_and_declined_cost_preserve_use_but_an_exiled_discard_consumes_it() {
    let _rules = fixture_rules::activate(&[(Card::GrizzlyBears, &RUMMAGE)]);
    let player = Player::P0;
    let mut game = scenario(player);
    put(&mut game, player, "Grizzly Bears", Zone::Battlefield);

    enter(&mut game, player, "Bear Cub");
    pass_pair(&mut game);
    assert_eq!(game.legal_actions(), vec![Action::DeclineResolutionMana]);
    reject(&mut game, Action::ChooseOptional { accept: true });
    act(&mut game, Action::DeclineResolutionMana);

    let discarded = put(&mut game, player, "Forest", Zone::Hand);
    enter(&mut game, player, "Bear Cub");
    pass_pair(&mut game);
    let selection = discard_choice(&game, discarded);
    assert!(game.legal_actions().contains(&selection));
    act(&mut game, Action::DeclineResolutionMana);
    assert_eq!(game.object_view(discarded).zone, Zone::Hand);
    assert_eq!(game.state.players[player.index()].cards_drawn_this_turn, 0);

    put(
        &mut game,
        player.other(),
        "Rest in Peace",
        Zone::Battlefield,
    );
    enter(&mut game, player, "Bear Cub");
    pass_pair(&mut game);
    let saved = game.snapshot();
    let revision = game.revision();
    let index = game
        .legal_actions()
        .iter()
        .position(|action| *action == selection)
        .unwrap();
    let mut raw = game.clone();
    act(&mut raw, selection.clone());
    game.apply_index(player, revision, index).unwrap();
    assert_eq!(game.snapshot(), raw.snapshot());
    assert!(
        game.observe(player.other())
            .resolution_payment
            .unwrap()
            .selected_objects
            .is_empty()
    );
    assert_eq!(game.object_view(discarded).zone, Zone::Hand);

    game.restore(&saved).unwrap();
    assert_eq!(
        game.apply_index(player, revision, index),
        Err(Error::StaleRevision)
    );
    act(&mut game, selection);
    for branch in [&mut raw, &mut game] {
        pay_discard(branch);
        assert_eq!(branch.object_view(discarded).zone, Zone::Exile);
        assert_eq!(
            branch.state.players[player.index()].cards_drawn_this_turn,
            1
        );
        enter(branch, player, "Bear Cub");
        assert!(branch.state.stack.is_empty());
    }
    assert_eq!(game.snapshot(), raw.snapshot());
}

#[test]
fn paying_for_a_copy_skips_the_original_and_another_queued_trigger() {
    let opt = Card::from_name("Opt").unwrap();
    let _rules = fixture_rules::activate(&[(Card::GrizzlyBears, &RUMMAGE), (opt, &COPY)]);
    let player = Player::P0;
    let mut game = scenario(player);
    put(&mut game, player, "Grizzly Bears", Zone::Battlefield);
    let discarded = put(&mut game, player, "Forest", Zone::Hand);
    let remaining = put(&mut game, player, "Shock", Zone::Hand);
    let copy = spawn(&mut game, player, opt, Zone::Hand);
    enter(&mut game, player, "Bear Cub");
    enter(&mut game, player, "Bear Cub");
    assert_eq!(game.state.stack.len(), 2);

    let target = Target::Stack(game.state.stack.last().unwrap().id);
    cast(&mut game, copy, Some(target));
    pass_pair(&mut game);
    assert!(game.state.stack.last().unwrap().is_copy);
    pass_pair(&mut game);
    let selection = discard_choice(&game, discarded);
    act(&mut game, selection);
    pay_discard(&mut game);
    settle(&mut game);

    assert_eq!(game.object_view(discarded).zone, Zone::Graveyard);
    assert_eq!(game.object_view(remaining).zone, Zone::Hand);
    assert_eq!(game.state.players[player.index()].cards_drawn_this_turn, 1);
}

#[test]
fn a_control_change_keeps_the_use_scoped_to_the_original_controller() {
    let _rules = fixture_rules::activate(&[(Card::GrizzlyBears, &RUMMAGE)]);
    let mut game = scenario(Player::P0);
    let source = put(&mut game, Player::P0, "Grizzly Bears", Zone::Battlefield);
    let source = game.object_ref(source);
    for controller in [Player::P0, Player::P1] {
        game.change_controller(source, controller);
        let discarded = put(&mut game, controller, "Forest", Zone::Hand);
        enter(&mut game, controller, "Bear Cub");
        pass_pair(&mut game);
        assert_eq!(game.actor(), Some(controller));
        let selection = discard_choice(&game, discarded);
        act(&mut game, selection);
        pay_discard(&mut game);
        assert_eq!(game.object_view(discarded).zone, Zone::Graveyard);
        assert_eq!(
            game.state.players[controller.index()].cards_drawn_this_turn,
            1
        );
    }
    game.change_controller(source, Player::P0);
    enter(&mut game, Player::P0, "Bear Cub");
    assert!(game.state.stack.is_empty());
}
