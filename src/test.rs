

use PubSub;
use std::thread;
use std::sync::mpsc;
use std::thread::sleep_ms;
use time::now;
use std::sync::{Arc, Mutex};

fn time_ms() -> String{
	let time = now();
	format!("now:{:?}", time.tm_nsec/1000000 + time.tm_sec*1000).to_string()
}



#[test]
fn basic_test() {
	let pubsub = PubSub::new(5);
	let count = Arc::new(Mutex::new(0));
	{
		let count1 = count.clone();
		let sub1 = pubsub.subscribe("channel1", move |msg|{
			sleep_ms(1000);
			*count1.lock().unwrap() += 1;
		});
		let count2 = count.clone();
		let sub2 = pubsub.subscribe("channel2", move |msg|{
			sleep_ms(1000);
			*count2.lock().unwrap() += 1;
		});
		pubsub.notify("channel1", "data1", None);
		pubsub.notify("channel1", "data2", None);
		
		pubsub.notify("channel2", "data3", None);
		pubsub.notify("channel2", "data4", None);
		
		sleep_ms(500);
		assert_eq!(*count.lock().unwrap(), 0);
		sub2.cancel();
		sleep_ms(1000);
		assert_eq!(*count.lock().unwrap(), 2);
		sleep_ms(1000);
		assert_eq!(*count.lock().unwrap(), 3);

	}
	assert!(pubsub.num_channels() == 0);
}

#[test]
fn lazy_subscribe(){
	let pubsub = PubSub::new(5);
	let count = Arc::new(Mutex::new(0));
	
	let sub1 = pubsub.lazy_subscribe("channel1");
	pubsub.notify("channel1", "data1", None);
	
	let count1 = count.clone();
	sub1.activate(move |msg|{
		assert!(msg == "data1");
		*count1.lock().unwrap() += 1;
	});
	sleep_ms(500);
	assert_eq!(*count.lock().unwrap(), 1);
}

#[test]
fn notify_exception() {
	let pubsub = PubSub::new(5);
	let count = Arc::new(Mutex::new(0));
	
	let count1 = count.clone();
	let sub1 = pubsub.subscribe("channel1", move |msg|{
		*count1.lock().unwrap() -= 1;
	});
	let count2 = count.clone();
	let sub2 = pubsub.subscribe("channel1", move |msg|{
		*count2.lock().unwrap() += 1;
	});
	
	sub1.notify_others("data1");
	
	sleep_ms(500);
	assert_eq!(*count.lock().unwrap(), 1);
}







