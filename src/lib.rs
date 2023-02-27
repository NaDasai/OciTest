use scrypto::prelude::*;
use std::ops::Add;
use std::ops::Div;

#[derive(ScryptoCategorize, ScryptoDecode, ScryptoEncode, LegacyDescribe)]
pub struct Bin {
    bin_id: Decimal,
    bin_vault: Vault,
}

#[derive(NonFungibleData)]
pub struct Lp {
    #[mutable]
    id_lp: HashMap<Decimal, Decimal>,
}

impl Bin {
    pub fn new(bin_id: Decimal, bin_vault: Vault) -> Self {
        Self {
            bin_id,
            bin_vault,
        }
    }
}

#[blueprint]
mod ociswap_module {
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
        a_bins: KeyValueStore<Decimal, Bin>,
        a_lp_id: KeyValueStore<ResourceAddress, Decimal>, // [Remove]

        b_bins: KeyValueStore<Decimal, Bin>,
        b_lp_id: KeyValueStore<ResourceAddress, Decimal>, // [Remove]

        // [Check] Do we add both addresses for checks when adding liquidity.
        a_token_address: ResourceAddress,
        b_token_address: ResourceAddress,

        lp_nfr_address: ResourceAddress,
        number_of_nfr: u64,
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
                .mint_initial_supply(1);

            let mut active_bin = price.log() / (dec!(1) + bin_step).log() + dec!(2).powi(23);

            active_bin = Decimal::floor(&active_bin);

            info!("[instantiate_pool]: Active bin: {}", active_bin);
            info!("[instantiate_pool]: Active bin round: {}", active_bin.0);
            debug!("[instantiate_pool]: Active bin round: {}", active_bin.0);

            let nfr_address = ResourceBuilder::new_integer_non_fungible()
                .metadata("name", "LP NFT")
                .metadata("description", "This is an NFT provided to each liquidity provider")
                .mintable(AccessRule::AllowAll, LOCKED)
                .updateable_non_fungible_data(AccessRule::AllowAll, LOCKED)
                .create_with_no_initial_supply();

