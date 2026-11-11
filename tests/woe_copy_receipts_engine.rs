//! Copiable Reflection exceptions and actual copied-Aura continuations.
mod support;

use rustmage as _;
use rustmage_core::cards::authored::CardRules;
use rustmage_core::{cards::*, engine::*};
use support::*;

const COPY_AURA: CardRules = CardRules {
    effect: Effect::Creature(3, 3),
    activated: &[ActivatedAbility {
        cost: ActivationCost {
            generic: 2,
            tap: true,
            mana_component: true,
            ..ActivationCost::ZERO
        },
        targets: TargetPlan::Single(TargetSpec::one(TargetRule::Filtered {
            base: &TargetRule::Object {
                zone: TargetZone::Battlefield,
                kinds: 32,
                relation: 0,
                other: false,
                nonland: false,
                tapped: false,
                min_mana: 0,
                max_mana: u32::MAX,
            },
            predicate: &TargetPredicate::NoOtherControlledSameName { tokens_only: false },
        })),
        program: &[
            Instruction::CreateNonlegendaryCopyRecordAura(Subject::Targets),
            Instruction::IfAmountAtLeast {
                value: AmountValue::MovedCount,
                minimum: 1,
                skip: 2,
            },
            Instruction::UntapSource,
            Instruction::Scry(2),
        ],
        guard: ActivationGuard::Always,
        origin: ActivationOrigin::Battlefield,
        cycling: false,
        exhaust: false,
        sorcery: true,
        once_per_turn: false,
    }],
    ..CardRules::EMPTY
};

const FOLLY_COPY: CardRules = CardRules {
    effect: Effect::Spell(
        TargetPlan::Single(TargetSpec::one(TargetRule::Creature)),
        &[Instruction::CreateReflectionCopy(Subject::Targets)],
    ),
    ..CardRules::EMPTY
};

const COPY: CardRules = CardRules {
    effect: Effect::Spell(
        TargetPlan::Single(TargetSpec::one(TargetRule::Creature)),
        &[Instruction::CreateCopyToken {
            subject: Subject::Targets,
            amount: 1,
        }],
    ),
    ..CardRules::EMPTY
};

const SACRIFICE: CardRules = CardRules {
    effect: Effect::Spell(
        TargetPlan::NONE,
        &[Instruction::SacrificeControlledSubtype("Reflection")],
    ),
    ..CardRules::EMPTY
};

fn activate(g: &mut Game, source: CardId, target: CardId) {
    act(
        g,
        Action::AnnounceActivate {
            card: source,
            ability: 0,
            counter_payment: Counters::default(),
        },
    );
    act(
        g,
        Action::ChooseTarget {
            target: Target::Card(target),
        },
    );
    pay(g);
}

fn pass_twice(g: &mut Game) {
    act(g, Action::Pass);
    act(g, Action::Pass);
}

