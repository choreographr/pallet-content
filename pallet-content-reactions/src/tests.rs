use crate::{mock::*, Emoji, Error, Event, Pallet as ContentReactions, ReactionsOf};
use pallet_content::{IpfsHash, ItemId, Nonce, Pallet as Content, RevisionId, RETRACTED};
use polkadot_sdk::frame_support::{assert_noop, assert_ok};

const REVISIONABLE: u8 = 1 << 0;
const INITIAL_REVISION_ID: RevisionId = 0;
const NEXT_REVISION_ID: RevisionId = 1;
const GRINNING_FACE: Emoji = Emoji(0x1F600);
const SMILING_FACE_WITH_HEART_EYES: Emoji = Emoji(0x1F60D);
const PARTY_POPPER: Emoji = Emoji(0x1F389);

fn publish_item(owner: u64, nonce: Nonce) -> ItemId {
    assert_ok!(Content::<Test>::publish_item(
        RuntimeOrigin::signed(owner),
        nonce,
        Default::default(),
        REVISIONABLE,
        Default::default(),
        Default::default(),
        IpfsHash::default(),
    ));

    System::events()
        .into_iter()
        .rev()
        .find_map(|record| match record.event {
            RuntimeEvent::Content(pallet_content::Event::PublishItem { item_id, .. }) => {
                Some(item_id)
            }
            _ => None,
        })
        .expect("publish item event must exist")
}

fn insert_owned_item(owner: u64, fill: u8) -> ItemId {
    let item_id = ItemId([fill; 32]);
    pallet_content::ItemState::<Test>::insert(
        item_id.clone(),
        pallet_content::Item {
            owner,
            revision_id: INITIAL_REVISION_ID,
            flags: REVISIONABLE,
        },
    );
    item_id
}

fn publish_revision(owner: u64, item_id: ItemId) -> RevisionId {
    assert_ok!(Content::<Test>::publish_revision(
        RuntimeOrigin::signed(owner),
        item_id,
        Default::default(),
        Default::default(),
        IpfsHash::default(),
    ));

    System::events()
        .into_iter()
        .rev()
        .find_map(|record| match record.event {
            RuntimeEvent::Content(pallet_content::Event::PublishRevision {
                revision_id, ..
            }) => Some(revision_id),
            _ => None,
        })
        .expect("publish revision event must exist")
}

fn reactions_vec(emojis: Vec<Emoji>) -> ReactionsOf<Test> {
    emojis
        .try_into()
        .expect("emojis fit within MaxEmojis bound")
}

#[test]
fn set_reactions_emits_event_with_full_set() {
    new_test_ext().execute_with(|| {
        let item_id = publish_item(1, Nonce::default());

        assert_ok!(ContentReactions::<Test>::set_reactions(
            RuntimeOrigin::signed(2),
            item_id.clone(),
            INITIAL_REVISION_ID,
            reactions_vec(vec![GRINNING_FACE]),
        ));

        System::assert_has_event(
            Event::<Test>::SetReactions {
                item_id,
                revision_id: INITIAL_REVISION_ID,
                item_owner: 1,
                reactor: 2,
                reactions: reactions_vec(vec![GRINNING_FACE]),
            }
            .into(),
        );
    });
}

#[test]
fn set_reactions_with_multiple_emojis() {
    new_test_ext().execute_with(|| {
        let item_id = publish_item(1, Nonce::default());

        assert_ok!(ContentReactions::<Test>::set_reactions(
            RuntimeOrigin::signed(2),
            item_id.clone(),
            INITIAL_REVISION_ID,
            reactions_vec(vec![GRINNING_FACE, SMILING_FACE_WITH_HEART_EYES]),
        ));

        System::assert_has_event(
            Event::<Test>::SetReactions {
                item_id,
                revision_id: INITIAL_REVISION_ID,
                item_owner: 1,
                reactor: 2,
                reactions: reactions_vec(vec![GRINNING_FACE, SMILING_FACE_WITH_HEART_EYES]),
            }
            .into(),
        );
    });
}

#[test]
fn set_reactions_with_empty_set() {
    new_test_ext().execute_with(|| {
        let item_id = publish_item(1, Nonce::default());

        assert_ok!(ContentReactions::<Test>::set_reactions(
            RuntimeOrigin::signed(2),
            item_id.clone(),
            INITIAL_REVISION_ID,
            reactions_vec(vec![]),
        ));

        System::assert_has_event(
            Event::<Test>::SetReactions {
                item_id,
                revision_id: INITIAL_REVISION_ID,
                item_owner: 1,
                reactor: 2,
                reactions: reactions_vec(vec![]),
            }
            .into(),
        );
    });
}

