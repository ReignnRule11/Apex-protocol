use apex_malachite_spike::{MALACHITE_RELEASE, MALACHITE_REVISION, RequestContext};

#[test]
fn release_and_revision_are_explicitly_pinned() {
    assert_eq!(MALACHITE_RELEASE, "0.8.2");
    assert_eq!(
        MALACHITE_REVISION,
        "a453b32155f4ba9f959724279b52f601bae92e09"
    );
}

#[test]
fn public_request_context_api_is_constructible() {
    let context = RequestContext::new(64);
    assert_eq!(context.channel_size, 64);
}
