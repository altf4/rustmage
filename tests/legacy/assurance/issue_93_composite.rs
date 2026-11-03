//! Explicitly synthetic composite loyalty and mana abilities on gated Blossombind.
use super::*;

fn owned_step(g: &mut Game, action: Action) {
    let player = g.actor().unwrap();
    let revision = g.revision();
    let index = g.legal_actions().iter().position(|a| *a == action).unwrap();
    let parent = g.clone();
    let mut indexed = parent.clone();
    drop(parent);
    let checkpoint = indexed.snapshot();
    indexed.apply_index(player, revision, index).unwrap();
    act(g, action);
    assert_eq!(g.snapshot(), indexed.snapshot());
    let expected = indexed.snapshot();
    indexed.restore(&checkpoint).unwrap();
    indexed
        .apply_index(player, indexed.revision(), index)
        .unwrap();
    assert_eq!(indexed.snapshot(), expected);
}

fn owned_pay(g: &mut Game) {
    let action = g
        .legal_actions()
        .into_iter()
        .find(|a| matches!(a, Action::PayCosts { .. }))
        .unwrap();
    owned_step(g, action);
}

fn board(player: Player) -> (Game, CardId, CardId) {
    let mut g = scenario(player);
    let walker = put(&mut g, player, "Bear Cub", Zone::Battlefield);
    g.state.objects[walker as usize].card = Card::JacePlaneswalkerToken;
    g.state.objects[walker as usize].counters.loyalty = 3;
    let object = g.object_ref(walker);
    let timestamp = g.next_role_timestamp();
    g.state
        .roles
        .get_or_insert_with(Default::default)
        .animations
        .push(AnimationView {
            object,
            timestamp,
            artifact: false,
            land: false,
            preserve_card_types: false,
            food_only: false,
            artifact_noncreature: false,
            enchantment_noncreature: false,
            colors: None,
            creature_types: None,
            all_creature_types: false,
            additional_creature_types: false,
            base: Some((3, 3)),
            keywords: Keywords::NONE,
            expires_turn: None,
        });
    let aura = put(&mut g, player, "Pacifism", Zone::Battlefield);
    g.state.objects[aura as usize].card = Card::from_name("Blossombind").unwrap();
    g.state.objects[aura as usize].attached_to = Some(object);
    assert!(replacements::counters_prohibited(&g, object));
    (g, walker, aura)
}

// Keep the targeted positive control at a final cost of one: its synthetic
// target reduction removes one from this raw cost of two.
const TARGETED_POSITIVE_LOYALTY: crate::cards::authored::CardRules =
    crate::cards::authored::CardRules {
        effect: Effect::Permanent,
        activated: &[ActivatedAbility {
            exhaust: false,
            guard: ActivationGuard::Always,
            origin: ActivationOrigin::Battlefield,
            cycling: false,
            cost: ActivationCost {
                generic: 2,
                mana_component: true,
                loyalty: Some(1),
                ..ActivationCost::ZERO
            },
            targets: TargetPlan::Single(TargetSpec::one(TargetRule::Creature)),
            program: &[],
            sorcery: true,
            once_per_turn: false,
        }],
        extras: &crate::cards::authored::CardExtras {
            starting_loyalty: 3,
            ..crate::cards::authored::CardExtras::EMPTY
        },
        ..crate::cards::authored::CardRules::EMPTY
    };

