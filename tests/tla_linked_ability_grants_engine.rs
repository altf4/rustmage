//! Exact linked donors supply separately owned activated and triggered instances.
mod support;

use rustmage as _;
use rustmage_core::cards::authored::CardRules;
use rustmage_core::{cards::*, engine::*};
use support::*;

const CHOOSER: CardRules = CardRules {
    effect: Effect::Creature(6, 6),
    static_groups: &[StaticAbilityGroup {
        line: 3,
        first_layer: AbilityLayer::Ability,
        fragments: &[StaticAbilityFragment::GrantChosenLinkedCreatureAbilities],
    }],
    triggers: &[
        TriggeredAbility {
            condition: TriggerCondition::SelfEnters,
            targets: TargetPlan::Single(TargetSpec {
                rule: TargetRule::Filtered {
                    base: &TargetRule::Creature,
                    predicate: &TargetPredicate::OtherThanSource,
                },
                min: 0,
                max: 1,
            }),
            program: &[Instruction::ExileTargetsLinkedToSource],
        },
        TriggeredAbility {
            condition: TriggerCondition::CreatureDies {
                relation: CastRelation::Any,
                other: true,
                nontoken: true,
                subtype: None,
            },
            targets: TargetPlan::NONE,
            program: &[
                Instruction::May { skip: 1 },
                Instruction::ExileEventGraveyardCardLinked,
            ],
        },
    ],
    activated: &[ActivatedAbility {
        cost: ActivationCost {
            life: 1,
            ..ActivationCost::ZERO
        },
        program: &[Instruction::ChooseLinkedCreatureAbilities],
        ..GAIN_ONE
    }],
    ..CardRules::EMPTY
};

const GAIN_ONE: ActivatedAbility = ActivatedAbility {
    exhaust: false,
    guard: ActivationGuard::Always,
    origin: ActivationOrigin::Battlefield,
    cycling: false,
    cost: ActivationCost::ZERO,
    targets: TargetPlan::NONE,
    program: &[Instruction::GainLife(PlayerSet::Controller, 1)],
    sorcery: false,
    once_per_turn: false,
};
const OWN_CAST: TriggerCondition = TriggerCondition::SpellCast {
    relation: CastRelation::You,
    filter: CastFilter::Any,
    ordinal: None,
};
const FIRST: CardRules = CardRules {
    effect: Effect::Creature(2, 2),
    keywords: Keywords::FLYING,
    triggers: &[
        TriggeredAbility {
            condition: TriggerCondition::OncePerTurn(&OWN_CAST),
            targets: TargetPlan::NONE,
            program: &[Instruction::GainLife(PlayerSet::Controller, 2)],
        },
        TriggeredAbility {
            condition: TriggerCondition::SelfDies,
            targets: TargetPlan::NONE,
            program: &[Instruction::Draw(PlayerSet::Controller, 1)],
        },
        TriggeredAbility {
            condition: TriggerCondition::Ward,
            targets: TargetPlan::NONE,
            program: &[Instruction::CounterRelatedUnlessPayLife(2)],
        },
    ],
    activated: &[GAIN_ONE],
    ..CardRules::EMPTY
};
const SECOND: CardRules = CardRules {
    effect: Effect::Creature(2, 2),
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::OncePerTurn(&OWN_CAST),
        targets: TargetPlan::NONE,
        program: &[Instruction::GainLife(PlayerSet::Controller, 3)],
    }],
    activated: &[ActivatedAbility {
        program: &[Instruction::GainLife(PlayerSet::Controller, 4)],
        ..GAIN_ONE
    }],
    ..CardRules::EMPTY
};
const LINKED_DONOR: CardRules = CardRules {
    effect: Effect::Creature(2, 2),
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::AtStep {
            step: TriggerStep::End,
            relation: CastRelation::You,
            guard: StepGuard::Always,
        },
        targets: TargetPlan::NONE,
        program: &[Instruction::ChooseLinkedCreatureReturn],
    }],
    activated: &[ActivatedAbility {
        targets: TargetPlan::Single(TargetSpec::one(TargetRule::Creature)),
        program: &[Instruction::ExileTargetsLinkedToSource],
        ..GAIN_ONE
    }],
    ..CardRules::EMPTY
};

