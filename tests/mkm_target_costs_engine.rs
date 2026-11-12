//! Cast a spell with a colored surcharge per additional target.
mod support;

use rustmage as _;
use rustmage_core::cards::authored::{CardExtras, CardRules};
use rustmage_core::{cards::*, engine::*};
use support::*;

const WU: Mana = [1, 1, 0, 0, 0, 0];
const WWUU: Mana = [2, 2, 0, 0, 0, 0];
const SURCHARGE: CardRules = CardRules {
    effect: Effect::Spell(
        TargetPlan::Single(TargetSpec {
            rule: TargetRule::Player {
                opponent_only: false,
            },
            min: 0,
            max: 2,
        }),
        &[Instruction::DrawScaled(
            PlayerSet::Controller,
            AmountValue::TargetPlayersCreatureCount,
        )],
    ),
    extras: &CardExtras {
        additional_target_mana: Some(ManaCost {
            colored: WU,
            generic: 0,
        }),
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
const CLUE: Card = Card::from_catalogue_id(5296);
const INVESTIGATE: CardRules = CardRules {
    effect: Effect::Spell(
        TargetPlan::NONE,
        &[Instruction::CreateTokensRepeated {
            token: CLUE,
            amount: AmountValue::Stat(StatValue::Count(CountFilter {
                zone: CountZone::Battlefield,
                relation: 0,
                kind: 16,
                exclude: 0,
                subtype: None,
                basic: false,
                other: false,
                cards_only: false,
            })),
        }],
    ),
    ..CardRules::EMPTY
};
const EXTRA_CLUE: ReplacementSpec = ReplacementSpec::AddToken(CLUE);
const CLUE_BATCH_OBSERVER: CardRules = CardRules {
    effect: Effect::Permanent,
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::FilteredEntersBatchYou {
            predicate: EntryPredicate::SubtypeAny(&["Clue"]),
            other: true,
        },
        targets: TargetPlan::NONE,
        program: &[Instruction::Draw(PlayerSet::Controller, 1)],
    }],
    static_groups: &[StaticAbilityGroup {
        line: 0,
        first_layer: AbilityLayer::Rule,
        fragments: &[StaticAbilityFragment::Replacement(0, EXTRA_CLUE)],
    }],
    extras: &CardExtras {
        replacement_effects: &[EXTRA_CLUE],
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
const HEXPROOF: &[StaticAbilityGroup] = &[StaticAbilityGroup {
    line: 0,
    first_layer: AbilityLayer::Ability,
    fragments: &[StaticAbilityFragment::QualifiedHexproof(
        SourceCharacteristicFilter::Multicolored,
    )],
}];
const HEXPROOF_CREATURE: CardRules = CardRules {
    effect: Effect::Creature(2, 2),
    static_groups: HEXPROOF,
    extras: &CardExtras {
        static_ability_groups: HEXPROOF,
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};

fn player_target(player: Player) -> Action {
    Action::ChooseTarget {
        target: Target::Player(player),
    }
}

#[test]
fn ordinary_target_surcharge_locks_colored_mana_and_counts_surviving_players() {
    let spell = Card::from_name("No More Lies").unwrap();
    let _scope = fixture_rules::activate(&[(spell, &SURCHARGE)]);
    let player = Player::P0;
    let opponent = player.other();

    for (targets, mana, draws) in [
        (&[][..], WU, 0),
        (&[opponent][..], WU, 2),
        (&[opponent, player][..], WWUU, 3),
    ] {
        let mut game = scenario(player);
        game.state.players[player.index()].mana = mana;
        put(&mut game, player, "Bear Cub", Zone::Battlefield);
        for _ in 0..2 {
            put(&mut game, opponent, "Grizzly Bears", Zone::Battlefield);
        }
        let source = spawn(&mut game, player, spell, Zone::Hand);
        act(&mut game, Action::AnnounceCast { card: source });
        for &target in targets {
            act(&mut game, player_target(target));
        }
        if targets.len() == 1 {
            reject(&mut game, player_target(player));
        }
        if game.legal_actions().contains(&Action::FinishTargets) {
            act(&mut game, Action::FinishTargets);
        }
        act(&mut game, Action::PayCosts { payment: mana });
        settle(&mut game);
        assert_eq!(game.state.players[player.index()].mana, [0; 6]);
        assert_eq!(
            game.state.players[player.index()].cards_drawn_this_turn,
            draws
        );
    }
}

#[test]
fn a_free_cast_still_owes_the_colored_surcharge_for_a_second_target() {
    let spell = Card::from_name("No More Lies").unwrap();
    let _scope = fixture_rules::activate(&[(spell, &SURCHARGE)]);
    let player = Player::P1;
    let mut game = scenario(player);
    let source = spawn(&mut game, player.other(), spell, Zone::Exile);
    game.grant_exile_free_permission(player, game.object_ref(source));
    game.state.players[player.index()].mana = WU;
    choose(
        &mut game,
        |action| matches!(action, Action::AnnounceExileFree { card, .. } if *card == source),
    );
    act(&mut game, player_target(player));
    let mut cannot_pay = game.clone();
    cannot_pay.state.players[player.index()].mana = [0, 2, 0, 0, 0, 0];
    reject(&mut cannot_pay, player_target(player.other()));
    act(&mut game, player_target(player.other()));
    act(&mut game, Action::PayCosts { payment: WU });
    assert_eq!(game.state.players[player.index()].mana, [0; 6]);
}

#[test]
fn multicolored_hexproof_and_pair_counts_use_actual_source_and_permanent_colors() {
    let protected_card = Card::from_name("Bear Cub").unwrap();
    let _scope = fixture_rules::activate(&[(protected_card, &HEXPROOF_CREATURE)]);
    let player = Player::P0;
    let opponent = player.other();
    let mut game = scenario(player);
    let protected = spawn(&mut game, opponent, protected_card, Zone::Battlefield);
    let unprotected = put(&mut game, opponent, "Grizzly Bears", Zone::Battlefield);
    let helix = put(&mut game, player, "Lightning Helix", Zone::Hand);
    act(&mut game, Action::AnnounceCast { card: helix });
    let target = |id| Action::ChooseTarget {
        target: Target::Card(id),
    };
    reject(&mut game, target(protected));
    act(&mut game, target(unprotected));
    pay(&mut game);
    settle(&mut game);
    assert_eq!(game.object_view(unprotected).zone, Zone::Graveyard);
    let shock = put(&mut game, player, "Shock", Zone::Hand);
    act(&mut game, Action::AnnounceCast { card: shock });
    assert!(game.legal_actions().contains(&target(protected)));

    let mut game = scenario(player);
    let first = put(&mut game, player, "Warleader's Call", Zone::Battlefield);
    let duplicate = put(&mut game, player, "Warleader's Call", Zone::Battlefield);
    put(
        &mut game,
        player,
        "Trostani, Three Whispers",
        Zone::Battlefield,
    );
    put(&mut game, player, "Riku of Many Paths", Zone::Battlefield);
    put(
        &mut game,
        opponent,
        "Vren, the Relentless",
        Zone::Battlefield,
    );
    let source = game.object_ref(first);
    assert_eq!(
        game.stat_value(StatValue::ControlledTwoColorPairs, source, player),
        2
    );
    game.move_object(duplicate, Zone::Graveyard);
    assert_eq!(
        game.stat_value(StatValue::ControlledTwoColorPairs, source, player),
        2
    );
}

#[test]
fn repeated_investigations_keep_the_count_frozen_through_owned_replacements() {
    let spell = Card::from_name("No More Lies").unwrap();
    let observer = Card::from_name("Soul-Guide Lantern").unwrap();
    let _scope =
        fixture_rules::activate(&[(spell, &INVESTIGATE), (observer, &CLUE_BATCH_OBSERVER)]);
    let player = Player::P0;
    let mut game = scenario(player);
    let extra = spawn(&mut game, player, observer, Zone::Battlefield);
    let double = put(&mut game, player, "Doubling Season", Zone::Battlefield);
    put(&mut game, player, "Short Bow", Zone::Battlefield);

    let source = spawn(&mut game, player, spell, Zone::Hand);
    cast(&mut game, source, None);
    act(&mut game, Action::Pass);
    act(&mut game, Action::Pass);
    assert_eq!(game.state.pending, Pending::Replacement { player });

    let checkpoint = game.snapshot();
    let actions = game.legal_actions();
    let index = actions
        .iter()
        .position(|action| matches!(action, Action::ChooseReplacement { source, .. } if source.card == double))
        .unwrap();
    let mut indexed = game.clone();
    indexed.apply_index(player, game.revision(), index).unwrap();
    act(&mut game, actions[index].clone());
    assert_eq!(indexed.snapshot(), game.snapshot());
    indexed.restore(&checkpoint).unwrap();

    // Two initial artifacts, even though the first event adds several new ones.
    for (branch, choice, initial, expected) in
        [(&mut game, double, 3, 6), (&mut indexed, extra, 0, 8)]
    {
        let tokens = |g: &Game| {
            g.controlled_token_ids(player)
                .filter(|&id| g.object_view(id).card == CLUE)
                .count()
        };
        assert_eq!(tokens(branch), initial);
        for _ in 0..2 {
            if branch.state.pending != (Pending::Replacement { player }) {
                break;
            }
            choose(
                branch,
                |action| matches!(action, Action::ChooseReplacement { source, .. } if source.card == choice),
            );
        }
        assert_eq!(tokens(branch), expected);
        while matches!(branch.state.pending, Pending::OrderTriggers { .. }) {
            choose(branch, |action| matches!(action, Action::PutTrigger { .. }));
        }
        assert_eq!(branch.state.stack.len(), 2);
        settle(branch);
        assert_eq!(
            branch.state.players[player.index()].cards_drawn_this_turn,
            2
        );
    }
}
