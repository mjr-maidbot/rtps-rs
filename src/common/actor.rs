use futures::{
    future::Future,
    stream::{
        Stream,
        StreamExt,
    },
    task::{
        Context,
        Poll,
        Waker,
    },
}; 
use std::{
    cell::RefCell,
    collections::VecDeque,
    marker::PhantomData,
    pin::Pin,
    rc::Rc,
    sync::{
        Arc,
        RwLock,
    },
};
use super::{
    shared_state::{
        SharedState,
    },
};
use tokio::task::{
    spawn,
    spawn_local,
    JoinHandle,
};

#[derive(Clone, Copy, Debug)]
pub enum AsyncError {
    Abort,
    Shutdown,
}

pub type Result<T> = std::result::Result<T, AsyncError>;

//
// AsyncRequestResponse
//
// AsyncRequestResponse is a simple, cloneable channel that can be used to
// transfer objects between tasks or threads. Each created object is designed
// for a single use.
//
// Create a channel with:
//
//   let channel_rx = AsyncRequestResponse::new(request);
//
// Clone the channel for tx and send to another task or thread:
//
//   let channel_tx = channel_rx.clone();
//   // ... send channel_tx to another task or thread ...
//
// On the receive side of the channel, await a response with:
//
//   let response = channel_rx.await;
//
// On the transmit side of the channel, get a request and send a response with:
//
//   let request = channel_tx.take_request();
//   // ... compute response from request ...
//   channel_tx.wake_with_response(response);
//
pub struct AsyncRequestResponse<Request, Response, SharedRequest, SharedResponse, SharedWaker>
where
    SharedRequest: SharedState<Option<Request>>,
    SharedResponse: SharedState<Option<Result<Response>>>,
    SharedWaker: SharedState<Option<Waker>>,
{
    request: SharedRequest,
    response: SharedResponse,
    waker: SharedWaker,

    _req: PhantomData<Request>,
    _rsp: PhantomData<Response>,
}

//
// This custom Clone does not add the restriction of Request: Clone, whereas the
// #[derive(Clone)] implementation does.
//
impl<Request, Response, SharedRequest, SharedResponse, SharedWaker>
Clone for AsyncRequestResponse<Request, Response, SharedRequest, SharedResponse, SharedWaker>
where
    SharedRequest: SharedState<Option<Request>>,
    SharedResponse: SharedState<Option<Result<Response>>>,
    SharedWaker: SharedState<Option<Waker>>,
{
    fn clone(&self) -> Self {
        AsyncRequestResponse {
            request: self.request.clone(),
            response: self.response.clone(),
            waker: self.waker.clone(),

            _req: PhantomData,
            _rsp: PhantomData,
        }
    }
}

impl<Request, Response, SharedRequest, SharedResponse, SharedWaker>
AsyncRequestResponse<Request, Response, SharedRequest, SharedResponse, SharedWaker>
where
    SharedRequest: SharedState<Option<Request>>,
    SharedResponse: SharedState<Option<Result<Response>>>,
    SharedWaker: SharedState<Option<Waker>>,
{
    pub fn new(
        req: Request
    ) -> AsyncRequestResponse<Request, Response, SharedRequest, SharedResponse, SharedWaker>
    {
        AsyncRequestResponse {
            request: SharedRequest::new(Some(req)),
            response: SharedResponse::new(None),
            waker: SharedWaker::new(None),

            _req: PhantomData,
            _rsp: PhantomData,
        }
    }

    pub fn wake_with_response(self, response: Result<Response>) {
        self.response.call_mut(move |inner_response| *inner_response = Some(response));
        self.waker.call_mut(|inner_waker| {
            if let Some(waker) = inner_waker.take() {
                waker.wake();
            }
        });
    }

    #[inline]
    fn take_request(&self) -> Option<Request> {
        self.request.call_mut(move |inner_request| inner_request.take())
    }

    #[inline]
    fn take_response(&self) -> Option<Result<Response>> {
        self.response.call_mut(move |inner_response| inner_response.take())
    }

    #[inline]
    fn update_waker(&self, waker: Waker) {
        self.waker.call_mut(move |inner_waker| *inner_waker = Some(waker));
    }
}

impl<Request, Response, SharedRequest, SharedResponse, SharedWaker>
Future for AsyncRequestResponse<Request, Response, SharedRequest, SharedResponse, SharedWaker>
where
    SharedRequest: SharedState<Option<Request>>,
    SharedResponse: SharedState<Option<Result<Response>>>,
    SharedWaker: SharedState<Option<Waker>>,
{
    type Output = Result<Response>;

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        self.take_response()
            .map_or_else(
                // response isn't ready yet; store the current waker and wait
                || {
                    self.update_waker(cx.waker().clone());
                    Poll::Pending
                },

                // response is ready; yield it now
                |response| Poll::Ready(response)
            )
    }
}

