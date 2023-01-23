use scrypto::prelude::*;

//const BASE_FACTOR: Decimal = Decimal::from("0.003");

#[scrypto(Debug, TypeId, Encode, Decode, Describe)]
pub struct Bin {
    bin_id: Decimal,
    bin_vault: Vault,
    bin_lp_address: ResourceAddress,
    bin_total_lp: Decimal,
}

impl Bin {
    pub fn new(
        bin_id: Decimal,
        bin_vault: Vault,
        bin_lp_address: ResourceAddress,
        bin_total_lp: Decimal
    ) -> Self {
        Self {
            bin_id,
            bin_vault,
            bin_lp_address,
            bin_total_lp,
        }
    }
}

blueprint! {
    struct Ociswap {
        // LP tokens mint badge.
        lp_badge: Vault,
        // The fee to apply for every swap
        // With fee = BASE_FACTOR * bin_step
        base_fee: Decimal,
        // [TODO] Variable fee will be added later.
        // XRD vault.
        xrd_fee: Vault,

        // For each bin ID a price is associated.
        // With: active_bin = log(price) / log(1 + bin_step) + 2^23
        active_bin: Decimal,
        bin_step: Decimal,

        // The reserve for token A and token B
        // [TODO] Check crate bimap v0.6.2
        a_bins: HashMap<Decimal, Bin>,
        a_lp_id: HashMap<ResourceAddress, Decimal>, // [TODO]

        b_bins: HashMap<Decimal, Bin>,
        b_lp_id: HashMap<ResourceAddress, Decimal>, // [TODO]

        // [TODO] Do we add both addresses for checks when adding liquidity.
        a_token_address: ResourceAddress,
        b_token_address: ResourceAddress,
    }

    impl Ociswap {
        /// Creates a Ociswap component for token pair A/B and returns the component address
        /// along with the initial LP tokens.
        pub fn instantiate_pool(
            a_token_address: ResourceAddress,
            b_token_address: ResourceAddress, // Not a Bucket
            price: Decimal,
            bin_step: Decimal
        ) -> ComponentAddress {
            // Performing the checks to see if this liquidity pool may be created or not.
            assert!(
                (bin_step >= Decimal::zero()) & (bin_step <= dec!("100")),
                "[Pool Creation]: Fee must be between 0 and 100"
            );
            // We will add an enum for fees.
            // [Check] 0 is Uniswap V2

            assert_ne!(
                a_token_address,
                b_token_address,
                "[Pool Creation]: Liquidity pools may only be created between two different tokens."
            );
            // At this point, we know that the pool creation can indeed go through.

            // Instantiate our LP token and mint an initial supply of them
            let lp_badge = ResourceBuilder::new_fungible()
                .divisibility(DIVISIBILITY_NONE)
                .metadata("name", "LP Token Mint Auth")
                .initial_supply(1);

            // [TODO] Check with Flo the log function.
            //let active_bin = log(price) / log(1.into() + bin_step) + 2.into().powi(23);
            let active_bin = Decimal::from(200) + price;

            // Instantiate our Ociswap component
            let ociswap = (Self {
                lp_badge: Vault::with_bucket(lp_badge),
                base_fee: Decimal::from("0.003") * bin_step, // BASE_FACTOR * 30
                xrd_fee: Vault::new(RADIX_TOKEN),

                active_bin,
                bin_step,

                a_bins: HashMap::new(),
                b_bins: HashMap::new(),
                a_lp_id: HashMap::new(), // [TODO]
                b_lp_id: HashMap::new(), // [TODO]

                a_token_address,
                b_token_address,
            }).instantiate();

            // [TODO] ociswap.add_access_check(access_rules);

            // Return the new Ociswap component
            ociswap.globalize()
        }

        /// Adds liquidity to this pool and return the LP tokens representing pool shares
        /// along with any remainder. [Check] No remainer for LB
        pub fn add_liquidity(
            &mut self,
            a_tokens: Bucket, //mut a_tokens: Bucket,
            b_tokens: Bucket, //mut b_tokens: Bucket,
            price_inf: Decimal,
            price_sup: Decimal
        ) -> Vec<Bucket> {
            // No remainer

            assert!(
                !a_tokens.is_empty() & !b_tokens.is_empty(),
                "[Pool Creation]: Can't create a pool from an empty bucket."
            );
            // [Check] Maybe enable one of tokens amount to be 0

            // Sorting the buckets and then creating the Hashmap of the vaults from the sorted buckets
            // [TODO] Check borrow and address of b1 and b2.
            let mut buckets: (Bucket, Bucket) = if
                a_tokens.resource_address().to_vec() > b_tokens.resource_address().to_vec()
            {
                (a_tokens, b_tokens)
            } else {
                (b_tokens, a_tokens)
            };

            // LP to mint
            // L = p * x + y
            //let supply_to_mint = price * buckets.0.amount() + buckets.1.amount();
            // You get back later: L * reserves / totalL with reserves getBin(ID)

            // [TODO] Range = (log(priceSup) - log(priceInf)) / log(1 + binStep)
            //let range = log(price_sup - price_inf) / log(dec!(1) + self.bin_step);
            let range = price_sup - price_inf;
            // [TODO] Round down and integer.
            // [TODO] Put a limit to range for gas.

            let mut inf_id = self.get_id(price_inf);
            // [TODO] Round down.

            // This case is for a normal Shape
            let b1_per_bin = buckets.0.amount() / (range / 2);
            let b2_per_bin = buckets.1.amount() / (range / 2);

            let mut lp_tokens: Vec<Bucket> = Vec::new();

            let range = 3; // 3 to remove
            for _ in 0..range {
                // bins are created when needed.
                if !self.a_bins.contains_key(&inf_id) {
                    // self.a_bins.insert(price, Vault::new(a_tokens.resource_address()));
                    // self.b_bins.insert(price, Vault::new(b_tokens.resource_address()));
                    if inf_id <= self.active_bin {
                        // Create LP token for thid ID.
                        let lp_addresss = self.create_lp_token();
                        self.a_lp_id.insert(lp_addresss, inf_id); // Will be used for remove
                        // Get the resource manager of the lp tokens
                        // Mint LP tokens according to the share the provider is contributing
                        let price: Decimal = self.get_price(self.active_bin);
                        let my_bin = self.a_bins.get(&inf_id).unwrap();
                        let lp_a_resource_manager = borrow_resource_manager!(my_bin.bin_lp_address);
                        let lp_a_tokens = self.lp_badge.authorize(||
                            lp_a_resource_manager.mint(price * b1_per_bin)
                        );

                        let new_bin = Bin::new(
                            inf_id,
                            Vault::with_bucket(buckets.0.take(b1_per_bin)),
                            lp_addresss,
                            lp_a_tokens.amount()
                        );
                        self.a_bins.insert(inf_id, new_bin);

                        lp_tokens.push(lp_a_tokens);
                    }
                    if inf_id >= self.active_bin {
                        // Create LP token for thid ID.
                        let lp_addresss = self.create_lp_token();
                        self.b_lp_id.insert(lp_addresss, inf_id); // Will be used for remove
                        // Get the resource manager of the lp tokens
                        // Mint LP tokens according to the share the provider is contributing
                        let my_bin = self.b_bins.get(&inf_id).unwrap();
                        let lp_b_resource_manager = borrow_resource_manager!(my_bin.bin_lp_address);
                        let lp_b_tokens = self.lp_badge.authorize(||
                            lp_b_resource_manager.mint(b2_per_bin)
                        );

                        let new_bin = Bin::new(
                            inf_id,
                            Vault::with_bucket(buckets.0.take(b1_per_bin)),
                            lp_addresss,
                            lp_b_tokens.amount()
                        );
                        self.b_bins.insert(inf_id, new_bin);

                        lp_tokens.push(lp_b_tokens);
                    }
                } else {
                    // Get Vault for that ID and add token.
                    if inf_id <= self.active_bin {
                        self.a_bins
                            .get_mut(&inf_id)
                            .unwrap()
                            .bin_vault.put(buckets.0.take(b1_per_bin));
                    }
                    if inf_id >= self.active_bin {
                        self.b_bins
                            .get_mut(&inf_id)
                            .unwrap()
                            .bin_vault.put(buckets.1.take(b1_per_bin));
                    }
                }

                inf_id += 1;
            }

            // Return the LP tokens, each Bucket of Vec<Bucket> will be added to the account
            lp_tokens
        }

        /// Removes liquidity from this pool.
        pub fn remove_liquidity(&mut self, lp_tokens: Bucket) -> (Bucket, Bucket) {
            // assert!(
            //     self.lp_resource_def == lp_tokens.resource_address(),
            //     "Wrong token type passed in"
            // );

            let lp_tokens_address = lp_tokens.resource_address();

            // // [Check] HashMap : .get_many_mut(
            // // [TODO] Also check with b.
            // let id: Decimal = *self.a_bins
            //     .iter()
            //     .find_map(|(key, &val)| (
            //         if val.bin_lp_address == lp_tokens_address {
            //             Some(key)
            //         } else {
            //             None
            //         }
            //     ))
            //     .unwrap();

            let id = self.a_lp_id.get(&lp_tokens_address).unwrap();

            let bin = self.a_bins.get_mut(&id).unwrap();

            // Return the withdrawn tokens
            (bin.bin_vault.take(1), bin.bin_vault.take(1))
        }

        // Swaps token A for B, or vice versa.
        pub fn swap(&mut self, input_tokens: Bucket) -> Bucket {
            // Calculate the swap fee
            let fee_amount = input_tokens.amount() * self.base_fee;

            self.xrd_fee.take(fee_amount)
        }

        // Creates an LP token for a bin
        fn create_lp_token(&mut self) -> ResourceAddress {
            let lp_resource_address = ResourceBuilder::new_fungible()
                .divisibility(DIVISIBILITY_MAXIMUM)
                // .metadata("symbol", pair_name)
                // .metadata("name", lp_id)
                .mintable(rule!(require(self.lp_badge.resource_address())), LOCKED)
                .burnable(rule!(require(self.lp_badge.resource_address())), LOCKED)
                .no_initial_supply();

            lp_resource_address
        }

        // Returns the ID of a price
        fn get_price(&mut self, id: Decimal) -> Decimal {
            let price = id;

            // Calculate price (constant sum)
            // p = (1+ binStep)*(activeBin - 2^23) (1+binstep) ^(activeId - 2**23)
            // let price: Decimal = (dec!(1) + self.bin_step)*(self.active_bin - dec!(2).powi(23));

            price
        }

        // Returns the ID for a certain price
        fn get_id(&mut self, price: Decimal) -> Decimal {
            //let id = log(price) / log(1.into() + self.bin_step) + 2.into().powi(23);

            //id
            price
        }
        // This function is to add tokens in a specific bin
        pub fn add_specific_liquidity(&mut self, mut tokens: Bucket, id: Decimal) -> Bucket {
            tokens.take(id)
        }
    }
}

// Trader Joe Docs : https://docs.traderjoexyz.com/concepts/bin-liquidty
// Trader Joe Whitepaper : https://github.com/traderjoe-xyz/LB-Whitepaper/blob/main/Joe%20v2%20Liquidity%20Book%20Whitepaper.pdf