//! Exact Saddle cost history and designation across owned games.
mod support;
use rustmage::Card;
use rustmage_core::cards::authored::CardRules;
use rustmage_core::cards::*;
use rustmage_core::engine::*;
use support::*;

const MOUNT: CardRules = CardRules {
    effect: Effect::Creature(2, 2),
    activated: &[ActivatedAbility {
        exhaust: false,
        guard: ActivationGuard::Always,
        cycling: false,
        origin: ActivationOrigin::Battlefield,
        cost: ActivationCost::ZERO,
        targets: TargetPlan::NONE,
        program: &[Instruction::BecomeSaddled { minimum: 2 }],
        sorcery: true,
        once_per_turn: false,
    }],
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::SelfAttacks,
        targets: TargetPlan::NONE,
        program: &[
            Instruction::If {
                guard: StepGuard::SourceSaddled,
                skip: 2,
            },
            Instruction::GainLife(PlayerSet::Controller, 3),
            Instruction::Skip(1),
            Instruction::LoseLife(PlayerSet::Controller, 4),
        ],
    }],
    ..CardRules::EMPTY
};
const COUNTER_ABILITY: CardRules = CardRules {
    effect: Effect::Spell(
        TargetPlan::Single(TargetSpec::one(TargetRule::Ability)),
        &[Instruction::Counter],
    ),
    ..CardRules::EMPTY
};
const DESIGNATE: CardRules = CardRules {
    effect: Effect::Spell(
        TargetPlan::Single(TargetSpec::one(TargetRule::Creature)),
        &[Instruction::MarkSaddled(Subject::Targets)],
    ),
    ..CardRules::EMPTY
};

fn start_saddle(g: &mut Game, mount: CardId) {
    act(
        g,
        Action::AnnounceActivate {
            card: mount,
            ability: 0,
            counter_payment: Counters::default(),
        },
    );
    assert!(matches!(g.state.pending, Pending::Saddle { .. }));
}

fn choose_rider(g: &mut Game, object: ObjectRef) {
    act(g, Action::ChooseSaddler { object });
}

fn saddle(g: &mut Game, mount: CardId, rider: ObjectRef) {
    start_saddle(g, mount);
    choose_rider(g, rider);
    act(g, Action::FinishSaddle);
}

#[test]
fn paid_saddle_records_exact_riders_even_if_countered_and_designation_is_separate() {
    let _rules = fixture_rules::activate(&[
        (Card::GrizzlyBears, &MOUNT),
        (Card::Counterspell, &COUNTER_ABILITY),
        (Card::GiantGrowth, &DESIGNATE),
    ]);
    for player in [Player::P0, Player::P1] {
        let mut g = scenario(player);
        let mount = put(&mut g, player, "Grizzly Bears", Zone::Battlefield);
        let first = put(&mut g, player, "Goblin Piker", Zone::Battlefield);
        let second = put(&mut g, player, "Goblin Piker", Zone::Battlefield);
        let exact_mount = g.object_ref(mount);
        let first_rider = g.object_ref(first);
        let second_rider = g.object_ref(second);

        start_saddle(&mut g, mount);
        reject(&mut g, Action::FinishSaddle);
        choose_rider(&mut g, first_rider);
        assert!(g.saddle_contributors(exact_mount).is_empty());
        act(&mut g, Action::FinishSaddle);
        assert_eq!(g.saddle_contributors(exact_mount), &[first_rider]);
        assert!(g.object_view(first).tapped);
        assert!(!g.object_view(mount).saddled);

        let activation = g.state.stack.last().unwrap().id;
        act(&mut g, Action::Pass);
        let counter = put(&mut g, player.other(), "Counterspell", Zone::Hand);
        cast(&mut g, counter, Some(Target::Stack(activation)));
        settle(&mut g);
        assert!(!g.object_view(mount).saddled);
        assert_eq!(g.saddle_contributors(exact_mount), &[first_rider]);

        // A direct designation can resolve without a rider. A later payment
        // on an already saddled mount still records its own contributor.
        let designate = put(&mut g, player, "Giant Growth", Zone::Hand);
        cast(&mut g, designate, Some(Target::Card(mount)));
        settle(&mut g);
        assert!(g.object_view(mount).saddled);
        assert_eq!(g.saddle_contributors(exact_mount), &[first_rider]);

        saddle(&mut g, mount, second_rider);
        settle(&mut g);
        assert_eq!(
            g.saddle_contributors(exact_mount),
            &[first_rider, second_rider]
        );

        // An untapped rider can pay again, without being counted as a new rider.
        g.state.objects[first as usize].tapped = false;
        saddle(&mut g, mount, first_rider);
        settle(&mut g);
        assert_eq!(
            g.saddle_contributors(exact_mount),
            &[first_rider, second_rider]
        );

        g.finish_cleanup();
        assert!(!g.source_saddled(exact_mount));
        assert!(g.saddle_contributors(exact_mount).is_empty());
    }
}

