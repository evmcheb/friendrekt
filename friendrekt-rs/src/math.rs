use ethers::types::U256;

pub fn wei_to_eth(wei: U256) -> f64 {
	let one_e18: U256 = U256::from(10).pow(18.into());
	let eth = wei / one_e18; // Integer division
	let remainder = wei % one_e18; // Remainder
							   // Convert to float, adding the remainder as a fraction
	eth.as_u64() as f64 + (remainder.as_u64() as f64 / one_e18.as_u64() as f64)
}

pub fn get_price(supply: U256, amount: U256) -> U256 {
	let zero = U256::zero();
	let one = U256::one();
	let two = U256::from(2);
	let six = U256::from(6);
	let one_ether = U256::from(10).pow(18.into());
	let sixteen_thousand = U256::from(16000);

	let sum1 = if supply == zero {
		zero
	} else {
		(supply - one) * supply * (two * (supply - one) + one) / six
	};

	let sum2 = if supply == zero && amount == one {
		zero
	} else {
		(supply - one + amount) * (supply + amount) * (two * (supply - one + amount) + one) / six
	};

	let summation = sum2 - sum1;
	summation * one_ether / sixteen_thousand
}