pub trait Actor: Sized + Unpin + 'static {
    type Request;
    type Response;
    fn handle(&mut self, request: Self::Request) -> Self::Response;
}

#[derive(Clone)]
pub struct Address<
    Request, Response, SharedRequest, SharedResponse, SharedWaker, SharedPending, SharedFlag
>
where
    SharedRequest: SharedState<Option<Request>>,
    SharedResponse: SharedState<Option<Result<Response>>>,
    SharedWaker: SharedState<Option<Waker>>,
    SharedPending: SharedState<
        VecDeque<
            AsyncRequestResponse<Request, Response, SharedRequest, SharedResponse, SharedWaker>
        >
    >,
    SharedFlag: SharedState<bool>,
{
    pending: SharedPending,
    mailbox_waker: SharedWaker,
    cancel_flag: SharedFlag,

    _req: PhantomData<Request>,
    _rsp: PhantomData<Response>,
    _sreq: PhantomData<SharedRequest>,
    _srsp: PhantomData<SharedResponse>,
}

impl<Request, Response, SharedRequest, SharedResponse, SharedWaker, SharedPending, SharedFlag>
Address<Request, Response, SharedRequest, SharedResponse, SharedWaker, SharedPending, SharedFlag>
where
    SharedRequest: SharedState<Option<Request>>,
    SharedResponse: SharedState<Option<Result<Response>>>,
    SharedWaker: SharedState<Option<Waker>>,
    SharedPending: SharedState<
        VecDeque<
            AsyncRequestResponse<Request, Response, SharedRequest, SharedResponse, SharedWaker>
        >
    >,
    SharedFlag: SharedState<bool>,
{
    pub async fn handle(&self, request: Request) -> Result<Response> {
        // move the request into a new request/response and clone it
        let async_request = AsyncRequestResponse::new(request);
        let async_response = async_request.clone();

        // put the request into a pending queue and wake up the mailbox task
        self.pending.call_mut(|inner_queue| inner_queue.push_back(async_request));
        self.wake_mailbox();

        // await the response
        async_response.await
    }

    pub fn shutdown(&self) {
        // set the cancel flag and wake up the mailbox task
        self.cancel_flag.call_mut(|inner_flag| *inner_flag = true);
        self.wake_mailbox();
    }

    fn new(
        pending: SharedPending,
        mailbox_waker: SharedWaker,
        cancel_flag: SharedFlag,
    ) -> Address<
        Request, Response, SharedRequest, SharedResponse, SharedWaker, SharedPending, SharedFlag
    >
    {
        Address {
            pending,
            mailbox_waker,
            cancel_flag,

            _req: PhantomData,
            _rsp: PhantomData,
            _sreq: PhantomData,
            _srsp: PhantomData,
        }
    }

    fn wake_mailbox(&self) {
        self.mailbox_waker.call_mut(|inner_waker| {
            if let Some(waker) = inner_waker.take() {
                waker.wake();
            }
        });
    }
}

pub type SingleThreadedAddress<Request, Response> = Address::<
    Request,
    Response,
    Rc<RefCell<Option<Request>>>,
    Rc<RefCell<Option<Result<Response>>>>,
    Rc<RefCell<Option<Waker>>>,
    Rc<RefCell<
        VecDeque<
            AsyncRequestResponse<
                Request,
                Response,
                Rc<RefCell<Option<Request>>>,
                Rc<RefCell<Option<Result<Response>>>>,
                Rc<RefCell<Option<Waker>>>>,
            >
        >
    >,
    Rc<RefCell<bool>>,
>;
pub type MultiThreadedAddress<Request, Response> = Address::<
    Request,
    Response,
    Arc<RwLock<Option<Request>>>,
    Arc<RwLock<Option<Result<Response>>>>,
    Arc<RwLock<Option<Waker>>>,
    Arc<RwLock<
        VecDeque<
            AsyncRequestResponse<
                Request,
                Response,
                Arc<RwLock<Option<Request>>>,
                Arc<RwLock<Option<Result<Response>>>>,
                Arc<RwLock<Option<Waker>>>>,
            >
        >
    >,
    Rc<RefCell<bool>>,
>;

pub struct Mailbox<
    A, Request, Response, SharedRequest, SharedResponse, SharedWaker, SharedPending, SharedFlag
