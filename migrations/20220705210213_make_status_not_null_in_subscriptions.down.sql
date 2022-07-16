-- Add down migration script here
ALTER TABLE subscriptions ALTER COLUMN status SET NULL;
