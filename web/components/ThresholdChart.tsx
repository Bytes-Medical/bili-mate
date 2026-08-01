"use client";

// Threshold chart (spec 06): age against µmol/L with server-provided display
// curves. Lines are distinguished by dash pattern and measurements by marker
// fill — never by colour. Points are plotted locally but never compared
// locally (WEB-014); each has a text alternative with the server
// relationship.

import type { EvaluationRequest, ThresholdAssessment, ThresholdCurve } from "@/lib/api/client";
import { formatElapsed } from "@/lib/time";

const WIDTH = 840;
const HEIGHT = 480;
const MARGIN = { top: 24, right: 24, bottom: 56, left: 64 };
const MAX_AGE = 20160;
const MAX_VALUE = 550;

function x(ageMinutes: number): number {
  return MARGIN.left + (ageMinutes / MAX_AGE) * (WIDTH - MARGIN.left - MARGIN.right);
}

function y(value: number): number {
  return HEIGHT - MARGIN.bottom - (value / MAX_VALUE) * (HEIGHT - MARGIN.top - MARGIN.bottom);
}

function relationText(relation: string | undefined): string {
  switch (relation) {
    case "below":
      return "below";
    case "at":
      return "exactly at";
    case "above":
      return "above";
    default:
      return "not compared to";
  }
}

export default function ThresholdChart({
  curve,
  measurements,
  assessments,
}: {
  curve: ThresholdCurve;
  measurements: EvaluationRequest["measurements"];
  assessments: ThresholdAssessment[];
}) {
  const photoPath = curve.points
    .map((p) => `${x(p.age_minutes)},${y(p.phototherapy_threshold_umol_l)}`)
    .join(" ");
  const exchangePath = curve.points
    .map((p) => `${x(p.age_minutes)},${y(p.exchange_threshold_umol_l)}`)
    .join(" ");

  const dayTicks = Array.from({ length: 15 }, (_, i) => i * 1440);
  const valueTicks = Array.from({ length: 12 }, (_, i) => i * 50);

  return (
    <figure className="chart-figure" data-testid="threshold-chart">
      <figcaption>
        Treatment thresholds for {curve.gestational_age_completed_weeks} completed weeks&rsquo;
        gestation, birth through 14 days — rule pack{" "}
        <span className="mono">{curve.rule_pack.id}</span>, values in µmol/L (display only).
      </figcaption>
      <svg
        viewBox={`0 0 ${WIDTH} ${HEIGHT}`}
        role="img"
        aria-label={`Chart of phototherapy and exchange transfusion thresholds for ${curve.gestational_age_completed_weeks} weeks gestation, with the entered bilirubin results. A table with identical values follows.`}
      >
        {/* Chart paper */}
        {dayTicks.map((tick) => (
          <line
            key={`x${tick}`}
            x1={x(tick)}
            y1={y(0)}
            x2={x(tick)}
            y2={MARGIN.top}
            stroke="#d9d9d9"
            strokeWidth={tick % 2880 === 0 ? 1 : 0.5}
          />
        ))}
        {valueTicks.map((tick) => (
          <line
            key={`y${tick}`}
            x1={MARGIN.left}
            y1={y(tick)}
            x2={WIDTH - MARGIN.right}
            y2={y(tick)}
            stroke="#d9d9d9"
            strokeWidth={tick % 100 === 0 ? 1 : 0.5}
          />
        ))}
        {/* Axes */}
        <line x1={MARGIN.left} y1={y(0)} x2={WIDTH - MARGIN.right} y2={y(0)} stroke="#000" strokeWidth={1.5} />
        <line x1={MARGIN.left} y1={y(0)} x2={MARGIN.left} y2={MARGIN.top} stroke="#000" strokeWidth={1.5} />
        {dayTicks
          .filter((tick) => tick % 2880 === 0)
          .map((tick) => (
            <text key={`xl${tick}`} x={x(tick)} y={HEIGHT - MARGIN.bottom + 20} fontSize={13} textAnchor="middle" fill="#4d4d4d">
              {tick / 1440}
            </text>
          ))}
        <text x={(MARGIN.left + WIDTH - MARGIN.right) / 2} y={HEIGHT - 12} fontSize={13} textAnchor="middle" fill="#1a1a1a">
          Age (days from birth)
        </text>
        {valueTicks
          .filter((tick) => tick % 100 === 0)
          .map((tick) => (
            <text key={`yl${tick}`} x={MARGIN.left - 10} y={y(tick) + 4} fontSize={13} textAnchor="end" fill="#4d4d4d">
              {tick}
            </text>
          ))}
        <text
          x={16}
          y={(MARGIN.top + HEIGHT - MARGIN.bottom) / 2}
          fontSize={13}
          textAnchor="middle"
          fill="#1a1a1a"
          transform={`rotate(-90 16 ${(MARGIN.top + HEIGHT - MARGIN.bottom) / 2})`}
        >
          Total bilirubin (µmol/L)
        </text>
        {/* Threshold lines: phototherapy solid, exchange dashed */}
        <polyline points={photoPath} fill="none" stroke="#000" strokeWidth={2.5} data-line="phototherapy" />
        <polyline
          points={exchangePath}
          fill="none"
          stroke="#000"
          strokeWidth={2.5}
          strokeDasharray="10 6"
          data-line="exchange"
        />
        {/* Measurements: serum filled, transcutaneous hollow */}
        {measurements.map((m) => {
          const assessment = assessments.find((a) => a.measurement_id === m.id);
          const description = `${m.method === "serum" ? "Serum" : "Transcutaneous"} result ${
            m.total_bilirubin_umol_l
          } µmol/L at ${formatElapsed(m.age_minutes)}: ${relationText(
            assessment?.phototherapy_relation,
          )} the phototherapy line and ${relationText(assessment?.exchange_relation)} the exchange line.`;
          return (
            <g key={m.id} role="img" aria-label={description} data-marker={m.method}>
              <title>{description}</title>
              <circle
                cx={x(m.age_minutes)}
                cy={y(m.total_bilirubin_umol_l)}
                r={7}
                fill={m.method === "serum" ? "#000" : "#fff"}
                stroke="#000"
                strokeWidth={2.5}
              />
            </g>
          );
        })}
      </svg>
      <ul className="chart-legend">
        <li>
          <svg className="swatch" width="34" height="8" aria-hidden="true">
            <line x1="0" y1="4" x2="34" y2="4" stroke="#000" strokeWidth="2.5" />
          </svg>
          Phototherapy threshold (solid line)
        </li>
        <li>
          <svg className="swatch" width="34" height="8" aria-hidden="true">
            <line x1="0" y1="4" x2="34" y2="4" stroke="#000" strokeWidth="2.5" strokeDasharray="10 6" />
          </svg>
          Exchange transfusion threshold (dashed line)
        </li>
        <li>
          <svg className="swatch" width="16" height="16" aria-hidden="true">
            <circle cx="8" cy="8" r="6" fill="#000" />
          </svg>
          Serum result (filled marker)
        </li>
        <li>
          <svg className="swatch" width="16" height="16" aria-hidden="true">
            <circle cx="8" cy="8" r="6" fill="#fff" stroke="#000" strokeWidth="2.5" />
          </svg>
          Transcutaneous result (hollow marker)
        </li>
      </ul>
    </figure>
  );
}
