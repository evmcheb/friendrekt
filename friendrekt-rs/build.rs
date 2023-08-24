//use ethers_solc::{Project, ProjectPathsConfig};
use ethers_contract_abigen::MultiAbigen;

fn main() {
	// configure the project with all its paths, solc, cache etc.
	println!("Building ethers contract");
	let gen = MultiAbigen::from_json_files("./abi").unwrap();
	let bindings = gen.build().unwrap();
	bindings.write_to_module("src/bindings", false).unwrap();
	println!("cargo:rerun-if-changed=abi/");
}
