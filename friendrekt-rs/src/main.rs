mod bindings;
mod bset;
mod fasthttp;
mod math;
mod prod_kosetto;

use futures::stream;
use tokio::sync::Mutex;
use prod_kosetto::{
    TwitterInfo,
    User,
};
use ethers::{
	providers::{Middleware, Provider, StreamExt, Ws},
	signers::LocalWallet,
    middleware::SignerMiddleware,
	signers::Signer,
    types::{Address, Bytes, Eip1559TransactionRequest, NameOrAddress, H256, U256, U64},
    types::transaction::eip2930::AccessList,
    utils::hex
};
use std::{
    env,
    collections::HashMap,
    fs::OpenOptions,
    io::Write,
    str::FromStr,
    sync::Arc,
    time::Duration,
};

use bindings::shares::shares::shares;
use bindings::sniper::sniper::sniper;
use bset::FIFOCache;

const MAX: u64 = 100;

/*
    here's a brief overview of the sniper:
    there are two tokio threads. 

    1. listen to every block.
        for every ETH transfer or bridge relays:
            reverse search the addresses involved on friend.tech
            find the number of followers
            if the address is not cached:
                cache the address

    2. listen to blast-api eth_newPendingTransactions
        there are multiple (4?) backend nodes so subscribe a few times
            (afaict its mostly rng which stream you get)
            (also run this bot on several geo-distributed servers)
            if its a first-share-buy (a signup):
                if the address is cached:
                    if follow count > 20k: send snipe tx
                otherwise:
                    do a live lookup of follow count
                    if follow count > 20k: send snipe tx
*/

async fn reverse_search(address: Address) -> Option<TwitterInfo> {
	let url = format!("https://prod-api.kosetto.com/users/{:?}", address);
	let client = reqwest::Client::new();

	if let Ok(resp) = client
		.get(url)
		.timeout(Duration::from_secs(60))
		.headers(fasthttp::make_headers())
		.send()
		.await
	{
		if resp.status().is_success() {
			if let Ok(data) = resp.text().await {
				if let Ok(response) = serde_json::from_str::<User>(&data) {
					let followers = get_followers(response.twitterUserId.clone()).await;
                    // Depending on the number of followers, set how 
                    // much we are willing to pay

					let supply_limit = match followers {
						f if f > 1_000_000 => MAX,
						f if f > 500_000 => 60,
						f if f > 250_000 => 60,
						f if f > 100_000 => 40,
						f if f > 20_000 => 30,
						_ => 0,
					};
					println!(
						"Found {} with {} followers",
						response.twitterUsername, followers
					);
					return Some(TwitterInfo {
						twitter_username: response.twitterUsername,
						twitter_user_id: response.twitterUserId,
						followers,
						supply_limit,
					});
				}
			}
		}
	}
	None
}

async fn get_followers(id: String) -> u64 {
	let newurl = format!("http://127.0.0.1:5000/followers/{}", id);
	let client = reqwest::Client::new();
	let resp = client.get(newurl).headers(fasthttp::make_headers()).send().await;
	if resp.is_err() || !resp.as_ref().unwrap().status().is_success() {
		println!("Failed to ping follower endpoint");
		return 0;
	}
	let data = resp.unwrap().text().await.unwrap();
	data.parse().unwrap_or(0)
}

