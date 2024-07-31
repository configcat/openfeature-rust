use configcat::FileDataSource;
use configcat::OverrideBehavior::LocalOnly;
use configcat_openfeature_provider::ConfigCatProvider;
use open_feature::{
    EvaluationContext, EvaluationError, EvaluationErrorCode, EvaluationReason, OpenFeature,
    StructValue,
};

#[tokio::test]
async fn eval_bool() {
    let mut api = OpenFeature::singleton_mut().await;
    let configcat_client = create_client();
    api.set_provider(ConfigCatProvider::new(configcat_client))
        .await;
    let client = api.create_client();

    let details = client
        .get_bool_details("enabledFeature", None, None)
        .await
        .unwrap();

    assert!(details.value);
    assert_eq!("v-enabled", details.variant.unwrap());
    assert_eq!(EvaluationReason::Default, details.reason.unwrap());
}

#[tokio::test]
async fn eval_int() {
    let mut api = OpenFeature::singleton_mut().await;
    let configcat_client = create_client();
    api.set_provider(ConfigCatProvider::new(configcat_client))
        .await;
    let client = api.create_client();

    let details = client
        .get_int_details("intSetting", None, None)
        .await
        .unwrap();

    assert_eq!(5, details.value);
    assert_eq!("v-int", details.variant.unwrap());
    assert_eq!(EvaluationReason::Default, details.reason.unwrap());
}

#[tokio::test]
async fn eval_float() {
    let mut api = OpenFeature::singleton_mut().await;
    let configcat_client = create_client();
    api.set_provider(ConfigCatProvider::new(configcat_client))
        .await;
    let client = api.create_client();

    let details = client
        .get_float_details("doubleSetting", None, None)
        .await
        .unwrap();

    assert_eq!(1.2, details.value);
    assert_eq!("v-double", details.variant.unwrap());
    assert_eq!(EvaluationReason::Default, details.reason.unwrap());
}

#[tokio::test]
async fn eval_string() {
    let mut api = OpenFeature::singleton_mut().await;
    let configcat_client = create_client();
    api.set_provider(ConfigCatProvider::new(configcat_client))
        .await;
    let client = api.create_client();

    let details = client
        .get_string_details("stringSetting", None, None)
        .await
        .unwrap();

    assert_eq!("test", details.value);
    assert_eq!("v-string", details.variant.unwrap());
    assert_eq!(EvaluationReason::Default, details.reason.unwrap());
}

#[tokio::test]
async fn eval_object() {
    let mut api = OpenFeature::singleton_mut().await;
    let configcat_client = create_client();
    api.set_provider(ConfigCatProvider::new(configcat_client))
        .await;
    let client = api.create_client();

    let details = client
        .get_struct_details::<Sample>("objectSetting", None, None)
        .await
        .unwrap();

    assert_eq!("value", details.value.text);
    assert!(details.value.boolean);
    assert_eq!("v-object", details.variant.unwrap());
    assert_eq!(EvaluationReason::Default, details.reason.unwrap());
}

#[tokio::test]
async fn eval_targeting() {
    let mut api = OpenFeature::singleton_mut().await;
    let configcat_client = create_client();
    api.set_provider(ConfigCatProvider::new(configcat_client))
        .await;
    let client = api.create_client();

    let details = client
        .get_bool_details(
            "disabledFeature",
            Some(&EvaluationContext::default().with_targeting_key("example@matching.com")),
            None,
        )
        .await
        .unwrap();

    assert!(details.value);
    assert_eq!("v-disabled-t", details.variant.unwrap());
    assert_eq!(EvaluationReason::TargetingMatch, details.reason.unwrap());
}

#[tokio::test]
async fn eval_key_not_found() {
    let mut api = OpenFeature::singleton_mut().await;
    let configcat_client = create_client();
    api.set_provider(ConfigCatProvider::new(configcat_client))
        .await;
    let client = api.create_client();

    let details = client.get_bool_details("non-existing", None, None).await;

    assert!(details.is_err());
    assert_eq!(
        EvaluationErrorCode::FlagNotFound,
        details.clone().err().unwrap().code
    );
    assert!(details.clone().err().unwrap().message.unwrap().starts_with("Failed to evaluate setting 'non-existing' (the key was not found in config JSON). Returning the `defaultValue` parameter that you specified in your application: 'false'. Available keys:"));
}

#[tokio::test]
async fn eval_type_mismatch() {
    let mut api = OpenFeature::singleton_mut().await;
    let configcat_client = create_client();
    api.set_provider(ConfigCatProvider::new(configcat_client))
        .await;
    let client = api.create_client();

    let details = client.get_bool_details("stringSetting", None, None).await;

    assert!(details.is_err());
    assert_eq!(
        EvaluationErrorCode::TypeMismatch,
        details.clone().err().unwrap().code
    );
    assert_eq!(details.clone().err().unwrap().message.unwrap(), "The type of a setting must match the requested type. Setting's type was 'String' but the requested type was 'bool'. Learn more: https://configcat.com/docs/sdk-reference/rust/#setting-type-mapping");
}

fn create_client() -> configcat::Client {
    configcat::Client::builder("local")
        .overrides(
            Box::new(FileDataSource::new("tests/data/test_json_complex.json").unwrap()),
            LocalOnly,
        )
        .build()
        .unwrap()
}

#[derive(Default)]
struct Sample {
    pub boolean: bool,
    pub text: String,
}

impl TryFrom<StructValue> for Sample {
    type Error = EvaluationError;

    fn try_from(value: StructValue) -> Result<Self, Self::Error> {
        let mut sample = Sample::default();
        if let Some(b) = value.fields.get("bool_field") {
            sample.boolean = b.as_bool().unwrap()
        };
        if let Some(b) = value.fields.get("text_field") {
            sample.text = b.as_str().unwrap().to_owned()
        };
        Ok(sample)
    }
}
