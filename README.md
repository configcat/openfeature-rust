# ConfigCat OpenFeature Provider for Rust

[![Build Status](https://github.com/configcat/openfeature-rust/actions/workflows/ci.yml/badge.svg?branch=main)](https://github.com/configcat/openfeature-rust/actions/workflows/ci.yml)
[![crates.io](https://img.shields.io/crates/v/configcat-openfeature-provider.svg?logo=rust)](https://crates.io/crates/configcat-openfeature-provider)
[![docs.rs](https://img.shields.io/badge/docs.rs-configcat-openfeature-provider-66c2a5?logo=docs.rs)](https://docs.rs/configcat-openfeature-provider)

This repository contains an OpenFeature provider that allows the usage of [ConfigCat](https://configcat.com) with the [OpenFeature Rust SDK](https://github.com/open-feature/rust-sdk).

## Installation

Run the following Cargo command in your project directory:
```shell
cargo add configcat-openfeature
```

Or add the following to your `Cargo.toml`:

```toml
[dependencies]
configcat-openfeature-provider = "0.1"
```

## Usage

The `ConfigCatProvider` needs a pre-configured [ConfigCat Rust SDK](https://github.com/configcat/php-sdk) client:

```rust
use std::time::Duration;
use configcat::{Client, PollingMode};
use open_feature::OpenFeature;
use configcat_openfeature_provider::ConfigCatProvider;

#[tokio::main]
async fn main() {
    // Acquire an OpenFeature API instance.
    let mut api = OpenFeature::singleton_mut().await;

    // Configure the ConfigCat SDK.
    let configcat_client = Client::builder("sdk-key")
        .polling_mode(PollingMode::AutoPoll(Duration::from_secs(60)))
        .build()
        .unwrap();

    // Configure the provider.
    api.set_provider(ConfigCatProvider::new(configcat_client)).await;

    // Create a client.
    let client = api.create_client();

    // Evaluate a feature flag.
    let is_awesome_feature_enabled = client
        .get_bool_value("isAwesomeFeatureEnabled", None, None)
        .await
        .unwrap_or(false);
}
```

For more information about all the configuration options, see the [Rust SDK documentation](https://configcat.com/docs/sdk-reference/rust/#creating-the-configcat-client).

## Example

This repository contains a simple [example application](./examples/print_eval.rs) that you can run with:
```shell
cargo run --example print_eval
```

## Need help?
https://configcat.com/support

## Contributing
Contributions are welcome. For more info please read the [Contribution Guideline](CONTRIBUTING.md).

## About ConfigCat
ConfigCat is a feature flag and configuration management service that lets you separate releases from deployments. You can turn your features ON/OFF using <a href="https://app.configcat.com" target="_blank">ConfigCat Dashboard</a> even after they are deployed. ConfigCat lets you target specific groups of users based on region, email or any other custom user attribute.

ConfigCat is a <a href="https://configcat.com" target="_blank">hosted feature flag service</a>. Manage feature toggles across frontend, backend, mobile, desktop apps. <a href="https://configcat.com" target="_blank">Alternative to LaunchDarkly</a>. Management app + feature flag SDKs.

- [Official ConfigCat SDKs for other platforms](https://github.com/configcat)
- [Documentation](https://configcat.com/docs)
- [Blog](https://configcat.com/blog)
