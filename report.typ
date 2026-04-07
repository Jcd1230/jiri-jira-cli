#set page(width: 14in, height: auto, margin: 1in)
#set text(font: ("Roboto", "Noto Sans", "Liberation Sans"), size: 10pt)

#import "@preview/one-liner:0.3.0": fit-to-width

#let jiri_data = json("jiri_data.json")

= ⚡ Capability Pipeline Report
#v(10pt)

#let raw_epics = jiri_data.issues.filter(i => i.fields.issuetype.name == "Epic")
#let tasks = jiri_data.issues.filter(i => i.fields.issuetype.name == "Task")

// Sort epics by our mocked Due Date
#let epics = raw_epics.sorted(key: e => e.fields.mockDueDate)

#let get_points(task) = {
  let pts = 1
  let labels = task.fields.labels
  if labels != none {
    for l in labels {
      if l == "story-points-1" { pts = 1 } else if l == "story-points-2" { pts = 2 } else if l == "story-points-3" {
        pts = 3
      } else if l == "story-points-5" { pts = 5 } else if l == "story-points-8" { pts = 8 }
    }
  }
  pts
}

#let format_date(d) = {
  let m = d.slice(5, 7)
  let day = d.slice(8, 10)
  if m.starts-with("0") { m = m.slice(1) }
  m + "/" + day
}

// Convert YYYY-MM-DD into a proxy integer sequence to allow math without complex DateTime objects
#let parse_date(date_str) = {
  let y = int(date_str.slice(0, 4))
  let m = int(date_str.slice(5, 7))
  let d = int(date_str.slice(8, 10))
  // Rough days approximation (accurate enough for continuous small spans)
  return (y - 2026) * 365 + m * 30 + d
}


// Calculate the global span to proportionally map the grid across all rows evenly
#let min_day = calc.min(..epics.map(e => parse_date(e.fields.mockStartDate)))
#let max_day = calc.max(..epics.map(e => parse_date(e.fields.mockDueDate)))
#let total_span = calc.max(max_day - min_day, 1)

#let month_names = ("Jan", "Feb", "Mar", "Apr", "May", "Jun", "Jul", "Aug", "Sep", "Oct", "Nov", "Dec")
#let markings = ()
#for y in range(2026, 2029) {
  for m in range(1, 13) {
    let day_val = (y - 2026) * 365 + m * 30 + 1
    if day_val >= min_day and day_val <= max_day {
      markings.push((name: month_names.at(m - 1), day: day_val))
    }
  }
}

