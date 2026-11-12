//! Filtered single-card discard and its simultaneous unfiltered alternative.
mod support;

use rustmage as _;
use rustmage_core::cards::authored::CardRules;
use rustmage_core::{cards::*, engine::*};
use support::*;

const DISCARD_CHOICE: CardRules = CardRules {
    effect: Effect::Creature(2, 2),
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::SelfEnters,
        targets: TargetPlan::NONE,
        program: &[
            Instruction::Draw(PlayerSet::Controller, 2),
            Instruction::DiscardOneMatchingOr {
                filter: SearchFilter::AnyOf(&[
                    SearchFilter::Kinds(4 | 8),
                    SearchFilter::AllOf(&[
                        SearchFilter::Creature,
                        SearchFilter::Keyword(Keywords::FLYING),
                    ]),
                ]),
                otherwise: 2,
            },
        ],
    }],
    ..CardRules::EMPTY
};
const FLYING_NONCREATURE: CardRules = CardRules {
    effect: Effect::Permanent,
    keywords: Keywords::FLYING,
    ..CardRules::EMPTY
};

fn named(name: &str) -> Card {
    Card::from_name(name).unwrap()
}

fn begin_discard_choice(game: &mut Game, player: Player, source: Card) {
    let source = spawn(game, player, source, Zone::Hand);
    cast(game, source, None);
    for _ in 0..8 {
        if game.state.pending == (Pending::EffectChoice { player }) {
            return;
        }
        assert_eq!(game.state.pending, Pending::Priority);
        act(game, Action::Pass);
    }
    panic!("expected a private discard choice");
}

#[test]
fn draw_then_either_discard_one_matching_or_discard_any_two_together() {
    let source = named("Grizzly Bears");
    let bow = named("Short Bow");
    let _rules = fixture_rules::activate(&[(source, &DISCARD_CHOICE), (bow, &FLYING_NONCREATURE)]);
    let player = Player::P0;
    let opponent = player.other();
    let mut game = scenario(player);
    let land = put(&mut game, player, "Island", Zone::Hand);
    let ground = put(&mut game, player, "Bear Cub", Zone::Hand);
    let noncreature = spawn(&mut game, player, bow, Zone::Hand);
    let instant = put(&mut game, player, "Shock", Zone::Hand);
    let sorcery = put(&mut game, player, "Sleight of Hand", Zone::Library);
    let flyer = put(&mut game, player, "Healer's Hawk", Zone::Library);
    let wrong_player = put(&mut game, opponent, "Healer's Hawk", Zone::Hand);

    begin_discard_choice(&mut game, player, source);
    let view = game.observe(player).effect_choice.unwrap();
    assert!(view.optional);
    assert_eq!(
        view.candidates,
        [instant, sorcery, flyer].map(|card| game.object_ref(card))
    );
    assert!(game.observe(opponent).effect_choice.is_none());
    for card in [land, ground, noncreature, wrong_player] {
        reject(&mut game, Action::Discard { card });
    }

    let root = game.snapshot();
    let action = Action::Discard { card: flyer };
    let index = game
        .legal_actions()
        .iter()
        .position(|a| *a == action)
        .unwrap();
    let mut indexed = game.clone();
    indexed.apply_index(player, game.revision(), index).unwrap();
    act(&mut game, action);
    assert_eq!(game.snapshot(), indexed.snapshot());
    assert_eq!(game.object_view(flyer).zone, Zone::Graveyard);
    for card in [land, ground, noncreature, instant, sorcery] {
        assert_eq!(game.object_view(card).zone, Zone::Hand);
    }

    game.restore(&root).unwrap();
    act(&mut game, Action::DeclineEffectPayment);
    assert_eq!(
        game.state.pending,
        Pending::EffectDiscard {
            player,
            remaining: 2
        }
    );
    act(&mut game, Action::Discard { card: instant });
    assert_eq!(game.object_view(instant).zone, Zone::Hand);
    reject(&mut game, Action::Discard { card: instant });
    act(&mut game, Action::Discard { card: noncreature });
    assert_eq!(game.object_view(instant).zone, Zone::Graveyard);
    assert_eq!(game.object_view(noncreature).zone, Zone::Graveyard);
    assert_eq!(game.object_view(flyer).zone, Zone::Hand);
}

#[test]
fn decline_is_private_without_candidates_and_filtered_payment_can_be_redirected() {
    let source = named("Grizzly Bears");
    let _rules = fixture_rules::activate(&[(source, &DISCARD_CHOICE)]);
    let player = Player::P1;
    let opponent = player.other();
    let mut no_match = scenario(player);
    begin_discard_choice(&mut no_match, player, source);
    assert!(
        no_match
            .observe(player)
            .effect_choice
            .unwrap()
            .candidates
            .is_empty()
    );
    assert!(no_match.observe(opponent).effect_choice.is_none());
    assert_eq!(no_match.legal_actions(), [Action::DeclineEffectPayment]);
    act(&mut no_match, Action::DeclineEffectPayment);
    for _ in 0..2 {
        choose(&mut no_match, |a| matches!(a, Action::Discard { .. }));
    }
    assert_eq!(no_match.hand_count(player), 0);

    let mut redirected = scenario(player);
    put(
        &mut redirected,
        opponent,
        "Rest in Peace",
        Zone::Battlefield,
    );
    let instant = put(&mut redirected, player, "Shock", Zone::Hand);
    begin_discard_choice(&mut redirected, player, source);
    act(&mut redirected, Action::Discard { card: instant });
    assert_eq!(redirected.object_view(instant).zone, Zone::Exile);
    assert_eq!(redirected.hand_count(player), 2);
    assert_eq!(redirected.state.pending, Pending::Priority);
}
