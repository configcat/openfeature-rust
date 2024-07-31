use configcat::*;
use configcat_openfeature_provider::ConfigCatProvider;
use open_feature::{EvaluationContext, OpenFeature};
use std::time::Duration;

#[tokio::main]
async fn main() {
    let mut api = OpenFeature::singleton_mut().await;

    let configcat_client = Client::builder("PKDVCLf-Hq-h-kCzMp-L7Q/HhOWfwVtZ0mb30i9wi17GQ")
        .polling_mode(PollingMode::AutoPoll(Duration::from_secs(5)))
        .build()
        .unwrap();

    api.set_provider(ConfigCatProvider::new(configcat_client))
        .await;

    let client = api.create_client();

    let is_awesome_enabled = client
        .get_bool_value("isAwesomeFeatureEnabled", None, None)
        .await
        .unwrap();

    println!("isAwesomeFeatureEnabled: {is_awesome_enabled}");

    let ctx = EvaluationContext::default()
        .with_targeting_key("#SOME-USER-ID#")
        .with_custom_field("Email", "configcat@example.com");

    let is_poc_enabled = client
        .get_bool_value("isPOCFeatureEnabled", Some(&ctx), None)
        .await
        .unwrap();

    println!("isPOCFeatureEnabled: {is_poc_enabled}");
}
