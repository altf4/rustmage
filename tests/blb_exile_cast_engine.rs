//! Exact immediate exile casting and paid permission from a gifted hand exile.
mod support;

use rustmage as _;
use rustmage_core::cards::authored::{CardExtras, CardRules};
use rustmage_core::{cards::*, engine::*};
use support::*;

const EXILE_SPELL: CardRules = CardRules {
    effect: Effect::Spell(
        TargetPlan::NONE,
        &[Instruction::ExileUntilNonlandCastByDiscarding],
    ),
    ..CardRules::EMPTY
};

const OPPONENT: TargetPlan = TargetPlan::Single(TargetSpec::one(TargetRule::Player {
    opponent_only: true,
}));
const HAND_EXILE: CardRules = CardRules {
    effect: Effect::Spell(
        OPPONENT,
        &[
            Instruction::GiveGift,
            Instruction::RevealTargetHandExileNonlandCastIfGift,
        ],
    ),
    extras: &CardExtras {
        gift: Some(GiftSpec {
            kind: GiftKind::Card,
            token: None,
            promised_targets: OPPONENT,
        }),
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};

fn until_choice(game: &mut Game) {
    for _ in 0..16 {
        if game.state.pending != Pending::Priority {
            return;
        }
        act(game, Action::Pass);
    }
    panic!("spell did not present its resolution choice");
}

fn choose_cost(game: &mut Game, clause: CastCostClause) {
    let index = game
        .observe(game.actor().unwrap())
        .cast_costs
        .unwrap()
        .clauses
        .iter()
        .position(|row| row.clause == clause && !row.paid)
        .unwrap() as u16;
    act(game, Action::ChooseCastCost { clause: index });
}

fn exiled_cast(game: &Game, spell: CardId, face: CastFace) -> Action {
    Action::CastExiledDuringResolution {
        object: game.object_ref(spell),
        face,
    }
}

fn indexed_twin(game: &mut Game, action: Action) {
    let actor = game.actor().unwrap();
    let index = game
        .legal_actions()
        .iter()
        .position(|a| *a == action)
        .unwrap();
    let mut twin = game.clone();
    twin.apply_index(actor, twin.revision(), index).unwrap();
    act(game, action);
    assert_eq!(game.snapshot(), twin.snapshot());
}

#[test]
fn the_exile_offer_preserves_both_discard_costs_and_can_be_declined() {
    let _rules = fixture_rules::activate(&[(Card::from_name("Divination").unwrap(), &EXILE_SPELL)]);
    for player in [Player::P0, Player::P1] {
        let mut game = scenario(player);
        let caster = put(&mut game, player, "Divination", Zone::Hand);
        let first = put(&mut game, player, "Forest", Zone::Hand);
        let spell = put(&mut game, player, "Thrill of Possibility", Zone::Library);
        let buried_land = put(&mut game, player, "Forest", Zone::Library);
        let top_land = put(&mut game, player, "Mountain", Zone::Library);

        let mut short = game.clone();
        cast(&mut short, caster, None);
        until_choice(&mut short);
        let unavailable = exiled_cast(&short, spell, CastFace::Normal);
        reject(&mut short, unavailable);
        act(&mut short, Action::FinishResolutionCasts);
        assert_eq!(short.object_view(first).zone, Zone::Hand);
        assert_eq!(short.object_view(spell).zone, Zone::Exile);

        let second = put(&mut game, player, "Mountain", Zone::Hand);
        let opposing = put(&mut game, player.other(), "Swamp", Zone::Hand);
        cast(&mut game, caster, None);
        until_choice(&mut game);
        for card in [top_land, buried_land, spell] {
            assert_eq!(game.object_view(card).zone, Zone::Exile);
        }
        let before = game.snapshot();
        let cast_action = exiled_cast(&game, spell, CastFace::Normal);
        let mut decline = game.clone();
        act(&mut decline, Action::FinishResolutionCasts);
        reject(&mut decline, Action::AnnounceExile { card: spell });

        for (before_mana, after_mana) in [
            (
                CastCostClause::OriginDiscard(None),
                CastCostClause::Additional,
            ),
            (
                CastCostClause::Additional,
                CastCostClause::OriginDiscard(None),
            ),
        ] {
            let mut branch = game.clone();
            indexed_twin(&mut branch, cast_action.clone());
            act(&mut branch, Action::BeginOrderedCosts);
            choose_cost(&mut branch, before_mana);
            let first_payment = Action::ChooseAdditionalCost {
                object: branch.object_ref(first),
            };
            let other_payment = Action::ChooseAdditionalCost {
                object: branch.object_ref(opposing),
            };
            reject(&mut branch, other_payment);
            assert!(
                !branch
                    .observe(player.other())
                    .objects
                    .iter()
                    .any(|object| { object.id == first && object.zone == Zone::Hand })
            );
            indexed_twin(&mut branch, first_payment.clone());
            choose_cost(&mut branch, CastCostClause::Mana);
            pay(&mut branch);
            choose_cost(&mut branch, after_mana);
            reject(&mut branch, first_payment);
            let payment = Action::ChooseAdditionalCost {
                object: branch.object_ref(second),
            };
            indexed_twin(&mut branch, payment);
            if branch.state.pending == (Pending::ResolutionCasts { player }) {
                act(&mut branch, Action::FinishResolutionCasts);
            }
            settle(&mut branch);
            assert_eq!(branch.object_view(second).zone, Zone::Graveyard);
            assert_eq!(
                branch.state.players[player.index()].cards_drawn_this_turn,
                2
            );
            branch.restore(&before).unwrap();
            assert_eq!(branch.snapshot(), before);
        }
    }
}

#[test]
fn gifted_hand_exile_grants_exact_indefinite_cast_permission_after_the_draw() {
    let _rules = fixture_rules::activate(&[(Card::from_name("Duress").unwrap(), &HAND_EXILE)]);
    let player = Player::P1;
    for gift in [false, true] {
        let mut game = scenario(player);
        let spell = put(&mut game, player, "Duress", Zone::Hand);
        let visible = put(&mut game, player.other(), "Bear Cub", Zone::Hand);
        let land = put(&mut game, player.other(), "Forest", Zone::Hand);
        let drawn = put(
            &mut game,
            player.other(),
            "Divination",
            if gift { Zone::Library } else { Zone::Hand },
        );
        act(&mut game, Action::AnnounceCast { card: spell });
        act(
            &mut game,
            Action::ChooseGift {
                beneficiary: gift.then_some(player.other()),
            },
        );
        act(
            &mut game,
            Action::ChooseTarget {
                target: Target::Player(player.other()),
            },
        );
        pay(&mut game);
        until_choice(&mut game);
        let exile = Action::ChooseHandCard {
            object: Some(game.object_ref(drawn)),
        };
        let land_choice = Action::ChooseHandCard {
            object: Some(game.object_ref(land)),
        };
        reject(&mut game, land_choice);
        assert!(
            game.observe(player)
                .objects
                .iter()
                .any(|object| object.id == visible)
        );
        indexed_twin(&mut game, exile);
        assert_eq!(game.object_view(drawn).zone, Zone::Exile);
        if gift {
            let exact = game.object_ref(drawn);
            assert!(game.state.exile_permissions.iter().any(|permission| {
                permission.object == exact
                    && permission.player == player
                    && permission.cast_only
                    && permission.spend_any_mana
            }));
            game.state.turn += 4;
            game.state.players[player.index()].mana = [0, 0, 0, 3, 0, 0];
            assert!(game.permitted_cast_available(
                player,
                drawn,
                CastMethod::Normal,
                CastFace::Normal,
                true,
                false
            ));
            game.move_object(drawn, Zone::Graveyard);
            game.move_object(drawn, Zone::Exile);
            assert!(
                !game
                    .state
                    .exile_permissions
                    .iter()
                    .any(|p| p.object.card == drawn)
            );
        } else {
            assert!(game.state.exile_permissions.is_empty());
        }
    }
}
