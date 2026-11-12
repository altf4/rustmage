//! Resolved characteristic additions and a multi-layer creature Aura.
mod support;

use rustmage as _;
use rustmage_core::cards::authored::CardRules;
use rustmage_core::{cards::*, engine::*};
use support::*;

const CREATURE: TargetPlan = TargetPlan::Single(TargetSpec::one(TargetRule::Creature));
const CREATURE_KIND: u16 = 1;
const ARTIFACT_KIND: u16 = 16;
const ENCHANTMENT_KIND: u16 = 32;
const WHITE: u8 = 1;
const BLUE: u8 = 2;
const BLACK: u8 = 4;
const GREEN: u8 = 16;

const BOOST: CardRules = CardRules {
    effect: Effect::Spell(
        CREATURE,
        &[
            Instruction::Grow {
                subject: Subject::Targets,
                power: 2,
                toughness: 2,
            },
            Instruction::GrantKeywords {
                subject: Subject::Targets,
                keywords: Keywords::FLYING,
            },
            Instruction::AddCharacteristics {
                subject: Subject::Targets,
                addition: CharacteristicAddition {
                    card_types: CREATURE_KIND | ENCHANTMENT_KIND,
                    creature_types: &["Horror"],
                    colors: 0,
                    duration: AnimationDuration::EndOfTurn,
                },
            },
        ],
    ),
    ..CardRules::EMPTY
};

const SILENCER: CardRules = CardRules {
    effect: Effect::Aura(CREATURE),
    aura: Some(AuraSpec {
        kinds: 1,
        relation: -1,
        bonus: EquipmentSpec {
            stats: Stats {
                power: 0,
                toughness: 0,
            },
            keywords: Keywords::NONE,
        },
    }),
    static_groups: &[StaticAbilityGroup {
        line: 1,
        first_layer: AbilityLayer::Type,
        fragments: &[
            StaticAbilityFragment::AttachedCharacteristics(&AttachedCharacteristics {
                name: None,
                form: PermanentForm::AddCardTypes(CREATURE_KIND | ARTIFACT_KIND),
                creature_types: &["Toy"],
                colors: &[],
                base: Some((0, 2)),
            }),
            StaticAbilityFragment::RemoveAttachedAbilities,
        ],
    }],
    ..CardRules::EMPTY
};

const EERIE: CardRules = CardRules {
    effect: Effect::Creature(2, 2),
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::Eerie,
        targets: TargetPlan::NONE,
        program: &[Instruction::Draw(PlayerSet::Controller, 1)],
    }],
    ..CardRules::EMPTY
};

const DEMON_CHANGE: CardRules = CardRules {
    effect: Effect::Creature(1, 1),
    activated: &[ActivatedAbility {
        origin: ActivationOrigin::Battlefield,
        guard: ActivationGuard::OncePerIncarnation(&ActivationGuard::Always),
        cost: ActivationCost {
            generic: 3,
            mana_component: true,
            additional: Some(AdditionalCost {
                kind: EffectChoiceKind::Discard,
                filter: SacrificeFilter {
                    keywords: Keywords::NONE,
                    source_only: false,
                    kinds: 0,
                    nonland: false,
                    other: false,
                    tokens: None,
                    artifact_enchantment_or_token: false,
                    subtype: None,
                },
            }),
            ..ActivationCost::ZERO
        },
        targets: TargetPlan::NONE,
        program: &[
            Instruction::PutCounters {
                subject: Subject::Source,
                kind: CounterKind::PlusOne,
                amount: 3,
            },
            Instruction::AddCharacteristics {
                subject: Subject::Source,
                addition: CharacteristicAddition {
                    card_types: 0,
                    creature_types: &["Demon"],
                    colors: BLACK,
                    duration: AnimationDuration::Permanent,
                },
            },
        ],
        sorcery: false,
        once_per_turn: false,
        cycling: false,
        exhaust: false,
    }],
    ..CardRules::EMPTY
};

fn card(name: &str) -> Card {
    Card::from_name(name).unwrap()
}

