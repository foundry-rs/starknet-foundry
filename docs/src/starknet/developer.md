# Developer Functionalities

## Logging

`sncast` supports emitting logs, including the logs from the requests sent to the network.
Logs can be enabled by setting `CAST_LOG` environment variable to desired log level.

For example, to enable logs with level `debug` and higher, run:

`SNCAST_LOG="debug" sncast ...`

Additional filtering can be set to the `CAST_LOG` environment variable,
see [tracing-subscriber](https://docs.rs/tracing-subscriber/0.3.20/tracing_subscriber/filter/struct.EnvFilter.html#directives)
documentation for more details.

> ⚠️ **Warning**
>
> Logs can expose sensitive information, such as private keys. Never use logging in production environments.
