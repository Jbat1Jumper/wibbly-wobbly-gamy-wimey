use std::sync::mpsc::{channel, Receiver, Sender};

struct PleaseSubscribeMe<T>(Sender<T>);
pub type Publication<T> = Sender<PleaseSubscribeMe<T>>;
pub type Subscription<T> = Receiver<T>;

pub struct Publisher<T> {
    subscription_requests: Receiver<PleaseSubscribeMe<T>>,
    subscriptors: Vec<Sender<T>>,
}

pub fn medium<T>() -> (Publication<T>, Publisher<T>) {
    let (publication_address, inbox) = channel();
    let publisher = Publisher {
        subscription_requests: inbox,
        subscriptors: vec![],
    };
    (publication_address, publisher)
}

impl<T> Publisher<T>
where
    T: Clone + Send,
{
    pub fn publish(&mut self, thing: &T) {
        for PleaseSubscribeMe(address) in self.subscription_requests.try_iter() {
            self.subscriptors.push(address);
        }
        self.subscriptors
            .retain(|subscriptor| subscriptor.send(thing.clone()).is_ok());
    }
}

pub trait PublicationTrait<T> {
    fn subscribe(&self) -> Subscription<T>;
}

impl<T> PublicationTrait<T> for Publication<T> {
    fn subscribe(&self) -> Subscription<T> {
        let (address, inbox) = channel();
        self.send(PleaseSubscribeMe(address));
        inbox
    }
}

//-------------------

pub struct Republisher<T> {
    to_publish: Receiver<T>,
    publisher: Publisher<T>,
}

impl<T> Republisher<T> {
    fn pump(&mut self) {}
}

pub fn open_medium<T>() -> (Publication<T>, Sender<T>, Republisher<T>) {
    let (publication, publisher) = medium();
    let (address, inbox) = channel();

    (
        publication,
        address,
        Republisher {
            to_publish: inbox,
            publisher,
        },
    )
}
