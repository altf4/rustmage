//! Grouped reveal costs retain only their paid type/exile outcomes on the spell.
mod support;
use rustmage as _;
use rustmage_core::cards::authored::{CardExtras, CardRules};
use rustmage_core::cards::*;
use rustmage_core::engine::*;
use support::*;
const CREATURES: SacrificeFilter = SacrificeFilter {
    keywords: Keywords::NONE,
    source_only: false,
    kinds: 1,
    nonland: false,
    other: true,
    tokens: None,
    artifact_enchantment_or_token: false,
    subtype: None,
};
const SEARCH: CardRules = CardRules {
    effect: Effect::Spell(
        TargetPlan::NONE,
        &[Instruction::SearchCreatureManaValue {
            maximum: AmountValue::ChosenX,
            destination: SearchDestination::BeheldCreatureType,
        }],
    ),
    extras: &CardExtras {
        x_symbols: 1,
        additional_cost: Some(AdditionalCost {
            kind: EffectChoiceKind::OptionalBeholdType(2),
            filter: CREATURES,
        }),
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
const EXILE: CardRules = CardRules {
    effect: Effect::Creature(2, 2),
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::SelfLeaves,
        targets: TargetPlan::NONE,
        program: &[Instruction::ReturnLinkedExileToHand],
    }],
    extras: &CardExtras {
        additional_cost: Some(AdditionalCost {
            kind: EffectChoiceKind::BeholdAndExile,
            filter: SacrificeFilter {
                subtype: Some("Elf"),
                ..CREATURES
            },
        }),
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
const FLASHBACK: CardRules = CardRules {
    effect: Effect::Spell(
        TargetPlan::NONE,
        &[Instruction::Draw(PlayerSet::Controller, 2)],
    ),
    extras: &CardExtras {
        flashback_cost: Some(ManaCost {
            colored: [0, 1, 0, 0, 0, 0],
            generic: 1,
        }),
        flashback_additional: Some(AdditionalCost {
            kind: EffectChoiceKind::BeholdMany(3),
            filter: CREATURES,
        }),
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
fn finish_cast(g: &mut Game) {
    for _ in 0..32 {
        if g.state.pending == Pending::Priority {
            return;
        }
        choose(g, |a| {
            matches!(
                a,
                Action::PayCosts { .. }
                    | Action::BeginOrderedCosts
                    | Action::ChooseCastCost { .. }
                    | Action::ChooseAdditionalCost { .. }
            )
        });
    }
    panic!("behold payment bound");
}
fn wait_search(g: &mut Game) {
    for _ in 0..20 {
        if matches!(g.state.pending, Pending::EffectSearch { .. }) {
            return;
        }
        choose(g, |a| matches!(a, Action::Pass | Action::PutTrigger { .. }));
    }
    panic!("private search not reached");
}
#[test]
fn optional_group_reveals_only_on_payment_and_paid_type_survives_witness_departure() {
    let _rules = fixture_rules::activate(&[(Card::Divination, &SEARCH)]);
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        let witness = put(&mut g, p, "Grizzly Bears", Zone::Hand);
        let permanent = put(&mut g, p, "Bear Cub", Zone::Battlefield);
        let found = put(&mut g, p, "Bear Cub", Zone::Library);
        let source = put(&mut g, p, "Divination", Zone::Hand);
        act(&mut g, Action::AnnounceCast { card: source });
        act(
            &mut g,
            Action::ChooseBeholdType {
                creature_type: Some(CreatureType::from_name("Bear").unwrap()),
            },
        );
        assert!(
            !g.observe(p.other())
                .known_hand
                .contains(&g.object_ref(witness))
        );
        act(&mut g, Action::ChooseX { value: 2 });
        finish_cast(&mut g);
        assert!(
            g.observe(p.other())
                .known_hand
                .contains(&g.object_ref(witness))
        );
        g.move_object(permanent, Zone::Graveyard);
        g.move_object(witness, Zone::Graveyard);
        wait_search(&mut g);
        assert!(g.observe(p.other()).search_choice.is_none());
        act(&mut g, Action::ChooseSearch { card: found });
        assert_eq!(g.state.objects[found as usize].zone, Zone::Library);
        act(&mut g, Action::FinishSearch);
        settle(&mut g);
        assert_eq!(g.state.objects[found as usize].zone, Zone::Battlefield);
    }
}
#[test]
fn optional_behold_decline_searches_to_hand_and_private_failure_is_allowed() {
    let _rules = fixture_rules::activate(&[(Card::Divination, &SEARCH)]);
    for fail in [false, true] {
        let mut g = scenario(Player::P0);
        let found = put(&mut g, Player::P0, "Grizzly Bears", Zone::Library);
        let source = put(&mut g, Player::P0, "Divination", Zone::Hand);
        act(&mut g, Action::AnnounceCast { card: source });
        reject(
            &mut g,
            Action::ChooseBeholdType {
                creature_type: Some(CreatureType::from_name("Bear").unwrap()),
            },
        );
        act(
            &mut g,
            Action::ChooseBeholdType {
                creature_type: None,
            },
        );
        act(&mut g, Action::ChooseX { value: 2 });
        finish_cast(&mut g);
        wait_search(&mut g);
        if !fail {
            act(&mut g, Action::ChooseSearch { card: found });
        }
        act(&mut g, Action::FinishSearch);
        settle(&mut g);
        assert_eq!(
            g.state.objects[found as usize].zone,
            if fail { Zone::Library } else { Zone::Hand }
        );
    }
}
#[test]
fn behold_exile_links_the_entered_permanent_and_a_countered_spell_keeps_its_cost_exiled() {
    let _rules = fixture_rules::activate(&[(Card::GrizzlyBears, &EXILE)]);
    for counter in [false, true] {
        let mut g = scenario(Player::P1);
        let elf = put(&mut g, Player::P1, "Llanowar Elves", Zone::Hand);
        let source = put(&mut g, Player::P1, "Grizzly Bears", Zone::Hand);
        act(&mut g, Action::AnnounceCast { card: source });
        finish_cast(&mut g);
        assert_eq!(g.state.objects[elf as usize].zone, Zone::Exile);
        if counter {
            assert_eq!(g.state.stack.len(), 1, "{:#?}", g.state.stack);
            assert_eq!(g.state.stack[0].kind, StackKind::Spell);
            let negate = put(&mut g, Player::P0, "Cancel", Zone::Hand);
            act(&mut g, Action::Pass);
            cast(&mut g, negate, Some(Target::Spell(source)));
            settle(&mut g);
            assert_eq!(g.state.objects[elf as usize].zone, Zone::Exile);
        } else {
            settle(&mut g);
            assert_eq!(
                g.observe(Player::P1).linked_exile[0].source,
                g.object_ref(source)
            );
            g.move_object(source, Zone::Graveyard);
            g.prepare_triggers();
            settle(&mut g);
            assert_eq!(g.state.objects[elf as usize].zone, Zone::Hand);
        }
    }
}
#[test]
fn flashback_beholds_three_distinct_hand_or_battlefield_creatures_without_tapping_them() {
    let _rules = fixture_rules::activate(&[(Card::Divination, &FLASHBACK)]);
    let mut g = scenario(Player::P0);
    let a = put(&mut g, Player::P0, "Llanowar Elves", Zone::Hand);
    let b = put(&mut g, Player::P0, "Grizzly Bears", Zone::Battlefield);
    let source = put(&mut g, Player::P0, "Divination", Zone::Graveyard);
    assert!(
        !g.legal_actions()
            .contains(&Action::AnnounceFlashback { card: source })
    );
    let c = put(&mut g, Player::P0, "Fog Bank", Zone::Battlefield);
    act(&mut g, Action::AnnounceFlashback { card: source });
    finish_cast(&mut g);
    settle(&mut g);
    assert_eq!(g.state.objects[source as usize].zone, Zone::Exile);
    assert_eq!(g.state.objects[a as usize].zone, Zone::Hand);
    assert!(!g.state.objects[b as usize].tapped && !g.state.objects[c as usize].tapped);
    assert_eq!(g.hand_count(Player::P0), 3);
}
