//! Independent graveyard roles return in one exact animated batch.
mod support;

use rustmage as _;
use rustmage_core::cards::authored::CardRules;
use rustmage_core::{cards::*, engine::*};
use support::*;

const fn graveyard(kinds: u8) -> TargetRule {
    TargetRule::Object {
        zone: TargetZone::Graveyard,
        kinds,
        relation: 0,
        other: false,
        nonland: false,
        tapped: false,
        min_mana: 0,
        max_mana: u32::MAX,
    }
}
const fn optional(rule: TargetRule) -> TargetSpec {
    TargetSpec {
        rule,
        min: 0,
        max: 1,
    }
}

const ANIMATED_RETURN: CardRules = CardRules {
    effect: Effect::Spell(
        TargetPlan::Groups(&[
            optional(graveyard(16)),
            optional(graveyard(2)),
            optional(TargetRule::Filtered {
                base: &graveyard(32),
                predicate: &TargetPredicate::WithoutSubtype("Aura"),
            }),
        ]),
        &[Instruction::ReturnGraveyardTargetsAnimated {
            animation: &AnimationSpec {
                artifact: false,
                creature_types: AnimationCreatureTypes::Additional(&["Elemental"]),
                base: Some((5, 5)),
                keywords: Keywords::NONE,
                duration: AnimationDuration::Permanent,
            },
            x_counters: None,
        }],
    ),
    ..CardRules::EMPTY
};

const ARTIFACT_LAND: CardRules = CardRules {
    effect: Effect::Land,
    ..CardRules::EMPTY
};

#[test]
fn all_roles_animate_once_before_observers_see_the_combined_entries() {
    let spell = Card::from_name("Opt").unwrap();
    let land = Card::from_name("Scene of the Crime").unwrap();
    let _rules = fixture_rules::activate(&[(spell, &ANIMATED_RETURN), (land, &ARTIFACT_LAND)]);
    let player = Player::P0;
    let mut game = scenario(player);
    let target = spawn(&mut game, player, land, Zone::Graveyard);
    let enchantment = put(&mut game, player, "Impact Tremors", Zone::Graveyard);
    let source = spawn(&mut game, player, spell, Zone::Hand);
    let generation = game.object_ref(target).generation;
    announce(
        &mut game,
        source,
        &[
            Target::Card(target),
            Target::Card(target),
            Target::Card(enchantment),
        ],
    );
    act(&mut game, Action::Pass);
    act(&mut game, Action::Pass);
    for _ in 0..2 {
        if !matches!(game.state.pending, Pending::OrderTriggers { .. }) {
            break;
        }
        choose(&mut game, |action| {
            matches!(action, Action::PutTrigger { .. })
        });
    }
    assert_eq!(game.state.stack.len(), 2);
    settle(&mut game);
    assert_eq!(game.state.players[player.other().index()].life, 18);
    assert_eq!(game.object_ref(target).generation, generation + 1);
    for (id, kinds) in [(target, 1 | 2 | 16), (enchantment, 1 | 32)] {
        let object = game.object_view(id);
        assert_eq!(object.zone, Zone::Battlefield);
        assert_eq!(object.kinds, kinds);
        assert!(object.has_subtype("Elemental"));
        assert_eq!((object.power, object.toughness), (5, 5));
    }
}
