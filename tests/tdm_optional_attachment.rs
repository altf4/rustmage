//! Creation completes before a non-target optional exact-source attachment.
mod support;
use rustmage as _;
use rustmage_core::cards::authored::{CardExtras, CardRules};
use rustmage_core::cards::*;
use rustmage_core::engine::*;
use support::*;

const BONUS: EquipmentSpec = EquipmentSpec {
    stats: Stats {
        power: 1,
        toughness: 1,
    },
    keywords: Keywords::HASTE.union(Keywords::TRAMPLE),
};
const GROUPS: &[StaticAbilityGroup] = &[
    StaticAbilityGroup {
        line: 0,
        first_layer: AbilityLayer::Modify,
        fragments: &[StaticAbilityFragment::AttachmentStats(Stats {
            power: 1,
            toughness: 1,
        })],
    },
    StaticAbilityGroup {
        line: 0,
        first_layer: AbilityLayer::Ability,
        fragments: &[StaticAbilityFragment::AttachmentKeywords(
            Keywords::HASTE.union(Keywords::TRAMPLE),
        )],
    },
];
const CREATOR: CardRules = CardRules {
    effect: Effect::Permanent,
    static_groups: GROUPS,
    extras: &CardExtras {
        equipment: Some(BONUS),
        static_ability_groups: GROUPS,
        ..CardExtras::EMPTY
    },
    activated: &[ActivatedAbility {
        exhaust: false,
        guard: ActivationGuard::Always,
        origin: ActivationOrigin::Battlefield,
        cycling: false,
        cost: ActivationCost {
            generic: 1,
            mana_component: true,
            ..ActivationCost::ZERO
        },
        targets: TargetPlan::NONE,
        program: &[
            Instruction::CreateTokensThenMayAttachSource {
                token: Card(5486),
                amount: 1,
            },
            Instruction::Draw(PlayerSet::Controller, 1),
        ],
        sorcery: false,
        once_per_turn: false,
    }],
    ..CardRules::EMPTY
};
const PROTECTED_TOKEN: CardRules = CardRules {
    effect: Effect::Creature(1, 1),
    keywords: Keywords::PROTECTION_EVERYTHING,
    ..CardRules::EMPTY
};

fn wait(g: &mut Game, ready: impl Fn(&Game) -> bool) {
    for _ in 0..32 {
        if ready(g) {
            return;
        }
        assert_eq!(g.state.pending, Pending::Priority);
        act(g, Action::Pass);
    }
    panic!("32-action creation completion budget exhausted");
}
fn activate(g: &mut Game, source: CardId) {
    choose(
        g,
        |a| matches!(a, Action::Activate { card, ability: 0, target: None, .. } if *card == source),
    );
}
fn monks(g: &Game, p: Player) -> Vec<CardId> {
    g.state
        .objects
        .iter()
        .enumerate()
        .filter_map(|(id, o)| {
            (o.zone == Zone::Battlefield
                && o.controller == p
                && o.is_token()
                && o.card.has_subtype("Monk"))
            .then_some(id as CardId)
        })
        .collect()
}
fn setup(p: Player, doubled: bool) -> (Game, CardId, CardId) {
    let mut g = scenario(p);
    if doubled {
        let elspeth = put(&mut g, p, "Elspeth, Storm Slayer", Zone::Battlefield);
        g.add_counter(elspeth, CounterKind::Loyalty, 5);
    }
    let prior = put(&mut g, p, "Bear Cub", Zone::Battlefield);
    let source = put(&mut g, p, "Short Bow", Zone::Battlefield);
    let recipient = g.object_ref(prior);
    assert!(g.attach_equipment(g.object_ref(source), recipient));
    activate(&mut g, source);
    (g, source, prior)
}

#[test]
fn created_token_precedes_optional_attachment_and_decline_preserves_old_recipient() {
    let _rules = fixture_rules::activate(&[(Card::from_name("Short Bow").unwrap(), &CREATOR)]);
    for p in [Player::P0, Player::P1] {
        for attach in [false, true] {
            let (mut g, source, prior) = setup(p, false);
            wait(&mut g, |g| {
                matches!(g.state.pending, Pending::CreatedToken { .. })
            });
            let created = monks(&g, p);
            assert_eq!(created.len(), 1);
            let token = created[0];
            assert_eq!(stats(&g, token), (1, 1));
            assert_eq!(g.object_view(token).colors, 1);
            assert_eq!(g.state.players[p.index()].cards_drawn_this_turn, 0);
            reject(&mut g, Action::ChooseOptional { accept: true });
            let other = g.object_ref(prior);
            reject(&mut g, Action::ChooseCreatedToken { object: other });
            if attach {
                let object = g.object_ref(token);
                act(&mut g, Action::ChooseCreatedToken { object });
            } else {
                act(&mut g, Action::ChooseOptional { accept: false });
            }
            settle(&mut g);
            assert_eq!(
                g.object_view(source).attached_to,
                Some(g.object_ref(if attach { token } else { prior }))
            );
            assert_eq!(stats(&g, token), if attach { (2, 2) } else { (1, 1) });
            assert_eq!(g.state.players[p.index()].cards_drawn_this_turn, 1);
        }
    }
}

