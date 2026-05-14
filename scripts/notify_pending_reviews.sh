#!/bin/bash
#
# Reports pending PR review requests for foundry-rs/starknet-foundry.
#
# Queries the GitHub GraphQL API for open PRs, groups outstanding review
# requests by reviewer, computes how long each reviewer has been waiting,
# and builds a Slack Block Kit message.
#
# If SLACK_WEBHOOK_URL is set, the message is posted on the channel.
# Otherwise the JSON payload is printed to stdout (useful for previewing
# in Slack's Block Kit Builder: https://app.slack.com/block-kit-builder).
#
# Requires: gh (authenticated), jq, and curl (only when posting).
# Block Kit construction lives in notify_pending_reviews_payload.jq.

set -euo pipefail

REPO="foundry-rs/starknet-foundry"
IGNORE_TEAM="starknet-foundry"
SLACK_WEBHOOK_URL="${SLACK_WEBHOOK_URL:-}"

# Dependency check.
REQUIRED=(gh jq)
if [ -n "$SLACK_WEBHOOK_URL" ]; then
    REQUIRED+=(curl)
fi
for cmd in "${REQUIRED[@]}"; do
    if ! command -v "$cmd" &> /dev/null; then
        echo "❌ Error: $cmd not found. Please install it." >&2
        exit 1
    fi
done

echo "Fetching pending review requests from: $REPO..." >&2

# Pull current pending reviewers + ReviewRequestedEvent timestamps via GraphQL.
# shellcheck disable=SC2016  # $owner/$name are GraphQL variables, not shell.
QUERY='
query($owner: String!, $name: String!) {
  repository(owner: $owner, name: $name) {
    pullRequests(first: 50, states: OPEN, orderBy: {field: UPDATED_AT, direction: DESC}) {
      nodes {
        title
        url
        number
        additions
        deletions
        reviewRequests(first: 20) {
          nodes {
            requestedReviewer {
              __typename
              ... on User { login }
              ... on Team { name }
            }
          }
        }
        timelineItems(itemTypes: [REVIEW_REQUESTED_EVENT], last: 100) {
          nodes {
            ... on ReviewRequestedEvent {
              createdAt
              requestedReviewer {
                __typename
                ... on User { login }
                ... on Team { name }
              }
            }
          }
        }
      }
    }
  }
}
'

# Emit a JSON array, one object per (reviewer, PR) pair.
# Sorted by reviewer asc, then waiting time desc.
ROWS_JSON=$(gh api graphql -f query="$QUERY" \
        -F owner="${REPO%/*}" -F name="${REPO#*/}" \
    | jq --arg ignore_team "$IGNORE_TEAM" '
      [
        .data.repository.pullRequests.nodes[]
        | . as $pr
        | ($pr.reviewRequests.nodes
            | map(.requestedReviewer | (.login // .name))
            | map(select(. != null and . != $ignore_team))) as $pending
        | $pending[]
        | . as $reviewer
        | (
            $pr.timelineItems.nodes
            | map(select(.requestedReviewer != null
                and (.requestedReviewer.login // .requestedReviewer.name) == $reviewer))
            | map(.createdAt | fromdateiso8601)
            | max
          ) as $ts
        | {
            reviewer: $reviewer,
            title: $pr.title,
            url: $pr.url,
            number: $pr.number,
            waiting_s: (if $ts then ((now - $ts) | floor) else null end),
            additions: $pr.additions,
            deletions: $pr.deletions
          }
      ]
      # safe check: -1 is a guard as jq cannot negate null, so replace nulls with -1.
      | sort_by(.reviewer, -(.waiting_s // -1))
    ')

# Build the Block Kit payload.
SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
PAYLOAD=$(jq -f "$SCRIPT_DIR/notify_pending_reviews_payload.jq" --arg repo "$REPO" <<<"$ROWS_JSON")

# Send to Slack if a webhook is configured, otherwise print the payload.
if [ -n "$SLACK_WEBHOOK_URL" ]; then
    RESP=$(curl -sS -w $'\n%{http_code}' -X POST \
        -H 'Content-Type: application/json' \
        --data "$PAYLOAD" "$SLACK_WEBHOOK_URL")
    HTTP_CODE=${RESP##*$'\n'}
    BODY=${RESP%$'\n'*}
    if [ "$HTTP_CODE" = "200" ]; then
        echo "✅ Sent to Slack (HTTP $HTTP_CODE)" >&2
    else
        echo "❌ Slack webhook failed (HTTP $HTTP_CODE): $BODY" >&2
        exit 1
    fi
else
    printf '%s\n' "$PAYLOAD"
    echo "✅ Block Kit payload built (set SLACK_WEBHOOK_URL to post to Slack)" >&2
fi