            // Instantiate our Ociswap component
            let ociswap = (Self {
                lp_badge: Vault::with_bucket(lp_badge),
                base_fee: Decimal::from("0.003") * bin_step, // BASE_FACTOR * 30
                xrd_fee: Vault::new(RADIX_TOKEN),

                active_bin,
                bin_step,

                a_bins: KeyValueStore::new(),
                b_bins: KeyValueStore::new(),
                a_lp_id: KeyValueStore::new(),
                b_lp_id: KeyValueStore::new(),

                a_token_address,
                b_token_address,
                lp_nfr_address: nfr_address,
                number_of_nfr: 0,
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
            price_sup: Decimal,
            opt_lp_nfr: Option<Bucket>
        ) -> Vec<Bucket> {
            // No remainer

            assert!(
                !a_tokens.is_empty() & !b_tokens.is_empty(),
                "[Pool Creation]: Can't create a pool from an empty bucket."
            );
            // [Check] Maybe enable one of tokens amount to be 0

            // Sorting the buckets and then creating the KeyValueStore of the vaults from the sorted buckets
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
            let range = (price_sup.log() - price_inf.log()) / (dec!(1) + self.bin_step).log();
            // [TODO] Put a limit to range for gas.

            debug!("[add_liquidity]: Range: {}", range);
            //debug!("[add_liquidity]: Range floor round: {}", Decimal::floor(&range));

            let mut inf_id = self.get_id(price_inf);
            let mut sup_id = self.get_id(price_sup);

            inf_id = Decimal::floor(&inf_id);
            sup_id = Decimal::floor(&sup_id);
            debug!("[add_liquidity]: Round Inf id: {}", inf_id);
            debug!("[add_liquidity]: Round Sup id: {}", sup_id);

            debug!("[add_liquidity]: Active bin: {}", self.active_bin);

            // This case is for a normal Shape
            let b1_per_bin = buckets.0.amount() / (self.active_bin - inf_id + 1); // + 1 to include active bin
            let b2_per_bin = buckets.1.amount() / (sup_id - self.active_bin + 1);

            debug!(
                "[add_liquidity]: Bucket A amount: {}, Bucket B amount: {}",
                buckets.0.amount(),
                buckets.0.amount()
            );
            debug!(
                "[add_liquidity]: Amount per a bin: {}, Amount per b bin: {}",
                b1_per_bin,
                b2_per_bin
            );

            let mut all_buckets: Vec<Bucket> = Vec::new();

            // We are checking here if we have an NFR or do we have to create one.
            let my_lp_nfr = match opt_lp_nfr {
                None => {
                    let nft_data = Lp {
                        id_lp: HashMap::new(),
                    };

                    self.number_of_nfr = self.number_of_nfr + 1;

                    self.lp_badge.authorize(|| {
                        borrow_resource_manager!(self.lp_nfr_address).mint_non_fungible(
                            // The NFT id
                            &NonFungibleLocalId::Integer(self.number_of_nfr.into()),
                            // The NFT data
                            nft_data
                        )
                    })
                }
                Some(lp_nfr_bucket) => { lp_nfr_bucket }
            };

            let lp_nfr = my_lp_nfr.non_fungible::<Lp>();
            let nfr_id = lp_nfr.local_id();

            //let range = range.to_string().parse::<i64>().unwrap();
            //let range: i64 = range.round(0, RoundingMode::TowardsZero).to_string().parse().unwrap();
            debug!("[add_liquidity]: i64 Range: {}", range);

            let mut transition_id = inf_id;
            // While we are in range.
            while transition_id <= sup_id {
                // Bins are created when needed.
                // We are checking only A bins we both we create in both each time.
                match self.a_bins.get(&transition_id) {
                    // We don't have been create, will create for both A and B
                    None => {
                        if transition_id <= self.active_bin && buckets.0.amount() >= b1_per_bin {
                            let price_of_bin: Decimal = self.get_price(transition_id); // [Check] If it's better to calculate without ID.

                            let new_bin = Bin::new(
                                transition_id,
                                Vault::with_bucket(buckets.0.take(b1_per_bin))
                            );

                            info!(
                                "[add_liquidity]: New A bin id: {} (Active bin: {})",
                                new_bin.bin_id,
                                self.active_bin
                            );
                            info!("[add_liquidity]: Bucket A amount left: {}", buckets.0.amount());
                            self.a_bins.insert(transition_id, new_bin);

                            if !(transition_id == self.active_bin) {
                                let other_bin = Bin::new(
                                    transition_id,
                                    Vault::new(buckets.1.resource_address())
                                );
                                self.b_bins.insert(transition_id, other_bin);
                            }

                            self.update_position(
                                nfr_id.clone(),
                                transition_id,
                                price_of_bin * b1_per_bin
                            );
                        }
                        if transition_id >= self.active_bin && buckets.1.amount() >= b2_per_bin {
                            let new_bin = Bin::new(
                                transition_id,
                                Vault::with_bucket(buckets.1.take(b2_per_bin))
                            );
                            info!(
                                "[add_liquidity]: New B bin id: {} (Active bin: {})",
                                new_bin.bin_id,
                                self.active_bin
                            );
                            info!("[add_liquidity]: Bucket B amount left: {}", buckets.1.amount());
                            self.b_bins.insert(transition_id, new_bin);

                            if !(transition_id == self.active_bin) {
                                let other_bin = Bin::new(
                                    transition_id,
                                    Vault::new(buckets.0.resource_address())
                                );
                                self.a_bins.insert(transition_id, other_bin);
                            }

                            self.update_position(nfr_id.clone(), transition_id, b2_per_bin);
                        }
                    }
                    // We already have both bins
                    Some(_) => {
                        // Get Vault for that ID and add token.
                        if transition_id <= self.active_bin && buckets.0.amount() >= b1_per_bin {
                            let mut my_bin = self.a_bins.get_mut(&transition_id).unwrap();
                            my_bin.bin_vault.put(buckets.0.take(b1_per_bin));

                            info!(
                                "[add_liquidity]: {} Old A bin id: {} (Active bin: {})",
                                transition_id,
                                my_bin.bin_id,
                                self.active_bin
                            );
                            info!("[add_liquidity]: Bucket A amount left: {}", buckets.0.amount());

                            self.update_position(nfr_id.clone(), transition_id, b2_per_bin);
                        }
                        if transition_id >= self.active_bin && buckets.1.amount() >= b2_per_bin {
                            let mut my_bin = self.b_bins.get_mut(&transition_id).unwrap();
                            my_bin.bin_vault.put(buckets.1.take(b2_per_bin));

                            info!(
                                "[add_liquidity]: {} Old B bin id: {} (Active bin: {})",
                                transition_id,
                                my_bin.bin_id,
                                self.active_bin
                            );
                            info!("[add_liquidity]: Bucket B amount left: {}", buckets.1.amount());

                            //lp_tokens.push(lp_b_tokens);
                            self.update_position(nfr_id.clone(), transition_id, b2_per_bin);
                        }
                    }
                }

                transition_id = transition_id + 1; // [TODO] Change name
            }

            //info!("[add_liquidity]: LP Tokens: {:?}", lp_tokens);
            info!("[add_liquidity]: Amount bucket a end: {}", buckets.0.amount());
            info!("[add_liquidity]: Amount bucket b end: {}", buckets.1.amount());

            all_buckets.push(buckets.0);
            all_buckets.push(buckets.1);
            all_buckets.push(my_lp_nfr);

            info!("[add_liquidity]: All Buckets returned: {:?}", all_buckets);

            // Return the LP tokens, each Bucket of Vec<Bucket> will be added to the account
            all_buckets
        }

        pub fn add_specific_liquidity(
            &mut self,
            a_tokens: Bucket, //mut a_tokens: Bucket,
            b_tokens: Bucket, //mut b_tokens: Bucket,
            distribution: Vec<(Decimal, Decimal)>,
            opt_lp_nfr: Option<Bucket>
        ) -> Vec<Bucket> {
            let mut all_buckets: Vec<Bucket> = Vec::new();

            let mut buckets: (Bucket, Bucket) = if
                a_tokens.resource_address().to_vec() > b_tokens.resource_address().to_vec()
            {
                (a_tokens, b_tokens)
            } else {
                (b_tokens, a_tokens)
            };

            // We are checking here if we have an NFR or do we have to create one.
            let my_lp_nfr = match opt_lp_nfr {
                None => {
                    let nft_data = Lp {
                        id_lp: HashMap::new(),
                    };

                    self.number_of_nfr = self.number_of_nfr + 1;

                    self.lp_badge.authorize(|| {
                        borrow_resource_manager!(self.lp_nfr_address).mint_non_fungible(
                            // The NFT id
                            &NonFungibleLocalId::Integer(self.number_of_nfr.into()),
                            // The NFT data
                            nft_data
                        )
                    })
                }
                Some(lp_nfr_bucket) => { lp_nfr_bucket }
            };

            let lp_nfr = my_lp_nfr.non_fungible::<Lp>();
            let nfr_id = lp_nfr.local_id();

            for i in &distribution {
                let (bin_id, amount) = i;
                // Bins are created when needed.
                // We are checking only A bins we both we create in both each time.
                match self.a_bins.get(bin_id) {
                    None => {
                        if *bin_id <= self.active_bin && buckets.0.amount() >= *amount {
                            // [TODO] How to calculate LP amount?

                            let new_bin = Bin::new(
                                *bin_id,
                                Vault::with_bucket(buckets.0.take(*bin_id))
                            );

                            info!(
                                "[add_liquidity]: New A bin id: {} (Active bin: {})",
                                new_bin.bin_id,
                                self.active_bin
                            );
                            info!("[add_liquidity]: Bucket A amount left: {}", buckets.0.amount());
                            self.a_bins.insert(*bin_id, new_bin);

                            if !(*bin_id == self.active_bin) {
                                let other_bin = Bin::new(
                                    *bin_id,
                                    Vault::new(buckets.1.resource_address())
                                );
                                self.b_bins.insert(*bin_id, other_bin);
                            }

                            self.update_position(nfr_id.clone(), *bin_id, *amount);
                        }
                        if *bin_id >= self.active_bin && buckets.1.amount() >= *amount {
                            let new_bin = Bin::new(
                                *bin_id,
                                Vault::with_bucket(buckets.1.take(*amount))
                            );
                            info!(
                                "[add_liquidity]: New B bin id: {} (Active bin: {})",
                                new_bin.bin_id,
                                self.active_bin
                            );
                            info!("[add_liquidity]: Bucket B amount left: {}", buckets.1.amount());
                            self.b_bins.insert(*bin_id, new_bin);

                            if !(*bin_id == self.active_bin) {
                                let other_bin = Bin::new(
                                    *bin_id,
                                    Vault::new(buckets.0.resource_address())
                                );
                                self.a_bins.insert(*bin_id, other_bin);
                            }

                            self.update_position(nfr_id.clone(), *bin_id, *amount);
                        }
                    }
                    Some(_) => {
                        if *bin_id <= self.active_bin && buckets.0.amount() >= *amount {
                            let mut my_bin = self.a_bins.get_mut(&*bin_id).unwrap();
                            my_bin.bin_vault.put(buckets.0.take(*amount));

                            info!(
                                "[add_liquidity]: {} Old A bin id: {} (Active bin: {})",
                                *bin_id,
                                my_bin.bin_id,
                                self.active_bin
                            );
                            info!("[add_liquidity]: Bucket A amount left: {}", buckets.0.amount());

                            self.update_position(nfr_id.clone(), *bin_id, *amount);
                        }
                        if *bin_id >= self.active_bin && buckets.1.amount() >= *amount {
                            let mut my_bin = self.b_bins.get_mut(&*bin_id).unwrap();
                            my_bin.bin_vault.put(buckets.1.take(*amount));

                            info!(
                                "[add_liquidity]: {} Old B bin id: {} (Active bin: {})",
                                *bin_id,
                                my_bin.bin_id,
                                self.active_bin
                            );
                            info!("[add_liquidity]: Bucket B amount left: {}", buckets.1.amount());

                            //lp_tokens.push(lp_b_tokens);
                            self.update_position(nfr_id.clone(), *bin_id, *amount);
                        }
                    }
                }
            }

            all_buckets.push(buckets.0);
            all_buckets.push(buckets.1);
            all_buckets.push(my_lp_nfr);

            all_buckets
        }

        /// Removes liquidity from this pool.
        /// [TODO] Return unclaimed fees.
        pub fn remove_liquidity(&mut self, lp_tokens: Bucket) -> Bucket {
            // assert!(
            //     self.lp_resource_def == lp_tokens.resource_address(),
            //     "Wrong token type passed in"
            // );

            let lp_tokens_address = lp_tokens.resource_address();
            let lp_resource_manager = borrow_resource_manager!(lp_tokens_address);

            // L * reserves / totalL
            if self.a_lp_id.get(&lp_tokens_address).is_some() {
                let bin_id = self.a_lp_id.get(&lp_tokens_address).unwrap(); // [Check]
                let bin_price = self.get_price(*bin_id);
                let mut my_a_bin = self.a_bins.get_mut(&bin_id).unwrap();
                let a_amount =
                    (lp_tokens.amount() * my_a_bin.bin_vault.amount()) /
                    lp_resource_manager.total_supply() /
                    bin_price;
                // Burning LP tokens received
                self.lp_badge.authorize(|| {
                    lp_tokens.burn();
                });
                my_a_bin.bin_vault.take(a_amount)
            } else {
                let bin_id = self.a_lp_id.get(&lp_tokens_address).unwrap();
                let mut my_b_bin = self.b_bins.get_mut(&bin_id).unwrap();
                let b_amount =
                    (lp_tokens.amount() * my_b_bin.bin_vault.amount()) /
                    lp_resource_manager.total_supply();
                // Burning LP tokens received
                self.lp_badge.authorize(|| {
                    lp_tokens.burn();
                });
                my_b_bin.bin_vault.take(b_amount)
            } // TODO
        }

        /// Swaps token A for B, or vice versa.
        /// [TODO] Add slippage and belief price
        pub fn swap(&mut self, mut input_tokens: Bucket) -> Vec<Bucket> {
            // Calculate the swap fee.
            //let fee_amount = input_tokens.amount() * self.base_fee;

            let mut all_output_tokens: Vec<Bucket> = Vec::new();

            // Get the price of active bin.
            let mut price_of_active_bin: Decimal = self.get_price(self.active_bin);
            debug!("[swap]: Active bin: {}", self.active_bin);
            debug!("[swap]: Price of active bin: {}", price_of_active_bin);

            let output_tokens = if input_tokens.resource_address() == self.a_token_address {
                // Calculate how much of token B we will return.
                let mut b_amount = price_of_active_bin * input_tokens.amount();
                debug!("[swap]: B amount that will be returned: {}", b_amount);
                // Get B bin to get B active bin and take output B tokens
                // [Check] Do we have the correct bin when mut.
                let mut my_b_bin = self.b_bins.get_mut(&self.active_bin).unwrap();

                debug!(
                    "[swap]: A amount in active bin: {}, ID: {}",
                    self.a_bins.get_mut(&self.active_bin).unwrap().bin_vault.amount(),
                    self.a_bins.get_mut(&self.active_bin).unwrap().bin_id
                );

                debug!(
                    "[swap]: B amount in active bin: {}, ID: {}",
                    my_b_bin.bin_vault.amount(),
                    my_b_bin.bin_id
                );

                while my_b_bin.bin_vault.amount() > Decimal::zero() {
                    // Check amount of B available.
                    if b_amount <= my_b_bin.bin_vault.amount() {
                        // Enough B in active bin.
                        //let mut my_a_bin = self.a_bins.get_mut(&self.active_bin).unwrap();

                        info!(
                            "[swap]: {} A amount Swapped to {} B and ended.",
                            input_tokens.amount(),
                            b_amount
                        );

                        // Put the input A into respective A bin
                        //my_a_bin.bin_vault.put(input_tokens);
                        break;
                    } else {
                        // More A than B.

                        // [Check] Calculate again with new price.
                        b_amount =
                            price_of_active_bin *
                            (input_tokens.amount() -
                                my_b_bin.bin_vault.amount() / price_of_active_bin);

                        info!(
                            "[swap]: {} A amount Swapped to {} B and going to next bin.",
                            my_b_bin.bin_vault.amount() / price_of_active_bin,
                            my_b_bin.bin_vault.amount()
                        );

                        //self.swap(my_b_bin.bin_vault.take(my_b_bin.bin_vault.amount()));
                        // Taking amount of B in the bin.
                        let my_b_bin_amount = my_b_bin.bin_vault.amount();
                        let bin_bucket = my_b_bin.bin_vault.take(my_b_bin_amount);

                        all_output_tokens.push(bin_bucket);

                        let mut my_a_bin = self.a_bins.get_mut(&self.active_bin).unwrap();
                        info!(
                            "[swap]: {} A Vault amount before take part.",
                            my_a_bin.bin_vault.amount()
                        );

                        info!(
                            "[swap]: Will take {} A. With b_bin amount = {} and price active bin = {}",
                            my_b_bin_amount / price_of_active_bin,
                            my_b_bin_amount,
                            price_of_active_bin
                        );

                        let transition_bucket = input_tokens.take(
                            my_b_bin_amount / price_of_active_bin
                        );
                        my_a_bin.bin_vault.put(transition_bucket);
                        info!(
                            "[swap]: {} A Vault amount after take part.",
                            my_a_bin.bin_vault.amount()
                        );
                        info!(
                            "[swap]: {} Input tokens left after taking a part.",
                            input_tokens.amount()
                        );

                        self.active_bin = self.active_bin + 1; // [Check] Decimal + i32.

                        price_of_active_bin = self.get_price(self.active_bin);
                        // [TODO] Calculate A amount to take.

                        my_b_bin = self.b_bins.get_mut(&self.active_bin).unwrap();
                    }
                }
                // [TODO][Check] Get amount of A and take fees from B.
                // Give B amount to user.
                let mut my_a_bin = self.a_bins.get_mut(&self.active_bin).unwrap();
                info!("[swap]: {} A token left.", input_tokens.amount());
                my_a_bin.bin_vault.put(input_tokens);
                my_b_bin.bin_vault.take(b_amount)
            } else {
                // B to A  <- self.b_token_address B to A
                // Calculate how much of token B we will return.
                let mut a_amount = input_tokens.amount() / price_of_active_bin;
                // Get B bin to get B active bin and take output B tokens
                // [Check] Do we have the correct bin when mut.
                let mut my_a_bin = self.a_bins.get_mut(&self.active_bin).unwrap();

                while my_a_bin.bin_vault.amount() > Decimal::zero() {
                    // Check amount of B available.
                    if a_amount <= my_a_bin.bin_vault.amount() {
                        // Enough B in active bin.
                        let mut my_b_bin = self.b_bins.get_mut(&self.active_bin).unwrap();
                        my_b_bin.bin_vault.put(input_tokens);
                        break;
                    } else {
                        // More A than B.

                        // [Check] Calculate again with new price.
                        a_amount =
                            price_of_active_bin *
                            (input_tokens.amount() -
                                my_a_bin.bin_vault.amount() / price_of_active_bin);

                        //self.swap(my_b_bin.bin_vault.take(my_b_bin.bin_vault.amount()));
                        let my_a_bin_amount = my_a_bin.bin_vault.amount();
                        my_a_bin.bin_vault.take(my_a_bin_amount);

                        self.active_bin = self.active_bin - 1; // [Check] Decimal + i32.

                        price_of_active_bin = self.get_price(self.active_bin);
                        // [TODO] Calculate A amount to take.

                        my_a_bin = self.a_bins.get_mut(&self.active_bin).unwrap();
                    }
                }
                // [TODO][Check] Get amount of A and take fees from B.
                my_a_bin.bin_vault.take(a_amount)
            };

            info!(
                "[swap]: A amount in active bin after Swap: {}, ID: {}",
                self.a_bins.get_mut(&self.active_bin).unwrap().bin_vault.amount(),
                self.b_bins.get_mut(&self.active_bin).unwrap().bin_id
            );
            info!(
                "[swap]: B amount in active bin after Swap: {}, ID: {}",
                self.b_bins.get_mut(&self.active_bin).unwrap().bin_vault.amount(),
                self.b_bins.get_mut(&self.active_bin).unwrap().bin_id
            );

            all_output_tokens.push(output_tokens);
            all_output_tokens
            //self.xrd_fee.take(fee_amount)
        }

        // Creates an LP token for a bin
        // [TODO] Add symbol and name
        // [Check] If we can use badges
        // fn create_lp_token(&mut self) -> ResourceAddress {
        //     let lp_resource_address = ResourceBuilder::new_fungible()
        //         .divisibility(DIVISIBILITY_MAXIMUM)
        //         // .metadata("symbol", pair_name)
        //         // .metadata("name", lp_id)
        //         .mintable(rule!(require(self.lp_badge.resource_address())), LOCKED)
        //         .burnable(rule!(require(self.lp_badge.resource_address())), LOCKED)
        //         .create_with_no_initial_supply();

        //     lp_resource_address
        // }

        // Returns the ID of a price
        fn get_price(&mut self, id: Decimal) -> Decimal {
            // // Calculate price (constant sum)
            // // p = (1+ binStep)^(activeBin - 2^23)
            // let price: Decimal = (dec!(1) + self.bin_step).powi(
            //     (id - dec!(2).powi(23)).to_string().parse::<i64>().unwrap()
            // );

            // price;

            // Calculate price (constant sum)
            // p = (1+ binStep)^(activeBin - 2^23)
            let price: Decimal = (dec!(1) + self.bin_step).pow(id - dec!(2).powi(23));

            price

            //[Remove]
            //println!("ID: {}", id);
            //dec!(20)
        }

        // Returns the ID for a certain price
        fn get_id(&mut self, price: Decimal) -> Decimal {
            let id = price.log() / (dec!(1) + self.bin_step).log() + dec!(2).powi(23);

            id
        }
        // // This function is to add tokens in a specific bin
        // pub fn add_specific_liquidity(&mut self, mut tokens: Bucket, id: Decimal) -> Bucket {
        //     tokens.take(id)
        // }

        pub fn update_position(&self, id: NonFungibleLocalId, bin_id: Decimal, lp_amount: Decimal) {
            let resource_manager = borrow_resource_manager!(self.lp_nfr_address);
            let mut nft_data: Lp = resource_manager.get_non_fungible_data(&id); // [Check] mut

            // Update the `amount` field
            if nft_data.id_lp.contains_key(&bin_id) {
                *nft_data.id_lp.get_mut(&bin_id).unwrap() += lp_amount;
                info!(
                    "[update_position]: Updated existing value. NonFungibleLocalId {}, bin ID {}",
                    id,
                    bin_id
                );
            } else {
                nft_data.id_lp.insert(bin_id, lp_amount);
                info!(
                    "[update_position]: Created new value. NonFungibleLocalId {}, bin ID {}",
                    id,
                    bin_id
                );
            }

            // Update the data on the network
            resource_manager.update_non_fungible_data(&id, nft_data);
        }
    }
}

