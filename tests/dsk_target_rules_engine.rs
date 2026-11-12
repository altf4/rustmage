//! Rules for ignoring creature hexproof and ward, and for one distinct target.
mod support;
use rustmage as _;
use rustmage_core::cards::authored::CardRules;
use rustmage_core::{cards::*, engine::*};
use support::*;

const BYPASS: CardRules = CardRules {
    effect: Effect::Permanent,
    static_groups: &[StaticAbilityGroup {
        line: 0,
        first_layer: AbilityLayer::Rule,
        fragments: &[StaticAbilityFragment::OpposingCreatureHexproofAndWardDisabled],
    }],
    ..CardRules::EMPTY
};
const HEXPROOF: CardRules = CardRules {
    effect: Effect::Creature(2, 3),
    keywords: Keywords::HEXPROOF,
    ..CardRules::EMPTY
};
const QUALIFIED: CardRules = CardRules {
    effect: Effect::Creature(3, 3),
    static_groups: &[StaticAbilityGroup {
        line: 0,
        first_layer: AbilityLayer::Ability,
        fragments: &[StaticAbilityFragment::QualifiedHexproof(
            SourceCharacteristicFilter::Instant,
        )],
    }],
    ..CardRules::EMPTY
};
const PROTECTED: CardRules = CardRules {
    keywords: Keywords::HEXPROOF.union(Keywords::PROTECTION_INSTANT_SORCERY),
    ..HEXPROOF
};
const NONCREATURE: CardRules = CardRules {
    effect: Effect::Permanent,
    keywords: Keywords::HEXPROOF,
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::Ward,
        targets: TargetPlan::NONE,
        program: &[Instruction::CounterRelatedUnlessPayLife(2)],
    }],
    ..CardRules::EMPTY
};
const ANY_PERMANENT: TargetRule = TargetRule::Object {
    zone: TargetZone::Battlefield,
    kinds: 0,
    relation: -1,
    other: false,
    nonland: false,
    tapped: false,
    min_mana: 0,
    max_mana: u32::MAX,
};
const TARGET_PERMANENT: CardRules = CardRules {
    effect: Effect::Spell(
        TargetPlan::Single(TargetSpec::one(ANY_PERMANENT)),
        &[Instruction::Draw(PlayerSet::Controller, 1)],
    ),
    ..CardRules::EMPTY
};

fn card(name: &str) -> Card {
    Card::from_name(name).unwrap()
}

fn triggers_on(game: &Game, source: CardId) -> Vec<&SpellView> {
    game.state
        .stack
        .iter()
        .chain(&game.state.pending_triggers)
        .chain(&game.state.ordering_triggers)
        .filter(|item| {
            matches!(item.kind, StackKind::TriggeredAbility { .. })
                && item.source.is_some_and(|s| s.id == source)
        })
        .collect()
}

#[test]
fn live_controller_can_ignore_creature_hexproof_and_rechecks_on_resolution() {
    let bypass = card("Banishing Light");
    let qualified = card("Hill Giant");
    let protected = card("Gigantosaurus");
    let noncreature = card("Candy Trail");
    let spell = card("Opt");
    let _rules = fixture_rules::activate(&[
        (bypass, &BYPASS),
        (Card::GrizzlyBears, &HEXPROOF),
        (qualified, &QUALIFIED),
        (protected, &PROTECTED),
        (noncreature, &NONCREATURE),
        (spell, &TARGET_PERMANENT),
    ]);
    let player = Player::P0;
    let mut game = scenario(player);
    let source = spawn(&mut game, player, bypass, Zone::Battlefield);
    let bare = spawn(
        &mut game,
        player.other(),
        Card::GrizzlyBears,
        Zone::Battlefield,
    );
    let instant_only = spawn(&mut game, player.other(), qualified, Zone::Battlefield);
    let still_protected = spawn(&mut game, player.other(), protected, Zone::Battlefield);
    let artifact = spawn(&mut game, player.other(), noncreature, Zone::Battlefield);
    let cast_id = spawn(&mut game, player, spell, Zone::Hand);
    let source_ref = game.object_ref(source);

    assert!(game.object_view(bare).keywords.contains(Keywords::HEXPROOF));
    for disable in 0..3 {
        let mut branch = game.clone();
        if disable == 1 {
            branch.change_controller(source_ref, player.other());
        } else if disable == 2 {
            branch.remove_abilities_exact(
                &[source_ref],
                source_ref,
                player,
                AnimationDuration::Permanent,
            );
        }
        act(&mut branch, Action::AnnounceCast { card: cast_id });
        for (id, offered) in [
            (bare, disable == 0),
            (instant_only, disable == 0),
            (still_protected, false),
            (artifact, false),
        ] {
            let action = Action::ChooseTarget {
                target: Target::Card(id),
            };
            assert_eq!(branch.legal_actions().contains(&action), offered);
            if !offered {
                reject(&mut branch, action);
            }
        }
    }
    cast(&mut game, cast_id, Some(Target::Card(instant_only)));
    game.move_object(source, Zone::Graveyard);
    settle(&mut game);
    assert_eq!(game.object_view(cast_id).zone, Zone::Graveyard);
    assert_eq!(game.state.players[player.index()].cards_drawn_this_turn, 0);
}

