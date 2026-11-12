//! Typed object costs and exact public hand selection continuations.
mod support;

use rustmage as _;
use rustmage_core::cards::authored::{CardExtras, CardRules};
use rustmage_core::{cards::*, engine::*};
use support::*;

const TARGET: TargetPlan = TargetPlan::Single(TargetSpec::one(TargetRule::Player {
    opponent_only: true,
}));
const SPIRIT: Card = Card::from_catalogue_id(5342);
const OBJECT_FILTER: SacrificeFilter = SacrificeFilter {
    keywords: Keywords::NONE,
    source_only: false,
    kinds: 0,
    nonland: false,
    other: false,
    tokens: None,
    artifact_enchantment_or_token: false,
    subtype: None,
};
const TWO_PERMANENTS: CardRules = CardRules {
    effect: Effect::Spell(
        TARGET,
        &[
            Instruction::MayPayCostsFor {
                players: PlayerSet::Targeted,
                costs: &[ResolutionCost {
                    objects: Some(ResolutionObjectCost {
                        cost: AdditionalCost {
                            kind: EffectChoiceKind::Sacrifice,
                            filter: SacrificeFilter {
                                nonland: true,
                                tokens: Some(false),
                                ..OBJECT_FILTER
                            },
                        },
                        count: 2,
                    }),
                    ..ResolutionCost::FREE
                }],
                skip: 1,
            },
            Instruction::Skip(1),
            Instruction::Draw(PlayerSet::Controller, 2),
        ],
    ),
    ..CardRules::EMPTY
};

fn choose_cost(game: &Game, card: CardId) -> Action {
    Action::ChooseAdditionalCost {
        object: game.object_ref(card),
    }
}

#[test]
fn targeted_player_can_only_pay_with_two_distinct_nonland_nontokens() {
    let card = Card::from_name("Opt").unwrap();
    let _rules = fixture_rules::activate(&[(card, &TWO_PERMANENTS)]);
    for player in [Player::P0, Player::P1] {
        let payer = player.other();
        let mut game = scenario(player);
        let forest = put(&mut game, payer, "Forest", Zone::Battlefield);
        let first = put(&mut game, payer, "Grizzly Bears", Zone::Battlefield);
        let second = put(&mut game, payer, "Llanowar Elves", Zone::Battlefield);
        let own = put(&mut game, player, "Llanowar Elves", Zone::Battlefield);
        game.create_tokens(payer, SPIRIT, 1, false);
        assert!(!game.advance_entries());
        let token = game.state.objects.len() as CardId - 1;

        let spell = spawn(&mut game, player, card, Zone::Hand);
        cast(&mut game, spell, Some(Target::Player(payer)));
        act(&mut game, Action::Pass);
        act(&mut game, Action::Pass);
        assert_eq!(game.actor(), Some(payer));
        for illegal in [forest, token, own] {
            let action = choose_cost(&game, illegal);
            reject(&mut game, action);
        }
        let select = choose_cost(&game, first);
        act(&mut game, select.clone());
        assert_eq!(game.object_view(first).zone, Zone::Battlefield);
        reject(&mut game, select);
        reject(&mut game, Action::PayResolutionMana { payment: [0; 6] });

        let saved = game.snapshot();
        let mut declined = game.clone();
        act(&mut declined, Action::DeclineResolutionMana);
        assert_eq!(declined.object_view(first).zone, Zone::Battlefield);
        assert_eq!(
            declined.state.players[player.index()].cards_drawn_this_turn,
            2
        );

        let select = choose_cost(&game, second);
        let index = game
            .legal_actions()
            .iter()
            .position(|action| *action == select)
            .unwrap();
        declined.restore(&saved).unwrap();
        declined
            .apply_index(payer, declined.revision(), index)
            .unwrap();
        act(&mut game, select);
        assert_eq!(declined.snapshot(), game.snapshot());
        act(&mut game, Action::PayResolutionMana { payment: [0; 6] });
        assert_eq!(game.object_view(first).zone, Zone::Graveyard);
        assert_eq!(game.object_view(second).zone, Zone::Graveyard);
        assert_eq!(game.state.players[player.index()].cards_drawn_this_turn, 0);
    }
}