fn end_turn(game: &mut Game) {
    let active = game.state.active;
    game.state.step = Step::End;
    act(game, Action::Pass);
    act(game, Action::Pass);
    assert_eq!(game.state.active, active.other());
    assert_eq!(game.state.step, Step::Upkeep);
}

#[test]
fn temporary_enchantment_and_horror_types_keep_the_artifact_and_do_not_enter() {
    let boost = card("Opt");
    let _rules = fixture_rules::activate(&[(boost, &BOOST), (Card::GrizzlyBears, &EERIE)]);
    let player = Player::P0;
    let mut game = scenario(player);
    let creature = put(&mut game, player, "Gingerbrute", Zone::Battlefield);
    spawn(&mut game, player, Card::GrizzlyBears, Zone::Battlefield);
    let spell = spawn(&mut game, player, boost, Zone::Hand);
    let original = game.object_view(creature);
    cast(&mut game, spell, Some(Target::Card(creature)));
    settle(&mut game);
    let boosted = game.object_view(creature);
    assert_eq!(boosted.reference, original.reference);
    assert_eq!(boosted.kinds, original.kinds | ENCHANTMENT_KIND);
    assert!(boosted.has_subtype("Food") && boosted.has_subtype("Golem"));
    assert!(boosted.has_subtype("Horror"));
    assert!(boosted.keywords.contains(Keywords::FLYING));
    assert_eq!(stats(&game, creature), (3, 3));
    assert_eq!(game.state.players[player.index()].cards_drawn_this_turn, 0);

    end_turn(&mut game);
    let ended = game.object_view(creature);
    assert_eq!(ended.kinds, original.kinds);
    assert!(!ended.has_subtype("Horror"));
    assert!(!ended.keywords.contains(Keywords::FLYING));
    assert_eq!(stats(&game, creature), (1, 1));
}

#[test]
fn attached_type_ability_and_base_effects_keep_one_timestamp_and_exact_recipient() {
    let boost = card("Opt");
    let silencer = card("Charmed Sleep");
    let _rules = fixture_rules::activate(&[(boost, &BOOST), (silencer, &SILENCER)]);
    let player = Player::P0;
    for aura_first in [false, true] {
        let mut game = scenario(player);
        let creature = put(&mut game, player, "Healer's Hawk", Zone::Battlefield);
        let other = put(&mut game, player, "Healer's Hawk", Zone::Battlefield);
        let aura = spawn(&mut game, player, silencer, Zone::Hand);
        let spell = spawn(&mut game, player, boost, Zone::Hand);
        let setter = put(&mut game, player, "Witness Protection", Zone::Hand);
        let reference = game.object_ref(creature);
        game.add_counter(creature, CounterKind::PlusOne, 1);

        let order = if aura_first {
            [aura, spell]
        } else {
            [spell, aura]
        };
        for source in order {
            cast(&mut game, source, Some(Target::Card(creature)));
            settle(&mut game);
        }
        let check_combined = |game: &Game| {
            let view = game.object_view(creature);
            assert_eq!(view.reference, reference);
            assert_eq!(view.kinds, CREATURE_KIND | ARTIFACT_KIND | ENCHANTMENT_KIND);
            assert_eq!(view.colors, WHITE);
            assert!(view.has_subtype("Toy") && view.has_subtype("Horror"));
            assert!(view.has_subtype("Bird"));
            assert_eq!(view.keywords.contains(Keywords::FLYING), aura_first);
            assert!(!view.keywords.contains(Keywords::LIFELINK));
            assert_eq!(stats(game, creature), (3, 5));
        };
        assert_eq!(game.object_view(aura).attached_to, Some(reference));
        check_combined(&game);
        assert_eq!(game.object_view(other).kinds, CREATURE_KIND);
        assert!(!game.object_view(other).has_subtype("Toy"));
        assert_eq!(stats(&game, other), (1, 1));

        cast(&mut game, setter, Some(Target::Card(creature)));
        settle(&mut game);
        let overwritten = game.object_view(creature);
        assert_eq!(overwritten.kinds, CREATURE_KIND);
        assert!(overwritten.has_subtype("Citizen"));
        assert!(!overwritten.has_subtype("Toy") && !overwritten.has_subtype("Horror"));
        assert_eq!(stats(&game, creature), (4, 4));
        game.move_object(setter, Zone::Exile);
        check_combined(&game);

        let aura_ref = game.object_ref(aura);
        game.remove_abilities_exact(&[aura_ref], aura_ref, player, AnimationDuration::Permanent);
        check_combined(&game);
        game.move_object(aura, Zone::Exile);
        let freed = game.object_view(creature);
        assert_eq!(freed.kinds, CREATURE_KIND | ENCHANTMENT_KIND);
        assert!(!freed.has_subtype("Toy") && freed.has_subtype("Horror"));
        assert!(
            freed
                .keywords
                .contains(Keywords::FLYING.union(Keywords::LIFELINK))
        );
        assert_eq!(stats(&game, creature), (4, 4));
    }
}

