use std::mem::{self, MaybeUninit};
use std::os::raw::c_void;

use crate::context::TVContext;
use crate::error::TVResult;
use crate::message::Message;
use crate::options::TVOptions;
use crate::peers::TVPeers;
use crate::ptr::Pointer;
use crate::socket::TVSocket;
use crate::{Context, KeySecret, Options, Peers, Socket, Transaction};

/// Handle for the Tashi Vertex (TV) engine.
pub struct Engine {
    pub(crate) handle: Pointer<TVEngine>,
}

impl Engine {
    /// Starts the consensus engine.
    pub fn start(
        context: &Context,
        socket: Socket,
        options: Options,
        secret: &KeySecret,
        peers: Peers,
        joining_running_session: bool,
    ) -> crate::Result<Self> {
        let mut socket_ptr = socket.handle.as_ptr();
        let mut options_ptr = options.handle.as_ptr();
        let mut peers_ptr = peers.handle.as_ptr();

        // ownership of these pointers is transferred to the engine
        mem::forget(socket);
        mem::forget(options);
        mem::forget(peers);

        let mut handle = MaybeUninit::<Pointer<TVEngine>>::uninit();

        unsafe {
            tv_engine_start(
                context.handle.as_ptr(),
                &mut socket_ptr,
                &mut options_ptr,
                secret,
                &mut peers_ptr,
                handle.as_mut_ptr(),
                joining_running_session
            )
        }
        .ok()?;

        let handle = unsafe { handle.assume_init() };

        Ok(Self { handle })
    }

    /// Listens for the next incoming message on the given engine.
    pub async fn recv_message(&self) -> crate::Result<Option<Message>> {
        Message::recv(self).await
    }

    /// Sends a transaction to the network.
    pub fn send_transaction(&self, transaction: Transaction) -> crate::Result<()> {
        transaction.send(self)
    }

    /// Gracefully stops the consensus engine.
    ///
    /// Signals the engine to wind down. After this returns, an in-flight or
    /// subsequent [`recv_message`](Self::recv_message) resolves to `Ok(None)`
    /// once the engine has stopped — even if the session has stalled and no
    /// further messages would otherwise arrive, which a cooperative flag on the
    /// consumer side cannot achieve on its own. Idempotent; safe to call more
    /// than once.
    ///
    /// Only signals the engine (a non-blocking watch-channel send), so it is
    /// safe to call from any context — including from inside the async loop that
    /// is driving [`recv_message`](Self::recv_message).
    pub fn stop(&self) -> crate::Result<()> {
        unsafe { tv_engine_stop(self.handle.as_ptr()) }.ok()
    }
}

/// Signals a graceful stop before the engine handle is freed, so the engine
/// winds down cooperatively rather than being abandoned. The explicit
/// [`stop`](Engine::stop) path is still preferred when the caller needs to
/// observe the wind-down (e.g. by draining `recv_message` to `None`).
impl Drop for Engine {
    fn drop(&mut self) {
        let _ = self.stop();
    }
}

pub(crate) type TVEngine = c_void;

unsafe extern "C" {
    fn tv_engine_start(
        context: *mut TVContext,
        socket: *mut *mut TVSocket,
        options: *mut *mut TVOptions,
        secret: *const KeySecret,
        peers: *mut *mut TVPeers,
        engine: *mut Pointer<TVEngine>,
        joining_running_session: bool
    ) -> TVResult;

    fn tv_engine_stop(engine: *const TVEngine) -> TVResult;
}
