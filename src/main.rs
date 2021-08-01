// use async_std::task::{sleep, spawn};
// use std::time::Duration;

// async fn sleepus() {
//     for i in 1..=10 {
//         println!("Sleepus {}", i);
//         sleep(Duration::from_millis(1000)).await;
//     }
// }

// async fn interruptus() {
//     for i in 1..=5 {
//         println!("Interruptus {}", i);
//         sleep(Duration::from_millis(1000)).await;
//     }
// }

// #[async_std::main]
// async fn main() {
//     let sleepus = spawn(sleepus());
//     interruptus().await;

//     sleepus.await; // 如果不加这一行，会直接exit
// }


// // `foo()` returns a type that implements `Future<Output = u8>`.
// // `foo().await` will result in a value of type `u8`.
// async fn foo() -> u8 { 5 }

// fn bar() -> impl Future<Output = u8> {
//     // This `async` block results in a type that implements
//     // `Future<Output = u8>`.
//     async {
//         let x: u8 = foo().await;
//         x + 5
//     }
// }


use async_std::task::{sleep, spawn};
use futures::executor::block_on;
use std::sync::mpsc;
use std::sync::mpsc::{Sender, Receiver};
use std::thread;
use std::time;

struct Fork(String); 

struct Msg { // make it a package, .command: request or return, .which: the 2 forks that was taken
    cmd   : i32,          // 1 for retrieve, 2 for return 
    which : (i32, i32)    // the 2 forks that are involved.
}

impl std::fmt::Display for Msg {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "cmd:{}, ({}, {})", self.cmd, self.which.0, self.which.1)
    }
}

async fn request(tx: &Sender<Msg>) -> (Fork, Fork) { 
    println!("I'm hungry now");
    let s = String::from("first fork");
    let s1 = String::from("second fork");
    // start sending 

    (Fork(s), Fork(s1))
}

async fn eat(rx: &Receiver<Msg>, forks: (Fork, Fork)) { 
    println!("Yum Yum"); 

    // give back the forks
}

async fn think() { 
    println!("Thinking");    
}

async fn request_and_eat(tx: &Sender<Msg>, rx: &Receiver<Msg>) {
    // Wait until the song has been learned before singing it.
    // We use `.await` here rather than `block_on` to prevent blocking the
    // thread, which makes it possible to `dance` at the same time.
    let (leftFork, rightFork) = request(&tx).await; // yield control of the thread, tell the executor other futures can tun
    // .await turns a future into a value of specified type 

    // 
    eat(&rx, (leftFork, rightFork)).await;
}

async fn async_main(tx: Sender<Msg>, recv_chans: Vec<Receiver<Msg>>) { 
    // first philospher 
    let f1 = request_and_eat(&tx, &recv_chans[0]);
    let f2 = think();

    // second philospher 
    let f3 = request_and_eat(&tx, &recv_chans[1]);
    let f4 = think(); 

    // more...

    // `join!` is like `.await` but can wait for multiple futures concurrently.
    // If we're temporarily blocked in the `learn_and_sing` future, the `dance`
    // future will take over the current thread. If `dance` becomes blocked,
    // `learn_and_sing` can take back over. If both futures are blocked, then
    // `async_main` is blocked and will yield to the executor.
    futures::join!(f1, f2, f3, f4); // join/await actually start the execution of the futures
}

fn main() {
    let mut forks = [0; 5]; // used by the waiter
    println!("fork: {}", forks[0]);

    // a channel for asking forks and returning forks
    let (tx, rx): (Sender<Msg>, Receiver<Msg>) = mpsc::channel();
    
    // channel from the waiter to each philo to send the 2 forks
    let mut send_philo_chans: Vec<Sender<Msg>> = Vec::new();
    let mut recv_philo_chans: Vec<Receiver<Msg>> = Vec::new();

    for _ in 1..=5 {
        let (ty, ry): (Sender<Msg>, Receiver<Msg>) = mpsc::channel();
        send_philo_chans.push(ty);
        recv_philo_chans.push(ry);
    }

    // this is the waiter thread
    // uses send_philo_chans, and rx. 
    thread::spawn(move || {
        loop {
            let req = rx.recv().unwrap();
            println!("received a request {}", req);

            // look at forks and decide
            // mark as in use(give the forks out) or otherwise
            println!("fork: {}", forks[0]);

        }
    });

    // `block_on` blocks the current thread until the provided future has run to
    // completion. Other executors provide more complex behavior, like scheduling
    // multiple futures onto the same thread.
    block_on(async_main(tx, recv_philo_chans));
}














// backupppp-------------

// use async_std::task::{sleep, spawn};
// use futures::executor::block_on;

// use std::sync::mpsc;
// use std::thread;
// use std::time::Duration;

// struct Song(String);

// async fn learn_song() -> Song { 
//     println!("I'm hungry now");
//     sleep(Duration::from_millis(1000)).await;
//     println!("Finished learning");
//     let s = String::from("Waltzing matilda");
//     Song(s)
// }

// async fn sing_song(song: Song) { 
//     println!("Yum Yum"); 
// }

// async fn dance() { 
//     println!("Dancing");    
// }

// async fn learn_and_sing() {
//     // Wait until the song has been learned before singing it.
//     // We use `.await` here rather than `block_on` to prevent blocking the
//     // thread, which makes it possible to `dance` at the same time.
//     let song = learn_song().await;
//     sing_song(song).await;
// }

// async fn async_main() {
//     let f1 = learn_and_sing();
//     let f2 = dance();

//     // `join!` is like `.await` but can wait for multiple futures concurrently.
//     // If we're temporarily blocked in the `learn_and_sing` future, the `dance`
//     // future will take over the current thread. If `dance` becomes blocked,
//     // `learn_and_sing` can take back over. If both futures are blocked, then
//     // `async_main` is blocked and will yield to the executor.
//     futures::join!(f1, f2);
// }

// fn main() {
//     let mut forks = [0; 6]; // used by the waiter
//     println!("fork: {}", forks[0]);

//     // `block_on` blocks the current thread until the provided future has run to
//     // completion. Other executors provide more complex behavior, like scheduling
//     // multiple futures onto the same thread.
//     block_on(async_main());
// }