#[test]
fn permanent_additive_color_follows_setter_timestamps_and_survives_ability_loss() {
    let definition = card("Healer's Hawk");
    let _rules = fixture_rules::activate(&[(definition, &DEMON_CHANGE)]);
    let player = Player::P0;
    let mut game = scenario(player);
    let creature = spawn(&mut game, player, definition, Zone::Battlefield);
    let discarded = put(&mut game, player, "Forest", Zone::Hand);
    put(&mut game, player, "Forest", Zone::Hand);
    let original = game.object_ref(creature);
    let activation = Action::AnnounceActivate {
        card: creature,
        ability: 0,
        counter_payment: Counters::default(),
    };
    game.set_color_effects(&[creature], GREEN, AnimationDuration::EndOfTurn);
    act(&mut game, activation.clone());
    let discard = game.object_ref(discarded);
    act(&mut game, Action::ChooseAdditionalCost { object: discard });
    pay(&mut game);
    settle(&mut game);
    assert_eq!(game.object_view(discarded).zone, Zone::Graveyard);
    assert_eq!(game.object_view(creature).colors, GREEN | BLACK);
    assert!(game.object_view(creature).has_subtype("Bird"));
    assert!(game.object_view(creature).has_subtype("Demon"));
    assert_eq!(stats(&game, creature), (4, 4));
    reject(&mut game, activation.clone());

    game.set_color_effects(&[creature], BLUE, AnimationDuration::EndOfTurn);
    assert_eq!(game.object_view(creature).colors, BLUE);
    game.remove_abilities_exact(&[original], original, player, AnimationDuration::Permanent);
    end_turn(&mut game);
    assert_eq!(game.object_view(creature).colors, WHITE | BLACK);
    assert!(game.object_view(creature).has_subtype("Demon"));
    assert_eq!(stats(&game, creature), (4, 4));

    game.move_object(creature, Zone::Exile);
    game.move_object(creature, Zone::Battlefield);
    assert_ne!(game.object_ref(creature), original);
    assert_eq!(game.object_view(creature).colors, WHITE);
    assert!(!game.object_view(creature).has_subtype("Demon"));
    assert_eq!(stats(&game, creature), (1, 1));
    act(&mut game, Action::Pass);
    game.state.players[player.index()].mana = [0, 0, 0, 0, 0, 3];
    assert!(game.legal_actions().contains(&activation));
}

