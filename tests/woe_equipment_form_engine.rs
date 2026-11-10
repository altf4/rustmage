//! Permanent Equipment form, layered ability replacement, and exact acquisition.
mod support;

use rustmage as _;
use rustmage_core::cards::authored::{CardExtras, CardRules};
use rustmage_core::{cards::*, engine::*};
use support::*;

const EQUIPMENT_NAME: &str = "Testforge, Last Hammer";
const SOURCE_NAME: &str = "A Realm Reborn";
const COLORLESS: Mana = [0, 0, 0, 0, 0, 1];
const GREEN: Mana = [0, 0, 0, 0, 1, 0];
const BASE: ActivatedAbility = ActivatedAbility {
    exhaust: false,
    guard: ActivationGuard::Always,
    cycling: false,
    origin: ActivationOrigin::Battlefield,
    cost: ActivationCost::ZERO,
    targets: TargetPlan::NONE,
    program: &[],
    sorcery: false,
    once_per_turn: false,
};
const CONTROLLED_CREATURE: TargetRule = TargetRule::Object {
    zone: TargetZone::Battlefield,
    kinds: 1,
    relation: 0,
    other: false,
    nonland: false,
    tapped: false,
    min_mana: 0,
    max_mana: u32::MAX,
};
const SOURCE: CardRules = CardRules {
    effect: Effect::Permanent,
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::FilteredEntersYou(EntryPredicate::LegendaryCreature),
        targets: TargetPlan::NONE,
        program: &[
            Instruction::MayIfSourceOnBattlefield { skip: 1 },
            Instruction::BecomeLegendaryEquipment {
                name: EQUIPMENT_NAME,
                equip: 1,
                power: 3,
                toughness: 3,
            },
        ],
    }],
    activated: &[
        ActivatedAbility {
            cost: ActivationCost {
                tap: true,
                ..ActivationCost::ZERO
            },
            program: &[Instruction::AddMana(ManaProduction::Fixed(COLORLESS))],
            ..BASE
        },
        ActivatedAbility {
            cost: ActivationCost {
                generic: 3,
                mana_component: true,
                ..ActivationCost::ZERO
            },
            targets: TargetPlan::Single(TargetSpec::one(CONTROLLED_CREATURE)),
            program: &[Instruction::AttachSource],
            sorcery: true,
            ..BASE
        },
    ],
    extras: &CardExtras {
        self_granted_activation_templates: &[1],
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
const RETURN_TOGETHER: CardRules = CardRules {
    effect: Effect::Spell(
        TargetPlan::NONE,
        &[Instruction::ReturnOwnedGraveyardNonlandPermanents],
    ),
    ..CardRules::EMPTY
};
const MANA_GRANT: CardRules = CardRules {
    effect: Effect::Permanent,
    static_groups: &[StaticAbilityGroup {
        line: 0,
        first_layer: AbilityLayer::Ability,
        fragments: &[StaticAbilityFragment::ControlledPermanentTapMana {
            kinds: 16 | 32,
            production: ManaProduction::Fixed(GREEN),
        }],
    }],
    ..CardRules::EMPTY
};
const fn target_source(program: &'static [Instruction]) -> CardRules {
    CardRules {
        effect: Effect::Spell(
            TargetPlan::Single(TargetSpec::one(TargetRule::Object {
                zone: TargetZone::Battlefield,
                kinds: 16 | 32,
                relation: -1,
                other: false,
                nonland: false,
                tapped: false,
                min_mana: 0,
                max_mana: u32::MAX,
            })),
            program,
        ),
        ..CardRules::EMPTY
    }
}
const COPY: CardRules = target_source(&[Instruction::CreateCopyToken {
    subject: Subject::Targets,
    amount: 1,
}]);
const BLANK: CardRules = target_source(&[Instruction::RemoveTargetAbilitiesUntilEndOfTurn]);
const BLINK: CardRules = target_source(&[Instruction::BlinkImmediately {
    counter: None,
    tapped: false,
}]);

fn card(name: &str) -> Card {
    Card::from_name(name).unwrap()
}

fn tap_source(card: CardId) -> Action {
    Action::ActivateMana {
        card,
        ability: 0,
        counter_payment: Counters::default(),
        payment: [0; 6],
        production: COLORLESS,
    }
}

fn tap_granted(game: &Game, grantor: CardId, card: CardId) -> Action {
    Action::TapGrantedMana {
        card,
        grantor: game.object_ref(grantor),
        production: GREEN,
    }
}

fn reach_optional(game: &mut Game, player: Player) {
    for _ in 0..24 {
        if let Pending::EffectOptional {
            player: chooser, ..
        } = game.state.pending
        {
            assert_eq!(chooser, player);
            return;
        }
        choose(game, |action| {
            matches!(
                action,
                Action::Pass | Action::PutTrigger { .. } | Action::ChooseEntryTimestamp { .. }
            )
        });
    }
    panic!("equipment form did not offer its optional resolution");
}

fn equip_action(game: &Game, source: ObjectRef) -> Action {
    let rows = game
        .observe(game.actor().unwrap())
        .self_granted_abilities
        .unwrap();
    let grant = rows
        .iter()
        .find(|row| {
            row.recipient == source
                && matches!(row.kind, SelfGrantedAbilityKind::EquipAndBoost { .. })
        })
        .unwrap();
    Action::AnnounceSelfGrantedAbility {
        source,
        grant_timestamp: grant.timestamp,
        counter_payment: Counters::default(),
    }
}

fn equip(game: &mut Game, source: ObjectRef, target: CardId) {
    let action = equip_action(game, source);
    act(game, action);
    act(
        game,
        Action::ChooseTarget {
            target: Target::Card(target),
        },
    );
    if game.legal_actions().contains(&Action::FinishTargets) {
        act(game, Action::FinishTargets);
    }
    pay(game);
    settle(game);
}

#[test]
fn simultaneous_legendary_creature_entry_offers_an_owned_choice_and_paid_equip() {
    let _rules = fixture_rules::activate(&[
        (card(SOURCE_NAME), &SOURCE),
        (card("Opt"), &RETURN_TOGETHER),
    ]);
    for player in [Player::P0, Player::P1] {
        let mut parent = scenario(player);
        let source = put(&mut parent, player, SOURCE_NAME, Zone::Graveyard);
        let legendary = put(&mut parent, player, "Fynn, the Fangbearer", Zone::Graveyard);
        let bear = put(&mut parent, player, "Grizzly Bears", Zone::Graveyard);
        let opposing_bear = put(
            &mut parent,
            player.other(),
            "Grizzly Bears",
            Zone::Battlefield,
        );
        let artifact = put(&mut parent, player, "Mox Jasper", Zone::Graveyard);
        let returned = put(&mut parent, player, "Opt", Zone::Hand);
        cast(&mut parent, returned, None);
        reach_optional(&mut parent, player);
        let exact_source = parent.object_ref(source);
        let object = parent.object_view(source);
        assert_eq!(parent.object_view(artifact).zone, Zone::Battlefield);
        assert_eq!(parent.object_view(legendary).zone, Zone::Battlefield);
        assert_eq!(parent.object_view(bear).zone, Zone::Battlefield);
        assert_eq!(object.name(), SOURCE_NAME);
        assert_eq!(object.kinds, 32);
        assert_eq!(
            parent
                .state
                .resolving
                .as_ref()
                .unwrap()
                .item
                .entering_object,
            Some(parent.object_ref(legendary))
        );

        let accept = Action::ChooseOptional { accept: true };
        let mut declined = parent.clone();
        act(&mut declined, Action::ChooseOptional { accept: false });
        assert_eq!(declined.state.pending, Pending::Priority);
        assert!(declined.state.stack.is_empty());
        assert_eq!(declined.object_view(source).name(), SOURCE_NAME);
        assert!(declined.legal_actions().contains(&tap_source(source)));

        let snapshot = parent.snapshot();
        let revision = parent.revision();
        let index = parent
            .legal_actions()
            .iter()
            .position(|action| *action == accept)
            .unwrap();
        let mut indexed = parent.clone();
        indexed.apply_index(player, revision, index).unwrap();
        act(&mut parent, accept.clone());
        assert_eq!(indexed.snapshot(), parent.snapshot());
        indexed.restore(&snapshot).unwrap();
        assert_eq!(
            indexed.apply_index(player, revision, index),
            Err(Error::StaleRevision)
        );
        act(&mut indexed, accept);
        assert_eq!(indexed.snapshot(), parent.snapshot());

        let transformed = parent.object_view(source);
        assert_eq!(transformed.reference, exact_source);
        assert_eq!(transformed.name(), EQUIPMENT_NAME);
        assert_eq!(transformed.kinds, 16);
        assert_eq!(transformed.colors, object.colors);
        assert!(transformed.is_legendary());
        assert!(transformed.has_subtype("Equipment"));
        assert_eq!(parent.object_mana_value(source), 6);
        reject(&mut parent, tap_source(source));

        let announce = equip_action(&parent, exact_source);
        parent.state.players[player.index()].mana = [0, 0, 0, 0, 0, 2];
        reject(&mut parent, announce.clone());
        parent.state.players[player.index()].mana[5] = 3;
        act(&mut parent, announce);
        reject(
            &mut parent,
            Action::ChooseTarget {
                target: Target::Card(opposing_bear),
            },
        );
        act(
            &mut parent,
            Action::ChooseTarget {
                target: Target::Card(bear),
            },
        );
        if parent.legal_actions().contains(&Action::FinishTargets) {
            act(&mut parent, Action::FinishTargets);
        }
        pay(&mut parent);
        settle(&mut parent);
        assert_eq!(parent.state.players[player.index()].mana, [0; 6]);
        assert_eq!(
            parent.object_view(source).attached_to,
            Some(parent.object_ref(bear))
        );
        assert!(parent.object_view(bear).modified);
        assert_eq!(stats(&parent, bear), (5, 5));

        parent.state.step = Step::End;
        parent.give_priority();
        act(&mut parent, Action::Pass);
        act(&mut parent, Action::Pass);
        assert_eq!(parent.state.active, player.other());
        assert_eq!(parent.object_view(source).name(), EQUIPMENT_NAME);
        assert_eq!(stats(&parent, bear), (5, 5));
    }
}

#[test]
fn older_grants_are_removed_later_grants_work_and_copies_use_the_base_definition() {
    let _rules = fixture_rules::activate(&[
        (card(SOURCE_NAME), &SOURCE),
        (card("Swiftfoot Boots"), &MANA_GRANT),
        (card("Think Twice"), &COPY),
        (card("Opt"), &BLANK),
        (card("Unsummon"), &BLINK),
    ]);
    let player = Player::P0;
    let mut game = scenario(player);
    let source = put(&mut game, player, SOURCE_NAME, Zone::Battlefield);
    let bear = put(&mut game, player, "Grizzly Bears", Zone::Battlefield);
    let first_provider = put(&mut game, player, "Swiftfoot Boots", Zone::Battlefield);
    let next_provider = put(&mut game, player, "Swiftfoot Boots", Zone::Hand);
    let legendary = put(&mut game, player, "Fynn, the Fangbearer", Zone::Hand);
    let copy = put(&mut game, player, "Think Twice", Zone::Hand);
    let blank = put(&mut game, player, "Opt", Zone::Hand);
    let blink = put(&mut game, player, "Unsummon", Zone::Hand);
    let exact_source = game.object_ref(source);
    game.set_counters(source, Counters::of(CounterKind::Charge, 1));
    let earlier_mana = tap_granted(&game, first_provider, source);
    assert!(game.legal_actions().contains(&earlier_mana));

    cast(&mut game, legendary, None);
    reach_optional(&mut game, player);
    act(&mut game, Action::ChooseOptional { accept: true });
    assert_eq!(game.object_ref(source), exact_source);
    assert_eq!(game.counter_count(source, CounterKind::Charge), 1);
    reject(&mut game, earlier_mana);
    reject(&mut game, tap_source(source));

    cast(&mut game, next_provider, None);
    settle(&mut game);
    let later_mana = tap_granted(&game, next_provider, source);
    let green = game.state.players[player.index()].mana[4];
    act(&mut game, later_mana);
    assert_eq!(game.state.players[player.index()].mana[4], green + 1);
    equip(&mut game, exact_source, bear);
    assert_eq!(stats(&game, bear), (5, 5));
    // The attachment link has not been removed by the next SBA pass yet.
    let mut animated = game.clone();
    animated.change_object_form(
        exact_source,
        AnimationSpec {
            artifact: false,
            creature_types: AnimationCreatureTypes::Unchanged,
            base: Some((2, 2)),
            keywords: Keywords::NONE,
            duration: AnimationDuration::EndOfTurn,
        },
        characteristic_layers::AnimationTypeMode::AddCreature,
    );
    assert!(animated.object_view(source).is_creature());
    assert_eq!(
        animated.object_view(source).attached_to,
        Some(animated.object_ref(bear))
    );
    assert_eq!(stats(&animated, bear), (2, 2));

    cast(&mut game, copy, Some(Target::Card(source)));
    settle(&mut game);
    let copied = game
        .state
        .objects
        .iter()
        .enumerate()
        .find_map(|(id, object)| {
            (object.zone == Zone::Battlefield
                && object.card == card(SOURCE_NAME)
                && id as CardId != source)
                .then(|| game.object_view(id as CardId))
        })
        .unwrap();
    assert!(copied.is_token());
    assert_eq!(copied.name(), SOURCE_NAME);
    assert_eq!(copied.kinds, 32);
    assert!(!copied.has_subtype("Equipment"));
    assert!(game.legal_actions().contains(&tap_source(copied.id)));

    let granted_equip = equip_action(&game, exact_source);
    cast(&mut game, blank, Some(Target::Card(source)));
    settle(&mut game);
    assert_eq!(game.object_view(source).name(), EQUIPMENT_NAME);
    assert_eq!(
        game.object_view(source).attached_to,
        Some(game.object_ref(bear))
    );
    assert_eq!(stats(&game, bear), (2, 2));
    reject(&mut game, granted_equip.clone());

    cast(&mut game, blink, Some(Target::Card(source)));
    settle(&mut game);
    assert_ne!(game.object_ref(source), exact_source);
    assert_eq!(game.object_view(source).name(), SOURCE_NAME);
    assert_eq!(game.object_view(source).kinds, 32);
    assert_eq!(game.object_view(source).attached_to, None);
    assert_eq!(game.counter_count(source, CounterKind::Charge), 0);
    assert!(game.legal_actions().contains(&tap_source(source)));
    reject(&mut game, granted_equip);
}

#[test]
fn an_old_trigger_cannot_offer_the_transformation_after_its_source_blinks() {
    let _rules =
        fixture_rules::activate(&[(card(SOURCE_NAME), &SOURCE), (card("Unsummon"), &BLINK)]);
    let player = Player::P0;
    let mut game = scenario(player);
    let source = put(&mut game, player, SOURCE_NAME, Zone::Battlefield);
    let legendary = put(&mut game, player, "Fynn, the Fangbearer", Zone::Hand);
    let blink = put(&mut game, player, "Unsummon", Zone::Hand);
    let exact_source = game.object_ref(source);
    cast(&mut game, legendary, None);
    for _ in 0..12 {
        if game.state.stack.iter().any(|item| {
            item.source
                .is_some_and(|source| source.object == exact_source)
        }) {
            break;
        }
        choose(&mut game, |action| {
            matches!(action, Action::Pass | Action::PutTrigger { .. })
        });
    }
    assert_eq!(game.state.stack.len(), 1);
    assert_eq!(game.state.stack[0].source.unwrap().object, exact_source);
    cast(&mut game, blink, Some(Target::Card(source)));
    settle(&mut game);
    assert_ne!(game.object_ref(source), exact_source);
    assert_eq!(game.object_view(source).name(), SOURCE_NAME);
    assert!(game.legal_actions().contains(&tap_source(source)));
    reject(&mut game, Action::ChooseOptional { accept: true });
}
