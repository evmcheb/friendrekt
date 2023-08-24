use serde::Deserialize;

// Kosetto user
#[derive(Deserialize)]
pub struct User {
	pub address: String,
	pub twitterUsername: String,
	pub twitterUserId: String, // other fields can be added here if needed
}

// Kosetto response
#[derive(Deserialize)]
pub struct ApiResponse {
	pub users: Vec<User>,
}

// Hold some info about a Twitter user
#[derive(Clone)]
pub struct TwitterInfo {
	pub twitter_username: String,
	pub twitter_user_id: String,
	pub followers: u64,
	pub supply_limit: u64,
}