#[test]
fn doubled_creation_selects_only_one_of_the_exact_created_tokens_or_declines() {
    let _rules = fixture_rules::activate(&[(Card::from_name("Short Bow").unwrap(), &CREATOR)]);
    for attach in [false, true] {
        let p = Player::P0;
        let (mut g, source, prior) = setup(p, true);
        wait(&mut g, |g| {
            matches!(g.state.pending, Pending::CreatedToken { .. })
        });
        let created = monks(&g, p);
        assert_eq!(created.len(), 2);
        assert_eq!(
            g.observe(p)
                .created_token_choice
                .as_ref()
                .unwrap()
                .candidates
                .len(),
            2
        );
        let object = g.object_ref(created[1]);
        if attach {
            act(&mut g, Action::ChooseCreatedToken { object });
        } else {
            act(&mut g, Action::ChooseOptional { accept: false });
        }
        settle(&mut g);
        assert_eq!(
            g.object_view(source).attached_to,
            Some(if attach { object } else { g.object_ref(prior) })
        );
        assert_eq!(stats(&g, created[0]), (1, 1));
        assert_eq!(stats(&g, created[1]), if attach { (2, 2) } else { (1, 1) });
        assert_eq!(g.state.players[p.index()].cards_drawn_this_turn, 1);
    }
}

#[test]
fn departed_source_creates_tokens_but_cannot_attach_its_new_incarnation() {
    let _rules = fixture_rules::activate(&[(Card::from_name("Short Bow").unwrap(), &CREATOR)]);
    let p = Player::P0;
    let (mut g, source, _) = setup(p, false);
    let original = g.object_ref(source);
    act(&mut g, Action::Pass);
    let removal = put(&mut g, p.other(), "Disenchant", Zone::Hand);
    cast(&mut g, removal, Some(Target::Card(source)));
    wait(&mut g, |g| g.object_view(source).zone == Zone::Graveyard);
    g.move_object(source, Zone::Battlefield);
    assert_ne!(g.object_ref(source), original);
    settle(&mut g);
    assert_eq!(monks(&g, p).len(), 1);
    assert_eq!(g.object_view(source).attached_to, None);
    assert!(g.observe(p).created_token_choice.is_none());
    assert_eq!(g.state.players[p.index()].cards_drawn_this_turn, 1);
}

#[test]
fn illegal_created_recipient_keeps_the_created_token_and_continues_the_program() {
    let _rules = fixture_rules::activate(&[
        (Card::from_name("Short Bow").unwrap(), &CREATOR),
        (Card(5486), &PROTECTED_TOKEN),
    ]);
    let p = Player::P0;
    let (mut g, source, prior) = setup(p, false);
    settle(&mut g);
    assert_eq!(monks(&g, p).len(), 1);
    assert_eq!(g.object_view(source).attached_to, Some(g.object_ref(prior)));
    assert!(g.observe(p).created_token_choice.is_none());
    assert_eq!(g.state.players[p.index()].cards_drawn_this_turn, 1);
}

#[test]
fn optional_created_attachment_pending_state_is_owned_and_restorable() {
    let _rules = fixture_rules::activate(&[(Card::from_name("Short Bow").unwrap(), &CREATOR)]);
    let p = Player::P0;
    let (mut parent, source, prior) = setup(p, true);
    wait(&mut parent, |g| {
        matches!(g.state.pending, Pending::CreatedToken { .. })
    });
    let object = parent.object_ref(monks(&parent, p)[1]);
    let action = Action::ChooseCreatedToken { object };
    let saved = parent.snapshot();
    let rev = parent.revision();
    let index = parent
        .legal_actions()
        .iter()
        .position(|a| *a == action)
        .unwrap();
    let mut raw = parent.clone();
    let mut indexed = parent.clone();
    let mut declined = parent.clone();
    drop(parent);
    assert_eq!(raw.apply(p.other(), action.clone()), Err(Error::WrongActor));
    assert_eq!(raw.snapshot(), saved);
    raw.apply(p, action.clone()).unwrap();
    indexed.apply_index(p, rev, index).unwrap();
    settle(&mut raw);
    settle(&mut indexed);
    assert_eq!(raw.snapshot(), indexed.snapshot());
    assert_eq!(raw.object_view(source).attached_to, Some(object));
    act(&mut declined, Action::ChooseOptional { accept: false });
    settle(&mut declined);
    assert_eq!(
        declined.object_view(source).attached_to,
        Some(declined.object_ref(prior))
    );
    indexed.restore(&saved).unwrap();
    assert_eq!(
        indexed.apply_index(p, rev, index),
        Err(Error::StaleRevision)
    );
    act(&mut indexed, action);
    settle(&mut indexed);
    assert_eq!(raw.snapshot(), indexed.snapshot());
}

#[test]
fn existing_job_selection_remains_mandatory_after_doubling() {
    let p = Player::P0;
    let mut g = scenario(p);
    let elspeth = put(&mut g, p, "Elspeth, Storm Slayer", Zone::Battlefield);
    g.add_counter(elspeth, CounterKind::Loyalty, 5);
    let sword = put(&mut g, p, "Dwarven Shortsword", Zone::Hand);
    cast(&mut g, sword, None);
    wait(&mut g, |g| {
        matches!(g.state.pending, Pending::CreatedToken { .. })
    });
    reject(&mut g, Action::ChooseOptional { accept: false });
    let object = g.observe(p).created_token_choice.unwrap().candidates[1];
    act(&mut g, Action::ChooseCreatedToken { object });
    settle(&mut g);
    assert_eq!(g.object_view(sword).attached_to, Some(object));
    assert_eq!(stats(&g, object.card), (3, 4));
}
