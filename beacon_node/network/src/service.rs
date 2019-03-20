use crate::beacon_chain::BeaconChain;
use crate::error;
use crate::message_handler::{HandlerMessage, MessageHandler};
use crate::NetworkConfig;
use crossbeam_channel::{unbounded as channel, Sender, TryRecvError};
use eth2_libp2p::RPCEvent;
use eth2_libp2p::Service as LibP2PService;
use eth2_libp2p::{Libp2pEvent, PeerId};
use futures::prelude::*;
use futures::sync::oneshot;
use futures::Stream;
use slog::{debug, info, o, trace};
use std::sync::Arc;
use tokio::runtime::TaskExecutor;

/// Service that handles communication between internal services and the eth2_libp2p network service.
pub struct Service {
    //eth2_libp2p_service: Arc<Mutex<LibP2PService>>,
    eth2_libp2p_exit: oneshot::Sender<()>,
    network_send: crossbeam_channel::Sender<NetworkMessage>,
    //message_handler: MessageHandler,
    //message_handler_send: Sender<HandlerMessage>,
}

impl Service {
    pub fn new(
        beacon_chain: Arc<BeaconChain>,
        config: &NetworkConfig,
        executor: &TaskExecutor,
        log: slog::Logger,
    ) -> error::Result<(Arc<Self>, Sender<NetworkMessage>)> {
        // build the network channel
        let (network_send, network_recv) = channel::<NetworkMessage>();
        // launch message handler thread
        let message_handler_log = log.new(o!("Service" => "MessageHandler"));
        let message_handler_send = MessageHandler::spawn(
            beacon_chain,
            network_send.clone(),
            executor,
            message_handler_log,
        )?;

        // launch eth2_libp2p service
        let eth2_libp2p_log = log.new(o!("Service" => "Libp2p"));
        let eth2_libp2p_service = LibP2PService::new(config.clone(), eth2_libp2p_log)?;

        // TODO: Spawn thread to handle eth2_libp2p messages and pass to message handler thread.
        let eth2_libp2p_exit = spawn_service(
            eth2_libp2p_service,
            network_recv,
            message_handler_send,
            executor,
            log,
        )?;
        let network_service = Service {
            eth2_libp2p_exit,
            network_send: network_send.clone(),
        };

        Ok((Arc::new(network_service), network_send))
    }

    // TODO: Testing only
    pub fn send_message(&self) {
        self.network_send
            .send(NetworkMessage::Send(
                PeerId::random(),
                OutgoingMessage::NotifierTest,
            ))
            .unwrap();
    }
}

fn spawn_service(
    eth2_libp2p_service: LibP2PService,
    network_recv: crossbeam_channel::Receiver<NetworkMessage>,
    message_handler_send: crossbeam_channel::Sender<HandlerMessage>,
    executor: &TaskExecutor,
    log: slog::Logger,
) -> error::Result<oneshot::Sender<()>> {
    let (network_exit, exit_rx) = oneshot::channel();

    // spawn on the current executor
    executor.spawn(
        network_service(
            eth2_libp2p_service,
            network_recv,
            message_handler_send,
            log.clone(),
        )
        // allow for manual termination
        .select(exit_rx.then(|_| Ok(())))
        .then(move |_| {
            info!(log.clone(), "Network service shutdown");
            Ok(())
        }),
    );

    Ok(network_exit)
}

fn network_service(
    mut eth2_libp2p_service: LibP2PService,
    network_recv: crossbeam_channel::Receiver<NetworkMessage>,
    message_handler_send: crossbeam_channel::Sender<HandlerMessage>,
    log: slog::Logger,
) -> impl futures::Future<Item = (), Error = eth2_libp2p::error::Error> {
    futures::future::poll_fn(move || -> Result<_, eth2_libp2p::error::Error> {
        // poll the swarm
        loop {
            match eth2_libp2p_service.poll() {
                Ok(Async::Ready(Some(Libp2pEvent::RPC(peer_id, rpc_event)))) => {
                    trace!(
                        eth2_libp2p_service.log,
                        "RPC Event: RPC message received: {:?}",
                        rpc_event
                    );
                    message_handler_send
                        .send(HandlerMessage::RPC(peer_id, rpc_event))
                        .map_err(|_| "failed to send rpc to handler")?;
                }
                Ok(Async::Ready(Some(Libp2pEvent::PeerDialed(peer_id)))) => {
                    debug!(eth2_libp2p_service.log, "Peer Dialed: {:?}", peer_id);
                    message_handler_send
                        .send(HandlerMessage::PeerDialed(peer_id))
                        .map_err(|_| "failed to send rpc to handler")?;
                }
                Ok(Async::Ready(Some(Libp2pEvent::Message(m)))) => debug!(
                    eth2_libp2p_service.log,
                    "Network Service: Message received: {}", m
                ),
                _ => break,
            }
        }
        // poll the network channel
        // TODO: refactor - combine poll_fn's?
        loop {
            match network_recv.try_recv() {
                // TODO: Testing message - remove
                Ok(NetworkMessage::Send(peer_id, outgoing_message)) => {
                    match outgoing_message {
                        OutgoingMessage::RPC(rpc_event) => {
                            trace!(log, "Sending RPC Event: {:?}", rpc_event);
                            //TODO: Make swarm private
                            //TODO: Implement correct peer id topic message handling
                            eth2_libp2p_service.swarm.send_rpc(peer_id, rpc_event);
                        }
                        OutgoingMessage::NotifierTest => {
                            debug!(log, "Received message from notifier");
                        }
                    };
                }
                Err(TryRecvError::Empty) => break,
                Err(TryRecvError::Disconnected) => {
                    return Err(eth2_libp2p::error::Error::from(
                        "Network channel disconnected",
                    ));
                }
            }
        }
        Ok(Async::NotReady)
    })
}

/// Types of messages that the network service can receive.
#[derive(Debug, Clone)]
pub enum NetworkMessage {
    /// Send a message to eth2_libp2p service.
    //TODO: Define typing for messages across the wire
    Send(PeerId, OutgoingMessage),
}

/// Type of outgoing messages that can be sent through the network service.
#[derive(Debug, Clone)]
pub enum OutgoingMessage {
    /// Send an RPC request/response.
    RPC(RPCEvent),
    //TODO: Remove
    NotifierTest,
}
