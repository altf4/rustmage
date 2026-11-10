//! Prevention effects with after-damage work and Aura-controlled redirection.
mod support;

use rustmage as _;
use rustmage_core::cards::authored::{CardExtras, CardRules};
use rustmage_core::cards::*;
use rustmage_core::engine::*;
use support::*;

const PREVENT: ReplacementSpec = ReplacementSpec::PreventSelfDamageWithCounters;
const PREVENTING_CREATURE: CardRules = CardRules {
    effect: Effect::Creature(3, 3),
    static_groups: &[StaticAbilityGroup {
        line: 0,
        first_layer: AbilityLayer::Rule,
        fragments: &[StaticAbilityFragment::Replacement(0, PREVENT)],
    }],
    extras: &CardExtras {
        replacement_effects: &[PREVENT],
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};

const UNPREVENTABLE: CardRules = CardRules {
    effect: Effect::Creature(2, 2),
    static_groups: &[StaticAbilityGroup {
        line: 0,
        first_layer: AbilityLayer::Rule,
        fragments: &[StaticAbilityFragment::DamageCantBePrevented],
    }],
    ..CardRules::EMPTY
};

const REDIRECT: ReplacementSpec = ReplacementSpec::RedirectControllerDamageToEnchantedCreature;
const AURA: AuraSpec = AuraSpec {
    kinds: 1,
    relation: 0,
    bonus: EquipmentSpec {
        stats: Stats {
            power: 0,
            toughness: 0,
        },
        keywords: Keywords::NONE,
    },
};
const REDIRECTING_AURA: CardRules = CardRules {
    effect: Effect::Aura(TargetPlan::Single(TargetSpec::one(
        TargetRule::creature_you_control(),
    ))),
    aura: Some(AURA),
    static_groups: &[
        StaticAbilityGroup {
            line: 1,
            first_layer: AbilityLayer::Modify,
            fragments: &[StaticAbilityFragment::AttachmentStatsPerAttachment(Stats {
                power: 2,
                toughness: 2,
            })],
        },
        StaticAbilityGroup {
            line: 2,
            first_layer: AbilityLayer::Rule,
            fragments: &[StaticAbilityFragment::Replacement(0, REDIRECT)],
        },
    ],
    extras: &CardExtras {
        aura: Some(AURA),
        replacement_effects: &[REDIRECT],
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};

#[test]
fn prevention_places_counters_through_replacement_even_when_damage_cannot_be_prevented() {
    let _rules = fixture_rules::activate(&[
        (Card::HillGiant, &PREVENTING_CREATURE),
        (Card::from_name("Bear Cub").unwrap(), &UNPREVENTABLE),
    ]);
    let attacker = Player::P0;
    for unpreventable in [false, true] {
        let mut game = scenario(attacker);
        let recipient = put(&mut game, attacker.other(), "Hill Giant", Zone::Battlefield);
        put(
            &mut game,
            attacker.other(),
            "Doubling Season",
            Zone::Battlefield,
        );
        if unpreventable {
            put(&mut game, attacker, "Bear Cub", Zone::Battlefield);
        }
        let shock = put(&mut game, attacker, "Shock", Zone::Hand);
        cast(&mut game, shock, Some(Target::Card(recipient)));
        settle(&mut game);
        assert_eq!(game.counter_count(recipient, CounterKind::PlusOne), 4);
        assert_eq!(
            game.object_view(recipient).damage,
            if unpreventable { 2 } else { 0 }
        );
    }
}

#[test]
fn redirection_uses_exact_aura_recipients_and_dynamic_stats_count_attachments_on_either_side() {
    let _rules =
        fixture_rules::activate(&[(Card::from_name("Pacifism").unwrap(), &REDIRECTING_AURA)]);
    let attacker = Player::P0;
    let defender = attacker.other();
    let mut parent = scenario(attacker);
    let first = put(&mut parent, defender, "Hill Giant", Zone::Battlefield);
    let second = put(&mut parent, defender, "Hill Giant", Zone::Battlefield);
    let first_aura = put(&mut parent, defender, "Pacifism", Zone::Battlefield);
    let second_aura = put(&mut parent, defender, "Pacifism", Zone::Battlefield);
    let equipment = put(
        &mut parent,
        attacker,
        "Quick-Draw Katana",
        Zone::Battlefield,
    );
    parent.state.objects[first_aura as usize].attached_to = Some(parent.object_ref(first));
    parent.state.objects[second_aura as usize].attached_to = Some(parent.object_ref(second));
    parent.state.objects[equipment as usize].attached_to = Some(parent.object_ref(first));

    assert_eq!(parent.object_view(second).toughness, 5);
    assert_eq!(parent.object_view(first).toughness, 7);
    let shock = put(&mut parent, attacker, "Shock", Zone::Hand);
    cast(&mut parent, shock, Some(Target::Player(defender)));
    act(&mut parent, Action::Pass);
    act(&mut parent, Action::Pass);
    assert_eq!(parent.actor(), Some(defender));
    let choose = |game: &Game, aura| Action::ChooseReplacement {
        source: game.object_ref(aura),
        ability: 0,
    };
    let first_choice = choose(&parent, first_aura);
    let second_choice = choose(&parent, second_aura);
    let revision = parent.revision();
    let index = parent
        .legal_actions()
        .iter()
        .position(|action| *action == first_choice)
        .unwrap();
    let before = parent.snapshot();
    let mut raw = parent.clone();
    let mut indexed = parent.clone();
    drop(parent);

    act(&mut raw, first_choice);
    indexed.apply_index(defender, revision, index).unwrap();
    settle(&mut raw);
    settle(&mut indexed);
    assert_eq!(raw.snapshot(), indexed.snapshot());
    assert_eq!(raw.state.players[defender.index()].life, 20);
    assert_eq!(raw.object_view(first).damage, 2);
    assert_eq!(raw.object_view(second).damage, 0);

    indexed.restore(&before).unwrap();
    act(&mut indexed, second_choice);
    settle(&mut indexed);
    assert_eq!(indexed.state.players[defender.index()].life, 20);
    assert_eq!(indexed.object_view(first).damage, 0);
    assert_eq!(indexed.object_view(second).damage, 2);
}

const ENEMY: TargetRule = TargetRule::Object {
    zone: TargetZone::Battlefield,
    kinds: 1,
    relation: 1,
    other: false,
    nonland: false,
    tapped: false,
    min_mana: 0,
    max_mana: u32::MAX,
};

const TEAM_UP: CardRules = CardRules {
    effect: Effect::Spell(
        TargetPlan::Groups(&[
            TargetSpec {
                rule: TargetRule::creature_you_control(),
                min: 1,
                max: 2,
            },
            TargetSpec::one(ENEMY),
        ]),
        &[
            Instruction::Grow {
                subject: Subject::Targets,
                power: 1,
                toughness: 0,
            },
            Instruction::GroupPowerDamage {
                source: 0,
                recipient: 1,
            },
        ],
    ),
    ..CardRules::EMPTY
};

const TOUGH_TARGET: CardRules = CardRules {
    effect: Effect::Creature(3, 8),
    ..CardRules::EMPTY
};

#[test]
fn both_group_members_deal_updated_power_damage_and_stale_members_are_ignored() {
    let _rules = fixture_rules::activate(&[
        (Card::from_name("Opt").unwrap(), &TEAM_UP),
        (Card::HillGiant, &TOUGH_TARGET),
    ]);
    let player = Player::P0;
    let mut game = scenario(player);
    let first = put(&mut game, player, "Grizzly Bears", Zone::Battlefield);
    let second = put(&mut game, player, "Grizzly Bears", Zone::Battlefield);
    let recipient = put(&mut game, player.other(), "Hill Giant", Zone::Battlefield);
    let spell = put(&mut game, player, "Opt", Zone::Hand);
    announce(
        &mut game,
        spell,
        &[
            Target::Card(first),
            Target::Card(second),
            Target::Card(recipient),
        ],
    );

    let mut partial = game.clone();
    let bounce = put(&mut partial, player, "Unsummon", Zone::Hand);
    cast(&mut partial, bounce, Some(Target::Card(second)));
    settle(&mut partial);
    assert_eq!(partial.object_view(second).zone, Zone::Hand);
    assert_eq!(partial.object_view(first).power, 3);
    assert_eq!(partial.object_view(recipient).damage, 3);

    settle(&mut game);
    assert_eq!(game.object_view(first).power, 3);
    assert_eq!(game.object_view(second).power, 3);
    assert_eq!(game.object_view(recipient).damage, 6);
}

const FIGHT_EXCESS: CardRules = CardRules {
    effect: Effect::Spell(
        TargetPlan::Groups(&[
            TargetSpec::one(TargetRule::creature_you_control()),
            TargetSpec::one(ENEMY),
        ]),
        &[
            Instruction::FightForExcess {
                source: 0,
                recipient: 1,
            },
            Instruction::ReflexiveForExcessDamage { ability: 0 },
        ],
    ),
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::Reflexive,
        targets: TargetPlan::Single(TargetSpec {
            rule: TargetRule::Object {
                zone: TargetZone::Battlefield,
                kinds: 16,
                relation: -1,
                other: false,
                nonland: false,
                tapped: false,
                min_mana: 0,
                max_mana: 3,
            },
            min: 0,
            max: 1,
        }),
        program: &[Instruction::Destroy],
    }],
    ..CardRules::EMPTY
};

