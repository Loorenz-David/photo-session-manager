//! The pure rules of conversations sync and replies (`master_plan_conversations.md` §5 MC-C1,
//! MC-C2.1, §4 Event).

use crate::{
    Error, Result,
    domain::conversation::{
        AuthorKind, Conversation, ConversationNotice, ConversationRow, ConversationStatus, Message,
        NewMessage, NoticeKind,
    },
};
use std::{cmp::Ordering, collections::HashSet, time::Duration};

/// MC-C1.7: the wait after `failures` consecutive failed checks (≥ 1): 10 s doubling, at most 5 min.
pub fn check_backoff(failures: u32) -> Duration {
    let exponent = failures.saturating_sub(1).min(16);
    Duration::from_secs((10_u64 << exponent).min(300))
}

/// MC-C1.10: a message is unread ⇔ the asker wrote it and it is newer than the read mark.
pub fn unread(message: &Message, read_mark: Option<i64>) -> bool {
    message.author.kind == AuthorKind::Asker
        && read_mark.is_none_or(|mark| message.created_at > mark)
}

/// MC-C1.10: some message of the conversation is unread.
pub fn has_news(conversation: &Conversation) -> bool {
    conversation
        .messages
        .iter()
        .any(|m| unread(m, conversation.read_mark))
}

/// MC-C1.4 insert rule: an open conversation enters unread; any other enters read up to its
/// newest message.
pub fn entering_read_mark(conversation: &Conversation) -> Option<i64> {
    if conversation.status == ConversationStatus::Open {
        None
    } else {
        conversation.messages.iter().map(|m| m.created_at).max()
    }
}

/// The Resolved tab's window (09B §7): 30 days in milliseconds.
pub const RESOLVED_WINDOW_MS: i64 = 30 * 86_400_000;
/// The longest `first_line`, in characters (the `…` included).
const FIRST_LINE_MAX: usize = 120;

/// 09B §7 list order: news first, then blocking, then latest activity, then id.
pub fn list_order(a: &ConversationRow, b: &ConversationRow) -> Ordering {
    b.has_news
        .cmp(&a.has_news)
        .then(b.blocking.cmp(&a.blocking))
        .then(b.last_activity_at.cmp(&a.last_activity_at))
        .then(a.id.cmp(&b.id))
}

/// 09B §7 Resolved tab: the conversation's last update is at most 30 days before `now`.
pub fn resolved_recent(updated_at: i64, now: i64) -> bool {
    now.saturating_sub(updated_at) <= RESOLVED_WINDOW_MS
}

/// 09B §7: the first non-empty line, trimmed, at most 120 characters (cut on a character boundary
/// with `…`). Empty when the text has no non-empty line.
pub fn first_line(text: &str) -> String {
    let line = text
        .lines()
        .map(str::trim)
        .find(|l| !l.is_empty())
        .unwrap_or("");
    if line.chars().count() <= FIRST_LINE_MAX {
        return line.to_owned();
    }
    let mut cut: String = line.chars().take(FIRST_LINE_MAX - 1).collect();
    cut.push('…');
    cut
}

/// The words of one notice (09B §9): `New question · <SKU> — <topic>` or `New reply · <SKU>`,
/// the product title in place of a missing SKU.
fn notice_line(notice: &ConversationNotice) -> String {
    let product = notice.sku.as_deref().unwrap_or(&notice.product_title);
    match notice.kind {
        NoticeKind::Question => format!("New question · {product} — {}", notice.topic),
        NoticeKind::Reply => format!("New reply · {product}"),
    }
}

/// §4 Event: at most three lines, in order, and the number left out.
pub fn notice_lines(notices: &[ConversationNotice]) -> (Vec<(String, String)>, usize) {
    let lines = notices
        .iter()
        .take(3)
        .map(|n| (n.conversation_id.clone(), notice_line(n)))
        .collect();
    (lines, notices.len().saturating_sub(3))
}

/// The longest reply, in Unicode scalar values (the package's `maxLength`).
pub const REPLY_MAX: usize = 4000;
/// The longest `client_mutation_id` (openapi `clientMutationId`).
pub const CLIENT_ID_MAX: usize = 80;
/// The most photos one reply carries.
pub const ATTACHMENTS_MAX: usize = 10;
/// The longest reply excerpt, in Unicode scalar values.
const EXCERPT_MAX: usize = 80;

/// MC-C2.1 Accept, the pure part: the text trimmed and 1–4000 Unicode scalar values, the
/// `client_mutation_id` 1–80 characters, at most 10 media ids and none repeated. A refusal is
/// `invalid_value` naming the field. Whether a media id is acceptable is the provider's (H7).
pub fn checked_reply(input: NewMessage) -> Result<NewMessage> {
    let text = input.text.trim().to_owned();
    if text.is_empty() || text.chars().count() > REPLY_MAX {
        return Err(Error::InvalidValue("text".into()));
    }
    let id_length = input.client_mutation_id.chars().count();
    if id_length == 0 || id_length > CLIENT_ID_MAX {
        return Err(Error::InvalidValue("client_mutation_id".into()));
    }
    let ids = &input.attachment_media_ids;
    let distinct: HashSet<&String> = ids.iter().collect();
    if ids.len() > ATTACHMENTS_MAX || distinct.len() != ids.len() {
        return Err(Error::InvalidValue("attachment_media_ids".into()));
    }
    Ok(NewMessage { text, ..input })
}