// Trader Joe Docs : https://docs.traderjoexyz.com/concepts/bin-liquidty
// Trader Joe Whitepaper : https://github.com/traderjoe-xyz/LB-Whitepaper/blob/main/Joe%20v2%20Liquidity%20Book%20Whitepaper.pdf

// Tolerance for inaccuracies when calculating exp
const EXP_TOLERANCE: i64 = 1; //Decimal::from_str().unwrap();
const LN2: i64 = 693147180559945309;
const LN2HI: i64 = 693147180369123816; //6.93147180369123816490e-01; /* 0x3fe62e42, 0xfee00000 */
const LN2LO: i64 = 190821492; // 1.90821492927058770002e-10; /* 0x3dea39ef, 0x35793c76 */
const HALF: i64 = 500000000000000000;
const INVLN2: i64 = 1442695040888963387;
const SQRT: i64 = 1414213562373095048;
const DECIMAL_PLACES: i64 = 1000000000000000000;

const P1: i64 = 166666666666666019; // 1.66666666666666019037e-01; /* 0x3FC55555, 0x5555553E */
const P2: i64 = -2777777777701559; //  -2.77777777770155933842e-03; /* 0xBF66C16C, 0x16BEBD93 */
const P3: i64 = 66137563214379; // 6.61375632143793436117e-05; /* 0x3F11566A, 0xAF25DE2C */
const P4: i64 = -1653390220546; // -1.65339022054652515390e-06; /* 0xBEBBBD41, 0xC5D26BF1 */
const P5: i64 = 41381367970; //4.13813679705723846039e-08; /* 0x3E663769, 0x72BEA4D0 */

