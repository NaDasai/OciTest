use scrypto::prelude::*;
use std::ops::Add;
use std::ops::Div;

// Bins are here to deposit and withdraw liquidity.
#[derive(ScryptoCategorize, ScryptoDecode, ScryptoEncode, LegacyDescribe)]
pub struct Bin {
    // ID of the Bin
    bin_id: Decimal,
    // With A or B not both.
    bin_vault: Vault,
}

// To create a new Bin
impl Bin {
    pub fn new(bin_id: Decimal, bin_vault: Vault) -> Self {
        Self {
            bin_id,
            bin_vault,
        }
    }
}

// Metadata for the LP NFR
#[derive(NonFungibleData)]
pub struct Lp {
    #[mutable]
    id_lp: HashMap<Decimal, Decimal>,
    // [TODO] add A and B Token addresses to check in remove
}

#[blueprint]
mod ociswap_module {
    struct Ociswap {
        // LP tokens mint badge.
        lp_badge: Vault,
        // The fee to apply for every swap (BASE_FACTOR * bin_step)
        base_fee: Decimal,
        // [TODO] XRD vault. (variable fees)
        xrd_fee: Vault,

        // Bin ID of current price.
        active_bin: Decimal,
        // [TODO] 0.003 for now
        bin_step: Decimal,

        // List of A and B Tokens
        a_bins: KeyValueStore<Decimal, Bin>,
        b_bins: KeyValueStore<Decimal, Bin>,
        // Total LP by Bin
        id_total: HashMap<Decimal, Decimal>, // [Remove]

        // [Check] Do we add both addresses for checks when adding liquidity.
        a_token_address: ResourceAddress,
        b_token_address: ResourceAddress,

        // Address of NFR
        lp_nfr_address: ResourceAddress,
        // Number of NFR
        number_of_nfr: u64,
    }

    impl Ociswap {
        /// Creates a Ociswap component for token pair A/B with initial price and returns the component address
        pub fn instantiate_pool(
            a_token_address: ResourceAddress,
            b_token_address: ResourceAddress, // Not a Bucket
            price: Decimal,
            bin_step: Decimal
        ) -> ComponentAddress {
            // Performing the checks to see if this liquidity pool may be created or not.
            assert!(
                (bin_step >= Decimal::zero()) & (bin_step <= dec!("1")),
                "[Pool Creation]: Fee must be between 0 and 1"
            );

            assert_ne!(
                a_token_address,
                b_token_address,
                "[Pool Creation]: Liquidity pools may only be created between two different tokens."
            );

            // Instantiate our LP NFR
            // Badge
            let lp_badge = ResourceBuilder::new_fungible()
                .divisibility(DIVISIBILITY_NONE)
                .metadata("name", "LP Token Mint Auth")
                .mint_initial_supply(1);

            // NFR
            let nfr_address = ResourceBuilder::new_integer_non_fungible()
                .metadata("name", "LP NFT")
                .metadata("description", "This is an NFT provided to each liquidity provider")
                .mintable(AccessRule::AllowAll, LOCKED)
                .updateable_non_fungible_data(AccessRule::AllowAll, LOCKED)
                .create_with_no_initial_supply();

            // Calculate active Bin
            let mut active_bin = price.log() / (dec!(1) + bin_step).log() + dec!(2).powi(23);
            active_bin = Decimal::floor(&active_bin);

            info!("[instantiate_pool]: Active bin: {}", active_bin);

            // Instantiate our Ociswap component
            let ociswap = (Self {
                lp_badge: Vault::with_bucket(lp_badge),
                base_fee: Decimal::from("0.003") * bin_step, // BASE_FACTOR * 30
                xrd_fee: Vault::new(RADIX_TOKEN),

                active_bin,
                bin_step,

                a_bins: KeyValueStore::new(),
                b_bins: KeyValueStore::new(),
                id_total: HashMap::new(),

                a_token_address,
                b_token_address,
                lp_nfr_address: nfr_address,
                number_of_nfr: 0,
            }).instantiate();

            // Returns the new Ociswap component
            ociswap.globalize()
        }

