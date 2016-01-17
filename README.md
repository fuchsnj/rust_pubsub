Local publish/subscribe
====
[![Build Status](https://travis-ci.org/fuchsnj/rust_pubsub.svg?branch=master)](https://travis-ci.org/fuchsnj/rust_pubsub)
[![crates.io](https://img.shields.io/crates/v/pubsub.svg)](https://crates.io/crates/pubsub)
[![MIT license](https://img.shields.io/crates/l/pubsub.svg)](./LICENSE)
## Documentation

http://fuchsnj.github.io/rust_pubsub

## Usage

Add the following to your `Cargo.toml`

```toml
[dependencies]
pubsub = "*"
```

and this to your crate root:

```rust
extern crate pubsub;
```

##Quick Start

```rust
	extern crate pubsub;
	use pubsub::PubSub;
	
	//create pubsub with 5 threads. These threads run notification functions.
	//messages are queued indefinitely until they can be run
	let pubsub = PubSub::new(5);
	
	//subscribe to a channel. The subscription will stay active as long
	//as sub1 stays in scope
	let sub1 = pubsub.subscribe("channel1", move |msg|{
		println!("channel 1 was notified with msg: {}", msg);
	});

	//notify all subscribers of "channel1" with message "data1"
	//This will always return immediately since messages are queued
	pubsub.notify("channel1", "data1");
	
	//notify all subscribers of "channel1" except sub1
	sub1.notify_others("data3");
	
	//If you want to start queueing messsages for a subscription, but
	//don't want to process them until after some initialization
	let sub2_activator = pubsub.lazy_subscribe("channel2");
	
	//do some initialization here...
	
	//notifications received here are queued
	pubsub.notify("channel2", "data4");
	
	//this creates a subscription and processes any messages in the backlog
	let sub2 = sub2_activator.activate(move |msg|{
		println!("channel 2 was notified with msg: {}", msg);
	});
	
	//subscriptions can be cancelled by dropping or cancelling
	drop(sub1);
	sub2.cancel();
```