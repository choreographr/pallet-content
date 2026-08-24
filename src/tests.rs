//! Tests for referenda template pallet.

use super::{Error, Event, Nonce, Pallet as Content};
use crate::{
    mock::*, Error as ContentError, IpfsHash, Item, ItemId, ItemState, RETRACTABLE, RETRACTED,
    REVISIONABLE,
};
use codec::Encode;
use frame_support::{assert_noop, assert_ok, BoundedVec};
use polkadot_sdk::{frame_support, sp_io};

fn expected_item_id(account: u64, nonce: Nonce) -> ItemId {
    let mut item_id = ItemId::default();
    item_id.0.copy_from_slice(&sp_io::hashing::blake2_256(
        &[account.encode(), nonce.encode(), ITEM_ID_NAMESPACE.encode()].concat(),
    ));
    item_id
}

#[test]
fn publish_item() {
    let item_id = expected_item_id(1, Nonce::default());
    new_test_ext().execute_with(|| {
        let mentions: BoundedVec<u64, <Test as crate::Config>::MaxMentions> =
            vec![2, 3].try_into().expect("mentions fit in bounds");

        assert_ok!(Content::<Test>::publish_item(
            RuntimeOrigin::signed(1),
            Nonce::default(),
            Default::default(),
            REVISIONABLE | RETRACTABLE,
            Default::default(),
            mentions.clone(),
            IpfsHash::default()
        ));
        System::assert_has_event(
            Event::<Test>::PublishItem {
                item_id: item_id.clone(),
                owner: 1,
                parents: Default::default(),
                flags: REVISIONABLE | RETRACTABLE,
            }
            .into(),
        );
        System::assert_has_event(
            Event::<Test>::PublishRevision {
                item_id: item_id.clone(),
                owner: 1,
                revision_id: 0,
                links: Default::default(),
                mentions,
                ipfs_hash: IpfsHash::default(),
            }
            .into(),
        );

        let item = Content::<Test>::item(item_id);

        assert!(
            item == Some(Item {
                owner: 1,
                revision_id: 0,
                flags: REVISIONABLE | RETRACTABLE,
            })
        );
    });
    new_test_ext().execute_with(|| {
        assert_ok!(Content::<Test>::publish_item(
            RuntimeOrigin::signed(1),
            Nonce::default(),
            Default::default(),
            REVISIONABLE,
            Default::default(),
            Default::default(),
            IpfsHash::default()
        ));
        assert_noop!(
            Content::<Test>::publish_item(
                RuntimeOrigin::signed(1),
                Nonce::default(),
                Default::default(),
                REVISIONABLE,
                Default::default(),
                Default::default(),
                IpfsHash::default()
            ),
            Error::<Test>::ItemAlreadyExists
        );
    });
}

#[test]
fn publish_revision() {
    let item_id = expected_item_id(1, Nonce::default());
    let ipfs_hash = IpfsHash([
        2, 171, 77, 116, 200, 110, 195, 179, 153, 122, 79, 173, 243, 62, 85, 232, 39, 150, 80, 200,
        83, 158, 166, 126, 5, 60, 2, 220, 44, 253, 243, 52,
    ]);
    new_test_ext().execute_with(|| {
        let mentions: BoundedVec<u64, <Test as crate::Config>::MaxMentions> =
            vec![2].try_into().expect("mentions fit in bounds");

        assert_ok!(Content::<Test>::publish_item(
            RuntimeOrigin::signed(1),
            Nonce::default(),
            Default::default(),
            REVISIONABLE,
            Default::default(),
            Default::default(),
            IpfsHash::default()
        ));
        let item = Content::<Test>::item(&item_id);
        assert!(
            item == Some(Item {
                owner: 1,
                revision_id: 0,
                flags: REVISIONABLE,
            })
        );
        assert_ok!(Content::<Test>::publish_revision(
            RuntimeOrigin::signed(1),
            item_id.clone(),
            Default::default(),
            mentions.clone(),
            ipfs_hash.clone(),
        ));
        let item = Content::<Test>::item(&item_id);
        assert!(
            item == Some(Item {
                owner: 1,
                revision_id: 1,
                flags: REVISIONABLE,
            })
        );
        System::assert_has_event(
            Event::<Test>::PublishRevision {
                item_id: item_id.clone(),
                owner: 1,
                revision_id: 1,
                links: Default::default(),
                mentions,
                ipfs_hash: ipfs_hash.clone(),
            }
            .into(),
        );
    });
    new_test_ext().execute_with(|| {
        assert_ok!(Content::<Test>::publish_item(
            RuntimeOrigin::signed(1),
            Nonce::default(),
            Default::default(),
            REVISIONABLE,
            Default::default(),
            Default::default(),
            IpfsHash::default()
        ));
        assert_noop!(
            Content::<Test>::publish_revision(
                RuntimeOrigin::signed(2),
                item_id.clone(),
                Default::default(),
                Default::default(),
                ipfs_hash.clone(),
            ),
            Error::<Test>::WrongAccount
        );
    });
    new_test_ext().execute_with(|| {
        assert_ok!(Content::<Test>::publish_item(
            RuntimeOrigin::signed(1),
            Nonce::default(),
            Default::default(),
            REVISIONABLE | RETRACTABLE,
            Default::default(),
            Default::default(),
            IpfsHash::default()
        ));
        assert_ok!(Content::<Test>::retract_item(
            RuntimeOrigin::signed(1),
            item_id.clone(),
        ));
        assert_noop!(
            Content::<Test>::publish_revision(
                RuntimeOrigin::signed(1),
                item_id.clone(),
                Default::default(),
                Default::default(),
                ipfs_hash.clone(),
            ),
            Error::<Test>::ItemRetracted
        );
    });
    new_test_ext().execute_with(|| {
        assert_ok!(Content::<Test>::publish_item(
            RuntimeOrigin::signed(1),
            Nonce::default(),
            Default::default(),
            0,
            Default::default(),
            Default::default(),
            IpfsHash::default()
        ));
        assert_noop!(
            Content::<Test>::publish_revision(
                RuntimeOrigin::signed(1),
                item_id.clone(),
                Default::default(),
                Default::default(),
                ipfs_hash.clone(),
            ),
            Error::<Test>::ItemNotRevisionable
        );
    });
}

