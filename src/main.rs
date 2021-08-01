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

// #[tokio::main]
// async fn main() {
//     let s = tokio::spawn(async move {
//         sleepus().await;
//     });
//     interruptus().await;
//     s.await;
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
use futures::executor::block_on; // replaced with tokio's 
use std::thread;
use std::time;

type Fork = i32;

// msg sent by diners 
struct Msg { // make it a package, .command: request or return, .which: the 2 forks that was taken
    cmd     : i32,          // 1 for retrieve, 2 for return 
    who     : i32,          // which philosopher, 0 if sent by waiter
    re_chan : Option<tokio::sync::mpsc::Sender<Msg>>,   // where i'm seated
    forks   : (Option<Fork>, Option<Fork>)      // optional 
}

impl std::fmt::Display for Msg {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "cmd:{}, sender:{}, forks:({}, {})", self.cmd, self.who, self.forks.0.unwrap_or(-1), self.forks.1.unwrap_or(-1))
    }
}

async fn request(tx: tokio::sync::mpsc::Sender<Msg>, id: i32) -> (Fork, Fork) { 
    println!("{}: I'm hungry now", id);

    // create a new channel so that waiter knows how to give the forks to me
    let (send_side, mut recv_side): (tokio::sync::mpsc::Sender<Msg>, tokio::sync::mpsc::Receiver<Msg>) = tokio::sync::mpsc::channel(10);
    // for this new chan, i'm the receiver 
    let m: Msg = Msg{ 
        cmd     : 1, 
        who     : id,
        re_chan : Some(send_side), 
        forks   : (None, None)
    };
    // lord waiter, can i have two forks pls?
    tx.send(m).await;

    let result: Option<Msg> = recv_side.recv().await;

    // this is SUPER ugly, fix it up later pls
    let mut ready_forks = (-2, -2);
    match result {
        // implicit handling here results in either the inner wrapped value or panic
        Some(x) => ready_forks = (x.forks.0.unwrap_or(-1), x.forks.1.unwrap_or(-1)),  // we going 6-feet deep into hell if unwraps err, panic!
        None    => println!("wtf how is this possible"),
    };

    println!("{}: got the forks i need", id);

    ready_forks
}

async fn eat(forks: (Fork, Fork), id: i32) { 
    println!("{}: Yum Yum, used forks {} and {}", id, forks.0, forks.1); 

    // give back the forks
    
}

async fn think(id: i32, deep_think: tokio::sync::mpsc::Receiver<bool>) { 
    for i in 1..=50 {
        println!("{}: thinking", id);  // a little caveat, might handle later TODO, async callback/non-block msg passing, maybe poll_recv
        tokio::time::sleep(tokio::time::Duration::from_millis(50)).await;  
    }
}

async fn request_and_eat(tx: tokio::sync::mpsc::Sender<Msg>, id: i32, reminder: tokio::sync::mpsc::Sender<bool>) {
    // Wait until the song has been learned before singing it.
    // We use `.await` here rather than `block_on` to prevent blocking the
    // thread, which makes it possible to `dance` at the same time.
    let (left_fork, right_fork) = request(tx, id).await; // yield control of the thread, tell the executor other futures can tun
    // .await turns a future into a value of specified type 

    eat((left_fork, right_fork), id).await;
}

async fn async_main(tx: tokio::sync::mpsc::Sender<Msg>) { 
    loop {
        // first philospher 
        let (reminder, mut deep_think): (tokio::sync::mpsc::Sender<bool>, tokio::sync::mpsc::Receiver<bool>) = tokio::sync::mpsc::channel(1);
        let f1 = request_and_eat(tx.clone(), 1, reminder); // this is where the actual 'async' magic takes place
        let f2 = think(1, deep_think);

        let (reminder, mut deep_think): (tokio::sync::mpsc::Sender<bool>, tokio::sync::mpsc::Receiver<bool>) = tokio::sync::mpsc::channel(1);
        let f3 = request_and_eat(tx.clone(), 2, reminder); // this is where the actual 'async' magic takes place
        let f4 = think(2, deep_think);
        
        // more...

        // `join!` is like `.await` but can wait for multiple futures concurrently.
        // If we're temporarily blocked in the `learn_and_sing` future, the `dance`
        // future will take over the current thread. If `dance` becomes blocked,
        // `learn_and_sing` can take back over. If both futures are blocked, then
        // `async_main` is blocked and will yield to the executor.
        futures::join!(f1, f2, f3, f4); // join/await actually start the execution of the futures
        // another caveat...

        tokio::time::sleep(tokio::time::Duration::from_millis(1000)).await;
    }
}

fn main() {
    let rt = tokio::runtime::Runtime::new().unwrap();

    let mut forks = [0; 5]; // used by the waiter

    // a channel for asking forks and returning forks
    // sync <-> async / async <-> async, bounded channel 
    let (tx, mut rx): (tokio::sync::mpsc::Sender<Msg>, tokio::sync::mpsc::Receiver<Msg>) = tokio::sync::mpsc::channel(10);

    // this is the waiter thread
    // uses send_philo_chans, and rx. 
    thread::spawn(move || {
        loop {
            let m = rx.blocking_recv().unwrap(); // blocking synchronously, sent from an async task

            println!("Waiter: {} received a request", m.who);

            if m.cmd == 1 {
                // look at forks and decide
                // standard dining philo here
                let r: Msg = Msg{ 
                    cmd     : 1, 
                    who     : 0,
                    re_chan : None, 
                    forks   : (Some(3), Some(4))
                };
                m.re_chan.unwrap().blocking_send(r);
            } 
            else if m.cmd == 2 { // returns
                forks[m.forks.0.unwrap() as usize] = 0;
                forks[m.forks.1.unwrap() as usize] = 0;
            }
        }
    });

    // `block_on` blocks the current thread until the provided future has run to
    // completion. Other executors provide more complex behavior, like scheduling
    // multiple futures onto the same thread.

    rt.block_on(async {
        async_main(tx).await;
    });
    //Spawn a future onto the runtime
    
    // let rt_handle1 = rt.handle();
    // let rt_handle2 = rt.handle();
    // let rt_handle3 = rt.handle();
    // let rt_handle4 = rt.handle();
    // let rt_handle5 = rt.handle();

    // thread::spawn(move || {
    //     println!("started philo #1");
    //     rt.handle().clone().block_on(async {
    //         async_main(tx).await;
    //     });
    // });
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

