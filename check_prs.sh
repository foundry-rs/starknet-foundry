#!/bin/bash

# 1. Set your repository here
REPO_OWNER="foundry-rs"
REPO_NAME="starknet-foundry"

# Slack integration: when SLACK_WEBHOOK_URL is set, POST the message instead
# of printing it. SLACK_PAYLOAD_FIELD is the JSON field name expected by the
# webhook ("text" for incoming webhooks; for Workflow Builder set this to the
# variable name your trigger defines, e.g. "message").
SLACK_WEBHOOK_URL="${SLACK_WEBHOOK_URL:-}"
SLACK_PAYLOAD_FIELD="${SLACK_PAYLOAD_FIELD:-text}"

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

# Emit one TSV row per (reviewer, PR) pair: reviewer, title, "<url|#N>", seconds.
# Skips the `starknet-foundry` team. Sorted by reviewer asc, then waiting time desc.
ROWS=$(gh api graphql -f query="$QUERY" -F owner="$REPO_OWNER" -F name="$REPO_NAME" | jq -r '
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
        link: "<\($pr.url)|#\($pr.number)>",
        waiting_s: (if $ts then ((now - $ts) | floor) else null end)
      }
  ]
  | sort_by(.reviewer, -(.waiting_s // -1))
  | .[]
  | [.reviewer, .title, .link, (.waiting_s // "" | tostring)]
  | @tsv
')

humanize() {
    local s=$1
    if [ -z "$s" ]; then echo "?"; return; fi
    if [ "$s" -lt 60 ]; then echo "${s}s"
    elif [ "$s" -lt 3600 ]; then echo "$((s/60))m"
    elif [ "$s" -lt 86400 ]; then echo "$((s/3600))h"
    else echo "$((s/86400))d"
    fi
}

# 4. Build the Slack-formatted message in $MSG.
if [ -z "$ROWS" ]; then
    MSG="No pending review requests. :tada:"
else
    MSG=$(
        current_reviewer=""
        while IFS=$'\t' read -r reviewer title link waiting; do
            [ -z "$reviewer" ] && continue
            if [ "$reviewer" != "$current_reviewer" ]; then
                [ -n "$current_reviewer" ] && echo ""
                echo "*$reviewer*"
                current_reviewer="$reviewer"
            fi
            echo "• \`$(humanize "$waiting")\` $link $title"
        done <<< "$ROWS"
    )
fi

# 5. Send to Slack if a webhook is configured; otherwise print to stdout.
if [ -n "$SLACK_WEBHOOK_URL" ]; then
    PAYLOAD=$(jq -n --arg msg "$MSG" --arg field "$SLACK_PAYLOAD_FIELD" \
        '{($field): $msg}')
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
    printf '%s\n' "$MSG"
    echo "" >&2
    echo "✅ Extraction complete. (set SLACK_WEBHOOK_URL to post to Slack)" >&2
fi