>
where
    A: Actor<Request = Request, Response = Response>,
    SharedRequest: SharedState<Option<Request>>,
    SharedResponse: SharedState<Option<Result<Response>>>,
    SharedWaker: SharedState<Option<Waker>>,
    SharedPending: SharedState<
        VecDeque<
            AsyncRequestResponse<Request, Response, SharedRequest, SharedResponse, SharedWaker>
        >
    >,
    SharedFlag: SharedState<bool>,
{
    //
    // After the Mailbox stream is shut down, the original actor will be yielded
    // back to the caller via a JoinHandle. This requires that actor be moved
    // out of the Mailbox. However, since Mailbox implements Drop, we cannot
    // simply pull apart the Mailbox with something like:
    //
    //   let Mailbox { actor, .. } = mailbox;
    //
    // because this will produce a compiler error:
    //
    //   error[E0509]: cannot move out of type `Mailbox...`, which implements
    //   the `Drop` trait
    //
    // Wrapping the original object in an Option allows it to be moved out with
    // a take() call.
    //
    actor: Option<A>,

    pending: SharedPending,
    mailbox_waker: SharedWaker,
    cancel_flag: SharedFlag,

    _sreq: PhantomData<SharedRequest>,
    _srsp: PhantomData<SharedResponse>,
}

impl<A, Request, Response, SharedRequest, SharedResponse, SharedWaker, SharedPending, SharedFlag>
Mailbox<A, Request, Response, SharedRequest, SharedResponse, SharedWaker, SharedPending, SharedFlag>
where
    A: Actor<Request = Request, Response = Response>,
    SharedRequest: SharedState<Option<Request>>,
    SharedResponse: SharedState<Option<Result<Response>>>,
    SharedWaker: SharedState<Option<Waker>>,
    SharedPending: SharedState<
        VecDeque<
            AsyncRequestResponse<Request, Response, SharedRequest, SharedResponse, SharedWaker>
        >
    >,
    SharedFlag: SharedState<bool>,
{
    // iterate through all pending AsyncRequestResponse items and wake each one
    // with an error
    fn stop_all_pending_with_error(&self, error: AsyncError) {
        self.pending.call_mut(|arrs| {
            for arr in arrs.drain(..) {
                arr.wake_with_response(Err(error));
            }
        });
    }
}

impl<A, Request, Response, SharedRequest, SharedResponse, SharedWaker, SharedPending, SharedFlag>
Stream for Mailbox<
    A, Request, Response, SharedRequest, SharedResponse, SharedWaker, SharedPending, SharedFlag
>
where
    A: Actor<Request = Request, Response = Response>,
    SharedRequest: SharedState<Option<Request>>,
    SharedResponse: SharedState<Option<Result<Response>>>,
    SharedWaker: SharedState<Option<Waker>>,
    SharedPending: SharedState<
        VecDeque<
            AsyncRequestResponse<Request, Response, SharedRequest, SharedResponse, SharedWaker>
        >
    >,
    SharedFlag: SharedState<bool>,
{
    type Item = AsyncRequestResponse<Request, Response, SharedRequest, SharedResponse, SharedWaker>;

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        // TBD if cancel flag is set:
        //   iterate through all pending ARRs:
        //     call wake_with_response with Err(Shutdown)
        //   end the stream with Poll::Ready(None)

        // TBD if there is an element in the pending queue, pop it and yield it;
        // if the queue is empty, save waker in mailbox_waker and return pending
        Poll::Pending
    }
}

impl<A, Request, Response, SharedRequest, SharedResponse, SharedWaker, SharedPending, SharedFlag>
Drop for Mailbox<
    A, Request, Response, SharedRequest, SharedResponse, SharedWaker, SharedPending, SharedFlag
>
where
    A: Actor<Request = Request, Response = Response>,
    SharedRequest: SharedState<Option<Request>>,
    SharedResponse: SharedState<Option<Result<Response>>>,
    SharedWaker: SharedState<Option<Waker>>,
    SharedPending: SharedState<
        VecDeque<
            AsyncRequestResponse<Request, Response, SharedRequest, SharedResponse, SharedWaker>
        >
    >,
    SharedFlag: SharedState<bool>,
{
    fn drop(&mut self) {
        self.stop_all_pending_with_error(AsyncError::Abort);
    }
}

pub trait SingleThreadedActor: Actor {
    fn start_mailbox_loop(
        self
    ) -> (SingleThreadedAddress<Self::Request, Self::Response>, JoinHandle<Self>)
    {
        let actor = Some(self);
        let pending = Rc::new(RefCell::new(VecDeque::new()));
        let mailbox_waker = Rc::new(RefCell::new(None));
        let cancel_flag = Rc::new(RefCell::new(false));
        let address = Address::new(
            pending.clone(),
            mailbox_waker.clone(),
            cancel_flag.clone()
        );
        let mut mailbox = Mailbox {
            actor,
            pending,
            mailbox_waker,
            cancel_flag,

            _sreq: PhantomData,
            _srsp: PhantomData,
        };

        let handle = spawn_local(async move {
            while let Some(async_rr) = mailbox.next().await {
                // TBD add comments here
                let request = async_rr.take_request().unwrap();
                let response = mailbox.actor.as_mut().unwrap().handle(request);
                async_rr.wake_with_response(Ok(response));
            }

            // yield back the original Self object
            mailbox.actor.take().unwrap()
        });

        (address, handle)
    }
}
