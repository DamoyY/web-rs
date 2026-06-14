use crate::{error::AppError, net::SsrfGuard};
use reqwest::dns::{Addrs, Name, Resolve, Resolving};
#[derive(Debug)]
pub(crate) struct GuardedResolver {
    guard: SsrfGuard,
}
impl GuardedResolver {
    #[inline]
    #[must_use]
    pub(crate) const fn new(guard: SsrfGuard) -> Self {
        Self { guard }
    }
}
impl Resolve for GuardedResolver {
    fn resolve(&self, name: Name) -> Resolving {
        let guard = self.guard.clone();
        let domain = name.as_str().to_owned();
        Box::pin(async move {
            let addresses = guard
                .resolve_allowed_domain(&domain, 0)
                .await
                .map_err(boxed_error)?;
            let addrs: Addrs = Box::new(addresses.into_iter());
            Ok(addrs)
        })
    }
}
fn boxed_error(error: AppError) -> Box<dyn core::error::Error + Send + Sync> {
    Box::new(error)
}
