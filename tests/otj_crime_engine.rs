//! Crime events are committed once per qualifying spell or ability.
mod support;

use rustmage as _;
use rustmage_core::cards::authored::{CardExtras, CardRules};
use rustmage_core::{cards::*, engine::*};
use support::*;

const EACH: CardRules = CardRules {
    effect: Effect::Creature(3, 3),
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::CommittedCrime,
        targets: TargetPlan::NONE,
        program: &[Instruction::GainLife(PlayerSet::Controller, 1)],
    }],
    ..CardRules::EMPTY
};

const ONCE: CardRules = CardRules {
    effect: Effect::Creature(2, 2),
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::OncePerTurn(&TriggerCondition::CommittedCrime),
        targets: TargetPlan::NONE,
        program: &[Instruction::GainLife(PlayerSet::Controller, 2)],
    }],
    ..CardRules::EMPTY
};

const GRAVE: CardRules = CardRules {
    effect: Effect::Creature(1, 1),
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::InZone {
            zone: TargetZone::Graveyard,
            event: &TriggerCondition::CommittedCrime,
        },
        targets: TargetPlan::NONE,
        program: &[Instruction::GainLife(PlayerSet::Controller, 4)],
    }],
    ..CardRules::EMPTY
};

const TWO_TARGETS: CardRules = CardRules {
    effect: Effect::Spell(
        TargetPlan::Single(TargetSpec {
            rule: TargetRule::Any,
            min: 2,
            max: 2,
        }),
        &[Instruction::Damage(1)],
    ),
    ..CardRules::EMPTY
};

fn card(name: &str) -> Card {
    Card::from_name(name).unwrap()
}

fn quiet(g: &mut Game) {
    for _ in 0..64 {
        if g.state.pending == Pending::Priority
            && g.state.stack.is_empty()
            && g.state.pending_triggers.is_empty()
        {
            return;
        }
        choose(g, |a| matches!(a, Action::Pass | Action::PutTrigger { .. }));
    }
    panic!("crime triggers did not settle");
}

#[test]
fn two_targets_are_one_crime_and_later_items_reach_unlimited_and_graveyard_observers() {
    let _rules = fixture_rules::activate(&[
        (card("Hill Giant"), &EACH),
        (card("Grizzly Bears"), &ONCE),
        (card("Healer's Hawk"), &GRAVE),
        (card("Shock"), &TWO_TARGETS),
    ]);
    let p = Player::P0;
    let mut g = scenario(p);
    put(&mut g, p, "Hill Giant", Zone::Battlefield);
    let once = put(&mut g, p, "Grizzly Bears", Zone::Battlefield);
    let grave = put(&mut g, p, "Healer's Hawk", Zone::Graveyard);
    put(&mut g, p, "Healer's Hawk", Zone::Battlefield);
    let opposing_giant = put(&mut g, p.other(), "Hill Giant", Zone::Battlefield);
    let shock = put(&mut g, p, "Shock", Zone::Hand);
    let land = put(&mut g, p, "Abraded Bluffs", Zone::Hand);

    act(&mut g, Action::AnnounceCast { card: shock });
    act(
        &mut g,
        Action::ChooseTarget {
            target: Target::Player(p.other()),
        },
    );
    assert!(!g.state.players[p.index()].committed_crime_this_turn);
    reject(
        &mut g,
        Action::ChooseTarget {
            target: Target::Player(p.other()),
        },
    );
    act(
        &mut g,
        Action::ChooseTarget {
            target: Target::Card(opposing_giant),
        },
    );
    if g.legal_actions().contains(&Action::FinishTargets) {
        act(&mut g, Action::FinishTargets);
    }
    assert!(!g.state.players[p.index()].committed_crime_this_turn);

    let actions = g.legal_actions();
    let index = actions
        .iter()
        .position(|a| matches!(a, Action::PayCosts { .. }))
        .unwrap();
    let revision = g.revision();
    let saved = g.snapshot();
    let mut raw = g.clone();
    let mut indexed = g;
    act(&mut raw, actions[index].clone());
    indexed.apply_index(p, revision, index).unwrap();
    quiet(&mut raw);
    quiet(&mut indexed);
    assert_eq!(raw.snapshot(), indexed.snapshot());
    assert_eq!(raw.state.players[p.index()].life, 27);
    assert_eq!(raw.state.players[p.other().index()].life, 19);

    indexed.restore(&saved).unwrap();
    assert!(!indexed.state.players[p.index()].committed_crime_this_turn);
    assert_eq!(indexed.state.players[p.index()].life, 20);
    assert_eq!(
        indexed.apply_index(p, revision, index),
        Err(Error::StaleRevision)
    );

    act(&mut raw, Action::PlayLand { card: land });
    choose(&mut raw, |a| {
        *a == Action::ChooseTarget {
            target: Target::Player(p.other()),
        }
    });
    quiet(&mut raw);
    assert_eq!(raw.state.players[p.index()].life, 32);
    assert_eq!(raw.state.players[p.other().index()].life, 18);

    // Reset the observer's incarnation and remove the graveyard observer.
    raw.move_object(once, Zone::Exile);
    raw.move_object(once, Zone::Battlefield);
    raw.move_object(grave, Zone::Exile);
    let duelist = put(&mut raw, p, "Deadeye Duelist", Zone::Battlefield);
    act(
        &mut raw,
        Action::AnnounceActivate {
            card: duelist,
            ability: 0,
            counter_payment: Default::default(),
        },
    );
    choose(&mut raw, |a| {
        *a == Action::ChooseTarget {
            target: Target::Player(p.other()),
        }
    });
    pay(&mut raw);
    quiet(&mut raw);
    assert_eq!(raw.state.players[p.index()].life, 35);
    assert_eq!(raw.state.players[p.other().index()].life, 17);
}

