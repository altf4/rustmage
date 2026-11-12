//! Live type and combat restrictions, exact damage history, and turn callbacks.
mod support;

use rustmage as _;
use rustmage_core::cards::authored::{CardExtras, CardRules};
use rustmage_core::{cards::*, engine::*};
use support::*;

const SPIRIT_RESTRICTION: CardRules = CardRules {
    effect: Effect::Creature(1, 1),
    static_groups: &[StaticAbilityGroup {
        line: 0,
        first_layer: AbilityLayer::Rule,
        fragments: &[StaticAbilityFragment::CanOnlyBlockOrBeBlockedBySubtype(
            "Spirit",
        )],
    }],
    ..CardRules::EMPTY
};

const LESSON_LIFELINK: CardRules = CardRules {
    effect: Effect::Creature(2, 2),
    static_groups: &[StaticAbilityGroup {
        line: 0,
        first_layer: AbilityLayer::Ability,
        fragments: &[
            StaticAbilityFragment::ControlledInstantSorcerySubtypeKeywords {
                subtype: "Lesson",
                keywords: Keywords::LIFELINK,
            },
        ],
    }],
    ..CardRules::EMPTY
};

const NAME_FORM: CardRules = CardRules {
    effect: Effect::Permanent,
    static_groups: &[StaticAbilityGroup {
        line: 0,
        first_layer: AbilityLayer::Type,
        fragments: &[
            StaticAbilityFragment::SourceArtifactCreatureWhileControlNamed {
                name: "Broadcast Rambler",
                minimum: 8,
            },
        ],
    }],
    extras: &CardExtras {
        vehicle_stats: Some(Stats {
            power: 4,
            toughness: 4,
        }),
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};

const DAMAGED_EXILE: CardRules = CardRules {
    effect: Effect::Spell(
        TargetPlan::Single(TargetSpec::one(TargetRule::Creature)),
        &[
            Instruction::RememberCount(AmountValue::TargetDamagedThisTurn(0)),
            Instruction::Exile,
            Instruction::IfAmountAtLeast {
                value: AmountValue::RememberedCount,
                minimum: 1,
                skip: 1,
            },
            Instruction::CreateTokens {
                token: Card(5296),
                amount: 1,
                tapped: false,
            },
        ],
    ),
    ..CardRules::EMPTY
};

const DEATH_CALLBACK: CardRules = CardRules {
    effect: Effect::Spell(
        TargetPlan::NONE,
        &[
            Instruction::GrowPlayerCreatures {
                players: PlayerSet::Each,
                power: -2,
                toughness: -2,
            },
            Instruction::IfWaterbendPaid { skip: 1 },
            Instruction::ScheduleTurnTrigger {
                event: TurnCallbackEvent::AnyCreatureDeath,
                ability: 0,
                once: false,
            },
        ],
    ),
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::Reflexive,
        targets: TargetPlan::NONE,
        program: &[Instruction::GainLife(PlayerSet::Controller, 1)],
    }],
    extras: &CardExtras {
        waterbend_additional: Some(WaterbendAdditional::Optional(4)),
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
const DISCARD_ONE: CardRules = CardRules {
    effect: Effect::Spell(
        TargetPlan::NONE,
        &[
            Instruction::Draw(PlayerSet::Controller, 1),
            Instruction::DiscardOneNonlandReflexive(0),
        ],
    ),
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::Reflexive,
        targets: TargetPlan::Single(TargetSpec::one(TargetRule::creature_you_control())),
        program: &[Instruction::PutCounters {
            subject: Subject::Targets,
            kind: CounterKind::PlusOne,
            amount: 1,
        }],
    }],
    ..CardRules::EMPTY
};

fn declare_attacker(game: &mut Game, attacker: CardId) {
    game.state.step = Step::Attackers;
    game.state.pending = Pending::Attackers;
    act(game, Action::Attack { card: attacker });
    act(game, Action::FinishAttackers);
    act(game, Action::Pass);
    act(game, Action::Pass);
    assert_eq!(game.state.pending, Pending::Blockers);
}

