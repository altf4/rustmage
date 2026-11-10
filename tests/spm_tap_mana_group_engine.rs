//! Owned two-creature tap payments for a mana ability nested in other costs.
mod support;
use rustmage as _;
use rustmage_core::cards::authored::CardRules;
use rustmage_core::{cards::*, engine::*};
use support::*;

const WHITE: Mana = [1, 0, 0, 0, 0, 0];
const EMPTY: Mana = [0; 6];
const BASE: ActivatedAbility = ActivatedAbility {
    guard: ActivationGuard::Always,
    origin: ActivationOrigin::Battlefield,
    cycling: false,
    exhaust: false,
    once_per_turn: false,
    sorcery: false,
    cost: ActivationCost::ZERO,
    targets: TargetPlan::NONE,
    program: &[],
};
const TAP_TWO: CardRules = CardRules {
    effect: Effect::Creature(3, 3),
    activated: &[ActivatedAbility {
        cost: ActivationCost {
            additional: Some(AdditionalCost {
                kind: EffectChoiceKind::TapPermanents(2),
                filter: SacrificeFilter {
                    kinds: 1,
                    source_only: false,
                    other: false,
                    nonland: false,
                    tokens: None,
                    subtype: None,
                    keywords: Keywords::NONE,
                    artifact_enchantment_or_token: false,
                },
            }),
            ..ActivationCost::ZERO
        },
        program: &[Instruction::AddMana(ManaProduction::Choice {
            mask: 31,
            amount: 1,
        })],
        ..BASE
    }],
    ..CardRules::EMPTY
};
const OUTER: CardRules = CardRules {
    effect: Effect::Creature(1, 1),
    activated: &[ActivatedAbility {
        cost: ActivationCost {
            colored: WHITE,
            mana_component: true,
            tap: true,
            ..ActivationCost::ZERO
        },
        program: &[Instruction::Draw(PlayerSet::Controller, 1)],
        ..BASE
    }],
    ..CardRules::EMPTY
};

fn new_source(game: &mut Game, player: Player) -> CardId {
    let source = put(game, player, "Grizzly Bears", Zone::Battlefield);
    game.state.objects[source as usize].summoning_sick = true;
    source
}

fn begin_mana(game: &mut Game, source: CardId) {
    choose(game, |action| {
        matches!(action, Action::AnnounceMana { card, production, .. }
        if *card == source && [WHITE, EMPTY].contains(production))
    });
}

fn tap_choice(game: &Game, card: CardId) -> Action {
    Action::ChooseAdditionalCost {
        object: game.object_ref(card),
    }
}

fn pay_mana_ability(game: &mut Game, player: Player) {
    pay(game);
    if matches!(game.state.pending, Pending::EffectMana { .. }) {
        act(game, Action::ChooseMana { production: WHITE });
    }
    assert_eq!(game.state.players[player.index()].mana, WHITE);
    assert!(game.state.stack.is_empty());
}

