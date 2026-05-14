#!/bin/bash

# 1. Set your repository here
REPO_OWNER="foundry-rs"
REPO_NAME="starknet-foundry"

# Slack integration: when SLACK_WEBHOOK_URL is set, POST the message as a
# Block Kit payload (header + context + one divider/section group per reviewer).
SLACK_WEBHOOK_URL="${SLACK_WEBHOOK_URL:-}"

# 2. Check for dependencies
REQUIRED=(gh jq)
[ -n "$SLACK_WEBHOOK_URL" ] && REQUIRED+=(curl)
for cmd in "${REQUIRED[@]}"; do
    if ! command -v "$cmd" &> /dev/null; then
        echo "❌ Error: $cmd not found. Please install it." >&2
        exit 1
    fi
done

echo "Fetching pending review requests from: $REPO_OWNER/$REPO_NAME..." >&2
echo "" >&2

# 3. Pull current pending reviewers + ReviewRequestedEvent timestamps via GraphQL,
#    so we can compute how long each reviewer has been sitting on each request.
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

# Emit a JSON array, one object per (reviewer, PR) pair. Skips the
# `starknet-foundry` team. Sorted by reviewer asc, then waiting time desc.
ROWS_JSON=$(gh api graphql -f query="$QUERY" -F owner="$REPO_OWNER" -F name="$REPO_NAME" | jq '
  [
    .data.repository.pullRequests.nodes[]
    | . as $pr
    | ($pr.reviewRequests.nodes
        | map(.requestedReviewer | (.login // .name))
        | map(select(. != null and . != "starknet-foundry"))) as $pending
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
  | sort_by(.reviewer, -(.waiting_s // -1))
')

# 4. Build the Block Kit payload directly from the JSON rows.
PAYLOAD=$(printf '%s' "$ROWS_JSON" | jq --arg repo "$REPO_OWNER/$REPO_NAME" '
  def humanize:
    if . == null then "?"
    elif . < 60 then "\(.)s"
    elif . < 3600 then "\(. / 60 | floor)m"
    elif . < 86400 then "\(. / 3600 | floor)h"
    else "\(. / 86400 | floor)d"
    end;

  # Age dot: green < 1d, yellow 1-3d, red > 3d.
  def age_name:
    if . == null then "grey_question"
    elif . < 86400 then "large_green_circle"
    elif . < 259200 then "large_yellow_circle"
    else "red_circle"
    end;

  def plural($n; $word):
    "\($n) \($word)\(if $n == 1 then "" else "s" end)";

  def rpad($s; $n):
    ($n - ($s | length)) as $p
    | if $p > 0 then $s + (" " * $p) else $s end;

  if length == 0 then
    {
      blocks: [
        {type: "header",  text: {type: "plain_text", text: "Pending Review Requests", emoji: true}},
        {type: "section", text: {type: "mrkdwn", text: ":tada: No pending review requests in *<https://github.com/\($repo)|\($repo)>*."}}
      ]
    }
  else
    group_by(.reviewer) as $groups
    | {
        blocks: (
          [
            {type: "header",  text: {type: "plain_text", text: "Pending Review Requests", emoji: true}},
            {type: "context", elements: [
              {type: "mrkdwn", text: "*<https://github.com/\($repo)|\($repo)>* — \(plural(length; "pending review")) across \(plural($groups | length; "reviewer"))"}
            ]}
          ]
          + ($groups | map(
              . as $group
              | ($group | map(.waiting_s | humanize | length) | max) as $wmax
              | ($group | map("#\(.number)" | length) | max) as $nmax
              | [
                  {type: "divider"},
                  {type: "section", text: {type: "mrkdwn",
                    text: "*\(.[0].reviewer)*  ·  \(plural(length; "PR"))"
                  }},
                  {
                    type: "rich_text",
                    elements: ($group | map(
                      (.waiting_s | humanize) as $w
                      | "+\(.additions)/-\(.deletions)" as $sz
                      | "#\(.number)" as $num
                      | {
                          type: "rich_text_section",
                          elements: [
                            {type: "emoji", name: (.waiting_s | age_name)},
                            {type: "text",  text: " "},
                            {type: "text",  text: rpad($w; $wmax), style: {bold: true, code: true}},
                            {type: "text",  text: "  "},
                            {type: "link",  url: .url, text: rpad($num; $nmax)},
                            {type: "text",  text: "  \(.title) (\($sz))\n"}
                          ]
                        }
                    ))
                  }
                ]
            ) | add)
        )
      }
  end
')

# 5. Send to Slack if a webhook is configured; otherwise print the payload.
if [ -n "$SLACK_WEBHOOK_URL" ]; then
    RESP_FILE=$(mktemp)
    HTTP_CODE=$(curl -sS -o "$RESP_FILE" -w "%{http_code}" \
        -X POST -H 'Content-Type: application/json' \
        --data "$PAYLOAD" "$SLACK_WEBHOOK_URL")
    BODY=$(cat "$RESP_FILE"); rm -f "$RESP_FILE"
    if [ "$HTTP_CODE" = "200" ]; then
        echo "✅ Sent to Slack (HTTP $HTTP_CODE)." >&2
    else
        echo "❌ Slack webhook failed (HTTP $HTTP_CODE): $BODY" >&2
        exit 1
    fi
else
    printf '%s\n' "$PAYLOAD"
    echo "" >&2
    echo "✅ Block Kit payload built. (set SLACK_WEBHOOK_URL to post to Slack)" >&2
fi
