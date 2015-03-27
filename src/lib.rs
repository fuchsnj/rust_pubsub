use std::collections::{HashMap,HashSet};
use std::sync::{Arc,Mutex};
use std::hash::{Hash,Hasher};

#[test]
fn it_works() {
	let pubsub = PubSub::new();
	let sub = pubsub.subscribe("test",|msg|{
		println!("received msg:{}", msg);
	});
	pubsub.notify("test","message");
	println!("canceling");
	sub.cancel();
	println!("cancelled");
	panic!("fail tests");
}
struct Subscription{
	pubsub: PubSub,
	func_id: usize,
	channel: String
}
impl Subscription{
	pub fn cancel(self){}
}
impl Drop for Subscription{
	fn drop(&mut self){
		self.pubsub.unregister(self);
	}
}

struct InnerPubSub{
	channels: HashMap<String, HashMap<usize, Box<FnMut(&str) + Send + 'static>>>
}
#[derive(Clone)]
struct PubSub{
	inner: Arc<Mutex<InnerPubSub>>
}
impl PubSub{
	fn new() -> PubSub{
		PubSub{
			inner: Arc::new(Mutex::new(InnerPubSub{
				channels: HashMap::new()
			}))
		}
	}
	fn subscribe<F>(&self, channel: &str, func: F) -> Subscription
	where F: FnMut(&str) + Send + 'static{
		let mut lock_guard = self.inner.lock().unwrap();
		let mut inner: &mut InnerPubSub = &mut *lock_guard;
		if !inner.channels.contains_key(channel){
			println!("creating new channel");
			inner.channels.insert(channel.to_string(), HashMap::new());
		}
		let box_func = Box::new(func);
		let func_id = (&box_func as *const _) as usize;
		println!("registering id:{}", func_id);
		inner.channels.get_mut(channel).unwrap().insert(func_id, box_func);
		Subscription{
			pubsub: self.clone(),
			func_id: func_id,
			channel: channel.to_string()
		}
	}
	fn unregister(&self, sub: &Subscription){
		let mut inner = self.inner.lock().unwrap();
		let mut remove_channel = false;
		{
			let sub_list = inner.channels.get_mut(&sub.channel).unwrap();
			sub_list.remove(&sub.func_id);
			if sub_list.len() == 0{
				remove_channel = true;
			}
		}
		if remove_channel{
			println!("removing channel!");
			inner.channels.remove(&sub.channel);
		}
	}
	pub fn notify(&self, channel: &str, msg: &str){
		if let Some(subscriptions) = self.inner.lock().unwrap().channels.get_mut(channel){
			for (id,func) in subscriptions{
				func(msg);
			}
		}
	}
}
