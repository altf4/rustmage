//! Legendary cast qualification applies to any spell type and uses the caster.
mod support;
use rustmage as _;
use rustmage_core::cards::authored::CardRules;
use rustmage_core::{cards::*, engine::*};
use support::*;

const WATCHER: CardRules = CardRules {
    effect: Effect::Creature(2, 2),
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::OncePerTurn(&TriggerCondition::SpellCast {
            relation: CastRelation::You,
            filter: CastFilter::Legendary,
            ordinal: None,
        }),
        targets: TargetPlan::NONE,
        program: &[Instruction::Draw(PlayerSet::Controller, 1)],
    }],
    ..CardRules::EMPTY
};

#[test]
fn legendary_casts_match_noncreature_spells_and_only_once_each_turn() {
    let _rules = fixture_rules::activate(&[(Card::GrizzlyBears, &WATCHER)]);
    let player = Player::P0;
    let mut game = scenario(player);
    let watcher = put(&mut game, player, "Grizzly Bears", Zone::Battlefield);
    let ordinary = put(&mut game, player, "Short Bow", Zone::Hand);
    cast(&mut game, ordinary, None);
    assert_eq!(game.state.stack.len(), 1);
    settle(&mut game);

    let mut opponent = game.clone();
    opponent.state.active = player.other();
    opponent.state.priority = player.other();
    let their_spell = put(&mut opponent, player.other(), "Mox Jasper", Zone::Hand);
    cast(&mut opponent, their_spell, None);
    assert_eq!(opponent.state.stack.len(), 1);
    settle(&mut opponent);

    let legendary = put(&mut game, player, "Mox Jasper", Zone::Hand);
    assert!(!game.object_view(legendary).card.is_creature());
    cast(&mut game, legendary, None);
    assert_eq!(game.state.stack.len(), 2);
    let trigger = game.state.stack.last().unwrap();
    assert_eq!(trigger.source().unwrap().id, watcher);
    settle(&mut game);
    assert_eq!(game.hand_count(player), 1);

    let second = put(&mut game, player, "The Aetherspark", Zone::Hand);
    cast(&mut game, second, None);
    assert_eq!(game.state.stack.len(), 1);
    settle(&mut game);
    assert_eq!(game.hand_count(player), 1);
}
