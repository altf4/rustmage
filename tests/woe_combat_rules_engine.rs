//! Live defender, blocker-power and independent turn-death rules.
mod support;

use rustmage as _;
use rustmage_core::cards::authored::CardRules;
use rustmage_core::{cards::*, engine::*};
use support::*;

const SQUEAK: CardRules = CardRules {
    effect: Effect::Spell(
        TargetPlan::Single(TargetSpec::one(TargetRule::Creature)),
        &[
            Instruction::Grow {
                subject: Subject::Targets,
                power: 1,
                toughness: 1,
            },
            Instruction::RestrictBlockerPower {
                group: 0,
                minimum: 3,
            },
        ],
    ),
    ..CardRules::EMPTY
};
const STRIP: CardRules = CardRules {
    effect: Effect::Spell(
        TargetPlan::Single(TargetSpec::one(TargetRule::Creature)),
        &[Instruction::RemoveTargetAbilitiesUntilEndOfTurn],
    ),
    ..CardRules::EMPTY
};
const PROTECTOR: CardRules = CardRules {
    effect: Effect::Creature(3, 3),
    static_groups: &[StaticAbilityGroup {
        line: 0,
        first_layer: AbilityLayer::Rule,
        fragments: &[StaticAbilityFragment::AuraEnchantedCantAttackController],
    }],
    ..CardRules::EMPTY
};
const RAT: Card = Card(5323); // Wilds' black 1/1 that cannot block.
const CRESCENDO: CardRules = CardRules {
    effect: Effect::Spell(
        TargetPlan::NONE,
        &[
            Instruction::ModifyPlayerCreatures {
                players: PlayerSet::Controller,
                power: 2,
                toughness: 0,
                keywords: Keywords::NONE,
            },
            Instruction::ScheduleTurnTrigger {
                event: TurnCallbackEvent::ControlledNontokenCreatureDeath,
                ability: 0,
                once: false,
            },
        ],
    ),
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::Reflexive,
        targets: TargetPlan::NONE,
        program: &[Instruction::CreateTokens {
            token: RAT,
            amount: 1,
            tapped: false,
        }],
    }],
    ..CardRules::EMPTY
};

fn pass_twice(g: &mut Game) {
    act(g, Action::Pass);
    act(g, Action::Pass);
}

#[test]
fn blocker_limit_tracks_live_blocker_power_even_after_the_attacker_loses_abilities() {
    let strip = Card::from_name("Unsummon").unwrap();
    let _scope = fixture_rules::activate(&[(Card::Divination, &SQUEAK), (strip, &STRIP)]);
    for player in [Player::P0, Player::P1] {
        let other = player.other();
        let mut g = scenario(player);
        let attacker = put(&mut g, player, "Bear Cub", Zone::Battlefield);
        let small = put(&mut g, other, "Bear Cub", Zone::Battlefield);
        let large = put(&mut g, other, "Hill Giant", Zone::Battlefield);
        let squeak = put(&mut g, player, "Divination", Zone::Hand);
        let strip = put(&mut g, player, "Unsummon", Zone::Hand);
        let pump = put(&mut g, other, "Giant Growth", Zone::Hand);
        let forest = put(&mut g, other, "Forest", Zone::Battlefield);
        cast(&mut g, squeak, Some(Target::Card(attacker)));
        settle(&mut g);
        cast(&mut g, strip, Some(Target::Card(attacker)));
        settle(&mut g);
        assert_eq!(g.object_view(attacker).power, 3);
        assert!(g.can_block_pair(small, attacker));
        assert!(!g.can_block_pair(large, attacker));

        let mut returned = g.clone();
        let old = returned.object_ref(attacker);
        returned.move_object(attacker, Zone::Exile);
        returned.move_object(attacker, Zone::Battlefield);
        assert_ne!(returned.object_ref(attacker), old);
        assert!(returned.can_block_pair(large, attacker));
        assert!(returned.observe(player).blocker_power_limits.is_empty());

        pass_twice(&mut g);
        pass_twice(&mut g);
        act(&mut g, Action::Attack { card: attacker });
        act(&mut g, Action::FinishAttackers);
        act(&mut g, Action::Pass);
        let mut increased = g.clone();
        act(&mut increased, Action::TapMana { card: forest });
        cast(&mut increased, pump, Some(Target::Card(small)));
        settle(&mut increased);
        pass_twice(&mut increased);
        act(&mut g, Action::Pass);
        assert_eq!(g.state.pending, Pending::Blockers);
        assert_eq!(increased.state.pending, Pending::Blockers);
        let block = Action::Block {
            blocker: small,
            attacker,
        };
        assert!(g.legal_actions().contains(&block));
        reject(
            &mut g,
            Action::Block {
                blocker: large,
                attacker,
            },
        );
        reject(&mut increased, block);
        act(&mut increased, Action::FinishBlockers);
    }
}

