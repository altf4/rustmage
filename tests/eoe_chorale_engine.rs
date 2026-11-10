//! Attachment attack triggers and independent defenders for reanimated attackers.
mod support;

use rustmage as _;
use rustmage_core::cards::authored::CardRules;
use rustmage_core::cards::*;
use rustmage_core::engine::*;
use support::*;

const AURA_TARGETS: TargetPlan = TargetPlan::Single(TargetSpec::one(TargetRule::Object {
    zone: TargetZone::Battlefield,
    kinds: 1,
    relation: 0,
    other: false,
    nonland: false,
    tapped: false,
    min_mana: 0,
    max_mana: u32::MAX,
}));
const GRAVE_TARGET: TargetPlan = TargetPlan::Single(TargetSpec::one(TargetRule::Object {
    zone: TargetZone::Graveyard,
    kinds: 1,
    relation: 1,
    other: false,
    nonland: false,
    tapped: false,
    min_mana: 0,
    max_mana: u32::MAX,
}));
const AURA: CardRules = CardRules {
    effect: Effect::Aura(AURA_TARGETS),
    aura: Some(AuraSpec {
        kinds: 1,
        relation: 0,
        bonus: EquipmentSpec {
            stats: Stats {
                power: 0,
                toughness: 0,
            },
            keywords: Keywords::NONE,
        },
    }),
    triggers: &[
        TriggeredAbility {
            condition: TriggerCondition::AttachedAttacks,
            targets: GRAVE_TARGET,
            program: &[Instruction::ReturnTargetTappedAttacking],
        },
        TriggeredAbility {
            condition: TriggerCondition::AtStep {
                step: TriggerStep::End,
                relation: CastRelation::You,
                guard: StepGuard::Always,
            },
            targets: TargetPlan::NONE,
            program: &[
                Instruction::If {
                    guard: StepGuard::VoidActive,
                    skip: 1,
                },
                Instruction::Skip(1),
                Instruction::SacrificeSource,
            ],
        },
    ],
    ..CardRules::EMPTY
};
const REANIMATED: CardRules = CardRules {
    effect: Effect::Creature(2, 2),
    triggers: &[
        TriggeredAbility {
            condition: TriggerCondition::SelfEnters,
            targets: TargetPlan::NONE,
            program: &[Instruction::GainLife(PlayerSet::Controller, 2)],
        },
        TriggeredAbility {
            condition: TriggerCondition::SelfAttacks,
            targets: TargetPlan::NONE,
            program: &[Instruction::GainLife(PlayerSet::Controller, 7)],
        },
    ],
    ..CardRules::EMPTY
};
const AURA_NAME: &str = "Sheltered by Ghosts";

fn cast_aura(game: &mut Game, p: Player, recipient: CardId) -> CardId {
    let aura = put(game, p, AURA_NAME, Zone::Hand);
    cast(game, aura, Some(Target::Card(recipient)));
    settle(game);
    assert_eq!(
        game.object_view(aura).attached_to,
        Some(game.object_ref(recipient))
    );
    aura
}

fn advance_until(g: &mut Game, ready: impl Fn(&Game) -> bool) {
    for _ in 0..24 {
        if ready(g) {
            return;
        }
        choose(g, |action| {
            matches!(action, Action::Pass | Action::PutTrigger { .. })
        });
    }
    panic!("advance budget exhausted at {:?}", g.state.pending);
}