#[test]
fn a_copy_continues_only_after_a_real_aura_enters_and_scry_survives_source_departure() {
    let _scope = fixture_rules::activate(&[(Card::HillGiant, &COPY_AURA)]);
    for player in [Player::P0, Player::P1] {
        let mut g = scenario(player);
        let source = put(&mut g, player, "Hill Giant", Zone::Battlefield);
        let bearer = put(&mut g, player, "Bear Cub", Zone::Battlefield);
        let aura = put(&mut g, player, "Bestial Bloodline", Zone::Hand);
        cast(&mut g, aura, Some(Target::Card(bearer)));
        settle(&mut g);
        let mut recipe = CopyRecipe::new(g.object_view(aura).card);
        recipe.overrides = Some(CopyOverrides {
            legendary: Some(true),
            ..CopyOverrides::NONE
        });
        g.apply_temporary_copy(g.object_ref(aura), recipe, CopyDeadline::EndOfTurn);
        assert!(g.object_view(aura).is_legendary());

        let mut non_aura = g.clone();
        let anthem = put(
            &mut non_aura,
            player,
            "Anthem of Champions",
            Zone::Battlefield,
        );
        activate(&mut non_aura, source, anthem);
        settle(&mut non_aura);
        assert!(non_aura.object_view(source).tapped);
        assert_eq!(non_aura.state.pending, Pending::Priority);
        assert!(non_aura.observe(player).objects.iter().any(|object| {
            object.zone == Zone::Battlefield
                && object.is_token()
                && object.name() == "Anthem of Champions"
        }));

        activate(&mut g, source, aura);
        assert!(g.object_view(source).tapped);
        let mut gone = g.clone();
        act(&mut gone, Action::Pass);
        let bounce = put(&mut gone, player.other(), "Unsummon", Zone::Hand);
        cast(&mut gone, bounce, Some(Target::Card(source)));
        pass_twice(&mut gone);
        for branch in [&mut g, &mut gone] {
            pass_twice(branch);
            assert_eq!(branch.state.pending, Pending::AuraEntry { player });
            let recipient = branch.object_ref(bearer);
            act(branch, Action::ChooseAuraAttachment { object: recipient });
            assert_eq!(branch.state.pending, Pending::EffectLibrary { player });
            let result = branch
                .observe(player)
                .objects
                .into_iter()
                .find(|object| {
                    object.zone == Zone::Battlefield
                        && object.is_token()
                        && object.name() == "Bestial Bloodline"
                })
                .unwrap();
            assert!(result.has_subtype("Aura"));
            assert!(!result.is_legendary());
            assert_eq!(result.attached_to, Some(recipient));
        }
        assert_eq!(gone.object_view(source).zone, Zone::Hand);
        assert_eq!(g.object_view(source).zone, Zone::Battlefield);
        assert!(!g.object_view(source).tapped);
    }
}

#[test]
fn reflections_copy_their_haste_and_subtype_and_the_final_sacrifice_uses_live_control() {
    let _scope = fixture_rules::activate(&[
        (Card::Divination, &FOLLY_COPY),
        (Card::from_name("Opt").unwrap(), &COPY),
        (Card::from_name("Quick Study").unwrap(), &SACRIFICE),
    ]);
    let player = Player::P0;
    let mut g = scenario(player);
    let original = put(&mut g, player, "Fynn, the Fangbearer", Zone::Battlefield);
    let friend = put(&mut g, player, "Changeling Wayfinder", Zone::Battlefield);
    let enemy = put(
        &mut g,
        player.other(),
        "Changeling Wayfinder",
        Zone::Battlefield,
    );
    g.state.objects[original as usize].growth.power = 5;
    let first_spell = put(&mut g, player, "Divination", Zone::Hand);
    cast(&mut g, first_spell, Some(Target::Card(original)));
    settle(&mut g);
    let first = g
        .observe(player)
        .objects
        .into_iter()
        .find(|object| object.zone == Zone::Battlefield && object.is_token())
        .unwrap();
    let second_spell = put(&mut g, player, "Opt", Zone::Hand);
    cast(&mut g, second_spell, Some(Target::Card(first.id)));
    settle(&mut g);
    let copies: Vec<_> = g
        .observe(player)
        .objects
        .into_iter()
        .filter(|object| object.zone == Zone::Battlefield && object.is_token())
        .collect();
    assert_eq!(copies.len(), 2);
    for copy in &copies {
        assert_eq!(copy.name(), "Fynn, the Fangbearer");
        assert_eq!(copy.power, 1);
        assert!(copy.has_subtype("Human") && copy.has_subtype("Reflection"));
        assert!(
            copy.keywords
                .contains(Keywords::HASTE.union(Keywords::DEATHTOUCH))
        );
        assert!(!copy.is_legendary());
    }
    let final_spell = put(&mut g, player, "Quick Study", Zone::Hand);
    cast(&mut g, final_spell, None);
    settle(&mut g);
    assert_eq!(g.object_view(friend).zone, Zone::Graveyard);
    assert_eq!(g.object_view(enemy).zone, Zone::Battlefield);
    assert_eq!(g.object_view(original).zone, Zone::Battlefield);
    assert!(copies.iter().all(|copy| !g.on_battlefield(copy.id)));
}
