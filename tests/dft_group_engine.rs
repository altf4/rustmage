//! Independent group membership and layer-six keyword-loss outcomes.
mod support;
use rustmage::Card;
use rustmage_core::cards::authored::{CardExtras, CardRules};
use rustmage_core::cards::*;
use rustmage_core::engine::*;
use support::*;

const CREATURES: BattlefieldFilter = BattlefieldFilter {
    kinds: 1,
    relation: -1,
    nonland: false,
    tapped: None,
    without: Keywords::NONE,
    subtype: None,
    max_mana: u32::MAX,
};
const VEHICLES: BattlefieldFilter = BattlefieldFilter {
    kinds: 0,
    subtype: Some("Vehicle"),
    ..CREATURES
};
const OWN_NONLAND: BattlefieldFilter = BattlefieldFilter {
    kinds: 0,
    relation: 0,
    nonland: true,
    ..CREATURES
};
const PROTECT: CardRules = CardRules {
    effect: Effect::Creature(2, 2),
    activated: &[ActivatedAbility {
        exhaust: false,
        guard: ActivationGuard::Always,
        origin: ActivationOrigin::Battlefield,
        cycling: false,
        cost: ActivationCost {
            sacrifice: true,
            ..ActivationCost::ZERO
        },
        targets: TargetPlan::NONE,
        program: &[Instruction::GrantCounterBearingCreatures(
            Keywords::HEXPROOF.union(Keywords::INDESTRUCTIBLE),
        )],
        sorcery: false,
        once_per_turn: false,
    }],
    ..CardRules::EMPTY
};
const UNTAP: CardRules = CardRules {
    effect: Effect::Spell(
        TargetPlan::NONE,
        &[Instruction::UntapPermanentGroup(&[OWN_NONLAND])],
    ),
    ..CardRules::EMPTY
};
const INDESTRUCTIBLE_CREATURE: CardRules = CardRules {
    effect: Effect::Creature(2, 2),
    keywords: Keywords::INDESTRUCTIBLE.union(Keywords::MENACE),
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::SelfDies,
        targets: TargetPlan::NONE,
        program: &[Instruction::GainLife(PlayerSet::Controller, 3)],
    }],
    ..CardRules::EMPTY
};
const INDESTRUCTIBLE_VEHICLE: CardRules = CardRules {
    effect: Effect::Permanent,
    keywords: Keywords::INDESTRUCTIBLE,
    extras: &CardExtras {
        vehicle_stats: Some(Stats {
            power: 5,
            toughness: 5,
        }),
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
const REMOVE_ONLY: CardRules = CardRules {
    effect: Effect::Spell(
        TargetPlan::NONE,
        &[Instruction::RemovePermanentGroupKeywords {
            filters: &[CREATURES, VEHICLES],
            keywords: Keywords::INDESTRUCTIBLE,
        }],
    ),
    ..CardRules::EMPTY
};
const REMOVE_DESTROY: CardRules = CardRules {
    effect: Effect::Spell(
        TargetPlan::NONE,
        &[
            Instruction::RemovePermanentGroupKeywords {
                filters: &[CREATURES, VEHICLES],
                keywords: Keywords::INDESTRUCTIBLE,
            },
            Instruction::MoveBattlefield {
                filters: &[CREATURES, VEHICLES],
                exile: false,
            },
        ],
    ),
    ..CardRules::EMPTY
};

#[test]
fn counter_qualified_grant_uses_any_counter_and_freezes_recipient_membership() {
    let _scope = fixture_rules::activate(&[(Card::GrizzlyBears, &PROTECT)]);
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        let source = put(&mut g, p, "Grizzly Bears", Zone::Battlefield);
        let plus = put(&mut g, p, "Hill Giant", Zone::Battlefield);
        let stun = put(&mut g, p, "Bear Cub", Zone::Battlefield);
        let named = put(&mut g, p, "Savannah Lions", Zone::Battlefield);
        let later = put(&mut g, p, "Fire Elemental", Zone::Battlefield);
        let enemy = put(&mut g, p.other(), "Hill Giant", Zone::Battlefield);
        for (id, kind) in [
            (plus, CounterKind::PlusOne),
            (stun, CounterKind::Stun),
            (named, CounterKind::Incubation),
            (enemy, CounterKind::PlusOne),
        ] {
            g.add_counter(id, kind, 1);
        }
        act(
            &mut g,
            Action::AnnounceActivate {
                card: source,
                ability: 0,
                counter_payment: Counters::default(),
            },
        );
        pay(&mut g);
        assert_eq!(g.object_view(source).zone, Zone::Graveyard);
        settle(&mut g);
        let both = Keywords::HEXPROOF.union(Keywords::INDESTRUCTIBLE);
        for id in [plus, stun, named] {
            assert!(g.keywords(id).contains(both));
        }
        assert!(!g.keywords(enemy).contains(both));
        assert!(!g.keywords(later).contains(both));
        g.set_counters(named, Counters::default());
        g.add_counter(later, CounterKind::Page, 1);
        assert!(
            g.keywords(named).contains(both),
            "losing counter after resolution does not retract grant"
        );
        assert!(
            !g.keywords(later).contains(both),
            "later counter does not join frozen group"
        );
        g.finish_cleanup();
        for id in [plus, stun, named] {
            assert!(!g.keywords(id).intersects(both));
        }
    }
}

#[test]
fn nonland_untap_includes_noncreatures_and_obeys_stun_without_touching_lands() {
    let _scope = fixture_rules::activate(&[(Card::Divination, &UNTAP)]);
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        let creature = put(&mut g, p, "Hill Giant", Zone::Battlefield);
        let artifact = put(&mut g, p, "Short Bow", Zone::Battlefield);
        let enchantment = put(&mut g, p, "Perilous Snare", Zone::Battlefield);
        let land = put(&mut g, p, "Forest", Zone::Battlefield);
        let enemy = put(&mut g, p.other(), "Short Bow", Zone::Battlefield);
        for id in [creature, artifact, enchantment, land, enemy] {
            g.state.objects[id as usize].tapped = true;
        }
        g.add_counter(creature, CounterKind::Stun, 1);
        let spell = put(&mut g, p, "Divination", Zone::Hand);
        cast(&mut g, spell, None);
        settle(&mut g);
        assert!(!g.object_view(artifact).tapped);
        assert!(!g.object_view(enchantment).tapped);
        assert!(g.object_view(creature).tapped);
        assert_eq!(g.counter_count(creature, CounterKind::Stun), 0);
        assert!(g.object_view(land).tapped);
        assert!(g.object_view(enemy).tapped);
    }
}

