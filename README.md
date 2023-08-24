
# <img src="https://github.com/evmcheb/friendrekt/assets/50129617/e3ba3f2d-62fd-4c6c-a9db-df95f93b9794" width="48"> friend.tech mempool sniper bot <img src="https://github.com/evmcheb/friendrekt/assets/50129617/e3ba3f2d-62fd-4c6c-a9db-df95f93b9794" width="48">

mempool sniper bot for new friend.tech joiners. 
the story goes: 
- op-stack is supposed to be blind mempool. but base node default config had txpool+ws rpcs open (see the [patch notes](https://github.com/ethereum-optimism/op-geth/pull/118))
- friend.tech uses blast-api rpc
- blast-api used base node default config for their backend nodes

## important parts

1. friendrekt-rs
    - rust bot for finding new joiners, caching follow count and sniping
2. friendrekt-contracts
    - solidity contract for sniping w/ fail-safes
3. twt-follower-api
    - python http api for retrieving follower count

## algo
```
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
```

## devs
- [cheb](https://twitter.com/evmcheb)
- [rage](https://twitter.com/rage_pit)