#[test]
fn spirit_restriction_is_bilateral_and_tracks_live_subtypes_and_abilities() {
    let _rules = fixture_rules::activate(&[(Card::GoblinPiker, &SPIRIT_RESTRICTION)]);
    for active in [Player::P0, Player::P1] {
        let mut game = scenario(active);
        let restricted = put(&mut game, active, "Goblin Piker", Zone::Battlefield);
        let ordinary = put(&mut game, active.other(), "Hill Giant", Zone::Battlefield);
        let changeling = put(
            &mut game,
            active.other(),
            "Mutable Explorer",
            Zone::Battlefield,
        );
        let aura = put(&mut game, active, "Unable to Scream", Zone::Hand);
        let ordinary_block = Action::Block {
            blocker: ordinary,
            attacker: restricted,
        };
        let spirit_block = Action::Block {
            blocker: changeling,
            attacker: restricted,
        };

        let mut blanked = game.clone();
        cast(&mut blanked, aura, Some(Target::Card(restricted)));
        settle(&mut blanked);
        declare_attacker(&mut blanked, restricted);
        assert!(blanked.legal_actions().contains(&ordinary_block));

        declare_attacker(&mut game, restricted);
        reject(&mut game, ordinary_block);
        let checkpoint = game.snapshot();
        let mut indexed = game.clone();
        let index = game
            .legal_actions()
            .iter()
            .position(|action| *action == spirit_block)
            .unwrap();
        indexed
            .apply_index(active.other(), indexed.revision(), index)
            .unwrap();
        act(&mut game, spirit_block);
        assert_eq!(indexed.snapshot(), game.snapshot());
        indexed.restore(&checkpoint).unwrap();
        act(&mut indexed, Action::FinishBlockers);
        assert!(indexed.state.blocks.is_empty());

        for attacker_name in ["Hill Giant", "Mutable Explorer"] {
            let mut defending = scenario(active);
            let attacker = put(&mut defending, active, attacker_name, Zone::Battlefield);
            let blocker = put(
                &mut defending,
                active.other(),
                "Goblin Piker",
                Zone::Battlefield,
            );
            declare_attacker(&mut defending, attacker);
            let block = Action::Block { blocker, attacker };
            assert_eq!(
                defending.legal_actions().contains(&block),
                attacker_name == "Mutable Explorer"
            );
        }
    }
}

#[test]
fn only_live_controlled_lesson_spells_deal_damage_with_lifelink() {
    let _rules = fixture_rules::activate(&[(Card::GrizzlyBears, &LESSON_LIFELINK)]);
    for active in [Player::P0, Player::P1] {
        let mut game = scenario(active);
        let grantor = put(&mut game, active, "Grizzly Bears", Zone::Battlefield);
        let lesson = put(&mut game, active, "Firebending Lesson", Zone::Hand);
        let other_spell = put(&mut game, active, "Lightning Strike", Zone::Hand);
        let target = put(&mut game, active.other(), "Hill Giant", Zone::Battlefield);
        let life = game.state.players[active.index()].life;

        let mut ordinary = game.clone();
        cast(
            &mut ordinary,
            other_spell,
            Some(Target::Player(active.other())),
        );
        assert!(!ordinary.keywords(other_spell).contains(Keywords::LIFELINK));
        settle(&mut ordinary);
        assert_eq!(ordinary.state.players[active.index()].life, life);

        let mut foreign = game.clone();
        foreign.change_controller(foreign.object_ref(grantor), active.other());
        cast_lesson(&mut foreign, lesson, target);
        assert!(!foreign.keywords(lesson).contains(Keywords::LIFELINK));
        settle(&mut foreign);
        assert_eq!(foreign.state.players[active.index()].life, life);

        cast_lesson(&mut game, lesson, target);
        assert!(game.keywords(lesson).contains(Keywords::LIFELINK));
        let mut departed = game.clone();
        departed.move_object(grantor, Zone::Exile);
        settle(&mut departed);
        assert_eq!(departed.state.players[active.index()].life, life);

        settle(&mut game);
        assert_eq!(game.state.players[active.index()].life, life + 2);
        assert_eq!(game.object_view(target).damage, 2);
    }
}

fn cast_lesson(game: &mut Game, lesson: CardId, target: CardId) {
    act(game, Action::AnnounceCast { card: lesson });
    act(game, Action::ChooseKicker { pay: false });
    act(
        game,
        Action::ChooseTarget {
            target: Target::Card(target),
        },
    );
    pay(game);
}

