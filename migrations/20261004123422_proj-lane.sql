-- Project lanes.
--
-- `project_lane` is mandatory on every event, cached response and embedding, so it is
-- added NOT NULL. Existing rows are backfilled to the seeded `default` lane by adding the
-- column with a DEFAULT (Postgres 11+ applies it without a table rewrite), and the DEFAULT
-- is then dropped so that callers are forced to supply a lane explicitly.

-- Must exist before the FKs below can reference it.
CREATE TABLE IF NOT EXISTS ProjectLane (
    id UUID PRIMARY KEY DEFAULT uuidv7(),
    name VARCHAR(1000)
);

-- Seed lane for pre-lane data and for callers that have no lane to attribute. The id is
-- fixed so every installation agrees on the same default lane.
INSERT INTO ProjectLane (id, name)
VALUES ('019b76da-a800-7ab1-800d-efaddedead01', 'default')
ON CONFLICT (id) DO NOTHING;

ALTER TABLE LogEvent
    ADD COLUMN project_lane UUID NOT NULL DEFAULT '019b76da-a800-7ab1-800d-efaddedead01'
        REFERENCES ProjectLane(id);
ALTER TABLE LogEvent ALTER COLUMN project_lane DROP DEFAULT;

ALTER TABLE CachedAgentResponse
    ADD COLUMN project_lane UUID NOT NULL DEFAULT '019b76da-a800-7ab1-800d-efaddedead01'
        REFERENCES ProjectLane(id);
ALTER TABLE CachedAgentResponse ALTER COLUMN project_lane DROP DEFAULT;

ALTER TABLE UserInputEmbedding
    ADD COLUMN project_lane UUID NOT NULL DEFAULT '019b76da-a800-7ab1-800d-efaddedead01'
        REFERENCES ProjectLane(id);
ALTER TABLE UserInputEmbedding ALTER COLUMN project_lane DROP DEFAULT;