use scrypto::prelude::*;

blueprint! {
    struct Ocipaws {
        a_vault : Vault,
        b_vault : Vault,
        xrd_vault : Vault,
        // Fee should be defined at instanciation.
        fee : Decimal,
        // The idea is to have price update through this Hashmap.
        pools : HashMap<ResourceAddress, Decimal>,

        lp_NFTs : Vault
    }

    impl Ocipaws {

        pub fn instantiate_ocipaws(fee : Decimal) -> (ComponentAddress, Bucket) {

            assert!((fee >= Decimal::zero()) & (fee <= dec!("100")), "Fee must be between 0 and 100");

            let my_a_bucket: Bucket = ResourceBuilder::new_fungible()
               .metadata("name", "TokenA")
               .metadata("symbol", "TA")
               .initial_supply(1000);

            let my_b_bucket: Bucket = ResourceBuilder::new_fungible()
              .metadata("name", "TokenB")
              .metadata("symbol", "TB")
              .initial_supply(1000);

            // For test purpose, we'll be giving manually the price for both Vaults.
            let mut my_pool = HashMap::new();
            my_pool.insert(my_a_bucket.resource_address(), 2.into());
            my_pool.insert(my_b_bucket.resource_address(), Decimal::from("0.5"));

// // TODO
// let mut tickets = Vec::new();
// tickets.push((
//                 NonFungibleId::random(),
//                 Ticket { row, column },
//             ));

    //         let ticket_bucket = ResourceBuilder::new_non_fungible(NonNonFungibleIdType::UUID)
    // .metadata("name", "Ticket")
    // .initial_supply(100); // tickets

    let ticket_bucket: Bucket = ResourceBuilder::new_fungible()
    .divisibility(DIVISIBILITY_MAXIMUM)
    .metadata("symbol", "XRD-OCI")
    .metadata("name", "LP")
    .metadata("amount", "1")
    .initial_supply(100);



            let mut component = Self {
                a_vault : Vault::with_bucket(my_a_bucket),
                b_vault : Vault::with_bucket(my_b_bucket),
                xrd_vault : Vault::new(RADIX_TOKEN),
                fee,
                pools : my_pool,

                lp_NFTs : Vault::with_bucket(ticket_bucket)
            }
            .instantiate();

            // Create the admin badges
            let admin_badge: Bucket = ResourceBuilder::new_fungible()
            .divisibility(DIVISIBILITY_NONE)
            .metadata("name", "Ocipaws Admin Badge")
            .initial_supply(1);


            // Define the access rules for this blueprint.
            let access_rules = AccessRules::new()
            .method("withdraw", rule!(require(admin_badge.resource_address())), AccessRule::DenyAll)
            .default(rule!(allow_all), AccessRule::DenyAll);

            component.add_access_check(access_rules);

            let component = component.globalize();

            (component, admin_badge)
        }

        // //TODO
        // // Create NFT for each LP.
        // #[#[scrypto(NonFungibleData)]]
        // pub struct Ticket {
        //     pub row: u32,
        //     pub column: u32,
        // }


        pub fn paws(&mut self, mut swap : Bucket, received_address : ResourceAddress) -> (Bucket, Bucket) {

            info!("My balance is: {} TokenA. Now swapping a token!", self.a_vault.amount());
            info!("My balance is: {} TokenB. Now swapping a token!", self.b_vault.amount());

            //assert_ne!(swap.resource_address(), self.xrd_vault.resource_address(),"Need XRD!");

            self.xrd_vault.put(swap.take(swap.amount())); //  + (swap.amount() * self.fee)

            if received_address == self.a_vault.resource_address() {
                return (self.a_vault.take(swap.amount() * *self.pools.get(&received_address).unwrap()), swap);
            } else if received_address == self.b_vault.resource_address() {
                return (self.b_vault.take(swap.amount() * *self.pools.get(&received_address).unwrap()), swap);
            } else {
                panic!("Address not found!");
            }

        }

        pub fn withdraw(&mut self) -> Bucket {
            // This method can only be called if the caller presents an admin badge
            self.xrd_vault.take_all()
        }

//         // TODO
//         pub fn buy_ticket_by_id(&mut self, id: u128, mut payment: Bucket) -> (Bucket, Bucket) {
//     // Take our price out of the payment bucket
//     self.collected_xrd.put(payment.take(self.ticket_price));

//     // Take the specific ticket
//     let ticket = self
//         .available_tickets
//         .take_non_fungible(&NonFungibleId::UUID(id));

//     // Return the ticket and change
//     (ticket, payment)
// }

    }
}
