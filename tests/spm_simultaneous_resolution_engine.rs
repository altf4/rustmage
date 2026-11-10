//! APNAP private discard choices and simultaneous public library events.
mod support;

use rustmage as _;
use rustmage_core::cards::authored::CardRules;
use rustmage_core::cards::*;
use rustmage_core::engine::*;
use support::*;

const OPTIONAL_DISCARDS: CardRules = CardRules {
    effect: Effect::Spell(
        TargetPlan::NONE,
        &[Instruction::MayDiscardOrLoseLife {
            players: PlayerSet::Each,
            amount: 3,
        }],
    ),
    ..CardRules::EMPTY
};
const SWAP_REVEALS: CardRules = CardRules {
    effect: Effect::Spell(
        TargetPlan::Single(TargetSpec {
            rule: TargetRule::Player {
                opponent_only: false,
            },
            min: 2,
            max: 2,
        }),
        &[Instruction::RevealTargetTopsToHandLoseOtherManaValue],
    ),
    ..CardRules::EMPTY
};
const PLAYER_HEXPROOF: CardRules = CardRules {
    effect: Effect::Spell(TargetPlan::NONE, &[Instruction::PlayerHexproofThisTurn]),
    ..CardRules::EMPTY
};
const SIMULTANEOUS_LOSS_OBSERVER: CardRules = CardRules {
    effect: Effect::Creature(1, 1),
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::When {
            event: &TriggerCondition::LifeLost {
                relation: CastRelation::You,
            },
            guard: StepGuard::Expression {
                value: StatValue::OpponentLifeAtMost(17),
                minimum: Some(1),
                maximum: None,
            },
        },
        targets: TargetPlan::NONE,
        program: &[Instruction::GainLife(PlayerSet::Controller, 1)],
    }],
    ..CardRules::EMPTY
};

fn card(name: &str) -> Card {
    Card::from_name(name).unwrap()
}

fn resolve_top(game: &mut Game) {
    act(game, Action::Pass);
    act(game, Action::Pass);
}

fn cast_discard(game: &mut Game, controller: Player) {
    let spell = put(game, controller, "Divination", Zone::Hand);
    cast(game, spell, None);
    resolve_top(game);
}

fn cast_reveal(game: &mut Game, controller: Player) {
    let spell = put(game, controller, "Divination", Zone::Hand);
    announce(
        game,
        spell,
        &[Target::Player(Player::P0), Target::Player(Player::P1)],
    );
}

#[test]
fn simultaneous_discards_keep_chosen_identities_private_until_both_players_finish() {
    let _rules = fixture_rules::activate(&[(Card::Divination, &OPTIONAL_DISCARDS)]);
    for player in [Player::P0, Player::P1] {
        let opponent = player.other();
        let mut game = scenario(player);
        let first = put(&mut game, player, "Island", Zone::Hand);
        let alternate = put(&mut game, player, "Mountain", Zone::Hand);
        let second = put(&mut game, opponent, "Plains", Zone::Hand);
        cast_discard(&mut game, player);
        assert_eq!(game.actor(), Some(player));
        assert_eq!(
            game.observe(opponent).optional_effect_discards,
            Some([None; 2])
        );
        let choose_first = Action::Discard { card: first };
        let start = game.snapshot();
        let revision = game.revision();
        let index = game
            .legal_actions()
            .iter()
            .position(|a| *a == choose_first)
            .unwrap();
        let mut indexed = game.clone();
        act(&mut game, choose_first);
        indexed.apply_index(player, revision, index).unwrap();
        assert_eq!(indexed.snapshot(), game.snapshot());
        assert_eq!(game.actor(), Some(opponent));
        assert_eq!(game.object_view(first).zone, Zone::Hand);
        let mut public = [None; 2];
        public[player.index()] = Some(true);
        let view = game.observe(opponent);
        assert_eq!(view.optional_effect_discards, Some(public));
        assert!(view.effect_discards.is_empty());
        assert_eq!(game.observe(player).effect_discards, [first]);
        assert!(
            view.objects
                .iter()
                .all(|object| object.id != first && object.id != alternate)
        );

        indexed.restore(&start).unwrap();
        act(&mut indexed, Action::Discard { card: alternate });
        let original = game.observe(opponent);
        let mut different_hidden_card = indexed.observe(opponent);
        different_hidden_card.revision = original.revision;
        assert_eq!(
            serde_json::to_value(original).unwrap(),
            serde_json::to_value(different_hidden_card).unwrap()
        );
        act(&mut game, Action::Discard { card: second });
        assert_eq!(game.object_view(first).zone, Zone::Graveyard);
        assert_eq!(game.object_view(second).zone, Zone::Graveyard);
        assert_eq!(game.state.players[player.index()].life, 20);
        assert_eq!(game.state.players[opponent.index()].life, 20);

        act(&mut indexed, Action::ChooseOptional { accept: false });
        assert_eq!(indexed.object_view(alternate).zone, Zone::Graveyard);
        assert_eq!(indexed.object_view(second).zone, Zone::Hand);
        assert_eq!(indexed.state.players[player.index()].life, 20);
        assert_eq!(indexed.state.players[opponent.index()].life, 17);
    }
}

struct ClearPrevention;
impl Drop for ClearPrevention {
    fn drop(&mut self) {
        zone_replacements::TEST_PREVENT_ZONE_MOVE.with(|target| target.set(None));
    }
}

