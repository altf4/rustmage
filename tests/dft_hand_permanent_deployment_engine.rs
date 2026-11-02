//! Existing private deployment extended to one controller and every permanent card type.
mod support;
use rustmage as _;
use rustmage_core::cards::authored::{CardExtras, CardRules};
use rustmage_core::cards::*;
use rustmage_core::engine::*;
use support::*;
const DEPLOY: CardRules = CardRules {
    effect: Effect::Spell(TargetPlan::NONE, &[Instruction::DeployHandPermanents]),
    ..CardRules::EMPTY
};
const EACH: CardRules = CardRules {
    effect: Effect::Spell(TargetPlan::NONE, &[Instruction::DeployHandCreaturesEach]),
    ..CardRules::EMPTY
};
const OBSERVER: CardRules = CardRules {
    effect: Effect::Creature(2, 2),
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::PermanentEntersYou {
            kinds: 1,
            nontoken: false,
            subtype: None,
            include_source: false,
        },
        targets: TargetPlan::NONE,
        program: &[Instruction::PutCounters {
            subject: Subject::Source,
            kind: CounterKind::PlusOne,
            amount: 1,
        }],
    }],
    ..CardRules::EMPTY
};
fn cast_deploy(g: &mut Game, p: Player) {
    let spell = put(g, p, "Opt", Zone::Hand);
    cast(g, spell, None);
    for _ in 0..32 {
        if matches!(g.state.pending, Pending::HandCardChoice { .. }) {
            return;
        }
        choose(g, |a| matches!(a, Action::Pass | Action::PutTrigger { .. }));
    }
    panic!("private deployment not reached");
}
fn select(g: &mut Game, id: CardId) {
    let object = Some(g.object_ref(id));
    act(g, Action::ChooseHandCard { object });
}
fn finish(g: &mut Game) {
    act(g, Action::ChooseHandCard { object: None });
}
fn drain(g: &mut Game) {
    for _ in 0..96 {
        if g.state.pending == Pending::Priority
            && g.state.stack.is_empty()
            && g.state.resolving.is_none()
            && g.state.pending_triggers.is_empty()
        {
            return;
        }
        choose(g, |a| {
            matches!(
                a,
                Action::Pass
                    | Action::PutTrigger { .. }
                    | Action::ChooseReplacement { .. }
                    | Action::ChooseEntryTimestamp { .. }
                    | Action::FinishTargets
                    | Action::FinishStackTargets
            )
        });
    }
    panic!("bounded deployment resolution");
}
#[test]
fn controller_only_private_selection_can_decline_on_an_opponents_turn_without_second_player_phase()
{
    let _scope = fixture_rules::activate(&[(Card::from_name("Opt").unwrap(), &DEPLOY)]);
    let mut g = scenario(Player::P0);
    let own = put(&mut g, Player::P1, "Hill Giant", Zone::Hand);
    let other = put(&mut g, Player::P0, "Grizzly Bears", Zone::Hand);
    g.state.priority = Player::P1;
    cast_deploy(&mut g, Player::P1);
    assert_eq!(g.actor(), Some(Player::P1));
    let view = g.observe(Player::P1).hand_choice.unwrap();
    assert_eq!(view.kind, HandChoiceKind::DeployPermanents);
    assert!(view.candidates.contains(&g.object_ref(own)));
    assert!(!view.candidates.contains(&g.object_ref(other)));
    assert!(g.observe(Player::P0).hand_choice.is_none());
    finish(&mut g);
    drain(&mut g);
    assert_eq!(g.state.objects[own as usize].zone, Zone::Hand);
    assert_eq!(g.state.objects[other as usize].zone, Zone::Hand);
    assert!(!matches!(g.state.pending, Pending::HandCardChoice { .. }));
    println!(
        "HandDeployment={} HandChoiceKind={}",
        std::mem::size_of::<library_hand_exchange::HandDeployment>(),
        std::mem::size_of::<HandChoiceKind>()
    );
}
#[test]
fn simultaneous_selected_creatures_observe_each_other_and_partial_private_choices_do_not_move() {
    let _scope = fixture_rules::activate(&[
        (Card::from_name("Opt").unwrap(), &DEPLOY),
        (Card::GrizzlyBears, &OBSERVER),
    ]);
    let mut g = scenario(Player::P0);
    let first = put(&mut g, Player::P0, "Grizzly Bears", Zone::Hand);
    let second = put(&mut g, Player::P0, "Grizzly Bears", Zone::Hand);
    cast_deploy(&mut g, Player::P0);
    select(&mut g, first);
    assert_eq!(g.state.objects[first as usize].zone, Zone::Hand);
    assert_eq!(g.state.objects[second as usize].zone, Zone::Hand);
    assert!(g.state.pending_triggers.is_empty());
    let hidden = g.observe(Player::P1);
    assert!(hidden.hand_choice.is_none());
    assert!(!hidden.known_hand.contains(&g.object_ref(first)));
    assert_eq!(
        hidden.hand_deployment.unwrap().selected_counts,
        [None, None]
    );
    select(&mut g, second);
    assert_eq!(g.state.objects[first as usize].zone, Zone::Hand);
    finish(&mut g);
    drain(&mut g);
    for id in [first, second] {
        assert_eq!(g.state.objects[id as usize].zone, Zone::Battlefield);
        assert_eq!(g.counter_count(id, CounterKind::PlusOne), 1);
    }
}
#[test]
fn every_available_permanent_card_kind_enters_together_and_the_cards_were_not_cast() {
    let _scope = fixture_rules::activate(&[(Card::from_name("Opt").unwrap(), &DEPLOY)]);
    let mut g = scenario(Player::P0);
    let names = [
        "Hill Giant",
        "Swiftfoot Boots",
        "Doubling Season",
        "Forest",
        "Ajani, Caller of the Pride",
    ];
    let cards: Vec<_> = names
        .iter()
        .map(|n| put(&mut g, Player::P0, n, Zone::Hand))
        .collect();
    let bad = put(&mut g, Player::P0, "Lightning Strike", Zone::Hand);
    let foreign = put(&mut g, Player::P1, "Hill Giant", Zone::Hand);
    cast_deploy(&mut g, Player::P0);
    for id in [bad, foreign] {
        let object = Some(g.object_ref(id));
        reject(&mut g, Action::ChooseHandCard { object });
    }
    for id in &cards {
        select(&mut g, *id);
    }
    let duplicate = Some(g.object_ref(cards[0]));
    reject(&mut g, Action::ChooseHandCard { object: duplicate });
    finish(&mut g);
    drain(&mut g);
    for id in &cards {
        assert_eq!(g.state.objects[*id as usize].zone, Zone::Battlefield);
        assert_eq!(g.state.objects[*id as usize].controller, Player::P0);
    }
    assert_eq!(
        g.counter_count(cards[4], CounterKind::Loyalty),
        4,
        "new Doubling Season is not present before the simultaneous entry"
    );
    assert_eq!(
        g.state.players[0].spells_cast_this_turn, 1,
        "only the original resolving spell was cast"
    );
}
#[test]
fn aura_choices_use_existing_battlefield_objects_and_never_simultaneous_incoming_creatures() {
    let _scope = fixture_rules::activate(&[(Card::from_name("Opt").unwrap(), &DEPLOY)]);
    for existing in [false, true] {
        let mut g = scenario(Player::P0);
        let recipient = existing.then(|| put(&mut g, Player::P1, "Hill Giant", Zone::Battlefield));
        if let Some(id) = recipient {
            g.state.objects[id as usize].gained_keywords = Keywords::HEXPROOF;
        }
        let creature = put(&mut g, Player::P0, "Grizzly Bears", Zone::Hand);
        let aura = put(&mut g, Player::P0, "Pacifism", Zone::Hand);
        cast_deploy(&mut g, Player::P0);
        select(&mut g, creature);
        select(&mut g, aura);
        finish(&mut g);
        if let Some(recipient) = recipient {
            assert!(matches!(g.state.pending, Pending::AuraEntry { .. }));
            assert!(g.observe(Player::P0).hand_choice.is_none());
            assert!(g.observe(Player::P1).hand_choice.is_none());
            let incoming = g.object_ref(creature);
            reject(&mut g, Action::ChooseAuraAttachment { object: incoming });
            let object = g.object_ref(recipient);
            act(&mut g, Action::ChooseAuraAttachment { object });
            drain(&mut g);
            assert_eq!(
                g.state.objects[aura as usize].attached_to,
                Some(g.object_ref(recipient)),
                "non-target entry does not care about hexproof"
            );
        } else {
            drain(&mut g);
            assert_eq!(g.state.objects[aura as usize].zone, Zone::Hand);
        }
        assert_eq!(g.state.objects[creature as usize].zone, Zone::Battlefield);
    }
}
#[test]
fn as_entry_copy_uses_existing_blueprints_and_never_other_incoming_selected_objects() {
    const COPIER: CardRules = CardRules {
        effect: Effect::Creature(0, 1),
        extras: &CardExtras {
            entry_copy: Some(EntryCopySpec {
                filters: &[BattlefieldFilter {
                    kinds: 17,
                    relation: 0,
                    nonland: false,
                    tapped: None,
                    without: Keywords::NONE,
                    subtype: None,
                    max_mana: u32::MAX,
                }],
                optional: true,
            }),
            ..CardExtras::EMPTY
        },
        ..CardRules::EMPTY
    };
    let _scope = fixture_rules::activate(&[
        (Card::from_name("Opt").unwrap(), &DEPLOY),
        (Card::GrizzlyBears, &COPIER),
    ]);
    let mut g = scenario(Player::P0);
    let donor = put(&mut g, Player::P0, "Serra Angel", Zone::Battlefield);
    let first = put(&mut g, Player::P0, "Grizzly Bears", Zone::Hand);
    let second = put(&mut g, Player::P0, "Grizzly Bears", Zone::Hand);
    cast_deploy(&mut g, Player::P0);
    select(&mut g, first);
    select(&mut g, second);
    finish(&mut g);
    for _ in 0..2 {
        let view = g.observe(Player::P0).entry_copy.unwrap();
        assert!(view.candidates.contains(&g.object_ref(donor)));
        assert!(
            !view.candidates.contains(&g.object_ref(first))
                && !view.candidates.contains(&g.object_ref(second))
        );
        assert!(g.observe(Player::P1).hand_choice.is_none());
        let object = Some(g.object_ref(donor));
        act(&mut g, Action::ChooseEntryCopy { object });
    }
    drain(&mut g);
    for id in [first, second] {
        assert_eq!(stats(&g, id), (4, 4));
        assert!(
            g.keywords(id)
                .contains(Keywords::FLYING.union(Keywords::VIGILANCE))
        );
        assert!(!g.state.objects[id as usize].is_token());
    }
}
struct ResetPrevention;
impl Drop for ResetPrevention {
    fn drop(&mut self) {
        zone_replacements::TEST_PREVENT_ZONE_MOVE.with(|v| v.set(None));
    }
}
#[test]
fn a_prevented_entry_does_not_consume_other_selected_cards_or_rebind_a_stale_selection() {
    let _scope = fixture_rules::activate(&[(Card::from_name("Opt").unwrap(), &DEPLOY)]);
    let _reset = ResetPrevention;
    let mut g = scenario(Player::P0);
    let prevented = put(&mut g, Player::P0, "Hill Giant", Zone::Hand);
    let success = put(&mut g, Player::P0, "Grizzly Bears", Zone::Hand);
    let stale = put(&mut g, Player::P0, "Forest", Zone::Hand);
    cast_deploy(&mut g, Player::P0);
    let old = g.object_ref(stale);
    g.move_object(stale, Zone::Exile);
    g.move_object(stale, Zone::Hand);
    reject(&mut g, Action::ChooseHandCard { object: Some(old) });
    select(&mut g, prevented);
    select(&mut g, success);
    let old = g.object_ref(prevented);
    zone_replacements::TEST_PREVENT_ZONE_MOVE.with(|p| p.set(Some(old)));
    g.state.zone_rules.get_or_insert_default();
    finish(&mut g);
    drain(&mut g);
    assert_eq!(g.object_ref(prevented), old);
    assert_eq!(g.state.objects[prevented as usize].zone, Zone::Hand);
    assert_eq!(g.state.objects[success as usize].zone, Zone::Battlefield);
    assert_eq!(g.state.objects[stale as usize].zone, Zone::Hand);
}
#[test]
fn owned_private_selection_can_finish_independently_after_parent_drop_raw_index_and_restore() {
    let _scope = fixture_rules::activate(&[(Card::from_name("Opt").unwrap(), &DEPLOY)]);
    let mut parent = scenario(Player::P0);
    let first = put(&mut parent, Player::P0, "Grizzly Bears", Zone::Hand);
    let second = put(&mut parent, Player::P0, "Forest", Zone::Hand);
    cast_deploy(&mut parent, Player::P0);
    select(&mut parent, first);
    let saved = parent.snapshot();
    let action = Action::ChooseHandCard {
        object: Some(parent.object_ref(second)),
    };
    let mut raw = parent.clone();
    let mut indexed = parent.clone();
    let mut other = parent.clone();
    let index = indexed
        .legal_actions()
        .iter()
        .position(|a| *a == action)
        .unwrap();
    let actor = indexed.actor().unwrap();
    let revision = indexed.revision();
    act(&mut raw, action);
    indexed.apply_index(actor, revision, index).unwrap();
    assert_eq!(raw.snapshot(), indexed.snapshot());
    drop(parent);
    finish(&mut raw);
    finish(&mut indexed);
    drain(&mut raw);
    drain(&mut indexed);
    assert_eq!(raw.snapshot(), indexed.snapshot());
    finish(&mut other);
    drain(&mut other);
    assert_eq!(other.state.objects[first as usize].zone, Zone::Battlefield);
    assert_eq!(other.state.objects[second as usize].zone, Zone::Hand);
    indexed.restore(&saved).unwrap();
    assert_eq!(
        indexed
            .observe(Player::P0)
            .hand_choice
            .unwrap()
            .selected
            .len(),
        1
    );
    assert!(indexed.observe(Player::P1).hand_choice.is_none());
}
#[test]
fn completed_deployment_metadata_does_not_intercept_a_following_positive_hand_selector() {
    const FOLLOW: CardRules = CardRules {
        effect: Effect::Spell(
            TargetPlan::Single(TargetSpec::one(TargetRule::Player {
                opponent_only: true,
            })),
            &[
                Instruction::DeployHandPermanents,
                Instruction::RevealTargetHandChoose {
                    kinds: 1,
                    exile: true,
                },
            ],
        ),
        ..CardRules::EMPTY
    };
    let _scope = fixture_rules::activate(&[(Card::from_name("Opt").unwrap(), &FOLLOW)]);
    let mut g = scenario(Player::P0);
    let own = put(&mut g, Player::P0, "Forest", Zone::Hand);
    let enemy = put(&mut g, Player::P1, "Hill Giant", Zone::Hand);
    let spell = put(&mut g, Player::P0, "Opt", Zone::Hand);
    cast(&mut g, spell, Some(Target::Player(Player::P1)));
    for _ in 0..24 {
        if matches!(g.state.pending, Pending::HandCardChoice { .. }) {
            break;
        }
        choose(&mut g, |a| matches!(a, Action::Pass));
    }
    select(&mut g, own);
    finish(&mut g);
    let view = g.observe(Player::P0).hand_choice.unwrap();
    assert_eq!(view.kind, HandChoiceKind::ExileRevealedKinds { kinds: 1 });
    assert!(view.candidates.contains(&g.object_ref(enemy)));
    select(&mut g, enemy);
    drain(&mut g);
    assert_eq!(g.state.objects[enemy as usize].zone, Zone::Exile);
    assert_eq!(g.state.objects[own as usize].zone, Zone::Battlefield);
}
#[test]
fn inherited_each_player_creature_deployment_keeps_apnap_order_and_simultaneous_entries() {
    let _scope = fixture_rules::activate(&[(Card::from_name("Opt").unwrap(), &EACH)]);
    let mut g = scenario(Player::P0);
    let first = put(&mut g, Player::P0, "Grizzly Bears", Zone::Hand);
    let second = put(&mut g, Player::P1, "Hill Giant", Zone::Hand);
    let artifact = put(&mut g, Player::P0, "Swiftfoot Boots", Zone::Hand);
    cast_deploy(&mut g, Player::P0);
    assert_eq!(g.actor(), Some(Player::P0));
    let object = Some(g.object_ref(artifact));
    reject(&mut g, Action::ChooseHandCard { object });
    select(&mut g, first);
    finish(&mut g);
    assert_eq!(g.actor(), Some(Player::P1));
    assert_eq!(g.state.objects[first as usize].zone, Zone::Hand);
    assert!(g.observe(Player::P0).hand_choice.is_none());
    select(&mut g, second);
    finish(&mut g);
    drain(&mut g);
    for id in [first, second] {
        assert_eq!(g.state.objects[id as usize].zone, Zone::Battlefield);
    }
    assert_eq!(g.state.objects[artifact as usize].zone, Zone::Hand);
}

