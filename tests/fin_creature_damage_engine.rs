//! Creature-powered effects use the actual sources and one damage batch.
mod support;
use rustmage as _;
use rustmage_core::cards::authored::{CardExtras, CardRules};
use rustmage_core::{cards::*, engine::*};
use support::*;

const SWEEP: CardRules = CardRules {
    effect: Effect::Spell(
        TargetPlan::Single(TargetSpec::one(TargetRule::creature_you_control())),
        &[Instruction::CreaturePowerDamageBatch {
            source: 0,
            recipients: CreaturePowerDamageRecipients::OtherCreatures,
        }],
    ),
    ..CardRules::EMPTY
};
const SELF_HIT: CardRules = CardRules {
    effect: Effect::Spell(
        TargetPlan::Groups(&[
            TargetSpec::one(TargetRule::creature_you_control()),
            TargetSpec::one(TargetRule::Grouped {
                base: &TargetRule::Any,
                constraint: TargetGroupConstraint::OtherThanGroup(0),
            }),
        ]),
        &[Instruction::CreaturePowerDamageBatch {
            source: 0,
            recipients: CreaturePowerDamageRecipients::TargetGroupAndSelf(1),
        }],
    ),
    ..CardRules::EMPTY
};
const BIRD_GROUP: CardRules = CardRules {
    effect: Effect::Creature(4, 3),
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::SelfEnters,
        targets: TargetPlan::Single(TargetSpec::one(TargetRule::Object {
            zone: TargetZone::Battlefield,
            kinds: 1,
            relation: 1,
            other: false,
            nonland: false,
            tapped: false,
            min_mana: 0,
            max_mana: u32::MAX,
        })),
        program: &[Instruction::PermanentsDealPowerDamage {
            filters: &[BattlefieldFilter {
                kinds: 0,
                relation: 0,
                nonland: false,
                tapped: None,
                without: Keywords::NONE,
                subtype: Some("Bird"),
                max_mana: u32::MAX,
            }],
            exclude_source: true,
        }],
    }],
    ..CardRules::EMPTY
};
const PLUS_ONE_DAMAGE: ReplacementSpec = ReplacementSpec::NoncombatDamageToOpponentPlus(1);
const DAMAGE_BOOSTER: CardRules = CardRules {
    effect: Effect::Permanent,
    static_groups: &[StaticAbilityGroup {
        line: 0,
        first_layer: AbilityLayer::Rule,
        fragments: &[StaticAbilityFragment::Replacement(0, PLUS_ONE_DAMAGE)],
    }],
    extras: &CardExtras {
        replacement_effects: &[PLUS_ONE_DAMAGE],
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};

fn card(name: &str) -> Card {
    Card::from_name(name).unwrap()
}

fn give_priority(g: &mut Game, player: Player) {
    if g.actor() != Some(player) {
        act(g, Action::Pass);
    }
    assert_eq!(g.actor(), Some(player));
}

fn finish_damage(g: &mut Game, target: Option<Target>) {
    for _ in 0..24 {
        if g.state.pending == Pending::Priority
            && g.state.stack.is_empty()
            && g.state.pending_triggers.is_empty()
            && g.state.resolving.is_none()
        {
            return;
        }
        choose(g, |action| match action {
            Action::ChooseTarget { target: found } => Some(*found) == target,
            Action::Pass
            | Action::PutTrigger { .. }
            | Action::FinishTargets
            | Action::FinishStackTargets
            | Action::ChooseReplacement { .. }
            | Action::ChooseDamageEvent { .. } => true,
            _ => false,
        });
    }
    panic!("damage batch did not finish");
}

#[test]
fn legal_creature_hits_all_other_creatures_with_its_live_power_and_lifelink() {
    let _scope = fixture_rules::activate(&[(card("Opt"), &SWEEP)]);
    let player = Player::P1;
    let mut g = scenario(player);
    let source = put(&mut g, player, "Healer's Hawk", Zone::Battlefield);
    let ally = put(&mut g, player, "Bear Cub", Zone::Battlefield);
    let foe = put(
        &mut g,
        player.other(),
        "Scorpion Sentinel",
        Zone::Battlefield,
    );
    let land = put(&mut g, player.other(), "Forest", Zone::Battlefield);
    let spell = put(&mut g, player, "Opt", Zone::Hand);
    let pump = put(&mut g, player.other(), "Giant Growth", Zone::Hand);
    cast(&mut g, spell, Some(Target::Card(source)));
    give_priority(&mut g, player.other());
    cast(&mut g, pump, Some(Target::Card(source)));
    settle(&mut g);
    assert_eq!(g.object_view(ally).zone, Zone::Graveyard);
    assert_eq!(g.object_view(foe).zone, Zone::Graveyard);
    assert_eq!(g.object_view(land).zone, Zone::Battlefield);
    assert_eq!(g.object_view(source).damage, 0);
    assert_eq!(g.state.players[player.index()].life, 28);
}

#[test]
fn self_hit_requires_a_live_source_but_survives_the_other_targets_departure() {
    let _scope = fixture_rules::activate(&[(card("Opt"), &SELF_HIT)]);
    let player = Player::P0;
    for removed_group in [None, Some(0), Some(1)] {
        let mut g = scenario(player);
        let source = put(&mut g, player, "Healer's Hawk", Zone::Battlefield);
        let foe = put(&mut g, player.other(), "Bear Cub", Zone::Battlefield);
        let spell = put(&mut g, player, "Opt", Zone::Hand);
        let bounce = put(&mut g, player.other(), "Unsummon", Zone::Hand);
        act(&mut g, Action::AnnounceCast { card: spell });
        let choose_source = Action::ChooseTarget {
            target: Target::Card(source),
        };
        act(&mut g, choose_source.clone());
        reject(&mut g, choose_source);
        act(
            &mut g,
            Action::ChooseTarget {
                target: Target::Card(foe),
            },
        );
        if g.legal_actions().contains(&Action::FinishTargets) {
            act(&mut g, Action::FinishTargets);
        }
        pay(&mut g);
        if let Some(group) = removed_group {
            let object = if group == 0 { source } else { foe };
            give_priority(&mut g, player.other());
            cast(&mut g, bounce, Some(Target::Card(object)));
            act(&mut g, Action::Pass);
            act(&mut g, Action::Pass);
            assert_eq!(g.object_view(object).zone, Zone::Hand);
            let bounced = g.object_ref(object);
            g.move_object(object, Zone::Battlefield);
            assert_ne!(g.object_ref(object), bounced);
        }
        settle(&mut g);
        assert_eq!(
            g.object_view(source).zone,
            if removed_group == Some(0) {
                Zone::Battlefield
            } else {
                Zone::Graveyard
            },
        );
        assert_eq!(
            g.object_view(foe).damage,
            i128::from(removed_group.is_none())
        );
        assert_eq!(
            g.state.players[player.index()].life,
            20 + match removed_group {
                None => 2,
                Some(0) => 0,
                Some(1) => 1,
                _ => unreachable!(),
            },
        );
    }
}

#[test]
fn matching_other_permanents_use_their_own_power_and_damage_abilities() {
    let _scope = fixture_rules::activate(&[(card("Sazh's Chocobo"), &BIRD_GROUP)]);
    let player = Player::P0;
    let mut g = scenario(player);
    let source = put(&mut g, player, "Sazh's Chocobo", Zone::Hand);
    put(&mut g, player, "Sazh's Chocobo", Zone::Battlefield);
    put(&mut g, player, "Healer's Hawk", Zone::Battlefield);
    put(&mut g, player, "Bear Cub", Zone::Battlefield);
    put(&mut g, player.other(), "Healer's Hawk", Zone::Battlefield);
    let foe = put(
        &mut g,
        player.other(),
        "Scorpion Sentinel",
        Zone::Battlefield,
    );
    g.state.objects[foe as usize].growth.toughness = 3;
    cast(&mut g, source, None);
    finish_damage(&mut g, Some(Target::Card(foe)));
    assert_eq!(g.object_view(foe).damage, 5);
    assert_eq!(g.state.players[player.index()].life, 21);
    assert_eq!(g.state.players[player.other().index()].life, 20);
}

#[test]
fn damage_batch_replacement_choices_branch_and_restore_with_actual_source() {
    let _scope =
        fixture_rules::activate(&[(card("Opt"), &SWEEP), (card("Short Bow"), &DAMAGE_BOOSTER)]);
    let player = Player::P0;
    let mut parent = scenario(player);
    let source = put(&mut parent, player, "Healer's Hawk", Zone::Battlefield);
    let plus = put(&mut parent, player, "Short Bow", Zone::Battlefield);
    let double = put(&mut parent, player, "Twinflame Tyrant", Zone::Battlefield);
    let foe = put(
        &mut parent,
        player.other(),
        "Scorpion Sentinel",
        Zone::Battlefield,
    );
    parent.state.objects[foe as usize].growth.toughness = 3;
    let spell = put(&mut parent, player, "Opt", Zone::Hand);
    cast(&mut parent, spell, Some(Target::Card(source)));
    act(&mut parent, Action::Pass);
    act(&mut parent, Action::Pass);
    assert_eq!(parent.actor(), Some(player.other()));
    let pending_damage = parent
        .observe(player.other())
        .replacement_choice
        .unwrap()
        .damage
        .unwrap();
    assert_eq!(pending_damage.source.reference, parent.object_ref(source));
    assert_eq!(pending_damage.target, Target::Card(foe));
    assert_eq!(pending_damage.amount, 1);
    let plus_first = Action::ChooseReplacement {
        source: parent.object_ref(plus),
        ability: 0,
    };
    let double_first = Action::ChooseReplacement {
        source: parent.object_ref(double),
        ability: 0,
    };
    let actions = parent.legal_actions();
    let index = actions.iter().position(|a| *a == double_first).unwrap();
    assert!(actions.contains(&plus_first));
    let revision = parent.revision();
    let saved = parent.snapshot();
    let mut raw = parent.clone();
    let mut indexed = parent.clone();
    let mut sibling = parent;
    assert_eq!(
        raw.apply(player, double_first.clone()),
        Err(Error::WrongActor),
    );
    assert_eq!(raw.snapshot(), saved);
    act(&mut raw, double_first);
    indexed
        .apply_index(player.other(), revision, index)
        .unwrap();
    act(&mut sibling, plus_first.clone());
    for game in [&mut raw, &mut indexed, &mut sibling] {
        finish_damage(game, None);
    }
    assert_eq!(raw.snapshot(), indexed.snapshot());
    assert_eq!(raw.object_view(foe).damage, 3);
    assert_eq!(sibling.object_view(foe).damage, 4);
    assert_eq!(raw.object_view(double).damage, 1);
    assert_eq!(raw.state.players[player.index()].life, 24);
    assert_eq!(sibling.state.players[player.index()].life, 25);
    indexed.restore(&saved).unwrap();
    assert_eq!(
        indexed.apply_index(player.other(), revision, index),
        Err(Error::StaleRevision),
    );
    act(&mut indexed, plus_first);
    finish_damage(&mut indexed, None);
    assert_eq!(indexed.snapshot(), sibling.snapshot());
}
