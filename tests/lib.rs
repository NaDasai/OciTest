use radix_engine_interface::model::FromPublicKey;
use scrypto::prelude::*;
use scrypto_unit::*;
use transaction::builder::ManifestBuilder;

use transaction::model::TransactionManifest;
use radix_engine::transaction::TransactionReceipt;
use transaction::model::TestTransaction;

#[test]
fn test_ociswap() {
    let amount_to_swap = dec!(8);
    let opt_bucket_1: Option<Bucket> = None;
    let opt_bucket_2: Option<Bucket> = None;
    let opt_bucket_3: Option<Bucket> = None;
    // Setup the environment
    //let mut store = TypedInMemorySubstateStore::with_bootstrap();
    //let mut test_runner = TestRunner::new(true, &mut store);
    let mut test_runner = TestRunner::builder().without_trace().build();

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

    // Test the `instantiate_pool` function.
    let manifest = ManifestBuilder::new()
        .call_function(
            package_address,
            "Ociswap",
            "instantiate_pool",
            args!(token_a, token_b, dec!(20), dec!("0.003"))
        )
        .build();
    let receipt = test_runner.execute_manifest_with_max_cost_unit_limit(
        manifest,
        vec![NonFungibleGlobalId::from_public_key(&public_key)]
    );
    println!("{:?}\n", receipt);
    receipt.expect_commit_success();
    let component = receipt.expect_commit().entity_changes.new_component_addresses[0];

    println!("First transaction manifest: add_liquidity 1\n");
    //**************************************************************************************************************************************/
    // Test the `add_liquidity` method.
    let manifest = ManifestBuilder::new()
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
                        args!(bucket_id_a, bucket_id_b, dec!("19.5"), dec!("20.5"), opt_bucket_1)
                    )
                }
            )
        })

        .call_method(account_component, "deposit_batch", args!(ManifestExpression::EntireWorktop))
        .build();
    let receipt = test_runner.execute_manifest_with_max_cost_unit_limit(
        manifest,
        vec![NonFungibleGlobalId::from_public_key(&public_key)]
    );

    println!("{:?}\n", receipt);
    receipt.expect_commit_success();

    // //**************************************************************************************************************************************/
    println!("Second transaction manifest: add_liquidity 2\n");
    //**************************************************************************************************************************************/
    // Test the `add_liquidity` method.
    let manifest = ManifestBuilder::new()
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
                        args!(bucket_id_a, bucket_id_b, dec!("19.8"), dec!("20.7"), opt_bucket_2)
                    )
                }
            )
        })

        .call_method(account_component, "deposit_batch", args!(ManifestExpression::EntireWorktop))
        .build();
    let receipt = test_runner.execute_manifest_with_max_cost_unit_limit(
        manifest,
        vec![NonFungibleGlobalId::from_public_key(&public_key)]
    );
    println!("{:?}\n", receipt);
    receipt.expect_commit_success();
    // //**************************************************************************************************************************************/

    // //**************************************************************************************************************************************/
    println!("Third transaction manifest: add_liquidity 3\n");
    //**************************************************************************************************************************************/
    // Test the `add_liquidity` method.

    let mut distribution: Vec<(Decimal, Decimal)> = Vec::new();
    distribution.push((dec!(8389604), dec!(20)));
    distribution.push((dec!(8389605), dec!(20)));
    distribution.push((dec!(8389606), dec!(20)));
    distribution.push((dec!(8389607), dec!(20)));
    //distribution.push((dec!(8389608), dec!(20)));
    distribution.push((dec!(8389608), dec!(8))); // Active Bin
    distribution.push((dec!(8389609), dec!(8)));
    distribution.push((dec!(8389610), dec!(8)));
    distribution.push((dec!(8389611), dec!(8)));
    distribution.push((dec!(8389612), dec!(8)));
    distribution.push((dec!(8389613), dec!(8)));
    distribution.push((dec!(8389614), dec!(8)));
    distribution.push((dec!(8389615), dec!(8)));
    distribution.push((dec!(8389616), dec!(8)));
    distribution.push((dec!(8389617), dec!(8)));
    distribution.push((dec!(8389618), dec!(8)));
    distribution.push((dec!(8389619), dec!(8)));

    let manifest = ManifestBuilder::new()
        .withdraw_from_account_by_amount(account_component, dec!(100), token_a)
        .withdraw_from_account_by_amount(account_component, dec!(100), token_b)
        .take_from_worktop_by_amount(dec!(100), token_a, |continue_transaction, bucket_id_a| {
            continue_transaction.take_from_worktop_by_amount(
                dec!(100),
                token_b,
                |continue_transaction2, bucket_id_b| {
                    continue_transaction2.call_method(
                        component,
                        "add_specific_liquidity",
                        args!(bucket_id_a, bucket_id_b, distribution, opt_bucket_3)
                    )
                }
            )
        })

        .call_method(account_component, "deposit_batch", args!(ManifestExpression::EntireWorktop))
        .build();
    let receipt = test_runner.execute_manifest_with_max_cost_unit_limit(
        manifest,
        vec![NonFungibleGlobalId::from_public_key(&public_key)]
    );
    println!("{:?}\n", receipt);
    receipt.expect_commit_success();
    // //**************************************************************************************************************************************/

    // //**************************************************************************************************************************************/
    println!("Transaction manifest: Swap\n");
    //**************************************************************************************************************************************/
    // Test the `swap` method.
    let manifest = ManifestBuilder::new()
        .withdraw_from_account_by_amount(account_component, dec!(10), token_a)
        .take_from_worktop_by_amount(amount_to_swap, token_a, |continue_transaction, bucket_id_a| {
            continue_transaction.call_method(component, "swap", args!(bucket_id_a))
        })
        .call_method(account_component, "deposit_batch", args!(ManifestExpression::EntireWorktop))
        .build();
    let receipt = test_runner.execute_manifest_with_max_cost_unit_limit(
        manifest,
        vec![NonFungibleGlobalId::from_public_key(&public_key)]
    );
    println!("{:?}\n", receipt);
    receipt.expect_commit_success();
    //**************************************************************************************************************************************/
}

trait ExecuteManifestWithMaxCostUnitLimit {
    fn execute_manifest_with_max_cost_unit_limit(
        &mut self,
        manifest: TransactionManifest,
        initial_proofs: Vec<NonFungibleGlobalId>
    ) -> TransactionReceipt;
}

impl ExecuteManifestWithMaxCostUnitLimit for TestRunner {
    fn execute_manifest_with_max_cost_unit_limit(
        &mut self,
        mut manifest: TransactionManifest,
        initial_proofs: Vec<NonFungibleGlobalId>
    ) -> TransactionReceipt {
        manifest.instructions.insert(0, transaction::model::BasicInstruction::CallMethod {
            component_address: FAUCET_COMPONENT,
            method_name: "lock_fee".to_string(),
            args: args!(dec!("1000000")), // Note that I'm locking 1 million XRD here which should be much more than enough.
        });

        let transaction = TestTransaction::new(manifest, self.next_transaction_nonce(), u32::MAX);
        let executable = transaction.get_executable(initial_proofs);
        self.execute_transaction(executable)
    }
}

// https://github.com/radixdlt/radixdlt-scrypto/blob/main/scrypto-unit/src/test_runner.rs