const COPY_OWN_SPELL: CardRules = CardRules {
    effect: Effect::Spell(
        TargetPlan::Single(TargetSpec::one(TargetRule::YourSpell)),
        &[Instruction::CopyTargetSpell {
            group: 0,
            retarget: false,
            exceptions: CopyExceptions::NONE,
        }],
    ),
    ..CardRules::EMPTY
};

#[test]
fn a_copy_of_a_qualifying_spell_does_not_commit_another_crime() {
    let _rules =
        fixture_rules::activate(&[(card("Hill Giant"), &EACH), (card("Opt"), &COPY_OWN_SPELL)]);
    let p = Player::P0;
    let mut g = scenario(p);
    put(&mut g, p, "Hill Giant", Zone::Battlefield);
    let shock = put(&mut g, p, "Shock", Zone::Hand);
    let copy = put(&mut g, p, "Opt", Zone::Hand);
    cast(&mut g, shock, Some(Target::Player(p.other())));
    cast(&mut g, copy, Some(Target::Spell(shock)));
    quiet(&mut g);
    assert_eq!(g.state.players[p.index()].life, 21);
    assert_eq!(g.state.players[p.other().index()].life, 16);
    assert_eq!(g.state.players[p.index()].spells_cast_this_turn, 2);
}

const OUTLAW_CAST: CardRules = CardRules {
    effect: Effect::Creature(3, 3),
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::SpellCast {
            relation: CastRelation::You,
            filter: CastFilter::Outlaw,
            ordinal: None,
        },
        targets: TargetPlan::NONE,
        program: &[Instruction::GainLife(PlayerSet::Controller, 1)],
    }],
    ..CardRules::EMPTY
};

#[test]
fn outlaw_cast_filter_counts_changeling_once_and_rejects_unrelated_subtypes() {
    let _rules = fixture_rules::activate(&[(card("Hill Giant"), &OUTLAW_CAST)]);
    let p = Player::P0;
    let mut g = scenario(p);
    put(&mut g, p, "Hill Giant", Zone::Battlefield);
    for (name, life) in [
        ("Ankle Biter", 20),
        ("Deadeye Duelist", 21),
        ("Three Tree Mascot", 22),
    ] {
        let creature = put(&mut g, p, name, Zone::Hand);
        cast(&mut g, creature, None);
        quiet(&mut g);
        assert_eq!(g.state.players[p.index()].life, life, "{name}");
    }
}

const SACRIFICE_OBSERVER: CardRules = CardRules {
    effect: Effect::Creature(3, 3),
    activated: &[ActivatedAbility {
        exhaust: false,
        guard: ActivationGuard::Always,
        cycling: false,
        origin: ActivationOrigin::Battlefield,
        cost: ActivationCost {
            generic: 1,
            mana_component: true,
            additional: Some(AdditionalCost {
                kind: EffectChoiceKind::Sacrifice,
                filter: SacrificeFilter {
                    keywords: Keywords::NONE,
                    source_only: false,
                    other: true,
                    kinds: 1,
                    nonland: false,
                    tokens: None,
                    artifact_enchantment_or_token: false,
                    subtype: None,
                },
            }),
            ..ActivationCost::ZERO
        },
        targets: TargetPlan::NONE,
        program: &[
            Instruction::IfSacrificedOutlaw { skip: 1 },
            Instruction::GainLife(PlayerSet::Controller, 1),
        ],
        sorcery: false,
        once_per_turn: false,
    }],
    ..CardRules::EMPTY
};

