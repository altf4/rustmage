//! Live conditional noncombat damage replacements.
mod support;

use rustmage as _;
use rustmage_core::cards::authored::{CardExtras, CardRules};
use rustmage_core::{cards::*, engine::*};
use support::*;

const DOUBLE: ReplacementSpec = ReplacementSpec::ConditionalDoubleControlledNoncombatDamage {
    condition: StatValue::AtLeast {
        value: &StatValue::CONTROLLER_GRAVEYARD_CARD_TYPES,
        minimum: 4,
    },
};
const RIDE: CardRules = CardRules {
    effect: Effect::Permanent,
    static_groups: &[StaticAbilityGroup {
        line: 0,
        first_layer: AbilityLayer::Rule,
        fragments: &[StaticAbilityFragment::Replacement(0, DOUBLE)],
    }],
    extras: &CardExtras {
        replacement_effects: &[DOUBLE],
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
const PLUS: ReplacementSpec = ReplacementSpec::NoncombatDamageToOpponentPlus(1);
const BOOSTER: CardRules = CardRules {
    effect: Effect::Creature(2, 2),
    static_groups: &[StaticAbilityGroup {
        line: 0,
        first_layer: AbilityLayer::Rule,
        fragments: &[StaticAbilityFragment::Replacement(0, PLUS)],
    }],
    extras: &CardExtras {
        replacement_effects: &[PLUS],
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};

fn provider_card() -> Card {
    Card::from_name("Anthem of Champions").unwrap()
}

fn four_graveyard_types(game: &mut Game, player: Player) {
    for name in ["Forest", "Jade-Cast Sentinel", "Pacifism"] {
        put(game, player, name, Zone::Graveyard);
    }
}

fn pass_priority(game: &mut Game) {
    act(game, Action::Pass);
    act(game, Action::Pass);
}

fn shock(game: &mut Game, player: Player, target: Target) -> CardId {
    let spell = put(game, player, "Shock", Zone::Hand);
    cast(game, spell, Some(target));
    spell
}

fn replacement(game: &Game, source: CardId) -> Action {
    Action::ChooseReplacement {
        source: game.object_ref(source),
        ability: 0,
    }
}

#[test]
fn the_graveyard_is_checked_at_damage_time_for_own_player_and_permanent() {
    let _rules = fixture_rules::activate(&[(provider_card(), &RIDE)]);
    let player = Player::P1;
    let opponent = player.other();
    let mut game = scenario(player);
    spawn(&mut game, player, provider_card(), Zone::Battlefield);
    let sentinel = put(&mut game, player, "Jade-Cast Sentinel", Zone::Battlefield);
    let lantern = put(&mut game, opponent, "Soul-Guide Lantern", Zone::Battlefield);
    for name in ["Forest", "Plains", "Bear Cub", "Pacifism"] {
        put(&mut game, player, name, Zone::Graveyard);
    }
    put(&mut game, opponent, "Opt", Zone::Graveyard);

    shock(&mut game, player, Target::Player(player));
    let mut three_types = game.clone();
    settle(&mut three_types);
    assert_eq!(three_types.state.players[player.index()].life, 18);

    let growth = put(&mut game, player, "Giant Growth", Zone::Hand);
    cast(&mut game, growth, Some(Target::Card(sentinel)));
    settle(&mut game);
    assert_eq!(game.object_view(growth).zone, Zone::Graveyard);
    assert_eq!(game.state.players[player.index()].life, 16);

    let blast = put(&mut game, player, "Blooming Blast", Zone::Hand);
    act(&mut game, Action::AnnounceCast { card: blast });
    act(
        &mut game,
        Action::ChooseGift {
            beneficiary: Some(opponent),
        },
    );
    act(
        &mut game,
        Action::ChooseTarget {
            target: Target::Card(sentinel),
        },
    );
    pay(&mut game);
    let mut emptied = game.clone();
    act(&mut emptied, Action::Pass);
    assert_eq!(emptied.actor(), Some(opponent));
    act(
        &mut emptied,
        Action::Activate {
            card: lantern,
            ability: 0,
            target: None,
            payment: [0; 6],
            counter_payment: Counters::default(),
        },
    );
    settle(&mut emptied);
    assert_eq!(emptied.object_view(growth).zone, Zone::Exile);
    assert_eq!(emptied.object_view(sentinel).damage, 2);
    assert_eq!(emptied.state.players[player.index()].life, 13);

    settle(&mut game);
    assert_eq!(game.object_view(sentinel).damage, 4);
    assert_eq!(game.state.players[player.index()].life, 10);
}

#[test]
fn simultaneous_fight_packets_use_their_own_sources_and_combat_is_unchanged() {
    let _rules = fixture_rules::activate(&[(provider_card(), &RIDE)]);
    let player = Player::P0;
    let opponent = player.other();
    let mut game = scenario(player);
    spawn(&mut game, player, provider_card(), Zone::Battlefield);
    four_graveyard_types(&mut game, player);
    let friendly = put(&mut game, player, "Jade-Cast Sentinel", Zone::Battlefield);
    let enemy = put(&mut game, opponent, "Jade-Cast Sentinel", Zone::Battlefield);
    let fight = put(&mut game, player, "Bushwhack", Zone::Hand);
    act(&mut game, Action::AnnounceCast { card: fight });
    act(&mut game, Action::ChooseMode { mode: 1 });
    for card in [friendly, enemy] {
        act(
            &mut game,
            Action::ChooseTarget {
                target: Target::Card(card),
            },
        );
    }
    pay(&mut game);
    settle(&mut game);
    assert_eq!(game.object_view(friendly).damage, 1);
    assert_eq!(game.object_view(enemy).damage, 2);

    pass_priority(&mut game);
    pass_priority(&mut game);
    act(&mut game, Action::Attack { card: friendly });
    act(&mut game, Action::FinishAttackers);
    pass_priority(&mut game);
    act(&mut game, Action::FinishBlockers);
    pass_priority(&mut game);
    assert_eq!(game.state.players[opponent.index()].life, 19);
}

#[test]
fn the_affected_player_orders_exact_damage_replacements_and_can_prevent_all_damage() {
    let _rules =
        fixture_rules::activate(&[(provider_card(), &RIDE), (Card::GrizzlyBears, &BOOSTER)]);
    let player = Player::P1;
    let opponent = player.other();
    let mut game = scenario(player);
    let ride = spawn(&mut game, player, provider_card(), Zone::Battlefield);
    let booster = spawn(&mut game, player, Card::GrizzlyBears, Zone::Battlefield);
    four_graveyard_types(&mut game, player);
    let target = put(&mut game, opponent, "Gigantosaurus", Zone::Battlefield);
    let mut shielded = game.clone();
    let spell = shock(&mut game, player, Target::Card(target));
    let spell_ref = game.object_ref(spell);
    pass_priority(&mut game);
    assert_eq!(game.actor(), Some(opponent));
    let choice = game.observe(opponent).replacement_choice.unwrap();
    let packet = choice.damage.unwrap();
    assert_eq!(packet.source.reference, spell_ref);
    assert_eq!(packet.source.controller, player);
    assert_eq!(packet.target, Target::Card(target));
    assert_eq!(packet.amount, 2);
    assert_eq!(choice.effects.len(), 2);

    let double = replacement(&game, ride);
    let add = replacement(&game, booster);
    let revision = game.revision();
    let index = game
        .legal_actions()
        .iter()
        .position(|action| *action == add)
        .unwrap();
    let before = game.snapshot();
    assert_eq!(game.apply(player, add.clone()), Err(Error::WrongActor));
    assert_eq!(game.snapshot(), before);

    let mut raw = game.clone();
    act(&mut raw, add);
    game.apply_index(opponent, revision, index).unwrap();
    assert_eq!(game.snapshot(), raw.snapshot());
    assert_eq!(game.object_view(target).damage, 6);

    game.restore(&before).unwrap();
    assert_eq!(
        game.apply_index(opponent, revision, index),
        Err(Error::StaleRevision)
    );
    assert_eq!(game.snapshot(), before);
    act(&mut game, double);
    assert_eq!(game.object_view(target).damage, 5);

    shielded.add_counter(target, CounterKind::Shield, 1);
    shock(&mut shielded, player, Target::Card(target));
    pass_priority(&mut shielded);
    let shield = shielded.object_ref(target);
    act(
        &mut shielded,
        Action::ChooseDamagePrevention {
            source: shield,
            prevention: DamagePreventionKind::ShieldCounter,
        },
    );
    settle(&mut shielded);
    assert_eq!(shielded.object_view(target).damage, 0);
    assert_eq!(shielded.counter_count(target, CounterKind::Shield), 0);
}

#[test]
fn departed_sources_and_current_provider_control_are_resolved_independently() {
    let _rules = fixture_rules::activate(&[(provider_card(), &RIDE)]);
    let player = Player::P1;
    let opponent = player.other();
    let mut game = scenario(player);
    let ride = spawn(&mut game, player, provider_card(), Zone::Battlefield);
    let ride_ref = game.object_ref(ride);
    four_graveyard_types(&mut game, player);
    let dealer = put(&mut game, player, "Deadeye Duelist", Zone::Battlefield);
    let dealer_ref = game.object_ref(dealer);
    act(
        &mut game,
        Action::AnnounceActivate {
            card: dealer,
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
    departed.move_object(dealer, Zone::Exile);
    settle(&mut departed);
    assert_eq!(departed.state.players[opponent.index()].life, 18);

    let mut blanked = game.clone();
    blanked.remove_abilities_exact(&[ride_ref], ride_ref, player, AnimationDuration::Permanent);
    settle(&mut blanked);
    assert_eq!(blanked.state.players[opponent.index()].life, 19);

    game.record_control(dealer_ref, opponent, None);
    game.record_control(ride_ref, opponent, None);
    let mut without_types = game.clone();
    settle(&mut without_types);
    assert_eq!(without_types.state.players[opponent.index()].life, 19);

    four_graveyard_types(&mut game, opponent);
    settle(&mut game);
    assert_eq!(game.state.players[opponent.index()].life, 18);
}