const LG1: i64 = 666666666666673513; // 6.666666666666735130e-01; /* 3FE55555 55555593 */
const LG2: i64 = 399999999994094190; // 3.999999999940941908e-01; /* 3FD99999 9997FA04 */
const LG3: i64 = 285714287436623914; // 2.857142874366239149e-01; /* 3FD24924 94229359 */
const LG4: i64 = 222221984321497839; // 2.222219843214978396e-01; /* 3FCC71C5 1D8E78AF */
const LG5: i64 = 181835721616180501; // 1.818357216161805012e-01; /* 3FC74664 96CB03DE */
const LG6: i64 = 153138376992093733; // 1.531383769920937332e-01; /* 3FC39A09 D078C69F */
const LG7: i64 = 147981986051165859; // 1.479819860511658591e-01; /* 3FC2F112 DF3E5244 */

// Table representing {index}!
const FACTORIAL: [u128; 35] = [
    1, 1, 2, 6, 24, 120, 720, 5040, 40320, 362880, 3628800, 39916800, 479001600, 6227020800,
    87178291200, 1307674368000, 20922789888000, 355687428096000, 6402373705728000, 121645100408832000,
    2432902008176640000, 51090942171709440000, 1124000727777607680000, 25852016738884976640000,
    620448401733239439360000, 15511210043330985984000000, 403291461126605635584000000, 10888869450418352160768000000,
    304888344611713860501504000000, 8841761993739701954543616000000,
    265252859812191058636308480000000, 8222838654177922817725562880000000,
    263130836933693530167218012160000000, 8683317618811886495518194401280000000,
    295232799039604140847618609643520000000,
];