#[test]
fn set_reactions_replaces_previous_set() {
    new_test_ext().execute_with(|| {
        let item_id = publish_item(1, Nonce::default());

        assert_ok!(ContentReactions::<Test>::set_reactions(
            RuntimeOrigin::signed(2),
            item_id.clone(),
            INITIAL_REVISION_ID,
            reactions_vec(vec![GRINNING_FACE]),
        ));

        assert_ok!(ContentReactions::<Test>::set_reactions(
            RuntimeOrigin::signed(2),
            item_id.clone(),
            INITIAL_REVISION_ID,
            reactions_vec(vec![PARTY_POPPER]),
        ));

        let events: Vec<_> = System::events()
            .into_iter()
            .filter_map(|record| match record.event {
                RuntimeEvent::ContentReactions(Event::SetReactions { reactions, .. }) => {
                    Some(reactions.into_inner())
                }
                _ => None,
            })
            .collect();

        assert_eq!(events.len(), 2);
        assert_eq!(events[0], vec![GRINNING_FACE]);
        assert_eq!(events[1], vec![PARTY_POPPER]);
    });
}

#[test]
fn set_reactions_rejects_duplicate_emojis() {
    new_test_ext().execute_with(|| {
        let item_id = insert_owned_item(1, 1);

        assert_noop!(
            ContentReactions::<Test>::set_reactions(
                RuntimeOrigin::signed(2),
                item_id,
                INITIAL_REVISION_ID,
                reactions_vec(vec![GRINNING_FACE, GRINNING_FACE]),
            ),
            Error::<Test>::DuplicateEmoji
        );
    });
}

#[test]
fn set_reactions_rejects_invalid_emoji_values() {
    new_test_ext().execute_with(|| {
        let item_id = insert_owned_item(1, 1);

        assert_noop!(
            ContentReactions::<Test>::set_reactions(
                RuntimeOrigin::signed(2),
                item_id.clone(),
                INITIAL_REVISION_ID,
                reactions_vec(vec![Emoji(0)]),
            ),
            Error::<Test>::InvalidEmoji
        );

        assert_noop!(
            ContentReactions::<Test>::set_reactions(
                RuntimeOrigin::signed(2),
                item_id.clone(),
                INITIAL_REVISION_ID,
                reactions_vec(vec![Emoji(0xD800)]),
            ),
            Error::<Test>::InvalidEmoji
        );

        assert_noop!(
            ContentReactions::<Test>::set_reactions(
                RuntimeOrigin::signed(2),
                item_id,
                INITIAL_REVISION_ID,
                reactions_vec(vec![Emoji(0x110000)]),
            ),
            Error::<Test>::InvalidEmoji
        );
    });
}

#[test]
fn set_reactions_rejects_missing_item() {
    new_test_ext().execute_with(|| {
        assert_noop!(
            ContentReactions::<Test>::set_reactions(
                RuntimeOrigin::signed(1),
                ItemId([7; 32]),
                INITIAL_REVISION_ID,
                reactions_vec(vec![GRINNING_FACE]),
            ),
            Error::<Test>::ItemNotFound
        );
    });
}

#[test]
fn set_reactions_rejects_retracted_item() {
    new_test_ext().execute_with(|| {
        let item_id = ItemId([9; 32]);
        pallet_content::ItemState::<Test>::insert(
            item_id.clone(),
            pallet_content::Item {
                owner: 1,
                revision_id: INITIAL_REVISION_ID,
                flags: RETRACTED,
            },
        );

        assert_noop!(
            ContentReactions::<Test>::set_reactions(
                RuntimeOrigin::signed(2),
                item_id,
                INITIAL_REVISION_ID,
                reactions_vec(vec![GRINNING_FACE]),
            ),
            Error::<Test>::ItemRetracted
        );
    });
}

#[test]
fn set_reactions_rejects_unknown_revision_id() {
    new_test_ext().execute_with(|| {
        let item_id = publish_item(1, Nonce::default());

        assert_noop!(
            ContentReactions::<Test>::set_reactions(
                RuntimeOrigin::signed(2),
                item_id,
                NEXT_REVISION_ID,
                reactions_vec(vec![GRINNING_FACE]),
            ),
            Error::<Test>::RevisionNotFound
        );
    });
}