#[test]
fn mana_window_must_remove_prohibition_before_positive_loyalty_payment() {
    let _mana = crate::cards::sacrifice_mana_fixture::Guard::new();
    for target_sensitive in [false, true] {
        let _loyalty = if target_sensitive {
            crate::cards::composite_loyalty_fixture::Guard::with_mana_positive_target()
        } else {
            crate::cards::composite_loyalty_fixture::Guard::with_mana_positive()
        };
        let _positive_target = target_sensitive.then(|| {
            crate::cards::fixture_rules::activate(&[(
                Card::JacePlaneswalkerToken,
                &TARGETED_POSITIVE_LOYALTY,
            )])
        });
        for player in [Player::P0, Player::P1] {
            let (mut g, walker, aura) = board(player);
            let target = put(&mut g, player.other(), "Bear Cub", Zone::Battlefield);
            let announce = Action::AnnounceActivate {
                card: walker,
                ability: 0,
                counter_payment: Counters::default(),
            };
            assert!(g.legal_actions().contains(&announce));
            assert!(!g.legal_actions().iter().any(|a| matches!(a,
            Action::Activate { card, ability: 0, .. }
            | Action::BeginActivate { card, ability: 0, .. } if *card == walker)));
            owned_step(&mut g, announce);
            if target_sensitive {
                owned_step(
                    &mut g,
                    Action::ChooseTarget {
                        target: Target::Card(target),
                    },
                );
                if g.legal_actions().contains(&Action::FinishTargets) {
                    owned_step(&mut g, Action::FinishTargets);
                }
            }
            assert!(g.mana_payment_domain().is_none());
            reject(
                &mut g,
                Action::PayCosts {
                    payment: if target_sensitive {
                        [0; 6]
                    } else {
                        [0, 0, 0, 0, 1, 0]
                    },
                },
            );
            let mana = g.legal_actions().into_iter().find(|a| matches!(a,
            Action::ActivateMana { card, .. } | Action::AnnounceMana { card, .. } if *card == aura
        )).expect("the Aura can sacrifice itself as a mana ability");
            owned_step(&mut g, mana);
            if matches!(g.state.pending, Pending::PayCosts { .. })
                && g.state
                    .announcing
                    .as_ref()
                    .is_some_and(|a| a.production.is_some())
            {
                owned_pay(&mut g);
            }
            assert_eq!(g.object_view(aura).zone, Zone::Graveyard);
            assert!(g.mana_payment_domain().is_some());
            owned_pay(&mut g);
            assert_eq!(g.counter_count(walker, CounterKind::Loyalty), 4);
            assert!(g.object_view(walker).loyalty_activated_this_turn);
        }
    }
}

#[test]
fn mana_in_pool_does_not_pay_a_prohibited_loyalty_cost() {
    let _loyalty = crate::cards::composite_loyalty_fixture::Guard::with_mana_positive();
    for player in [Player::P0, Player::P1] {
        let (mut g, walker, _) = board(player);
        let announce = Action::AnnounceActivate {
            card: walker,
            ability: 0,
            counter_payment: Counters::default(),
        };
        assert!(!g.legal_actions().contains(&announce));
        reject(&mut g, announce);
    }
}

#[test]
fn zero_target_total_cannot_open_a_mana_window_to_remove_a_loyalty_prohibition() {
    let _mana = crate::cards::sacrifice_mana_fixture::Guard::new();
    let _loyalty = crate::cards::composite_loyalty_fixture::Guard::with_mana_positive_target();
    for player in [Player::P0, Player::P1] {
        let (mut g, walker, aura) = board(player);
        g.state.players[player.index()].mana = [0; 6];
        let target = put(&mut g, player.other(), "Bear Cub", Zone::Battlefield);
        let forest = put(&mut g, player, "Forest", Zone::Battlefield);
        let announce = Action::AnnounceActivate {
            card: walker,
            ability: 0,
            counter_payment: Counters::default(),
        };
        assert!(!g.legal_actions().contains(&announce));
        reject(&mut g, announce.clone());
        // Priority independently permits mana abilities. Pay the Aura's
        // sacrifice here, before announcing the zero-total loyalty activation.
        let mana = g.legal_actions().into_iter().find(|action| matches!(action,
            Action::ActivateMana { card, .. } | Action::AnnounceMana { card, .. } if *card == aura
        )).expect("ordinary Priority mana action");
        owned_step(&mut g, mana);
        if matches!(g.state.pending, Pending::PayCosts { .. })
            && g.state
                .announcing
                .as_ref()
                .is_some_and(|a| a.production.is_some())
        {
            owned_pay(&mut g);
        }
        assert_eq!(g.object_view(aura).zone, Zone::Graveyard);
        owned_step(&mut g, announce);
        owned_step(
            &mut g,
            Action::ChooseTarget {
                target: Target::Card(target),
            },
        );
        if g.legal_actions().contains(&Action::FinishTargets) {
            owned_step(&mut g, Action::FinishTargets);
        }
        let late_mana = Action::TapMana { card: forest };
        assert!(!g.legal_actions().contains(&late_mana));
        reject(&mut g, late_mana);
        owned_step(&mut g, Action::PayCosts { payment: [0; 6] });
        assert_eq!(g.counter_count(walker, CounterKind::Loyalty), 4);
        assert_eq!(g.state.players[player.index()].mana, [0, 0, 0, 0, 1, 0]);
        assert!(!g.object_view(forest).tapped);
    }
}