fn wait_until(game: &mut Game, ready: impl Fn(&Game) -> bool) {
    for _ in 0..48 {
        if ready(game) {
            return;
        }
        choose(game, |action| {
            matches!(action, Action::Pass | Action::PutTrigger { .. })
        });
    }
    panic!("linked grant did not reach the expected choice");
}

fn enter_and_exile(game: &mut Game, host: CardId, target: CardId) {
    cast(game, host, None);
    wait_until(game, |g| matches!(g.state.pending, Pending::Targets { .. }));
    reject(
        game,
        Action::ChooseTarget {
            target: Target::Card(host),
        },
    );
    act(
        game,
        Action::ChooseTarget {
            target: Target::Card(target),
        },
    );
    if game.legal_actions().contains(&Action::FinishTargets) {
        act(game, Action::FinishTargets);
    }
    settle(game);
    assert_eq!(game.object_view(target).zone, Zone::Exile);
}

fn begin_choose(game: &mut Game, host: CardId) {
    act(
        game,
        Action::AnnounceActivate {
            card: host,
            ability: 0,
            counter_payment: Counters::default(),
        },
    );
    pay(game);
    wait_until(game, |g| {
        matches!(g.state.pending, Pending::LinkedExile { .. })
    });
}

fn select(game: &mut Game, donor: CardId) {
    let object = game.object_ref(donor);
    act(game, Action::ChooseLinkedExile { object });
}

fn borrowed_action(game: &Game, host: CardId, donor: CardId) -> Action {
    Action::AnnounceBorrowedAbility {
        card: host,
        exiled: game.object_ref(donor),
        ability: 0,
        counter_payment: Counters::default(),
    }
}

#[test]
fn replacing_a_linked_donor_preserves_its_queued_trigger_and_changes_future_actions() {
    let _rules = fixture_rules::activate(&[
        (Card::HillGiant, &CHOOSER),
        (Card::GrizzlyBears, &FIRST),
        (Card::GoblinPiker, &SECOND),
    ]);
    for player in [Player::P0, Player::P1] {
        let mut game = scenario(player);
        let host = put(&mut game, player, "Hill Giant", Zone::Hand);
        let first = put(&mut game, player, "Grizzly Bears", Zone::Battlefield);
        let second = put(&mut game, player.other(), "Goblin Piker", Zone::Battlefield);
        let unlinked = put(&mut game, player, "Bear Cub", Zone::Exile);
        enter_and_exile(&mut game, host, first);

        let kill = put(&mut game, player, "Murder", Zone::Hand);
        cast(&mut game, kill, Some(Target::Card(second)));
        let accept = Action::ChooseOptional { accept: true };
        wait_until(&mut game, |g| g.legal_actions().contains(&accept));
        let mut declined = game.clone();
        act(&mut declined, Action::ChooseOptional { accept: false });
        settle(&mut declined);
        assert_eq!(declined.object_view(second).zone, Zone::Graveyard);
        act(&mut game, accept);
        settle(&mut game);
        assert_eq!(game.object_view(second).zone, Zone::Exile);

        begin_choose(&mut game, host);
        let unlinked = game.object_ref(unlinked);
        reject(&mut game, Action::ChooseLinkedExile { object: unlinked });
        let checkpoint = game.snapshot();
        let mut indexed = game.clone();
        let action = Action::ChooseLinkedExile {
            object: game.object_ref(first),
        };
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
        indexed.restore(&checkpoint).unwrap();
        select(&mut indexed, second);
        assert_eq!(
            indexed
                .object_view(host)
                .acquired_triggers
                .unwrap()
                .exiled
                .card,
            second
        );

        let first_grant = game.object_view(host).acquired_triggers.unwrap();
        assert_eq!(first_grant.exiled, game.object_ref(first));
        assert!(!game.keywords(host).contains(Keywords::FLYING));
        assert!(
            game.legal_actions()
                .contains(&borrowed_action(&game, host, first))
        );
        assert!(
            !game
                .legal_actions()
                .contains(&borrowed_action(&game, host, second))
        );

        let draw = put(&mut game, player, "Quick Study", Zone::Hand);
        cast(&mut game, draw, None);
        let trigger = game.state.stack.last().unwrap();
        assert_eq!(trigger.acquired_link_cohort, Some(first_grant.cohort));
        assert_eq!(
            trigger.acquired_trigger_definition,
            Some(Card::GrizzlyBears)
        );
        assert_eq!(trigger.source().unwrap().object, game.object_ref(host));
        assert_eq!(trigger.source().unwrap().card, Card::HillGiant);

        begin_choose(&mut game, host);
        select(&mut game, second);
        settle(&mut game);
        assert_eq!(game.state.players[player.index()].life, 20);
        assert!(
            !game
                .legal_actions()
                .contains(&borrowed_action(&game, host, first))
        );
        let activation = borrowed_action(&game, host, second);
        assert!(game.legal_actions().contains(&activation));
        let draw = put(&mut game, player, "Quick Study", Zone::Hand);
        cast(&mut game, draw, None);
        settle(&mut game);
        assert_eq!(game.state.players[player.index()].life, 23);

        act(&mut game, activation);
        pay(&mut game);
        game.move_object(second, Zone::Hand);
        game.move_object(second, Zone::Exile);
        settle(&mut game);
        assert_eq!(game.state.players[player.index()].life, 27);
        assert!(game.object_view(host).acquired_triggers.is_none());
        let stale = borrowed_action(&game, host, second);
        reject(&mut game, stale);
    }
}

