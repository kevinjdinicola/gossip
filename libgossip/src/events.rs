use std::future::Future;
use tokio::sync::broadcast;
use tokio::sync::broadcast::error::SendError;
use tokio::sync::broadcast::Sender;
use tokio::sync::broadcast::Receiver;

pub fn create_broadcast<E: Clone>() -> Sender<E> {
    let (tx, _rx) = broadcast::channel(16);
    tx
}

pub fn broadcast<E>(tx: &Sender<E>, event: E) -> Result<usize, SendError<E>> {
    if tx.receiver_count() > 0 {
        tx.send(event)
    } else {
        Ok(0)
    }
}


pub trait Subscribable<E> {
    fn subscribe(&self) -> Receiver<E>;
}

// pub trait Watch {
//     async fn watch<E, F, Fut, S>(subbable: &S, f: F)
//         where
//             Fut: Future<Output =anyhow::Result<()>> + Send + 'static,
//             F: (Fn(&S, E) -> Fut) + Send + 'static,
//             E: Clone,
//             S: Subscribable<E>
//     {
//         let mut tx = subbable.subscribe();
//         while let Some(e) = tx.recv().await {
//             let x = f(subbable, e).await.expect("oops");
//         }
//     }
// }