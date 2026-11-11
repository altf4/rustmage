//! Committed cast origins, per-turn casting limits, and exact zone cost increases.
mod support;
use rustmage as _;
use rustmage_core::cards::authored::CardRules;
use rustmage_core::{cards::*, engine::*};
use support::*;

const HAND_HISTORY: CardRules = CardRules {
    effect: Effect::Permanent,
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::AtStep {
            step: TriggerStep::End,
            relation: CastRelation::You,
            guard: StepGuard::NoSpellCastFromOwnHandThisTurn,
        },
        targets: TargetPlan::NONE,
        program: &[Instruction::GainLife(PlayerSet::Controller, 3)],
    }],
    ..CardRules::EMPTY
};

const CAST_LIMIT: CardRules = CardRules {
    effect: Effect::Permanent,
    static_groups: &[StaticAbilityGroup {
        line: 0,
        first_layer: AbilityLayer::Rule,
        fragments: &[StaticAbilityFragment::SpellCastLimit(1)],
    }],
    ..CardRules::EMPTY
};

const ORIGIN_TAX: CardRules = CardRules {
    effect: Effect::Permanent,
    static_groups: &[StaticAbilityGroup {
        line: 0,
        first_layer: AbilityLayer::Rule,
        fragments: &[StaticAbilityFragment::OpponentGraveyardOrExileSpellTax(2)],
    }],
    ..CardRules::EMPTY
};

fn fixture_card() -> Card {
    Card::from_name("Short Bow").unwrap()
}

fn finish_targets(game: &mut Game) {
    if game.legal_actions().contains(&Action::FinishTargets) {
        act(game, Action::FinishTargets);
    }
}

#[test]
fn own_hand_history_ignores_lands_plotting_and_exile_casts_and_rechecks() {
    let _rules = fixture_rules::activate(&[(fixture_card(), &HAND_HISTORY)]);
    for player in [Player::P0, Player::P1] {
        let mut game = scenario(player);
        put(&mut game, player, "Short Bow", Zone::Battlefield);
        let land = put(&mut game, player, "Forest", Zone::Hand);
        act(&mut game, Action::PlayLand { card: land });
        let coyote = put(&mut game, player, "Cunning Coyote", Zone::Hand);
        act(&mut game, Action::AnnouncePlot { card: coyote });
        choose(&mut game, |action| matches!(action, Action::PayPlot { .. }));
        let exile = put(&mut game, player, "Shock", Zone::Exile);
        game.grant_exile_play_permission(player, game.object_ref(exile), false);
        act(&mut game, Action::AnnounceExile { card: exile });
        act(
            &mut game,
            Action::ChooseTarget {
                target: Target::Player(player.other()),
            },
        );
        finish_targets(&mut game);
        pay(&mut game);
        settle(&mut game);
        let facts = game.observe(player).turn_facts.unwrap()[player.index()];
        assert!(facts.played_from_outside_own_hand);
        assert!(!facts.cast_spell_from_own_hand);

        let hand = put(&mut game, player, "Shock", Zone::Hand);
        game.begin_step(Step::End);
        game.give_priority();
        game.prepare_triggers();
        assert_eq!(game.state.stack.len(), 1);
        let mut response = game.clone();
        cast(&mut response, hand, Some(Target::Player(player.other())));
        settle(&mut response);
        assert_eq!(response.state.players[player.index()].life, 20);
        assert!(
            response.observe(player).turn_facts.unwrap()[player.index()].cast_spell_from_own_hand
        );

        settle(&mut game);
        assert_eq!(game.state.players[player.index()].life, 23);
    }
}

#[test]
fn per_player_cast_limit_counts_a_countered_spell_before_the_provider_entered() {
    let _rules = fixture_rules::activate(&[(fixture_card(), &CAST_LIMIT)]);
    let player = Player::P1;
    let mut game = scenario(player);
    let first = put(&mut game, player, "Shock", Zone::Hand);
    let negate = put(&mut game, player.other(), "Negate", Zone::Hand);
    cast(&mut game, first, Some(Target::Player(player.other())));
    if game.actor() != Some(player.other()) {
        act(&mut game, Action::Pass);
    }
    cast(&mut game, negate, Some(Target::Spell(first)));
    settle(&mut game);
    assert_eq!(game.object_view(first).zone, Zone::Graveyard);

    let provider = put(&mut game, player, "Short Bow", Zone::Battlefield);
    let second = put(&mut game, player, "Shock", Zone::Hand);
    let plotted = put(&mut game, player, "Cunning Coyote", Zone::Hand);
    reject(&mut game, Action::AnnounceCast { card: second });
    act(&mut game, Action::AnnouncePlot { card: plotted });
    choose(&mut game, |action| matches!(action, Action::PayPlot { .. }));
    assert_eq!(game.object_view(plotted).zone, Zone::Exile);

    let opponent_second = put(&mut game, player.other(), "Shock", Zone::Hand);
    let mountain = put(&mut game, player.other(), "Mountain", Zone::Battlefield);
    let mut off_turn = game.clone();
    act(&mut off_turn, Action::Pass);
    reject(
        &mut off_turn,
        Action::AnnounceCast {
            card: opponent_second,
        },
    );
    off_turn.move_object(provider, Zone::Graveyard);
    act(&mut off_turn, Action::TapMana { card: mountain });
    cast(&mut off_turn, opponent_second, Some(Target::Player(player)));
    settle(&mut off_turn);
    assert_eq!(off_turn.state.players[player.index()].life, 18);

    game.begin_step(Step::End);
    let turn = game.state.turn;
    for _ in 0..12 {
        if game.state.turn > turn {
            break;
        }
        act(&mut game, Action::Pass);
    }
    assert!(game.state.turn > turn);
    if game.actor() != Some(player.other()) {
        act(&mut game, Action::Pass);
    }
    act(&mut game, Action::TapMana { card: mountain });
    cast(&mut game, opponent_second, Some(Target::Player(player)));
    settle(&mut game);
    assert_eq!(game.state.players[player.index()].life, 18);
}