const WARD: TriggeredAbility = TriggeredAbility {
    condition: TriggerCondition::Ward,
    targets: TargetPlan::NONE,
    program: &[Instruction::CounterRelatedUnlessPayLife(2)],
};
const WARD_AND_TARGETED: CardRules = CardRules {
    effect: Effect::Creature(2, 4),
    triggers: &[
        WARD,
        TriggeredAbility {
            condition: TriggerCondition::SelfTargeted {
                relation: CastRelation::Opponent,
                spells_only: true,
            },
            targets: TargetPlan::NONE,
            program: &[Instruction::Draw(PlayerSet::Controller, 1)],
        },
    ],
    ..CardRules::EMPTY
};
const WARD_GRANT: CardRules = CardRules {
    effect: Effect::Creature(1, 1),
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::Granted(&TriggerCondition::Ward),
        ..WARD
    }],
    static_groups: &[StaticAbilityGroup {
        line: 0,
        first_layer: AbilityLayer::Ability,
        fragments: &[StaticAbilityFragment::ControlledCreatureTrigger {
            ability: 0,
            other: true,
            nontoken: false,
        }],
    }],
    ..CardRules::EMPTY
};

#[test]
fn suppresses_printed_and_granted_ward_only_at_the_targeting_event() {
    let bypass = card("Banishing Light");
    let granter = card("Llanowar Elves");
    let _rules = fixture_rules::activate(&[
        (bypass, &BYPASS),
        (Card::GrizzlyBears, &WARD_AND_TARGETED),
        (granter, &WARD_GRANT),
    ]);
    let player = Player::P0;
    let enemy = player.other();
    let mut game = scenario(player);
    let source = spawn(&mut game, player, bypass, Zone::Battlefield);
    let recipient = spawn(&mut game, enemy, Card::GrizzlyBears, Zone::Battlefield);
    spawn(&mut game, enemy, granter, Zone::Battlefield);
    let aura = put(&mut game, enemy, "Sheltered by Ghosts", Zone::Battlefield);
    game.state.objects[aura as usize].attached_to = Some(game.object_ref(recipient));
    let spell = put(&mut game, player, "Shock", Zone::Hand);
    let mut ordinary = game.clone();
    ordinary.move_object(source, Zone::Graveyard);
    cast(&mut ordinary, spell, Some(Target::Card(recipient)));
    assert_eq!(triggers_on(&ordinary, recipient).len(), 4);

    cast(&mut game, spell, Some(Target::Card(recipient)));
    let triggers = triggers_on(&game, recipient);
    assert_eq!(triggers.len(), 1);
    assert_eq!(triggers[0].kind, StackKind::TriggeredAbility { ability: 1 });
    game.move_object(source, Zone::Graveyard);
    settle(&mut game);
    assert_eq!(game.object_view(recipient).damage, 2);
    assert_eq!(game.state.players[enemy.index()].cards_drawn_this_turn, 1);
}