#[test]
fn set_reactions_is_scoped_to_revision_id() {
    new_test_ext().execute_with(|| {
        let item_id = publish_item(1, Nonce::default());
        let revision_id = publish_revision(1, item_id.clone());

        assert_eq!(revision_id, NEXT_REVISION_ID);

        assert_ok!(ContentReactions::<Test>::set_reactions(
            RuntimeOrigin::signed(2),
            item_id.clone(),
            INITIAL_REVISION_ID,
            reactions_vec(vec![GRINNING_FACE]),
        ));
        assert_ok!(ContentReactions::<Test>::set_reactions(
            RuntimeOrigin::signed(2),
            item_id.clone(),
            revision_id,
            reactions_vec(vec![PARTY_POPPER]),
        ));

        let events: Vec<_> = System::events()
            .into_iter()
            .filter_map(|record| match record.event {
                RuntimeEvent::ContentReactions(Event::SetReactions {
                    revision_id,
                    reactions,
                    ..
                }) => Some((revision_id, reactions.into_inner())),
                _ => None,
            })
            .collect();

        assert_eq!(events.len(), 2);
        assert_eq!(events[0], (INITIAL_REVISION_ID, vec![GRINNING_FACE]));
        assert_eq!(events[1], (NEXT_REVISION_ID, vec![PARTY_POPPER]));
    });
}

#[test]
fn set_reactions_validates_emojis_before_duplicate_check() {
    new_test_ext().execute_with(|| {
        let item_id = insert_owned_item(1, 1);

        assert_noop!(
            ContentReactions::<Test>::set_reactions(
                RuntimeOrigin::signed(2),
                item_id,
                INITIAL_REVISION_ID,
                reactions_vec(vec![Emoji(0), GRINNING_FACE]),
            ),
            Error::<Test>::InvalidEmoji
        );
    });
}

#[test]
fn weight_info_is_positive_for_all_dispatchables() {
    use crate::{SubstrateWeight, WeightInfo};

    fn assert_weight_impl<W: WeightInfo>() {
        // Placeholder weights: assert only that the dispatchable reports a
        // non-zero weight until runtime benchmarks replace this value.
        assert!(W::set_reactions().ref_time() > 0);
    }

    assert_weight_impl::<()>();
    assert_weight_impl::<SubstrateWeight<Test>>();
}

#[test]
fn dispatch_info_and_runtime_dispatch_cover_dispatchables() {
    use crate::Call;
    use polkadot_sdk::frame_support::dispatch::GetDispatchInfo;
    use polkadot_sdk::frame_support::traits::UnfilteredDispatchable;

    new_test_ext().execute_with(|| {
        let owner = 1;
        let item_id = publish_item(owner, Nonce::default());
        let reactions: ReactionsOf<Test> = vec![GRINNING_FACE]
            .try_into()
            .expect("reaction fits in bounds");

        let set = Call::<Test>::set_reactions {
            item_id,
            revision_id: INITIAL_REVISION_ID,
            reactions,
        };
        assert!(set.get_dispatch_info().call_weight.ref_time() > 0);

        // Dispatch through the runtime so the generated `RuntimeCall` glue runs.
        assert_ok!(
            RuntimeCall::ContentReactions(set).dispatch_bypass_filter(RuntimeOrigin::signed(owner))
        );
    });
}

#[test]
fn generated_runtime_glue_is_covered() {
    use polkadot_sdk::frame_support::view_functions::ViewFunctionId;
    use scale_info::TypeInfo;

    fn touch<T: TypeInfo>() {
        let _ = T::type_info();
    }

    touch::<Test>();
    touch::<RuntimeCall>();
    touch::<RuntimeEvent>();
    touch::<RuntimeSlashReason>();
    touch::<RuntimeLockId>();
    touch::<RuntimeHoldReason>();
    touch::<RuntimeFreezeReason>();
    touch::<RuntimeTask>();
    touch::<RuntimeViewFunction>();

    let id = ViewFunctionId {
        prefix: [0u8; 16],
        suffix: [0u8; 16],
    };
    assert!(Test::execute_view_function(id, vec![]).is_err());
}