#[test]
fn acquired_ward_and_death_templates_use_the_live_or_last_known_recipient() {
    let _rules =
        fixture_rules::activate(&[(Card::HillGiant, &CHOOSER), (Card::GrizzlyBears, &FIRST)]);
    let player = Player::P0;
    let mut game = scenario(player);
    let host = put(&mut game, player, "Hill Giant", Zone::Hand);
    let donor = put(&mut game, player, "Grizzly Bears", Zone::Battlefield);
    enter_and_exile(&mut game, host, donor);
    begin_choose(&mut game, host);
    select(&mut game, donor);

    let mut blanked = game.clone();
    let aura = put(&mut blanked, player, "Unable to Scream", Zone::Hand);
    cast(&mut blanked, aura, Some(Target::Card(host)));
    settle(&mut blanked);
    assert!(blanked.object_view(host).acquired_triggers.is_none());
    blanked.move_object(aura, Zone::Graveyard);
    assert!(blanked.object_view(host).acquired_triggers.is_some());

    let mut warded = game.clone();
    act(&mut warded, Action::Pass);
    let hostile = put(&mut warded, player.other(), "Murder", Zone::Hand);
    cast(&mut warded, hostile, Some(Target::Card(host)));
    let ward = warded.state.stack.last().unwrap();
    assert_eq!(ward.acquired_trigger_definition, Some(Card::GrizzlyBears));
    assert_eq!(ward.source().unwrap().object, warded.object_ref(host));
    let decline = Action::ChooseOptional { accept: false };
    wait_until(&mut warded, |g| g.legal_actions().contains(&decline));
    act(&mut warded, decline);
    settle(&mut warded);
    assert_eq!(warded.object_view(host).zone, Zone::Battlefield);

    let before = game.object_ref(host);
    game.move_object(host, Zone::Graveyard);
    game.prepare_triggers();
    let death = game.state.stack.last().unwrap();
    assert_eq!(death.acquired_trigger_definition, Some(Card::GrizzlyBears));
    assert_eq!(death.source().unwrap().object, before);
    assert_ne!(game.object_ref(host), before);
    settle(&mut game);
    assert_eq!(game.state.players[player.index()].cards_drawn_this_turn, 1);
    game.move_object(host, Zone::Battlefield);
    assert!(game.object_view(host).acquired_triggers.is_none());
}

