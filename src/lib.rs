#![feature(scoped)]

extern crate threadpool;
extern crate time;

use threadpool::ThreadPool;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::rc::Rc;
use std::fmt::{Formatter, Error, Debug};
use std::collections::vec_deque::VecDeque;

#[cfg(test)]
mod test;

pub struct Subscription{
	pubsub: PubSub,
	channel_id: String,
	id: u64
}

impl Debug for Subscription{
	fn fmt(&self, fmt: &mut Formatter) -> Result<(), Error>{
		fmt.write_str(&format!("Sub(channel={})", self.channel_id))
	}
}

impl Subscription{
	pub fn cancel(self){}
	
	pub fn activate<F>(&self, func: F) -> Result<(),()>
	where F: FnMut(String) + 'static + Send{
		self.pubsub.activate(&self.channel_id, self.id, func)	
	}
	
	pub fn notify_others(&self, msg: &str){
		self.pubsub.notify(&self.channel_id, msg, Some(self.id));
	}
}
impl Drop for Subscription{
	fn drop(&mut self){
		println!("subscription dropping");
		self.pubsub.unregister(self);
		println!("subscription done dropping");
	}
}

struct SubData{
	running: RAIIBool,
	backlog: VecDeque<String>,
	func: Option<Arc<Mutex<Box<FnMut(String) + Send>>>>
}

struct InnerPubSub{
	channels: HashMap<String, HashMap<u64, SubData>>,
	//id will stay unique for hundreds of years, even at ~1 billion/sec
	next_id: u64,
	thread_pool: Rc<ThreadPool>
}
#[derive(Clone)]
pub struct PubSub{
	inner: Arc<Mutex<InnerPubSub>>
}
unsafe impl Send for PubSub{}
unsafe impl Sync for PubSub{}

#[derive(Clone)]
pub struct RAIIBool{
	value: Arc<Mutex<bool>>
}
impl RAIIBool{
	fn new(value: bool) -> RAIIBool{
		RAIIBool{
			value: Arc::new(Mutex::new(value))
		}
	}
	fn get(&self) -> bool{
		*self.value.lock().unwrap()
	}
	fn set(&self, value: bool){
		*self.value.lock().unwrap() = value;
	}
	fn new_guard(&self, value: bool) -> RAIIBoolGuard{
		RAIIBoolGuard::new(self.clone(), value)
	}
}


pub struct RAIIBoolGuard{
	data: RAIIBool,
	value: bool
}
impl RAIIBoolGuard{
	fn new(data: RAIIBool, value: bool) -> RAIIBoolGuard{
		RAIIBoolGuard{
			data: data,
			value: value
		}
	}
	fn done(self){}
}
impl Drop for RAIIBoolGuard{
	fn drop(&mut self){
		self.data.set(self.value);
	}
}
//pub struct RAIIBool{
//	value: Rc<Cell<bool>>
//}
//impl RAIIBool{
//	fn new<F>(value: bool) -> RAIIBool{
//		RAIIBool{
//			value: Cell::new(value)
//		}
//	}
//	fn get(setValue:bool, finalValue:bool) -> RAIIGuard{
//		
//	}
//}
impl PubSub{
	pub fn new(num_threads: usize) -> PubSub{
		PubSub{
			inner: Arc::new(Mutex::new(InnerPubSub{
				channels: HashMap::new(),
				next_id: 0,
				thread_pool: Rc::new(ThreadPool::new(num_threads))
			}))
		}
	}
	fn internal_subscribe<F>(&self, channel: &str, func: Option<F>) -> Subscription
	where F: FnMut(String) + 'static + Send{
		
		let mut data = self.inner.lock().unwrap();
		//let mut inner: &mut InnerPubSub = &mut *lock_guard;
		if !data.channels.contains_key(channel){
			data.channels.insert(channel.to_string(), HashMap::new());
		}
		let id = data.next_id;
		data.next_id += 1;
		
		println!("registering: channel = {}, id={}", channel, id);
		
		let sub_data = SubData{
			running: RAIIBool::new(false),
			backlog: VecDeque::new(),
			func: func.map(|f|Arc::new(Mutex::new(Box::new(f) as Box<_>)))
		};
		
		let subscriptions = data.channels.get_mut(channel).unwrap();
		subscriptions.insert(id, sub_data);
		Subscription{
			pubsub: self.clone(),
			channel_id: channel.to_string(),
			id: id
		}
	}
	pub fn subscribe<F>(&self, channel: &str, func: F) -> Subscription
	where F: FnMut(String) + 'static + Send{
		self.internal_subscribe(channel, Some(func))
	}
	pub fn lazy_subscribe(&self, channel: &str) -> Subscription{
		let mut func = Some(|_|{});
		func = None;
		self.internal_subscribe(channel, func)
	}
	fn activate<F>(&self, channel: &str, id: u64, func: F) -> Result<(),()>
	where F: FnMut(String) + 'static + Send{
		let mut inner = self.inner.lock().unwrap();
		let pool = inner.thread_pool.clone();
		if let Some(subs) = inner.channels.get_mut(channel){
			if let Some(sub_data) = subs.get_mut(&id){
				match sub_data.func{
					Some(_) => return Err(()),
					None => {
						sub_data.func = Some(Arc::new(Mutex::new(Box::new(func))));
						
						self.schedule_worker(sub_data, channel, id, &pool);
						return Ok(())
					}
				}
			}
		}
		Err(())
	}
	pub fn num_channels(&self) -> usize{
		let data = self.inner.lock().unwrap();
		data.channels.len()
	}
	fn unregister(&self, sub: &Subscription){
		println!("unregistering: channel = {}, id={}", sub.channel_id, sub.id);
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
	fn schedule_worker(&self, sub_data: &mut SubData, channel: &str, id: u64, pool: &Rc<ThreadPool>){
		//println!("schedule worker: channel={}, id={}", channel, id);
		if !sub_data.running.get(){
			let thread_running = sub_data.running.clone();
			

			//println!("checking for func");
			if let Some(func) = sub_data.func.clone(){
				//println!("got func");
				thread_running.set(true);
				let pubsub = self.clone();
				let channel = channel.to_string();
				let id = id.clone();
				pool.execute(move ||{	
					use std::ops::DerefMut;
					
					let finish_guard = thread_running.new_guard(false);
					let mut guard = func.lock().unwrap();
					let mut func = guard.deref_mut();
					let mut running = true;
					while running{
						//println!("working loop running");
						let mut notification_message = None;
						
						{
							let mut inner = pubsub.inner.lock().unwrap();
							if let Some(subs) = inner.channels.get_mut(&channel){
								if let Some(sub_data) = subs.get_mut(&id){
									if let Some(msg) = sub_data.backlog.pop_front(){
										notification_message = Some(msg);
									}
								}
							}
						}//unlock 'inner'
						
						if let Some(msg) = notification_message{
							func(msg);
						}else{
							running = false;
						}
					}
					finish_guard.done();
				});
			}
		}
	}
	pub fn notify(&self, channel: &str, msg: &str, exception: Option<u64>){
		let mut inner = self.inner.lock().unwrap();
		let pool = inner.thread_pool.clone();
		if let Some(subscriptions) = inner.channels.get_mut(channel){
			for (id,sub_data) in subscriptions{
				if Some(*id) != exception{
					sub_data.backlog.push_back(msg.to_string());
					self.schedule_worker(sub_data, channel, *id, &pool);
				}
			}
		}
	}
}
