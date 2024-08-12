use std::sync::Weak;
use async_trait::async_trait;
use tokio::sync::broadcast;
use tokio::sync::broadcast::error::SendError;
use tokio::sync::broadcast::{Receiver, Sender};

#[async_trait]
pub trait Starter {
    async fn start(&self) -> Result<(), anyhow::Error>;
}
pub fn start_with<T>(starter: T) -> T
where
    T: Starter + Clone + Send + 'static
{
    let o = starter.clone();
    tokio::spawn(async move {
        let r = o.start().await;
        if let Err(e) = r {
            eprintln!("{}", e);
        }
    });
    starter
}

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

pub trait WeakService<I, O> {
    fn get_weak(&self) -> Weak<I>;
    fn from_weak(weak: &Weak<I>) -> Option<O>;
}



#[async_trait]
pub trait Subscriber<E, I, O>: WeakService<I, Self> + Sized + Send
where
    E: Clone + Sync + Send + 'static,
    I: Send + Sync + 'static
{

     async fn event(&self, event: E) -> anyhow::Result<()>;

     fn listen_bc(&self, mut receiver: Receiver<E>) {
        let weak = self.get_weak();
        tokio::spawn(async move {
            let weak = weak;
            while let Ok(e) = receiver.recv().await {
                if let Some(listener) = Self::from_weak(&weak) {
                    listener.event(e).await.expect("boom");
                }
            }
        });
     }

    fn listen_mpsc(&self, mut receiver: tokio::sync::mpsc::Receiver<E>) {
        let weak = self.get_weak();
        tokio::spawn(async move {
            let weak = weak;
            while let Some(e) = receiver.recv().await {
                if let Some(listener) = Self::from_weak(&weak) {
                    listener.event(e).await.expect("boom");
                }
            }
        });

    }
}