pub trait MathematicalOps {
    /// The estimated exponential function, e<sup>x</sup>. Stops calculating when it is within
    /// tolerance of roughly `0.0000002`.
    fn exp_factorial(&self) -> Decimal;

    /// The estimated exponential function, e<sup>x</sup> using the `tolerance` provided as a hint
    /// as to when to stop calculating. A larger tolerance will cause the number to stop calculating
    /// sooner at the potential cost of a slightly less accurate result.
    fn exp_with_tolerance(&self, tolerance: Decimal) -> Decimal;
    fn exp(&self) -> Decimal;
    fn log(&self) -> Decimal;
    fn pow(&self, exponent: Decimal) -> Decimal;
}

impl MathematicalOps for Decimal {
    fn exp_factorial(&self) -> Decimal {
        self.exp_with_tolerance(Decimal(BnumI256::from(EXP_TOLERANCE)))
    }

    fn exp(&self) -> Decimal {
        // based on https://github.com/rust-lang/libm/blob/master/src/math/exp.rs
        if self.is_zero() {
            return Decimal::ONE;
        }

        let sign = if self.is_negative() { dec!(-1) } else { dec!(1) };
        // r = x - floor(x/ln(2) +- 0.5) * ln(2)
        // https://www.wolframalpha.com/input?i=x+-+floor%28x%2Fln%282%29+%2B+0.5%29+*+ln%282%29
        let k_ = Decimal(BnumI256::from(INVLN2)) * *self + sign * Decimal(BnumI256::from(HALF));
        let k = (k_.0 / BnumI256::from(DECIMAL_PLACES)).to_i64().unwrap();

        let hi = *self - Decimal::from(k) * Decimal(BnumI256::from(LN2HI));
        let lo = Decimal::from(k) * Decimal(BnumI256::from(LN2LO));
        let r = hi - lo;

        if k > 195 {
            panic!("Overflow");
        }

        let p1 = Decimal(BnumI256::from(P1));
        let p2 = Decimal(BnumI256::from(P2));
        let p3 = Decimal(BnumI256::from(P3));
        let p4 = Decimal(BnumI256::from(P4));
        let p5 = Decimal(BnumI256::from(P5));

        let rr = r * r;
        let c = r - rr * (p1 + rr * (p2 + rr * (p3 + rr * (p4 + rr * p5))));
        let result = Decimal::ONE + ((r * c) / (dec!(2) - c) - lo + hi);

        // buggy alternative - check with team
        // Decimal(Decimal::ONE.0 << BnumI256::from(k)) * result
        Decimal::from(2).powi(k) * result // works until e^85
    }