#[test]
fn publish_revision_rejects_missing_item() {
    new_test_ext().execute_with(|| {
        assert_noop!(
            Content::<Test>::publish_revision(
                RuntimeOrigin::signed(1),
                ItemId([7; 32]),
                Default::default(),
                Default::default(),
                IpfsHash::default(),
            ),
            Error::<Test>::ItemNotFound
        );
    });
}

#[test]
fn retract_item() {
    let item_id = expected_item_id(1, Nonce::default());
    new_test_ext().execute_with(|| {
        assert_ok!(Content::<Test>::publish_item(
            RuntimeOrigin::signed(1),
            Nonce::default(),
            Default::default(),
            RETRACTABLE,
            Default::default(),
            Default::default(),
            IpfsHash::default()
        ));
        let item = Content::<Test>::item(&item_id);
        assert!(
            item == Some(Item {
                owner: 1,
                revision_id: 0,
                flags: RETRACTABLE,
            })
        );
        assert_ok!(Content::<Test>::retract_item(
            RuntimeOrigin::signed(1),
            item_id.clone(),
        ));
        let item = Content::<Test>::item(&item_id);
        assert!(
            item == Some(Item {
                owner: 1,
                revision_id: 0,
                flags: RETRACTED,
            })
        );
        System::assert_has_event(
            Event::<Test>::RetractItem {
                item_id: item_id.clone(),
                owner: 1,
            }
            .into(),
        );
    });
    new_test_ext().execute_with(|| {
        assert_ok!(Content::<Test>::publish_item(
            RuntimeOrigin::signed(1),
            Nonce::default(),
            Default::default(),
            RETRACTABLE,
            Default::default(),
            Default::default(),
            IpfsHash::default()
        ));
        assert_noop!(
            Content::<Test>::retract_item(RuntimeOrigin::signed(2), item_id.clone()),
            Error::<Test>::WrongAccount
        );
    });
    new_test_ext().execute_with(|| {
        assert_ok!(Content::<Test>::publish_item(
            RuntimeOrigin::signed(1),
            Nonce::default(),
            Default::default(),
            RETRACTABLE,
            Default::default(),
            Default::default(),
            IpfsHash::default()
        ));
        assert_ok!(Content::<Test>::retract_item(
            RuntimeOrigin::signed(1),
            item_id.clone(),
        ));
        assert_noop!(
            Content::<Test>::retract_item(RuntimeOrigin::signed(1), item_id.clone()),
            Error::<Test>::ItemRetracted
        );
    });
    new_test_ext().execute_with(|| {
        assert_ok!(Content::<Test>::publish_item(
            RuntimeOrigin::signed(1),
            Nonce::default(),
            Default::default(),
            0,
            Default::default(),
            Default::default(),
            IpfsHash::default()
        ));
        assert_noop!(
            Content::<Test>::retract_item(RuntimeOrigin::signed(1), item_id.clone()),
            Error::<Test>::ItemNotRetractable
        );
    });
}

