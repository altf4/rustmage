//! Live target qualifications and actual destruction and token-creation outcomes.
mod support;

use rustmage as _;
use rustmage_core::cards::authored::CardRules;
use rustmage_core::engine::replacements::TokenGroupView;
use rustmage_core::{cards::*, engine::*};
use support::*;

const HUMAN: Card = Card(5254);
const FOOD: Card = Card(5246);
const RAT: Card = Card(5323);
const WICKED_ROLE: Card = Card(5248);
const CREATURE: TargetPlan = TargetPlan::Single(TargetSpec::one(TargetRule::Creature));

const CONTROLLED_CREATURE: TargetRule = TargetRule::creature_you_control();

const fn filtered(predicate: &'static TargetPredicate) -> TargetPlan {
    TargetPlan::Single(TargetSpec::one(TargetRule::Filtered {
        base: &CONTROLLED_CREATURE,
        predicate,
    }))
}

const UNIQUE: CardRules = CardRules {
    effect: Effect::Spell(
        filtered(&TargetPredicate::NoOtherControlledSameName { tokens_only: false }),
        &[Instruction::Draw(PlayerSet::Controller, 1)],
    ),
    ..CardRules::EMPTY
};

const NO_TOKEN_NAMESAKE: CardRules = CardRules {
    effect: Effect::Spell(
        TargetPlan::Single(TargetSpec::one(TargetRule::Filtered {
            base: &TargetRule::Filtered {
                base: &CONTROLLED_CREATURE,
                predicate: &TargetPredicate::Nontoken,
            },
            predicate: &TargetPredicate::NoOtherControlledSameName { tokens_only: true },
        })),
        &[Instruction::Draw(PlayerSet::Controller, 1)],
    ),
    ..CardRules::EMPTY
};

const COPY: CardRules = CardRules {
    effect: Effect::Spell(
        CREATURE,
        &[Instruction::CreateCopyToken {
            subject: Subject::Targets,
            amount: 1,
        }],
    ),
    ..CardRules::EMPTY
};

const NEED_AURA: CardRules = CardRules {
    effect: Effect::Spell(
        filtered(&TargetPredicate::Enchanted),
        &[Instruction::Draw(PlayerSet::Controller, 1)],
    ),
    ..CardRules::EMPTY
};

const DESTROY_AND_DRAW: &[Instruction] = &[
    Instruction::DestroyThenRememberControlledOrToken,
    Instruction::DrawScaled(PlayerSet::Controller, AmountValue::MovedCount),
];
const DESTROY: CardRules = CardRules {
    effect: Effect::Spell(CREATURE, DESTROY_AND_DRAW),
    ..CardRules::EMPTY
};
const DESTROY_ENCHANTMENT: CardRules = CardRules {
    effect: Effect::Spell(
        TargetPlan::Single(TargetSpec::one(TargetRule::Object {
            zone: TargetZone::Battlefield,
            kinds: 32,
            relation: -1,
            other: false,
            nonland: false,
            tapped: false,
            min_mana: 0,
            max_mana: u32::MAX,
        })),
        DESTROY_AND_DRAW,
    ),
    ..CardRules::EMPTY
};

const CREATE: CardRules = CardRules {
    effect: Effect::Spell(
        TargetPlan::NONE,
        &[Instruction::CreateTokenBatch(&[
            TokenGroupView {
                token: HUMAN,
                amount: 1,
                tapped: false,
            },
            TokenGroupView {
                token: FOOD,
                amount: 1,
                tapped: false,
            },
        ])],
    ),
    ..CardRules::EMPTY
};

fn cast_available(game: &Game, id: CardId) -> bool {
    game.legal_actions().iter().any(|action| {
        matches!(action, Action::AnnounceCast { card } | Action::Cast { card, .. } if *card == id)
    })
}

#[test]
fn a_controlled_copy_invalidates_a_previously_unique_target() {
    let _rules = fixture_rules::activate(&[
        (Card::from_name("Giant Growth").unwrap(), &UNIQUE),
        (Card::from_name("Opt").unwrap(), &NO_TOKEN_NAMESAKE),
        (Card::from_name("Shock").unwrap(), &COPY),
    ]);
    let player = Player::P0;
    let mut game = scenario(player);
    let target = put(&mut game, player, "Bear Cub", Zone::Battlefield);
    let duplicate = put(&mut game, player, "Bear Cub", Zone::Battlefield);
    put(&mut game, player.other(), "Bear Cub", Zone::Battlefield);
    let unique = put(&mut game, player, "Giant Growth", Zone::Hand);
    let no_token = put(&mut game, player, "Opt", Zone::Hand);
    let copy = put(&mut game, player, "Shock", Zone::Hand);
    assert!(!cast_available(&game, unique));
    assert!(cast_available(&game, no_token));

    game.move_object(duplicate, Zone::Graveyard);
    cast(&mut game, unique, Some(Target::Card(target)));
    cast(&mut game, copy, Some(Target::Card(target)));
    settle(&mut game);
    assert_eq!(game.state.players[player.index()].cards_drawn_this_turn, 0);
    assert!(!cast_available(&game, no_token));
}