#[test]
fn noncreature_ward_is_unchanged() {
    let bypass = card("Banishing Light");
    let artifact = card("Candy Trail");
    let spell = card("Opt");
    let _rules = fixture_rules::activate(&[
        (bypass, &BYPASS),
        (
            artifact,
            &CardRules {
                keywords: Keywords::NONE,
                ..NONCREATURE
            },
        ),
        (spell, &TARGET_PERMANENT),
    ]);
    let player = Player::P0;
    let mut game = scenario(player);
    spawn(&mut game, player, bypass, Zone::Battlefield);
    let recipient = spawn(&mut game, player.other(), artifact, Zone::Battlefield);
    let cast_id = spawn(&mut game, player, spell, Zone::Hand);
    cast(&mut game, cast_id, Some(Target::Card(recipient)));
    assert_eq!(triggers_on(&game, recipient).len(), 1);
}

const COPY_OBSERVER: CardRules = CardRules {
    effect: Effect::Permanent,
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::SpellCast {
            relation: CastRelation::You,
            filter: CastFilter::OnlyTargeted {
                filter: &CastFilter::InstantSorcery,
                target: CastTarget::Creature(CastRelation::You),
            },
            ordinal: None,
        },
        targets: TargetPlan::NONE,
        program: &[Instruction::CopyCastSpell {
            prior_spells: false,
        }],
    }],
    ..CardRules::EMPTY
};
const TWO_TARGET_GROUPS: CardRules = CardRules {
    effect: Effect::Spell(
        TargetPlan::Groups(&[
            TargetSpec::one(TargetRule::Creature),
            TargetSpec::one(TargetRule::Any),
        ]),
        &[Instruction::Draw(PlayerSet::Controller, 1)],
    ),
    ..CardRules::EMPTY
};

#[test]
fn two_target_groups_can_name_one_creature_and_retarget_the_copy_independently() {
    let observer = card("Banishing Light");
    let spell = card("Opt");
    let _rules =
        fixture_rules::activate(&[(observer, &COPY_OBSERVER), (spell, &TWO_TARGET_GROUPS)]);
    let player = Player::P0;
    let mut game = scenario(player);
    let source = spawn(&mut game, player, observer, Zone::Battlefield);
    let first = put(&mut game, player, "Grizzly Bears", Zone::Battlefield);
    let second = put(&mut game, player, "Hill Giant", Zone::Battlefield);
    let enemy = put(
        &mut game,
        player.other(),
        "Grizzly Bears",
        Zone::Battlefield,
    );
    let cast_id = spawn(&mut game, player, spell, Zone::Hand);

    for (targets, expected) in [
        ([Target::Card(first), Target::Card(first)], 1),
        ([Target::Card(first), Target::Card(second)], 0),
        ([Target::Card(first), Target::Player(player)], 0),
        ([Target::Card(enemy), Target::Card(enemy)], 0),
    ] {
        let mut branch = game.clone();
        announce(&mut branch, cast_id, &targets);
        assert_eq!(triggers_on(&branch, source).len(), expected);
    }

    announce(
        &mut game,
        cast_id,
        &[Target::Card(first), Target::Card(first)],
    );
    act(&mut game, Action::Pass);
    act(&mut game, Action::Pass);
    let copy = game.retarget_view().unwrap().item;
    assert!(copy.is_copy);
    assert_eq!(copy.targets.len(), 2);
    assert_eq!(copy.targets[0].object, copy.targets[1].object);
    assert_ne!(copy.targets[0].group, copy.targets[1].group);
    let retarget = Action::SetStackTarget {
        index: 1,
        target: Target::Card(second),
    };
    let before = game.snapshot();
    let mut indexed = game.clone();
    let index = indexed
        .legal_actions()
        .iter()
        .position(|a| a == &retarget)
        .unwrap();
    indexed
        .apply_index(player, indexed.revision(), index)
        .unwrap();
    act(&mut game, retarget.clone());
    assert_eq!(game.snapshot(), indexed.snapshot());
    let targets = &game.retarget_view().unwrap().item.targets;
    assert_eq!(targets[0].target, Target::Card(first));
    assert_eq!(targets[1].target, Target::Card(second));
    assert_eq!(game.state.stack[0].targets[1].target, Target::Card(first));

    game.restore(&before).unwrap();
    act(&mut game, retarget);
    assert_eq!(game.snapshot(), indexed.snapshot());
    act(&mut game, Action::FinishStackTargets);
    settle(&mut game);
    assert_eq!(game.state.players[player.index()].cards_drawn_this_turn, 2);
    assert_eq!(
        game.state.players[player.index()].instant_sorcery_cast_this_turn,
        1
    );
}
