//! Exact non-targeted attached-permanent untap, including source LKI.
mod support;
use rustmage as _;
use rustmage_core::cards::authored::{CardExtras, CardRules};
use rustmage_core::cards::*;
use rustmage_core::engine::*;
use support::*;

const SPEC: AuraSpec = AuraSpec {
    kinds: 1,
    relation: -1,
    bonus: EquipmentSpec {
        stats: Stats {
            power: 0,
            toughness: 0,
        },
        keywords: Keywords::NONE,
    },
};
const AURA: CardRules = CardRules {
    effect: Effect::Aura(TargetPlan::Single(TargetSpec::one(
        TargetRule::CreatureOrVehicle { relation: -1 },
    ))),
    aura: Some(SPEC),
    extras: &CardExtras {
        aura: Some(SPEC),
        aura_kind_or_subtype: Some("Vehicle"),
        ..CardExtras::EMPTY
    },
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::SelfEnters,
        targets: TargetPlan::NONE,
        program: &[Instruction::UntapAttached],
    }],
    ..CardRules::EMPTY
};

fn await_entry_trigger(g: &mut Game, aura: CardId) {
    for _ in 0..32 {
        if g.state.pending == Pending::Priority
            && g.state.stack.iter().any(|item| {
                matches!(item.kind, StackKind::TriggeredAbility { .. })
                    && item.source.is_some_and(|source| source.id == aura)
            })
        {
            return;
        }
        match g.state.pending {
            Pending::Priority => act(g, Action::Pass),
            Pending::OrderTriggers { .. } => choose(g, |a| matches!(a, Action::PutTrigger { .. })),
            _ => panic!("unexpected entry choice {:?}", g.state.pending),
        }
    }
    panic!("entry trigger was not published");
}

fn ready(p: Player, recipient_name: &str) -> (Game, CardId, CardId) {
    let mut g = scenario(p);
    let recipient = put(&mut g, p.other(), recipient_name, Zone::Battlefield);
    g.state.objects[recipient as usize].tapped = true;
    let aura = put(&mut g, p, "Pacifism", Zone::Hand);
    cast(&mut g, aura, Some(Target::Card(recipient)));
    await_entry_trigger(&mut g, aura);
    assert!(g.object_view(recipient).tapped);
    (g, aura, recipient)
}

#[test]
fn attached_untap_accepts_an_opposing_uncrewed_vehicle_and_owned_branches() {
    let _scope = fixture_rules::activate(&[(Card::from_name("Pacifism").unwrap(), &AURA)]);
    for p in [Player::P0, Player::P1] {
        for name in ["Bear Cub", "Clamorous Ironclad"] {
            let (mut parent, _, recipient) = ready(p, name);
            if name == "Clamorous Ironclad" {
                assert!(!parent.is_creature(recipient));
            }
            parent.state.objects[recipient as usize].gained_keywords = Keywords::HEXPROOF;
            let saved = parent.snapshot();
            let actor = parent.actor().unwrap();
            let mut raw = parent.clone();
            let mut indexed = parent.clone();
            let mut restored = parent.clone();
            restored.restore(&saved).unwrap();
            drop(parent);
            for (i, branch) in [&mut raw, &mut indexed, &mut restored]
                .into_iter()
                .enumerate()
            {
                if i == 1 {
                    let index = branch
                        .legal_actions()
                        .iter()
                        .position(|a| *a == Action::Pass)
                        .unwrap();
                    branch.apply_index(actor, branch.revision(), index).unwrap();
                } else {
                    act(branch, Action::Pass);
                }
                settle(branch);
                assert!(!branch.object_view(recipient).tapped);
            }
            assert_eq!(raw.snapshot(), indexed.snapshot());
            assert_eq!(raw.snapshot(), restored.snapshot());
            for viewer in [Player::P0, Player::P1] {
                assert_eq!(
                    serde_json::to_value(raw.observe(viewer)).unwrap(),
                    serde_json::to_value(indexed.observe(viewer)).unwrap()
                );
            }
        }
    }
}

#[test]
fn attached_untap_uses_the_last_attachment_when_the_source_leaves() {
    let _scope = fixture_rules::activate(&[(Card::from_name("Pacifism").unwrap(), &AURA)]);
    for leave in [false, true] {
        let (mut g, aura, first) = ready(Player::P0, "Bear Cub");
        let last = put(&mut g, Player::P1, "Clamorous Ironclad", Zone::Battlefield);
        g.state.objects[last as usize].tapped = true;
        g.state.objects[aura as usize].attached_to = Some(g.object_ref(last));
        if leave {
            g.move_object(aura, Zone::Graveyard);
        }
        settle(&mut g);
        assert!(
            g.object_view(first).tapped,
            "the old attachment is not the recipient"
        );
        assert!(
            !g.object_view(last).tapped,
            "current attachment or its final LKI is used"
        );
    }
}

#[test]
fn attached_untap_does_not_rebind_a_blinked_recipient() {
    let _scope = fixture_rules::activate(&[(Card::from_name("Pacifism").unwrap(), &AURA)]);
    let (mut g, _, recipient) = ready(Player::P0, "Bear Cub");
    let old = g.object_ref(recipient);
    g.move_object(recipient, Zone::Hand);
    g.state_based_actions();
    g.move_object(recipient, Zone::Battlefield);
    g.state.objects[recipient as usize].tapped = true;
    assert_ne!(g.object_ref(recipient), old);
    settle(&mut g);
    assert!(g.object_view(recipient).tapped);
}

#[test]
fn attached_untap_uses_the_normal_stun_replacement() {
    let _scope = fixture_rules::activate(&[(Card::from_name("Pacifism").unwrap(), &AURA)]);
    let (mut g, _, recipient) = ready(Player::P1, "Clamorous Ironclad");
    g.state.objects[recipient as usize].counters.stun = 1;
    settle(&mut g);
    assert!(g.object_view(recipient).tapped);
    assert_eq!(g.counter_count(recipient, CounterKind::Stun), 0);
}