#[test]
fn named_vehicle_form_counts_effective_names_and_current_control() {
    let vehicle = Card::from_name("Broadcast Rambler").unwrap();
    let _rules = fixture_rules::activate(&[(vehicle, &NAME_FORM)]);
    for active in [Player::P0, Player::P1] {
        let mut game = scenario(active);
        let source = spawn(&mut game, active, vehicle, Zone::Battlefield);
        for _ in 0..6 {
            spawn(&mut game, active, vehicle, Zone::Battlefield);
        }
        let foreign = spawn(&mut game, active.other(), vehicle, Zone::Battlefield);
        let copier = put(&mut game, active, "Hill Giant", Zone::Battlefield);
        assert_eq!(game.current_kinds(source), 16);

        game.change_controller(game.object_ref(foreign), active);
        assert_eq!(game.current_kinds(source), 17);
        assert_eq!(
            game.stats(source),
            Stats {
                power: 4,
                toughness: 4
            }
        );
        assert!(game.has_subtype(source, "Vehicle"));
        game.change_controller(game.object_ref(foreign), active.other());
        assert_eq!(game.current_kinds(source), 16);

        let checkpoint = game.snapshot();
        let recipe = game.copiable_recipe(game.object_ref(source));
        game.apply_temporary_copy(game.object_ref(copier), recipe, CopyDeadline::EndOfTurn);
        assert_eq!(game.current_kinds(source), 17);
        assert_eq!(game.current_kinds(copier), 17);
        game.finish_cleanup();
        assert_eq!(game.current_kinds(source), 16);
        assert_eq!(game.current_kinds(copier), 1);
        game.restore(&checkpoint).unwrap();
        assert_eq!(game.current_kinds(source), 16);
    }
}

fn mark_damage(game: &mut Game, source: CardId, target: CardId) {
    game.apply_damage_batch(&[DamageEvent {
        source: game.damage_source(source),
        target: Target::Card(target),
        amount: 2,
        preventable: true,
        combat: false,
    }]);
    game.advance_replacements(true);
}

fn clues(game: &Game, player: Player) -> usize {
    game.state
        .objects
        .iter()
        .filter(|object| {
            object.zone == Zone::Battlefield
                && object.controller == player
                && object.card == Card(5296)
        })
        .count()
}

#[test]
fn exile_remembers_damage_to_the_exact_current_incarnation() {
    let spell = Card::from_name("Lightning Strike").unwrap();
    let _rules = fixture_rules::activate(&[(spell, &DAMAGED_EXILE)]);
    for active in [Player::P0, Player::P1] {
        let mut game = scenario(active);
        let exile = spawn(&mut game, active, spell, Zone::Hand);
        let source = put(&mut game, active, "Bear Cub", Zone::Battlefield);
        let target = put(&mut game, active.other(), "Hill Giant", Zone::Battlefield);

        let mut prevented = game.clone();
        prevented.add_counter(target, CounterKind::Shield, 1);
        mark_damage(&mut prevented, source, target);
        assert_eq!(prevented.counter_count(target, CounterKind::Shield), 0);
        assert!(!prevented.object_view(target).damaged_this_turn);
        cast(&mut prevented, exile, Some(Target::Card(target)));
        settle(&mut prevented);
        assert_eq!(clues(&prevented, active), 0);

        mark_damage(&mut game, source, target);
        assert!(game.object_view(target).damaged_this_turn);
        game.state.objects[target as usize].damage = 0;
        let mut blinked = game.clone();
        blinked.move_object(target, Zone::Exile);
        blinked.move_object(target, Zone::Battlefield);
        cast(&mut blinked, exile, Some(Target::Card(target)));
        settle(&mut blinked);
        assert_eq!(clues(&blinked, active), 0);

        cast(&mut game, exile, Some(Target::Card(target)));
        settle(&mut game);
        assert_eq!(game.object_view(target).zone, Zone::Exile);
        assert_eq!(clues(&game, active), 1);
    }
}

fn cast_with_optional_waterbend(game: &mut Game, spell: CardId, paid: bool) {
    act(game, Action::AnnounceCast { card: spell });
    act(game, Action::ChooseWaterbendCost { pay: paid });
    pay(game);
    for _ in 0..24 {
        if game.state.pending == Pending::Priority
            && game.state.stack.is_empty()
            && game.state.pending_triggers.is_empty()
        {
            return;
        }
        choose(game, |action| {
            matches!(action, Action::Pass | Action::PutTrigger { .. })
        });
    }
    panic!("death callback resolution did not finish");
}