#[test]
fn acquired_activated_and_triggered_templates_share_their_own_links() {
    let _rules = fixture_rules::activate(&[
        (Card::HillGiant, &CHOOSER),
        (Card::GoblinPiker, &LINKED_DONOR),
    ]);
    let player = Player::P0;
    let mut game = scenario(player);
    let host = put(&mut game, player, "Hill Giant", Zone::Hand);
    let donor = put(&mut game, player, "Goblin Piker", Zone::Battlefield);
    let victim = put(&mut game, player.other(), "Bear Cub", Zone::Battlefield);
    enter_and_exile(&mut game, host, donor);
    begin_choose(&mut game, host);
    select(&mut game, donor);
    let before = game.object_view(host).acquired_triggers.unwrap();

    let write = borrowed_action(&game, host, donor);
    act(&mut game, write);
    act(
        &mut game,
        Action::ChooseTarget {
            target: Target::Card(victim),
        },
    );
    pay(&mut game);
    settle(&mut game);
    assert_eq!(game.object_view(victim).zone, Zone::Exile);

    begin_choose(&mut game, host);
    let foreign_link = Action::ChooseLinkedExile {
        object: game.object_ref(victim),
    };
    reject(&mut game, foreign_link);
    select(&mut game, donor);
    assert_eq!(game.object_view(host).acquired_triggers.unwrap(), before);

    game.begin_step(Step::End);
    game.prepare_triggers();
    wait_until(&mut game, |g| {
        matches!(g.state.pending, Pending::LinkedExile { .. })
    });
    let native_link = Action::ChooseLinkedExile {
        object: game.object_ref(donor),
    };
    reject(&mut game, native_link);
    select(&mut game, victim);
    assert_eq!(game.object_view(victim).zone, Zone::Battlefield);
    assert_eq!(game.object_view(victim).controller, player);
}

#[test]
fn decayed_supplies_only_its_attack_trigger_and_excludes_graveyard_activations() {
    let _rules = fixture_rules::activate(&[(Card::HillGiant, &CHOOSER)]);
    let player = Player::P0;
    let mut game = scenario(player);
    let host = put(&mut game, player, "Hill Giant", Zone::Hand);
    let donor = put(&mut game, player, "Rot-Curse Rakshasa", Zone::Battlefield);
    let attacker = put(&mut game, player.other(), "Bear Cub", Zone::Battlefield);
    enter_and_exile(&mut game, host, donor);
    begin_choose(&mut game, host);
    select(&mut game, donor);
    assert!(
        !game
            .keywords(host)
            .intersects(Keywords::DECAYED.union(Keywords::TRAMPLE))
    );
    assert!(
        !game
            .legal_actions()
            .contains(&borrowed_action(&game, host, donor))
    );

    let mut blocking = game.clone();
    blocking.state.active = player.other();
    blocking.state.step = Step::Attackers;
    blocking.state.pending = Pending::Attackers;
    act(&mut blocking, Action::Attack { card: attacker });
    act(&mut blocking, Action::FinishAttackers);
    act(&mut blocking, Action::Pass);
    act(&mut blocking, Action::Pass);
    assert!(blocking.legal_actions().contains(&Action::Block {
        blocker: host,
        attacker
    }));

    game.state.objects[host as usize].summoning_sick = false;
    game.state.step = Step::Attackers;
    game.state.pending = Pending::Attackers;
    act(&mut game, Action::Attack { card: host });
    act(&mut game, Action::FinishAttackers);
    assert_eq!(
        game.state.stack.last().unwrap().kind,
        StackKind::DecayedAttack
    );
    settle(&mut game);
    assert_eq!(game.object_view(host).zone, Zone::Battlefield);
}