        /// Adds liquidity to this pool and return the LP NFR representing pool shares
        // a_tokens         Tokens A that will be added
        // a_distribution   Token A distribution in the A Bins Vec<(A Bin ID, Amount of B to add)>
        // b_tokens         Tokens B that will be added
        // b_distribution   Token B distribution in the B Bins Vec<(B Bin ID, Amount of B to add)
        // opt_lp_nfr       LP NFR to track the LP of the user
        pub fn add_liquidity(
            &mut self,
            mut a_tokens: Bucket,
            a_distribution: Vec<(Decimal, Decimal)>,
            mut b_tokens: Bucket,
            b_distribution: Vec<(Decimal, Decimal)>,
            opt_lp_nfr: Option<Bucket>
        ) -> Vec<Bucket> {
            let mut all_buckets: Vec<Bucket> = Vec::new();

            assert!(a_tokens.resource_address() == self.a_token_address, "Not the right A Token");
            assert!(b_tokens.resource_address() == self.b_token_address, "Not the right B Token");

            // We are checking here if we have an NFR or do we have to create one for the user.
            let my_lp_nfr = match opt_lp_nfr {
                // No NFR, create new one to user.
                None => {
                    let nft_data = Lp {
                        id_lp: HashMap::new(),
                    };

                    // To track the number of NFR
                    self.number_of_nfr = self.number_of_nfr + 1;

                    // Minting the NFR
                    self.lp_badge.authorize(|| {
                        borrow_resource_manager!(self.lp_nfr_address).mint_non_fungible(
                            // The NFT id
                            &NonFungibleLocalId::Integer(self.number_of_nfr.into()),
                            // The NFT data
                            nft_data
                        )
                    })
                }
                // We just get the NFR provided by the user.
                Some(lp_nfr_bucket) => { lp_nfr_bucket }
            };

            // We get the NFR from the Bucket
            let lp_nfr = my_lp_nfr.non_fungible::<Lp>();
            // And its ID
            let nfr_id = lp_nfr.local_id();

            let resource_manager = borrow_resource_manager!(self.lp_nfr_address);
            // We get the data of the NFR
            let mut nft_data: Lp = resource_manager.get_non_fungible_data(&nfr_id);

            debug!("[add_specific_liquidity]: Active bin: {}", self.active_bin);

            // We add Token A from A Distribution
            for i_a in &a_distribution {
                // We get the Bin ID and the Amount for that Bin.
                // (Bin ID, A amount in Bin)
                let (bin_id, amount) = i_a;

                // Price is calculated to define the amount of LP
                let price_of_bin: Decimal = self.get_price(*bin_id);

                // Bins are created when needed.
                // Checking in b_bins is the same since we create both at the same time
                match self.a_bins.get(bin_id) {
                    // No Bin, create one.
                    None => {
                        if *bin_id <= self.active_bin && a_tokens.amount() >= *amount {
                            // New A Bin
                            let new_bin = Bin::new(
                                *bin_id,
                                Vault::with_bucket(a_tokens.take(*amount))
                            );

                            info!("[add_specific_liquidity]: New A bin id: {}", new_bin.bin_id);
                            info!(
                                "[add_specific_liquidity]: Bucket A amount left: {}",
                                a_tokens.amount()
                            );
                            // Insert Bin in list of A Bins
                            self.a_bins.insert(*bin_id, new_bin);

                            // Add empty B Bin
                            let other_bin = Bin::new(*bin_id, Vault::new(self.b_token_address));
                            // Insert Bin in list of B Bins
                            self.b_bins.insert(*bin_id, other_bin);

                            // Update NFR metadata
                            // Check if we already added to that Bin
                            if nft_data.id_lp.contains_key(&bin_id) {
                                *nft_data.id_lp.get_mut(&bin_id).unwrap() += *amount * price_of_bin;
                                info!(
                                    "[add_liquidity]: Before insert new amount liquidity amount: {}",
                                    self.id_total[bin_id]
                                );
                                self.id_total.insert(
                                    *bin_id,
                                    self.id_total[bin_id] + *amount * price_of_bin
                                );
                                info!(
                                    "[add_liquidity]: After insert new amount liquidity amount: {}",
                                    self.id_total[bin_id]
                                );
                                //*self.id_total.get_mut(bin_id).unwrap() += *amount * price_of_bin;
                            } else {
                                // We are adding in new Bin
                                nft_data.id_lp.insert(*bin_id, *amount * price_of_bin);
                                // Adding in Total liquidity
                                // Checking if any user added to that Bin before
                                if self.id_total.contains_key(&bin_id) {
                                    self.id_total.insert(
                                        *bin_id,
                                        self.id_total[bin_id] + *amount * price_of_bin
                                    );
                                } else {
                                    self.id_total.insert(*bin_id, *amount * price_of_bin);
                                }
                            }
                        } else {
                            // We are in here if we added in a Bin after active bin
                            // or if the amount provided is less than the one we are adding in the distribution
                            info!("[add_specific_liquidity]: Didn't add: {},{} A", bin_id, amount);
                        }
                    }
                    Some(_) => {
                        // Bin found.
                        if *bin_id <= self.active_bin && a_tokens.amount() >= *amount {
                            // Getting the existing A Bin
                            let mut my_bin = self.a_bins.get_mut(&*bin_id).unwrap();
                            info!(
                                "[add_specific_liquidity]: My bin A amount BEFORE add: {}",
                                my_bin.bin_vault.amount()
                            );
                            // Adding the amount in the A Bin
                            my_bin.bin_vault.put(a_tokens.take(*amount));
                            info!(
                                "[add_specific_liquidity]: My bin A amount AFTER add: {}",
                                my_bin.bin_vault.amount()
                            );

                            info!(
                                "[add_specific_liquidity]: {} Old A bin id: {}",
                                *bin_id,
                                my_bin.bin_id
                            );
                            info!(
                                "[add_specific_liquidity]: Bucket A amount left: {}",
                                a_tokens.amount()
                            );

                            // Update NFR metadata
                            // Check if we already added to that Bin
                            if nft_data.id_lp.contains_key(&bin_id) {
                                *nft_data.id_lp.get_mut(&bin_id).unwrap() += *amount * price_of_bin;
                                info!(
                                    "[add_liquidity]: Before insert new amount liquidity amount: {}",
                                    self.id_total[bin_id]
                                );
                                self.id_total.insert(
                                    *bin_id,
                                    self.id_total[bin_id] + *amount * price_of_bin
                                );
                                info!(
                                    "[add_liquidity]: After insert new amount liquidity amount: {}",
                                    self.id_total[bin_id]
                                );
                            } else {
                                // We are adding a new Bin in NFR
                                nft_data.id_lp.insert(*bin_id, *amount * price_of_bin);
                                if self.id_total.contains_key(&bin_id) {
                                    self.id_total.insert(
                                        *bin_id,
                                        self.id_total[bin_id] + *amount * price_of_bin
                                    );
                                } else {
                                    self.id_total.insert(*bin_id, *amount * price_of_bin);
                                }
                            }
                        } else {
                            info!("[add_specific_liquidity]: Didn't add: {},{} A", bin_id, amount);
                        }
                    }
                }
            }

            // (Bin ID, B amount in Bin)
            for i_b in &b_distribution {
                let (bin_id, amount) = i_b;

                match self.b_bins.get(bin_id) {
                    None => {
                        if *bin_id >= self.active_bin && b_tokens.amount() >= *amount {
                            // New B Bin
                            let new_bin = Bin::new(
                                *bin_id,
                                Vault::with_bucket(b_tokens.take(*amount))
                            );

                            info!("[add_specific_liquidity]: New B bin id: {}", new_bin.bin_id);
                            info!(
                                "[add_specific_liquidity]: Bucket B amount left: {}",
                                b_tokens.amount()
                            );
                            // Insert Bin in list of B Bins
                            self.b_bins.insert(*bin_id, new_bin);

                            // Add empty A Bin
                            let other_bin = Bin::new(*bin_id, Vault::new(self.a_token_address));
                            self.a_bins.insert(*bin_id, other_bin);

                            if nft_data.id_lp.contains_key(&bin_id) {
                                *nft_data.id_lp.get_mut(&bin_id).unwrap() += *amount;
                                self.id_total.insert(*bin_id, self.id_total[bin_id] + *amount);
                            } else {
                                nft_data.id_lp.insert(*bin_id, *amount);
                                if self.id_total.contains_key(&bin_id) {
                                    self.id_total.insert(*bin_id, self.id_total[bin_id] + *amount);
                                } else {
                                    self.id_total.insert(*bin_id, *amount);
                                }
                            }
                        } else {
                            info!("[add_specific_liquidity]: Didn't add: {},{} B", bin_id, amount);
                        }
                    }
                    Some(_) => {
                        if *bin_id >= self.active_bin && b_tokens.amount() >= *amount {
                            let mut my_bin = self.b_bins.get_mut(&*bin_id).unwrap();
                            my_bin.bin_vault.put(b_tokens.take(*amount));

                            info!(
                                "[add_specific_liquidity]: {} Old B bin id: {}",
                                *bin_id,
                                my_bin.bin_id
                            );
                            info!(
                                "[add_specific_liquidity]: Bucket B amount left: {}",
                                b_tokens.amount()
                            );

                            if nft_data.id_lp.contains_key(&bin_id) {
                                *nft_data.id_lp.get_mut(&bin_id).unwrap() += *amount;
                                self.id_total.insert(*bin_id, self.id_total[bin_id] + *amount);
                            } else {
                                // We are adding a new Bin in NFR
                                nft_data.id_lp.insert(*bin_id, *amount);
                                if self.id_total.contains_key(&bin_id) {
                                    self.id_total.insert(*bin_id, self.id_total[bin_id] + *amount);
                                } else {
                                    self.id_total.insert(*bin_id, *amount);
                                }
                            }
                        } else {
                            info!("[add_specific_liquidity]: Didn't add: {},{} B", bin_id, amount);
                        }
                    }
                }
            }

            // [TEST] To check LP
            for (lp_bin_id, lp_amount) in &self.id_total {
                info!("LP: {},{}", lp_bin_id, lp_amount);
            }

            // Updating the NFR metadata
            resource_manager.update_non_fungible_data(&nfr_id, nft_data);

            // Returning buckets
            all_buckets.push(a_tokens);
            all_buckets.push(b_tokens);
            all_buckets.push(my_lp_nfr);

            all_buckets
        }