#[test]
fn attached_hexproof_lasts_only_for_the_exact_auras_entry_turn() {
    use rustmage_core::cards::authored::CardExtras;
    const BONUS: Stats = Stats {
        power: 1,
        toughness: 1,
    };
    const AURA: AuraSpec = AuraSpec {
        kinds: 1,
        relation: 0,
        bonus: EquipmentSpec {
            stats: BONUS,
            keywords: Keywords::NONE,
        },
    };
    const GROUPS: &[StaticAbilityGroup] = &[
        StaticAbilityGroup {
            line: 1,
            first_layer: AbilityLayer::Ability,
            fragments: &[StaticAbilityFragment::AttachmentEntryTurnKeywords(
                Keywords::HEXPROOF,
            )],
        },
        StaticAbilityGroup {
            line: 2,
            first_layer: AbilityLayer::Modify,
            fragments: &[StaticAbilityFragment::AttachmentStats(BONUS)],
        },
    ];
    const RESCUE: CardRules = CardRules {
        effect: Effect::Aura(CREATURE),
        keywords: Keywords::FLASH,
        aura: Some(AURA),
        static_groups: GROUPS,
        extras: &CardExtras {
            aura: Some(AURA),
            static_ability_groups: GROUPS,
            ..CardExtras::EMPTY
        },
        ..CardRules::EMPTY
    };
    let _scope = fixture_rules::activate(&[(Card::from_name("Pacifism").unwrap(), &RESCUE)]);
    let active = Player::P0;
    let player = active.other();
    let mut game = scenario(active);
    let protected = put(&mut game, player, "Bear Cub", Zone::Battlefield);
    let shock = put(&mut game, active, "Shock", Zone::Hand);
    let aura = put(&mut game, player, "Pacifism", Zone::Hand);
    cast(&mut game, shock, Some(Target::Card(protected)));
    act(&mut game, Action::Pass);
    cast(&mut game, aura, Some(Target::Card(protected)));
    settle(&mut game);
    assert_eq!(game.object_view(protected).damage, 0);
    assert!(game.keywords(protected).contains(Keywords::HEXPROOF));
    assert_eq!(stats(&game, protected), (3, 3));

    advance_turn(&mut game);
    assert_eq!(stats(&game, protected), (3, 3));
    assert!(!game.keywords(protected).contains(Keywords::HEXPROOF));
}

fn advance_turn(game: &mut Game) {
    let turn = game.state.turn;
    for _ in 0..32 {
        if game.state.turn != turn {
            return;
        }
        choose(game, |action| {
            matches!(action, Action::Pass | Action::FinishAttackers)
        });
    }
    panic!("next turn did not begin");
}

#[test]
fn flight_removal_lasts_until_the_source_controllers_next_turn() {
    const WEAKEN: CardRules = CardRules {
        effect: Effect::Spell(
            CREATURE,
            &[
                Instruction::TargetStatsUntilNextTurn {
                    power: -2,
                    toughness: 0,
                },
                Instruction::RemoveKeywordsUntilNextTurn {
                    subject: Subject::Targets,
                    keywords: Keywords::FLYING,
                },
            ],
        ),
        ..CardRules::EMPTY
    };
    const FLIGHT: CardRules = CardRules {
        effect: Effect::Spell(
            CREATURE,
            &[Instruction::GrantKeywords {
                subject: Subject::Targets,
                keywords: Keywords::FLYING,
            }],
        ),
        ..CardRules::EMPTY
    };
    let _scope = fixture_rules::activate(&[
        (Card::from_name("Opt").unwrap(), &WEAKEN),
        (Card::from_name("Giant Growth").unwrap(), &FLIGHT),
    ]);
    let player = Player::P0;
    let opponent = player.other();
    let mut game = scenario(player);
    let creature = put(&mut game, opponent, "Serra Angel", Zone::Battlefield);
    assert!(game.keywords(creature).contains(Keywords::FLYING));
    let spell = put(&mut game, player, "Opt", Zone::Hand);
    cast(&mut game, spell, Some(Target::Card(creature)));
    settle(&mut game);
    assert!(!game.keywords(creature).contains(Keywords::FLYING));
    assert_eq!(stats(&game, creature), (2, 4));

    advance_turn(&mut game);
    assert_eq!(game.state.active, opponent);
    assert!(!game.keywords(creature).contains(Keywords::FLYING));
    assert_eq!(stats(&game, creature), (2, 4));
    let mut still_removed = game.clone();
    advance_turn(&mut still_removed);
    assert_eq!(still_removed.state.active, player);
    assert!(still_removed.keywords(creature).contains(Keywords::FLYING));
    assert_eq!(stats(&still_removed, creature), (4, 4));

    game.state.players[opponent.index()].mana[4] = 1;
    let spell = put(&mut game, opponent, "Giant Growth", Zone::Hand);
    cast(&mut game, spell, Some(Target::Card(creature)));
    settle(&mut game);
    assert!(game.keywords(creature).contains(Keywords::FLYING));
    assert_eq!(stats(&game, creature), (2, 4));
}
