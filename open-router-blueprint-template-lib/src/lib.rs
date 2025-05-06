use blueprint_sdk::extract::Context;
use blueprint_sdk::tangle::extract::{Optional, TangleArg, TangleResult};
use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};

// The job ID (to be generated?)
pub const SAY_HELLO_JOB_ID: u32 = 0;

// A context struct
//
// The context of a blueprint is set in the `Router`, and passed down to the job functions. See
// `foo-bin/src/main.rs`.
//
// This simply counts the number of times the `say_hello` job has been called, but it could be any
// global state needed by the blueprint.
#[derive(Clone)]
pub struct MyContext {
    total_greetings: Arc<AtomicUsize>,
}

impl MyContext {
    pub fn new() -> Self {
        Self {
            total_greetings: Arc::new(AtomicUsize::new(0)),
        }
    }
}

// The job function
//
// The arguments are made up of "extractors", which take a portion of the `JobCall` to convert into the
// target type.
//
// The context is passed in as a parameter, and can be used to store any shared state between job calls.
pub async fn say_hello(
    Context(ctx): Context<MyContext>,
    TangleArg(Optional(who)): TangleArg<Optional<String>>,
) -> TangleResult<String> {
    let greeting = match who {
        Some(who) => format!("Hello, {who}!"),
        None => "Hello World!".to_string(),
    };

    ctx.total_greetings.fetch_add(1, Ordering::SeqCst);

    // The result is then converted into a `JobResult` to be sent back to the caller.
    TangleResult(greeting)
}

#[cfg(test)]
mod tests {
    use super::*;
    use blueprint_sdk::{JobResult, IntoJobResult, tangle_subxt};
    use tangle_subxt::tangle_testnet_runtime::api::runtime_types::tangle_primitives::services::field::Field;
    use tangle_subxt::subxt_core::utils::AccountId32;
    use blueprint_sdk::tangle::serde::new_bounded_string;
    use tangle_subxt::parity_scale_codec::Decode;

    #[tokio::test]
    async fn it_works() {
        let context = MyContext::new();
        let JobResult::Ok {
            body: result_raw, ..
        } = say_hello(Context(context.clone()), TangleArg(None.into()))
            .await
            .into_job_result()
            .unwrap()
        else {
            panic!("Job call failed");
        };

        let result = Vec::<Field<AccountId32>>::decode(&mut (&*result_raw)).expect("Bad result");
        assert_eq!(
            result,
            vec![Field::String(new_bounded_string("Hello World!"))]
        );

        let JobResult::Ok {
            body: result2_raw, ..
        } = say_hello(
            Context(context.clone()),
            TangleArg(Some("Alice".to_string()).into()),
        )
            .await
            .into_job_result()
            .unwrap()
        else {
            panic!("Job call failed");
        };
        let result2 = Vec::<Field<AccountId32>>::decode(&mut (&*result2_raw)).expect("Bad result");
        assert_eq!(
            result2,
            vec![Field::String(new_bounded_string("Hello, Alice!"))]
        );

        assert_eq!(context.total_greetings.load(Ordering::SeqCst), 2);
    }
}
