-- Add down migration script here
ALTER TABLE subscription_tokens
    DROP COLUMN used;
