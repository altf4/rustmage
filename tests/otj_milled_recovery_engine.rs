//! A mandatory land recovery consumes the exact public mill receipt and tests the actual hand move.
mod support;
use rustmage::Card;
use rustmage_core::cards::authored::CardRules;
use rustmage_core::cards::*;
use rustmage_core::engine::*;
use support::*;

const TREASURE: Card = Card::from_catalogue_id(5302);
const NATURALIST: CardRules = CardRules {
    effect: Effect::Creature(2, 3),
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::SelfEnters,
        targets: TargetPlan::NONE,
        program: &[
            Instruction::Mill(PlayerSet::Controller, MillAmount::Fixed(3)),
            Instruction::SelectMilled(MilledSelectionSpec {
                filter: MilledFilter::Kinds(2),
                count: MilledCount::Range {
                    minimum: 1,
                    maximum: 1,
                },
                destination: MilledDestination::HandAndCount,
            }),
            Instruction::IfAmountAtLeast {
                value: AmountValue::MovedCount,
                minimum: 1,
                skip: 1,
            },
            Instruction::Skip(1),
            Instruction::CreateTokens {
                token: TREASURE,
                amount: 1,
                tapped: false,
            },
        ],
    }],
    ..CardRules::EMPTY
};

fn trigger(g: &mut Game, player: Player) {
    let source = put(g, player, "Grizzly Bears", Zone::Hand);
    cast(g, source, None);
    for _ in 0..4 {
        act(g, Action::Pass);
    }
}

fn treasures(g: &Game, player: Player) -> usize {
    g.state
        .objects
        .iter()
        .filter(|o| o.zone == Zone::Battlefield && o.controller == player && o.card == TREASURE)
        .count()
}

#[test]
fn an_exact_land_must_be_chosen_and_the_fallback_requires_failed_movement() {
    struct Reset;
    impl Drop for Reset {
        fn drop(&mut self) {
            zone_replacements::TEST_PREVENT_ZONE_MOVE.with(|rule| rule.set(None));
        }
    }

    let _rules = fixture_rules::activate(&[(Card::GrizzlyBears, &NATURALIST)]);
    let _reset = Reset;
    let player = Player::P0;
    let mut base = scenario(player);
    let old = put(&mut base, player, "Forest", Zone::Graveyard);
    let first = put(&mut base, player, "Island", Zone::Library);
    let second = put(&mut base, player, "Forest", Zone::Library);
    let nonland = put(&mut base, player, "Bear Cub", Zone::Library);
    trigger(&mut base, player);
    assert_eq!(base.state.pending, Pending::MilledSelection { player });
    reject(&mut base, Action::FinishMilledSelection);
    for id in [old, nonland] {
        let object = base.object_ref(id);
        reject(&mut base, Action::SelectMilled { object });
    }
    let selected = base.object_ref(first);
    let action = Action::SelectMilled { object: selected };
    let mut succeeded = base.clone();
    act(&mut succeeded, action.clone());
    assert_eq!(succeeded.object_view(first).zone, Zone::Hand);
    assert_eq!(succeeded.object_view(second).zone, Zone::Graveyard);
    assert_eq!(treasures(&succeeded, player), 0);
    assert!(
        succeeded
            .observe(player.other())
            .known_hand
            .contains(&succeeded.object_ref(first))
    );

    zone_replacements::TEST_PREVENT_ZONE_MOVE.with(|rule| rule.set(Some(selected)));
    act(&mut base, action);
    assert_eq!(base.object_view(first).zone, Zone::Graveyard);
    assert_eq!(treasures(&base, player), 1);
}

#[test]
fn an_empty_candidate_set_creates_treasure_and_a_public_mill_redirect_can_still_return() {
    let _rules = fixture_rules::activate(&[(Card::GrizzlyBears, &NATURALIST)]);
    let player = Player::P1;
    let mut empty = scenario(player);
    for _ in 0..3 {
        put(&mut empty, player, "Bear Cub", Zone::Library);
    }
    trigger(&mut empty, player);
    assert_eq!(treasures(&empty, player), 1);

    let mut redirected = scenario(player);
    put(
        &mut redirected,
        player.other(),
        "Rest in Peace",
        Zone::Battlefield,
    );
    let land = put(&mut redirected, player, "Forest", Zone::Library);
    trigger(&mut redirected, player);
    assert_eq!(redirected.object_view(land).zone, Zone::Exile);
    let object = redirected.object_ref(land);
    act(&mut redirected, Action::SelectMilled { object });
    assert_eq!(redirected.object_view(land).zone, Zone::Hand);
    assert_eq!(treasures(&redirected, player), 0);
}