    fn log(&self) -> Decimal {
        // based on https://github.com/rust-lang/libm/blob/master/src/math/log.rs
        let mut k = 255 - (self.0 / BnumI256::from(DECIMAL_PLACES)).leading_zeros(); // index highest integer bit
        let mut r = *self / Decimal::from(2).powi(k.to_i64().unwrap());
        // buggy alternative - check with team
        // let mut r = *self / Decimal(Decimal::ONE.0 << BnumI256::from(k));
        if r > Decimal(BnumI256::from(SQRT)) {
            k = k + 1;
            r = r / dec!(2);
        }
        // info!("k {} r {}", k, r);

        let f = r - dec!(1);
        let hfsq = Decimal(BnumI256::from(HALF)) * f * f;
        let s = f / (dec!(2) + f);
        let z = s * s;
        let w = z * z;
        let t1 =
            w *
            (Decimal(BnumI256::from(LG2)) +
                w * (Decimal(BnumI256::from(LG4)) + w * Decimal(BnumI256::from(LG6))));
        let t2 =
            z *
            (Decimal(BnumI256::from(LG1)) +
                w *
                    (Decimal(BnumI256::from(LG3)) +
                        w * (Decimal(BnumI256::from(LG5)) + w * Decimal(BnumI256::from(LG7)))));
        let res = t2 + t1;
        let dk = Decimal::from(k);
        s * (hfsq + res) +
            dk * Decimal(BnumI256::from(LN2LO)) -
            hfsq +
            f +
            dk * Decimal(BnumI256::from(LN2HI))
    }

