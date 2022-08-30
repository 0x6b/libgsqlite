use crate::error::{
    Error,
    Error::{CodeMissing, InvalidRedirectUrl, InvalidSheetId, UnexpectedResponse, UnexpectedToken},
};
use chrono::{DateTime, Duration, Utc};
use google_sheets4::api::Spreadsheet;
use oauth2::{
    basic::{BasicClient, BasicTokenResponse, BasicTokenType},
    reqwest::http_client,
    url::Url,
    AccessToken, AuthUrl, AuthorizationCode, ClientId, ClientSecret, CsrfToken,
    EmptyExtraTokenFields, RedirectUrl, Scope, TokenResponse, TokenUrl,
};
use serde::{Deserialize, Serialize};
use serde_json::Deserializer;
#[cfg(target_family = "unix")]
use std::os::unix::fs::PermissionsExt;
use std::{
    env::temp_dir,
    fs::File,
    io::{BufRead, BufReader, Write},
    net::TcpListener,
    ops::Sub,
    path::PathBuf,
};
use typed_builder::TypedBuilder;

#[derive(TypedBuilder)]
pub struct GoogleSheetsReadOnlyClient {
    #[builder(setter(into))]
    client_id: String,
    #[builder(setter(into))]
    client_secret: String,
    #[builder(setter(into), default = 8080)]
    port: u16,
    #[builder(setter(into), default = "https://accounts.google.com/o/oauth2/auth".to_string())]
    google_auth_url: String,
    #[builder(setter(into), default = "https://oauth2.googleapis.com/token".to_string())]
    google_token_url: String,
    #[builder(setter(into), default = "https://content-sheets.googleapis.com/v4/spreadsheets/".to_string())]
    content_url: String,
    #[builder(default = false)]
    cache_access_token: bool,
}

#[derive(Serialize, Deserialize)]
struct Cache {
    pub secret: String,
    pub created: DateTime<Utc>,
}

impl GoogleSheetsReadOnlyClient {
    pub fn get(
        &self,
        sheet_id: impl Into<String>,
        sheet_name: impl Into<String>,
        range: impl Into<String>,
    ) -> Result<Spreadsheet, Error> {
        let mut id = sheet_id.into();
        if id.starts_with("https://") {
            id = Url::parse(&id)?
                .path_segments()
                .ok_or(InvalidSheetId)?
                .collect::<Vec<&str>>()
                .get(2)
                .ok_or(InvalidSheetId)?
                .to_string();
        }

        let response = reqwest::blocking::Client::new()
            .get(format!("{}{}", self.content_url, id))
            .query(&[
                ("includeGridData", "true"),
                (
                    "ranges",
                    format!("{}!{}", sheet_name.into(), range.into()).as_str(),
                ),
            ])
            .header("Authorization", format!("Bearer {}", self.get_token()?))
            .send()?;

        if !response.status().is_success() {
            return Err(UnexpectedResponse(response.text().unwrap_or_else(|_| {
                "Unexpected response with no explanation from Google".to_string()
            })));
        }

        let text = response.text()?;
        Ok(serde_json::from_str(&text)?)
    }

    fn get_access_token_cache_path() -> PathBuf {
        let mut path_buf: PathBuf = temp_dir();
        path_buf.push("access_token.json");
        path_buf
    }

    fn get_token(&self) -> Result<String, Error> {
        if self.cache_access_token {
            if let Ok(f) = &File::open(Self::get_access_token_cache_path()) {
                let mut de = Deserializer::from_reader(f);
                let cache = Cache::deserialize(&mut de)?;
                if Utc::now().sub(Duration::minutes(59)) < cache.created {
                    return Ok(cache.secret);
                }
            }
        }

        let client = BasicClient::new(
            ClientId::new(self.client_id.clone()),
            Some(ClientSecret::new(self.client_secret.clone())),
            AuthUrl::new(self.google_auth_url.clone())?,
            Some(TokenUrl::new(self.google_token_url.clone())?),
        )
        .set_redirect_uri(RedirectUrl::new(format!("http://localhost:{}", self.port))?);

        let (authorize_url, _) = client
            .authorize_url(CsrfToken::new_random)
            .add_scopes(vec![
                Scope::new("https://www.googleapis.com/auth/drive.readonly".to_string()),
                Scope::new("https://www.googleapis.com/auth/spreadsheets.readonly".to_string()),
            ])
            .url();

        if open::that(authorize_url.to_string()).is_err() {
            println!(
                "Please open following URL with your browser:\n\n    {}",
                authorize_url
            );
        }

        let mut token: BasicTokenResponse = BasicTokenResponse::new(
            AccessToken::new("placeholder".into()),
            BasicTokenType::Bearer,
            EmptyExtraTokenFields {},
        );

        if let Some(mut stream) = TcpListener::bind(format!("127.0.0.1:{}", self.port))?
            .incoming()
            .flatten()
            .next()
        {
            let mut request_line = String::new();
            BufReader::new(&stream).read_line(&mut request_line)?;

            let redirect_url = request_line
                .split_whitespace()
                .nth(1)
                .ok_or(InvalidRedirectUrl)?;
            let url = Url::parse(format!("http://localhost{}", redirect_url).as_str())?;

            let code_pair = url
                .query_pairs()
                .find(|pair| {
                    let &(ref key, _) = pair;
                    key == "code"
                })
                .ok_or(CodeMissing)?;

            let (_, value) = code_pair;
            let code = AuthorizationCode::new(value.into_owned());

            let message = "Go back to your terminal. You can close this tab.";
            let response = format!(
                "HTTP/1.1 200 OK\r\ncontent-length: {}\r\n\r\n{}",
                message.len(),
                message
            );
            stream.write_all(response.as_bytes())?;
            token = client.exchange_code(code).request(http_client)?;
        }

        if token.access_token().secret() == "placeholder" {
            return Err(UnexpectedToken);
        }

        if self.cache_access_token {
            let file = File::create(Self::get_access_token_cache_path())?;
            let mut perms = file.metadata()?.permissions();

            #[cfg(target_family = "windows")]
            {
                perms.set_readonly(true);
            }

            #[cfg(target_family = "unix")]
            {
                perms.set_mode(0o600);
            }

            file.set_permissions(perms)?;

            serde_json::to_writer_pretty(
                &file,
                &Cache {
                    secret: token.access_token().secret().clone(),
                    created: Utc::now(),
                },
            )?;
        }
        Ok(token.access_token().secret().clone())
    }
}