#[test]
fn paid_waterbend_observes_each_actual_death_until_cleanup() {
    let spell = Card::from_name("Divination").unwrap();
    let _rules = fixture_rules::activate(&[(spell, &DEATH_CALLBACK)]);
    for active in [Player::P0, Player::P1] {
        let mut game = scenario(active);
        let spell = spawn(&mut game, active, spell, Zone::Hand);
        let own = put(&mut game, active, "Bear Cub", Zone::Battlefield);
        let opposing = put(&mut game, active.other(), "Goblin Piker", Zone::Battlefield);
        let token = spawn(&mut game, active.other(), Card(5266), Zone::Battlefield);
        assert!(game.object_view(token).is_token());
        let survivor = put(&mut game, active.other(), "Hill Giant", Zone::Battlefield);
        let life = game.state.players[active.index()].life;

        let mut unpaid = game.clone();
        cast_with_optional_waterbend(&mut unpaid, spell, false);
        assert_eq!(unpaid.object_view(own).zone, Zone::Graveyard);
        assert_eq!(unpaid.object_view(opposing).zone, Zone::Graveyard);
        assert_eq!(unpaid.state.players[active.index()].life, life);

        let mut replaced = game.clone();
        put(
            &mut replaced,
            active,
            "Leyline of the Void",
            Zone::Battlefield,
        );
        cast_with_optional_waterbend(&mut replaced, spell, true);
        assert_eq!(replaced.object_view(opposing).zone, Zone::Exile);
        assert_eq!(replaced.state.players[active.index()].life, life + 2);

        cast_with_optional_waterbend(&mut game, spell, true);
        assert_eq!(game.object_view(own).zone, Zone::Graveyard);
        assert_eq!(game.object_view(opposing).zone, Zone::Graveyard);
        assert_eq!(game.state.players[active.index()].life, life + 3);
        let mut died_this_turn = game.clone();
        let removal = put(&mut died_this_turn, active, "Murder", Zone::Hand);
        cast(&mut died_this_turn, removal, Some(Target::Card(survivor)));
        settle(&mut died_this_turn);
        assert_eq!(died_this_turn.state.players[active.index()].life, life + 4);

        let mut died_during_cleanup = game.clone();
        let fragile = put(
            &mut died_during_cleanup,
            active,
            "Goblin Piker",
            Zone::Battlefield,
        );
        died_during_cleanup.state.objects[fragile as usize].growth = Stats {
            power: 1,
            toughness: 1,
        };
        died_during_cleanup.add_counter(fragile, CounterKind::MinusOne, 1);
        died_during_cleanup.state.step = Step::Cleanup;
        died_during_cleanup.finish_cleanup();
        assert_eq!(
            died_during_cleanup.object_view(fragile).zone,
            Zone::Graveyard
        );
        died_during_cleanup.prepare_triggers();
        settle(&mut died_during_cleanup);
        assert_eq!(
            died_during_cleanup.state.players[active.index()].life,
            life + 4
        );

        game.finish_cleanup();
        game.move_object(survivor, Zone::Graveyard);
        game.prepare_triggers();
        settle(&mut game);
        assert_eq!(game.state.players[active.index()].life, life + 3);
    }
}

#[test]
fn a_mandatory_discard_queues_a_target_only_after_a_nonland_choice() {
    let spell = Card::from_name("Divination").unwrap();
    let _rules = fixture_rules::activate(&[(spell, &DISCARD_ONE)]);
    for player in [Player::P0, Player::P1] {
        let mut game = scenario(player);
        let spell = spawn(&mut game, player, spell, Zone::Hand);
        let nonland = put(&mut game, player, "Grizzly Bears", Zone::Hand);
        let land = game.top_library_card(player).unwrap();
        let own = put(&mut game, player, "Bear Cub", Zone::Battlefield);
        let opposing = put(&mut game, player.other(), "Hill Giant", Zone::Battlefield);
        cast(&mut game, spell, None);
        act(&mut game, Action::Pass);
        act(&mut game, Action::Pass);
        assert_eq!(game.object_view(land).zone, Zone::Hand);
        assert!(game.observe(player.other()).actions.is_empty());
        reject(&mut game, Action::ChooseOptional { accept: false });

        let mut chose_land = game.clone();
        act(&mut chose_land, Action::Discard { card: land });
        settle(&mut chose_land);
        assert_eq!(chose_land.object_view(land).zone, Zone::Graveyard);
        assert_eq!(chose_land.counter_count(own, CounterKind::PlusOne), 0);

        let checkpoint = game.snapshot();
        let mut indexed = game.clone();
        let action = Action::Discard { card: nonland };
        let index = game
            .legal_actions()
            .iter()
            .position(|a| *a == action)
            .unwrap();
        indexed
            .apply_index(player, indexed.revision(), index)
            .unwrap();
        act(&mut game, action);
        assert_eq!(indexed.snapshot(), game.snapshot());
        assert!(matches!(game.state.pending, Pending::Targets { .. }));
        reject(
            &mut game,
            Action::ChooseTarget {
                target: Target::Card(opposing),
            },
        );
        act(
            &mut game,
            Action::ChooseTarget {
                target: Target::Card(own),
            },
        );
        settle(&mut game);
        assert_eq!(game.counter_count(own, CounterKind::PlusOne), 1);
        indexed.restore(&checkpoint).unwrap();
        assert!(
            indexed
                .legal_actions()
                .contains(&Action::Discard { card: land })
        );
    }
}
