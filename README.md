# tower-webflow

`tower-webflow` is a crate to handle validating webhooks from webflow.

It depends on Axum for extracting the headers and body of incoming requests.

To run the example:

```
cargo run --example static
```

To ensure that cargo check also looks at examples:

```
cargo check --examples
```
