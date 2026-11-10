use crate::{mock::*, AccountProfile, Error, Event, Pallet as AccountProfilePallet};
use pallet_content::{IpfsHash, Item, ItemId, Nonce, Pallet as Content, RETRACTED};
use polkadot_sdk::frame_support::{assert_noop, assert_ok};

const REVISIONABLE: u8 = 1 << 0;

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
        Item {
            owner,
            revision_id: 0,
            flags: REVISIONABLE,
        },
    );
    item_id
}

#[test]
fn set_profile_records_membership_and_event() {
    new_test_ext().execute_with(|| {
        let item_id = publish_item(1, Nonce::default());

        assert_ok!(AccountProfilePallet::<Test>::set_profile(
            RuntimeOrigin::signed(1),
            item_id.clone()
        ));

        assert_eq!(AccountProfile::<Test>::get(1), Some(item_id.clone()));
        assert_eq!(
            AccountProfilePallet::<Test>::account_profile(1),
            Some(item_id.clone())
        );

        System::assert_has_event(
            Event::<Test>::ProfileSet {
                account: 1,
                item_id,
            }
            .into(),
        );
    });
}

#[test]
fn set_profile_rejects_missing_or_unowned_content() {
    new_test_ext().execute_with(|| {
        let missing_item_id = ItemId([7; 32]);

        assert_noop!(
            AccountProfilePallet::<Test>::set_profile(
                RuntimeOrigin::signed(1),
                missing_item_id.clone()
            ),
            Error::<Test>::ItemNotFound
        );

        let item_id = insert_owned_item(1, 1);
        assert_noop!(
            AccountProfilePallet::<Test>::set_profile(RuntimeOrigin::signed(2), item_id),
            Error::<Test>::WrongAccount
        );
    });
}

#[test]
fn set_profile_rejects_retracted_content() {
    new_test_ext().execute_with(|| {
        let item_id = ItemId([9; 32]);
        pallet_content::ItemState::<Test>::insert(
            item_id.clone(),
            Item {
                owner: 1,
                revision_id: 0,
                flags: RETRACTED,
            },
        );

        assert_noop!(
            AccountProfilePallet::<Test>::set_profile(RuntimeOrigin::signed(1), item_id),
            Error::<Test>::ItemRetracted
        );
    });
}

#[test]
fn set_profile_overwrites_previous_profile() {
    new_test_ext().execute_with(|| {
        let item_id_1 = insert_owned_item(1, 1);
        let item_id_2 = insert_owned_item(1, 2);

        assert_ok!(AccountProfilePallet::<Test>::set_profile(
            RuntimeOrigin::signed(1),
            item_id_1
        ));
        assert_ok!(AccountProfilePallet::<Test>::set_profile(
            RuntimeOrigin::signed(1),
            item_id_2.clone()
        ));

        assert_eq!(AccountProfile::<Test>::get(1), Some(item_id_2.clone()));

        System::assert_has_event(
            Event::<Test>::ProfileSet {
                account: 1,
                item_id: item_id_2,
            }
            .into(),
        );
    });
}

#[test]
fn set_profile_allows_setting_same_item_again() {
    new_test_ext().execute_with(|| {
        let item_id = insert_owned_item(1, 1);

        assert_ok!(AccountProfilePallet::<Test>::set_profile(
            RuntimeOrigin::signed(1),
            item_id.clone()
        ));
        assert_ok!(AccountProfilePallet::<Test>::set_profile(
            RuntimeOrigin::signed(1),
            item_id.clone()
        ));

        assert_eq!(AccountProfile::<Test>::get(1), Some(item_id.clone()));

        let profile_set_events = System::events()
            .into_iter()
            .filter(|record| {
                matches!(
                    record.event,
                    RuntimeEvent::AccountProfile(Event::ProfileSet { .. })
                )
            })
            .count();

        assert_eq!(profile_set_events, 2);
    });
}

#[test]
fn set_profile_is_scoped_per_account() {
    new_test_ext().execute_with(|| {
        let item_id_1 = insert_owned_item(1, 1);
        let item_id_2 = insert_owned_item(2, 2);

        assert_ok!(AccountProfilePallet::<Test>::set_profile(
            RuntimeOrigin::signed(1),
            item_id_1.clone()
        ));
        assert_ok!(AccountProfilePallet::<Test>::set_profile(
            RuntimeOrigin::signed(2),
            item_id_2.clone()
        ));

        assert_eq!(AccountProfile::<Test>::get(1), Some(item_id_1));
        assert_eq!(AccountProfile::<Test>::get(2), Some(item_id_2));
    });
}

#[test]
fn weight_info_is_positive_for_all_dispatchables() {
    use crate::{SubstrateWeight, WeightInfo};

    fn assert_weight_impl<W: WeightInfo>() {
        // Placeholder weights: assert only that the dispatchable reports a
        // non-zero weight until runtime benchmarks replace this value.
        assert!(W::set_profile().ref_time() > 0);
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

        let set = Call::<Test>::set_profile { item_id };
        assert!(set.get_dispatch_info().call_weight.ref_time() > 0);

        // Dispatch through the runtime so the generated `RuntimeCall` glue runs.
        assert_ok!(
            RuntimeCall::AccountProfile(set).dispatch_bypass_filter(RuntimeOrigin::signed(owner))
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
