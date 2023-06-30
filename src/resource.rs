/// A resource is a type that can be moved inside or outside of a guard.
pub trait Resource {
    /// Clone the resource state. The function is unsafe, because
    /// only one instance of the resource state should exist at a time.
    /// If there are multiple instances of the resource state, we need to make
    /// sure that their states are synchronized. If one instance is dropped
    /// and cleans up the resource, the other instances should become invalid.
    unsafe fn clone_state(&self) -> Self;
    /// Set weather the resource should be cleaned up when it is dropped.
    /// If have multiple states of the same resource, we need to make sure
    /// that only one of them is set to clean up the resource.
    ///
    /// This method is unsafe, because it is possible to disable the cleanup
    /// of a resource, which brings the resource into an invalid state.
    unsafe fn set_cleanup_enabled(&mut self, cleanup_enabled: bool);
}

// Implement the Resource trait for tuples of resources.
macro_rules! tuple_impls {
    ( $( $name:ident )+ ) => {
        impl <$($name: Resource),+> Resource for ($($name,)+) {
            unsafe fn clone_state(&self) -> Self {
                #[allow(nonstandard_style)]
                let ($($name,)+) = self;
                ($($name.clone_state(),)+)
            }
            unsafe fn set_cleanup_enabled(&mut self, cleanup_enabled: bool) {
                #[allow(nonstandard_style)]
                let ($($name,)+) = self;
                $($name.set_cleanup_enabled(cleanup_enabled);)+
            }
        }
    };
}

tuple_impls! { A }
tuple_impls! { A B }
tuple_impls! { A B C }
tuple_impls! { A B C D }
tuple_impls! { A B C D E }
tuple_impls! { A B C D E F }
tuple_impls! { A B C D E F G }
tuple_impls! { A B C D E F G H }
tuple_impls! { A B C D E F G H I }
tuple_impls! { A B C D E F G H I J }
tuple_impls! { A B C D E F G H I J K }
tuple_impls! { A B C D E F G H I J K L }

// Implement the Resource trait for empty tuple.
impl Resource for () {
    unsafe fn clone_state(&self) -> Self {}
    unsafe fn set_cleanup_enabled(&mut self, _cleanup_enabled: bool) {}
}
