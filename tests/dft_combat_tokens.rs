//! Actual token creation gets a source-independent requirement for this combat only.
mod support;
use rustmage::Card;
use rustmage_core::cards::authored::{CardExtras, CardRules};
use rustmage_core::cards::*;
use rustmage_core::engine::*;
use std::sync::OnceLock;
use support::*;

const HASTE: KeywordGrant = KeywordGrant {
    condition: None,
    requires_life_gain: false,
    filter: CreatureFilter {
        relation: 0,
        other: true,
        subtype: Some("Goblin"),
        keyword: Keywords::NONE,
        attacking: false,
        tokens: None,
        artifact: false,
    },
    source_only: false,
    own_turn: false,
    keywords: Keywords::HASTE,
};
const HASTE_GROUP: &[StaticAbilityGroup] = &[StaticAbilityGroup {
    line: 0,
    first_layer: AbilityLayer::Ability,
    fragments: &[StaticAbilityFragment::KeywordGrant(HASTE)],
}];
const HASTE_EXTRAS: CardExtras = CardExtras {
    keyword_grants: &[HASTE],
    ..CardExtras::EMPTY
};
const TAX: CardRules = CardRules {
    effect: Effect::Permanent,
    static_groups: &[StaticAbilityGroup {
        line: 0,
        first_layer: AbilityLayer::Rule,
        fragments: &[StaticAbilityFragment::AttackControllerTax {
            amount: 2,
            condition: StatValue::Fixed(1),
        }],
    }],
    ..CardRules::EMPTY
};
fn heavy() -> &'static CardRules {
    static RULES: OnceLock<CardRules> = OnceLock::new();
    RULES.get_or_init(|| CardRules {
        effect: Effect::Creature(2, 3),
        static_groups: HASTE_GROUP,
        extras: &HASTE_EXTRAS,
        triggers: Box::leak(
            vec![TriggeredAbility {
                condition: TriggerCondition::AtStep {
                    step: TriggerStep::BeginCombat,
                    relation: CastRelation::You,
                    guard: StepGuard::Always,
                },
                targets: TargetPlan::NONE,
                program: Box::leak(
                    vec![Instruction::CreateTokensAttackThisCombat {
                        token: Card::from_name("@token/e58ebd913e8be25509b853b8").unwrap(),
                        amount: 1,
                    }]
                    .into_boxed_slice(),
                ),
            }]
            .into_boxed_slice(),
        ),
        ..CardRules::EMPTY
    })
}
fn begin(g: &mut Game) {
    g.begin_step(Step::BeginCombat);
    g.give_priority();
    settle(g);
}
fn attackers(g: &mut Game) {
    for _ in 0..12 {
        if g.state.pending == Pending::Attackers {
            return;
        }
        act(g, Action::Pass);
    }
    panic!("attack declaration missing");
}
fn created(g: &Game) -> Vec<CardId> {
    g.state
        .objects
        .iter()
        .enumerate()
        .filter(|(_, o)| o.zone == Zone::Battlefield && o.is_token())
        .map(|(id, _)| id as CardId)
        .collect()
}
fn end_combat(g: &mut Game) {
    for _ in 0..64 {
        if g.state.step == Step::Main2 && g.state.pending == Pending::Priority {
            return;
        }
        match g.state.pending {
            Pending::Priority => act(g, Action::Pass),
            Pending::Blockers => act(g, Action::FinishBlockers),
            Pending::Damage => {
                let r = g.damage_assignment_range().unwrap();
                act(
                    g,
                    Action::AssignDamage {
                        target: r.target,
                        amount: r.maximum,
                    },
                );
            }
            _ => panic!("unexpected combat continuation {:?}", g.state.pending),
        }
    }
    panic!("combat failed to end");
}

#[test]
fn live_other_goblin_haste_allows_actual_created_token_and_requirement_forces_it_to_attack() {
    let _scope = fixture_rules::activate(&[(Card::GoblinPiker, heavy())]);
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        let source = put(&mut g, p, "Goblin Piker", Zone::Battlefield);
        let old = put(&mut g, p, "Grizzly Bears", Zone::Battlefield);
        begin(&mut g);
        let token = created(&g)[0];
        assert_eq!(stats(&g, token), (1, 1));
        assert_eq!(g.object_view(token).colors, 8);
        assert!(g.keywords(token).contains(Keywords::HASTE));
        assert!(!g.keywords(source).contains(Keywords::HASTE));
        assert!(!g.attacks_each_combat(old));
        assert_eq!(
            g.observe(p).must_attack_this_combat,
            vec![g.object_ref(token)]
        );
        attackers(&mut g);
        reject(&mut g, Action::FinishAttackers);
        act(&mut g, Action::Attack { card: token });
        act(&mut g, Action::FinishAttackers);
    }
}

