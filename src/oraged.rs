// This file is Copyright its original authors, visible in version control
// history.
//
// This file is licensed under the Apache License, Version 2.0 <LICENSE-APACHE
// or http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT> or http:://opensource.org/licenses/MIT>, at your option.
// You may not use this file except in accordance with one or both of these
// licenses.

use tonic::{transport::Server, Request, Response, Status};

use bitcoin::BlockHash;
use bitcoin::blockdata::transaction::Transaction;
use bitcoin::blockdata::constants::genesis_block;
use bitcoin::network::constants::Network;

use lightning::ln::channelmanager;
use lightning::ln::channelmanager::SimpleArcChannelManager;
use lightning::ln::channelmanager::ChainParameters;
use lightning::ln::{PaymentHash, PaymentPreimage, PaymentSecret};
use lightning::events::Event;
use lightning::ln::peer_handler::{IgnoringMessageHandler, MessageHandler, SimpleArcPeerManager};
use lightning::chain::chainmonitor;
use lightning::chain::channelmonitor::ChannelMonitor;
use lightning::chain::keysinterface::{EntropySource, KeysManager, InMemorySigner, SignerProvider};
use lightning::chain::chaininterface::{BroadcasterInterface, ConfirmationTarget, FeeEstimator};
use lightning::chain::{BestBlock, Filter};
use lightning::onion_message::SimpleArcOnionMessenger;
use lightning::routing::gossip;
use lightning::routing::gossip::P2PGossipSync;
use lightning::routing::router::DefaultRouter;
use lightning::routing::scoring::{ProbabilisticScorer, ProbabilisticScoringParameters};
use lightning::routing::utxo::{UtxoLookup, UtxoResult};
use lightning::util::config::UserConfig;
use lightning::util::logger::{Logger, Record};
use lightning::util::ser::Writer;
use lightning_background_processor::{process_events_async, GossipSync};
use lightning::util::persist::KVStorePersister;
use lightning_block_sync::init;
use lightning_block_sync::poll;
use lightning_block_sync::SpvClient;
use lightning_block_sync::UnboundedCache;
use lightning_net_tokio::SocketDescriptor;
use lightning_persister::FilesystemPersister;
use lightning::util::ser::Writeable;
use lightning::io;

use crate::oragectrl::orage_ctrl_server::{OrageCtrl, OrageCtrlServer};

use rand::{thread_rng, Rng};

use std::collections::HashMap;
use std::env;
use std::ffi::OsStr;
use std::fs;
use std::ops::Deref;
use std::sync::{Arc, Mutex, MutexGuard};
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::SystemTime;
use time::{OffsetDateTime};
use toml::value::{Table, Value};

use tokio::runtime::Runtime;

struct Store {}

impl Store {
	fn read_channelmonitors<SP: Deref>(&self) -> std::io::Result<Vec<(BlockHash, ChannelMonitor<<SP::Target as SignerProvider>::Signer>)>> 
		where
		    SP::Target: SignerProvider + Sized
	{
		Ok(vec![])
	}
}

impl KVStorePersister for Store {
	fn persist<W: Writeable>(&self, key: &str, object: &W) -> io::Result<()> {
		Ok(())
	}
}

pub struct FilesystemLogger{}
impl Logger for FilesystemLogger {
	fn log(&self, record: &Record) {
		let raw_log = record.args.to_string();
		let log = format!("{} {:<5} [{}:{}] {}", OffsetDateTime::now_utc().format("%F %T"),
		record.level.to_string(), record.module_path, record.line, raw_log);
		fs::create_dir_all("logs").unwrap();
		fs::OpenOptions::new().create(true).append(true).open("./logs/logs.txt").unwrap().write_all(log.as_bytes()).unwrap();
	}
}

pub struct BaseLayerStateProvider {}

impl BaseLayerStateProvider {
	pub fn new(host: String, port: u16, path: Option<String>, rpc_user: String, rpc_password: String) ->
		std::io::Result<Self>
	{
		Ok(Self {})
	}
}

impl FeeEstimator for BaseLayerStateProvider {
	fn get_est_sat_per_1000_weight(&self, confirmation_target: ConfirmationTarget) -> u32 {
		return 100;
	}
}

impl BroadcasterInterface for BaseLayerStateProvider {
	fn broadcast_transaction(&self, tx: &Transaction) {

	}
}

