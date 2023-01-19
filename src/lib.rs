use scrypto::prelude::*;

//const BASE_FACTOR: Decimal = Decimal::from("0.003");

blueprint! {
    struct Ociswap {
        // LP tokens mint badge.
        lp_badge: Vault,
        // The fee to apply for every swap
        // With fee = BASE_FACTOR * craw_step
        base_fee: Decimal,
        // [TODO] Variable fee will be added later.
        // XRD vault.
        xrd_fee: Vault,
        // For each craw ID a price is associated.
        // With: active_craw = log(price) / log(1 + craw_step) + 2^23
        active_craw: Decimal,
        craw_step: Decimal,
        // The reserve for token A and token B
        a_craws: HashMap<Decimal, Vault>,
        // [TODO] Check crate bimap v0.6.2
        a_lp_craws: HashMap<Decimal, ResourceAddress>,
        b_craws: HashMap<Decimal, Vault>,
        b_lp_craws: HashMap<Decimal, ResourceAddress>,

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
            craw_step: Decimal
        ) -> ComponentAddress {
            // Performing the checks to see if this liquidity pool may be created or not.
            assert!(
                (craw_step >= Decimal::zero()) & (craw_step <= dec!("100")),
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
            //let active_craw = log(price) / log(1.into() + craw_step) + 2.into().powi(23);
            let active_craw = Decimal::from(200) + price;

            // Instantiate our Ociswap component
            let ociswap = (Self {
                lp_badge: Vault::with_bucket(lp_badge),
                base_fee: Decimal::from("0.003") * craw_step, // BASE_FACTOR * 30
                xrd_fee: Vault::new(RADIX_TOKEN),
                active_craw,
                craw_step,
                a_craws: HashMap::new(),
                b_craws: HashMap::new(),
                a_lp_craws: HashMap::new(),
                b_lp_craws: HashMap::new(),
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
            // [TODO] Check borrow.
            let mut buckets: (Bucket, Bucket) = if
                a_tokens.resource_address().to_vec() > b_tokens.resource_address().to_vec()
            {
                (a_tokens, b_tokens)
            } else {
                (b_tokens, a_tokens)
            };

            let price: Decimal = self.get_price(self.active_craw);
            //[TODO] When to calculate LP

            // LP to mint
            // L = p * x + y
            //let supply_to_mint = price * buckets.0.amount() + buckets.1.amount();
            // You get back later: L * reserves / totalL with reserves getBin(ID)

            // [TODO] Range = (log(priceSup) - log(priceInf)) / log(1 + binStep)
            //let range = log(price_sup - price_inf) / log(dec!(1) + self.craw_step);
            let range = price_sup - price_inf;
            // [TODO] Round down.
            // [TODO] Put a limit to range for gas.

            let mut inf_id = self.get_id(price_inf);
            // [TODO] Round down.

            let b1_per_caw = buckets.0.amount() / (range / 2);
            let b2_per_caw = buckets.1.amount() / (range / 2);

            let mut lp_tokens: Vec<Bucket> = Vec::new();

            // [TODO] Decimal to integer
            let range = 3;
            for _ in 0..range {
                // Craws are created when needed.
                if !self.a_craws.contains_key(&inf_id) {
                    // self.a_craws.insert(price, Vault::new(a_tokens.resource_address()));
                    // self.b_craws.insert(price, Vault::new(b_tokens.resource_address()));
                    let lp_addresss = self.create_lp_token();
                    if inf_id <= self.active_craw {
                        self.a_craws.insert(inf_id, Vault::with_bucket(buckets.0.take(b1_per_caw)));
                        // Create LP token for thid ID.
                        self.a_lp_craws.insert(inf_id, lp_addresss);
                    }
                    if inf_id >= self.active_craw {
                        self.b_craws.insert(inf_id, Vault::with_bucket(buckets.1.take(b2_per_caw)));
                        // Create LP token for thid ID.
                        self.a_lp_craws.insert(inf_id, lp_addresss);
                    }
                } else {
                    // Get Vault for that ID and add token.
                    if inf_id <= self.active_craw {
                        self.a_craws.get_mut(&inf_id).unwrap().put(buckets.0.take(b1_per_caw));
                    }
                    if inf_id >= self.active_craw {
                        self.b_craws.get_mut(&inf_id).unwrap().put(buckets.1.take(b1_per_caw));
                    }
                }

                // Get the resource manager of the lp tokens
                // Mint LP tokens according to the share the provider is contributing
                if inf_id <= self.active_craw {
                    let lp_a_resource_address = self.a_lp_craws.get(&inf_id).unwrap();
                    let lp_a_resource_manager = borrow_resource_manager!(*lp_a_resource_address);
                    let lp_a_tokens = self.lp_badge.authorize(||
                        lp_a_resource_manager.mint(price * b1_per_caw)
                    );
                    lp_tokens.push(lp_a_tokens);
                }
                if inf_id >= self.active_craw {
                    let lp_b_resource_address = self.b_lp_craws.get(&inf_id).unwrap();
                    let lp_b_resource_manager = borrow_resource_manager!(*lp_b_resource_address);
                    let lp_b_tokens = self.lp_badge.authorize(||
                        lp_b_resource_manager.mint(b2_per_caw)
                    );
                    lp_tokens.push(lp_b_tokens);
                }

                inf_id += 1;
            }

            // Return the LP tokens
            lp_tokens
            // [TODO] Do we return Vec<Bucket>.
        }

        /// Removes liquidity from this pool.
        pub fn remove_liquidity(&mut self, lp_tokens: Bucket) -> (Bucket, Bucket) {
            // assert!(
            //     self.lp_resource_def == lp_tokens.resource_address(),
            //     "Wrong token type passed in"
            // );

            let lp_tokens_address = lp_tokens.resource_address();
            // [Check] HashMap : .get_many_mut(
            // [TODO] Also check with b.
            let id: Decimal = *self.a_lp_craws
                .iter()
                .find_map(|(key, &val)| if val == lp_tokens_address { Some(key) } else { None })
                .unwrap();

            let craw = self.a_craws.get_mut(&id).unwrap();

            // Return the withdrawn tokens
            (craw.take(1), craw.take(1))
        }

        // Swaps token A for B, or vice versa.
        pub fn swap(&mut self, input_tokens: Bucket) -> Bucket {
            // Calculate the swap fee
            let fee_amount = input_tokens.amount() * self.base_fee;

            self.xrd_fee.take(fee_amount)
        }

        // Creates an LP token for a craw
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
            // let price: Decimal = (dec!(1) + self.craw_step)*(self.active_craw - dec!(2).powi(23));

            price
        }

        // Returns the ID for a certain price
        fn get_id(&mut self, price: Decimal) -> Decimal {
            //let id = log(price) / log(1.into() + self.craw_step) + 2.into().powi(23);

            //id
            price
        }
        // This function is to add tokens in a specific craw
        pub fn add_specific_liquidity(&mut self, mut tokens: Bucket, id: Decimal) -> Bucket {
            tokens.take(id)
        }
    }
}

// Trader Joe Docs : https://docs.traderjoexyz.com/concepts/bin-liquidty
// Trader Joe Whitepaper : https://github.com/traderjoe-xyz/LB-Whitepaper/blob/main/Joe%20v2%20Liquidity%20Book%20Whitepaper.pdf