#[test]
fn enchanted_targets_accept_a_current_opponent_controlled_aura() {
    let _rules = fixture_rules::activate(&[(Card::from_name("Opt").unwrap(), &NEED_AURA)]);
    let player = Player::P0;
    let mut game = scenario(player);
    let recipient = put(&mut game, player, "Bear Cub", Zone::Battlefield);
    let spell = put(&mut game, player, "Opt", Zone::Hand);
    assert!(!cast_available(&game, spell));
    let reference = game.object_ref(recipient);
    let _ = game.create_token_groups_attached(
        player.other(),
        &[TokenGroupView {
            token: WICKED_ROLE,
            amount: 1,
            tapped: false,
        }],
        None,
        true,
        Some(reference),
    );
    cast(&mut game, spell, Some(Target::Card(recipient)));
    settle(&mut game);
    assert_eq!(game.state.players[player.index()].cards_drawn_this_turn, 1);
}

#[test]
fn destruction_draw_requires_a_controlled_permanent_or_token_to_move() {
    let _rules = fixture_rules::activate(&[(Card::from_name("Giant Growth").unwrap(), &DESTROY)]);
    let player = Player::P0;
    for (controlled, token, protected, draws) in [
        (true, false, false, 1),
        (false, false, false, 0),
        (false, true, false, 1),
        (true, false, true, 0),
    ] {
        let mut game = scenario(player);
        let owner = if controlled { player } else { player.other() };
        let target = if token {
            spawn(&mut game, owner, RAT, Zone::Battlefield)
        } else {
            put(&mut game, owner, "Bear Cub", Zone::Battlefield)
        };
        if protected {
            game.state.objects[target as usize].gained_keywords = Keywords::INDESTRUCTIBLE;
        }
        let spell = put(&mut game, player, "Giant Growth", Zone::Hand);
        cast(&mut game, spell, Some(Target::Card(target)));
        settle(&mut game);
        assert_eq!(
            game.state.players[player.index()].cards_drawn_this_turn,
            draws
        );
    }
}

#[test]
fn heterogeneous_tokens_share_one_replacement_event() {
    let _rules = fixture_rules::activate(&[(Card::from_name("Opt").unwrap(), &CREATE)]);
    let player = Player::P0;
    let mut game = scenario(player);
    put(&mut game, player, "Quina, Qu Gourmet", Zone::Battlefield);
    let spell = put(&mut game, player, "Opt", Zone::Hand);
    cast(&mut game, spell, None);
    settle(&mut game);
    let tokens: Vec<_> = game
        .state
        .objects
        .iter()
        .filter(|object| object.zone == Zone::Battlefield && object.card.is_token())
        .map(|object| object.card)
        .collect();
    assert_eq!(tokens.len(), 3, "one Human, one Food, and one Frog");
    assert!(tokens.contains(&HUMAN) && tokens.contains(&FOOD));
}

#[test]
fn a_linked_return_finishes_before_the_qualifying_destruction_draw() {
    let break_the_spell = Card::from_name("Break the Spell").unwrap();
    let _rules = fixture_rules::activate(&[(break_the_spell, &DESTROY_ENCHANTMENT)]);
    for player in [Player::P0, Player::P1] {
        for light_controller in [player, player.other()] {
            let return_controller = light_controller.other();
            let mut game = scenario(light_controller);
            let light = put(&mut game, light_controller, "Banishing Light", Zone::Hand);
            let automaton = put(
                &mut game,
                return_controller,
                "Adaptive Automaton",
                Zone::Battlefield,
            );
            let spell = spawn(&mut game, player, break_the_spell, Zone::Hand);
            announce(&mut game, light, &[]);
            act(&mut game, Action::Pass);
            act(&mut game, Action::Pass);
            act(
                &mut game,
                Action::ChooseTarget {
                    target: Target::Card(automaton),
                },
            );
            settle(&mut game);
            assert_eq!(game.object_view(automaton).zone, Zone::Exile);
            let exiled = game.object_ref(automaton);
            let library_size = game.state.players[player.index()].library.len();

            if light_controller != player {
                act(&mut game, Action::Pass);
            }
            announce(&mut game, spell, &[Target::Card(light)]);
            act(&mut game, Action::Pass);
            act(&mut game, Action::Pass);
            assert_eq!(game.actor(), Some(return_controller));
            assert_eq!(game.object_view(light).zone, Zone::Graveyard);
            assert_eq!(game.object_view(spell).zone, Zone::Stack);
            assert_eq!(game.object_view(automaton).zone, Zone::Exile);
            assert_eq!(game.state.players[player.index()].cards_drawn_this_turn, 0);
            assert_eq!(
                game.state.players[player.index()].library.len(),
                library_size
            );

            let entry = Action::ChooseEntryValue {
                value: EntryChoiceValue::CreatureType(CreatureType::from_name("Human").unwrap()),
            };
            let revision = game.revision();
            let index = game
                .legal_actions()
                .iter()
                .position(|a| *a == entry)
                .unwrap();
            let checkpoint = game.snapshot();
            let mut indexed = game.clone();
            act(&mut game, entry.clone());
            indexed
                .apply_index(return_controller, revision, index)
                .unwrap();
            assert_eq!(game.snapshot(), indexed.snapshot());
            indexed.restore(&checkpoint).unwrap();
            act(&mut indexed, entry);
            assert_eq!(game.snapshot(), indexed.snapshot());

            let draws = u64::from(light_controller == player);
            assert_eq!(game.object_view(automaton).zone, Zone::Battlefield);
            assert_eq!(game.object_view(automaton).controller, return_controller);
            assert_ne!(game.object_ref(automaton), exiled);
            assert_eq!(game.object_view(spell).zone, Zone::Graveyard);
            assert_eq!(
                game.state.players[player.index()].cards_drawn_this_turn,
                draws
            );
            assert_eq!(
                game.state.players[player.index()].library.len(),
                library_size - draws as usize
            );
        }
    }
}
