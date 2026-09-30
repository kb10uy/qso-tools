#import "@preview/oxifmt:1.0.0": strfmt

#set page(
  width: 140mm,
  height: 90mm,
  margin: 0mm,
)
#set text(
  font: "Monaspace Krypton",
  weight: "medium",
  // slashed-zero: true,
  features: ("cv01": 2),
)

#let qsl_entries = ()
#if "data_json" in sys.inputs {
  qsl_entries = json(sys.inputs.data_json)
}

#for entry in qsl_entries [
  #let station_call = entry.station_call
  #let bureau_call = entry.bureau_call
  #let call = entry.call
  #let date = entry.date
  #let time = entry.time
  #let timezone = entry.timezone
  #let report = entry.report
  #let freq_str = entry.freq
  #let mode = entry.mode
  #let received = entry.received
  #let rig = entry.rig
  #let power = entry.power
  #let antenna = entry.antenna
  #let operator = entry.operator
  #let operator_call = entry.operator_call
  #let location = entry.location

  #box(width: 100%, height: 100%)[
    // Bureau call
    #place(top + left, dx: 10mm, dy: 79mm)[
      #set text(size: 9mm)
      #let frame_x = 1.5mm
      #let frame_y = 1.5mm
      #rotate(-90deg, origin: top + left)[
        #block(width: 70mm, height: 9mm)[
          #for (i, c) in bureau_call.clusters().enumerate() [
            #place(top + left, dx: frame_x + i * 9mm, dy: frame_y)[#c]
          ]
        ]
      ]
    ]

    // Call
    #place(top + left, dx: 43mm, dy: 9mm)[
      #text(size: 7mm, call)
    ]

    // Date
    #place(top + left, dx: 26.5mm, dy: 25mm)[
      #set text(size: 6mm)
      #box(width: 44mm, height: 19mm, clip: true, align(center + horizon, date))
    ]

    // Time
    #place(top + left, dx: 70.5mm, dy: 25mm)[
      // JST/UTC mark
      #if timezone == "JST" {
        place(dx: 3mm, dy: 1.5mm, rect(fill: black, width: 2.5mm, height: 2.5mm))
      } else if timezone == "UTC" {
        place(dx: 3mm, dy: 5mm, rect(fill: black, width: 2.5mm, height: 2.5mm))
      }

      // HH:MM
      #set text(size: 4mm)
      #place(top + left, dy: 8mm, box(width: 15mm, height: 11mm, clip: true, align(center + horizon, time.slice(0, 5))))
    ]

    // Report
    #place(top + left, dx: 85mm, dy: 25mm)[
      #set text(size: 6mm)
      #box(width: 14mm, height: 19mm, clip: true, align(center + horizon, report))
    ]

    // MHz
    #place(top + left, dx: 100mm, dy: 25mm)[
      #let freq_mhz = float(freq_str)
      #let integer = calc.trunc(freq_mhz)
      #let fractional = calc.fract(freq_mhz)

      #box(width: 14mm, height: 19mm, clip: true, align(center + horizon, box(width: auto, align(top + left, stack(
        spacing: 1mm,
        text(size: 6mm, str(integer)),
        text(size: 4mm, strfmt("{:0.3}", fractional).slice(1)),
      )))))
    ]

    // Mode
    #place(top + left, dx: 115mm, dy: 25mm)[
      #set text(size: 6mm)
      #box(width: 14mm, height: 19mm, clip: true, align(center + horizon, mode))
    ]

    // Pse/QSL/Tnx
    #if received {
      place(dx: 65.5mm, dy: 46.5mm, rect(fill: black, width: 3mm, height: 3mm))
    } else {
      place(dx: 40.7mm, dy: 46.5mm, rect(fill: black, width: 3mm, height: 3mm))
    }

    // Rig, Power
    #place(top + left, dx: 40mm, dy: 52mm)[
      #text(size: 4mm, rig)
    ]
    #place(top + left, dx: 105mm, dy: 52mm)[
      #text(size: 4mm, str(int(float(power))))
    ]
    // Antenna
    #place(top + left, dx: 40mm, dy: 56.5mm)[
      #text(size: 4mm, antenna)
    ]
    // Operator
    #place(top + left, dx: 40mm, dy: 70mm)[
      #text(size: 4mm, if operator != "" { operator } else { operator_call })
    ]
    // Location
    #place(top + left, dx: 40mm, dy: 75mm)[
      #text(size: 4mm, location)
    ]
  ]
]
