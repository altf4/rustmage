//! Late output choices must finish real payments; soft resolution choices may decline.
mod support;
use rustmage as _;
use rustmage_core::cards::authored::CardRules;
use rustmage_core::cards::*;
use rustmage_core::engine::*;
use std::collections::BTreeSet;
use support::*;

const RED: Mana = [0, 0, 0, 1, 0, 0];
const BLUE: Mana = [0, 1, 0, 0, 0, 0];
const GREEN: Mana = [0, 0, 0, 0, 1, 0];
const RED_TWO: Mana = [0, 0, 0, 2, 0, 0];
const BASE: ActivatedAbility = ActivatedAbility {
    exhaust: false,
    guard: ActivationGuard::Always,
    cycling: false,
    origin: ActivationOrigin::Battlefield,
    cost: ActivationCost::ZERO,
    targets: TargetPlan::NONE,
    program: &[],
    sorcery: false,
    once_per_turn: false,
};
const BLUE_COST_RED_OUTPUT: CardRules = CardRules {
    effect: Effect::Creature(2, 2),
    activated: &[ActivatedAbility {
        cost: ActivationCost {
            colored: BLUE,
            mana_component: true,
            tap: true,
            ..ActivationCost::ZERO
        },
        program: &[Instruction::AddMana(ManaProduction::Fixed(RED_TWO))],
        ..BASE
    }],
    ..CardRules::EMPTY
};
const SOFT_RED_PAYMENT: CardRules = CardRules {
    effect: Effect::Spell(
        TargetPlan::NONE,
        &[
            Instruction::MayPayMana {
                cost: ManaCost {
                    colored: RED,
                    generic: 0,
                },
                skip: 1,
            },
            Instruction::GainLife(PlayerSet::Controller, 3),
        ],
    ),
    ..CardRules::EMPTY
};
fn color(index: usize) -> Mana {
    let mut m = [0; 6];
    m[index] = 1;
    m
}
fn all_colors() -> BTreeSet<Mana> {
    (0..5).map(color).collect()
}
fn late_start(card: CardId) -> Action {
    Action::ActivateMana {
        card,
        ability: 0,
        payment: [0; 6],
        counter_payment: Counters::default(),
        production: [0; 6],
    }
}
fn converter_start(card: CardId) -> Action {
    Action::AnnounceMana {
        card,
        ability: 0,
        counter_payment: Counters::default(),
        production: RED_TWO,
    }
}
fn choice(production: Mana) -> Action {
    Action::ChooseMana { production }
}
fn engineer(g: &mut Game, p: Player) -> CardId {
    let source = put(g, p, "Draconautics Engineer", Zone::Battlefield);
    g.state.players[p.index()].mana = [0; 6];
    source
}
fn begin_engineer(g: &mut Game, source: CardId) {
    act(
        g,
        Action::AnnounceActivate {
            card: source,
            ability: 0,
            counter_payment: Counters::default(),
        },
    );
}
fn treasure(g: &mut Game, p: Player) -> CardId {
    let token = Card::from_name("@token/b60f6cd3b44b22bd289fe4e2").unwrap();
    g.create_tokens(p, token, 1, false);
    g.state
        .objects
        .iter()
        .enumerate()
        .find_map(|(id, o)| {
            (o.card == token && o.controller == p && o.zone == Zone::Battlefield)
                .then_some(id as CardId)
        })
        .unwrap()
}
fn late_choices(g: &Game, p: Player) -> BTreeSet<Mana> {
    assert!(matches!(g.state.pending, Pending::EffectMana { .. }));
    let snapshot = g.snapshot();
    let revision = g.revision();
    let offered: Vec<_> = g
        .legal_actions()
        .into_iter()
        .filter_map(|a| match a {
            Action::ChooseMana { production } => Some(production),
            _ => None,
        })
        .collect();
    let set: BTreeSet<_> = offered.iter().copied().collect();
    assert_eq!(set.len(), offered.len(), "indexed choices are unique");
    assert_eq!(
        g.observe(p)
            .mana_choices
            .into_iter()
            .collect::<BTreeSet<_>>(),
        set
    );
    assert!(
        g.observe(p.other()).mana_choices.is_empty(),
        "only the current chooser owns the choice"
    );
    assert_eq!(g.snapshot(), snapshot);
    assert_eq!(g.revision(), revision);
    set
}
fn finish_engineer(g: &mut Game, source: CardId) {
    pay(g);
    settle(g);
    assert_eq!(g.counter_count(source, CounterKind::PlusOne), 1);
    assert_eq!(stats(g, source), (3, 3));
    assert_eq!(g.object_view(source).exhausted_abilities & 1, 1);
}
fn reach_resolution_payment(g: &mut Game) {
    for _ in 0..32 {
        if g.legal_actions().contains(&Action::DeclineResolutionMana) {
            return;
        }
        choose(g, |a| {
            matches!(
                a,
                Action::Pass | Action::PutTrigger { .. } | Action::FinishTargets
            )
        });
    }
    panic!("bounded optional resolution payment");
}
#[test]
fn actual_engineer_last_treasure_has_only_red_in_raw_index_and_observed_choices_owned() {
    for p in [Player::P0, Player::P1] {
        let mut parent = scenario(p);
        let source = engineer(&mut parent, p);
        let mana = treasure(&mut parent, p);
        begin_engineer(&mut parent, source);
        act(&mut parent, late_start(mana));
        assert_ne!(
            parent.object_view(mana).zone,
            Zone::Battlefield,
            "the mana source cost is already paid"
        );
        assert_eq!(late_choices(&parent, p), BTreeSet::from([RED]));
        assert!(
            !parent.legal_actions().contains(&choice(GREEN)),
            "no indexed green choice exists"
        );
        reject(&mut parent, choice(GREEN));
        let saved = parent.snapshot();
        let mut raw = parent.clone();
        let mut indexed = parent.clone();
        let mut restored = parent.clone();
        restored.restore(&saved).unwrap();
        drop(parent);
        for (n, g) in [&mut raw, &mut indexed, &mut restored]
            .into_iter()
            .enumerate()
        {
            if n == 1 {
                let index = g
                    .legal_actions()
                    .iter()
                    .position(|a| *a == choice(RED))
                    .unwrap();
                g.apply_index(p, g.revision(), index).unwrap();
            } else {
                act(g, choice(RED));
            }
            assert_eq!(g.state.players[p.index()].mana, RED);
            finish_engineer(g, source);
            assert_eq!(g.state.players[p.index()].mana, [0; 6]);
        }
        assert_eq!(raw.snapshot(), indexed.snapshot());
        assert_eq!(raw.snapshot(), restored.snapshot());
    }
}
#[test]
fn actual_red_spell_parents_reject_a_late_wrong_color_after_the_treasure_is_spent() {
    for p in [Player::P0, Player::P1] {
        for (name, generic, damage) in [("Shock", 0, 2), ("Lightning Strike", 1, 3)] {
            let mut g = scenario(p);
            let mana = treasure(&mut g, p);
            let spell = put(&mut g, p, name, Zone::Hand);
            g.state.players[p.index()].mana = [0, 0, 0, 0, 0, generic];
            act(&mut g, Action::AnnounceCast { card: spell });
            act(
                &mut g,
                Action::ChooseTarget {
                    target: Target::Player(p.other()),
                },
            );
            if g.legal_actions().contains(&Action::FinishTargets) {
                act(&mut g, Action::FinishTargets);
            }
            act(&mut g, late_start(mana));
            assert_eq!(late_choices(&g, p), BTreeSet::from([RED]));
            reject(&mut g, choice(GREEN));
            act(&mut g, choice(RED));
            pay(&mut g);
            settle(&mut g);
            assert_eq!(g.object_view(spell).zone, Zone::Graveyard);
            assert_eq!(g.state.players[p.other().index()].life, 20 - damage);
            assert_eq!(g.state.players[p.index()].mana, [0; 6]);
        }
    }
}
#[test]
fn last_treasure_output_must_finish_the_real_blue_mana_child_and_its_red_ancestor() {
    let _scope = fixture_rules::activate(&[(Card::GrizzlyBears, &BLUE_COST_RED_OUTPUT)]);
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        let source = engineer(&mut g, p);
        let converter = put(&mut g, p, "Grizzly Bears", Zone::Battlefield);
        let mana = treasure(&mut g, p);
        begin_engineer(&mut g, source);
        act(&mut g, converter_start(converter));
        act(&mut g, late_start(mana));
        assert_eq!(g.observe(p).suspended_mana_announcements.len(), 2);
        assert_eq!(late_choices(&g, p), BTreeSet::from([BLUE]));
        for wrong in [GREEN, RED] {
            reject(&mut g, choice(wrong));
        }
        act(&mut g, choice(BLUE));
        pay(&mut g);
        assert_eq!(g.state.players[p.index()].mana, RED_TWO);
        finish_engineer(&mut g, source);
        assert_eq!(g.state.players[p.index()].mana, RED);
        assert!(g.object_view(converter).tapped);
    }
}
#[test]
fn another_real_red_source_keeps_a_green_late_output_viable() {
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        let source = engineer(&mut g, p);
        let mountain = put(&mut g, p, "Mountain", Zone::Battlefield);
        let mana = treasure(&mut g, p);
        begin_engineer(&mut g, source);
        act(&mut g, late_start(mana));
        assert_eq!(late_choices(&g, p), all_colors());
        act(&mut g, choice(GREEN));
        act(&mut g, Action::TapMana { card: mountain });
        finish_engineer(&mut g, source);
        assert_eq!(g.state.players[p.index()].mana, GREEN);
        assert!(g.object_view(mountain).tapped);
    }
}
#[test]
fn frozen_tap_bonus_funds_red_while_the_sacrificed_sources_green_keeps_its_turn_quality() {
    const BONUS_SPELL: CardRules = CardRules {
        effect: Effect::Spell(TargetPlan::NONE, &[Instruction::AddMountainManaTrigger]),
        ..CardRules::EMPTY
    };
    const MOUNTAIN: CardRules = CardRules {
        effect: Effect::Land,
        activated: &[ActivatedAbility {
            cost: ActivationCost {
                tap: true,
                sacrifice: true,
                ..ActivationCost::ZERO
            },
            program: &[Instruction::AddMana(ManaProduction::Tagged {
                production: &ManaProduction::Choice {
                    mask: 31,
                    amount: 1,
                },
                quality: &ManaQuality {
                    restriction: ManaRestriction::AbilityActivation,
                    spell_bonus: ManaSpellBonus::None,
                    retention: ManaRetention::Turn,
                },
            })],
            ..BASE
        }],
        ..CardRules::EMPTY
    };
    let _scope = fixture_rules::activate(&[
        (Card::from_name("Opt").unwrap(), &BONUS_SPELL),
        (Card::Mountain, &MOUNTAIN),
    ]);
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        let bonus = put(&mut g, p, "Opt", Zone::Hand);
        cast(&mut g, bonus, None);
        settle(&mut g);
        let source = engineer(&mut g, p);
        let mana = put(&mut g, p, "Mountain", Zone::Battlefield);
        begin_engineer(&mut g, source);
        act(&mut g, late_start(mana));
        assert_eq!(g.object_view(mana).zone, Zone::Graveyard);
        assert_eq!(g.state.players[p.index()].mana, [0; 6]);
        assert_eq!(
            late_choices(&g, p),
            all_colors(),
            "saved red tap bonus can pay the red cost even when chosen output is green"
        );
        reject(&mut g, choice([0, 0, 0, 0, 2, 0]));
        act(&mut g, choice(GREEN));
        assert_eq!(
            g.state.players[p.index()].mana,
            [0, 0, 0, 1, 1, 0],
            "base output and frozen bonus each occur exactly once"
        );
        finish_engineer(&mut g, source);
        assert_eq!(g.state.players[p.index()].mana, GREEN);
        g.clear_step_mana(false, true);
        assert_eq!(
            g.state.players[p.index()].mana,
            GREEN,
            "paid source's saved Turn retention remains"
        );
        g.clear_step_mana(false, false);
        assert_eq!(g.state.players[p.index()].mana, [0; 6]);
    }
}
#[test]
fn priority_mana_has_no_hard_parent_and_keeps_every_color() {
    for p in [Player::P0, Player::P1] {
        for index in 0..5 {
            let mut g = scenario(p);
            let mana = treasure(&mut g, p);
            g.state.players[p.index()].mana = [0; 6];
            act(&mut g, late_start(mana));
            assert_eq!(late_choices(&g, p), all_colors());
            act(&mut g, choice(color(index)));
            assert_eq!(g.state.pending, Pending::Priority);
            assert_eq!(g.state.players[p.index()].mana, color(index));
        }
    }
}
#[test]
fn ordinary_resolving_add_mana_keeps_every_color_and_continues_the_spell() {
    const RITUAL: CardRules = CardRules {
        effect: Effect::Spell(
            TargetPlan::NONE,
            &[
                Instruction::AddMana(ManaProduction::Choice {
                    mask: 31,
                    amount: 1,
                }),
                Instruction::GainLife(PlayerSet::Controller, 1),
            ],
        ),
        ..CardRules::EMPTY
    };
    let _scope = fixture_rules::activate(&[(Card::from_name("Opt").unwrap(), &RITUAL)]);
    for p in [Player::P0, Player::P1] {
        for index in 0..5 {
            let mut g = scenario(p);
            let spell = put(&mut g, p, "Opt", Zone::Hand);
            cast(&mut g, spell, None);
            g.state.players[p.index()].mana = [0; 6];
            for _ in 0..16 {
                if matches!(g.state.pending, Pending::EffectMana { .. }) {
                    break;
                }
                act(&mut g, Action::Pass);
            }
            assert_eq!(late_choices(&g, p), all_colors());
            act(&mut g, choice(color(index)));
            settle(&mut g);
            assert_eq!(g.state.players[p.index()].mana, color(index));
            assert_eq!(g.state.players[p.index()].life, 21);
            assert_eq!(g.object_view(spell).zone, Zone::Graveyard);
        }
    }
}
#[test]
fn optional_resolution_payment_keeps_all_colors_because_the_resolution_can_decline() {
    let _scope = fixture_rules::activate(&[(Card::from_name("Opt").unwrap(), &SOFT_RED_PAYMENT)]);
    for p in [Player::P0, Player::P1] {
        for accept in [false, true] {
            let mut g = scenario(p);
            let spell = put(&mut g, p, "Opt", Zone::Hand);
            let mana = treasure(&mut g, p);
            cast(&mut g, spell, None);
            g.state.players[p.index()].mana = [0; 6];
            reach_resolution_payment(&mut g);
            act(&mut g, late_start(mana));
            assert_eq!(late_choices(&g, p), all_colors());
            act(&mut g, choice(if accept { RED } else { GREEN }));
            if accept {
                choose(&mut g, |a| matches!(a, Action::PayResolutionMana { .. }));
            } else {
                act(&mut g, Action::DeclineResolutionMana);
            }
            settle(&mut g);
            assert_eq!(
                g.state.players[p.index()].life,
                if accept { 23 } else { 20 }
            );
            assert_eq!(
                g.state.players[p.index()].mana,
                if accept { [0; 6] } else { GREEN }
            );
        }
    }
}
#[test]
fn declining_an_outer_resolution_does_not_excuse_stranding_its_real_mana_child() {
    let _scope = fixture_rules::activate(&[
        (Card::from_name("Opt").unwrap(), &SOFT_RED_PAYMENT),
        (Card::GrizzlyBears, &BLUE_COST_RED_OUTPUT),
    ]);
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        let spell = put(&mut g, p, "Opt", Zone::Hand);
        let converter = put(&mut g, p, "Grizzly Bears", Zone::Battlefield);
        let mana = treasure(&mut g, p);
        cast(&mut g, spell, None);
        g.state.players[p.index()].mana = [0; 6];
        reach_resolution_payment(&mut g);
        act(&mut g, converter_start(converter));
        act(&mut g, late_start(mana));
        assert_eq!(late_choices(&g, p), BTreeSet::from([BLUE]));
        reject(&mut g, choice(GREEN));
        act(&mut g, choice(BLUE));
        pay(&mut g);
        choose(&mut g, |a| matches!(a, Action::PayResolutionMana { .. }));
        settle(&mut g);
        assert_eq!(g.state.players[p.index()].life, 23);
        assert_eq!(g.state.players[p.index()].mana, RED);
    }
}
