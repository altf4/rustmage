//! Public reveals that move cards to hand and a live permission through defender.
mod support;
use rustmage as _;
use rustmage_core::cards::authored::CardRules;
use rustmage_core::{cards::*, engine::*};
use support::*;

const REVEAL: CardRules = CardRules {
    effect: Effect::Spell(
        TargetPlan::NONE,
        &[Instruction::RevealTopToHandLoseManaValue],
    ),
    ..CardRules::EMPTY
};

const DEFENDER: CardRules = CardRules {
    effect: Effect::Creature(2, 3),
    keywords: Keywords::DEFENDER,
    static_groups: &[
        StaticAbilityGroup {
            line: 0,
            first_layer: AbilityLayer::Ability,
            fragments: &[StaticAbilityFragment::IntrinsicKeywords(Keywords::DEFENDER)],
        },
        StaticAbilityGroup {
            line: 1,
            first_layer: AbilityLayer::Rule,
            fragments: &[StaticAbilityFragment::AttackWithoutDefenderIf(
                StatValue::SourceTotalCounters,
            )],
        },
    ],
    ..CardRules::EMPTY
};

fn pass_pair(game: &mut Game) {
    act(game, Action::Pass);
    act(game, Action::Pass);
}

#[test]
fn the_public_card_is_moved_without_drawing_and_pays_its_frozen_mana_value() {
    let opt = Card::from_name("Opt").unwrap();
    let _rules = fixture_rules::activate(&[(opt, &REVEAL)]);
    let player = Player::P0;
    let mut game = scenario(player);
    let spell = spawn(&mut game, player, opt, Zone::Hand);
    let top = put(&mut game, player, "Cruel Truths", Zone::Library);
    let before = game.object_ref(top);
    cast(&mut game, spell, None);
    act(&mut game, Action::Pass);
    let saved = game.snapshot();
    let index = game
        .legal_actions()
        .iter()
        .position(|action| *action == Action::Pass)
        .unwrap();
    let mut indexed = game.clone();
    indexed
        .apply_index(player.other(), game.revision(), index)
        .unwrap();
    act(&mut game, Action::Pass);
    assert_eq!(game.snapshot(), indexed.snapshot());
    let seen = game.observe(player.other());
    let reveal = seen.last_library_reveal.unwrap();
    assert_eq!(reveal.revealed.object, before);
    assert!(reveal.drawn.is_none());
    assert!(seen.known_hand.contains(&game.object_ref(top)));
    assert_eq!(game.state.players[player.index()].cards_drawn_this_turn, 0);
    assert_eq!(game.state.players[player.index()].life, 16);
    assert_eq!(game.object_view(top).zone, Zone::Hand);

    struct Clear;
    impl Drop for Clear {
        fn drop(&mut self) {
            zone_replacements::TEST_PREVENT_ZONE_MOVE.with(|rule| rule.set(None));
        }
    }
    let _clear = Clear;
    game.restore(&saved).unwrap();
    zone_replacements::TEST_PREVENT_ZONE_MOVE.with(|rule| rule.set(Some(before)));
    act(&mut game, Action::Pass);
    assert_eq!(game.state.players[player.index()].life, 16);
    assert_eq!(game.object_view(top).zone, Zone::Library);
    assert_eq!(
        game.observe(player.other())
            .last_library_reveal
            .unwrap()
            .revealed
            .object,
        before
    );
}

#[test]
fn an_exact_defender_can_attack_with_any_counter_and_loses_permission_with_the_last() {
    let _rules = fixture_rules::activate(&[(Card::GrizzlyBears, &DEFENDER)]);
    let player = Player::P0;
    let mut game = scenario(player);
    let first = spawn(&mut game, player, Card::GrizzlyBears, Zone::Battlefield);
    let second = spawn(&mut game, player, Card::GrizzlyBears, Zone::Battlefield);
    pass_pair(&mut game);
    pass_pair(&mut game);
    let attack = Action::Attack { card: first };
    reject(&mut game, attack.clone());
    game.set_counters(first, Counters::of(CounterKind::Quest, 1));
    assert!(game.keywords(first).contains(Keywords::DEFENDER));
    assert!(game.legal_actions().contains(&attack));
    reject(&mut game, Action::Attack { card: second });
    game.set_counters(first, Counters::default());
    reject(&mut game, attack.clone());
    game.set_counters(first, Counters::of(CounterKind::Stun, 1));
    act(&mut game, attack);
    act(&mut game, Action::FinishAttackers);
    assert!(game.object_view(first).tapped);
}
