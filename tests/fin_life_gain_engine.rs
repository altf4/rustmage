//! Additive and doubling life-gain effects act on the same real life event.
mod support;
use rustmage as _;
use rustmage_core::cards::authored::CardRules;
use rustmage_core::{cards::*, engine::*};
use support::*;

const DOUBLE: CardRules = CardRules {
    effect: Effect::Permanent,
    static_groups: &[StaticAbilityGroup {
        line: 0,
        first_layer: AbilityLayer::Rule,
        fragments: &[StaticAbilityFragment::LifeGainDouble],
    }],
    ..CardRules::EMPTY
};
const PROHIBIT: CardRules = CardRules {
    effect: Effect::Creature(2, 2),
    static_groups: &[StaticAbilityGroup {
        line: 0,
        first_layer: AbilityLayer::Rule,
        fragments: &[StaticAbilityFragment::CantGainLife],
    }],
    ..CardRules::EMPTY
};
const GAIN: CardRules = CardRules {
    effect: Effect::Spell(
        TargetPlan::NONE,
        &[
            Instruction::GainLife(PlayerSet::Controller, 3),
            Instruction::Draw(PlayerSet::Controller, 1),
        ],
    ),
    ..CardRules::EMPTY
};
const BATCH_DAMAGE: CardRules = CardRules {
    effect: Effect::Spell(
        TargetPlan::NONE,
        &[Instruction::DamagePlayers(PlayerSet::Each, 2)],
    ),
    keywords: Keywords::LIFELINK,
    ..CardRules::EMPTY
};

fn resolve_spell(game: &mut Game, player: Player, name: &str) {
    let spell = put(game, player, name, Zone::Hand);
    cast(game, spell, None);
    act(game, Action::Pass);
    act(game, Action::Pass);
}

fn life(game: &Game, player: Player) -> i128 {
    game.state.players[player.index()].life
}

fn choose_effect(game: &Game, source: CardId) -> Action {
    let source = game.object_ref(source);
    game.legal_actions()
        .into_iter()
        .find(
            |action| matches!(action, Action::ChooseReplacement { source: s, .. } if *s == source),
        )
        .expect("exact life replacement")
}

#[test]
fn affected_player_orders_noncommuting_amounts_with_owned_raw_index_and_restore() {
    let _rules = fixture_rules::activate(&[
        (Card::from_name("Swiftfoot Boots").unwrap(), &DOUBLE),
        (Card::from_name("Opt").unwrap(), &GAIN),
    ]);
    let player = Player::P1;
    let mut game = scenario(player);
    let wind = put(&mut game, player, "Swiftfoot Boots", Zone::Battlefield);
    let angel = put(&mut game, player, "Angel of Vitality", Zone::Battlefield);
    resolve_spell(&mut game, player, "Opt");
    assert_eq!(game.state.pending, Pending::LifeGainReplacement { player });
    assert_eq!(life(&game, player), 20);
    assert_eq!(game.hand_count(player), 0);
    let own = game.observe(player).life_gain_replacement_choice.unwrap();
    assert_eq!(own.amount, 3);
    assert_eq!(own.effects.len(), 2);
    let opponent = game.observe(player.other());
    assert!(opponent.life_gain_replacement_choice.is_none());
    assert!(
        !opponent
            .actions
            .iter()
            .any(|action| matches!(action, Action::ChooseReplacement { .. }))
    );

    let add = choose_effect(&game, angel);
    let double = choose_effect(&game, wind);
    let saved = game.snapshot();
    let revision = game.revision();
    let index = game
        .legal_actions()
        .iter()
        .position(|action| *action == add)
        .unwrap();
    let mut indexed = game.clone();
    let mut raw = game.clone();
    let stale = Action::ChooseReplacement {
        source: ObjectRef {
            card: wind,
            generation: game.object_ref(wind).generation + 1,
        },
        ability: 0,
    };
    reject(&mut game, stale);
    act(&mut raw, add.clone());
    indexed.apply_index(player, revision, index).unwrap();
    assert_eq!(indexed.snapshot(), raw.snapshot());
    assert_eq!(life(&raw, player), 28);
    assert_eq!(raw.hand_count(player), 1);
    assert_eq!(
        raw.state.players[player.index()].life_gain_events_this_turn,
        1
    );

    indexed.restore(&saved).unwrap();
    assert_eq!(
        indexed.apply_index(player, revision, index),
        Err(Error::StaleRevision)
    );
    act(&mut indexed, double);
    assert_eq!(life(&indexed, player), 27);
    assert_eq!(indexed.hand_count(player), 1);
    assert_eq!(game.snapshot(), saved);
}

