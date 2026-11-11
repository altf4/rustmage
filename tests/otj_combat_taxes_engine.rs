//! Scoped attacker taxes and the owned declaration payment used for blockers.
mod support;
use rustmage::Card;
use rustmage_core::cards::authored::CardRules;
use rustmage_core::cards::*;
use rustmage_core::engine::*;
use support::*;

const TAX: CardRules = CardRules {
    effect: Effect::Creature(2, 2),
    static_groups: &[StaticAbilityGroup {
        line: 0,
        first_layer: AbilityLayer::Rule,
        fragments: &[
            StaticAbilityFragment::AttackControllerOrPlaneswalkerTax {
                amount: 1,
                condition: StatValue::SourceUntapped,
            },
            StaticAbilityFragment::BlockTaxWhileAttacking(1),
        ],
    }],
    ..CardRules::EMPTY
};
const MUST_ATTACK: CardRules = CardRules {
    effect: Effect::Creature(2, 2),
    static_groups: &[StaticAbilityGroup {
        line: 0,
        first_layer: AbilityLayer::Rule,
        fragments: &[StaticAbilityFragment::AttacksEachCombat],
    }],
    ..CardRules::EMPTY
};
const MUST_BE_BLOCKED: CardRules = CardRules {
    effect: Effect::Creature(2, 2),
    static_groups: &[StaticAbilityGroup {
        line: 0,
        first_layer: AbilityLayer::Rule,
        fragments: &[StaticAbilityFragment::MustBeBlocked],
    }],
    ..CardRules::EMPTY
};

fn attackers(g: &mut Game) {
    g.state.step = Step::Attackers;
    g.state.pending = Pending::Attackers;
}

fn blockers(g: &mut Game, cards: &[CardId]) {
    attackers(g);
    for &card in cards {
        act(g, Action::Attack { card });
    }
    act(g, Action::FinishAttackers);
    act(g, Action::Pass);
    act(g, Action::Pass);
    assert_eq!(g.state.pending, Pending::Blockers);
}

const FEROCIOUS: CardRules = CardRules {
    effect: Effect::Creature(3, 3),
    keywords: Keywords::DEFENDER,
    static_groups: &[StaticAbilityGroup {
        line: 1,
        first_layer: AbilityLayer::Rule,
        fragments: &[StaticAbilityFragment::AttackWithoutDefenderWhen(
            StepGuard::Threshold {
                value: StepValue::GreatestCreaturePower,
                minimum: Some(4),
                maximum: None,
            },
        )],
    }],
    ..CardRules::EMPTY
};

#[test]
fn a_live_power_threshold_waives_defender_without_removing_it() {
    let _rules = fixture_rules::activate(&[(Card::GrizzlyBears, &FEROCIOUS)]);
    let player = Player::P1;
    let mut game = scenario(player);
    let defender = put(&mut game, player, "Grizzly Bears", Zone::Battlefield);
    let large = put(
        &mut game,
        player.other(),
        "Daring Thunder-Thief",
        Zone::Battlefield,
    );
    attackers(&mut game);
    let attack = Action::Attack { card: defender };
    reject(&mut game, attack.clone());

    let mut own_power = game.clone();
    own_power.add_counter(defender, CounterKind::PlusOne, 1);
    assert!(own_power.keywords(defender).contains(Keywords::DEFENDER));
    assert!(own_power.legal_actions().contains(&attack));

    let large_ref = game.object_ref(large);
    game.record_control(large_ref, player, None);
    act(&mut game, attack);
    act(&mut game, Action::FinishAttackers);
    game.move_object(large, Zone::Graveyard);
    assert!(
        game.state
            .attackers
            .iter()
            .any(|attack| attack.card == defender)
    );
    assert!(game.keywords(defender).contains(Keywords::DEFENDER));
}