        /// Removes liquidity from this pool.
        // my_lp_nfr            Bucket containing the NFR
        // opt_r_distribution   L distribution to withdraw from Bins Vec<(B Bin ID, Amount of L to withdraw)
        pub fn remove_liquidity(
            &mut self,
            my_lp_nfr: Bucket,
            opt_r_distribution: Option<Vec<(Decimal, Decimal)>>
        ) -> Vec<Bucket> {
            let mut all_buckets: Vec<Bucket> = Vec::new();

            debug!("[remove_liquidity]: Removing liquidity started.");

            // We get the NFR from the Bucket
            let lp_nfr = my_lp_nfr.non_fungible::<Lp>();
            // And its ID
            let nfr_id = lp_nfr.local_id();

            let resource_manager = borrow_resource_manager!(self.lp_nfr_address);
            // We get the data of the NFR
            let mut nft_data: Lp = resource_manager.get_non_fungible_data(&nfr_id);

            debug!("[remove_liquidity]: Active Bin: {}", self.active_bin);

            // [TEST] Check the LP metadata before remove
            for (lp_bin_id, lp_amount) in &nft_data.id_lp {
                info!("LP before remove: Bin {}, Amount {}", lp_bin_id, lp_amount);
            }

            match opt_r_distribution {
                // Here we are not specifing the Bins and the amounts so we remove eveything
                None => {
                    // We'll go through all the Bins of the NFR
                    for (lp_bin_id, lp_amount) in &nft_data.id_lp {
                        // Before and in active Bin (Token A)
                        if *lp_bin_id <= self.active_bin {
                            // Get A Bin
                            let mut my_a_bin = self.a_bins.get_mut(lp_bin_id).unwrap();
                            info!(
                                "[remove_liquidity]: Removing from A, bin ID {}, with amount {}\n
                                    Amount L = {}\n
                                    Reserve in A Bin = {}\n
                                    Total Liquidity = {}\n",
                                lp_bin_id,
                                (*lp_amount *
                                    self.a_bins.get_mut(lp_bin_id).unwrap().bin_vault.amount()) /
                                    self.id_total[lp_bin_id],
                                *lp_amount,
                                self.a_bins.get_mut(lp_bin_id).unwrap().bin_vault.amount(),
                                self.id_total[lp_bin_id]
                            );
                            // Get A tokens
                            // L * reserves / totalL
                            let a_bucket = my_a_bin.bin_vault.take(
                                (*lp_amount *
                                    self.a_bins.get_mut(lp_bin_id).unwrap().bin_vault.amount()) /
                                    self.id_total[lp_bin_id]
                            );
                            all_buckets.push(a_bucket);
                            // Remove liquidity from total
                            self.id_total.insert(
                                *lp_bin_id,
                                self.id_total[lp_bin_id] -
                                    (*lp_amount *
                                        self.a_bins
                                            .get_mut(lp_bin_id)
                                            .unwrap()
                                            .bin_vault.amount()) /
                                        self.id_total[lp_bin_id]
                            );
                        }
                        // After and in active Bin (Token B)
                        if *lp_bin_id >= self.active_bin {
                            let mut my_b_bin = self.b_bins.get_mut(lp_bin_id).unwrap();
                            info!(
                                "[remove_liquidity]: Removing from B, bin ID {}, with amount {}",
                                lp_bin_id,
                                (*lp_amount *
                                    self.b_bins.get_mut(lp_bin_id).unwrap().bin_vault.amount()) /
                                    self.id_total[lp_bin_id]
                            );
                            // L * reserves / totalL
                            let b_bucket = my_b_bin.bin_vault.take(
                                (*lp_amount *
                                    self.b_bins.get_mut(lp_bin_id).unwrap().bin_vault.amount()) /
                                    self.id_total[lp_bin_id]
                            );
                            all_buckets.push(b_bucket);
                            self.id_total.insert(
                                *lp_bin_id,
                                self.id_total[lp_bin_id] -
                                    (*lp_amount *
                                        self.b_bins
                                            .get_mut(lp_bin_id)
                                            .unwrap()
                                            .bin_vault.amount()) /
                                        self.id_total[lp_bin_id]
                            );
                        }
                    }
                    nft_data.id_lp.clear();
                }
                // User defined specific Bins and amounts
                Some(r_distribution) => {
                    for l_i in &r_distribution {
                        // (Bin ID, L amount in Bin)
                        let (bin_id, amount) = l_i;

                        info!(
                            "[remove_liquidity]: Before remove. NonFungibleLocalId {}, bin ID {}, with amount {}",
                            nfr_id,
                            bin_id,
                            *nft_data.id_lp.get_mut(&bin_id).unwrap()
                        );

                        // Check if Bin and amount are valid
                        if
                            nft_data.id_lp.contains_key(&bin_id) &&
                            nft_data.id_lp[bin_id] >= *amount
                        {
                            // Before and in active Bin (Token A)
                            if *bin_id <= self.active_bin {
                                let mut my_a_bin = self.a_bins.get_mut(bin_id).unwrap();
                                info!(
                                    "[remove_liquidity]: Amount in A Bin: {}",
                                    my_a_bin.bin_vault.amount()
                                );
                                info!(
                                    "[remove_liquidity]: Amount in A Bin in self: {}",
                                    self.a_bins.get_mut(bin_id).unwrap().bin_vault.amount()
                                );
                                info!(
                                    "[remove_liquidity]: Removing from A, bin ID {}, with amount {}\n
                                    Amount L = {}\n
                                    Reserve in A Bin = {}\n
                                    Total Liquidity = {}\n",
                                    bin_id,
                                    (*amount *
                                        self.a_bins.get_mut(bin_id).unwrap().bin_vault.amount()) /
                                        self.id_total[bin_id],
                                    *amount,
                                    self.a_bins.get_mut(bin_id).unwrap().bin_vault.amount(),
                                    self.id_total[bin_id]
                                );
                                // {A/B} = L * reserve{A/B} / totalL
                                let a_bucket = my_a_bin.bin_vault.take(
                                    (*amount *
                                        self.a_bins.get_mut(bin_id).unwrap().bin_vault.amount()) /
                                        self.id_total[bin_id]
                                );
                                // Check amount A token given back
                                info!(
                                    "[remove_liquidity]: Giving back {} A token",
                                    a_bucket.amount()
                                );
                                all_buckets.push(a_bucket);
                            }
                            // After and in active Bin (Token B)
                            if *bin_id >= self.active_bin {
                                let mut my_b_bin = self.b_bins.get_mut(bin_id).unwrap();
                                info!(
                                    "[remove_liquidity]: Amount in B Bin: {}",
                                    my_b_bin.bin_vault.amount()
                                );
                                info!(
                                    "[remove_liquidity]: Removing from B, bin ID {}, with amount {}",
                                    bin_id,
                                    (*amount *
                                        self.b_bins.get_mut(bin_id).unwrap().bin_vault.amount()) /
                                        self.id_total[bin_id]
                                );
                                // {A/B} = L * reserve{A/B} / totalL
                                let b_bucket = my_b_bin.bin_vault.take(
                                    (*amount *
                                        self.b_bins.get_mut(bin_id).unwrap().bin_vault.amount()) /
                                        self.id_total[bin_id]
                                );
                                // Check amount B token given back
                                info!(
                                    "[remove_liquidity]: Giving back {} B token",
                                    b_bucket.amount()
                                );
                                all_buckets.push(b_bucket);
                            }

                            // Remove amount from NFR
                            *nft_data.id_lp.get_mut(&bin_id).unwrap() -= *amount;
                            info!(
                                "[remove_liquidity]: After remove. NonFungibleLocalId {}, bin ID {}, with amount {}",
                                nfr_id,
                                bin_id,
                                *nft_data.id_lp.get_mut(&bin_id).unwrap()
                            );
                        } else {
                            info!("[remove_liquidity]: Amount too big!");
                        }
                    }
                }
            }

            // [TEST] Check the LP metadata after remove
            for (lp_bin_id, lp_amount) in &nft_data.id_lp {
                info!("LP after remove: Bin {}, Amount {}", lp_bin_id, lp_amount);
            }

            // Updating the NFR metadata
            resource_manager.update_non_fungible_data(&nfr_id, nft_data);

            // Returning buckets
            all_buckets.push(my_lp_nfr);
            all_buckets
        }