#[test]
fn source_departure_before_or_after_creation_leaves_requirement_but_loses_live_haste() {
    let _scope = fixture_rules::activate(&[(Card::GoblinPiker, heavy())]);
    for before in [false, true] {
        let mut g = scenario(Player::P0);
        let source = put(&mut g, Player::P0, "Goblin Piker", Zone::Battlefield);
        g.begin_step(Step::BeginCombat);
        g.give_priority();
        if before {
            g.move_object(source, Zone::Graveyard);
        }
        settle(&mut g);
        let token = created(&g)[0];
        if !before {
            g.move_object(source, Zone::Graveyard);
        }
        assert!(g.attacks_each_combat(token));
        assert!(!g.keywords(token).contains(Keywords::HASTE));
        attackers(&mut g);
        assert!(
            !g.can_declare_attacker(Player::P0, token),
            "summoning sickness remains relevant"
        );
        act(&mut g, Action::FinishAttackers);
    }
}

#[test]
fn doubled_token_creation_binds_every_actual_result_and_requirements_expire_before_next_combat() {
    let _scope = fixture_rules::activate(&[(Card::GoblinPiker, heavy())]);
    let mut g = scenario(Player::P0);
    let source = put(&mut g, Player::P0, "Goblin Piker", Zone::Battlefield);
    put(&mut g, Player::P0, "Doubling Season", Zone::Battlefield);
    begin(&mut g);
    let tokens = created(&g);
    assert_eq!(tokens.len(), 2);
    assert_eq!(g.observe(Player::P0).must_attack_this_combat.len(), 2);
    attackers(&mut g);
    act(&mut g, Action::Attack { card: tokens[0] });
    reject(&mut g, Action::FinishAttackers);
    act(&mut g, Action::Attack { card: tokens[1] });
    act(&mut g, Action::FinishAttackers);
    g.move_object(source, Zone::Graveyard);
    end_combat(&mut g);
    assert!(g.observe(Player::P0).must_attack_this_combat.is_empty());
    for &id in &tokens {
        assert!(!g.attacks_each_combat(id));
        g.state.objects[id as usize].tapped = false;
        g.state.objects[id as usize].summoning_sick = false;
    }
    g.add_combat();
    g.begin_step(Step::BeginCombat);
    g.give_priority();
    attackers(&mut g);
    act(&mut g, Action::FinishAttackers);
}

#[test]
fn combat_requirement_is_not_a_granted_ability_and_does_not_bind_a_reused_token_slot() {
    let _scope = fixture_rules::activate(&[(Card::GoblinPiker, heavy())]);
    let mut g = scenario(Player::P0);
    let source = put(&mut g, Player::P0, "Goblin Piker", Zone::Battlefield);
    begin(&mut g);
    let token = created(&g)[0];
    let original = g.object_ref(token);
    let origin = g.object_ref(source);
    let timestamp = g.next_role_timestamp();
    g.state
        .roles
        .as_mut()
        .unwrap()
        .source_linked_effects
        .push(SourceLinkedEffectView {
            source: origin,
            recipient: original,
            controller: Player::P0,
            kind: SourceLinkedEffectKind::RemoveAbilitiesUntilEndOfTurn,
            timestamp,
        });
    assert!(
        g.attacks_each_combat(token),
        "resolution restriction survives ability removal"
    );
    g.move_object(token, Zone::Graveyard);
    g.cease_token(token);
    let factory = Card::from_name("@token/e58ebd913e8be25509b853b8").unwrap();
    g.create_tokens(Player::P0, factory, 1, false);
    assert_ne!(g.object_ref(token), original);
    assert!(
        !g.attacks_each_combat(token),
        "fresh reuse cannot acquire old requirement"
    );
}

#[test]
fn tapped_token_and_optional_attack_tax_do_not_create_an_impossible_requirement() {
    let _scope = fixture_rules::activate(&[
        (Card::GoblinPiker, heavy()),
        (Card::from_name("Short Bow").unwrap(), &TAX),
    ]);
    for tax in [false, true] {
        let mut g = scenario(Player::P0);
        put(&mut g, Player::P0, "Goblin Piker", Zone::Battlefield);
        if tax {
            put(&mut g, Player::P1, "Short Bow", Zone::Battlefield);
        }
        begin(&mut g);
        let token = created(&g)[0];
        if !tax {
            g.tap_objects(&[token]);
        }
        attackers(&mut g);
        act(&mut g, Action::FinishAttackers);
    }
}

#[test]
fn combat_requirement_and_live_haste_remain_owned_across_raw_index_restore() {
    let _scope = fixture_rules::activate(&[(Card::GoblinPiker, heavy())]);
    let mut g = scenario(Player::P0);
    let source = put(&mut g, Player::P0, "Goblin Piker", Zone::Battlefield);
    begin(&mut g);
    let token = created(&g)[0];
    attackers(&mut g);
    let saved = g.snapshot();
    let mut raw = g.clone();
    let mut indexed = g.clone();
    let action = Action::Attack { card: token };
    let index = indexed
        .legal_actions()
        .iter()
        .position(|a| *a == action)
        .unwrap();
    let rev = indexed.revision();
    act(&mut raw, action);
    indexed.apply_index(Player::P0, rev, index).unwrap();
    assert_eq!(raw.snapshot(), indexed.snapshot());
    indexed.restore(&saved).unwrap();
    indexed.move_object(source, Zone::Graveyard);
    drop(g);
    act(&mut indexed, Action::FinishAttackers);
    act(&mut raw, Action::FinishAttackers);
    assert_eq!(raw.state.attackers.len(), 1);
    assert!(indexed.state.attackers.is_empty());
}