use dotenv::dotenv;
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
	dotenv().ok();
	let args = env::args().collect::<Vec<String>>();
	if args.len() < 3 {
		println!("Usage: friendrekt <amount> <ws>");
		return Ok(());
	}

	// Setup the sniping wallet
	let rpc = args[2].clone();
	let private_key = env::var("PRIVATE_KEY").expect("PRIVATE_KEY must be set in .env");
	let sniper_address = env::var("SNIPER_ADDRESS").expect("SNIPER_ADDRESS must be set in .env");
    let ft_address = env::var("FT_ADDRESS").expect("FT_ADDRESS must be set in .env");

	let _httpprovider = Arc::new(Provider::try_from("https://mainnet.base.org").unwrap());
	let client = Provider::<Ws>::connect(rpc.clone()).await?;
	let cid = client.get_chainid().await?.as_u64();
	let signer: LocalWallet = private_key
		.parse::<LocalWallet>()
		.unwrap()
		.with_chain_id(cid);
	let client = Arc::new(SignerMiddleware::new(client, signer.clone()));

	let share_sniper = Arc::new(sniper::new(
		Address::from_str(&sniper_address).unwrap(),
		client.clone(),
	));

	let _friendtech = Arc::new(shares::new(
		Address::from_str(&ft_address).unwrap(),
		client.clone(),
	));

	// Log the price curve
	let amount = args[1].parse::<u64>().unwrap();
	for supply in 1..40 {
		let price = math::get_price(U256::from(supply), U256::from(amount));
		let price = math::wei_to_eth(price);
		println!("Cost for {} shares @ {}: {}", amount, supply, price);
	}

	// KOSETTO CACHE THREAD (listens to new blocks)
	let blockclient = client.clone();
	let address_to_info = Arc::new(Mutex::new(HashMap::<Address, TwitterInfo>::new()));
	let address_to_info2 = address_to_info.clone();
	tokio::spawn(async move {
		let mut blockstream = blockclient.subscribe_blocks().await.unwrap();
		while let Some(block) = blockstream.next().await {
			// Get the full block
			let block = blockclient
				.get_block_with_txs(block.hash.unwrap())
				.await
				.unwrap();
			let address_to_info2 = address_to_info2.clone();
			if let Some(block) = block {
				// Iterate through the transactions
				for tx in block.transactions {
					let blockclient = blockclient.clone();
					let relay_txn_sig = Bytes::from_str("0xd764ad0b").unwrap();
					let address_to_info3 = address_to_info2.clone();
					tokio::spawn(async move {
						if tx.input.starts_with(&relay_txn_sig) {
							// We need to find the event
							let event = blockclient.get_transaction_receipt(tx.hash).await.unwrap();
							if event.is_none() {
								return;
							}
							let event = event.unwrap();
							// Find the event where topic0=0xb0444523268717a02698be47d0803aa7468c00acbed2f8bd93a0459cde61dd89
							let deposit_event = event.logs.iter().find(
                                |e| e.topics[0] == H256::from_str("0xb0444523268717a02698be47d0803aa7468c00acbed2f8bd93a0459cde61dd89").unwrap());
							if deposit_event.is_none() {
								return;
							}
							let deposit_event = deposit_event.unwrap();
							// The address will be the first slot of data
							let address = Address::from_slice(&deposit_event.data[12..32]);
							// Check if we have already cached this address (don't spam followers)
							if address_to_info3.lock().await.contains_key(&address) {
								return;
							}

							let info = reverse_search(address).await;
							if let Some(info) = info {
								println!(
									"Cached (deposit) {} with {} followers {:?}",
									info.twitter_username, info.followers, address
								);
								address_to_info3.lock().await.insert(address, info);
							}
						} else if tx.input.len() == 0 {
							// Check to and from addresses
							if tx.to.is_none() {
								return;
							}
							let to = tx.to.unwrap();
							let from = tx.from;
							let to_info = reverse_search(to).await;
							let from_info = reverse_search(from).await;
							// Check if we have already cached this address (don't spam followers)
							if address_to_info3.lock().await.contains_key(&to) {
								return;
							}
							if address_to_info3.lock().await.contains_key(&from) {
								return;
							}
							if let Some(to_info) = to_info {
								println!(
									"Cached (to) {} with {} followers {:?}",
									to_info.twitter_username, to_info.followers, to
								);
								address_to_info3.lock().await.insert(to, to_info);
							}
							if let Some(from_info) = from_info {
								println!(
									"Cached (from) {} with {} followers {:?}",
									from_info.twitter_username, from_info.followers, from
								);
								address_to_info3.lock().await.insert(from, from_info);
							}
						}
					});
				}
			}
		}
	});

    enum StreamID {
        StreamOne(H256),
        StreamTwo(H256),
        StreamThree(H256),
        StreamFour(H256),
        StreamFive(H256),
    }

	// PENDING TRANSACTION THREAD
	// sleep for 1 second
	let wsclient = Provider::<Ws>::connect(rpc.clone()).await?;
	tokio::time::sleep(Duration::from_secs(1)).await;
	let wsclient2 = Provider::<Ws>::connect(rpc.clone()).await?;
	tokio::time::sleep(Duration::from_secs(1)).await;
	let wsclient3 = Provider::<Ws>::connect(rpc.clone()).await?;
	tokio::time::sleep(Duration::from_secs(1)).await;
	let wsclient4 = Provider::<Ws>::connect(rpc.clone()).await?;
	tokio::time::sleep(Duration::from_secs(1)).await;
	let wsclient5 = Provider::<Ws>::connect(rpc.clone()).await?;
	let fasthttp = fasthttp::FastHttp::new("https://mainnet-sequencer.base.org/".to_string());
	tokio::spawn(async move {
		let mut current_nonce = client
			.get_transaction_count(signer.address(), None)
			.await
			.unwrap();
		let mut seen = FIFOCache::<H256>::new(10);
		let buy_sig = Bytes::from_str("0x6945b123").unwrap();

		let stream = wsclient.subscribe_pending_txs().await.unwrap();
		tokio::time::sleep(Duration::from_secs(1)).await;
		let stream2 = wsclient2.subscribe_pending_txs().await.unwrap();
		tokio::time::sleep(Duration::from_secs(1)).await;
		let stream3 = wsclient3.subscribe_pending_txs().await.unwrap();
		tokio::time::sleep(Duration::from_secs(1)).await;
		let stream4 = wsclient4.subscribe_pending_txs().await.unwrap();
		tokio::time::sleep(Duration::from_secs(1)).await;
		let stream5 = wsclient5.subscribe_pending_txs().await.unwrap();

		let stream_with_tag = stream.map(StreamID::StreamOne).boxed();
		let stream_with_tag2 = stream2.map(StreamID::StreamTwo).boxed();
		let stream_with_tag3 = stream3.map(StreamID::StreamThree).boxed();
		let stream_with_tag4 = stream4.map(StreamID::StreamFour).boxed();
		let stream_with_tag5 = stream5.map(StreamID::StreamFive).boxed();

		let mut combined_stream = stream::select_all(vec![
			stream_with_tag,
			stream_with_tag2,
			stream_with_tag3,
			stream_with_tag4,
			stream_with_tag5,
		]);

		while let Some(pending_tx) = combined_stream.next().await {
			let tx = match pending_tx {
				StreamID::StreamOne(tx) => wsclient.get_transaction(tx).await,
				StreamID::StreamTwo(tx) => wsclient2.get_transaction(tx).await,
				StreamID::StreamThree(tx) => wsclient3.get_transaction(tx).await,
				StreamID::StreamFour(tx) => wsclient4.get_transaction(tx).await,
				StreamID::StreamFive(tx) => wsclient5.get_transaction(tx).await,
			};
			// Check if we have already seen the transaction
			if let Ok(tx) = tx {
				if let Some(tx) = tx {
					// Check if we've seen it
					if seen.contains(&tx.hash) {
						continue;
					}
					seen.insert(tx.hash);
					println!("New pending tx: {:?}", tx.hash);
					if tx.to.is_none() {
						continue;
					}
					if let Some(tt) = tx.transaction_type {
						if tt != U64::from(2) {
							continue;
						}
					}
					if tx.value == U256::zero()
						&& tx.to.unwrap() == _friendtech.address()
                        && tx.input.len() == 68
						&& tx.input.starts_with(&buy_sig)
					{
						let max_fee = tx.max_fee_per_gas.unwrap();
						let prio_fee = tx.max_priority_fee_per_gas.unwrap();

						let mut address_to_info = address_to_info.lock().await;
						let info = match address_to_info.get(&tx.from) {
							Some(info) => {
								println!(
									"Cache hit for {} {}",
									info.twitter_username, info.followers
								);
								Some(info.clone()) // Clone the cached value
							},
							None => {
								// Live check
								if let Some(live_info) = reverse_search(tx.from).await {
									println!(
										"Live fetched {} with {} followers",
										live_info.twitter_username, live_info.followers
									);
									// Insert the fetched info into the mapping
									address_to_info.insert(tx.from, live_info);
									// Retrieve a reference to the newly inserted value
									address_to_info.get(&tx.from).cloned()
								} else {
									println!("Live fetch failed for {}", tx.from);
									None
								}
							},
						};
						drop(address_to_info);

						if info.is_none() {
							continue;
						}
						let info = info.unwrap();

						if info.supply_limit == 0 {
							continue;
						};

						let binding = share_sniper
							.snipe_many_shares(
								vec![tx.from],
								vec![U256::from(amount)],
								vec![U256::from(info.supply_limit)],
							)
							.calldata()
							.unwrap();

						let txn = Eip1559TransactionRequest {
							to: Some(NameOrAddress::Address(share_sniper.address())),
							from: Some(signer.address()),
							nonce: Some(current_nonce),
							gas: Some(U256::from(1_000_000)),
							value: None,
							data: Some(binding),
							chain_id: Some(U64::from(cid)),
							max_priority_fee_per_gas: Some(prio_fee),
							max_fee_per_gas: Some(max_fee),
							access_list: AccessList::default(),
						}
						.into();

						let sig = signer.sign_transaction(&txn).await.unwrap();
						let raw = txn.rlp_signed(&sig);
						let hash = fasthttp
							.send_request(format!("0x{}", hex::encode(raw)))
							.await;
						println!(
							"{} {} Sent snipe: https://basescan.org/tx/{:#?}#eventlog",
							info.twitter_username, info.followers, hash
						);

						// Write to a file
						let mut file = OpenOptions::new()
							.append(true)
							.create(true)
							.open("friendrekt.txt")
							.unwrap();

						writeln!(
							file,
							"sent {} {} {:#?} {:?}",
							info.twitter_username, info.followers, hash, tx.from
						)
						.unwrap();

						current_nonce += U256::one();
					}
				}
			}
		}
	});

	// OLD sniping method
    // Listen to new blocks
	/*
	let client = client.clone();
	tokio::spawn(async move {
		let filter = Filter::new()
		.address(Address::from_str(sniper_address).unwrap())
		.topic0(H256::from_str("0x2c76e7a47fd53e2854856ac3f0a5f3ee40d15cfaa82266357ea9779c486ab9c3").unwrap());
		let mut stream = client.subscribe_logs(&filter).await.unwrap();
		while let Some(log) = stream.next().await {
			// Trader (Address) is the first 32 bytes of data
			let address = Address::from_slice(&log.data[12..32]);
			// Subject (Address) is the second 32 bytes of data
			let subject = Address::from_slice(&log.data[44..64]);
			// Supply is the last 32 bytes of data
			let supply = U256::from_big_endian(&log.data[32*7..32*8]);
			if address == subject && supply == 1.into() {
				let client = client.clone();
				let signer = signer.clone();
				let share_sniper = share_sniper.clone();
				let httpprovider = httpprovider.clone();
				tokio::spawn(async move {
						// Try to snipe
						let twitter_response = reverse_search(subject).await.unwrap();

						println!("debug {} {} {} {}", username, followers, should_buy, limit);
						if should_buy {
							println!("{} {} Should buy {:?}", username, followers, subject);
							// Write to a file
							let mut file = OpenOptions::new()
								.append(true)
								.create(true)
								.open("friendrekt.txt")
								.unwrap();
							writeln!(file, "bought {} {} {}", username, followers, subject).unwrap();

							// TODO: get rid of nonce call
							let current_nonce = client.get_transaction_count(signer.address(), None).await.unwrap();
							let mod_amount = if limit == MAX { amount * 2 } else { amount };

							let binding = share_sniper.snipe_many_shares(
								vec![address], vec![U256::from(mod_amount)], vec![U256::from(limit)]
							).calldata().unwrap();

							let tx = TransactionRequest {
								from: Some(signer.address()),
								to: Some(NameOrAddress::Address(share_sniper.address())),
								gas: Some(U256::from(2_000_000)),
								gas_price: Some(U256::from(6_000_000_000_u64)),
								value: None,
								nonce: Some(current_nonce),
								data: Some(binding),
								chain_id: Some(U64::from(cid))
							}.into();
							let sig = signer.sign_transaction(&tx).await.unwrap();
							let raw = tx.rlp_signed(&sig);
							let x = httpprovider.send_raw_transaction(raw).await;
							let y = x.unwrap().await;
							let hash = tx.hash(&sig);
							println!("{} {} Sent snipe: https://basescan.org/tx/{:#?}#eventlog", username, followers, tx.hash(&sig));

							// Write to a file
							let mut file = OpenOptions::new()
								.append(true)
								.create(true)
								.open("friendrekt.txt")
								.unwrap();
							writeln!(file, "sent {} {} {:#?}#eventlog", username, followers, hash).unwrap();
					}
				});
			}
		}
	});*/

	// Run Forever
	loop {
		tokio::time::sleep(tokio::time::Duration::from_secs(100)).await;
	}
}
