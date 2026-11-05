//! CR702.16 / CR702.147 shared-engine obligations; no TDM card admissions.
mod support;
use rustmage as _;
use rustmage_core::cards::authored::{CardExtras, CardRules};
use rustmage_core::cards::*;
use rustmage_core::engine::ability_world::CharacteristicWorld;
use rustmage_core::engine::funding::{FundingGoal, FundingState};
use rustmage_core::engine::*;
use support::*;

const CREATURE: TargetPlan = TargetPlan::Single(TargetSpec::one(TargetRule::Creature));
const fn ability(
    cost: ActivationCost,
    targets: TargetPlan,
    program: &'static [Instruction],
) -> ActivatedAbility {
    ActivatedAbility {
        exhaust: false,
        guard: ActivationGuard::Always,
        origin: ActivationOrigin::Battlefield,
        cycling: false,
        cost,
        targets,
        program,
        sorcery: false,
        once_per_turn: false,
    }
}
const fn protected(keywords: Keywords) -> CardRules {
    CardRules {
        effect: Effect::Creature(10, 10),
        keywords,
        ..CardRules::EMPTY
    }
}
static PROTECTED: [CardRules; 5] = [
    protected(Keywords::PROTECTION_WHITE),
    protected(Keywords::PROTECTION_BLUE),
    protected(Keywords::PROTECTION_BLACK),
    protected(Keywords::PROTECTION_RED),
    protected(Keywords::PROTECTION_GREEN),
];
const ALL_COLORS: Keywords = Keywords::PROTECTION_WHITE
    .union(Keywords::PROTECTION_BLUE)
    .union(Keywords::PROTECTION_BLACK)
    .union(Keywords::PROTECTION_RED)
    .union(Keywords::PROTECTION_GREEN);
