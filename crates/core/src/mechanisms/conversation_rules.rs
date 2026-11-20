//! The pure rules of conversations sync (`master_plan_conversations.md` §5 MC-C1, §4 Event).

use crate::domain::conversation::{
    AuthorKind, Conversation, ConversationNotice, ConversationStatus, Message, NoticeKind,
};
use std::time::Duration;

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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::conversation::Author;

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
