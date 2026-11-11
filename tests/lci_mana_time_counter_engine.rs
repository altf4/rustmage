//! Removing a Time counter after a mana ability produces its mana.
mod support;
use rustmage as _;
use rustmage_core::cards::authored::CardRules;
use rustmage_core::{cards::*, engine::*};
use support::*;

const BLUE: Mana = [0, 1, 0, 0, 0, 0];
const GREEN: Mana = [0, 0, 0, 0, 1, 0];
const BLACK: Mana = [0, 0, 1, 0, 0, 0];
const TIME_MANA: ActivatedAbility = ActivatedAbility {
    exhaust: false,
    guard: ActivationGuard::Always,
    origin: ActivationOrigin::Battlefield,
    cycling: false,
    cost: ActivationCost {
        tap: true,
        ..ActivationCost::ZERO
    },
    targets: TargetPlan::NONE,
    program: &[
        Instruction::AddMana(ManaProduction::Fixed(BLUE)),
        Instruction::RemoveSourceCounterIfAny {
            kind: CounterKind::Time,
            skip: 0,
        },
    ],
    sorcery: false,
    once_per_turn: false,
};
const LAND: CardRules = CardRules {
    effect: Effect::Land,
    activated: &[TIME_MANA],
    ..CardRules::EMPTY
};
const GUARDED: CardRules = CardRules {
    activated: &[
        TIME_MANA,
        ActivatedAbility {
            guard: ActivationGuard::SourceCounterExactly {
                kind: CounterKind::Time,
                amount: 0,
            },
            cost: ActivationCost::ZERO,
            program: &[Instruction::AddMana(ManaProduction::Fixed(GREEN))],
            once_per_turn: true,
            ..TIME_MANA
        },
    ],
    ..LAND
};

fn mana(card: CardId, ability: u16, production: Mana) -> Action {
    Action::ActivateMana {
        card,
        ability,
        counter_payment: Counters::default(),
        payment: [0; 6],
        production,
    }
}

#[test]
fn the_last_counter_unlocks_projected_funding_and_zero_still_produces_mana() {
    let _rules =
        fixture_rules::activate(&[(Card::from_name("Hidden Cataract").unwrap(), &GUARDED)]);
    let p = Player::P0;
    let mut game = scenario(p);
    game.state.players[p.index()].mana = [0; 6];
    let source = spawn(
        &mut game,
        p,
        Card::from_name("Hidden Cataract").unwrap(),
        Zone::Battlefield,
    );
    let spell = put(&mut game, p, "Pond Prophet", Zone::Hand);
    game.add_counter(source, CounterKind::Time, 1);
    reject(&mut game, mana(source, 1, GREEN));
    act(&mut game, Action::AnnounceCast { card: spell });
    if matches!(game.state.pending, Pending::SpellCost { .. }) {
        choose(&mut game, |action| {
            matches!(action, Action::ChooseSpellCost { .. })
        });
    }
    let action = mana(source, 0, BLUE);
    let index = game
        .legal_actions()
        .iter()
        .position(|candidate| *candidate == action)
        .unwrap();
    let saved = game.snapshot();
    let revision = game.revision();
    let mut indexed = game.clone();
    act(&mut game, action.clone());
    indexed.apply_index(p, revision, index).unwrap();
    assert_eq!(game.snapshot(), indexed.snapshot());
    assert_eq!(game.counter_count(source, CounterKind::Time), 0);
    assert!(game.state.stack.is_empty());
    for branch in [&mut game, &mut indexed] {
        act(branch, mana(source, 1, GREEN));
        pay(branch);
        assert_eq!(branch.object_view(spell).zone, Zone::Stack);
    }
    assert_eq!(game.snapshot(), indexed.snapshot());
    indexed.restore(&saved).unwrap();
    assert_eq!(indexed.counter_count(source, CounterKind::Time), 1);
    act(&mut indexed, action.clone());
    act(&mut indexed, mana(source, 1, GREEN));
    pay(&mut indexed);
    assert_eq!(game.snapshot(), indexed.snapshot());
    for branch in [&mut game, &mut indexed] {
        settle(branch);
        branch.untap_objects(&[source], untap_rules::UntapCause::Effect);
        act(branch, action.clone());
        assert_eq!(branch.counter_count(source, CounterKind::Time), 0);
        assert_eq!(branch.state.players[p.index()].mana, BLUE);
    }
}