const DAMAGE_SPELL: CardRules = CardRules {
    effect: Effect::Spell(CREATURE, &[Instruction::Damage(2)]),
    ..CardRules::EMPTY
};
const SOURCE: CardRules = CardRules {
    effect: Effect::Creature(3, 3),
    activated: &[ability(
        ActivationCost {
            generic: 1,
            mana_component: true,
            ..ActivationCost::ZERO
        },
        CREATURE,
        &[Instruction::Damage(2)],
    )],
    ..CardRules::EMPTY
};
const OMEN_SOURCE: CardRules = CardRules {
    effect: Effect::Creature(3, 3),
    extras: &CardExtras {
        omen_spell: Some(Card::LightningBolt),
        adventure_identity: Some("@test/green creature with red Omen"),
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
const DECAYED: CardRules = protected(Keywords::DECAYED);
const ALL_PROTECTED: CardRules = protected(ALL_COLORS.union(Keywords::DECAYED));
const AURA: CardRules = CardRules {
    effect: Effect::Aura(CREATURE),
    aura: Some(AuraSpec {
        kinds: 1,
        relation: -1,
        bonus: EquipmentSpec {
            stats: Stats {
                power: 0,
                toughness: 0,
            },
            keywords: Keywords::NONE,
        },
    }),
    ..CardRules::EMPTY
};
const COUNTER_MANA: CardRules = CardRules {
    effect: Effect::Creature(2, 2),
    activated: &[ability(
        ActivationCost {
            remove: Some((Some(CounterKind::Decayed), 1)),
            ..ActivationCost::ZERO
        },
        TargetPlan::NONE,
        MANA,
    )],
    ..CardRules::EMPTY
};
const BLANK: CardRules = CardRules {
    effect: Effect::Spell(
        CREATURE,
        &[Instruction::RemoveTargetAbilitiesUntilEndOfTurn],
    ),
    ..CardRules::EMPTY
};
const MANA: &[Instruction] = &[Instruction::AddMana(ManaProduction::Fixed([
    0, 0, 0, 0, 0, 1,
]))];
const PROTECTION_PROVIDER: CardRules = CardRules {
    effect: Effect::Creature(2, 2),
    activated: &[ability(
        ActivationCost {
            sacrifice: true,
            ..ActivationCost::ZERO
        },
        TargetPlan::NONE,
        MANA,
    )],
    static_groups: &[StaticAbilityGroup {
        line: 0,
        first_layer: AbilityLayer::Ability,
        fragments: &[StaticAbilityFragment::ControlledPermanentKeywords {
            kinds: 1,
            keywords: Keywords::PROTECTION_GREEN,
        }],
    }],
    ..CardRules::EMPTY
};
fn goal(p: Player) -> FundingGoal {
    FundingGoal {
        actor: p,
        required: [0; 6],
        generic: 1,
        reserve: None,
        source: None,
    }
}
fn paint(g: &mut Game, id: CardId, colors: u8) {
    g.change_object_form(
        g.object_ref(id),
        AnimationSpec {
            artifact: false,
            creature_types: AnimationCreatureTypes::Unchanged,
            base: None,
            keywords: Keywords::NONE,
            duration: AnimationDuration::EndOfTurn,
        },
        characteristic_layers::AnimationTypeMode::PreserveCardTypes,
    );
    g.state
        .roles
        .as_mut()
        .unwrap()
        .animations
        .last_mut()
        .unwrap()
        .colors = Some(colors);
}
fn event(g: &Game, source: CardId, target: CardId, preventable: bool, combat: bool) -> DamageEvent {
    DamageEvent {
        source: g.damage_source(source),
        target: Target::Card(target),
        amount: 2,
        preventable,
        combat,
    }
}
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
fn declare(g: &mut Game, source: CardId) {
    g.state.pending = Pending::Attackers;
    g.state.step = Step::Attackers;
    act(g, Action::Attack { card: source });
    act(g, Action::FinishAttackers);
}
fn end_combat(g: &mut Game) {
    g.begin_step(Step::EndCombat);
    g.prepare_triggers();
}
fn decayed_packets(g: &Game) -> usize {
    g.state
        .stack
        .iter()
        .chain(g.state.pending_triggers.iter())
        .chain(g.state.ordering_triggers.iter())
        .filter(|item| matches!(item.kind, StackKind::DecayedAttack))
        .count()
}

#[test]
fn colored_protection_targeting_checks_all_five_colors_for_both_controllers_before_payment() {
    let names = ["Pacifism", "Opt", "Murder", "Shock", "Giant Growth"];
    for p in [Player::P0, Player::P1] {
        for (protected_color, rules) in PROTECTED.iter().enumerate() {
            let mut rows = vec![(Card::HillGiant, rules)];
            rows.extend(
                names
                    .iter()
                    .map(|n| (Card::from_name(n).unwrap(), &DAMAGE_SPELL)),
            );
            let _scope = fixture_rules::activate(&rows);
            for owner in [p, p.other()] {
                let mut g = scenario(p);
                let recipient = put(&mut g, owner, "Hill Giant", Zone::Battlefield);
                for (source_color, name) in names.iter().enumerate() {
                    let source = put(&mut g, p, name, Zone::Hand);
                    assert_eq!(
                        g.cast_target_matches(
                            TargetRule::Creature,
                            p,
                            g.object_ref(source),
                            Card::from_name(name).unwrap(),
                            Target::Card(recipient)
                        ),
                        protected_color != source_color,
                        "{name} against protection color {protected_color}; own permanents also protected"
                    );
                    if source_color == protected_color {
                        reject(&mut g, Action::AnnounceCast { card: source });
                    }
                }
            }
        }
    }
}

#[test]
fn paid_chosen_omen_face_uses_red_spell_qualities_instead_of_green_physical_card() {
    for red_protection in [false, true] {
        let _scope = fixture_rules::activate(&[
            (Card::GrizzlyBears, &OMEN_SOURCE),
            (Card::LightningBolt, &DAMAGE_SPELL),
            (
                Card::HillGiant,
                if red_protection {
                    &PROTECTED[3]
                } else {
                    &PROTECTED[4]
                },
            ),
        ]);
        let mut g = scenario(Player::P0);
        let recipient = put(&mut g, Player::P1, "Hill Giant", Zone::Battlefield);
        let source = put(&mut g, Player::P0, "Grizzly Bears", Zone::Hand);
        assert_eq!(
            g.cast_definition_targets_available(
                CREATURE,
                Player::P0,
                g.object_ref(source),
                Card::LightningBolt
            ),
            !red_protection
        );
        if red_protection {
            reject(&mut g, Action::AnnounceOmen { card: source });
        } else {
            act(&mut g, Action::AnnounceOmen { card: source });
            act(
                &mut g,
                Action::ChooseTarget {
                    target: Target::Card(recipient),
                },
            );
            pay(&mut g);
            assert_eq!(g.state.stack[0].source.unwrap().card, Card::LightningBolt);
            settle(&mut g);
            assert_eq!(g.state.objects[recipient as usize].damage, 2);
            assert_eq!(g.state.objects[source as usize].zone, Zone::Library);
        }
    }
}

#[test]
fn ability_targeting_rechecks_current_source_color_and_uses_exact_departure_lki_without_rebinding()
{
    let _scope = fixture_rules::activate(&[
        (Card::from_name("Savannah Lions").unwrap(), &SOURCE),
        (Card::HillGiant, &PROTECTED[2]),
    ]);
    for mode in 0..4 {
        let mut g = scenario(Player::P0);
        let source = put(&mut g, Player::P0, "Savannah Lions", Zone::Battlefield);
        let recipient = put(&mut g, Player::P1, "Hill Giant", Zone::Battlefield);
        activate(&mut g, source, recipient);
        let old = g.object_ref(source);
        match mode {
            0 => paint(&mut g, source, 4), // live black: target becomes illegal
            1 => {
                paint(&mut g, source, 4);
                g.move_object(source, Zone::Hand);
                g.move_object(source, Zone::Battlefield);
            } // black LKI, new white object
            2 => {
                g.move_object(source, Zone::Hand);
                g.move_object(source, Zone::Battlefield);
                paint(&mut g, source, 4);
            } // white LKI, new black object
            3 => {
                g.apply_temporary_copy(
                    old,
                    CopyRecipe::new(Card::from_name("Vampire Nighthawk").unwrap()),
                    CopyDeadline::EndOfTurn,
                );
            }
            _ => unreachable!(),
        }
        if mode == 1 || mode == 2 {
            assert_ne!(old, g.object_ref(source));
        }
        settle(&mut g);
        assert_eq!(
            g.state.objects[recipient as usize].damage,
            if mode == 2 { 2 } else { 0 }
        );
    }
    let mut g = scenario(Player::P0);
    let source = put(&mut g, Player::P0, "Savannah Lions", Zone::Battlefield);
    let recipient = put(&mut g, Player::P1, "Hill Giant", Zone::Battlefield);
    paint(&mut g, source, 2 | 4); // any matching component of a multicolor source is enough
    assert!(!g.activation_target_matches(
        TargetRule::Creature,
        Player::P0,
        g.object_ref(source),
        Target::Card(recipient)
    ));
    paint(&mut g, source, 0);
    activate(&mut g, source, recipient);
    settle(&mut g);
    assert_eq!(g.state.objects[recipient as usize].damage, 2);
}

#[test]
fn colored_protection_blocks_matching_creatures_and_prevents_both_damage_kinds_but_not_unpreventable_damage()
 {
    for (index, rules) in PROTECTED.iter().enumerate() {
        let _scope = fixture_rules::activate(&[
            (Card::HillGiant, rules),
            (Card::from_name("Savannah Lions").unwrap(), &SOURCE),
        ]);
        for p in [Player::P0, Player::P1] {
            let mut g = scenario(p);
            let attacker = put(&mut g, p, "Hill Giant", Zone::Battlefield);
            let source = put(&mut g, p.other(), "Savannah Lions", Zone::Battlefield);
            paint(&mut g, source, 1 << index);
            g.state.step = Step::Blockers;
            g.state.pending = Pending::Blockers;
            g.state.attackers.push(AttackDeclaration {
                card: attacker,
                defender: Some(CombatDefender::Player(p.other())),
                blocked: false,
            });
            assert!(!g.can_block_pair(source, attacker));
            reject(
                &mut g,
                Action::Block {
                    blocker: source,
                    attacker,
                },
            );
            paint(&mut g, source, 0);
            assert!(g.can_block_pair(source, attacker));
            paint(&mut g, source, 1 << index);
            for combat in [false, true] {
                let hit = event(&g, source, attacker, true, combat);
                g.apply_damage_batch(&[hit]);
                g.advance_replacements(true);
                assert_eq!(g.state.objects[attacker as usize].damage, 0);
            }
            let hit = event(&g, source, attacker, false, false);
            g.apply_damage_batch(&[hit]);
            g.advance_replacements(true);
            assert_eq!(g.state.objects[attacker as usize].damage, 2);
            g.state.damage_prevention_prohibited = true;
            let hit = event(&g, source, attacker, true, true);
            g.apply_damage_batch(&[hit]);
            g.advance_replacements(true);
            assert_eq!(g.state.objects[attacker as usize].damage, 4);
        }
    }
}

#[test]
fn protection_participates_in_affected_players_prevention_replacement_order_and_preserves_shields()
{
    let _scope = fixture_rules::activate(&[
        (Card::HillGiant, &PROTECTED[0]),
        (Card::from_name("Savannah Lions").unwrap(), &SOURCE),
    ]);
    for choose_double_first in [false, true] {
        let mut g = scenario(Player::P0);
        let source = put(&mut g, Player::P0, "Savannah Lions", Zone::Battlefield);
        let recipient = put(&mut g, Player::P1, "Hill Giant", Zone::Battlefield);
        put(&mut g, Player::P0, "Gratuitous Violence", Zone::Battlefield);
        g.add_counter(recipient, CounterKind::Shield, 1);
        let hit = event(&g, source, recipient, true, false);
        g.apply_damage_batch(&[hit]);
        g.advance_replacements(true);
        assert_eq!(g.actor(), Some(Player::P1));
        assert_eq!(g.state.pending, Pending::Replacement { player: Player::P1 });
        assert!(g.legal_actions().iter().any(|a| matches!(
            a,
            Action::ChooseDamagePrevention {
                prevention: DamagePreventionKind::ProtectionColor(ChoiceColor::White),
                ..
            }
        )));
        if choose_double_first {
            choose(&mut g, |a| matches!(a, Action::ChooseReplacement { .. }));
        }
        choose(&mut g, |a| {
            matches!(
                a,
                Action::ChooseDamagePrevention {
                    prevention: DamagePreventionKind::ProtectionColor(ChoiceColor::White),
                    ..
                }
            )
        });
        assert_eq!(g.state.objects[recipient as usize].damage, 0);
        assert_eq!(
            g.counter_count(recipient, CounterKind::Shield),
            1,
            "prevention chosen before consuming shield"
        );
    }
}

#[test]
fn protection_from_each_color_rejects_aura_entry_and_equipment_then_cleans_later_color_changes() {
    let _scope = fixture_rules::activate(&[
        (Card::HillGiant, &PROTECTED[0]),
        (Card::from_name("Witness Protection").unwrap(), &AURA),
    ]);
    let mut g = scenario(Player::P0);
    let recipient = put(&mut g, Player::P0, "Hill Giant", Zone::Battlefield);
    assert!(!g.aura_can_enchant(
        Card::from_name("Pacifism").unwrap(),
        Player::P0,
        g.object_ref(recipient)
    ));
    let aura = put(&mut g, Player::P0, "Pacifism", Zone::Graveyard);
    let movement = g.entry_move(aura, Zone::Battlefield, false);
    let world = FundingState::from_game(&g, &goal(Player::P0));
    let mut projected = false;
    world.visit_entry_moves(vec![movement.clone()], true, None, &mut |next| {
        assert_eq!(next.object(aura).zone, Zone::Graveyard);
        projected = true;
        false
    });
    assert!(
        projected,
        "projected entry refuses the same protected attachment"
    );
    g.stage_entries(vec![movement], None);
    g.advance_entries();
    assert_eq!(
        g.state.objects[aura as usize].zone,
        Zone::Graveyard,
        "no legal Aura attachment leaves it in origin"
    );
    let equipment = put(&mut g, Player::P0, "Short Bow", Zone::Battlefield);
    assert!(g.equipment_can_attach(g.object_ref(equipment), g.object_ref(recipient)));
    assert_eq!(
        g.attach_equipment_batch(&[g.object_ref(equipment)], g.object_ref(recipient)),
        1
    );
    paint(&mut g, equipment, 1);
    assert!(!g.equipment_can_attach(g.object_ref(equipment), g.object_ref(recipient)));
    g.state_based_actions();
    assert_eq!(g.state.objects[equipment as usize].attached_to, None);
    assert_eq!(g.state.objects[equipment as usize].zone, Zone::Battlefield);
    let blue_aura = put(&mut g, Player::P0, "Witness Protection", Zone::Hand);
    cast(&mut g, blue_aura, Some(Target::Card(recipient)));
    settle(&mut g);
    assert_eq!(g.state.objects[blue_aura as usize].zone, Zone::Battlefield);
    assert_eq!(
        g.state.objects[blue_aura as usize].attached_to,
        Some(g.object_ref(recipient))
    );
    paint(&mut g, blue_aura, 1);
    g.state_based_actions();
    assert_eq!(
        g.state.objects[blue_aura as usize].zone,
        Zone::Graveyard,
        "SBA uses live Aura colors"
    );
}

#[test]
fn actual_and_funding_attachment_worlds_requalify_after_virtual_provider_departure() {
    let _scope = fixture_rules::activate(&[(Card::LlanowarElves, &PROTECTION_PROVIDER)]);
    let mut g = scenario(Player::P0);
    let provider = put(&mut g, Player::P0, "Llanowar Elves", Zone::Battlefield);
    let recipient = put(&mut g, Player::P0, "Grizzly Bears", Zone::Battlefield);
    let equipment = put(&mut g, Player::P0, "Short Bow", Zone::Battlefield);
    paint(&mut g, equipment, 16);
    g.state.players[0].mana = [0; 6];
    let request = goal(Player::P0);
    let world = FundingState::from_game(&g, &request);
    assert!(!equipment::can_attach(
        &g,
        g.object_ref(equipment),
        g.object_ref(recipient)
    ));
    assert!(!equipment::can_attach(
        &world,
        g.object_ref(equipment),
        g.object_ref(recipient)
    ));
    let saved = g.snapshot();
    let step = world
        .moves(&request)
        .into_iter()
        .find(|s| world.resources[s.resource].id == provider && s.ability == Some(0))
        .unwrap();
    let mut visited = false;
    world.visit_applied(&step, &mut |next| {
        assert_eq!(next.object(provider).zone, Zone::Graveyard);
        assert!(equipment::can_attach(
            next,
            g.object_ref(equipment),
            g.object_ref(recipient)
        ));
        visited = true;
        false
    });
    assert!(visited);
    assert_eq!(g.snapshot(), saved);
    act(
        &mut g,
        Action::ActivateMana {
            card: provider,
            ability: 0,
            payment: [0; 6],
            counter_payment: Counters::default(),
            production: [0, 0, 0, 0, 0, 1],
        },
    );
    assert!(g.equipment_can_attach(g.object_ref(equipment), g.object_ref(recipient)));
}

#[test]
fn layer_six_suppression_removes_protection_and_decayed_then_later_counter_grant_restores_only_decayed()
 {
    let _scope = fixture_rules::activate(&[
        (Card::HillGiant, &ALL_PROTECTED),
        (Card::from_name("Opt").unwrap(), &BLANK),
    ]);
    let mut g = scenario(Player::P0);
    let recipient = put(&mut g, Player::P1, "Hill Giant", Zone::Battlefield);
    g.add_counter(recipient, CounterKind::Decayed, 2);
    assert!(
        g.keywords(recipient)
            .contains(ALL_COLORS.union(Keywords::DECAYED))
    );
    let blank = put(&mut g, Player::P0, "Opt", Zone::Hand);
    // All five protection abilities make a blue suppression spell illegal.
    reject(&mut g, Action::AnnounceCast { card: blank });
    // A source-independent, nontargeted layer-six removal can still apply.
    let source = g.object_ref(blank);
    let reference = g.object_ref(recipient);
    let timestamp = g.next_role_timestamp();
    g.state
        .roles
        .as_mut()
        .unwrap()
        .resolved_ability_modifications
        .push(ResolvedAbilityModificationView {
            recipient: reference,
            source,
            keywords: Keywords::NONE,
            timestamp,
        });
    assert!(
        !g.keywords(recipient)
            .intersects(ALL_COLORS.union(Keywords::DECAYED))
    );
    assert!(g.can_be_blocker(recipient));
    let world = FundingState::from_game(&g, &goal(Player::P0));
    assert!(
        !world
            .activation_keywords(recipient)
            .contains(Keywords::DECAYED)
    );
    g.add_counter(recipient, CounterKind::Decayed, 1);
    assert!(g.keywords(recipient).contains(Keywords::DECAYED));
    assert!(!g.keywords(recipient).intersects(ALL_COLORS));
    assert!(!g.can_be_blocker(recipient));
    g.pay_counters(recipient, &Counters::of(CounterKind::Decayed, 2));
    assert!(
        g.keywords(recipient).contains(Keywords::DECAYED),
        "remaining counter retains kind's grant timestamp"
    );
    g.pay_counters(recipient, &Counters::of(CounterKind::Decayed, 1));
    assert!(!g.keywords(recipient).contains(Keywords::DECAYED));
    assert!(g.can_be_blocker(recipient));
}

#[test]
fn decayed_counter_payment_is_projected_and_printed_or_copied_decayed_has_both_parts() {
    let _scope = fixture_rules::activate(&[
        (Card::LlanowarElves, &COUNTER_MANA),
        (Card::HillGiant, &DECAYED),
    ]);
    let mut g = scenario(Player::P0);
    let source = put(&mut g, Player::P0, "Llanowar Elves", Zone::Battlefield);
    g.add_counter(source, CounterKind::Decayed, 1);
    g.state.players[0].mana = [0; 6];
    let request = goal(Player::P0);
    let world = FundingState::from_game(&g, &request);
    assert!(
        world
            .activation_keywords(source)
            .contains(Keywords::DECAYED)
    );
    let step = world
        .moves(&request)
        .into_iter()
        .find(|s| world.resources[s.resource].id == source && s.ability == Some(0))
        .unwrap();
    let mut visited = false;
    world.visit_applied(&step, &mut |next| {
        assert_eq!(next.counter_count(source, CounterKind::Decayed), 0);
        assert!(!next.activation_keywords(source).contains(Keywords::DECAYED));
        visited = true;
        false
    });
    assert!(visited);
    assert_eq!(g.counter_count(source, CounterKind::Decayed), 1);
    act(
        &mut g,
        Action::ActivateMana {
            card: source,
            ability: 0,
            payment: [0; 6],
            counter_payment: Counters::of(CounterKind::Decayed, 1),
            production: [0, 0, 0, 0, 0, 1],
        },
    );
    assert!(!g.keywords(source).contains(Keywords::DECAYED));
    let copied = put(&mut g, Player::P1, "Grizzly Bears", Zone::Battlefield);
    g.apply_temporary_copy(
        g.object_ref(copied),
        CopyRecipe::new(Card::HillGiant),
        CopyDeadline::EndOfTurn,
    );
    assert!(g.keywords(copied).contains(Keywords::DECAYED));
    assert!(!g.can_be_blocker(copied));
    g.state.active = Player::P1;
    declare(&mut g, copied);
    assert_eq!(decayed_packets(&g), 1);
    settle(&mut g);
    assert_eq!(g.observe(Player::P1).delayed_sacrifices.len(), 1);
    // The source no longer has decayed, but its independent delayed ability remains.
    g.expire_temporary_copies(true);
    end_combat(&mut g);
    settle(&mut g);
    assert_eq!(g.state.objects[copied as usize].zone, Zone::Graveyard);
}

#[test]
fn decayed_sacrifices_once_per_attack_after_counter_removal_but_gaining_it_after_attack_is_not_retroactive()
 {
    for had_at_attack in [false, true] {
        let mut g = scenario(Player::P0);
        let source = put(&mut g, Player::P0, "Grizzly Bears", Zone::Battlefield);
        if had_at_attack {
            g.add_counter(source, CounterKind::Decayed, 3);
        }
        declare(&mut g, source);
        assert_eq!(decayed_packets(&g), usize::from(had_at_attack));
        if had_at_attack {
            g.pay_counters(source, &Counters::of(CounterKind::Decayed, 3));
        } else {
            g.add_counter(source, CounterKind::Decayed, 1);
        }
        settle(&mut g);
        assert_eq!(
            g.observe(Player::P0).delayed_sacrifices.len(),
            usize::from(had_at_attack)
        );
        if had_at_attack {
            assert!(g.observe(Player::P0).delayed_sacrifices[0].end_combat);
        }
        end_combat(&mut g);
        settle(&mut g);
        assert_eq!(
            g.state.objects[source as usize].zone,
            if had_at_attack {
                Zone::Graveyard
            } else {
                Zone::Battlefield
            }
        );
        assert!(g.observe(Player::P0).delayed_sacrifices.is_empty());
    }
}

#[test]
fn decayed_delayed_sacrifice_retains_attack_controller_exact_incarnation_and_token_slot() {
    let _scope = fixture_rules::activate(&[(Card::GrizzlyBears, &DECAYED)]);
    for change in 0..4 {
        let mut g = scenario(Player::P0);
        let source = if change == 3 {
            g.create_copy_groups_inner(
                Player::P0,
                &[TokenGroupView {
                    token: Card::GrizzlyBears,
                    amount: 1,
                    tapped: false,
                }],
                &[Some(CopyRecipe::new(Card::GrizzlyBears))],
                None,
            )[0]
            .card
        } else {
            put(&mut g, Player::P0, "Grizzly Bears", Zone::Battlefield)
        };
        g.state.objects[source as usize].summoning_sick = false;
        let old = g.object_ref(source);
        declare(&mut g, source);
        settle(&mut g);
        match change {
            0 => {
                g.record_control(old, Player::P1, None);
            }
            1 => {
                g.move_object(source, Zone::Hand);
                g.move_object(source, Zone::Battlefield);
            }
            2 => {
                g.apply_temporary_copy(
                    old,
                    CopyRecipe::new(Card::HillGiant),
                    CopyDeadline::EndOfTurn,
                );
            }
            3 => {
                assert!(g.state.objects[source as usize].is_token());
            }
            _ => unreachable!(),
        }
        end_combat(&mut g);
        settle(&mut g);
        assert_eq!(
            g.state.objects[source as usize].zone,
            match change {
                0 | 1 => Zone::Battlefield,
                2 => Zone::Graveyard,
                3 => Zone::Outside,
                _ => unreachable!(),
            }
        );
        assert!(
            g.observe(Player::P0).delayed_sacrifices.is_empty(),
            "delayed trigger only happens once even if sacrifice impossible"
        );
        if change == 3 {
            let fresh = g.create_copy_groups_inner(
                Player::P0,
                &[TokenGroupView {
                    token: Card::GrizzlyBears,
                    amount: 1,
                    tapped: false,
                }],
                &[Some(CopyRecipe::new(Card::GrizzlyBears))],
                None,
            )[0];
            assert_eq!(fresh.card, old.card);
            assert_ne!(fresh, old);
            end_combat(&mut g);
            settle(&mut g);
            assert_eq!(g.state.objects[fresh.card as usize].zone, Zone::Battlefield);
        }
    }
}

#[test]
fn delayed_attack_and_end_combat_packets_are_counterable_and_owned_raw_index_restore_are_deterministic()
 {
    const COUNTER: CardRules = CardRules {
        effect: Effect::Spell(
            TargetPlan::Single(TargetSpec::one(TargetRule::Ability)),
            &[Instruction::Counter],
        ),
        ..CardRules::EMPTY
    };
    let _scope = fixture_rules::activate(&[
        (Card::GrizzlyBears, &DECAYED),
        (Card::from_name("Opt").unwrap(), &COUNTER),
    ]);
    let mut g = scenario(Player::P0);
    let source = put(&mut g, Player::P0, "Grizzly Bears", Zone::Battlefield);
    declare(&mut g, source);
    let trigger = g.state.stack.last().unwrap().id;
    let counter = put(&mut g, Player::P0, "Opt", Zone::Hand);
    cast(&mut g, counter, Some(Target::Stack(trigger)));
    settle(&mut g);
    end_combat(&mut g);
    settle(&mut g);
    assert_eq!(
        g.state.objects[source as usize].zone,
        Zone::Battlefield,
        "countering attack trigger prevents delayed ability creation"
    );
    g.state.attackers.clear();
    g.state.objects[source as usize].tapped = false;
    declare(&mut g, source);
    settle(&mut g);
    end_combat(&mut g);
    assert_eq!(g.state.stack.len(), 1);
    let saved = g.snapshot();
    let revision = g.revision();
    let actor = g.actor().unwrap();
    let index = g
        .legal_actions()
        .iter()
        .position(|a| *a == Action::Pass)
        .unwrap();
    let mut raw = g.clone();
    let mut indexed = g.clone();
    let mut sibling = g.clone();
    drop(g);
    assert_eq!(
        raw.apply(actor.other(), Action::Pass),
        Err(Error::WrongActor)
    );
    assert_eq!(raw.snapshot(), saved);
    raw.apply(actor, Action::Pass).unwrap();
    indexed.apply_index(actor, revision, index).unwrap();
    settle(&mut raw);
    settle(&mut indexed);
    assert_eq!(raw.snapshot(), indexed.snapshot());
    assert_eq!(raw.state.objects[source as usize].zone, Zone::Graveyard);
    let counter = put(&mut sibling, actor, "Opt", Zone::Hand);
    let trigger = sibling.state.stack.last().unwrap().id;
    cast(&mut sibling, counter, Some(Target::Stack(trigger)));
    settle(&mut sibling);
    assert_eq!(
        sibling.state.objects[source as usize].zone,
        Zone::Battlefield,
        "end-combat delayed trigger is independently counterable"
    );
    indexed.restore(&saved).unwrap();
    assert_eq!(
        indexed.apply_index(actor, revision, index),
        Err(Error::StaleRevision)
    );
    act(&mut indexed, Action::Pass);
    settle(&mut indexed);
    assert_eq!(indexed.snapshot(), raw.snapshot());
}

#[test]
fn keyword_widening_retains_old_numeric_wire_values_and_records_hot_layout() {
    assert_eq!(
        serde_json::to_string(&Keywords::SPLIT_SECOND).unwrap(),
        "2147483648"
    );
    assert_eq!(
        serde_json::to_string(&Keywords::PROTECTION_EVERYTHING).unwrap(),
        "134217728"
    );
    assert_eq!(
        serde_json::to_string(&Keywords::PROTECTION_GREEN).unwrap(),
        "68719476736"
    );
    assert_eq!(std::mem::size_of::<Keywords>(), 8);
    if let Ok(path) = std::env::var("RUSTMAGE_TDM_PROTECTION_LAYOUT") {
        std::fs::write(
            path,
            format!(
                "Object={} ObjectView={} State={} DamageSource={}\n",
                std::mem::size_of::<Object>(),
                std::mem::size_of::<ObjectView>(),
                std::mem::size_of::<State>(),
                std::mem::size_of::<DamageSource>()
            ),
        )
        .unwrap();
    }
}

#[test]
fn printed_decayed_and_keyword_counter_are_independent_counterable_attack_abilities() {
    const COUNTER: CardRules = CardRules {
        effect: Effect::Spell(
            TargetPlan::Single(TargetSpec::one(TargetRule::Ability)),
            &[Instruction::Counter],
        ),
        ..CardRules::EMPTY
    };
    let _scope = fixture_rules::activate(&[
        (Card::GrizzlyBears, &DECAYED),
        (Card::from_name("Opt").unwrap(), &COUNTER),
    ]);
    let mut g = scenario(Player::P0);
    let source = put(&mut g, Player::P0, "Grizzly Bears", Zone::Battlefield);
    g.add_counter(source, CounterKind::Decayed, 3);
    declare(&mut g, source);
    assert_eq!(
        decayed_packets(&g),
        2,
        "printed ability plus one ability from interchangeable same-kind counters"
    );
    while matches!(g.state.pending, Pending::OrderTriggers { .. }) {
        choose(&mut g, |a| matches!(a, Action::PutTrigger { .. }));
    }
    let victim = g.state.stack.last().unwrap().id;
    let counter = put(&mut g, Player::P0, "Opt", Zone::Hand);
    cast(&mut g, counter, Some(Target::Stack(victim)));
    settle(&mut g);
    assert_eq!(
        g.observe(Player::P0).delayed_sacrifices.len(),
        1,
        "countering one independent attack ability leaves the other"
    );
    end_combat(&mut g);
    settle(&mut g);
    assert_eq!(g.state.objects[source as usize].zone, Zone::Graveyard);
}

#[test]
fn decayed_multiplicity_counts_only_instances_surviving_layer_six_provider_and_recipient_suppression()
 {
    const PROVIDER: CardRules = CardRules {
        effect: Effect::Creature(2, 2),
        static_groups: &[StaticAbilityGroup {
            line: 0,
            first_layer: AbilityLayer::Ability,
            fragments: &[StaticAbilityFragment::ControlledSubtypeKeywords {
                subtype: "Bear",
                keywords: Keywords::DECAYED,
            }],
        }],
        ..CardRules::EMPTY
    };
    let _scope = fixture_rules::activate(&[
        (Card::GrizzlyBears, &DECAYED),
        (Card::LlanowarElves, &PROVIDER),
        (Card::from_name("Opt").unwrap(), &BLANK),
    ]);
    for mode in 0..4 {
        let mut g = scenario(Player::P0);
        let source = put(&mut g, Player::P0, "Grizzly Bears", Zone::Battlefield);
        let provider = put(&mut g, Player::P0, "Llanowar Elves", Zone::Battlefield);
        g.add_counter(source, CounterKind::Decayed, 2);
        if mode > 0 {
            let recipient = if mode == 1 { provider } else { source };
            let blank = put(&mut g, Player::P0, "Opt", Zone::Hand);
            cast(&mut g, blank, Some(Target::Card(recipient)));
            settle(&mut g);
        }
        if mode == 3 {
            g.add_counter(source, CounterKind::Decayed, 1);
        }
        let world = FundingState::from_game(&g, &goal(Player::P0));
        assert_eq!(
            characteristic_query::CharacteristicQuery::in_world(&world).decayed_instances(source),
            [3, 2, 0, 1][mode]
        );
        if mode == 0 {
            let saved = g.snapshot();
            let movement = g.entry_move(provider, Zone::Graveyard, false);
            let mut visited = false;
            world.visit_entry_moves(vec![movement], true, None, &mut |next| {
                assert_eq!(
                    characteristic_query::CharacteristicQuery::in_world(next)
                        .decayed_instances(source),
                    2,
                    "virtual provider departure removes only its granted instance"
                );
                visited = true;
                false
            });
            assert!(visited);
            assert_eq!(g.snapshot(), saved);
        }
        declare(&mut g, source);
        assert_eq!(
            decayed_packets(&g),
            [3, 2, 0, 1][mode],
            "only surviving printed, static-granted and counter-granted origins count; mode={mode}"
        );
    }
}

#[test]
fn distinct_resolution_and_copiable_decayed_instances_follow_keyword_removal_and_newer_grants() {
    const REMOVE: CardRules = CardRules {
        effect: Effect::Spell(
            TargetPlan::NONE,
            &[Instruction::RemovePermanentGroupKeywords {
                filters: &[BattlefieldFilter {
                    kinds: 1,
                    relation: -1,
                    nonland: false,
                    tapped: None,
                    without: Keywords::NONE,
                    subtype: None,
                    max_mana: u32::MAX,
                }],
                keywords: Keywords::DECAYED,
            }],
        ),
        ..CardRules::EMPTY
    };
    let _scope = fixture_rules::activate(&[
        (Card::GrizzlyBears, &DECAYED),
        (Card::from_name("Opt").unwrap(), &REMOVE),
    ]);
    let mut g = scenario(Player::P0);
    let source = put(&mut g, Player::P0, "Grizzly Bears", Zone::Battlefield);
    let mut recipe = CopyRecipe::new(Card::GrizzlyBears);
    recipe.overrides = Some(CopyOverrides {
        keywords: Keywords::DECAYED,
        ..CopyOverrides::NONE
    });
    g.apply_temporary_copy(g.object_ref(source), recipe, CopyDeadline::EndOfTurn);
    g.add_counter(source, CounterKind::Decayed, 3);
    let origin = g.object_ref(source);
    g.grant_keyword_effects(&[source], origin, Keywords::DECAYED);
    g.grant_keyword_effects(&[source], origin, Keywords::DECAYED);
    assert_eq!(
        g.characteristic_query().decayed_instances(source),
        5,
        "one printed, one copiable exception, one same-kind counter effect, two distinct resolutions"
    );
    let remove = put(&mut g, Player::P0, "Opt", Zone::Hand);
    cast(&mut g, remove, None);
    settle(&mut g);
    assert_eq!(g.characteristic_query().decayed_instances(source), 0);
    g.pay_counters(source, &Counters::of(CounterKind::Decayed, 2));
    assert_eq!(
        g.characteristic_query().decayed_instances(source),
        0,
        "removing only some counters does not refresh an old grant"
    );
    g.grant_keyword_effects(&[source], origin, Keywords::DECAYED);
    assert_eq!(g.characteristic_query().decayed_instances(source), 1);
    g.add_counter(source, CounterKind::Decayed, 1);
    assert_eq!(
        g.characteristic_query().decayed_instances(source),
        2,
        "new same-kind placement refreshes its one grant after removal"
    );
    declare(&mut g, source);
    assert_eq!(decayed_packets(&g), 2);
}
