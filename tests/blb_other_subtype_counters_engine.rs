//! Other permanents of a creature subtype can include a noncreature Kindred.
mod support;

use rustmage as _;
use rustmage_core::cards::authored::{CardExtras, CardRules};
use rustmage_core::engine::characteristic_query::CharacteristicQuery;
use rustmage_core::engine::funding::{FundingGoal, FundingState};
use rustmage_core::{cards::*, engine::*};
use support::*;

const MENACE: KeywordGrant = KeywordGrant {
    condition: None,
    requires_life_gain: false,
    filter: CreatureFilter {
        relation: 0,
        other: true,
        subtype: Some("Squirrel"),
        keyword: Keywords::NONE,
        attacking: false,
        tokens: None,
        artifact: false,
    },
    source_only: false,
    own_turn: false,
    keywords: Keywords::MENACE,
};

const SQUIRRELS: CardRules = CardRules {
    effect: Effect::Creature(2, 1),
    keywords: Keywords::CHANGELING,
    activated: &[ActivatedAbility {
        guard: ActivationGuard::Always,
        origin: ActivationOrigin::Battlefield,
        targets: TargetPlan::NONE,
        cycling: false,
        exhaust: false,
        sorcery: false,
        once_per_turn: false,
        cost: ActivationCost::ZERO,
        program: &[Instruction::PutFilteredCounters {
            filter: CountFilter {
                zone: CountZone::Battlefield,
                relation: 0,
                kind: 0,
                exclude: 0,
                subtype: Some("Squirrel"),
                basic: false,
                other: true,
                cards_only: false,
            },
            kind: CounterKind::PlusOne,
            amount: 1,
        }],
    }],
    static_groups: &[
        StaticAbilityGroup {
            line: 0,
            first_layer: AbilityLayer::Type,
            fragments: &[StaticAbilityFragment::Changeling],
        },
        StaticAbilityGroup {
            line: 1,
            first_layer: AbilityLayer::Ability,
            fragments: &[StaticAbilityFragment::KeywordGrant(0, MENACE)],
        },
    ],
    extras: &CardExtras {
        keyword_grants: &[MENACE],
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};

#[test]
fn a_noncreature_kindred_receives_the_grant_and_a_counter_before_animation() {
    let source_card = Card::from_name("Three Tree Mascot").unwrap();
    let _rules = fixture_rules::activate(&[(source_card, &SQUIRRELS)]);
    for player in [Player::P0, Player::P1] {
        let mut game = scenario(player);
        let source = spawn(&mut game, player, source_card, Zone::Battlefield);
        let core = put(&mut game, player, "Firdoch Core", Zone::Battlefield);
        let harvester = put(&mut game, player, "Barkform Harvester", Zone::Battlefield);
        let foreign = put(&mut game, player.other(), "Firdoch Core", Zone::Battlefield);
        let bears = put(&mut game, player, "Grizzly Bears", Zone::Battlefield);

        assert!(!game.is_creature(core));
        assert!(game.has_subtype(core, "Squirrel"));
        for recipient in [core, harvester] {
            assert!(game.keywords(recipient).contains(Keywords::MENACE));
        }
        for excluded in [source, foreign, bears] {
            assert!(!game.keywords(excluded).contains(Keywords::MENACE));
        }
        let goal = FundingGoal {
            actor: player,
            required: [0; 6],
            generic: 0,
            reserve: None,
            source: None,
        };
        let world = FundingState::from_game(&game, &goal);
        let projected = CharacteristicQuery::in_world(&world);
        assert!(projected.keywords(core).contains(Keywords::MENACE));
        assert!(!projected.keywords(foreign).contains(Keywords::MENACE));

        let mut layered = game.clone();
        let source_ref = layered.object_ref(source);
        layered.remove_abilities_exact(
            &[layered.object_ref(bears)],
            source_ref,
            player,
            AnimationDuration::Permanent,
        );
        assert!(layered.keywords(core).contains(Keywords::MENACE));
        layered.remove_abilities_exact(
            &[source_ref],
            source_ref,
            player,
            AnimationDuration::Permanent,
        );
        assert!(!layered.keywords(core).contains(Keywords::MENACE));

        choose(&mut game, |action| {
            matches!(action, Action::Activate { card, ability: 0, .. }
                | Action::AnnounceActivate { card, ability: 0, .. } if *card == source)
        });
        if matches!(game.state.pending, Pending::PayCosts { .. }) {
            pay(&mut game);
        }

        let mut reentered = game.clone();
        let before = reentered.object_ref(source);
        reentered.move_object(source, Zone::Exile);
        reentered.move_object(source, Zone::Battlefield);
        assert_ne!(reentered.object_ref(source), before);
        settle(&mut game);
        settle(&mut reentered);

        for branch in [&game, &reentered] {
            assert!(!branch.is_creature(core));
            assert_eq!(branch.counter_count(core, CounterKind::PlusOne), 1);
            assert_eq!(branch.counter_count(harvester, CounterKind::PlusOne), 1);
            assert_eq!(branch.counter_count(foreign, CounterKind::PlusOne), 0);
            assert_eq!(branch.counter_count(bears, CounterKind::PlusOne), 0);
        }
        assert_eq!(game.counter_count(source, CounterKind::PlusOne), 0);
        assert_eq!(reentered.counter_count(source, CounterKind::PlusOne), 1);
    }
}