#[test]
fn outlaw_sacrifice_uses_the_exact_cost_object_even_when_death_is_redirected() {
    let _rules = fixture_rules::activate(&[(card("Hill Giant"), &SACRIFICE_OBSERVER)]);
    let p = Player::P0;
    let mut g = scenario(p);
    let observer = put(&mut g, p, "Hill Giant", Zone::Battlefield);
    put(&mut g, p.other(), "Rest in Peace", Zone::Battlefield);
    for (name, life) in [("Ankle Biter", 20), ("Three Tree Mascot", 21)] {
        let creature = put(&mut g, p, name, Zone::Battlefield);
        act(
            &mut g,
            Action::AnnounceActivate {
                card: observer,
                ability: 0,
                counter_payment: Default::default(),
            },
        );
        let object = g.object_ref(creature);
        act(&mut g, Action::ChooseAdditionalCost { object });
        pay(&mut g);
        quiet(&mut g);
        assert_eq!(g.state.players[p.index()].life, life, "{name}");
        assert_eq!(g.object_view(creature).zone, Zone::Exile);
    }
}

const CRIME_FLASH: CardRules = CardRules {
    effect: Effect::Spell(
        TargetPlan::Single(TargetSpec::one(TargetRule::Creature)),
        &[Instruction::Damage(1)],
    ),
    extras: &CardExtras {
        self_cast_timing: Some(SelfCastTiming::FlashAfterCrime),
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};

#[test]
fn conditional_flash_checks_the_caster_before_announcement_and_follows_the_spell_on_stack() {
    let _rules = fixture_rules::activate(&[(Card::Divination, &CRIME_FLASH)]);
    for player in [Player::P0, Player::P1] {
        let opponent = player.other();
        let mut game = scenario(opponent);
        let ours = put(&mut game, player, "Hill Giant", Zone::Battlefield);
        let theirs = put(&mut game, opponent, "Hill Giant", Zone::Battlefield);
        let hand = put(&mut game, player, "Divination", Zone::Hand);
        let exiled = put(&mut game, opponent, "Divination", Zone::Exile);
        let opponent_shock = put(&mut game, opponent, "Shock", Zone::Hand);
        let friendly_shock = put(&mut game, player, "Shock", Zone::Hand);
        let hostile_shock = put(&mut game, player, "Shock", Zone::Hand);
        game.grant_exile_play_permission(player, game.object_ref(exiled), false);
        let announce_hand = Action::AnnounceCast { card: hand };
        let announce_exile = Action::AnnounceExile { card: exiled };

        cast(&mut game, opponent_shock, Some(Target::Player(player)));
        act(&mut game, Action::Pass);
        assert_eq!(game.actor(), Some(player));
        // The exiled card's owner committed a crime. Its prospective caster has not.
        assert!(game.state.players[opponent.index()].committed_crime_this_turn);
        reject(&mut game, announce_hand.clone());
        reject(&mut game, announce_exile.clone());
        assert!(
            !game
                .legal_actions()
                .iter()
                .any(|action| { matches!(action, Action::Cast { card, .. } if *card == hand) })
        );

        cast(&mut game, friendly_shock, Some(Target::Card(ours)));
        assert!(!game.state.players[player.index()].committed_crime_this_turn);
        reject(&mut game, announce_hand);
        reject(&mut game, announce_exile.clone());

        // Committing the hostile cast is sufficient while it is still on the stack.
        cast(&mut game, hostile_shock, Some(Target::Player(opponent)));
        assert!(game.state.players[player.index()].committed_crime_this_turn);
        assert!(!game.keywords(hand).contains(Keywords::FLASH));
        assert!(!game.keywords(exiled).contains(Keywords::FLASH));
        let mut exile_branch = game.clone();
        act(&mut exile_branch, announce_exile);
        act(
            &mut exile_branch,
            Action::ChooseTarget {
                target: Target::Card(theirs),
            },
        );
        pay(&mut exile_branch);
        let spell = exile_branch.object_view(exiled);
        assert_eq!(spell.zone, Zone::Stack);
        assert_eq!(spell.controller, player);
        assert_eq!(spell.owner, opponent);
        assert!(spell.keywords.contains(Keywords::FLASH));

        cast(&mut game, hand, Some(Target::Card(theirs)));
        assert!(game.keywords(hand).contains(Keywords::FLASH));
        settle(&mut game);
        assert!(!game.keywords(hand).contains(Keywords::FLASH));

        game.begin_step(Step::End);
        act(&mut game, Action::Pass);
        act(&mut game, Action::Pass);
        assert_eq!(game.state.active, player);
        assert!(!game.state.players[player.index()].committed_crime_this_turn);
        game.begin_step(Step::BeginCombat);
        game.state.players[player.index()].mana = [8; 6];
        let second = put(&mut game, player, "Divination", Zone::Hand);
        reject(&mut game, Action::AnnounceCast { card: second });
    }
}