#table(
  columns: (28%, 35%, 37%),
  stroke: none,
  align: horizon,

  table.header(
    text(weight: "bold")[Capability (Epic)],
    text(weight: "bold")[Execution Progress (Weighted by Points)],
    box(width: 100%, height: 2.2em)[
      #place(top + left, dy: 0.2em)[*Rough Timeline Focus*]

      // Draw an axis line at the bottom of the header cell
      #place(bottom, line(length: 100%, stroke: 0.5pt + gray))

      #for m in markings {
        let pct = ((m.day - min_day) / total_span) * 100%
        place(bottom + left, dx: pct)[
          // Drop a small tick mark extending up from the axis
          #line(length: 4pt, angle: 90deg, stroke: 1pt + black)
        ]
        place(bottom + left, dx: pct)[
          // Use an absolute negative em shift with a fixed box to reliably center over the tick
          #move(dx: -1.5em, dy: -0.6em)[
            #box(width: 3em, align(center)[#text(size: 8pt, weight: "bold", fill: black)[#m.name]])
          ]
        ]
      }
    ],
  ),
  table.hline(stroke: 1pt + black),

  ..epics
    .map(epic => {
      let children = tasks.filter(t => t.fields.parent != none and t.fields.parent.id == epic.id)

      // Group children
      let todo = children.filter(t => t.fields.status.name == "To Do")
      let in_prog = children.filter(t => t.fields.status.name == "In Progress" or t.fields.status.name == "In progress")
      let done = children.filter(t => t.fields.status.name == "Done")

      let total_pts = children.fold(0, (acc, t) => acc + get_points(t))
      let done_pts = done.fold(0, (acc, t) => acc + get_points(t))
      let in_prog_pts = in_prog.fold(0, (acc, t) => acc + get_points(t))
      let todo_pts = todo.fold(0, (acc, t) => acc + get_points(t))

      let progress_bar = if total_pts > 0 {
        let cols = ()
        let rects = ()

        // Done -> Green
        if done_pts > 0 {
          cols.push(done_pts * 1fr)
          rects.push(rect(fill: green.lighten(20%), width: 100%, height: 2.0em, stroke: 1pt + white)[
            #box(width: 100%, height: 100%, clip: true)[
              #align(center + horizon)[#fit-to-width(max-text-size: 9pt, min-text-size: 4pt)[#text(
                weight: "bold",
                fill: black.lighten(20%),
              )[DONE]]]
            ]
          ])
        }
        // In Prog -> Blue
        if in_prog_pts > 0 {
          cols.push(in_prog_pts * 1fr)
          rects.push(rect(fill: rgb("#3b82f6").lighten(30%), width: 100%, height: 2.0em, stroke: 1pt + white)[
            #box(width: 100%, height: 100%, clip: true)[
              #align(center + horizon)[#fit-to-width(max-text-size: 9pt, min-text-size: 4pt)[#text(
                weight: "bold",
                fill: black.lighten(20%),
              )[IN~PROG]]]
            ]
          ])
        }
        // To Do -> Gray
        if todo_pts > 0 {
          cols.push(todo_pts * 1fr)
          rects.push(rect(fill: gray.lighten(60%), width: 100%, height: 2.0em, stroke: 1pt + white)[
            #box(width: 100%, height: 100%, clip: true)[
              #align(center + horizon)[#fit-to-width(max-text-size: 9pt, min-text-size: 4pt)[#text(
                weight: "bold",
                fill: black.lighten(20%),
              )[TODO]]]
            ]
          ])
        }

        grid(columns: cols, gutter: 0pt, ..rects)
      } else {
        [ _No scoped tasks_ ]
      }

      // Epic info block
      let epic_header = pad(y: 5pt)[
        #text(weight: "bold", size: 11pt)[#epic.key: ]
        #epic.fields.summary \
        #v(2pt)
      ]

      // Gantt Timeline logic
      let e_start = parse_date(epic.fields.mockStartDate)
      let e_end = calc.max(parse_date(epic.fields.mockDueDate), e_start + 1)

      let pre_w = e_start - min_day
      let dur_w = e_end - e_start
      let post_w = max_day - e_end

      // Safety check fallback
      let pre_w_fr = calc.max(pre_w, 0) * 1fr
      let dur_w_fr = calc.max(dur_w, 1) * 1fr
      let post_w_fr = calc.max(post_w, 0) * 1fr

      let date_meta = box(width: 100%)[
        #for m in markings {
          let pct = ((m.day - min_day) / total_span) * 100%
          place(top + left, dx: pct)[
            #line(stroke: (dash: "dotted", paint: gray.lighten(20%)), length: 2.5em, angle: 90deg)
          ]
        }
        #pad(y: 5pt)[
          #grid(
            columns: (pre_w_fr, dur_w_fr, post_w_fr),
            align: horizon,
            [],
            rect(fill: purple.lighten(40%), width: 100%, height: 1.1em, radius: 2pt)[
              #place(left + horizon, dx: 3pt)[#text(size: 7pt, weight: "bold", fill: black.lighten(20%))[#format_date(
                epic.fields.mockStartDate,
              )]]
              #place(right + horizon, dx: -3pt)[#text(size: 7pt, weight: "bold", fill: black.lighten(20%))[#format_date(
                epic.fields.mockDueDate,
              )]]
            ],
            [],
          )
        ]
      ]

      // Return flat array for exactly 3 columns + row divider!
      (
        epic_header,
        progress_bar,
        date_meta,
        table.hline(stroke: 0.5pt + luma(200)),
      )
    })
    .flatten(),
)
