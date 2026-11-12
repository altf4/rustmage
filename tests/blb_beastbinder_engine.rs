//! Power-restricted blocking and controller-timed, exact layer-six and layer-seven effects.
mod support;
use rustmage as _;
use rustmage_core::cards::authored::CardRules;
use rustmage_core::engine::funding::FundingGoal;
use rustmage_core::{cards::*, engine::*};
use support::*;

const BEASTBINDER: CardRules = CardRules {
    effect: Effect::Creature(1, 3),
    keywords: Keywords::VIGILANCE,
    static_groups: &[StaticAbilityGroup {
        line: 0,
        first_layer: AbilityLayer::Rule,
        fragments: &[StaticAbilityFragment::CannotBeBlockedByCreaturesWithPowerAtLeast(2)],
    }],
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::SelfAttacks,
        targets: TargetPlan::Single(TargetSpec {
            rule: TargetRule::Object {
                zone: TargetZone::Battlefield,
                kinds: 1 | 16 | 64,
                relation: 1,
                other: false,
                nonland: false,
                tapped: false,
                min_mana: 0,
                max_mana: u32::MAX,
            },
            min: 0,
            max: 1,
        }),
        program: &[
            Instruction::RemoveAbilitiesAndSetCreatureBaseUntilNextTurn {
                power: 2,
                toughness: 2,
            },
        ],
    }],
    ..CardRules::EMPTY
};

fn setup() -> fixture_rules::Guard {
    fixture_rules::activate(&[(Card::GrizzlyBears, &BEASTBINDER)])
}
fn attack(g: &mut Game, attacker: CardId) {
    g.state.step = Step::Attackers;
    g.state.pending = Pending::Attackers;
    act(g, Action::Attack { card: attacker });
    act(g, Action::FinishAttackers);
}
fn pick(card: CardId) -> Action {
    Action::ChooseTarget {
        target: Target::Card(card),
    }
}
fn select(g: &mut Game, card: CardId) {
    act(g, pick(card));
    if g.legal_actions().contains(&Action::FinishTargets) {
        act(g, Action::FinishTargets);
    }
}

#[test]
fn blockers_use_their_live_power_and_ignore_the_attackers_power() {
    let _rules = setup();
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        let attacker = put(&mut g, p, "Grizzly Bears", Zone::Battlefield);
        let small = put(&mut g, p.other(), "Healer's Hawk", Zone::Battlefield);
        let big = put(&mut g, p.other(), "Bear Cub", Zone::Battlefield);
        g.state.objects[attacker as usize].growth.power = 5;
        attack(&mut g, attacker);
        act(&mut g, Action::FinishTargets);
        settle(&mut g);
        assert!(!g.object_view(attacker).tapped);

        g.state.step = Step::Blockers;
        g.state.pending = Pending::Blockers;
        let block_small = Action::Block {
            blocker: small,
            attacker,
        };
        let block_big = Action::Block {
            blocker: big,
            attacker,
        };
        assert!(g.legal_actions().contains(&block_small));
        reject(&mut g, block_big.clone());

        let mut reduced = g.clone();
        reduced.state.objects[big as usize].growth.power = -1;
        act(&mut reduced, block_big.clone());
        let mut grown = g.clone();
        grown.add_counter(small, CounterKind::PlusOne, 1);
        reject(&mut grown, block_small);

        let exact = g.object_ref(attacker);
        g.remove_abilities_exact(&[exact], exact, p, AnimationDuration::Permanent);
        act(&mut g, block_big);
    }
}

