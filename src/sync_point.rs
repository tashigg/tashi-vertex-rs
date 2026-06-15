use std::ffi::c_void;

use crate::error::TVResult;
use crate::ptr::Pointer;

/// A sync point describes a decision or action related to the management of
/// the consensus engine which a super-majority of peers agreed upon.
pub struct SyncPoint {
    pub(crate) handle: Pointer<TVSyncPoint>,
}

impl SyncPoint {
    /// The index of the epoch this sync point begins.
    ///
    /// A new epoch is the application's cue to checkpoint its state for this
    /// epoch (and, if it submits state proofs, to report or request state).
    pub fn epoch_index(&self) -> u64 {
        let mut epoch_index = 0u64;

        unsafe { tv_sync_point_get_epoch_index(self.handle.as_ptr(), &mut epoch_index) }
            .assert_ok();

        epoch_index
    }

    /// Whether this is the first sync point observed after the node synced into
    /// the session. On a `just_synced` sync point the application should load
    /// its initial state.
    pub fn just_synced(&self) -> bool {
        let mut just_synced = false;

        unsafe { tv_sync_point_is_just_synced(self.handle.as_ptr(), &mut just_synced) }
            .assert_ok();

        just_synced
    }

    /// Whether this sync point marks the end of the session, after which no
    /// further consensus messages will be produced.
    pub fn session_ended(&self) -> bool {
        let mut session_ended = false;

        unsafe { tv_sync_point_is_session_ended(self.handle.as_ptr(), &mut session_ended) }
            .assert_ok();

        session_ended
    }
}

pub(crate) type TVSyncPoint = c_void;

unsafe extern "C" {
    fn tv_sync_point_get_epoch_index(sync_point: *const TVSyncPoint, epoch_index: *mut u64)
    -> TVResult;

    fn tv_sync_point_is_just_synced(sync_point: *const TVSyncPoint, just_synced: *mut bool)
    -> TVResult;

    fn tv_sync_point_is_session_ended(
        sync_point: *const TVSyncPoint,
        session_ended: *mut bool,
    ) -> TVResult;
}