#[test]
fn identical_doublers_stack_and_prohibition_produces_no_life_event() {
    let _rules = fixture_rules::activate(&[
        (Card::from_name("Swiftfoot Boots").unwrap(), &DOUBLE),
        (Card::from_name("Bear Cub").unwrap(), &PROHIBIT),
        (Card::from_name("Opt").unwrap(), &GAIN),
    ]);
    let player = Player::P0;
    for prohibited in [false, true] {
        let mut game = scenario(player);
        for _ in 0..2 {
            put(&mut game, player, "Swiftfoot Boots", Zone::Battlefield);
        }
        put(
            &mut game,
            player.other(),
            "Swiftfoot Boots",
            Zone::Battlefield,
        );
        if prohibited {
            put(&mut game, player.other(), "Bear Cub", Zone::Battlefield);
            put(&mut game, player, "Angel of Vitality", Zone::Battlefield);
        }
        resolve_spell(&mut game, player, "Opt");
        assert_eq!(game.state.pending, Pending::Priority);
        assert_eq!(life(&game, player), if prohibited { 20 } else { 32 });
        assert_eq!(game.hand_count(player), 1);
        assert_eq!(
            game.state.players[player.index()].life_gain_events_this_turn,
            u64::from(!prohibited)
        );
    }
}

#[test]
fn lifelink_freezes_one_gain_per_source_before_any_damage_results_commit() {
    let _rules = fixture_rules::activate(&[
        (Card::from_name("Swiftfoot Boots").unwrap(), &DOUBLE),
        (Card::from_name("Shock").unwrap(), &BATCH_DAMAGE),
    ]);
    let player = Player::P0;
    let mut game = scenario(player);
    put(&mut game, player, "Swiftfoot Boots", Zone::Battlefield);
    let angel = put(&mut game, player, "Angel of Vitality", Zone::Battlefield);
    resolve_spell(&mut game, player, "Shock");
    assert_eq!(game.state.pending, Pending::LifeGainReplacement { player });
    assert_eq!(
        game.observe(player)
            .life_gain_replacement_choice
            .unwrap()
            .amount,
        4
    );
    assert_eq!(life(&game, player), 20);
    assert_eq!(life(&game, player.other()), 20);

    let add = choose_effect(&game, angel);
    act(&mut game, add);
    assert_eq!(life(&game, player), 28);
    assert_eq!(life(&game, player.other()), 18);
    let state = &game.state.players[player.index()];
    assert_eq!(state.life_gained_this_turn, 10);
    assert_eq!(state.life_gain_events_this_turn, 1);
}

#[test]
fn projected_amounts_collapse_equivalent_orderings() {
    let _rules = fixture_rules::activate(&[(Card::from_name("Swiftfoot Boots").unwrap(), &DOUBLE)]);
    let player = Player::P1;
    let mut game = scenario(player);
    for _ in 0..2 {
        put(&mut game, player, "Swiftfoot Boots", Zone::Battlefield);
        put(&mut game, player, "Angel of Vitality", Zone::Battlefield);
    }
    assert_eq!(
        rustmage_core::engine::life_gain::replacement_amounts(&game, player, 3),
        [14, 15, 16, 17, 18, 20]
    );
}