#[test]
fn source_departure_preserves_both_layers_until_the_initiating_controllers_turn() {
    let _rules = setup();
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        let attacker = put(&mut g, p, "Grizzly Bears", Zone::Battlefield);
        let elf = put(&mut g, p.other(), "Llanowar Elves", Zone::Battlefield);
        let friendly = put(&mut g, p, "Llanowar Elves", Zone::Battlefield);
        let opposing_land = put(&mut g, p.other(), "Forest", Zone::Battlefield);
        g.add_counter(elf, CounterKind::PlusOne, 1);
        attack(&mut g, attacker);
        reject(&mut g, pick(friendly));
        reject(&mut g, pick(opposing_land));
        let before = g.snapshot();
        let index = g
            .legal_actions()
            .iter()
            .position(|a| *a == pick(elf))
            .unwrap();
        let revision = g.revision();
        let mut indexed = g.clone();
        act(&mut g, pick(elf));
        indexed.apply_index(p, revision, index).unwrap();
        assert_eq!(g.snapshot(), indexed.snapshot());
        indexed.restore(&before).unwrap();
        assert_eq!(
            indexed.apply_index(p, revision, index),
            Err(Error::StaleRevision)
        );
        act(&mut indexed, pick(elf));
        assert_eq!(g.snapshot(), indexed.snapshot());

        if g.legal_actions().contains(&Action::FinishTargets) {
            act(&mut g, Action::FinishTargets);
        }
        if g.actor() != Some(p.other()) {
            act(&mut g, Action::Pass);
        }
        let bounce = put(&mut g, p.other(), "Unsummon", Zone::Hand);
        cast(&mut g, bounce, Some(Target::Card(attacker)));
        settle(&mut g);
        assert_eq!(g.object_view(attacker).zone, Zone::Hand);
        assert!(!g.printed_abilities_available(elf));
        assert_eq!(stats(&g, elf), (3, 3));
        assert_eq!(
            g.observe(p).base_stat_effects[0].expires_at_next_turn_of,
            Some(p)
        );

        let mut blinked = g.clone();
        let old = blinked.object_ref(elf);
        blinked.move_object(elf, Zone::Hand);
        blinked.move_object(elf, Zone::Battlefield);
        assert_ne!(blinked.object_ref(elf), old);
        assert!(blinked.printed_abilities_available(elf));
        assert_eq!(stats(&blinked, elf), (1, 1));

        g.move_object(opposing_land, Zone::Graveyard);
        let green = FundingGoal {
            actor: p.other(),
            required: [0, 0, 0, 0, 1, 0],
            generic: 0,
            reserve: None,
            source: None,
        };
        g.finish_cleanup();
        assert_eq!(g.state.active, p.other());
        assert_eq!(stats(&g, elf), (3, 3));
        assert!(!g.printed_abilities_available(elf));
        assert!(!g.can_fund_for(green.clone(), mana_lots::Purpose::Other));
        g.finish_cleanup();
        assert_eq!(g.state.active, p);
        assert_eq!(stats(&g, elf), (2, 2));
        assert!(g.printed_abilities_available(elf));
        assert!(g.can_fund_for(green, mana_lots::Purpose::Other));
        assert!(g.observe(p).base_stat_effects.is_empty());
    }
}

#[test]
fn creature_status_is_sampled_once_and_the_effect_never_changes_types() {
    let _rules = setup();
    for p in [Player::P0, Player::P1] {
        let mut root = scenario(p);
        let attacker = put(&mut root, p, "Grizzly Bears", Zone::Battlefield);
        let artifact = put(&mut root, p.other(), "Mazemind Tome", Zone::Battlefield);
        attack(&mut root, attacker);
        let mut declined = root.clone();
        act(&mut declined, Action::FinishTargets);
        settle(&mut declined);
        assert!(declined.printed_abilities_available(artifact));
        select(&mut root, artifact);

        for animated_early in [false, true] {
            let mut g = root.clone();
            let animate = |g: &mut Game| {
                g.change_object_form(
                    g.object_ref(artifact),
                    AnimationSpec {
                        artifact: false,
                        creature_types: AnimationCreatureTypes::Unchanged,
                        base: Some((6, 6)),
                        keywords: Keywords::NONE,
                        duration: AnimationDuration::Permanent,
                    },
                    characteristic_layers::AnimationTypeMode::AddCreature,
                );
            };
            if animated_early {
                animate(&mut g);
            }
            settle(&mut g);
            assert!(!g.printed_abilities_available(artifact));
            assert_eq!(g.is_creature(artifact), animated_early);
            assert_eq!(g.current_kinds(artifact) & 16, 16);
            if animated_early {
                assert_eq!(stats(&g, artifact), (2, 2));
                assert_eq!(g.observe(p).base_stat_effects.len(), 1);
            } else {
                assert!(g.observe(p).base_stat_effects.is_empty());
                animate(&mut g);
                assert_eq!(stats(&g, artifact), (6, 6));
            }
        }
    }
}