#[test]
fn an_aura_protects_its_controllers_player_and_planeswalkers_but_not_battles() {
    let _scope = fixture_rules::activate(&[(Card::HillGiant, &PROTECTOR)]);
    for defender in [Player::P0, Player::P1] {
        let attacker = defender.other();
        let mut g = scenario(defender);
        put(&mut g, defender, "Hill Giant", Zone::Battlefield);
        let bearer = put(&mut g, attacker, "Bear Cub", Zone::Battlefield);
        let second = put(&mut g, attacker, "Bear Cub", Zone::Battlefield);
        let aura = put(&mut g, defender, "Bestial Bloodline", Zone::Hand);
        let walker = put(&mut g, defender, "Ajani, Caller of the Pride", Zone::Hand);
        cast(&mut g, aura, Some(Target::Card(bearer)));
        settle(&mut g);
        cast(&mut g, walker, None);
        settle(&mut g);
        let siege = put(&mut g, defender, "Goblin Piker", Zone::Battlefield);
        let mut recipe = CopyRecipe::new(Card::GoblinPiker);
        recipe.overrides = Some(CopyOverrides {
            kinds: Some(BATTLE_KIND),
            ..CopyOverrides::NONE
        });
        g.apply_temporary_copy(g.object_ref(siege), recipe, CopyDeadline::EndOfTurn);
        g.add_counter(siege, CounterKind::Defense, 9);
        g.set_battle_protector(g.object_ref(siege), defender);

        g.state.active = attacker;
        g.state.priority = attacker;
        let attackers_aura = put(&mut g, attacker, "Bestial Bloodline", Zone::Hand);
        cast(&mut g, attackers_aura, Some(Target::Card(second)));
        settle(&mut g);

        let mut removed = g.clone();
        let disenchant = put(&mut removed, attacker, "Disenchant", Zone::Hand);
        cast(&mut removed, disenchant, Some(Target::Card(aura)));
        settle(&mut removed);
        for branch in [&mut g, &mut removed] {
            pass_twice(branch);
            pass_twice(branch);
            assert_eq!(branch.state.pending, Pending::Attackers);
        }
        let attack_player = Action::Attack { card: bearer };
        let attack_walker = Action::AttackPlaneswalker {
            card: bearer,
            defender: g.object_ref(walker),
        };
        let attack_battle = Action::AttackBattle {
            card: bearer,
            defender: g.object_ref(siege),
        };
        reject(&mut g, attack_player.clone());
        reject(&mut g, attack_walker.clone());
        assert!(g.legal_actions().contains(&attack_battle));
        assert!(g.legal_actions().contains(&Action::Attack { card: second }));
        assert!(removed.legal_actions().contains(&attack_player));
        assert!(removed.legal_actions().contains(&attack_walker));
        act(&mut g, attack_battle);
        act(&mut g, Action::FinishAttackers);
    }
}

fn count_rats(g: &Game, player: Player) -> usize {
    g.observe(player)
        .objects
        .iter()
        .filter(|object| {
            object.zone == Zone::Battlefield
                && object.controller == player
                && object.is_token()
                && object.has_subtype("Rat")
        })
        .count()
}

#[test]
fn a_turn_death_listener_includes_later_entrants_and_excludes_tokens_and_opponents() {
    let crescendo = Card::from_name("Giant Growth").unwrap();
    let _scope = fixture_rules::activate(&[(crescendo, &CRESCENDO)]);
    for player in [Player::P0, Player::P1] {
        let mut g = scenario(player);
        let old = put(&mut g, player, "Hill Giant", Zone::Battlefield);
        let later = put(&mut g, player, "Bear Cub", Zone::Hand);
        put(&mut g, player.other(), "Bear Cub", Zone::Battlefield);
        g.create_tokens(player, RAT, 1, false);
        let spell = put(&mut g, player, "Giant Growth", Zone::Hand);
        cast(&mut g, spell, None);
        settle(&mut g);
        assert_eq!(g.object_view(old).power, 5);
        cast(&mut g, later, None);
        settle(&mut g);
        assert_eq!(g.object_view(later).power, 2);
        assert_eq!(count_rats(&g, player), 1);

        let sweeper = put(&mut g, player, "Day of Judgment", Zone::Hand);
        cast(&mut g, sweeper, None);
        pass_twice(&mut g);
        assert_eq!(count_rats(&g, player), 0);
        for _ in 0..2 {
            if matches!(g.state.pending, Pending::OrderTriggers { .. }) {
                choose(&mut g, |action| matches!(action, Action::PutTrigger { .. }));
            }
        }
        assert_eq!(g.state.stack.len(), 2);
        settle(&mut g);
        assert_eq!(count_rats(&g, player), 2);
        assert_eq!(count_rats(&g, player.other()), 0);

        let turn = g.state.turn;
        g.begin_step(Step::End);
        for _ in 0..16 {
            if g.state.turn != turn {
                break;
            }
            act(&mut g, Action::Pass);
        }
        assert_ne!(g.state.turn, turn);
        assert!(g.observe(player).turn_callbacks.is_none());
        g.begin_step(Step::Main1);
        g.state.players[player.other().index()].mana = [8; 6];
        put(&mut g, player, "Bear Cub", Zone::Battlefield);
        let sweeper = put(&mut g, player.other(), "Day of Judgment", Zone::Hand);
        cast(&mut g, sweeper, None);
        settle(&mut g);
        assert_eq!(count_rats(&g, player), 0);
    }
}