#[test]
fn attack_tax_stacks_for_the_player_and_their_walkers_and_excludes_battles() {
    let _rules = fixture_rules::activate(&[
        (Card::GrizzlyBears, &TAX),
        (Card::from_name("Bear Cub").unwrap(), &MUST_ATTACK),
    ]);
    for player in [Player::P0, Player::P1] {
        let mut base = scenario(player);
        let creature = put(&mut base, player, "Bear Cub", Zone::Battlefield);
        let source = put(
            &mut base,
            player.other(),
            "Grizzly Bears",
            Zone::Battlefield,
        );
        let second = put(
            &mut base,
            player.other(),
            "Grizzly Bears",
            Zone::Battlefield,
        );
        let walker = put(
            &mut base,
            player.other(),
            "Oko, the Ringleader",
            Zone::Battlefield,
        );
        base.set_counters(walker, Counters::of(CounterKind::Loyalty, 6));
        let walker = base.object_ref(walker);
        attackers(&mut base);

        // Either unchosen attack costs mana, so the requirement can be declined.
        assert!(base.legal_actions().contains(&Action::FinishAttackers));
        for declaration in [
            Action::Attack { card: creature },
            Action::AttackPlaneswalker {
                card: creature,
                defender: walker,
            },
        ] {
            let mut paid = base.clone();
            act(&mut paid, declaration.clone());
            act(&mut paid, Action::FinishAttackers);
            assert_eq!(paid.observe(player).attack_payment.unwrap().generic, 2);
            reject(
                &mut paid,
                Action::PayAttackCost {
                    payment: [0, 0, 0, 0, 1, 0],
                },
            );
            reject(
                &mut paid,
                Action::PayBlockCost {
                    payment: [0, 0, 0, 0, 2, 0],
                },
            );
            act(
                &mut paid,
                Action::PayAttackCost {
                    payment: [0, 0, 0, 0, 2, 0],
                },
            );
            assert!(paid.object_view(creature).tapped);

            let mut tapped = base.clone();
            tapped.state.objects[source as usize].tapped = true;
            tapped.state.objects[second as usize].tapped = true;
            reject(&mut tapped, Action::FinishAttackers);
            act(&mut tapped, declaration);
            act(&mut tapped, Action::FinishAttackers);
            assert!(tapped.observe(player).attack_payment.is_none());
        }

        let battle = spawn(
            &mut base,
            player,
            Card::from_name("@fixture/siege-front").unwrap(),
            Zone::Battlefield,
        );
        base.add_counter(battle, CounterKind::Defense, 9);
        let battle = base.object_ref(battle);
        base.set_battle_protector(battle, player.other());
        reject(&mut base, Action::FinishAttackers);
        act(
            &mut base,
            Action::AttackBattle {
                card: creature,
                defender: battle,
            },
        );
        act(&mut base, Action::FinishAttackers);
        assert!(base.observe(player).attack_payment.is_none());
    }
}