#[test]
fn keyword_loss_preserves_other_abilities_and_later_grants_use_timestamp_order() {
    let _scope = fixture_rules::activate(&[
        (Card::GrizzlyBears, &INDESTRUCTIBLE_CREATURE),
        (Card::Divination, &REMOVE_ONLY),
    ]);
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        let target = put(&mut g, p, "Grizzly Bears", Zone::Battlefield);
        let spell = put(&mut g, p, "Divination", Zone::Hand);
        cast(&mut g, spell, None);
        settle(&mut g);
        assert!(!g.keywords(target).contains(Keywords::INDESTRUCTIBLE));
        assert!(g.keywords(target).contains(Keywords::MENACE));
        assert!(g.printed_abilities_available(target));
        let saved = g.snapshot();
        let mut child = g.clone();
        child.grant_keyword_effects(
            &[target],
            child.object_ref(target),
            Keywords::INDESTRUCTIBLE,
        );
        assert!(
            child.keywords(target).contains(Keywords::INDESTRUCTIBLE),
            "newer granting effect wins"
        );
        assert!(
            !g.keywords(target).contains(Keywords::INDESTRUCTIBLE),
            "child chronology stays owned"
        );
        g.restore(&saved).unwrap();
        g.finish_cleanup();
        assert!(
            g.keywords(target).contains(Keywords::INDESTRUCTIBLE),
            "printed ability returns at cleanup"
        );
    }
}

#[test]
fn keyword_only_loss_allows_destruction_and_retains_real_death_triggers() {
    let vehicle = Card::from_name("Broadcast Rambler").unwrap();
    let _scope = fixture_rules::activate(&[
        (Card::GrizzlyBears, &INDESTRUCTIBLE_CREATURE),
        (vehicle, &INDESTRUCTIBLE_VEHICLE),
        (Card::Divination, &REMOVE_DESTROY),
    ]);
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        let creature = put(&mut g, p, "Grizzly Bears", Zone::Battlefield);
        let vehicle = put(&mut g, p.other(), "Broadcast Rambler", Zone::Battlefield);
        let unaffected = put(&mut g, p, "Short Bow", Zone::Battlefield);
        assert!(!g.is_creature(vehicle));
        let spell = put(&mut g, p, "Divination", Zone::Hand);
        cast(&mut g, spell, None);
        settle(&mut g);
        assert_eq!(g.object_view(creature).zone, Zone::Graveyard);
        assert_eq!(g.object_view(vehicle).zone, Zone::Graveyard);
        assert_eq!(g.object_view(unaffected).zone, Zone::Battlefield);
        assert_eq!(
            g.state.players[p.index()].life,
            23,
            "death trigger was not erased with indestructible"
        );
    }
}
