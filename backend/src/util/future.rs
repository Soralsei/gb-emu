use std::{
    future::Future,
    pin::Pin,
    task::{Context, Poll},
};

/// Drives `background` alongside `main`, dropping it when `main` returns.
/// `background` is polled first each round => fixed order keeps behaviour
/// deterministic.
pub struct Race<'a, A: Future, B: Future> {
    main: Pin<&'a mut A>,
    background: Pin<&'a mut B>,
}

impl<'a, A, B> Race<'a, A, B>
where
    A: Future,
    B: Future,
{
    pub fn new(main: Pin<&'a mut A>, background: Pin<&'a mut B>) -> Self {
        Self { main, background }
    }
}

impl<A: Future, B: Future> Future for Race<'_, A, B> {
    type Output = A::Output;
    fn poll(mut self: Pin<&mut Self>, cx: &mut Context) -> Poll<A::Output> {
        // Both fields are `Pin<&mut _>`, so `Race` is `Unpin`.
        let this = self.as_mut().get_mut();
        let _ = this.background.as_mut().poll(cx);
        this.main.as_mut().poll(cx)
    }
}
