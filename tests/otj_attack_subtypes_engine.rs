//! One receipt for each declared Mount or Vehicle, including both types at once.
mod support;
use rustmage::Card;
use rustmage_core::cards::authored::{CardExtras, CardRules};
use rustmage_core::cards::*;
use rustmage_core::engine::*;
use support::*;

const fn own_turn_hexproof(subtype: &'static str) -> KeywordGrant {
    KeywordGrant {
        condition: None,
        requires_life_gain: false,
        filter: CreatureFilter {
            relation: 0,
            other: false,
            subtype: Some(subtype),
            keyword: Keywords::NONE,
            attacking: false,
            tokens: None,
            artifact: false,
        },
        source_only: false,
        own_turn: true,
        keywords: Keywords::HEXPROOF,
    }
}
const MOUNT: KeywordGrant = own_turn_hexproof("Mount");
const VEHICLE: KeywordGrant = own_turn_hexproof("Vehicle");
const OBSERVER: CardRules = CardRules {
    effect: Effect::Creature(3, 2),
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::ControlledSubtypeAttacksAny(&["Mount", "Vehicle"]),
        targets: TargetPlan::NONE,
        program: &[Instruction::PutCounters {
            subject: Subject::Entering,
            kind: CounterKind::PlusOne,
            amount: 1,
        }],
    }],
    static_groups: &[StaticAbilityGroup {
        line: 0,
        first_layer: AbilityLayer::Ability,
        fragments: &[
            StaticAbilityFragment::KeywordGrant(0, MOUNT),
            StaticAbilityFragment::KeywordGrant(1, VEHICLE),
        ],
    }],
    extras: &CardExtras {
        keyword_grants: &[MOUNT, VEHICLE],
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
const MOUNT_FIXTURE: CardRules = CardRules {
    effect: Effect::Creature(3, 3),
    ..CardRules::EMPTY
};
const ADD_MOUNT: CardRules = CardRules {
    effect: Effect::Spell(
        TargetPlan::Single(TargetSpec::one(TargetRule::Creature)),
        &[Instruction::AddCreatureTypes {
            subject: Subject::Targets,
            types: &["Mount"],
        }],
    ),
    ..CardRules::EMPTY
};

fn resolve(g: &mut Game) {
    while matches!(g.state.pending, Pending::OrderTriggers { .. }) {
        choose(g, |action| matches!(action, Action::PutTrigger { .. }));
    }
    settle(g);
}

#[test]
fn a_declared_dual_subtype_counts_once_and_preserves_only_its_exact_subject() {
    let mount_definition = Card::from_name("Giant Beaver").unwrap();
    let _rules = fixture_rules::activate(&[
        (Card::GrizzlyBears, &OBSERVER),
        (mount_definition, &MOUNT_FIXTURE),
        (Card::GiantGrowth, &ADD_MOUNT),
    ]);
    let player = Player::P0;
    let mut g = scenario(player);
    put(&mut g, player, "Grizzly Bears", Zone::Battlefield);
    let mount = spawn(&mut g, player, mount_definition, Zone::Battlefield);
    let foreign_mount = spawn(&mut g, player.other(), mount_definition, Zone::Battlefield);
    let vehicle = put(&mut g, player, "Clamorous Ironclad", Zone::Battlefield);
    let inactive_vehicle = put(&mut g, player, "Clamorous Ironclad", Zone::Battlefield);
    let regular = put(&mut g, player, "Goblin Piker", Zone::Battlefield);
    let crew = put(&mut g, player, "Hill Giant", Zone::Battlefield);
    let add = put(&mut g, player, "Giant Growth", Zone::Hand);

    assert!(g.keywords(mount).contains(Keywords::HEXPROOF));
    assert!(g.keywords(inactive_vehicle).contains(Keywords::HEXPROOF));
    assert!(!g.is_creature(inactive_vehicle));
    assert!(!g.keywords(foreign_mount).contains(Keywords::HEXPROOF));
    g.state.active = player.other();
    assert!(!g.keywords(inactive_vehicle).contains(Keywords::HEXPROOF));
    g.state.active = player;

    act(
        &mut g,
        Action::AnnounceActivate {
            card: vehicle,
            ability: 0,
            counter_payment: Counters::default(),
        },
    );
    let crew = g.object_ref(crew);
    act(&mut g, Action::ChooseCrewer { object: crew });
    act(&mut g, Action::FinishCrew);
    settle(&mut g);
    cast(&mut g, add, Some(Target::Card(vehicle)));
    settle(&mut g);
    assert!(g.has_subtype(vehicle, "Mount"));
    assert!(g.has_subtype(vehicle, "Vehicle"));

    g.state.step = Step::Attackers;
    g.state.pending = Pending::Attackers;
    for card in [mount, vehicle, regular] {
        act(&mut g, Action::Attack { card });
    }
    act(&mut g, Action::FinishAttackers);
    let mount_ref = g.object_ref(mount);
    g.apply_temporary_copy(
        mount_ref,
        CopyRecipe::new(Card::GoblinPiker),
        CopyDeadline::EndOfTurn,
    );
    assert!(!g.has_subtype(mount, "Mount"));

    let mut returned = g.clone();
    let vehicle_ref = g.object_ref(vehicle);
    returned.move_object(vehicle, Zone::Hand);
    returned.move_object(vehicle, Zone::Battlefield);
    assert_ne!(returned.object_ref(vehicle), vehicle_ref);
    resolve(&mut returned);
    assert_eq!(returned.counter_count(vehicle, CounterKind::PlusOne), 0);

    resolve(&mut g);
    assert_eq!(g.counter_count(mount, CounterKind::PlusOne), 1);
    assert_eq!(g.counter_count(vehicle, CounterKind::PlusOne), 1);
    assert_eq!(g.counter_count(regular, CounterKind::PlusOne), 0);
    assert_eq!(g.counter_count(inactive_vehicle, CounterKind::PlusOne), 0);
}
