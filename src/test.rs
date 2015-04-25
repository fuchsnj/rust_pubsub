

use PubSub;
use std::thread;
use std::sync::mpsc;
use std::thread::sleep_ms;
use time::now;

fn time_ms() -> String{
	let time = now();
	format!("now:{:?}", time.tm_nsec/1000000 + time.tm_sec*1000).to_string()
}

#[test]
fn it_works() {
	let pubsub = PubSub::new(5);
	{
		let sub1 = pubsub.subscribe("channel1", |msg|{
			println!("channel1:{}: running. time={}", msg, time_ms());
			sleep_ms(1000);
			println!("channel1:{}: finished", msg);
		});
		let sub2 = pubsub.subscribe("channel2", |msg|{
			println!("channel2:{}: running. time={}", msg, time_ms());
			sleep_ms(1000);
			println!("channel2:{}: finished", msg);
		});
		println!("notify data1:{}", time_ms());
		pubsub.notify("channel1", "data1");
		println!("notify data2:{}", time_ms());
		pubsub.notify("channel1", "data2");
		
		println!("notify data3:{}", time_ms());
		pubsub.notify("channel2", "data3");
		println!("notify data4:{}", time_ms());
		pubsub.notify("channel2", "data4");
		
		println!("Done notifying");
		sleep_ms(500);
		sub2.cancel();
		
		sleep_ms(2000);
		
//		let sub1 = pubsub.subscribe("channel1");
//		let sub2 = pubsub.subscribe("channel2");
//		let sub3 = pubsub.subscribe("channel1");
//
//		assert!(pubsub.num_channels() == 2);
//
//		let (send, recv) = mpsc::channel();
//
//		let pubsub2 = pubsub.clone();
//		let guard = thread::scoped(move ||{
//			sub1.recv().unwrap();
//			sub1.cancel();
//			assert!(pubsub2.num_channels() == 2);
//			sub2.recv().unwrap();
//			sub3.recv().unwrap();
//			send.send("test").unwrap();
//		});
//
//		pubsub.notify("channel1", "test");
//		pubsub.notify("channel2", "test");
//		pubsub.notify("noChannel", "test");
//		recv.recv().unwrap();
//		guard.join();
	}
	assert!(pubsub.num_channels() == 0);
	panic!();
}