#[test]
fn attacking_providers_tax_all_blockers_and_the_receipt_owns_payment_and_restore() {
    let _rules = fixture_rules::activate(&[
        (Card::GrizzlyBears, &TAX),
        (Card::GoblinPiker, &MUST_BE_BLOCKED),
    ]);
    let player = Player::P0;
    let defender = player.other();
    let mut base = scenario(player);
    let source = put(&mut base, player, "Grizzly Bears", Zone::Battlefield);
    let second = put(&mut base, player, "Grizzly Bears", Zone::Battlefield);
    let other_attacker = put(&mut base, player, "Goblin Piker", Zone::Battlefield);
    let elves = put(&mut base, defender, "Llanowar Elves", Zone::Battlefield);
    let forest = put(&mut base, defender, "Forest", Zone::Battlefield);
    blockers(&mut base, &[source, second, other_attacker]);

    // A requirement to block cannot force payment. The tax applies even
    // though this creature would block the other attacker.
    assert!(base.legal_actions().contains(&Action::FinishBlockers));
    act(
        &mut base,
        Action::Block {
            blocker: elves,
            attacker: other_attacker,
        },
    );
    let declaration = base.snapshot();
    assert!(base.observe(player).blocks.is_empty());
    act(&mut base, Action::FinishBlockers);
    assert_eq!(
        base.state.pending,
        Pending::BlockPayment { player: defender }
    );
    let public = base.observe(player);
    assert_eq!(public.block_payment.unwrap().generic, 2);
    assert!(public.attack_payment.is_none());
    assert_eq!(public.blocks, vec![(elves, other_attacker)]);
    assert!(base.state.blocks.is_empty());
    choose(&mut base, |action| {
        matches!(action,
        Action::TapMana { card } | Action::ActivateMana { card, .. } if *card == elves)
    });
    act(&mut base, Action::TapMana { card: forest });

    let payment = Action::PayBlockCost {
        payment: [0, 0, 0, 0, 2, 0],
    };
    reject(
        &mut base,
        Action::PayBlockCost {
            payment: [0, 0, 0, 0, 1, 0],
        },
    );
    reject(
        &mut base,
        Action::PayAttackCost {
            payment: [0, 0, 0, 0, 2, 0],
        },
    );
    let checkpoint = base.snapshot();
    let mut raw = base.clone();
    let mut indexed = base.clone();
    let mut restored = base.clone();
    act(&mut raw, payment.clone());
    let index = indexed
        .legal_actions()
        .iter()
        .position(|a| *a == payment)
        .unwrap();
    indexed
        .apply_index(defender, indexed.revision(), index)
        .unwrap();
    restored.restore(&checkpoint).unwrap();
    act(&mut restored, payment);
    assert_eq!(raw.snapshot(), indexed.snapshot());
    assert_eq!(raw.snapshot(), restored.snapshot());
    assert!(raw.object_view(elves).tapped);
    assert!(raw.object_view(forest).tapped);
    assert_eq!(raw.state.blocks, vec![(elves, other_attacker)]);
    assert!(
        raw.state
            .attackers
            .iter()
            .find(|a| a.card == other_attacker)
            .unwrap()
            .blocked
    );
    assert!(raw.observe(player).block_payment.is_none());

    let mut kept = base.clone();
    act(&mut kept, Action::AbortBlockPayment);
    while matches!(kept.state.pending, Pending::ManaRollback { .. }) {
        act(&mut kept, Action::ChooseManaRollback { keep: true });
    }
    assert!(kept.object_view(elves).tapped);
    assert!(kept.state.selected_blocks.is_empty());
    act(&mut kept, Action::FinishBlockers);
    assert!(kept.state.blocks.is_empty());

    act(&mut base, Action::AbortBlockPayment);
    while matches!(base.state.pending, Pending::ManaRollback { .. }) {
        act(&mut base, Action::ChooseManaRollback { keep: false });
    }
    assert_eq!(base.snapshot(), declaration);
    act(&mut base, Action::CancelBlockDeclaration);
    act(&mut base, Action::FinishBlockers);
    assert!(base.state.blocks.is_empty());
    assert!(!base.object_view(elves).tapped);
    assert!(!base.object_view(forest).tapped);
}

#[test]
fn a_taxed_blocker_cannot_start_an_unfundable_menace_declaration() {
    let _rules = fixture_rules::activate(&[(Card::GrizzlyBears, &TAX)]);
    let player = Player::P1;
    let mut g = scenario(player);
    let source = put(&mut g, player, "Grizzly Bears", Zone::Battlefield);
    g.state.objects[source as usize].gained_keywords = Keywords::MENACE;
    let blocker = put(&mut g, player.other(), "Bear Cub", Zone::Battlefield);
    put(&mut g, player.other(), "Bear Cub", Zone::Battlefield);
    put(&mut g, player.other(), "Forest", Zone::Battlefield);
    blockers(&mut g, &[source]);
    reject(
        &mut g,
        Action::Block {
            blocker,
            attacker: source,
        },
    );
    act(&mut g, Action::FinishBlockers);
    assert!(g.state.blocks.is_empty());
}