/// The excerpt a reply shows of its target (projection L9): the target's first line (up to the
/// first `\n`), at most 80 Unicode scalar values.
pub fn reply_excerpt(text: &str) -> String {
    let line = text.split('\n').next().unwrap_or("");
    line.chars().take(EXCERPT_MAX).collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::conversation::Author;

    fn new_message(cmid: &str, text: &str, media: &[&str]) -> NewMessage {
        NewMessage {
            client_mutation_id: cmid.into(),
            text: text.into(),
            reply_to_message_id: None,
            attachment_media_ids: media.iter().map(|m| (*m).to_owned()).collect(),
        }
    }

    fn refused_field(result: Result<NewMessage>) -> String {
        match result {
            Err(Error::InvalidValue(field)) => field,
            other => panic!("expected a refusal, got {other:?}"),
        }
    }

    #[test]
    fn c3_c1a_the_text_is_trimmed() {
        let checked = checked_reply(new_message("m-1", "  Original glass.  ", &[])).unwrap();
        assert_eq!(checked.text, "Original glass.");
    }

    #[test]
    fn c3_c1b_the_text_is_1_to_4000_scalar_values() {
        assert_eq!(
            refused_field(checked_reply(new_message("m-1", "   ", &[]))),
            "text"
        );
        let full = "é".repeat(4000);
        assert_eq!(
            checked_reply(new_message("m-1", &full, &[])).unwrap().text,
            full
        );
        let over = "é".repeat(4001);
        assert_eq!(
            refused_field(checked_reply(new_message("m-1", &over, &[]))),
            "text"
        );
    }

    #[test]
    fn c3_c1c_the_client_id_is_1_to_80_characters() {
        assert_eq!(
            refused_field(checked_reply(new_message("", "t", &[]))),
            "client_mutation_id"
        );
        assert_eq!(
            refused_field(checked_reply(new_message(&"a".repeat(81), "t", &[]))),
            "client_mutation_id"
        );
        let eighty = "a".repeat(80);
        assert_eq!(
            checked_reply(new_message(&eighty, "t", &[]))
                .unwrap()
                .client_mutation_id,
            eighty
        );
    }

    #[test]
    fn c3_c1d_at_most_ten_distinct_media_ids() {
        let ids: Vec<String> = (1..=11).map(|n| format!("media-{n}")).collect();
        let eleven: Vec<&str> = ids.iter().map(String::as_str).collect();
        assert_eq!(
            refused_field(checked_reply(new_message("m-1", "t", &eleven))),
            "attachment_media_ids"
        );
        assert_eq!(
            checked_reply(new_message("m-1", "t", &eleven[..10]))
                .unwrap()
                .attachment_media_ids,
            ids[..10].to_vec()
        );
        assert_eq!(
            refused_field(checked_reply(new_message(
                "m-1",
                "t",
                &["media-1", "media-1"]
            ))),
            "attachment_media_ids"
        );
    }

    fn message(kind: AuthorKind, at: i64) -> Message {
        Message {
            id: format!("m{at}"),
            conversation_id: "c".into(),
            author: Author {
                kind,
                name: "n".into(),
            },
            text: "t".into(),
            quote: None,
            reply_to: None,
            attachments: vec![],
            created_at: at,
            edited_at: None,
        }
    }

    fn conversation(status: ConversationStatus, times: &[i64]) -> Conversation {
        Conversation {
            id: "c".into(),
            store_id: "s".into(),
            product_id: "p".into(),
            sku: None,
            product_title: "t".into(),
            topic: "Condition".into(),
            blocking: false,
            status,
            resolved_by: None,
            created_at: 0,
            updated_at: 0,
            read_mark: None,
            messages: times
                .iter()
                .map(|t| message(AuthorKind::Asker, *t))
                .collect(),
        }
    }

    fn notice(kind: NoticeKind, sku: Option<&str>, title: &str, topic: &str) -> ConversationNotice {
        ConversationNotice {
            conversation_id: format!("c-{title}"),
            sku: sku.map(str::to_owned),
            product_title: title.into(),
            topic: topic.into(),
            kind,
        }
    }

    #[test]
    fn c1_c15a_one_failure_waits_10_s() {
        assert_eq!(check_backoff(1), Duration::from_secs(10));
    }
    #[test]
    fn c1_c15b_two_failures_wait_20_s() {
        assert_eq!(check_backoff(2), Duration::from_secs(20));
    }
    #[test]
    fn c1_c15c_three_failures_wait_40_s() {
        assert_eq!(check_backoff(3), Duration::from_secs(40));
    }
    #[test]
    fn c1_c15d_four_failures_wait_80_s() {
        assert_eq!(check_backoff(4), Duration::from_secs(80));
    }
    #[test]
    fn c1_c15e_five_failures_wait_160_s() {
        assert_eq!(check_backoff(5), Duration::from_secs(160));
    }
    #[test]
    fn c1_c15f_six_failures_wait_300_s() {
        assert_eq!(check_backoff(6), Duration::from_secs(300));
    }
    #[test]
    fn c1_c15g_more_failures_stay_at_300_s() {
        assert_eq!(check_backoff(7), Duration::from_secs(300));
        assert_eq!(check_backoff(40), Duration::from_secs(300));
    }

    #[test]
    fn c1_c17g_notice_lines_words() {
        let (lines, more) = notice_lines(&[
            notice(
                NoticeKind::Question,
                Some("BEYO-1880"),
                "Mirror",
                "Condition",
            ),
            notice(NoticeKind::Reply, Some("BEYO-7"), "Lamp", "Other"),
            notice(NoticeKind::Question, None, "Brass Mirror", "Other"),
        ]);
        let texts: Vec<&str> = lines.iter().map(|(_, t)| t.as_str()).collect();
        assert_eq!(
            texts,
            [
                "New question · BEYO-1880 — Condition",
                "New reply · BEYO-7",
                "New question · Brass Mirror — Other",
            ]
        );
        assert_eq!(lines[0].0, "c-Mirror");
        assert_eq!(more, 0);
    }

    #[test]
    fn c1_c17h_at_most_three_lines_then_the_rest_counted() {
        let five: Vec<_> = (1..=5)
            .map(|n| notice(NoticeKind::Reply, Some(&format!("BEYO-{n}")), "t", "Other"))
            .collect();
        let (lines, more) = notice_lines(&five);
        let texts: Vec<&str> = lines.iter().map(|(_, t)| t.as_str()).collect();
        assert_eq!(
            texts,
            [
                "New reply · BEYO-1",
                "New reply · BEYO-2",
                "New reply · BEYO-3"
            ]
        );
        assert_eq!(more, 2);
        assert_eq!(notice_lines(&[]), (vec![], 0));
    }

    #[test]
    fn c1_c18a_unread_and_has_news() {
        let pricer = message(AuthorKind::Asker, 100);
        assert!(unread(&pricer, None));
        assert!(unread(&pricer, Some(99)));
        assert!(!unread(&pricer, Some(100)));
        let mine = message(AuthorKind::Me, 200);
        assert!(!unread(&mine, None));
        let mut c = conversation(ConversationStatus::Open, &[100]);
        assert!(has_news(&c));
        c.read_mark = Some(100);
        assert!(!has_news(&c));
    }

    fn row(id: &str, has_news: bool, blocking: bool, last_activity_at: i64) -> ConversationRow {
        ConversationRow {
            id: id.into(),
            product_id: format!("p-{id}"),
            sku: None,
            product_title: "t".into(),
            topic: "Condition".into(),
            status: ConversationStatus::Open,
            blocking,
            waiting: blocking,
            last_line: None,
            last_activity_at,
            has_news,
            session_id: None,
            questions: 1,
        }
    }

    #[test]
    fn c2_c2a_news_comes_before_blocking() {
        let a = row("a", true, false, 100);
        let b = row("b", false, true, 200);
        assert_eq!(list_order(&a, &b), Ordering::Less);
        assert_eq!(list_order(&b, &a), Ordering::Greater);
    }

    #[test]
    fn c2_c2b_then_blocking_before_newer() {
        let a = row("a", false, true, 100);
        let b = row("b", false, false, 200);
        assert_eq!(list_order(&a, &b), Ordering::Less);
    }

    #[test]
    fn c2_c2c_then_latest_activity_first() {
        let a = row("a", false, false, 200);
        let b = row("b", false, false, 100);
        assert_eq!(list_order(&a, &b), Ordering::Less);
    }

    #[test]
    fn c2_c2d_both_with_news_blocking_first() {
        let a = row("a", true, false, 200);
        let b = row("b", true, true, 100);
        assert_eq!(list_order(&b, &a), Ordering::Less);
    }

    #[test]
    fn c2_c3a_resolved_recent_is_thirty_days_inclusive() {
        let now = 10_000_000_000;
        let day = 86_400_000;
        assert!(resolved_recent(now - 29 * day, now));
        assert!(resolved_recent(now - 30 * day, now));
        assert!(!resolved_recent(now - 30 * day - 1, now));
    }

    #[test]
    fn c2_c4a_first_line() {
        assert_eq!(
            first_line("Is the glass original?\nSecond line"),
            "Is the glass original?"
        );
        assert_eq!(first_line("\n  Send a close-up  \n"), "Send a close-up");
        let long = "é".repeat(200);
        let cut = first_line(&long);
        assert_eq!(cut, format!("{}…", "é".repeat(119)));
        assert_eq!(cut.chars().count(), 120);
    }

    #[test]
    fn c1_c18b_entering_read_mark() {
        assert_eq!(
            entering_read_mark(&conversation(ConversationStatus::Open, &[100, 200])),
            None
        );
        assert_eq!(
            entering_read_mark(&conversation(ConversationStatus::Answered, &[100, 200])),
            Some(200)
        );
        assert_eq!(
            entering_read_mark(&conversation(ConversationStatus::Resolved, &[])),
            None
        );
    }
}
