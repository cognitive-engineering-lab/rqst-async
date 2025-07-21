use std::{
    fmt::Debug,
    future::Future,
    pin::pin,
    time::{Duration, Instant},
};

use tokio::{
    sync::{mpsc, oneshot},
    task::JoinHandle,
};

pub trait StatefulFunction: Send + 'static {
    type Input: Send;
    type Output: Send + Debug;
    fn call(&mut self, input: Self::Input) -> impl Future<Output = Self::Output> + Send;
}

type Payload<F> = (
    <F as StatefulFunction>::Input,
    oneshot::Sender<Option<<F as StatefulFunction>::Output>>,
);

pub struct StatefulThread<F: StatefulFunction> {
    _handle: JoinHandle<()>,
    input_tx: mpsc::Sender<Payload<F>>,
    cancel_tx: mpsc::Sender<()>,
}

impl<F: StatefulFunction> StatefulThread<F> {
    pub fn new(mut func: F) -> Self {
        let (input_tx, mut input_rx) = mpsc::channel::<Payload<F>>(1024);
        let (cancel_tx, mut cancel_rx) = mpsc::channel::<()>(1);
        let _handle = tokio::spawn(async move {
            while let Some((input, responder)) = input_rx.recv().await {
                let mut output_fut = pin!(func.call(input));
                let mut cancel_fut = pin!(cancel_rx.recv());
                let start = Instant::now();
                loop {
                    let log_fut = tokio::time::sleep(Duration::from_secs(1));
                    tokio::select! {
                        response = &mut output_fut => {
                            responder.send(Some(response)).unwrap();
                            break;
                        }
                        _ = &mut cancel_fut => {
                            responder.send(None).unwrap();
                            break;
                        }
                        _ = log_fut => {
                            println!("Waiting for {} seconds", start.elapsed().as_secs());
                        }
                    }
                }
            }
        });
        StatefulThread {
            _handle,
            input_tx,
            cancel_tx,
        }
    }

    pub async fn call(&self, input: F::Input) -> Option<F::Output> {
        let (tx, rx) = oneshot::channel();
        self.input_tx.send((input, tx)).await.unwrap();
        rx.await.unwrap()
    }

    pub async fn cancel(&self) {
        self.cancel_tx.send(()).await.unwrap();
    }
}
