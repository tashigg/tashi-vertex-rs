use std::os::raw::c_void;

use crate::{KeyPublic, error::TVResult, ptr::Pointer};

/// An event that has reached consensus in the Tashi Vertex network.
pub struct Event {
    pub(crate) handle: Pointer<TVEvent>,
}

// NOTE: we directly access the first few fields of the TVEvent struct here
//  rather than creating FFI functions for them, because they are simple
//  fields and this reduces the amount of FFI boilerplate needed.
#[repr(C)]
struct TVEventFields {
    /// The timestamp that this event has reached consensus.
    consensus_at: u64,

    /// A cryptographically secure hash of this event.
    hash: [u8; 32],

    /// The public key of the peer that created the event.
    creator: KeyPublic,
}

impl Event {
    /// Gets the Unix timestamp at which this event was created.
    pub fn created_at(&self) -> u64 {
        let mut created_at: u64 = 0;

        unsafe { tv_event_get_created_at(self.handle.as_ptr(), &mut created_at) }.assert_ok();

        created_at
    }

    /// Gets the Unix timestamp at which this event has reached consensus.
    pub fn consensus_at(&self) -> u64 {
        let fields = self.handle.as_ptr() as *const TVEventFields;

        unsafe { (*fields).consensus_at }
    }

    /// Gets a cryptographically secure hash of this event.
    pub fn hash(&self) -> &[u8; 32] {
        let fields = self.handle.as_ptr() as *const TVEventFields;

        unsafe { &(*fields).hash }
    }

    /// Gets the whitened signature, created by bytewise XORing the signature with the signatures
    /// of all the unique famous witnesses for this event.
    ///
    /// This value is infeasible to predict ahead of time and is relatively high in entropy,
    /// but all peers that see the event come to consensus will calculate the same result,
    /// which makes it a good seed for a consensus-driven PRNG.
    ///
    pub fn whitened_signature(&self) -> &[u8] {
        let mut signature_ptr: *mut u8 = std::ptr::null_mut();
        let mut signature_size: usize = 0;

        unsafe {
            tv_event_get_whitened_signature(
                self.handle.as_ptr(),
                &mut signature_ptr,
                &mut signature_size,
            )
        }
        .assert_ok();

        unsafe { std::slice::from_raw_parts(signature_ptr, signature_size) }
    }

    /// Gets the public key of the peer that created the event.
    pub fn creator(&self) -> &KeyPublic {
        let fields = self.handle.as_ptr() as *const TVEventFields;

        unsafe { &(*fields).creator }
    }

    /// Gets the number of transactions contained in this event.
    pub fn transaction_count(&self) -> usize {
        let mut count: usize = 0;

        unsafe { tv_event_get_transaction_count(self.handle.as_ptr(), &mut count) }.assert_ok();

        count
    }

    /// Gets the transaction at the given index contained in this event.
    pub fn transaction(&self, index: usize) -> Option<&[u8]> {
        if index >= self.transaction_count() {
            return None;
        }

        let mut transaction_ptr: *mut u8 = std::ptr::null_mut();
        let mut transaction_size: usize = 0;

        unsafe {
            tv_event_get_transaction(
                self.handle.as_ptr(),
                index,
                &mut transaction_ptr,
                &mut transaction_size,
            )
        }
        .assert_ok();

        let transaction = unsafe { std::slice::from_raw_parts(transaction_ptr, transaction_size) };

        Some(transaction)
    }

    /// Gets an iterator over the transactions contained in this event.
    pub fn transactions(&self) -> impl Iterator<Item = &'_ [u8]> {
        EventTransactionIter {
            event: self,
            index: 0,
            count: self.transaction_count(),
        }
    }
}

pub struct EventTransactionIter<'e> {
    event: &'e Event,
    index: usize,
    count: usize,
}

impl<'e> Iterator for EventTransactionIter<'e> {
    type Item = &'e [u8];

    fn next(&mut self) -> Option<Self::Item> {
        let transaction = self.event.transaction(self.index)?;

        self.index += 1;

        Some(transaction)
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let remaining = self.count - self.index;
        (remaining, Some(remaining))
    }
}

impl ExactSizeIterator for EventTransactionIter<'_> {}

type TVEvent = c_void;

unsafe extern "C" {
    fn tv_event_get_created_at(event: *const TVEvent, created_at: *mut u64) -> TVResult;

    fn tv_event_get_transaction_count(event: *const TVEvent, count: *mut usize) -> TVResult;

    fn tv_event_get_transaction(
        event: *const TVEvent,
        index: usize,
        transaction: *mut *mut u8,
        transaction_size: *mut usize,
    ) -> TVResult;

    fn tv_event_get_whitened_signature(
        event: *const TVEvent,
        signature: *mut *mut u8,
        signature_size: *mut usize,
    ) -> TVResult;
}