#[test]
fn only_actual_outgoing_excess_opens_a_separate_target_choice() {
    let _rules = fixture_rules::activate(&[(Card::from_name("Opt").unwrap(), &FIGHT_EXCESS)]);
    let player = Player::P0;
    for (recipient_name, shield, excess) in [
        ("Grizzly Bears", false, true),
        ("Hill Giant", false, false),
        ("Grizzly Bears", true, false),
    ] {
        let mut game = scenario(player);
        let first = put(&mut game, player, "Hill Giant", Zone::Battlefield);
        let second = put(&mut game, player.other(), recipient_name, Zone::Battlefield);
        let artifact = put(
            &mut game,
            player.other(),
            "Quick-Draw Katana",
            Zone::Battlefield,
        );
        let spell = put(&mut game, player, "Opt", Zone::Hand);
        if shield {
            game.add_counter(second, CounterKind::Shield, 1);
        }
        announce(
            &mut game,
            spell,
            &[Target::Card(first), Target::Card(second)],
        );
        assert_eq!(game.state.stack.len(), 1);
        act(&mut game, Action::Pass);
        act(&mut game, Action::Pass);
        assert_eq!(
            matches!(game.state.pending, Pending::Targets { .. }),
            excess
        );
        if excess {
            assert_eq!(game.object_view(second).zone, Zone::Graveyard);
            act(
                &mut game,
                Action::ChooseTarget {
                    target: Target::Card(artifact),
                },
            );
        }
        settle(&mut game);
        assert_eq!(
            game.object_view(artifact).zone,
            if excess {
                Zone::Graveyard
            } else {
                Zone::Battlefield
            }
        );
    }
}