#[test]
fn failed_cast_can_keep_or_reverse_the_completed_counter_removal_with_its_mana() {
    let _rules = fixture_rules::activate(&[(Card::from_name("Hidden Cataract").unwrap(), &LAND)]);
    let p = Player::P0;
    let mut game = scenario(p);
    let silver = put(
        &mut game,
        p,
        "Silverquill, the Disputant",
        Zone::Battlefield,
    );
    game.state.objects[silver as usize].growth.power = -1;
    let enemy = put(&mut game, p.other(), "Bear Cub", Zone::Battlefield);
    let aura = put(
        &mut game,
        p.other(),
        "Witness Protection",
        Zone::Battlefield,
    );
    game.state.objects[aura as usize].attached_to = Some(game.object_ref(enemy));
    let prison = put(&mut game, p, "Banishing Light", Zone::Hand);
    cast(&mut game, prison, None);
    for _ in 0..32 {
        if game.state.pending != Pending::Priority {
            break;
        }
        act(&mut game, Action::Pass);
    }
    act(
        &mut game,
        Action::ChooseTarget {
            target: Target::Card(aura),
        },
    );
    for _ in 0..32 {
        if game.state.pending == Pending::Priority
            && game.state.stack.is_empty()
            && game.state.resolving.is_none()
        {
            break;
        }
        choose(&mut game, |a| {
            matches!(
                a,
                Action::Pass | Action::PutTrigger { .. } | Action::FinishStackTargets
            )
        });
    }
    let source = spawn(
        &mut game,
        p,
        Card::from_name("Hidden Cataract").unwrap(),
        Zone::Battlefield,
    );
    game.add_counter(source, CounterKind::Time, 1);
    let spell = put(&mut game, p, "Candy Grapple", Zone::Hand);
    game.state.players[p.index()].mana = BLACK;
    act(&mut game, Action::AnnounceCast { card: spell });
    let sacrifice = Some(game.object_ref(prison));
    act(&mut game, Action::ChooseBargain { sacrifice });
    act(&mut game, Action::ChooseCasualty { pay: true });
    act(
        &mut game,
        Action::ChooseTarget {
            target: Target::Card(enemy),
        },
    );
    act(&mut game, mana(source, 0, BLUE));
    assert_eq!(game.counter_count(source, CounterKind::Time), 0);
    pay(&mut game);
    let clause = game
        .observe(p)
        .cast_costs
        .unwrap()
        .clauses
        .iter()
        .position(|r| r.clause == CastCostClause::Additional)
        .unwrap() as u16;
    act(&mut game, Action::ChooseCastCost { clause });
    let object = game.object_ref(silver);
    act(&mut game, Action::ChooseAuraAttachment { object });
    assert!(matches!(game.state.pending, Pending::ManaRollback { .. }));
    assert!(game.observe(p.other()).mana_rollback.is_none());
    let saved = game.snapshot();
    for keep in [false, true] {
        let mut raw = game.clone();
        let mut indexed = game.clone();
        indexed.restore(&saved).unwrap();
        let action = Action::ChooseManaRollback { keep };
        let index = indexed
            .legal_actions()
            .iter()
            .position(|candidate| *candidate == action)
            .unwrap();
        let revision = indexed.revision();
        act(&mut raw, action);
        indexed.apply_index(p, revision, index).unwrap();
        assert_eq!(raw.snapshot(), indexed.snapshot());
        assert_eq!(raw.object_view(spell).zone, Zone::Hand);
        assert_eq!(raw.object_view(source).tapped, keep);
        assert_eq!(
            raw.counter_count(source, CounterKind::Time),
            u64::from(!keep)
        );
        assert_eq!(
            raw.state.players[p.index()].mana,
            [0, u32::from(keep), 1, 0, 0, 0]
        );
    }
}
