use radix_engine::ledger::*;
use radix_engine_interface::core::NetworkDefinition;
use radix_engine_interface::model::FromPublicKey;
use scrypto::prelude::*;
use scrypto_unit::*;
use transaction::builder::ManifestBuilder;

#[test]
fn test_ociswap() {
    // Setup the environment
    let mut store = TypedInMemorySubstateStore::with_bootstrap();
    let mut test_runner = TestRunner::new(true, &mut store);

    // Create an account
    let (public_key, _private_key, account_component) = test_runner.new_allocated_account();

    // Publish package
    let package_address = test_runner.compile_and_publish(this_package!());

    let token_a = test_runner.create_fungible_resource(
        dec!(1000),
        DIVISIBILITY_MAXIMUM,
        account_component
    );
    let token_b = test_runner.create_fungible_resource(
        dec!(1000),
        DIVISIBILITY_MAXIMUM,
        account_component
    );

    // pub fn instantiate_pool(
    //     a_token_address: ResourceAddress,
    //     b_token_address: ResourceAddress, // Not a Bucket
    //     price: Decimal,
    //     bin_step: Decimal
    // ) -> ComponentAddress;
    // Test the `instantiate_pool` function.
    let manifest = ManifestBuilder::new(&NetworkDefinition::simulator())
        .call_function(
            package_address,
            "Ociswap",
            "instantiate_pool",
            args!(token_a, token_b, dec!(20), dec!("0.003"))
        )
        .build();
    let receipt = test_runner.execute_manifest_ignoring_fee(
        manifest,
        vec![NonFungibleAddress::from_public_key(&public_key)]
    );
    println!("{:?}\n", receipt);
    receipt.expect_commit_success();
    let component = receipt.expect_commit().entity_changes.new_component_addresses[0];

    // let resources = test_runner.get_component_resources(component);
    // for (key, value) in &resources {
    //     println!("Resource {:?}: {}", key, value);
    // }

    println!("First transaction manifest: add_liquidity 1\n");
    //**************************************************************************************************************************************/
    // Test the `add_liquidity` method.
    let manifest = ManifestBuilder::new(&NetworkDefinition::simulator())
        .withdraw_from_account_by_amount(account_component, dec!(500), token_a)
        .withdraw_from_account_by_amount(account_component, dec!(500), token_b)
        .take_from_worktop_by_amount(dec!(500), token_a, |continue_transaction, bucket_id_a| {
            continue_transaction.take_from_worktop_by_amount(
                dec!(500),
                token_b,
                |continue_transaction2, bucket_id_b| {
                    continue_transaction2.call_method(
                        component,
                        "add_liquidity",
                        args!(Bucket(bucket_id_a), Bucket(bucket_id_b), dec!("19.5"), dec!("20.5"))
                    )
                }
            )
        })

        .call_method(account_component, "deposit_batch", args!(Expression::entire_worktop()))
        .build();
    let receipt = test_runner.execute_manifest_ignoring_fee(
        manifest,
        vec![NonFungibleAddress::from_public_key(&public_key)]
    );
    println!("{:?}\n", receipt);
    receipt.expect_commit_success();

    //**************************************************************************************************************************************/
    println!("Second transaction manifest: add_liquidity 2\n");
    //**************************************************************************************************************************************/
    // Test the `add_liquidity` method.
    let manifest = ManifestBuilder::new(&NetworkDefinition::simulator())
        .withdraw_from_account_by_amount(account_component, dec!(100), token_a)
        .withdraw_from_account_by_amount(account_component, dec!(100), token_b)
        .take_from_worktop_by_amount(dec!(100), token_a, |continue_transaction, bucket_id_a| {
            continue_transaction.take_from_worktop_by_amount(
                dec!(100),
                token_b,
                |continue_transaction2, bucket_id_b| {
                    continue_transaction2.call_method(
                        component,
                        "add_liquidity",
                        args!(Bucket(bucket_id_a), Bucket(bucket_id_b), dec!("19.8"), dec!("20.7"))
                    )
                }
            )
        })

        .call_method(account_component, "deposit_batch", args!(Expression::entire_worktop()))
        .build();
    let receipt = test_runner.execute_manifest_ignoring_fee(
        manifest,
        vec![NonFungibleAddress::from_public_key(&public_key)]
    );
    println!("{:?}\n", receipt);
    receipt.expect_commit_success();
    //**************************************************************************************************************************************/

    //**************************************************************************************************************************************/
    println!("Third transaction manifest: Swap\n");
    //**************************************************************************************************************************************/
    // Test the `swap` method.
    let manifest = ManifestBuilder::new(&NetworkDefinition::simulator())
        .withdraw_from_account_by_amount(account_component, dec!(10), token_a)
        .take_from_worktop_by_amount(dec!(2), token_a, |continue_transaction, bucket_id_a| {
            continue_transaction.call_method(component, "swap", args!(Bucket(bucket_id_a)))
        })
        .call_method(account_component, "deposit_batch", args!(Expression::entire_worktop()))
        .build();
    let receipt = test_runner.execute_manifest_ignoring_fee(
        manifest,
        vec![NonFungibleAddress::from_public_key(&public_key)]
    );
    println!("{:?}\n", receipt);
    receipt.expect_commit_success();
    //**************************************************************************************************************************************/
}

// https://github.com/radixdlt/radixdlt-scrypto/blob/main/scrypto-unit/src/test_runner.rs