/// A builder for constructing a Altair HTML page.
/// https://altairgraphql.dev/
///
/// # Example
///
/// ```rust
/// use async_graphql::http::*;
///
/// AltairGraphQL::build()
///     .endpoint("http://localhost:8000")
///     .subscription_endpoint("ws://localhost:8000/ws")
///     .header("Authorization", "Bearer <token>")
///     .finish();
/// ```
#[derive(Default)]
pub struct AltairGraphQL<'a> {
    endpoint: &'a str,
    subscription_endpoint: Option<&'a str>,
    title: Option<&'a str>,
}

impl<'a> AltairGraphQL<'a> {
    /// Creates a builder for constructing a Altair HTML page.
    pub fn build() -> AltairGraphQL<'a> {
        Default::default()
    }

    /// Sets the endpoint of the server Altair will connect to.
    #[must_use]
    pub fn endpoint(self, endpoint: &'a str) -> AltairGraphQL<'a> {
        AltairGraphQL { endpoint, ..self }
    }

    /// Sets the subscription endpoint of the server Altair will connect to.
    pub fn subscription_endpoint(self, endpoint: &'a str) -> AltairGraphQL<'a> {
        AltairGraphQL {
            subscription_endpoint: Some(endpoint),
            ..self
        }
    }

    /// Sets the html document title.
    pub fn title(self, title: &'a str) -> AltairGraphQL<'a> {
        AltairGraphQL {
            title: Some(title),
            ..self
        }
    }

    /// Returns a Altair HTML page.
    pub fn finish(self) -> String {
        let altair_url = format!("'{}'", self.endpoint);
        let altair_subscription_url = self
            .subscription_endpoint
            .map(|endpoint| format!("'{}'", endpoint))
            .unwrap_or_else(|| "undefined".into());
        let altair_title = self.title.unwrap_or("Altair IDE");

        r#"
<!doctype html>
<html>

  <head>
    <meta charset="utf-8">
    <title>%ALTAIR_TITLE%</title>
    <base href="https://unpkg.com/altair-static@latest/build/dist/">

    <meta name="viewport" content="width=device-width, initial-scale=1">
    <link rel="icon" type="image/x-icon" href="favicon.ico">
    <link rel="stylesheet" href="styles.css">
  </head>

  <body>
    <script>
		document.addEventListener('DOMContentLoaded', () => {
      AltairGraphQL.init({
        endpointURL: %ALTAIR_URL%,
        subscriptionsEndpoint: %ALTAIR_SUBSCRIPTION_URL%
      });
    });
    </script>
    <app-root>
      <style>
        .loading-screen {
          /*Prevents the loading screen from showing until CSS is downloaded*/
          display: none;
        }

      </style>
      <div class="loading-screen styled">
        <div class="loading-screen-inner">
          <div class="loading-screen-logo-container">
            <img src="assets/img/logo_350.svg" alt="Altair">
          </div>
          <div class="loading-screen-loading-indicator">
            <span class="loading-indicator-dot"></span>
            <span class="loading-indicator-dot"></span>
            <span class="loading-indicator-dot"></span>
          </div>
        </div>
      </div>
    </app-root>
    <script type="text/javascript" src="runtime.js"></script>
    <script type="text/javascript" src="polyfills.js"></script>
    <script type="text/javascript" src="main.js"></script>
  </body>

</html>
"#
        .replace("%ALTAIR_URL%", &altair_url)
        .replace("%ALTAIR_SUBSCRIPTION_URL%", &altair_subscription_url)
        .replace("%ALTAIR_TITLE%", altair_title)
    }
}
