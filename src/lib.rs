use std::thread::{spawn, JoinHandle};
use futures::executor::block_on;
use tokio::runtime::Builder;
use irc::client::prelude::*;
use futures::StreamExt;
use std::sync::RwLock;
use std::ptr::NonNull;


// Interoperability with the main program

#[repr(C)] pub struct Color(u8, u8, u8);
#[repr(C)] pub struct Comment {
    text: String,
    color: Color,
}


// Multi-threading helpers

type Thread = (JoinHandle<()>, Vec<Comment>, bool);
static NOTIFIER_THREAD: RwLock<Option<Thread>> = RwLock::new(None);
macro_rules! thread_stats {
    (const) => {NOTIFIER_THREAD.read().unwrap().as_ref().unwrap()};
    (mut)   => {NOTIFIER_THREAD.write().unwrap().as_mut().unwrap()};
}

#[inline] fn put_comment(comment: Comment) -> () {
    thread_stats!(mut).1.push(comment);
}


// Async function listening to IRC messages

async fn irc_listener(mut client: Client) -> () {
    let mut stream = client.stream().expect("Stream was already taken");

    while let Some(message) = stream.next().await.transpose().expect("message failure") {
        put_comment(Comment { color: Color(255, 192, 127), text: message.to_string() });
        if !thread_stats!(const).2 {break;}
    }
}

fn message_generator_thread() -> () {
    let ver = _version();
    put_comment(Comment { color: Color(127, 127, 255),
        text: format!("Notifier v{ver} started")});
    
    let futures_runtime = Builder::new_current_thread()
        .enable_all()
        .build().expect("cannot start async runtime");
    
    let irc_config = Config {
        nickname: Some("ProgramCrafter".to_owned()),
        server: Some("irc.esper.net".to_owned()),
        channels: vec!["#cc.ru".to_owned()],
        ..Config::default()
    };
    let client = futures_runtime.block_on(async {
        Client::from_config(irc_config).await
    }).expect("Connection failed");
    client.identify().expect("Connection failed");
    
    put_comment(Comment { color: Color(127, 127, 255),
        text: "Connected to #cc.ru as ProgramCrafter".to_owned()});
    
    futures_runtime.block_on(async {
        irc_listener(client).await
    });
}

fn _version() -> u64 {3}

#[no_mangle] pub unsafe extern "C" fn version() -> u64 {_version()}

#[no_mangle] pub unsafe extern "C" fn start() -> () {
    let mut notifier = NOTIFIER_THREAD.write().unwrap();
    if notifier.is_none() {
        *notifier = Some((spawn(message_generator_thread), vec![], true));
    }
}

#[no_mangle] pub unsafe extern "C" fn stop() -> () {
    let mut notifier = NOTIFIER_THREAD.write().unwrap();
    if notifier.is_none() {return;}
    
    notifier.as_mut().unwrap().2 = false;    // signal to stop working
    drop(notifier);
    
    let mut notifier = NOTIFIER_THREAD.write().unwrap();
    let (thread, _, _) = notifier.take().unwrap();
    thread.join().expect("Could not stop notifier thread");
}

#[no_mangle] pub unsafe extern "C" fn sync_process_queue(mut messages: NonNull<Vec<Comment>>) -> () {
    let mut notifier = NOTIFIER_THREAD.write().unwrap();
    let our_queue: &mut Vec<Comment> = &mut notifier.as_mut().unwrap().1;
    let app_queue = messages.as_mut();
    app_queue.append(our_queue);
}