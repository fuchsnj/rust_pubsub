use PubSub;
use std::thread::sleep;
use std::sync::{Arc, Mutex};
use std::time::Duration;

#[test]
fn basic_test() {
	let pubsub = PubSub::new(5);
	let count = Arc::new(Mutex::new(0));
	{
		let count1 = count.clone();
		let sub1 = pubsub.subscribe("channel1", move |_|{
			sleep(Duration::from_millis(1000));
			*count1.lock().unwrap() += 1;
		});
		let count2 = count.clone();
		let sub2 = pubsub.subscribe("channel2", move |_|{
			sleep(Duration::from_millis(1000));
			*count2.lock().unwrap() += 1;
		});
		pubsub.notify("channel1", "data1");
		pubsub.notify("channel1", "data2");
		
		pubsub.notify("channel2", "data3");
		pubsub.notify("channel2", "data4");
		
		sleep(Duration::from_millis(500));
		assert_eq!(*count.lock().unwrap(), 0);
		sub2.cancel();
		sleep(Duration::from_millis(1000));
		assert_eq!(*count.lock().unwrap(), 2);
		sleep(Duration::from_millis(1000));
		assert_eq!(*count.lock().unwrap(), 3);
		sub1.cancel();
	}
	assert!(pubsub.num_channels() == 0);
}

#[test]
fn lazy_subscribe(){
	let pubsub = PubSub::new(5);
	let count = Arc::new(Mutex::new(0));
	
	let sub1_activator = pubsub.lazy_subscribe("channel1");
	pubsub.notify("channel1", "data1");
	
	let count1 = count.clone();
	let sub1 = sub1_activator.activate(move |msg|{
		assert_eq!(msg, "data1");
		*count1.lock().unwrap() += 1;
	});
	sleep(Duration::from_millis(500));
	assert_eq!(*count.lock().unwrap(), 1);
	sub1.cancel();
}

#[test]
fn notify_exception() {
	let pubsub = PubSub::new(5);
	let count = Arc::new(Mutex::new(0));
	
	let count1 = count.clone();
	let sub1 = pubsub.subscribe("channel1", move |_|{
		*count1.lock().unwrap() -= 1;
	});
	let count2 = count.clone();
	let sub2 = pubsub.subscribe("channel1", move |_|{
		*count2.lock().unwrap() += 1;
	});
	
	sub1.notify_others("data1");
	
	sleep(Duration::from_millis(500));
	assert_eq!(*count.lock().unwrap(), 1);
	sub1.cancel();
	sub2.cancel();
}