#[test]
fn two_summoning_sick_creatures_can_fund_the_only_cast_route_and_rollback_as_a_group() {
    let _rules = fixture_rules::activate(&[(Card::GrizzlyBears, &TAP_TWO)]);
    let player = Player::P0;
    let opponent = player.other();
    let mut game = scenario(player);
    game.state.players[player.index()].mana = EMPTY;
    let source = new_source(&mut game, player);
    let enemy = put(&mut game, opponent, "Hill Giant", Zone::Battlefield);
    let artifact = put(&mut game, player, "Swiftfoot Boots", Zone::Battlefield);
    let spell = put(&mut game, player, "Healer's Hawk", Zone::Hand);
    let announce = Action::AnnounceCast { card: spell };
    reject(&mut game, announce.clone());
    let witness = put(&mut game, player, "Hill Giant", Zone::Battlefield);
    game.state.objects[witness as usize].summoning_sick = true;

    // Keep the ordinary mana receipt for the focused failed-payment replay below.
    game.begin_attack_rollback(player);
    act(&mut game, announce);
    begin_mana(&mut game, source);
    let enemy_tap = tap_choice(&game, enemy);
    let artifact_tap = tap_choice(&game, artifact);
    reject(&mut game, enemy_tap);
    reject(&mut game, artifact_tap);
    let source_tap = tap_choice(&game, source);
    let witness_tap = tap_choice(&game, witness);
    act(&mut game, source_tap.clone());
    reject(&mut game, source_tap);
    assert!(!game.object_view(source).tapped);
    let saved = game.snapshot();
    let revision = game.revision();
    let index = game
        .legal_actions()
        .iter()
        .position(|action| *action == witness_tap)
        .unwrap();
    let mut raw = game.clone();
    act(&mut raw, witness_tap);
    game.apply_index(player, revision, index).unwrap();
    assert_eq!(raw.snapshot(), game.snapshot());
    assert!(!game.object_view(witness).tapped);
    game.restore(&saved).unwrap();
    assert_eq!(
        game.apply_index(player, revision, index),
        Err(Error::StaleRevision)
    );
    game.apply_index(player, game.revision(), index).unwrap();
    pay_mana_ability(&mut game, player);
    assert!(game.object_view(source).tapped && game.object_view(witness).tapped);
    assert!(!game.object_view(enemy).tapped && !game.object_view(artifact).tapped);

    let mut failed = game.clone();
    failed.finish_cast_rollback(false);
    assert_eq!(failed.state.pending, Pending::ManaRollback { player });
    for id in [source, witness] {
        let mut stale = failed.clone();
        stale.state.objects[id as usize].generation += 1;
        reject(&mut stale, Action::ChooseManaRollback { keep: true });
    }
    for keep in [false, true] {
        let mut branch = failed.clone();
        act(&mut branch, Action::ChooseManaRollback { keep });
        assert_eq!(branch.object_view(source).tapped, keep);
        assert_eq!(branch.object_view(witness).tapped, keep);
        assert_eq!(
            branch.state.players[player.index()].mana,
            if keep { WHITE } else { EMPTY }
        );
        assert_eq!(branch.object_view(spell).zone, Zone::Hand);
    }
    pay(&mut game);
    settle(&mut game);
    assert_eq!(game.object_view(spell).zone, Zone::Battlefield);
}

#[test]
fn nested_tap_payment_preserves_the_enclosing_activations_source_tap() {
    let _rules = fixture_rules::activate(&[
        (Card::GrizzlyBears, &TAP_TWO),
        (Card::LlanowarElves, &OUTER),
    ]);
    let player = Player::P0;
    let mut game = scenario(player);
    game.state.players[player.index()].mana = EMPTY;
    let source = new_source(&mut game, player);
    let outer = put(&mut game, player, "Llanowar Elves", Zone::Battlefield);
    let outer_activation = Action::AnnounceActivate {
        card: outer,
        ability: 0,
        counter_payment: Counters::default(),
    };
    reject(&mut game, outer_activation.clone());
    let witness = put(&mut game, player, "Hill Giant", Zone::Battlefield);
    act(&mut game, outer_activation);
    begin_mana(&mut game, source);
    let illegal = tap_choice(&game, outer);
    reject(&mut game, illegal);
    for id in [source, witness] {
        let choice = tap_choice(&game, id);
        act(&mut game, choice);
    }
    pay_mana_ability(&mut game, player);
    assert!(!game.object_view(outer).tapped);
    pay(&mut game);
    assert!(game.object_view(outer).tapped);
    settle(&mut game);
    assert_eq!(game.hand_count(player), 1);
}

#[test]
fn tap_group_sources_and_convoke_share_the_untapped_creature_pool() {
    let _rules = fixture_rules::activate(&[(Card::GrizzlyBears, &TAP_TWO)]);
    let player = Player::P0;
    let mut game = scenario(player);
    game.state.players[player.index()].mana = EMPTY;
    for _ in 0..2 {
        put(&mut game, player, "Forest", Zone::Battlefield);
        let source = new_source(&mut game, player);
        game.state.objects[source as usize].tapped = true;
    }
    for _ in 0..5 {
        put(&mut game, player, "Hill Giant", Zone::Battlefield);
    }
    put(&mut game, player.other(), "Hill Giant", Zone::Battlefield);
    let gwenom = put(&mut game, player, "Gwenom, Remorseless", Zone::Hand);
    let web = put(&mut game, player, "Web of Life and Destiny", Zone::Hand);
    reject(&mut game, Action::AnnounceCast { card: gwenom });
    reject(&mut game, Action::AnnounceCast { card: web });

    put(&mut game, player, "Hill Giant", Zone::Battlefield);
    let legal = game.legal_actions();
    assert!(legal.contains(&Action::AnnounceCast { card: gwenom }));
    assert!(legal.contains(&Action::AnnounceCast { card: web }));
}
