use std::collections::HashMap;
use std::sync::{Arc,Mutex};
use std::sync::mpsc::{Receiver, Sender};
use std::sync::mpsc;
use std::ops::Deref;
use std::fmt::{Formatter, Error, Debug};

#[cfg(test)]
mod test;

pub struct Subscription{
	pubsub: PubSub,
	channel_id: String,
	receiver: Receiver<String>,
	id: u64
}

impl Debug for Subscription{
	fn fmt(&self, fmt: &mut Formatter) -> Result<(), Error>{
		fmt.write_str(&format!("Sub(channel={})", self.channel_id))
	}
}

impl Subscription{
	pub fn cancel(self){}
}
impl Drop for Subscription{
	fn drop(&mut self){
		self.pubsub.unregister(self);
	}
}
impl Deref for Subscription{
	type Target = Receiver<String>;
	fn deref<'a>(&'a self) -> &'a Receiver<String>{
		&self.receiver
	}
}

struct InnerPubSub{
	channels: HashMap<String, HashMap<u64, Sender<String>>>,

	//id will stay unique for hundreds of years, even at ~1 billion/sec
	next_id: u64
}
#[derive(Clone)]
pub struct PubSub{
	inner: Arc<Mutex<InnerPubSub>>
}
impl PubSub{
	pub fn new() -> PubSub{
		PubSub{
			inner: Arc::new(Mutex::new(InnerPubSub{
				channels: HashMap::new(),
				next_id: 0
			}))
		}
	}
	pub fn subscribe(&self, channel: &str) -> Subscription{
		let mut data = self.inner.lock().unwrap();
		//let mut inner: &mut InnerPubSub = &mut *lock_guard;
		if !data.channels.contains_key(channel){
			data.channels.insert(channel.to_string(), HashMap::new());
		}
		let id = data.next_id;
		data.next_id += 1;
		let (sender, receiver) = mpsc::channel();
		data.channels.get_mut(channel).unwrap().insert(id, sender);
		Subscription{
			pubsub: self.clone(),
			channel_id: channel.to_string(),
			receiver: receiver,
			id: id
		}
	}
	pub fn num_channels(&self) -> usize{
		let data = self.inner.lock().unwrap();
		data.channels.len()
	}
	fn unregister(&self, sub: &Subscription){
		let mut inner = self.inner.lock().unwrap();
		let mut remove_channel = false;
		{
			let sub_list = inner.channels.get_mut(&sub.channel_id).unwrap();
			sub_list.remove(&sub.id);
			if sub_list.len() == 0{
				remove_channel = true;
			}
		}
		if remove_channel{
			inner.channels.remove(&sub.channel_id);
		}
	}
	pub fn notify(&self, channel: &str, msg: &str){
		if let Some(subscriptions) = self.inner.lock().unwrap().channels.get_mut(channel){
			for (_,sender) in subscriptions{
				let _ = sender.send(msg.to_string());
			}
		}
	}
}