#[test]
fn noncast_hand_entrants_do_not_inherit_a_resolving_parent_spells_chosen_x() {
    const VARIABLE: CardRules = CardRules {
        extras: &CardExtras {
            x_symbols: 1,
            ..CardExtras::EMPTY
        },
        ..DEPLOY
    };
    let _scope = fixture_rules::activate(&[(Card::from_name("Opt").unwrap(), &VARIABLE)]);
    let mut g = scenario(Player::P0);
    let permanent = put(&mut g, Player::P0, "Pterafractyl", Zone::Hand);
    let spell = put(&mut g, Player::P0, "Opt", Zone::Hand);
    act(&mut g, Action::AnnounceCast { card: spell });
    act(&mut g, Action::ChooseX { value: 3 });
    pay(&mut g);
    for _ in 0..24 {
        if matches!(g.state.pending, Pending::HandCardChoice { .. }) {
            break;
        }
        choose(&mut g, |a| matches!(a, Action::Pass));
    }
    select(&mut g, permanent);
    finish(&mut g);
    drain(&mut g);
    assert_eq!(g.state.objects[permanent as usize].zone, Zone::Graveyard);
    assert_eq!(g.counter_count(permanent, CounterKind::PlusOne), 0);
    // Its printed 1/0 dies at X=0; inheriting the parent's 3 would leave a 4/3 survivor.
}