const DAMAGING_CREATURE_COST: CardRules = CardRules {
    effect: Effect::Spell(
        TargetPlan::NONE,
        &[Instruction::Draw(PlayerSet::Controller, 1)],
    ),
    extras: &CardExtras {
        additional_cost: Some(AdditionalCost {
            kind: EffectChoiceKind::FilteredCost {
                action: CostAction::Sacrifice,
                any: &[CostMatch::DealtDamageThisTurn],
            },
            filter: SacrificeFilter {
                kinds: 1,
                ..OBJECT_FILTER
            },
        }),
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};

#[test]
fn sacrifice_cost_reads_positive_damage_by_the_current_exact_creature() {
    let card = Card::from_name("Opt").unwrap();
    let _rules = fixture_rules::activate(&[(card, &DAMAGING_CREATURE_COST)]);
    let player = Player::P0;
    let opponent = player.other();
    let mut game = scenario(player);
    let dealer = put(&mut game, player, "Grizzly Bears", Zone::Battlefield);
    let stale = put(&mut game, player, "Bear Cub", Zone::Battlefield);
    let unused = put(&mut game, player, "Healer's Hawk", Zone::Battlefield);
    let spell = spawn(&mut game, player, card, Zone::Hand);
    let announce = Action::AnnounceCast { card: spell };
    reject(&mut game, announce.clone());
    let target = Target::Player(opponent);
    game.apply_damage_batch(&[dealer, stale].map(|id| DamageEvent {
        source: game.damage_source(id),
        target,
        amount: 1,
        combat: false,
        preventable: true,
    }));
    game.move_object(stale, Zone::Exile);
    game.move_object(stale, Zone::Battlefield);
    act(&mut game, announce);
    for illegal in [unused, stale] {
        let action = choose_cost(&game, illegal);
        reject(&mut game, action);
    }
    let action = choose_cost(&game, dealer);
    act(&mut game, action);
    pay(&mut game);
    settle(&mut game);
    assert_eq!(game.object_view(dealer).zone, Zone::Graveyard);
    assert_eq!(game.object_view(stale).zone, Zone::Battlefield);
}

const EXILE_REWARD: CardRules = CardRules {
    effect: Effect::Spell(
        TARGET,
        &[Instruction::RevealTargetHandExileNonlandReward {
            maximum: 1,
            token: SPIRIT,
        }],
    ),
    ..CardRules::EMPTY
};

#[test]
fn revealing_the_hand_requires_a_nonland_choice_and_rewards_the_chosen_mana_value() {
    let card = Card::from_name("Opt").unwrap();
    let _rules = fixture_rules::activate(&[(card, &EXILE_REWARD)]);
    let player = Player::P0;
    let opponent = player.other();
    let mut game = scenario(player);
    let low = put(&mut game, opponent, "Shock", Zone::Hand);
    let high = put(&mut game, opponent, "Grizzly Bears", Zone::Hand);
    let land = put(&mut game, opponent, "Forest", Zone::Hand);
    let spell = spawn(&mut game, player, card, Zone::Hand);
    cast(&mut game, spell, Some(Target::Player(opponent)));
    act(&mut game, Action::Pass);
    act(&mut game, Action::Pass);
    let hand_refs = [low, high, land].map(|id| game.object_ref(id));
    assert!(
        hand_refs
            .iter()
            .all(|r| game.observe(player).known_hand.contains(r))
    );
    reject(&mut game, Action::ChooseHandCard { object: None });
    reject(
        &mut game,
        Action::ChooseHandCard {
            object: Some(hand_refs[2]),
        },
    );
    let mut costly = game.clone();
    act(
        &mut costly,
        Action::ChooseHandCard {
            object: Some(hand_refs[1]),
        },
    );
    assert_eq!(costly.object_view(high).zone, Zone::Exile);
    assert!(
        !costly
            .state
            .objects
            .iter()
            .any(|o| o.zone == Zone::Battlefield && o.is_token())
    );
    act(
        &mut game,
        Action::ChooseHandCard {
            object: Some(hand_refs[0]),
        },
    );
    assert_eq!(game.object_view(low).zone, Zone::Exile);
    assert!(
        game.state
            .objects
            .iter()
            .any(|o| o.zone == Zone::Battlefield
                && o.is_token()
                && o.card == SPIRIT
                && o.controller == player)
    );
}
