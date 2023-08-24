use reqwest::header::{HeaderMap, HeaderValue};
use serde::{Deserialize, Serialize};

pub struct FastHttp {
	pub rpc: String,
	pub client: reqwest::Client,
}

#[derive(Serialize, Debug)]
struct Params {
	jsonrpc: String,
	method: String,
	params: Vec<String>,
	id: i32,
}
#[derive(Deserialize, Debug)]
struct JSONError {
	code: i32,
	message: String,
}
#[derive(Deserialize, Debug)]
struct Ret {
	jsonrpc: String,
	id: i32,
	result: Option<String>,
	error: Option<JSONError>,
}

pub fn make_headers() -> HeaderMap {
	let mut headers = HeaderMap::new();
	headers.insert("Sec-Fetch-Site", HeaderValue::from_static("cross-site"));
	headers.insert("Accept", HeaderValue::from_static("*/*"));
	headers.insert(
		"Origin",
		HeaderValue::from_static("https://www.friend.tech"),
	);
	headers.insert("Sec-Fetch-Mode", HeaderValue::from_static("cors"));
	headers.insert(
		"Accept-Language",
		HeaderValue::from_static("en-US,en;q=0.9"),
	);
	headers.insert("Host", HeaderValue::from_static("prod-api.kosetto.com"));
	headers.insert(
		"Referer",
		HeaderValue::from_static("https://www.friend.tech/"),
	);
	headers.insert("Connection", HeaderValue::from_static("keep-alive"));
	headers
}

impl FastHttp {
	pub fn new(rpc: String) -> Self {
		FastHttp {
			client: reqwest::Client::new(),
			rpc,
		}
	}
	pub async fn send_request(&self, request: String) -> Option<String> {
		let request_params = Params {
			jsonrpc: "2.0".to_string(),
			method: "eth_sendRawTransaction".to_string(),
			params: vec![request],
			id: 1,
		};
		let res = self
			.client
			.post(&self.rpc)
			.json(&request_params)
			.send()
			.await;
		match res {
			Ok(res) => {
				let data: Result<Ret, _> = res.json().await;
				match data {
					Ok(data) => {
						if data.result.is_some() {
							Some(data.result.unwrap())
						} else if data.error.is_some() {
							println!("Error: {:?}", data.error.unwrap());
							return None;
						} else {
							println!("Unknown error");
							return None;
						}
					},
					Err(_) => None,
				}
			},
			Err(_) => None,
		}
	}
}
