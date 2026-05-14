# Build a Slack Block Kit payload from review-request rows.
#
# Input (stdin): JSON array of rows produced by notify_pending_reviews.sh, one entry per
# (reviewer, PR) pair, with shape:
#   { reviewer, title, url, number, waiting_s, additions, deletions }
#
# Args:
#   --arg repo  "owner/name" — used to render repo links in the header context.
#
# Output: a single JSON object suitable for POSTing to a Slack webhook.

def humanize:
  if . == null then "?"
  elif . < 60 then "\(.)s"
  elif . < 3600 then "\(. / 60 | floor)m"
  elif . < 86400 then "\(. / 3600 | floor)h"
  else "\(. / 86400 | floor)d"
  end;

# Age dot: green < 2d, yellow 2-4d, red > 4d.
def age_name:
  if . == null then "grey_question"
  elif . < 172800 then "large_green_circle"
  elif . < 345600 then "large_yellow_circle"
  else "red_circle"
  end;

def plural($n; $word):
  "\($n) \($word)\(if $n == 1 then "" else "s" end)";

def rpad($s; $n):
  ($n - ($s | length)) as $p
  | if $p > 0 then $s + (" " * $p) else $s end;

def header_block:
  {type: "header", text: {type: "plain_text", text: "Pending Review Requests", emoji: true}};

if length == 0 then
  {
    blocks: [
      header_block,
      {type: "section", text: {type: "mrkdwn",
        text: ":tada: No pending review requests in *<https://github.com/\($repo)|\($repo)>*."}}
    ]
  }
else
  group_by(.reviewer) as $groups
  | {
      blocks: (
        [
          header_block,
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