#[test]
fn retract_item_rejects_missing_item() {
    new_test_ext().execute_with(|| {
        assert_noop!(
            Content::<Test>::retract_item(RuntimeOrigin::signed(1), ItemId([7; 32])),
            Error::<Test>::ItemNotFound
        );
    });
}

#[test]
fn set_not_revisionable() {
    let item_id = expected_item_id(1, Nonce::default());
    new_test_ext().execute_with(|| {
        assert_ok!(Content::<Test>::publish_item(
            RuntimeOrigin::signed(1),
            Nonce::default(),
            Default::default(),
            RETRACTABLE | REVISIONABLE,
            Default::default(),
            Default::default(),
            IpfsHash::default()
        ));
        let item = Content::<Test>::item(&item_id);
        assert!(
            item == Some(Item {
                owner: 1,
                revision_id: 0,
                flags: RETRACTABLE | REVISIONABLE,
            })
        );
        assert_ok!(Content::<Test>::set_not_revisionable(
            RuntimeOrigin::signed(1),
            item_id.clone(),
        ));
        let item = Content::<Test>::item(&item_id);
        assert!(
            item == Some(Item {
                owner: 1,
                revision_id: 0,
                flags: RETRACTABLE,
            })
        );
        System::assert_has_event(
            Event::<Test>::SetNotRevsionable {
                item_id: item_id.clone(),
                owner: 1,
            }
            .into(),
        );
    });
    new_test_ext().execute_with(|| {
        assert_ok!(Content::<Test>::publish_item(
            RuntimeOrigin::signed(1),
            Nonce::default(),
            Default::default(),
            RETRACTABLE | REVISIONABLE,
            Default::default(),
            Default::default(),
            IpfsHash::default()
        ));
        assert_noop!(
            Content::<Test>::set_not_revisionable(RuntimeOrigin::signed(2), item_id.clone()),
            Error::<Test>::WrongAccount
        );
    });
    new_test_ext().execute_with(|| {
        assert_ok!(Content::<Test>::publish_item(
            RuntimeOrigin::signed(1),
            Nonce::default(),
            Default::default(),
            RETRACTABLE,
            Default::default(),
            Default::default(),
            IpfsHash::default()
        ));
        assert_noop!(
            Content::<Test>::set_not_revisionable(RuntimeOrigin::signed(1), item_id.clone()),
            Error::<Test>::ItemNotRevisionable
        );
    });
}

#[test]
fn set_not_revisionable_rejects_missing_item() {
    new_test_ext().execute_with(|| {
        assert_noop!(
            Content::<Test>::set_not_revisionable(RuntimeOrigin::signed(1), ItemId([7; 32])),
            Error::<Test>::ItemNotFound
        );
    });
}

#[test]
fn set_not_retractable() {
    let item_id = expected_item_id(1, Nonce::default());
    new_test_ext().execute_with(|| {
        assert_ok!(Content::<Test>::publish_item(
            RuntimeOrigin::signed(1),
            Nonce::default(),
            Default::default(),
            RETRACTABLE | REVISIONABLE,
            Default::default(),
            Default::default(),
            IpfsHash::default()
        ));
        let item = Content::<Test>::item(&item_id);
        assert!(
            item == Some(Item {
                owner: 1,
                revision_id: 0,
                flags: RETRACTABLE | REVISIONABLE,
            })
        );
        assert_ok!(Content::<Test>::set_not_retractable(
            RuntimeOrigin::signed(1),
            item_id.clone(),
        ));
        let item = Content::<Test>::item(&item_id);
        assert!(
            item == Some(Item {
                owner: 1,
                revision_id: 0,
                flags: REVISIONABLE,
            })
        );
        System::assert_has_event(
            Event::<Test>::SetNotRetractable {
                item_id: item_id.clone(),
                owner: 1,
            }
            .into(),
        );
    });
    new_test_ext().execute_with(|| {
        assert_ok!(Content::<Test>::publish_item(
            RuntimeOrigin::signed(1),
            Nonce::default(),
            Default::default(),
            RETRACTABLE | REVISIONABLE,
            Default::default(),
            Default::default(),
            IpfsHash::default()
        ));
        assert_noop!(
            Content::<Test>::set_not_retractable(RuntimeOrigin::signed(2), item_id.clone()),
            Error::<Test>::WrongAccount
        );
    });
    new_test_ext().execute_with(|| {
        assert_ok!(Content::<Test>::publish_item(
            RuntimeOrigin::signed(1),
            Nonce::default(),
            Default::default(),
            REVISIONABLE,
            Default::default(),
            Default::default(),
            IpfsHash::default()
        ));
        assert_noop!(
            Content::<Test>::set_not_retractable(RuntimeOrigin::signed(1), item_id.clone()),
            Error::<Test>::ItemNotRetractable
        );
    });
}