        /// Swaps Token A for B, or vice versa.
        /// [TODO] Add slippage and belief price
        pub fn swap(&mut self, mut input_tokens: Bucket) -> Vec<Bucket> {
            // Calculate the swap fee.
            // [Check] Accrued fees of master chef
            //let fee_amount = input_tokens.amount() * self.base_fee;

            let mut all_output_tokens: Vec<Bucket> = Vec::new();

            // Get the price of active bin.
            let mut price_of_active_bin: Decimal = self.get_price(self.active_bin);
            debug!("[swap]: Active bin: {}", self.active_bin);
            debug!("[swap]: Price of active bin: {}", price_of_active_bin);

            // We'll have in output_tokens either Token A or Token B, depending on input_tokens
            if input_tokens.resource_address() == self.a_token_address {
                // Input tokens A

                // Calculate how much of token B we will return.
                let mut b_amount = price_of_active_bin * input_tokens.amount();
                debug!("[swap]: A amount input (provided): {}", input_tokens.amount());
                debug!("[swap]: B amount output (that will be returned if we stay in active Bin): {}", b_amount);

                // Get B active bin
                let mut active_bin = self.b_bins.get_mut(&self.active_bin).unwrap();

                // We'll keep taking token B while we have some in active Bin
                while active_bin.bin_vault.amount() > Decimal::zero() {
                    if b_amount <= active_bin.bin_vault.amount() {
                        // If B amount less than B amount in active Bin go directly to last swap
                        break;
                    } else {
                        // Not enough B Tokens for the whole swap, get what's in the active Bin and to next one.

                        // Calculate new amount of B tokens with price of new active Bin
                        b_amount =
                            price_of_active_bin *
                            (input_tokens.amount() -
                                active_bin.bin_vault.amount() / price_of_active_bin);

                        // Taking amount of B in the Bin.
                        let active_bin_amount = active_bin.bin_vault.amount();
                        let bin_bucket = active_bin.bin_vault.take(active_bin_amount);
                        all_output_tokens.push(bin_bucket);

                        // Get current A Bin to deposit A Tokens
                        let mut my_a_bin = self.a_bins.get_mut(&self.active_bin).unwrap();
                        // Depositing swaped A Tokens to A Bin
                        // No taking all the A tokens since not enough B tokens
                        let transition_bucket = input_tokens.take(
                            active_bin_amount / price_of_active_bin
                        );
                        my_a_bin.bin_vault.put(transition_bucket);

                        info!(
                            "[swap]: Swapped A amount {} to B Amount {}. For price {} of Bin {}",
                            active_bin_amount / price_of_active_bin,
                            active_bin_amount,
                            price_of_active_bin,
                            self.active_bin
                        );

                        // Moving to next Bin
                        self.active_bin = self.active_bin + 1; // [Check] Decimal + i32.
                        // Calculating new price of active Bin
                        price_of_active_bin = self.get_price(self.active_bin);
                        // New active Bin
                        active_bin = self.b_bins.get_mut(&self.active_bin).unwrap();
                    }
                }

                // No more B Tokens, returning A Token left
                if active_bin.bin_vault.amount().is_zero() {
                    info!(
                        "[swap!]: Active Bin empty! Returning A amount {} left",
                        input_tokens.amount()
                    );
                    all_output_tokens.push(input_tokens);
                } else {
                    // Give B amount to user and get A Tokens.
                    let mut my_a_bin = self.a_bins.get_mut(&self.active_bin).unwrap();
                    info!(
                        "[swap]: Swaping last A tokens {}. We have in active Bin B amount {}",
                        input_tokens.amount(),
                        active_bin.bin_vault.amount()
                    );
                    my_a_bin.bin_vault.put(input_tokens);
                    let bin_bucket = active_bin.bin_vault.take(b_amount);
                    all_output_tokens.push(bin_bucket);
                }
            } else {
                // Input tokens B

                // Calculate how much of token B we will return.
                let mut a_amount = input_tokens.amount() / price_of_active_bin;
                debug!("[swap]: B amount input (provided): {}", input_tokens.amount());
                debug!("[swap]: A amount output (that will be returned if we stay in active Bin): {}", a_amount);

                // Get A active bin
                let mut active_bin = self.a_bins.get_mut(&self.active_bin).unwrap();

                // We'll keep taking token A while we have some in active Bin
                while active_bin.bin_vault.amount() > Decimal::zero() {
                    if a_amount <= active_bin.bin_vault.amount() {
                        // If A amount less than A amount in active Bin go directly to last swap
                        break;
                    } else {
                        // Not enough A Tokens for the whole swap, get what's in the active Bin and to next one.

                        // Calculate new amount of A tokens with price of new active Bin
                        a_amount =
                            price_of_active_bin *
                            (input_tokens.amount() -
                                active_bin.bin_vault.amount() * price_of_active_bin);

                        // Taking amount of A in the Bin.
                        let active_bin_amount = active_bin.bin_vault.amount();
                        let bin_bucket = active_bin.bin_vault.take(active_bin_amount);
                        all_output_tokens.push(bin_bucket);

                        // Get current B Bin to deposit B Tokens
                        let mut my_b_bin = self.b_bins.get_mut(&self.active_bin).unwrap();
                        // Depositing swaped B Tokens to B Bin
                        // No taking all the B tokens since not enough B tokens
                        let transition_bucket = input_tokens.take(
                            active_bin_amount * price_of_active_bin
                        );
                        my_b_bin.bin_vault.put(transition_bucket);

                        info!(
                            "[swap]: Swapped B amount {} to A Amount {}. For price {} of Bin {}",
                            active_bin_amount * price_of_active_bin,
                            active_bin_amount,
                            price_of_active_bin,
                            self.active_bin
                        );

                        // Moving to next Bin
                        self.active_bin = self.active_bin - 1; // [Check] Decimal + i32.
                        // Calculating new price of active Bin
                        price_of_active_bin = self.get_price(self.active_bin);
                        // New active Bin
                        active_bin = self.a_bins.get_mut(&self.active_bin).unwrap();
                    }
                }

                // No more A Tokens, returning B Token left
                if active_bin.bin_vault.amount().is_zero() {
                    info!(
                        "[swap!]: Active Bin empty! Returning A amount {} left",
                        input_tokens.amount()
                    );
                    all_output_tokens.push(input_tokens);
                } else {
                    // Give A amount to user and get B Tokens.
                    let mut my_b_bin = self.b_bins.get_mut(&self.active_bin).unwrap();
                    info!(
                        "[swap]: Swaping last A tokens {}. We have in active Bin B amount {}",
                        input_tokens.amount(),
                        active_bin.bin_vault.amount()
                    );
                    my_b_bin.bin_vault.put(input_tokens);
                    let bin_bucket = active_bin.bin_vault.take(a_amount);
                    all_output_tokens.push(bin_bucket);
                }
            }

            // [TEST] Check all output amount
            let mut total_output: Decimal = dec!(0);
            for i in all_output_tokens.iter() {
                total_output += i.amount();
            }
            debug!("[swap]: Total output: {}", total_output);

            all_output_tokens
            //self.xrd_fee.take(fee_amount)
        }

        // Returns the Price of an ID
        fn get_price(&mut self, id: Decimal) -> Decimal {
            // price = (1+binstep)^(activeId - 2^23)

            const DECIMAL_PLACES: i64 = 1000000000000000000;
            let id = (id.0 / BnumI256::from(DECIMAL_PLACES)).to_i64().unwrap();
            let price = (dec!(1) + self.bin_step).powi(id - 8388608);

            //let price: Decimal = (dec!(1) + self.bin_step).pow(id - dec!(2).powi(23));

            info!("[get_price]: ID: {}, Price: {}", id, price);

            price
        }

        // Returns the ID for a certain price
        fn get_id(&mut self, price: Decimal) -> Decimal {
            let id = price.log() / (dec!(1) + self.bin_step).log() + dec!(2).powi(23);

            id
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