impl UtxoLookup for BaseLayerStateProvider {
	fn get_utxo(&self, genesis_hash: &BlockHash, short_channel_id: u64) -> UtxoResult {
		todo!();
	}
}

//impl BlockSource for BaseLayerStateProvider {
//
//}

async fn handle_ldk_events(
	channel_manager: &Arc<ChannelManager>, base_layer_state_provider: &BaseLayerStateProvider,
	network_graph: &NetworkGraph, keys_manager: &KeysManager,
	inbound_payemnts: &PaymentInfoStorage, outbound_payments: &PaymentInfoStorage,
	persister: &Arc<Store>, network: Network, event: Event
) {
	todo!();
}

enum HTLCStatus {
	Pending,
	Succeeded,
	Failed,
}

struct MillisatAmount(Option<u64>);

struct PaymentInfo {
	preimage: Option<PaymentPreimage>,
	secret: Option<PaymentSecret>,
	status: HTLCStatus,
	amt_msat: MillisatAmount,
}

type PaymentInfoStorage = Arc<Mutex<HashMap<PaymentHash, PaymentInfo>>>;

type ChannelManager = SimpleArcChannelManager<ChainMonitor, BaseLayerStateProvider, BaseLayerStateProvider, FilesystemLogger>;

type NetworkGraph = gossip::NetworkGraph<Arc<FilesystemLogger>>;

type OnionMessenger = SimpleArcOnionMessenger<FilesystemLogger>;

type ChainMonitor = chainmonitor::ChainMonitor<
	InMemorySigner,
	Arc<dyn Filter + Send + Sync>,
	Arc<BaseLayerStateProvider>,
	Arc<BaseLayerStateProvider>,
	Arc<FilesystemLogger>,
	Arc<Store>,
>;

type PeerManager = SimpleArcPeerManager<
	SocketDescriptor,
	ChainMonitor,
	BaseLayerStateProvider,
	BaseLayerStateProvider,
	BaseLayerStateProvider,
	FilesystemLogger,
>;

pub mod oragectrl {
	tonic::include_proto!("oragectrl");
}

#[tonic::async_trait]
impl OrageCtrl for OrageManager {
	async fn orage_status(&self, request: Request<oragectrl::StatusRequest>) -> Result<Response<oragectrl::StatusReply>, Status> {
		println!("[ORAGED] orage status request");

		Ok(Response::new(oragectrl::StatusReply {}))
	}
	
	async fn add_gossip(&self, request: Request<oragectrl::GossipInsertionRequest>) -> Result<Response<oragectrl::GossipInsertionReply>, Status> {
		println!("[ORAGED] add gossip request");

		Ok(Response::new(oragectrl::GossipInsertionReply {}))
	}

	async fn autoclean_invoice(&self, request: Request<oragectrl::AutocleanInvoiceRequest>) -> Result<Response<oragectrl::AutocleanInvoiceReply>, Status> {
		println!("[ORAGED] autoclean invoice request");

		Ok(Response::new(oragectrl::AutocleanInvoiceReply {}))
	}

	async fn check(&self, request: Request<oragectrl::CheckRequest>) -> Result<Response<oragectrl::CheckReply>, Status> {
		println!("[ORAGED] check");

		Ok(Response::new(oragectrl::CheckReply {}))
	}

	async fn check_message(&self, request: Request<oragectrl::CheckMessageRequest>) -> Result<Response<oragectrl::CheckMessageReply>, Status> {
		println!("[ORAGED] check message");

		Ok(Response::new(oragectrl::CheckMessageReply {}))
	}

	async fn close(&self, request: Request<oragectrl::CloseRequest>) -> Result<Response<oragectrl::CloseReply>, Status> {
		println!("[ORAGED] check message");

		Ok(Response::new(oragectrl::CloseReply {}))
	}

	async fn connect_orage(&self, request: Request<oragectrl::ConnectOrageRequest>) -> Result<Response<oragectrl::ConnectOrageReply>, Status> {
		println!("[ORAGED] connect");

		Ok(Response::new(oragectrl::ConnectOrageReply {}))
	}

	async fn create_invoice(&self, request: Request<oragectrl::CreateInvoiceRequest>) -> Result<Response<oragectrl::CreateInvoiceReply>, Status> {
		println!("[ORAGED] create invoice");

		Ok(Response::new(oragectrl::CreateInvoiceReply {}))
	}
}

struct OrageManager {}

impl OrageManager
{
	pub fn new() -> Self {
		OrageManager {}
	}
}

