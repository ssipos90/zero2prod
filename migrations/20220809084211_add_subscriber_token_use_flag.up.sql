-- Add up migration script here
ALTER TABLE subscription_tokens
    ADD COLUMN used BOOLEAN NOT NULL DEFAULT FALSE;

UPDATE subscription_tokens
 SET used=TRUE
    FROM subscriptions AS s
    WHERE s.id=subscription_tokens.subscriber_id
      AND s.status!='pending_confirmation';
