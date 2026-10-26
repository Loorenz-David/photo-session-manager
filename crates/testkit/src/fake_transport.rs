//! `FakeTransport` (master plan §7.5): replays sanitized Shopify fixtures for the real
//! `ShopifyAdapter`. Replies are keyed by GraphQL operation name + a digest of the variables
//! (canonical JSON), or by operation name alone (any variables); each key holds a sequence of
//! replies (for polls), optionally repeating its last one. An unmatched request panics, which
//! fails the test. Every request is recorded.

use crate::fixtures::shopify::{Fixture, operation_name};
use psm_core::{Result, mechanisms::values::canonical_json};
use psm_integrations::commerce::shopify::transport::{HttpReply, HttpTransport};
use serde_json::Value as Json;
use std::{collections::VecDeque, sync::Mutex};

#[derive(Debug, Clone, PartialEq)]
pub enum Recorded {
    Graphql {
        url: String,
        access_token: String,
        operation: String,
        query: String,
        variables: Json,
    },
    Form {
        url: String,
        fields: Vec<(String, String)>,
    },
    Put {
        url: String,
        headers: Vec<(String, String)>,
        bytes: usize,
    },
}

struct Rule {
    key: String,
    digest: Option<String>,
    replies: VecDeque<HttpReply>,
    repeat_last: bool,
}

#[derive(Default)]
pub struct FakeTransport {
    rules: Mutex<Vec<Rule>>,
    requests: Mutex<Vec<Recorded>>,
}

impl FakeTransport {
    pub fn new() -> Self {
        Self::default()
    }

    fn push(
        &self,
        key: String,
        variables: Option<&Json>,
        replies: Vec<HttpReply>,
        repeat_last: bool,
    ) {
        let digest = variables.map(canonical_json);
        let mut rules = self.rules.lock().unwrap();
        if let Some(rule) = rules
            .iter_mut()
            .find(|r| r.key == key && r.digest == digest && !r.repeat_last && !repeat_last)
        {
            rule.replies.extend(replies);
        } else {
            rules.push(Rule {
                key,
                digest,
                replies: replies.into(),
                repeat_last,
            });
        }
    }

    /// Queue replies for a GraphQL operation; `variables: None` matches any variables.
    pub fn on_graphql(&self, operation: &str, variables: Option<&Json>, replies: Vec<HttpReply>) {
        self.push(format!("graphql:{operation}"), variables, replies, false);
    }

    /// Answer every matching GraphQL request with `reply`.
    pub fn on_graphql_always(&self, operation: &str, variables: Option<&Json>, reply: HttpReply) {
        self.push(format!("graphql:{operation}"), variables, vec![reply], true);
    }

    pub fn on_form(&self, url: &str, replies: Vec<HttpReply>) {
        self.push(format!("form:{url}"), None, replies, false);
    }

    pub fn on_form_always(&self, url: &str, reply: HttpReply) {
        self.push(format!("form:{url}"), None, vec![reply], true);
    }

    pub fn on_put(&self, url: &str, reply: HttpReply) {
        self.push(format!("put:{url}"), None, vec![reply], false);
    }

    /// Queue a GraphQL fixture under its own operation name and request variables.
    pub fn replay(&self, fixture: &Fixture) {
        let op = fixture
            .operation()
            .unwrap_or_else(|| panic!("fixture {} is not a GraphQL request", fixture.name));
        self.on_graphql(&op, Some(&fixture.variables()), vec![fixture.reply()]);
    }

    /// Queue a GraphQL fixture's reply for its operation with any variables.
    pub fn replay_any(&self, fixture: &Fixture) {
        let op = fixture
            .operation()
            .unwrap_or_else(|| panic!("fixture {} is not a GraphQL request", fixture.name));
        self.on_graphql(&op, None, vec![fixture.reply()]);
    }

    fn answer(&self, key: &str, variables: Option<&Json>, describe: &str) -> HttpReply {
        let digest = variables.map(canonical_json);
        let mut rules = self.rules.lock().unwrap();
        let pick = |exact: bool, rules: &mut Vec<Rule>| -> Option<HttpReply> {
            let rule = rules.iter_mut().find(|r| {
                r.key == key
                    && !r.replies.is_empty()
                    && if exact {
                        r.digest.is_some() && r.digest == digest
                    } else {
                        r.digest.is_none()
                    }
            })?;
            if rule.repeat_last && rule.replies.len() == 1 {
                rule.replies.front().cloned()
            } else {
                rule.replies.pop_front()
            }
        };
        if let Some(reply) = pick(true, &mut rules) {
            return reply;
        }
        if let Some(reply) = pick(false, &mut rules) {
            return reply;
        }
        drop(rules);
        panic!("FakeTransport: unmatched request {describe}");
    }

    pub fn requests(&self) -> Vec<Recorded> {
        self.requests.lock().unwrap().clone()
    }

    /// The recorded GraphQL requests of one operation, in order.
    pub fn graphql(&self, operation: &str) -> Vec<(String, Json)> {
        self.requests()
            .into_iter()
            .filter_map(|r| match r {
                Recorded::Graphql {
                    operation: op,
                    access_token,
                    variables,
                    ..
                } if op == operation => Some((access_token, variables)),
                _ => None,
            })
            .collect()
    }

    pub fn graphql_count(&self) -> usize {
        self.requests()
            .iter()
            .filter(|r| matches!(r, Recorded::Graphql { .. }))
            .count()
    }

    pub fn forms(&self) -> Vec<(String, Vec<(String, String)>)> {
        self.requests()
            .into_iter()
            .filter_map(|r| match r {
                Recorded::Form { url, fields } => Some((url, fields)),
                _ => None,
            })
            .collect()
    }

    pub fn puts(&self) -> Vec<(String, Vec<(String, String)>)> {
        self.requests()
            .into_iter()
            .filter_map(|r| match r {
                Recorded::Put { url, headers, .. } => Some((url, headers)),
                _ => None,
            })
            .collect()
    }
}

impl HttpTransport for FakeTransport {
    fn post_graphql(
        &self,
        url: &str,
        access_token: &str,
        query: &str,
        variables: &Json,
    ) -> Result<HttpReply> {
        let operation = operation_name(query).unwrap_or_default();
        self.requests.lock().unwrap().push(Recorded::Graphql {
            url: url.to_owned(),
            access_token: access_token.to_owned(),
            operation: operation.clone(),
            query: query.to_owned(),
            variables: variables.clone(),
        });
        Ok(self.answer(
            &format!("graphql:{operation}"),
            Some(variables),
            &format!("graphql {operation} {}", canonical_json(variables)),
        ))
    }

    fn post_form(&self, url: &str, fields: &[(&str, &str)]) -> Result<HttpReply> {
        self.requests.lock().unwrap().push(Recorded::Form {
            url: url.to_owned(),
            fields: fields
                .iter()
                .map(|(k, v)| ((*k).to_owned(), (*v).to_owned()))
                .collect(),
        });
        Ok(self.answer(&format!("form:{url}"), None, &format!("form {url}")))
    }

    fn put_bytes(
        &self,
        url: &str,
        headers: &[(String, String)],
        bytes: &[u8],
    ) -> Result<HttpReply> {
        self.requests.lock().unwrap().push(Recorded::Put {
            url: url.to_owned(),
            headers: headers.to_vec(),
            bytes: bytes.len(),
        });
        Ok(self.answer(&format!("put:{url}"), None, &format!("put {url}")))
    }
}
