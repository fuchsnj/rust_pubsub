use PubSub;
use std::thread;
use std::sync::mpsc;

#[test]
fn it_works() {
	let pubsub = PubSub::new();
	{
		let sub1 = pubsub.subscribe("channel1");
		let sub2 = pubsub.subscribe("channel2");
		let sub3 = pubsub.subscribe("channel1");

		assert!(pubsub.num_channels() == 2);

		let (send, recv) = mpsc::channel();

		let pubsub2 = pubsub.clone();
		let guard = thread::scoped(move ||{
			sub1.recv().unwrap();
			sub1.cancel();
			assert!(pubsub2.num_channels() == 2);
			sub2.recv().unwrap();
			sub3.recv().unwrap();
			send.send("test").unwrap();
		});

		pubsub.notify("channel1", "test");
		pubsub.notify("channel2", "test");
		pubsub.notify("noChannel", "test");
		recv.recv().unwrap();
		guard.join();
	}
	assert!(pubsub.num_channels() == 0);
}