//#[derive(Parser, Debug)]
//struct Cli {
//	data_path 
//
//	cli_port 
//}

const ORAGED_DIR_PATH: &str = ".";

#[tokio::main]
pub async fn main() -> Result<(), Box<dyn std::error::Error>> {


	let orage_manager = OrageManager::new();

	let cli_port = 20001;

	let addr = format!("[::1]:{}", cli_port).parse()?;

	let orage_svc = Server::builder()
		.add_service(OrageCtrlServer::new(orage_manager))
		.serve(addr);

    	tokio::spawn(async move {
		if let Err(e) = orage_svc.await {
			eprintln!("Error = {:?}", e);
		}
	});

	let bitcoind_host = "127.0.0.1".to_string();
	let bitcoind_port = 18443;

	let rpc_user = "orageuser".to_string();
	let rpc_password = "oragepass".to_string();

	//TODO: start bitcoind client in its own thread
	let state_provider = Arc::new(BaseLayerStateProvider::new(bitcoind_host.clone(), bitcoind_port, None,
						rpc_user.clone(), rpc_password.clone()).unwrap());

	let fee_estimator = state_provider.clone();

	let logger = Arc::new(FilesystemLogger{});

	let broadcaster = state_provider.clone();

	let persister = Arc::new(Store{});

	let chain_monitor: Arc<ChainMonitor> = Arc::new(chainmonitor::ChainMonitor::new(None, broadcaster.clone(),
							logger.clone(), fee_estimator.clone(),
							persister.clone()));

	//TODO: add toml config parser to access data path
	let node_privkey = {
		let mut key = [0;32];
		thread_rng().fill_bytes(&mut key);
		key
	};

	let cur = SystemTime::now().duration_since(SystemTime::UNIX_EPOCH).unwrap();

	let keys_manager = Arc::new(KeysManager::new(&node_privkey, cur.as_secs(), cur.subsec_nanos()));

	//TODO: add toml config parser to access canonical data path and feed it to network graph
	let network_graph_path = format!("{}/network_graph", ORAGED_DIR_PATH);

	let network_graph = Arc::new(NetworkGraph::new(bitcoin::Network::Bitcoin, logger.clone()));

	//TODO: add toml config parser to access canonical data path and feed it to network graph
	let scorer_path = format!("{}/scorer", ORAGED_DIR_PATH);

	let params = ProbabilisticScoringParameters::default();
	let scorer = Arc::new(Mutex::new(ProbabilisticScorer::new(params, Arc::clone(&network_graph), logger.clone())));

	let router = Arc::new(DefaultRouter::new(
		network_graph.clone(),
		logger.clone(),
		keys_manager.get_secure_random_bytes(),
		scorer.clone(),
	));

	// Here we initialize a ChannelManager from scratch
	let mut user_config = UserConfig::default();
	let network = bitcoin::Network::Bitcoin;
	let genesis_block = BestBlock::from_network(network);
	let genesis_block_hash = genesis_block.block_hash();

	//TODO: read channel monitors from disk

	let chain_params = ChainParameters { network, best_block: genesis_block };
	let fresh_channel_manager = channelmanager::ChannelManager::new(
		fee_estimator.clone(),
		chain_monitor.clone(),
		broadcaster.clone(),
		router,
		logger.clone(),
		keys_manager.clone(),
		keys_manager.clone(),
		keys_manager.clone(),
		user_config,
		chain_params,
	);

	//TODO: give channel deser channel monitor to chain monitor

	let gossip_sync = Arc::new(P2PGossipSync::new(
		Arc::clone(&network_graph),
		None::<Arc<BaseLayerStateProvider>>,
		logger.clone(),
	));

	let channel_manager: Arc<ChannelManager> = Arc::new(fresh_channel_manager);
	let onion_messenger: Arc<OnionMessenger> = Arc::new(OnionMessenger::new(
		Arc::clone(&keys_manager),
		Arc::clone(&keys_manager),
		Arc::clone(&logger),
		IgnoringMessageHandler {},
	));
	let mut ephemeral_bytes = [0; 32];
	let current_time = SystemTime::now().duration_since(SystemTime::UNIX_EPOCH).unwrap().as_secs();
	rand::thread_rng().fill_bytes(&mut ephemeral_bytes);
	let lightning_msg_handler = MessageHandler {
		chan_handler: channel_manager.clone(),
		route_handler: gossip_sync.clone(),
		onion_message_handler: onion_messenger.clone(),
	};
	let peer_manager: Arc<PeerManager> = Arc::new(PeerManager::new(
		lightning_msg_handler,
		current_time.try_into().unwrap(),
		&ephemeral_bytes,
		logger.clone(),
		IgnoringMessageHandler {},
		Arc::clone(&keys_manager),
	));

	let peer_manager_connection_handler = peer_manager.clone();
	//TODO: abstract, just use default LN port for now.
	let listening_port = 9735;
	let stop_listen_connect = Arc::new(AtomicBool::new(false));
	let stop_listen = Arc::clone(&stop_listen_connect);
	tokio::spawn(async move {
		let listener = tokio::net::TcpListener::bind(format!("[::]:{}", listening_port))
			.await
			.expect("Failed to bind to listen port - is something else already listening on it ?");
		loop {
			let peer_mgr = peer_manager_connection_handler.clone();
			let tcp_stream = listener.accept().await.unwrap().0;
			if stop_listen.load(Ordering::Acquire) {
				return;
			}
			tokio::spawn(async move {
				lightning_net_tokio::setup_inbound(
					peer_mgr.clone(),
					tcp_stream.into_std().unwrap(),
				)
				.await;
			});
		}
	});

	let channel_manager_listener = channel_manager.clone();
	let chain_monitor_listener = chain_monitor.clone();
	let state_provider = state_provider.clone();
	let network = bitcoin::Network::Bitcoin;
	let mut cache = UnboundedCache::new();

	tokio::spawn(async move {
		//let chain_poller = poll::ChainPoller::new(state_provider.as_ref(), network);
		let chain_listener = (chain_monitor_listener, channel_manager_listener);
		//let mut spv_client = SpvClient::new(chain_tip, chain_poller, &mut cache, &chain_listener);
		//loop {
		//	spv_client.poll_best_tip().await.unwrap();
		//	tokio::time::sleep(Duration::from_secs(1)).await;
		//}
	});

	let inbound_payments: PaymentInfoStorage = Arc::new(Mutex::new(HashMap::new()));
	let outbound_payments: PaymentInfoStorage = Arc::new(Mutex::new(HashMap::new()));

	let channel_manager_event_listener = Arc::clone(&channel_manager);
	let state_provider_listener = Arc::clone(&state_provider);
	let network_graph_event_listener = Arc::clone(&network_graph);
	let keys_manager_event_listener = Arc::clone(&keys_manager);
	let inbound_payments_event_listener = Arc::clone(&inbound_payments);
	let outbound_payments_event_listener = Arc::clone(&outbound_payments);
	let persister_event_listener = Arc::clone(&persister);
	let event_handler = move |event: Event| {
		let channel_manager_event_listener = Arc::clone(&channel_manager_event_listener);
		let state_provider_listener = Arc::clone(&state_provider_listener);
		let network_graph_event_listener = Arc::clone(&network_graph_event_listener);
		let keys_manager_event_listener = Arc::clone(&keys_manager_event_listener);
		let inbound_payments_event_listener = Arc::clone(&inbound_payments_event_listener);
		let outbound_payments_event_listener = Arc::clone(&outbound_payments_event_listener);
		let persister_event_listener = Arc::clone(&persister_event_listener);
		async move {
			handle_ldk_events(
				&channel_manager_event_listener,
				&state_provider_listener,
				&network_graph_event_listener,
				&keys_manager_event_listener,
				&inbound_payments_event_listener,
				&outbound_payments_event_listener,
				&persister_event_listener,
				network,
				event,
			)
			.await;
		}
	};

	let (bp_exit, bp_exit_check) = tokio::sync::watch::channel(());
	let background_processor = tokio::spawn(process_events_async(
		Arc::clone(&persister),
		event_handler,
		chain_monitor.clone(),
		channel_manager.clone(),
		GossipSync::p2p(gossip_sync.clone()),
		peer_manager.clone(),
		logger.clone(),
		Some(scorer.clone()),
		move |t| {
			let mut bp_exit_fut_check = bp_exit_check.clone();
			Box::pin(async move {
				tokio::select! {
					_ = tokio::time::sleep(t) => false,
					_ = bp_exit_fut_check.changed() => true,
				}
			})
		},
		false,
	));

	//TODO: reconnect to channel peers
	//TODO: broadcast node_announcement

	//TODO: is sweeping the right approach ? should we rather return them to our
	// own fee-bumping / liquidity ressource ?

	loop {}

	Ok(())
}
