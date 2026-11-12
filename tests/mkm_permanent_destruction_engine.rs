//! Nontargeting destruction of a simultaneous set of current permanents.
mod support;

use rustmage as _;
use rustmage_core::cards::authored::CardRules;
use rustmage_core::{cards::*, engine::*};
use support::*;

const BOTH_KINDS: BattlefieldFilter = BattlefieldFilter {
    kinds: 16 | 32,
    relation: -1,
    nonland: false,
    tapped: None,
    without: Keywords::NONE,
    subtype: None,
    max_mana: u32::MAX,
};

const OPPOSING_ARTIFACTS: CardRules = CardRules {
    effect: Effect::Spell(
        TargetPlan::NONE,
        &[Instruction::MoveBattlefield {
            filters: &[BattlefieldFilter {
                kinds: 16,
                relation: 1,
                ..BOTH_KINDS
            }],
            exile: false,
        }],
    ),
    ..CardRules::EMPTY
};
const ARTIFACTS_AND_ENCHANTMENTS: CardRules = CardRules {
    effect: Effect::Spell(
        TargetPlan::NONE,
        &[Instruction::MoveBattlefield {
            filters: &[BOTH_KINDS],
            exile: false,
        }],
    ),
    ..CardRules::EMPTY
};
const OBSERVER: CardRules = CardRules {
    effect: Effect::Creature(2, 2),
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::PermanentDiesBatch {
            kinds: 16 | 32,
            relation: CastRelation::Any,
            other: false,
        },
        targets: TargetPlan::NONE,
        program: &[Instruction::Draw(PlayerSet::Controller, 1)],
    }],
    ..CardRules::EMPTY
};

#[test]
fn current_control_and_kind_select_one_nontargeting_destruction_batch() {
    let _rules = fixture_rules::activate(&[
        (Card::Divination, &OPPOSING_ARTIFACTS),
        (
            Card::from_name("Chart a Course").unwrap(),
            &ARTIFACTS_AND_ENCHANTMENTS,
        ),
        (Card::from_name("Bear Cub").unwrap(), &OBSERVER),
    ]);
    let player = Player::P0;
    let opponent = player.other();
    for (spell, both_players, arrival_total) in
        [("Divination", false, 3), ("Chart a Course", true, 4)]
    {
        for redirect in [false, true] {
            let mut game = scenario(player);
            let earlier = put(&mut game, player, "Candy Trail", Zone::Battlefield);
            game.sacrifice_batch(&[(player, earlier)]);
            let observer = put(&mut game, player, "Bear Cub", Zone::Battlefield);
            let controlled = put(&mut game, player, "Candy Trail", Zone::Battlefield);
            let borrowed = put(&mut game, player, "Short Bow", Zone::Battlefield);
            let hexproof = put(&mut game, opponent, "Soul-Guide Lantern", Zone::Battlefield);
            let protected = put(&mut game, opponent, "Nutrient Block", Zone::Battlefield);
            let enchantments = [
                put(&mut game, player, "Impact Tremors", Zone::Battlefield),
                put(&mut game, opponent, "Doubling Season", Zone::Battlefield),
            ];
            game.record_control(game.object_ref(borrowed), opponent, None);
            game.state.objects[hexproof as usize].gained_keywords = Keywords::HEXPROOF;
            game.state.objects[protected as usize].gained_keywords = Keywords::INDESTRUCTIBLE;
            if redirect {
                put(&mut game, player, "Rest in Peace", Zone::Battlefield);
            }

            let spell = put(&mut game, player, spell, Zone::Hand);
            cast(&mut game, spell, None);
            settle(&mut game);
            let destroyed_to = if redirect {
                Zone::Exile
            } else {
                Zone::Graveyard
            };
            for id in [borrowed, hexproof] {
                assert_eq!(game.object_view(id).zone, destroyed_to);
            }
            for id in [controlled, enchantments[0], enchantments[1]] {
                assert_eq!(
                    game.object_view(id).zone,
                    if both_players {
                        destroyed_to
                    } else {
                        Zone::Battlefield
                    },
                );
            }
            assert_eq!(game.object_view(borrowed).owner, player);
            assert!(game.on_battlefield(protected));
            assert!(game.on_battlefield(observer));
            assert_eq!(
                game.state.players[player.index()].cards_drawn_this_turn,
                u64::from(!redirect),
            );
            assert_eq!(
                game.stat_value(
                    StatValue::ArtifactsPutIntoGraveyardsFromBattlefieldThisTurn,
                    game.object_ref(spell),
                    player,
                ),
                if redirect { 1 } else { arrival_total },
            );
        }
    }
}