#[test]
fn saddle_payment_branches_restore_and_keep_departed_incarnation_links() {
    let _rules = fixture_rules::activate(&[(Card::GrizzlyBears, &MOUNT)]);
    let player = Player::P0;
    let mut root = scenario(player);
    let mount = put(&mut root, player, "Grizzly Bears", Zone::Battlefield);
    let first = put(&mut root, player, "Goblin Piker", Zone::Battlefield);
    let second = put(&mut root, player, "Goblin Piker", Zone::Battlefield);
    let mount_ref = root.object_ref(mount);
    let first_ref = root.object_ref(first);
    let second_ref = root.object_ref(second);
    start_saddle(&mut root, mount);

    let saved = root.snapshot();
    let mut raw = root.clone();
    let mut indexed = root.clone();
    let mut restored = root.clone();
    drop(root);
    assert!(raw.observe(player.other()).saddle_choice.is_none());

    choose_rider(&mut raw, first_ref);
    reject(&mut raw, Action::ChooseSaddler { object: first_ref });
    act(&mut raw, Action::FinishSaddle);
    settle(&mut raw);
    assert_eq!(raw.saddle_contributors(mount_ref), &[first_ref]);

    let other_action = Action::ChooseSaddler { object: second_ref };
    let index = indexed
        .legal_actions()
        .iter()
        .position(|action| *action == other_action)
        .unwrap();
    indexed
        .apply_index(player, indexed.revision(), index)
        .unwrap();
    act(&mut indexed, Action::FinishSaddle);
    settle(&mut indexed);
    assert_eq!(indexed.saddle_contributors(mount_ref), &[second_ref]);
    assert!(!indexed.object_view(first).tapped);

    restored.restore(&saved).unwrap();
    choose_rider(&mut restored, first_ref);
    act(&mut restored, Action::FinishSaddle);
    settle(&mut restored);
    assert_eq!(restored.snapshot(), raw.snapshot());

    raw.move_object(mount, Zone::Hand);
    raw.move_object(mount, Zone::Battlefield);
    raw.move_object(first, Zone::Hand);
    raw.move_object(first, Zone::Battlefield);
    assert_ne!(raw.object_ref(first), first_ref);
    assert_ne!(raw.object_ref(mount), mount_ref);
    assert!(raw.source_saddled(mount_ref));
    assert_eq!(raw.saddle_contributors(mount_ref), &[first_ref]);
    assert!(!raw.object_view(mount).saddled);
    assert!(raw.saddle_contributors(raw.object_ref(mount)).is_empty());
}

#[test]
fn resolution_reads_live_saddle_or_the_departed_sources_designation() {
    let _rules = fixture_rules::activate(&[
        (Card::GrizzlyBears, &MOUNT),
        (Card::GiantGrowth, &DESIGNATE),
    ]);
    for (saddled_before_attack, remove_mount) in [(true, true), (false, true), (false, false)] {
        let player = Player::P0;
        let mut g = scenario(player);
        let mount = put(&mut g, player, "Grizzly Bears", Zone::Battlefield);
        let rider = put(&mut g, player, "Goblin Piker", Zone::Battlefield);
        let rider_ref = g.object_ref(rider);
        if saddled_before_attack {
            saddle(&mut g, mount, rider_ref);
            settle(&mut g);
        }
        for _ in 0..4 {
            act(&mut g, Action::Pass);
        }
        assert_eq!(g.state.pending, Pending::Attackers);
        act(&mut g, Action::Attack { card: mount });
        act(&mut g, Action::FinishAttackers);
        assert!(!g.state.stack.is_empty());

        let life = g.state.players[player.index()].life;
        if remove_mount {
            g.move_object(mount, Zone::Hand);
            g.move_object(mount, Zone::Battlefield);
            assert!(!g.object_view(mount).saddled);
        } else {
            let designate = put(&mut g, player, "Giant Growth", Zone::Hand);
            cast(&mut g, designate, Some(Target::Card(mount)));
        }
        settle(&mut g);
        let change = if saddled_before_attack || !remove_mount {
            3
        } else {
            -4
        };
        assert_eq!(g.state.players[player.index()].life, life + change);
    }
}
