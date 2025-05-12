use blueprint_sdk::Job;
use blueprint_sdk::Router;
use blueprint_sdk::contexts::tangle::TangleClientContext;
use blueprint_sdk::crypto::sp_core::SpSr25519;
use blueprint_sdk::crypto::tangle_pair_signer::TanglePairSigner;
use blueprint_sdk::keystore::backends::Backend;
use blueprint_sdk::runner::BlueprintRunner;
use blueprint_sdk::runner::config::BlueprintEnvironment;
use blueprint_sdk::runner::tangle::config::TangleConfig;
use blueprint_sdk::tangle::consumer::TangleConsumer;
use blueprint_sdk::tangle::filters::MatchesServiceId;
use blueprint_sdk::tangle::layers::TangleLayer;
use blueprint_sdk::tangle::producer::TangleProducer;
use open_router_blueprint_template_lib::{ 
    OpenRouterContext, PROCESS_LLM_REQUEST_JOB_ID, REPORT_METRICS_JOB_ID,
    process_llm_request, report_metrics,
};
use std::time::Duration;
use tower::filter::FilterLayer;
use tracing::level_filters::LevelFilter;
use tracing::{error, info};

#[tokio::main]
async fn main() -> Result<(), blueprint_sdk::Error> {
    setup_log();

    let env = BlueprintEnvironment::load()?;

    if let Some(data_dir) = env.data_dir.as_ref() {
        let config_path = data_dir.join("config.json");
        if config_path.exists() {
            info!("Using configuration file: {}", config_path.display());
        } else {
            info!("No configuration file found in data directory, using environment variables");
        }
    } else {
        info!("No data directory specified, using environment variables");
    }

    let sr25519_signer = env.keystore().first_local::<SpSr25519>()?;
    let sr25519_pair = env.keystore().get_secret::<SpSr25519>(&sr25519_signer)?;
    let st25519_signer = TanglePairSigner::new(sr25519_pair.0);

    let tangle_client = env.tangle_client().await?;
    let tangle_producer =
        TangleProducer::finalized_blocks(tangle_client.rpc_client.clone()).await?;
    let tangle_consumer = TangleConsumer::new(tangle_client.rpc_client.clone(), st25519_signer);

    let tangle_config = TangleConfig::default();

    info!("Creating OpenRouter context");
    let context = OpenRouterContext::new(env.clone()).await?;

    let config = context.blueprint_config.read().await;
    info!("LLM API URL: {}", config.llm.api_url);
    info!(
        "Load balancer strategy: {:?}",
        config.load_balancer.strategy
    );
    info!("API listening on {}:{}", config.api.host, config.api.port);

    let service_id = env.protocol_settings.tangle()?.service_id.unwrap();
    info!("Using Tangle service ID: {}", service_id);

    let metrics_interval = {
        let config = context.blueprint_config.read().await;
        Duration::from_secs(config.api.metrics_interval_seconds)
    };
    info!(
        "Metrics reporting interval: {} seconds",
        metrics_interval.as_secs()
    );

    let result = BlueprintRunner::builder(tangle_config, env)
        .router(
            Router::new()
                .route(
                    PROCESS_LLM_REQUEST_JOB_ID,
                    process_llm_request.layer(TangleLayer),
                )
                .route(REPORT_METRICS_JOB_ID, report_metrics.layer(TangleLayer))
                .layer(FilterLayer::new(MatchesServiceId(service_id)))
                .with_context(context.clone()),
        )
        .producer(tangle_producer)
        .consumer(tangle_consumer)
        .with_shutdown_handler(async {
            info!("Shutting down OpenRouter Blueprint!");
        })
        .run()
        .await;

    if let Err(e) = result {
        error!("Runner failed! {e:?}");
    }

    Ok(())
}

pub fn setup_log() {
    use tracing_subscriber::util::SubscriberInitExt;

    let _ = tracing_subscriber::fmt::SubscriberBuilder::default()
        .without_time()
        .with_span_events(tracing_subscriber::fmt::format::FmtSpan::NONE)
        .with_env_filter(
            tracing_subscriber::EnvFilter::builder()
                .with_default_directive(LevelFilter::INFO.into())
                .from_env_lossy(),
        )
        .finish()
        .try_init();
}
