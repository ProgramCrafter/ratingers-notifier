use std::sync::RwLock;
use std::thread::{sleep, spawn, JoinHandle};
use std::time::Duration;

#[repr(C)] pub struct Color(u8, u8, u8);
#[repr(C)] pub struct Message {
    text: *mut String,
    color: Color,
}


static NOTIFIER_THREAD: RwLock<Option<JoinHandle<()>>> = RwLock::new(None);

fn message_generator_thread(send: unsafe extern "C" fn(Message) -> ()) -> () {
    let ver = _version();
    unsafe {send(Message {
        text: Box::into_raw(Box::new(  format!("Notifier v{ver} started")  )),
        color: Color(40, 40, 255)
    });}
    
    for i in 0..65536 {
        unsafe {send(Message {
            text: Box::into_raw(Box::new(  format!("#{i}")  )),
            color: Color(40, 255, 40)
        });}
        sleep(Duration::from_millis(1));
    }
}

fn _version() -> u64 {2}

#[no_mangle] pub unsafe extern "C" fn version() -> u64 {_version()}

#[no_mangle] pub unsafe extern "C" fn start(callback: unsafe extern "C" fn(Message) -> ()) -> bool {
    let mut notifier = NOTIFIER_THREAD.write().unwrap();
    if notifier.is_none() {
        *notifier = Some(spawn(move || message_generator_thread(callback)));
        true
    } else {
        false
    }
}

#[no_mangle] pub unsafe extern "C" fn stop() -> () {
    let mut notifier = NOTIFIER_THREAD.write().unwrap();
    match notifier.take() {
        Some(thread) => {thread.join().expect("Could not stop notifier thread");},
        None         => {}
    }
}

#[no_mangle] pub unsafe extern "C" fn deallocate_message(msg: Message) -> () {
    let _ = Box::from_raw(msg.text);
}
