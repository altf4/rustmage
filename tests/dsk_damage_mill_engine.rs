//! The affected player orders prevention and its after-damage mill events.
mod support;

use rustmage as _;
use rustmage_core::cards::authored::{CardExtras, CardRules};
use rustmage_core::{cards::*, engine::*};
use support::*;

const PREVENT: ReplacementSpec = ReplacementSpec::PreventControlledOpponentDamageAndMill;
const PREVENTION_PROVIDER: CardRules = CardRules {
    effect: Effect::Creature(2, 2),
    static_groups: &[StaticAbilityGroup {
        line: 0,
        first_layer: AbilityLayer::Rule,
        fragments: &[StaticAbilityFragment::Replacement(0, PREVENT)],
    }],
    extras: &CardExtras {
        replacement_effects: &[PREVENT],
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
const SHIELD: CardRules = CardRules {
    effect: Effect::Spell(
        TargetPlan::NONE,
        &[Instruction::PreventPlayerDamageUntilNextTurn],
    ),
    ..CardRules::EMPTY
};

fn pass_priority(game: &mut Game) {
    act(game, Action::Pass);
    act(game, Action::Pass);
}

fn replacement(game: &Game, source: CardId) -> Action {
    Action::ChooseReplacement {
        source: game.object_ref(source),
        ability: 0,
    }
}

fn cast_shock(game: &mut Game, player: Player, target: Target) {
    let spell = put(game, player, "Shock", Zone::Hand);
    cast(game, spell, Some(target));
}

#[test]
fn simultaneous_combat_hits_keep_distinct_mills_and_pending_choices() {
    let _rules = fixture_rules::activate(&[(Card::GrizzlyBears, &PREVENTION_PROVIDER)]);
    let player = Player::P1;
    let opponent = player.other();
    let mut game = scenario(player);
    let provider = spawn(&mut game, player, Card::GrizzlyBears, Zone::Battlefield);
    let doubler = put(&mut game, player, "Twinflame Tyrant", Zone::Battlefield);
    put(&mut game, player, "The Water Crystal", Zone::Battlefield);
    let hawk = put(&mut game, player, "Healer's Hawk", Zone::Battlefield);
    let giant = put(&mut game, player, "Hill Giant", Zone::Battlefield);
    for _ in 0..8 {
        put(&mut game, opponent, "Island", Zone::Library);
    }
    pass_priority(&mut game);
    pass_priority(&mut game);
    for card in [hawk, giant] {
        act(&mut game, Action::Attack { card });
    }
    act(&mut game, Action::FinishAttackers);
    pass_priority(&mut game);
    act(&mut game, Action::FinishBlockers);
    pass_priority(&mut game);

    let prevent = replacement(&game, provider);
    let double = replacement(&game, doubler);
    assert_eq!(game.actor(), Some(opponent));
    let first = game
        .observe(opponent)
        .replacement_choice
        .unwrap()
        .damage
        .unwrap();
    assert_eq!(first.target, Target::Player(opponent));
    assert!(first.combat);
    assert!(game.legal_actions().contains(&double));
    let before = game.snapshot();
    assert_eq!(game.apply(player, prevent.clone()), Err(Error::WrongActor));
    assert_eq!(game.snapshot(), before);
    act(&mut game, prevent.clone());

    let pending = game.observe(opponent).replacement_choice.unwrap();
    let second = pending.damage.unwrap();
    assert_eq!(second.target, first.target);
    assert_ne!(second.source.reference, first.source.reference);
    assert_eq!(first.amount + second.amount, 4);
    assert_eq!(pending.damage_mills, [(opponent, first.amount as u128)]);
    assert_eq!(game.library_card_count(opponent), 16);

    let revision = game.revision();
    let index = game
        .legal_actions()
        .iter()
        .position(|choice| *choice == prevent)
        .unwrap();
    let checkpoint = game.snapshot();
    let mut raw = game.clone();
    let mut indexed = game.clone();
    drop(game);
    act(&mut raw, prevent);
    indexed.apply_index(opponent, revision, index).unwrap();
    assert_eq!(indexed.snapshot(), raw.snapshot());
    // Each mill gets The Water Crystal's four extra cards.
    assert_eq!(raw.library_card_count(opponent), 4);
    assert_eq!(raw.library_card_count(player), 8);
    for seat in [player, opponent] {
        assert_eq!(raw.state.players[seat.index()].life, 20);
    }
    assert!(!raw.object_view(hawk).dealt_combat_damage_this_incarnation);
    assert!(!raw.object_view(giant).dealt_combat_damage_this_incarnation);

    indexed.restore(&checkpoint).unwrap();
    act(&mut indexed, double);
    assert_eq!(
        indexed.library_card_count(opponent),
        4 - second.amount as usize
    );
    assert_eq!(indexed.state.players[opponent.index()].life, 20);
}

#[test]
fn the_affected_player_can_prevent_first_and_skip_the_mill() {
    let shield = Card::from_name("Opt").unwrap();
    let _rules = fixture_rules::activate(&[
        (Card::GrizzlyBears, &PREVENTION_PROVIDER),
        (shield, &SHIELD),
    ]);
    let player = Player::P0;
    let opponent = player.other();
    let mut game = scenario(player);
    let provider = spawn(&mut game, player, Card::GrizzlyBears, Zone::Battlefield);
    let other_provider = spawn(&mut game, player, Card::GrizzlyBears, Zone::Battlefield);
    let shield = spawn(&mut game, opponent, shield, Zone::Hand);
    act(&mut game, Action::Pass);
    cast(&mut game, shield, None);
    let shield_source = game.object_ref(shield);
    settle(&mut game);
    cast_shock(&mut game, player, Target::Player(opponent));
    pass_priority(&mut game);

    assert_eq!(game.actor(), Some(opponent));
    assert!(
        game.legal_actions()
            .contains(&replacement(&game, other_provider))
    );
    let prevent = replacement(&game, provider);
    let mut mill_first = game.clone();
    act(&mut mill_first, prevent);
    act(
        &mut game,
        Action::ChooseDamagePrevention {
            source: shield_source,
            prevention: DamagePreventionKind::UntilNextTurn,
        },
    );
    assert_eq!(game.library_card_count(opponent), 8);
    assert_eq!(mill_first.library_card_count(opponent), 6);
    for branch in [&game, &mill_first] {
        assert_eq!(branch.state.players[opponent.index()].life, 20);
    }
}

#[test]
fn unpreventable_damage_still_mills_the_amount_at_its_ordering_point() {
    let _rules = fixture_rules::activate(&[(Card::GrizzlyBears, &PREVENTION_PROVIDER)]);
    let player = Player::P0;
    let opponent = player.other();
    let mut game = scenario(player);
    game.state.players[opponent.index()].life = 4;
    for _ in 0..5 {
        let top = *game.state.players[opponent.index()].library.last().unwrap();
        game.move_object(top, Zone::Exile);
    }
    let provider = spawn(&mut game, player, Card::GrizzlyBears, Zone::Battlefield);
    let doubler = put(&mut game, player, "Twinflame Tyrant", Zone::Battlefield);
    put(&mut game, opponent, "Sunspine Lynx", Zone::Battlefield);
    cast_shock(&mut game, player, Target::Player(opponent));
    pass_priority(&mut game);
    assert_eq!(game.actor(), Some(opponent));
    for (first, cards_left) in [(provider, 1), (doubler, 0)] {
        let mut branch = game.clone();
        act(&mut branch, replacement(&game, first));
        assert_eq!(branch.state.players[opponent.index()].life, 0);
        assert_eq!(branch.library_card_count(opponent), cards_left);
        assert_eq!(branch.outcome(), Some(Outcome::Winner(player)));
    }
}

#[test]
fn follow_on_mill_replacement_keeps_the_library_card_private_until_committed() {
    let _rules = fixture_rules::activate(&[(Card::GrizzlyBears, &PREVENTION_PROVIDER)]);
    let player = Player::P0;
    let opponent = player.other();
    let mut game = scenario(player);
    spawn(&mut game, player, Card::GrizzlyBears, Zone::Battlefield);
    put(&mut game, opponent, "Rest in Peace", Zone::Battlefield);
    let ordinary = put(&mut game, opponent, "Hill Giant", Zone::Library);
    let hidden = put(&mut game, opponent, "Progenitus", Zone::Library);
    cast_shock(&mut game, player, Target::Player(opponent));
    pass_priority(&mut game);

    assert_eq!(game.actor(), Some(opponent));
    let view = game.observe(opponent).zone_replacement_choice.unwrap();
    assert_eq!(view.from, Zone::Library);
    assert_eq!(view.card, Card::from_name("Progenitus").unwrap());
    assert!(game.observe(player).zone_replacement_choice.is_none());
    assert!(
        game.observe(player)
            .objects
            .iter()
            .all(|object| object.id != hidden)
    );
    assert_eq!(game.library_card_count(opponent), 10);
    reject(
        &mut game,
        Action::ChooseZoneReplacement {
            effect: ZoneReplacementKind::Prevent,
        },
    );

    let checkpoint = game.snapshot();
    let exile = Action::ChooseZoneReplacement {
        effect: ZoneReplacementKind::Exile,
    };
    let index = game
        .legal_actions()
        .iter()
        .position(|choice| *choice == exile)
        .unwrap();
    let mut raw = game.clone();
    act(&mut raw, exile);
    game.apply_index(opponent, game.revision(), index).unwrap();
    assert_eq!(game.snapshot(), raw.snapshot());
    assert_eq!(game.object_view(hidden).zone, Zone::Exile);
    assert_eq!(game.object_view(ordinary).zone, Zone::Exile);
    assert_eq!(game.library_card_count(opponent), 8);

    game.restore(&checkpoint).unwrap();
    act(
        &mut game,
        Action::ChooseZoneReplacement {
            effect: ZoneReplacementKind::RevealShuffle,
        },
    );
    assert_eq!(game.object_view(hidden).zone, Zone::Library);
    assert_eq!(game.object_view(ordinary).zone, Zone::Exile);
    assert_eq!(game.library_card_count(opponent), 9);
    assert_eq!(game.state.players[opponent.index()].life, 20);
}

#[test]
fn damage_scope_uses_the_source_controller_and_only_opposing_players() {
    let _rules = fixture_rules::activate(&[(Card::GrizzlyBears, &PREVENTION_PROVIDER)]);
    let player = Player::P0;
    let opponent = player.other();
    let mut game = scenario(player);
    let provider = spawn(&mut game, player, Card::GrizzlyBears, Zone::Battlefield);
    let creature = put(&mut game, opponent, "Hill Giant", Zone::Battlefield);
    cast_shock(&mut game, player, Target::Card(creature));
    settle(&mut game);
    assert_eq!(game.object_view(creature).damage, 2);
    cast_shock(&mut game, player, Target::Player(player));
    settle(&mut game);
    assert_eq!(game.state.players[player.index()].life, 18);

    let source = put(&mut game, player, "Deadeye Duelist", Zone::Battlefield);
    let source_ref = game.object_ref(source);
    act(
        &mut game,
        Action::AnnounceActivate {
            card: source,
            ability: 0,
            counter_payment: Counters::default(),
        },
    );
    act(
        &mut game,
        Action::ChooseTarget {
            target: Target::Player(opponent),
        },
    );
    pay(&mut game);
    let mut departed = game.clone();
    departed.move_object(source, Zone::Exile);
    settle(&mut departed);
    assert_eq!(departed.library_card_count(opponent), 7);
    assert_eq!(departed.state.players[opponent.index()].life, 20);

    let mut blanked = game.clone();
    let provider_ref = blanked.object_ref(provider);
    blanked.remove_abilities_exact(
        &[provider_ref],
        provider_ref,
        player,
        AnimationDuration::Permanent,
    );
    settle(&mut blanked);
    assert_eq!(blanked.library_card_count(opponent), 8);
    assert_eq!(blanked.state.players[opponent.index()].life, 19);

    game.record_control(source_ref, opponent, None);
    settle(&mut game);
    assert_eq!(game.library_card_count(opponent), 8);
    assert_eq!(game.library_card_count(player), 8);
    assert_eq!(game.state.players[opponent.index()].life, 19);
}
