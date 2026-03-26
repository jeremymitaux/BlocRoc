//! # Marketplace Pallet
//!
//! Secondary ticket marketplace for BlocRoc.
//!
//! ## Overview
//!
//! Ticket owners can list their tickets for resale at a fixed price. Buyers can
//! purchase listed tickets directly. Organizers can configure a maximum resale
//! price and a royalty percentage taken on each secondary sale.
//!
//! ## Dispatchables
//!
//! - `list` — Ticket owner lists a ticket for sale at a given price.
//! - `delist` — Ticket owner cancels their listing.
//! - `buy` — Buyer purchases a listed ticket (funds transferred, ownership updated).

#![cfg_attr(not(feature = "std"), no_std)]

pub use pallet::*;

#[cfg(test)]
mod mock;
#[cfg(test)]
mod tests;

#[cfg(feature = "runtime-benchmarks")]
mod benchmarking;
pub mod weights;

#[frame_support::pallet]
pub mod pallet {
    use crate::weights::WeightInfo;
    use frame_support::pallet_prelude::*;
    use frame_system::pallet_prelude::*;

    // ── Pallet declaration ────────────────────────────────────────────────

    #[pallet::pallet]
    pub struct Pallet<T>(_);

    // ── Config ────────────────────────────────────────────────────────────

    #[pallet::config]
    pub trait Config: frame_system::Config {
        type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;
        type WeightInfo: WeightInfo;
    }

    // ── Types ─────────────────────────────────────────────────────────────

    /// An active ticket listing on the secondary market.
    #[derive(Clone, Encode, Decode, PartialEq, RuntimeDebug, TypeInfo, MaxEncodedLen)]
    #[scale_info(skip_type_params(T))]
    pub struct Listing<T: Config> {
        /// The account selling the ticket.
        pub seller: T::AccountId,
        /// The ticket being listed.
        pub ticket_id: u64,
        /// Asking price in the chain's native token (smallest unit).
        pub price: u128,
    }

    // ── Storage ───────────────────────────────────────────────────────────

    /// Auto-incrementing listing ID counter.
    #[pallet::storage]
    #[pallet::getter(fn next_listing_id)]
    pub type NextListingId<T: Config> = StorageValue<_, u64, ValueQuery>;

    /// Active listings keyed by ListingId.
    ///
    /// Invariant: a ticket can only appear in one listing at a time.
    #[pallet::storage]
    #[pallet::getter(fn listings)]
    pub type Listings<T: Config> =
        StorageMap<_, Blake2_128Concat, u64, Listing<T>, OptionQuery>;

    /// Reverse index: TicketId → ListingId (if the ticket is currently listed).
    #[pallet::storage]
    #[pallet::getter(fn ticket_listing)]
    pub type TicketListing<T: Config> =
        StorageMap<_, Blake2_128Concat, u64, u64, OptionQuery>;

    // ── Events ────────────────────────────────────────────────────────────

    #[pallet::event]
    #[pallet::generate_deposit(pub(super) fn deposit_event)]
    pub enum Event<T: Config> {
        /// A ticket was listed for sale. [listing_id, ticket_id, seller, price]
        TicketListed { listing_id: u64, ticket_id: u64, seller: T::AccountId, price: u128 },
        /// A listing was cancelled by the seller. [listing_id, ticket_id]
        ListingCancelled { listing_id: u64, ticket_id: u64 },
        /// A ticket was purchased. [listing_id, ticket_id, seller, buyer, price]
        TicketPurchased {
            listing_id: u64,
            ticket_id: u64,
            seller: T::AccountId,
            buyer: T::AccountId,
            price: u128,
        },
    }

    // ── Errors ────────────────────────────────────────────────────────────

    #[pallet::error]
    pub enum Error<T> {
        /// The listing does not exist.
        ListingNotFound,
        /// The caller is not the listing seller.
        NotSeller,
        /// The ticket is already listed.
        AlreadyListed,
        /// Listing ID counter overflow.
        ListingIdOverflow,
    }

    // ── Dispatchables ─────────────────────────────────────────────────────

    #[pallet::call]
    impl<T: Config> Pallet<T> {
        /// List a ticket for sale.
        ///
        /// The caller must own the ticket (enforced off-chain or via cross-pallet
        /// coupling — to be tightened in a future iteration).
        ///
        /// Weight: O(1)
        #[pallet::call_index(0)]
        #[pallet::weight(T::WeightInfo::list())]
        pub fn list(
            origin: OriginFor<T>,
            ticket_id: u64,
            price: u128,
        ) -> DispatchResult {
            let seller = ensure_signed(origin)?;

            ensure!(!TicketListing::<T>::contains_key(ticket_id), Error::<T>::AlreadyListed);

            let listing_id = Self::next_listing_id();
            let next = listing_id.checked_add(1).ok_or(Error::<T>::ListingIdOverflow)?;

            NextListingId::<T>::put(next);
            Listings::<T>::insert(listing_id, Listing { seller: seller.clone(), ticket_id, price });
            TicketListing::<T>::insert(ticket_id, listing_id);

            Self::deposit_event(Event::TicketListed { listing_id, ticket_id, seller, price });
            Ok(())
        }

        /// Cancel a ticket listing.
        ///
        /// Caller must be the original seller.
        ///
        /// Weight: O(1)
        #[pallet::call_index(1)]
        #[pallet::weight(T::WeightInfo::delist())]
        pub fn delist(origin: OriginFor<T>, listing_id: u64) -> DispatchResult {
            let caller = ensure_signed(origin)?;

            let listing =
                Listings::<T>::get(listing_id).ok_or(Error::<T>::ListingNotFound)?;
            ensure!(listing.seller == caller, Error::<T>::NotSeller);

            Listings::<T>::remove(listing_id);
            TicketListing::<T>::remove(listing.ticket_id);

            Self::deposit_event(Event::ListingCancelled {
                listing_id,
                ticket_id: listing.ticket_id,
            });
            Ok(())
        }

        /// Purchase a listed ticket.
        ///
        /// In this scaffold the balance transfer is not wired up — cross-pallet
        /// currency coupling will be added in the next iteration.
        ///
        /// Weight: O(1)
        #[pallet::call_index(2)]
        #[pallet::weight(T::WeightInfo::buy())]
        pub fn buy(origin: OriginFor<T>, listing_id: u64) -> DispatchResult {
            let buyer = ensure_signed(origin)?;

            let listing =
                Listings::<T>::get(listing_id).ok_or(Error::<T>::ListingNotFound)?;

            // TODO: transfer `listing.price` from buyer to seller via pallet-balances
            // TODO: transfer ticket ownership via pallet-ticket

            Listings::<T>::remove(listing_id);
            TicketListing::<T>::remove(listing.ticket_id);

            Self::deposit_event(Event::TicketPurchased {
                listing_id,
                ticket_id: listing.ticket_id,
                seller: listing.seller,
                buyer,
                price: listing.price,
            });
            Ok(())
        }
    }
}