    fn pow(&self, exponent: Decimal) -> Decimal {
        (exponent * self.log()).exp()
    }

    fn exp_with_tolerance(&self, tolerance: Decimal) -> Decimal {
        // based on https://docs.rs/rust_decimal/latest/src/rust_decimal/maths.rs.html
        // with argument reduction and scaling up from https://github.com/rust-lang/libm/blob/master/src/math/exp.rs
        if self.is_zero() {
            return Decimal::ONE;
        }
        if self.is_negative() {
            return Decimal::ONE.div(self.abs().exp_with_tolerance(tolerance));
        }

        let x = *self;
        let ln2 = BnumI256::from(LN2);
        let k = (x.0 / ln2).to_i64().unwrap();
        let r = Decimal(x.0 % ln2);

        if k > 195 {
            panic!("Overflow");
        }

        let mut term = r;
        let mut result = r.add(Decimal::ONE);

        for factorial in FACTORIAL.iter().skip(2) {
            term = r * term;
            let next = result + term / Decimal::from(*factorial);
            let diff = (next - result).abs();
            result = next;
            if diff <= tolerance {
                break;
            }
        }
        // buggy alternative - check with team
        // Decimal(Decimal::ONE.0 << BnumI256::from(k))
        Decimal::from(2).powi(k) * result // works until e^85
    }
}