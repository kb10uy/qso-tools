use std::collections::HashSet;

use adif_manip::document::Record as AdifRecord;
use compact_str::ToCompactString;
use tracing::warn;

use crate::{
    commands::loggen::{
        contest::{ContestSummary, QsoScore},
        log::{LoggedExchange, LoggedQso},
    },
    core::qso::{exchange::QsoExchanges, get_optional_field, record::QsoRecord},
};

/// Multiplier of a QSO with the key to count it only once.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Multiplier {
    pub kind: String,
    pub value: String,
    pub key: String,
}

/// Counts QSOs, points and multipliers in time order.
#[derive(Debug, Default)]
pub struct Tally {
    dupe_keys: HashSet<String>,
    multiplier_keys: HashSet<(String, String)>,
    summary: ContestSummary,
}

impl Tally {
    pub fn summary(&self) -> &ContestSummary {
        &self.summary
    }

    pub fn add(
        &mut self,
        qso: &QsoRecord,
        record: &AdifRecord,
        score: QsoScore,
        dupe_key: String,
        multipliers: Vec<Multiplier>,
    ) -> LoggedQso {
        let dupe = !self.dupe_keys.insert(dupe_key.clone());
        let band = self.summary.bands.entry(qso.band.to_string()).or_default();
        let mut new_multipliers = vec![];
        if dupe {
            warn!("duplicate QSO: {dupe_key}");
            self.summary.dupes += 1;
        } else {
            self.summary.qsos += 1;
            self.summary.points += score.point;
            band.qsos += 1;
            band.points += score.point;

            for multiplier in multipliers {
                if self
                    .multiplier_keys
                    .insert((multiplier.kind.clone(), multiplier.key))
                {
                    self.summary.multipliers += 1;
                    *self
                        .summary
                        .multipliers_by_kind
                        .entry(multiplier.kind)
                        .or_default() += 1;
                    band.multipliers += 1;
                    new_multipliers.push(multiplier.value);
                }
            }
        }

        let exchanges = QsoExchanges::new(record);
        let exchange_value = |value: Option<String>, adif: Option<&str>, name: &str| {
            let value = value.or(adif.map(str::to_string));
            if value.is_none() {
                warn!("{name} is missing");
            }
            value
        };

        LoggedQso {
            datetime: qso.datetime.into(),
            band: qso.band,
            frequency: qso.frequency,
            mode: qso.mode.clone(),
            submode: qso.submode.clone(),
            call: qso.call.clone(),
            station_callsign: get_optional_field(record, "STATION_CALLSIGN")
                .map(|c| c.to_compact_string()),
            sent: LoggedExchange {
                report: exchange_value(
                    score.tx_report,
                    exchanges.sent.report.as_deref(),
                    "sent report",
                ),
                number: exchange_value(
                    score.tx_number,
                    exchanges.sent.number.as_deref(),
                    "sent number",
                ),
            },
            received: LoggedExchange {
                report: exchange_value(
                    score.rx_report,
                    exchanges.received.report.as_deref(),
                    "received report",
                ),
                number: exchange_value(
                    score.rx_number,
                    exchanges.received.number.as_deref(),
                    "received number",
                ),
            },
            multipliers: new_multipliers,
            point: if dupe { 0 } else { score.point },
            dupe,
        }
    }
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;

    use adif_manip::{LengthMode, read_adi};
    use time::UtcOffset;

    use super::*;
    use crate::commands::loggen::contest::BandSummary;

    #[test]
    fn tallies_qsos() {
        let document = read_adi(
            "Test\n<EOH>
<QSO_DATE:8>20260606<TIME_ON:4>1200<BAND:3>40m<FREQ:5>7.010<MODE:2>CW<CALL:5>JA1RL<RST_SENT:3>599<STX_STRING:3>09M<RST_RCVD:3>599<SRX_STRING:5>1001H<EOR>
<QSO_DATE:8>20260606<TIME_ON:4>1201<BAND:3>40m<FREQ:5>7.010<MODE:2>CW<CALL:5>JA1RL<RST_SENT:3>599<STX_STRING:3>09M<RST_RCVD:3>599<SRX_STRING:5>1001H<EOR>
<QSO_DATE:8>20260606<TIME_ON:4>1202<BAND:3>40m<FREQ:5>7.010<MODE:2>CW<CALL:6>JA1ZZZ<RST_SENT:3>599<STX_STRING:3>09M<RST_RCVD:3>599<SRX_STRING:5>1001M<EOR>
<QSO_DATE:8>20260606<TIME_ON:4>1203<BAND:4>70cm<FREQ:7>433.000<MODE:2>FM<CALL:6>JA1YYY<RST_SENT:2>59<STX_STRING:3>09M<EOR>
",
            LengthMode::Bytes,
        )
        .unwrap();
        let multiplier = |kind: &str, value: &str, key: &str| Multiplier {
            kind: kind.to_string(),
            value: value.to_string(),
            key: key.to_string(),
        };
        let inputs = [
            ("40m/JA1RL", vec![multiplier("default", "1001", "40m/1001")]),
            ("40m/JA1RL", vec![multiplier("default", "1001", "40m/1001")]),
            (
                "40m/JA1ZZZ",
                vec![
                    multiplier("default", "1001", "40m/1001"),
                    multiplier("day", "2026-06-06", "2026-06-06"),
                ],
            ),
            (
                "70cm/JA1YYY",
                vec![multiplier("default", "1001", "70cm/1001")],
            ),
        ];

        let mut tally = Tally::default();
        let logged: Vec<_> = document
            .records()
            .iter()
            .zip(inputs)
            .map(|(record, (dupe_key, multipliers))| {
                let qso = QsoRecord::new(record, UtcOffset::UTC).unwrap();
                let score = QsoScore {
                    point: 1,
                    ..Default::default()
                };
                tally.add(&qso, record, score, dupe_key.to_string(), multipliers)
            })
            .collect();

        assert_eq!(
            tally.summary(),
            &ContestSummary {
                qsos: 3,
                dupes: 1,
                points: 3,
                multipliers: 3,
                multipliers_by_kind: BTreeMap::from([
                    ("day".to_string(), 1),
                    ("default".to_string(), 2),
                ]),
                bands: BTreeMap::from([
                    (
                        "40m".to_string(),
                        BandSummary {
                            qsos: 2,
                            points: 2,
                            multipliers: 2,
                        }
                    ),
                    (
                        "70cm".to_string(),
                        BandSummary {
                            qsos: 1,
                            points: 1,
                            multipliers: 1,
                        }
                    ),
                ]),
            }
        );
        let columns: Vec<_> = logged
            .iter()
            .map(|l| {
                (
                    l.received.number.as_deref(),
                    l.multipliers.clone(),
                    l.point,
                    l.dupe,
                )
            })
            .collect();
        assert_eq!(
            columns,
            [
                (Some("1001H"), vec!["1001".to_string()], 1, false),
                (Some("1001H"), vec![], 0, true),
                (Some("1001M"), vec!["2026-06-06".to_string()], 1, false),
                (None, vec!["1001".to_string()], 1, false),
            ]
        );
    }
}