const DAMAGE_SUBTYPE: CardRules = CardRules {
    effect: Effect::Creature(3, 3),
    activated: &[ActivatedAbility {
        exhaust: false,
        guard: ActivationGuard::Always,
        cycling: false,
        origin: ActivationOrigin::Battlefield,
        cost: ActivationCost::ZERO,
        targets: TargetPlan::NONE,
        program: &[Instruction::DamageCreaturesExceptControlledTokens(1)],
        sorcery: false,
        once_per_turn: false,
    }],
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::SourceDealtDamageToCreatureSubtype("Bird"),
        targets: TargetPlan::NONE,
        program: &[Instruction::DestroyEventPermanent],
    }],
    ..CardRules::EMPTY
};

const SHROUDED_BIRD: CardRules = CardRules {
    effect: Effect::Creature(1, 4),
    keywords: Keywords::SHROUD,
    ..CardRules::EMPTY
};

#[test]
fn only_the_positively_damaged_subtype_is_destroyed_without_targeting() {
    let _rules = fixture_rules::activate(&[
        (Card::HillGiant, &DAMAGE_SUBTYPE),
        (Card::from_name("Healer's Hawk").unwrap(), &SHROUDED_BIRD),
    ]);
    let player = Player::P0;
    let mut game = scenario(player);
    let source = put(&mut game, player, "Hill Giant", Zone::Battlefield);
    let bird = put(
        &mut game,
        player.other(),
        "Healer's Hawk",
        Zone::Battlefield,
    );
    let shielded = put(
        &mut game,
        player.other(),
        "Healer's Hawk",
        Zone::Battlefield,
    );
    let other = put(
        &mut game,
        player.other(),
        "Grizzly Bears",
        Zone::Battlefield,
    );
    game.add_counter(shielded, CounterKind::Shield, 1);
    act(
        &mut game,
        Action::AnnounceActivate {
            card: source,
            ability: 0,
            counter_payment: Counters::default(),
        },
    );
    if game
        .legal_actions()
        .iter()
        .any(|action| matches!(action, Action::PayCosts { .. }))
    {
        pay(&mut game);
    }
    settle(&mut game);
    assert_eq!(game.object_view(bird).zone, Zone::Graveyard);
    assert_eq!(game.object_view(shielded).zone, Zone::Battlefield);
    assert_eq!(game.object_view(shielded).damage, 0);
    assert_eq!(game.object_view(other).damage, 1);
}