#[test]
fn discard_fallback_uses_actual_events_and_commits_both_life_losses_before_observers() {
    let _rules = fixture_rules::activate(&[
        (Card::Divination, &OPTIONAL_DISCARDS),
        (card("Bear Cub"), &SIMULTANEOUS_LOSS_OBSERVER),
    ]);
    let player = Player::P0;
    let opponent = player.other();
    let mut game = scenario(player);
    let prevented = put(&mut game, player, "Forest", Zone::Hand);
    let redirected = put(&mut game, opponent, "Wilt-Leaf Liege", Zone::Hand);
    cast_discard(&mut game, player);
    act(&mut game, Action::Discard { card: prevented });
    {
        let _clear = ClearPrevention;
        zone_replacements::TEST_PREVENT_ZONE_MOVE.with(|target| {
            target.set(Some(game.object_ref(prevented)));
        });
        act(&mut game, Action::Discard { card: redirected });
    }
    settle(&mut game);
    assert_eq!(game.object_view(prevented).zone, Zone::Hand);
    assert_eq!(game.object_view(redirected).zone, Zone::Battlefield);
    assert_eq!(
        game.state.players[player.index()].cards_discarded_this_turn,
        0
    );
    assert_eq!(
        game.state.players[opponent.index()].cards_discarded_this_turn,
        1
    );
    assert_eq!(game.state.players[player.index()].life, 17);
    assert_eq!(game.state.players[opponent.index()].life, 20);

    let mut both = scenario(player);
    put(&mut both, player, "Bear Cub", Zone::Battlefield);
    put(&mut both, player, "Forest", Zone::Hand);
    cast_discard(&mut both, player);
    act(&mut both, Action::ChooseOptional { accept: false });
    assert_eq!(both.state.players[player.index()].life, 17);
    assert_eq!(both.state.players[opponent.index()].life, 17);
    settle(&mut both);
    assert_eq!(both.state.players[player.index()].life, 18);
    assert_eq!(both.state.players[opponent.index()].life, 17);
}

#[test]
fn both_public_tops_are_revealed_and_taken_without_drawing_even_after_one_target_fails() {
    let _rules = fixture_rules::activate(&[
        (Card::Divination, &SWAP_REVEALS),
        (card("Opt"), &PLAYER_HEXPROOF),
    ]);
    let player = Player::P0;
    let opponent = player.other();
    for partial in [false, true] {
        let mut game = scenario(player);
        let own = put(&mut game, player, "Dune Drifter", Zone::Library);
        let foreign = put(&mut game, opponent, "Shivan Dragon", Zone::Library);
        let old_own = game.object_ref(own);
        let old_foreign = game.object_ref(foreign);
        cast_reveal(&mut game, player);
        if partial {
            let response = put(&mut game, opponent, "Opt", Zone::Hand);
            act(&mut game, Action::Pass);
            assert_eq!(game.actor(), Some(opponent));
            cast(&mut game, response, None);
        }
        settle(&mut game);
        assert_eq!(game.object_view(own).zone, Zone::Hand);
        assert_ne!(game.object_ref(own), old_own);
        let own_hand = game.object_ref(own);
        assert!(game.observe(opponent).known_hand.contains(&own_hand));
        assert_eq!(
            game.state.players[player.index()].life,
            if partial { 20 } else { 14 }
        );
        assert_eq!(
            game.state.players[opponent.index()].life,
            if partial { 20 } else { 18 }
        );
        let reveals = game.observe(player).zone_reveals;
        assert_eq!(reveals.len(), if partial { 1 } else { 2 });
        assert_eq!(reveals[0].card, card("Dune Drifter"));
        assert_eq!(reveals[0].owner, player);
        assert_eq!(reveals[0].from, Zone::Library);
        if partial {
            assert_eq!(game.object_ref(foreign), old_foreign);
            assert_eq!(game.object_view(foreign).zone, Zone::Library);
            assert!(game.observe(player).known_library_top[opponent.index()].is_none());
        } else {
            assert_eq!(reveals[1].card, card("Shivan Dragon"));
            assert_eq!(reveals[1].owner, opponent);
            assert_eq!(reveals[1].from, Zone::Library);
            assert_eq!(game.object_view(foreign).zone, Zone::Hand);
            assert!(
                game.observe(player)
                    .known_hand
                    .contains(&game.object_ref(foreign))
            );
        }
        for p in [player, opponent] {
            assert_eq!(game.state.players[p.index()].cards_drawn_this_turn, 0);
        }
    }
}

#[test]
fn empty_library_does_not_draw_and_a_prevented_move_keeps_the_top_public() {
    let _rules = fixture_rules::activate(&[(Card::Divination, &SWAP_REVEALS)]);
    let player = Player::P0;
    let opponent = player.other();
    let mut game = scenario(player);
    for id in game.state.players[player.index()].library.clone() {
        game.move_object(id, Zone::Exile);
    }
    let foreign = put(&mut game, opponent, "Shivan Dragon", Zone::Library);
    let top = game.object_ref(foreign);
    cast_reveal(&mut game, player);
    let _clear = ClearPrevention;
    zone_replacements::TEST_PREVENT_ZONE_MOVE.with(|target| target.set(Some(top)));
    settle(&mut game);
    assert!(game.outcome().is_none());
    assert_eq!(game.state.players[player.index()].life, 14);
    assert_eq!(game.state.players[opponent.index()].life, 20);
    assert_eq!(game.object_ref(foreign), top);
    assert_eq!(game.object_view(foreign).zone, Zone::Library);
    assert_eq!(
        game.observe(player).known_library_top[opponent.index()]
            .unwrap()
            .object,
        top
    );
}