#[test]
fn attached_attack_reanimates_exact_defenders_card_for_an_independent_attack() {
    let _rules = fixture_rules::activate(&[
        (Card::from_name(AURA_NAME).unwrap(), &AURA),
        (Card::from_name("Bear Cub").unwrap(), &REANIMATED),
    ]);
    for p in [Player::P0, Player::P1] {
        let opponent = p.other();
        let mut g = scenario(p);
        let attacker = put(&mut g, p, "Grizzly Bears", Zone::Battlefield);
        let aura = cast_aura(&mut g, p, attacker);
        let ally_grave = put(&mut g, p, "Bear Cub", Zone::Graveyard);
        let enemy_grave = put(&mut g, opponent, "Bear Cub", Zone::Graveyard);
        let enemy_land = put(&mut g, opponent, "Island", Zone::Graveyard);
        let walker = put(&mut g, opponent, "Ajani, Caller of the Pride", Zone::Hand);
        g.move_object(walker, Zone::Battlefield);
        let walker = g.object_ref(walker);
        advance_until(&mut g, |g| g.state.pending == Pending::Attackers);
        act(
            &mut g,
            Action::AttackPlaneswalker {
                card: attacker,
                defender: walker,
            },
        );
        act(&mut g, Action::FinishAttackers);
        let choose_target = |id| Action::ChooseTarget {
            target: Target::Card(id),
        };
        advance_until(&mut g, |g| {
            g.legal_actions().contains(&choose_target(enemy_grave))
        });
        reject(&mut g, choose_target(ally_grave));
        reject(&mut g, choose_target(enemy_land));
        act(&mut g, choose_target(enemy_grave));
        if g.legal_actions().contains(&Action::FinishTargets) {
            act(&mut g, Action::FinishTargets);
        }
        let packet = g.state.stack.last().unwrap();
        let attachment = packet.attachment_trigger.as_ref().unwrap();
        assert_eq!(attachment.grantor, g.object_ref(aura));
        assert_eq!(attachment.recipient, g.object_ref(attacker));
        assert_eq!(attachment.defender, Some(opponent));

        let mut stale = g.clone();
        let original = stale.object_ref(enemy_grave);
        stale.move_object(enemy_grave, Zone::Exile);
        stale.move_object(enemy_grave, Zone::Graveyard);
        assert_ne!(stale.object_ref(enemy_grave), original);
        settle(&mut stale);
        assert_eq!(stale.object_view(enemy_grave).zone, Zone::Graveyard);

        g.move_object(aura, Zone::Graveyard);
        advance_until(&mut g, |g| {
            g.state.pending == Pending::EntryAttackingDefender { player: p }
        });
        let action = Action::ChooseEntryAttackingDefender {
            defender: CombatDefender::Player(opponent),
        };
        let walker_action = Action::ChooseEntryAttackingDefender {
            defender: CombatDefender::Planeswalker(walker),
        };
        assert!(g.legal_actions().contains(&walker_action));
        reject(
            &mut g,
            Action::ChooseEntryAttackingDefender {
                defender: CombatDefender::Player(p),
            },
        );
        let checkpoint = g.snapshot();
        let revision = g.revision();
        let index = g
            .legal_actions()
            .iter()
            .position(|candidate| *candidate == action)
            .unwrap();
        let mut indexed = g.clone();
        assert_eq!(g.apply(opponent, action.clone()), Err(Error::WrongActor));
        assert_eq!(g.snapshot(), checkpoint);
        act(&mut g, action);
        indexed.apply_index(p, revision, index).unwrap();
        assert_eq!(g.snapshot(), indexed.snapshot());
        indexed.restore(&checkpoint).unwrap();
        assert_eq!(
            indexed.apply_index(p, revision, index),
            Err(Error::StaleRevision)
        );
        act(&mut indexed, walker_action);
        for (branch, expected) in [
            (&mut g, CombatDefender::Player(opponent)),
            (&mut indexed, CombatDefender::Planeswalker(walker)),
        ] {
            settle(branch);
            let result = branch.object_view(enemy_grave);
            assert_eq!(result.zone, Zone::Battlefield);
            assert_eq!(result.controller, p);
            assert_eq!(result.owner, opponent);
            assert!(result.tapped);
            assert_eq!(result.counters.plus_one, 0);
            assert!(
                branch
                    .state
                    .attackers
                    .iter()
                    .any(|a| { a.card == enemy_grave && a.defender == Some(expected) })
            );
            assert!(!branch.state.attacked_this_turn.contains(&result.reference));
            assert_eq!(branch.state.players[p.index()].life, 22);
        }
    }
}

#[test]
fn attached_attack_clause_does_not_trigger_on_blocks_and_void_is_checked_at_resolution() {
    let _rules = fixture_rules::activate(&[(Card::from_name(AURA_NAME).unwrap(), &AURA)]);
    let p = Player::P0;
    let mut g = scenario(p);
    let recipient = put(&mut g, p, "Grizzly Bears", Zone::Battlefield);
    let other = put(&mut g, p, "Bear Cub", Zone::Battlefield);
    put(&mut g, p.other(), "Hill Giant", Zone::Graveyard);
    let aura = cast_aura(&mut g, p, recipient);

    let mut block = g.clone();
    block.state.blocking.push(recipient);
    block.attachment_block_triggers(p);
    assert!(block.state.pending_triggers.is_empty());

    g.begin_step(Step::End);
    advance_until(&mut g, |g| !g.state.stack.is_empty());
    let mut departure = g.clone();
    departure.move_object(other, Zone::Graveyard);
    assert!(departure.observe(p).void_active_this_turn);
    settle(&mut departure);
    assert_eq!(departure.object_view(aura).zone, Zone::Battlefield);
    settle(&mut g);
    assert_eq!(g.object_view(aura).zone, Zone::Graveyard);
}