#[test]
fn set_not_retractable_rejects_missing_item() {
    new_test_ext().execute_with(|| {
        assert_noop!(
            Content::<Test>::set_not_retractable(RuntimeOrigin::signed(1), ItemId([7; 32])),
            Error::<Test>::ItemNotFound
        );
    });
}

#[test]
fn reject_invalid_flags() {
    new_test_ext().execute_with(|| {
        assert_noop!(
            Content::<Test>::publish_item(
                RuntimeOrigin::signed(1),
                Nonce::default(),
                Default::default(),
                RETRACTED,
                Default::default(),
                Default::default(),
                IpfsHash::default()
            ),
            ContentError::<Test>::InvalidFlags
        );
    });
}

#[test]
fn reject_revision_id_overflow() {
    let item_id = expected_item_id(1, Nonce::default());

    new_test_ext().execute_with(|| {
        assert_ok!(Content::<Test>::publish_item(
            RuntimeOrigin::signed(1),
            Nonce::default(),
            Default::default(),
            REVISIONABLE,
            Default::default(),
            Default::default(),
            IpfsHash::default()
        ));

        ItemState::<Test>::insert(
            item_id.clone(),
            Item {
                owner: 1,
                revision_id: u32::MAX,
                flags: REVISIONABLE,
            },
        );

        assert_noop!(
            Content::<Test>::publish_revision(
                RuntimeOrigin::signed(1),
                item_id,
                Default::default(),
                Default::default(),
                IpfsHash::default(),
            ),
            ContentError::<Test>::RevisionIdOverflow
        );
    });
}

#[test]
fn publish_item_with_parents_and_links() {
    let item_id = expected_item_id(1, Nonce::default());
    new_test_ext().execute_with(|| {
        let parents: BoundedVec<ItemId, <Test as crate::Config>::MaxParents> =
            vec![ItemId([1; 32]), ItemId([2; 32])]
                .try_into()
                .expect("parents fit in bounds");
        let links: BoundedVec<ItemId, <Test as crate::Config>::MaxLinks> =
            vec![ItemId([3; 32]), ItemId([4; 32])]
                .try_into()
                .expect("links fit in bounds");

        assert_ok!(Content::<Test>::publish_item(
            RuntimeOrigin::signed(1),
            Nonce::default(),
            parents.clone(),
            REVISIONABLE | RETRACTABLE,
            links.clone(),
            Default::default(),
            IpfsHash::default(),
        ));

        // The populated `parents` must be emitted on `PublishItem`.
        System::assert_has_event(
            Event::<Test>::PublishItem {
                item_id: item_id.clone(),
                owner: 1,
                parents: parents.clone(),
                flags: REVISIONABLE | RETRACTABLE,
            }
            .into(),
        );

        // The populated `links` must be emitted on the initial `PublishRevision`.
        System::assert_has_event(
            Event::<Test>::PublishRevision {
                item_id: item_id.clone(),
                owner: 1,
                revision_id: 0,
                links: links.clone(),
                mentions: Default::default(),
                ipfs_hash: IpfsHash::default(),
            }
            .into(),
        );
    });
}

#[test]
fn publish_revision_with_links() {
    let item_id = expected_item_id(1, Nonce::default());
    new_test_ext().execute_with(|| {
        assert_ok!(Content::<Test>::publish_item(
            RuntimeOrigin::signed(1),
            Nonce::default(),
            Default::default(),
            REVISIONABLE,
            Default::default(),
            Default::default(),
            IpfsHash::default(),
        ));

        let links: BoundedVec<ItemId, <Test as crate::Config>::MaxLinks> =
            vec![ItemId([5; 32]), ItemId([6; 32])]
                .try_into()
                .expect("links fit in bounds");

        assert_ok!(Content::<Test>::publish_revision(
            RuntimeOrigin::signed(1),
            item_id.clone(),
            links.clone(),
            Default::default(),
            IpfsHash::default(),
        ));

        System::assert_has_event(
            Event::<Test>::PublishRevision {
                item_id: item_id.clone(),
                owner: 1,
                revision_id: 1,
                links: links.clone(),
                mentions: Default::default(),
                ipfs_hash: IpfsHash::default(),
            }
            .into(),
        );
    });
}
