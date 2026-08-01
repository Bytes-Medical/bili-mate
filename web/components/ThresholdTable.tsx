"use client";

// Tabular equivalent of the chart, rendering the identical server-provided
// display points (WEB-015).

import type { ThresholdCurve } from "@/lib/api/client";

export default function ThresholdTable({ curve }: { curve: ThresholdCurve }) {
  return (
    <div
      className="table-scroll"
      style={{ maxHeight: "24rem", overflowY: "auto" }}
      tabIndex={0}
      role="region"
      aria-label="Threshold values table (scrollable)"
    >
      <table className="data-table" data-testid="threshold-table">
        <caption>
          Threshold values for {curve.gestational_age_completed_weeks} completed weeks (identical to
          the chart; display values in µmol/L)
        </caption>
        <thead>
          <tr>
            <th scope="col">Age</th>
            <th scope="col" className="numeric">
              Age (minutes)
            </th>
            <th scope="col" className="numeric">
              Phototherapy
            </th>
            <th scope="col" className="numeric">
              Exchange transfusion
            </th>
          </tr>
        </thead>
        <tbody>
          {curve.points.map((point) => (
            <tr key={point.age_minutes}>
              <td>
                {Math.floor(point.age_minutes / 1440)} d {Math.floor((point.age_minutes % 1440) / 60)} h
              </td>
              <td className="numeric">{point.age_minutes}</td>
              <td className="numeric">{point.phototherapy_threshold_umol_l.toFixed(1)}</td>
              <td className="numeric">{point.exchange_threshold_umol_l.toFixed(1)}</td>
            </tr>
          ))}
        </tbody>
      </table>
    </div>
  );
}
