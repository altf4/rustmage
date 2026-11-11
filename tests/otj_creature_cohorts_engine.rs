//! Resolution-time keyword recipients and counter amounts use exact live facts.
mod support;
use rustmage::Card;
use rustmage_core::cards::authored::CardRules;
use rustmage_core::cards::*;
use rustmage_core::engine::*;
use support::*;

const REMOVE: CardRules = CardRules {
    effect: Effect::Spell(
        TargetPlan::Groups(&[
            TargetSpec::one(TargetRule::creature_you_control()),
            TargetSpec::one(TargetRule::Player {
                opponent_only: true,
            }),
        ]),
        &[
            Instruction::SelectGroup(1),
            Instruction::RemovePlayerCreatureKeywords {
                players: PlayerSet::Targeted,
                keywords: Keywords::FIRST_STRIKE.union(Keywords::DOUBLE_STRIKE),
            },
        ],
    ),
    ..CardRules::EMPTY
};
const GIVE_STRIKE: CardRules = CardRules {
    effect: Effect::Spell(
        TargetPlan::Single(TargetSpec::one(TargetRule::Creature)),
        &[Instruction::GrantKeywords {
            subject: Subject::Targets,
            keywords: Keywords::DOUBLE_STRIKE,
        }],
    ),
    ..CardRules::EMPTY
};
const BRAWLER: CardRules = CardRules {
    effect: Effect::Creature(2, 2),
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::OtherCreatureEntersYou,
        targets: TargetPlan::NONE,
        program: &[Instruction::PutCountersScaled {
            subject: Subject::Entering,
            kind: CounterKind::PlusOne,
            amount: AmountValue::EnteringCreaturePower,
        }],
    }],
    ..CardRules::EMPTY
};
const ENTERTAINER: CardRules = CardRules {
    effect: Effect::Creature(2, 2),
    activated: &[ActivatedAbility {
        exhaust: false,
        guard: ActivationGuard::Always,
        origin: ActivationOrigin::Battlefield,
        cycling: false,
        cost: ActivationCost {
            generic: 1,
            mana_component: true,
            tap: true,
            ..ActivationCost::ZERO
        },
        targets: TargetPlan::NONE,
        program: &[Instruction::PutCountersOnEnteredCreatures {
            players: PlayerSet::Controller,
            kind: CounterKind::PlusOne,
            amount: 1,
        }],
        sorcery: false,
        once_per_turn: false,
    }],
    ..CardRules::EMPTY
};

#[test]
fn remove_keywords_follows_the_surviving_player_role_and_only_current_creatures() {
    let _rules = fixture_rules::activate(&[
        (Card::GiantGrowth, &REMOVE),
        (Card::from_name("Opt").unwrap(), &GIVE_STRIKE),
    ]);
    let player = Player::P0;
    let mut g = scenario(player);
    let own = put(&mut g, player, "Grizzly Bears", Zone::Battlefield);
    let enemy = put(&mut g, player.other(), "Goblin Piker", Zone::Battlefield);
    g.state.objects[enemy as usize].gained_keywords = Keywords::FLYING
        .union(Keywords::FIRST_STRIKE)
        .union(Keywords::DOUBLE_STRIKE);
    let spell = put(&mut g, player, "Giant Growth", Zone::Hand);
    let grant = put(&mut g, player, "Opt", Zone::Hand);
    announce(
        &mut g,
        spell,
        &[Target::Card(own), Target::Player(player.other())],
    );
    g.move_object(own, Zone::Exile);
    settle(&mut g);
    assert!(g.keywords(enemy).contains(Keywords::FLYING));
    assert!(
        !g.keywords(enemy)
            .intersects(Keywords::FIRST_STRIKE.union(Keywords::DOUBLE_STRIKE))
    );

    let later = put(&mut g, player.other(), "Reckless Lackey", Zone::Battlefield);
    assert!(g.keywords(later).contains(Keywords::FIRST_STRIKE));
    cast(&mut g, grant, Some(Target::Card(enemy)));
    settle(&mut g);
    assert!(g.keywords(enemy).contains(Keywords::DOUBLE_STRIKE));
}

#[test]
fn entrant_power_is_sampled_at_resolution_for_the_exact_creature() {
    let _rules = fixture_rules::activate(&[(Card::GrizzlyBears, &BRAWLER)]);
    let player = Player::P1;
    let mut g = scenario(player);
    let source = put(&mut g, player, "Grizzly Bears", Zone::Battlefield);
    let entrant = put(&mut g, player, "Bear Cub", Zone::Hand);
    let growth = put(&mut g, player, "Giant Growth", Zone::Hand);
    cast(&mut g, entrant, None);
    act(&mut g, Action::Pass);
    act(&mut g, Action::Pass);
    assert_eq!(g.state.objects[entrant as usize].zone, Zone::Battlefield);
    assert_eq!(g.state.stack.len(), 1);
    let exact = g.object_ref(entrant);

    let mut stale = g.clone();
    stale.move_object(source, Zone::Graveyard);
    stale.move_object(entrant, Zone::Hand);
    stale.move_object(entrant, Zone::Battlefield);
    assert_ne!(exact, stale.object_ref(entrant));
    settle(&mut stale);
    assert_eq!(stale.counter_count(entrant, CounterKind::PlusOne), 0);

    cast(&mut g, growth, Some(Target::Card(entrant)));
    settle(&mut g);
    assert_eq!(g.counter_count(entrant, CounterKind::PlusOne), 5);
}

#[test]
fn counter_cohort_uses_current_control_and_exact_entry_this_turn() {
    let _rules = fixture_rules::activate(&[(Card::GrizzlyBears, &ENTERTAINER)]);
    let player = Player::P0;
    let mut g = scenario(player);
    let source = put(&mut g, player, "Grizzly Bears", Zone::Battlefield);
    let old = put(&mut g, player, "Bear Cub", Zone::Battlefield);
    let new = put(&mut g, player, "Bear Cub", Zone::Hand);
    let seized = put(&mut g, player.other(), "Goblin Piker", Zone::Hand);
    let enemy = put(&mut g, player.other(), "Bear Cub", Zone::Hand);
    for id in [new, seized, enemy] {
        g.move_object(id, Zone::Battlefield);
    }
    g.change_controller(g.object_ref(seized), player);
    act(
        &mut g,
        Action::AnnounceActivate {
            card: source,
            ability: 0,
            counter_payment: Counters::default(),
        },
    );
    pay(&mut g);
    settle(&mut g);
    assert!(g.object_view(source).tapped);
    for (id, counters) in [(source, 0), (old, 0), (new, 1), (seized, 1), (enemy, 0)] {
        assert_eq!(g.counter_count(id, CounterKind::PlusOne), counters);
    }
}