#[test]
fn opposing_graveyard_and_exile_casts_pay_generic_tax_from_their_exact_origin() {
    let _rules = fixture_rules::activate(&[(fixture_card(), &ORIGIN_TAX)]);
    for player in [Player::P0, Player::P1] {
        for zone in [Zone::Graveyard, Zone::Exile] {
            let mut game = scenario(player);
            let provider = put(&mut game, player.other(), "Short Bow", Zone::Battlefield);
            let creature = put(&mut game, player, "Grizzly Bears", zone);
            let object = game.object_ref(creature);
            let announce = if zone == Zone::Graveyard {
                game.grant_graveyard_permission(player, object, GraveyardCastKind::Cast);
                Action::AnnounceCast { card: creature }
            } else {
                game.grant_exile_play_permission(player, object, false);
                Action::AnnounceExile { card: creature }
            };

            game.state.players[player.index()].mana = [0, 0, 0, 0, 1, 1];
            reject(&mut game, announce.clone());
            let mut without_provider = game.clone();
            without_provider.move_object(provider, Zone::Graveyard);
            act(&mut without_provider, announce.clone());
            pay(&mut without_provider);
            settle(&mut without_provider);
            assert_eq!(
                without_provider.object_view(creature).zone,
                Zone::Battlefield
            );

            game.state.players[player.index()].mana = [0, 0, 0, 0, 1, 3];
            act(&mut game, announce);
            pay(&mut game);
            assert_eq!(game.state.players[player.index()].mana, [0; 6]);
            assert_eq!(game.state.stack[0].cast.unwrap().origin, zone);
            settle(&mut game);
            assert_eq!(game.object_view(creature).zone, Zone::Battlefield);
        }
    }
}

#[test]
fn plotted_spree_needs_both_mode_mana_and_the_exile_tax() {
    let _rules = fixture_rules::activate(&[(fixture_card(), &ORIGIN_TAX)]);
    let player = Player::P0;
    let mut game = scenario(player);
    put(&mut game, player.other(), "Short Bow", Zone::Battlefield);
    let creature = put(
        &mut game,
        player.other(),
        "Grizzly Bears",
        Zone::Battlefield,
    );
    let spell = put(&mut game, player, "Explosive Derailment", Zone::Exile);
    game.remember_plotted(player, game.object_ref(spell));
    game.state.turn += 1;
    let announce = Action::AnnouncePlotted { card: spell };

    game.state.players[player.index()].mana = [0, 0, 0, 0, 0, 3];
    reject(&mut game, announce.clone());
    game.state.players[player.index()].mana = [0, 0, 0, 0, 0, 4];
    let checkpoint = game.snapshot();
    let mut indexed = game.clone();
    let index = game
        .legal_actions()
        .iter()
        .position(|action| *action == announce)
        .unwrap();
    indexed.apply_index(player, game.revision(), index).unwrap();
    act(&mut game, announce.clone());
    assert_eq!(indexed.snapshot(), game.snapshot());
    reject(&mut game, Action::ChooseMode { mode: 2 });
    for branch in [&mut game, &mut indexed] {
        act(branch, Action::ChooseMode { mode: 0 });
        act(
            branch,
            Action::ChooseTarget {
                target: Target::Card(creature),
            },
        );
        finish_targets(branch);
        pay(branch);
        assert_eq!(branch.state.players[player.index()].mana, [0; 6]);
        settle(branch);
    }
    assert_eq!(indexed.snapshot(), game.snapshot());
    assert_eq!(game.object_view(creature).zone, Zone::Graveyard);
    indexed.restore(&checkpoint).unwrap();
    assert!(indexed.legal_actions().contains(&announce));
}
